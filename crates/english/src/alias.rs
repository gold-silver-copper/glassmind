//! Telling the system what the user will call a thing: "I'll call the
//! struct `Page` "the leaf" from now on", "i'll refer to `Handle` as the
//! runner", "`Saved` struct = "the memo"", and the system's answer,
//! "Noted: `the leaf` is `Page`.".
//!
//! The name the user gives is their own, not one the world knows, so it is
//! kept as typed: quoted, or "the" and words the system does not know.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Fact, Name, Ontology, Reply, Thing, Utterance};

use crate::lexicon::{ITEM, NAME, QUOTE, TEXT};
use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 6;

/// The words among the children, joined as typed.
fn words<O: Ontology>(c: &[Sem<O>]) -> String {
    c.iter()
        .filter_map(|v| match v {
            Sem::Quote(w) | Sem::Name(Name(w)) => Some(w.as_str()),
            _ => None,
        })
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The thing mentioned among the children, to be called the name among
/// them.
fn alias<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let (thing, kind) = c.iter().find_map(|v| match v {
        Sem::Mention { thing, kind } => Some((thing.clone(), kind.clone())),
        _ => None,
    })?;
    let alias = c.iter().find_map(|v| match v {
        Sem::Name(n) => Some(n.clone()),
        _ => None,
    })?;
    Some(Sem::Utterance(Utterance::Tell(Fact::Alias {
        thing,
        kind,
        alias,
    })))
}

/// The fact among the children.
fn fact<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter()
        .find(|v| matches!(v, Sem::Utterance(Utterance::Tell(_))))
        .cloned()
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let blank = || Sem::Quote(String::new());
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let name = |c: &[Sem<O>]| Some(Sem::Name(Name(words(c))));
    let mut rules = vec![
        // "call the struct `Page` "the leaf"", "calling `Gallery` the wall"
        rule(
            "en.fact.call",
            Cat::Fact,
            vec![C(Cat::Calling), C(Cat::Mention), C(Cat::Alias)],
            alias,
        ),
        // "refer to struct Store as "the vault""
        rule(
            "en.fact.refer",
            Cat::Fact,
            vec![C(Cat::Referring), C(Cat::Mention), W("as"), C(Cat::Alias)],
            alias,
        ),
        // "BlobObject = the blob"
        rule(
            "en.fact.equals",
            Cat::Fact,
            vec![C(Cat::Mention), W("="), C(Cat::Alias)],
            alias,
        ),
        // "I'm going to call ...", "gonna call ...", "I'll just call ..."
        rule(
            "en.fact.intent",
            Cat::Fact,
            vec![C(Cat::Intent), C(Cat::Fact)],
            fact,
        ),
        // "... from now on", "from here on, ..."
        rule(
            "en.fact.time_after",
            Cat::Fact,
            vec![C(Cat::Fact), C(Cat::Time)],
            fact,
        ),
        rule(
            "en.fact.time_before",
            Cat::Fact,
            vec![C(Cat::Time), C(Cat::Fact)],
            fact,
        ),
        rule(
            "en.fact.time_comma",
            Cat::Fact,
            vec![C(Cat::Time), W(","), C(Cat::Fact)],
            fact,
        ),
        // "for the rest of this chat": any words after it, which may well
        // be names.
        rule(
            "en.time.for_the_rest_of",
            Cat::Time,
            vec![W("for"), W("the"), W("rest"), W("of"), C(Cat::Words)],
            |_| Some(Sem::Quote(String::new())),
        ),
        rule(
            "en.utterance.fact",
            Cat::Utterance,
            vec![C(Cat::Fact)],
            fact,
        ),
        // ""the leaf"", "'the card'": any words, quoted.
        rule(
            "en.alias.quoted",
            Cat::Alias,
            vec![W("\""), C(Cat::Words), W("\"")],
            name,
        ),
        rule(
            "en.alias.single_quoted",
            Cat::Alias,
            vec![W("'"), C(Cat::Words), W("'")],
            name,
        ),
        // "the portrait", "the link spot": unquoted, words the system does
        // not know after "the", so the name ends where known words go on.
        rule(
            "en.alias.the",
            Cat::Alias,
            vec![W("the"), C(Cat::Unknown)],
            |c| Some(Sem::Name(Name(format!("the {}", words(c))))),
        ),
        // "just "the leaf"": only that name.
        rule(
            "en.alias.just",
            Cat::Alias,
            vec![W("just"), C(Cat::Alias)],
            |c| c.first().cloned(),
        ),
        rule("en.words.one", Cat::Words, vec![Class(TEXT)], name),
        rule(
            "en.words.more",
            Cat::Words,
            vec![C(Cat::Words), Class(TEXT)],
            name,
        ),
        rule("en.unknown.one", Cat::Unknown, vec![Class(NAME)], name),
        rule(
            "en.unknown.more",
            Cat::Unknown,
            vec![C(Cat::Unknown), Class(NAME)],
            name,
        ),
        // "Noted: `the leaf` is `Page`."
        Rule::new(
            RuleInfo::new("en.reply.noted", ROUND),
            Cat::Reply,
            vec![
                W("noted"),
                W(":"),
                Class(QUOTE),
                W("is"),
                Class(ITEM),
                W("."),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Noted(Fact::Alias {
                        thing: Thing::Item(item),
                        alias,
                        ..
                    })) => Some(vec![Sem::Quote(alias.0.clone()), Sem::Item(item.clone())]),
                    _ => None,
                },
            ),
        ),
    ];
    let words = |cat, table: &[(&'static str, &'static [&'static str])]| {
        table
            .iter()
            .map(move |(id, words)| word(RuleInfo::new(id, ROUND), cat, words, blank()).read_only())
            .collect::<Vec<_>>()
    };
    rules.extend(words(Cat::Calling, CALLING));
    rules.extend(words(Cat::Referring, REFERRING));
    rules.extend(words(Cat::Intent, INTENTS));
    rules.extend(words(Cat::Time, TIMES));
    rules
}

/// Verbs giving a thing a name of the user's own, before it and the name.
const CALLING: &[(&str, &[&str])] = &[
    ("en.calling.call", &["call"]),
    ("en.calling.calling", &["calling"]),
];

/// Verbs giving a thing a name of the user's own, before it and "as".
const REFERRING: &[(&str, &[&str])] = &[
    ("en.referring.refer_to", &["refer", "to"]),
    ("en.referring.referring_to", &["referring", "to"]),
];

/// Words saying the user means to do what follows: "I'm going to", "gonna".
const INTENTS: &[(&str, &[&str])] = &[
    ("en.intent.i_ll", &["i'll"]),
    ("en.intent.i_will", &["i", "will"]),
    ("en.intent.i_m", &["i'm"]),
    ("en.intent.going_to", &["going", "to"]),
    ("en.intent.gonna", &["gonna"]),
    ("en.intent.be", &["be"]),
    ("en.intent.just", &["just"]),
];

/// Words saying how long a name the user gives holds.
const TIMES: &[(&str, &[&str])] = &[
    ("en.time.from_now_on", &["from", "now", "on"]),
    ("en.time.from_now", &["from", "now"]),
    ("en.time.from_here_on", &["from", "here", "on"]),
    ("en.time.from_here", &["from", "here"]),
    ("en.time.going_forward", &["going", "forward"]),
    ("en.time.for_now", &["for", "now"]),
    ("en.time.for_short", &["for", "short"]),
];
