//! Telling the system what the user will call a thing: "I'll call the
//! struct `Page` "the leaf" from now on", "i'll refer to `Handle` as the
//! runner", "`Saved` struct = "the memo"", and the system's answer,
//! "Noted: `the leaf` is `Page`.".
//!
//! The name the user gives is their own, not one the world knows, so it is
//! kept as typed: quoted, or "the" and words the system does not know.

use glassmind_grammar::{Rule, Sym};
use glassmind_meaning::{Fact, Name, Ontology, Reply, Thing, Utterance};

use crate::asking::read_only_rule;
use crate::lexicon::{ITEM, QUOTE, TEXT, WORD};
use crate::locate::{fillers, nothing};
use crate::sem::{Cat, Sem};
use crate::spoken;

use Cat::*;
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

/// The thing mentioned last among the children, to be called the name
/// after it.
fn alias<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [.., Sem::Mention { thing, kind }, Sem::Name(alias)] => {
            Some(Sem::Utterance(Utterance::Tell(Fact::Alias {
                thing: thing.clone(),
                kind: kind.clone(),
                alias: alias.clone(),
            })))
        }
        _ => None,
    }
}

/// The fact among the children.
fn fact<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter()
        .find(|v| matches!(v, Sem::Utterance(Utterance::Tell(_))))
        .cloned()
}

/// The words among the children, as a name the user gave.
fn name<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    Some(Sem::Name(Name(words(c))))
}

/// The words among the children, as a name the user gave, standing for the
/// thing they gave it to.
fn named<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    Some(Sem::Thing(Thing::Named(Name(words(c)))))
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, round, lhs, rhs, build| read_only_rule(id, round, lhs, rhs, build);
    let mut rules = vec![
        // "call the struct `Page` "the leaf"", "calling `Gallery` the wall"
        rule(
            "en.fact.call",
            ROUND,
            Fact,
            vec![C(Calling), C(Mention), C(Alias)],
            alias,
        ),
        // "refer to struct Store as "the vault""
        rule(
            "en.fact.refer",
            ROUND,
            Fact,
            vec![C(Referring), C(Mention), W("as"), C(Alias)],
            alias,
        ),
        // "BlobObject = the blob"
        rule(
            "en.fact.equals",
            ROUND,
            Fact,
            vec![C(Mention), W("="), C(Alias)],
            alias,
        ),
        // "I'm going to call ...", "gonna call ...", "I'll just call ..."
        rule(
            "en.fact.intent",
            ROUND,
            Fact,
            vec![C(Intent), C(Fact)],
            fact,
        ),
        // "... from now on", "from here on, ..."
        rule(
            "en.fact.time_after",
            ROUND,
            Fact,
            vec![C(Fact), C(Time)],
            fact,
        ),
        rule(
            "en.fact.time_before",
            ROUND,
            Fact,
            vec![C(Time), C(Fact)],
            fact,
        ),
        // "for the rest of this chat": any words after it, which may well
        // be names.
        rule(
            "en.time.for_the_rest_of",
            ROUND,
            Time,
            vec![W("for the rest of"), C(Words)],
            nothing,
        ),
        rule("en.utterance.fact", ROUND, Utterance, vec![C(Fact)], fact),
    ];
    // ""the leaf"", "'the card'": any words, quoted.
    rules.extend(
        [("en.alias.quoted", "\""), ("en.alias.single_quoted", "'")]
            .map(|(id, quote)| rule(id, ROUND, Alias, vec![W(quote), C(Words), W(quote)], name)),
    );
    rules.extend([
        // "the portrait", "the link spot": unquoted, words the system does
        // not know after "the", so the name ends where known words go on.
        rule(
            "en.alias.the",
            ROUND,
            Alias,
            vec![W("the"), C(Unknown)],
            |c| Some(Sem::Name(Name(format!("the {}", words(c))))),
        ),
        // "what fields does the slot have?": a name the user gave, standing
        // for the thing they gave it to.
        rule("en.noun_phrase.alias", 7, NounPhrase, vec![C(Alias)], named),
        // "what fields does sun glow have?", "the history sheet": several
        // words the system does not know, a name the user gave, which may
        // have been given without "the".
        rule(
            "en.noun_phrase.unknown",
            7,
            NounPhrase,
            vec![C(Unknown), Class(WORD)],
            named,
        ),
        rule(
            "en.noun_phrase.the_unknown",
            7,
            NounPhrase,
            vec![W("the"), C(Unknown), Class(WORD)],
            named,
        ),
        // Any words, and words the system does not know, as typed.
        rule("en.words.one", ROUND, Words, vec![Class(TEXT)], name),
        rule(
            "en.words.more",
            ROUND,
            Words,
            vec![C(Words), Class(TEXT)],
            name,
        ),
        rule("en.unknown.one", ROUND, Unknown, vec![Class(WORD)], name),
        rule(
            "en.unknown.more",
            ROUND,
            Unknown,
            vec![C(Unknown), Class(WORD)],
            name,
        ),
        // "Noted: `the leaf` is `Page`."
        spoken(
            "en.reply.noted",
            ROUND,
            Reply,
            vec![W("noted :"), Class(QUOTE), W("is"), Class(ITEM), W(".")],
            |_| None,
            |v: &Sem<O>| match v {
                Sem::Reply(Reply::Noted(Fact::Alias {
                    thing: Thing::Item(item),
                    alias,
                    ..
                })) => Some(vec![Sem::Quote(alias.0.clone()), Sem::Item(item.clone())]),
                _ => None,
            },
        ),
    ]);
    rules.extend(fillers(ROUND, FILLERS));
    rules
}

/// Words that only shape the telling of a name, by category.
const FILLERS: &[(Cat, &[(&str, &str)])] = &[
    // Verbs giving a thing a name of the user's own, before it and the name.
    (
        Calling,
        &[
            ("en.calling.call", "call"),
            ("en.calling.calling", "calling"),
        ],
    ),
    // Verbs giving a thing a name of the user's own, before it and "as".
    (
        Referring,
        &[
            ("en.referring.refer_to", "refer to"),
            ("en.referring.referring_to", "referring to"),
        ],
    ),
    // Words saying the user means to do what follows: "I'm going to",
    // "gonna".
    (
        Intent,
        &[
            ("en.intent.i_ll", "i'll"),
            ("en.intent.i_will", "i will"),
            ("en.intent.i_m", "i'm"),
            ("en.intent.going_to", "going to"),
            ("en.intent.gonna", "gonna"),
            ("en.intent.be", "be"),
            ("en.intent.just", "just"),
        ],
    ),
    // Words saying how long a name the user gives holds.
    (
        Time,
        &[
            ("en.time.from_now_on", "from now on"),
            ("en.time.from_now", "from now"),
            ("en.time.from_here_on", "from here on"),
            ("en.time.from_here", "from here"),
            ("en.time.going_forward", "going forward"),
            ("en.time.for_now", "for now"),
            ("en.time.for_short", "for short"),
        ],
    ),
];
