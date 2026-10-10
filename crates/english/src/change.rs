//! Asking for a change: "rename `x` to `y`", "`x` -> `y`", and the system's
//! proposal, "I'd rename `x` to `y`.".

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Action, Ontology, Reply, Thing, Utterance};

use crate::lexicon::CODE;
use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 5;

/// Renaming the thing mentioned among the children, or else the thing
/// said before ("rename to `y`"), to the name among them.
fn rename<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let (thing, kind) = c
        .iter()
        .find_map(|v| match v {
            Sem::Mention { thing, kind } => Some((thing.clone(), kind.clone())),
            _ => None,
        })
        .unwrap_or((Thing::Previous, None));
    let name = c.iter().rev().find_map(|v| match v {
        Sem::Name(n) => Some(n.clone()),
        _ => None,
    })?;
    Some(Sem::Utterance(Utterance::Command(Action::Rename {
        thing,
        kind,
        name,
    })))
}

/// The mention, the wording and the new name a rename speaks.
fn unrename<O: Ontology>(v: &Sem<O>) -> Option<Vec<Sem<O>>> {
    match v {
        Sem::Utterance(Utterance::Command(Action::Rename { thing, kind, name })) => Some(vec![
            Sem::Mention {
                thing: thing.clone(),
                kind: kind.clone(),
            },
            Sem::Quote(String::new()),
            Sem::Name(name.clone()),
        ]),
        _ => None,
    }
}

/// The command among the children.
fn command<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter()
        .find(|v| matches!(v, Sem::Utterance(Utterance::Command(_))))
        .cloned()
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let blank = || Sem::Quote(String::new());
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let mut rules = vec![
        // "rename `x` to `y`": how the system says it, too.
        Rule::new(
            RuleInfo::new("en.command.rename", ROUND),
            Cat::Command,
            vec![W("rename"), C(Cat::Mention), C(Cat::Into), Class(CODE)],
            S::Both(rename, unrename),
        ),
        // "rename to `y`", after what is to be renamed was said.
        rule(
            "en.command.rename_to",
            Cat::Command,
            vec![W("rename"), C(Cat::Into), Class(CODE)],
            rename,
        ),
        // "rename it `submenu_modal`", "call it `vault_hand_placement`"
        rule(
            "en.command.naming",
            Cat::Command,
            vec![C(Cat::Naming), C(Cat::Mention), Class(CODE)],
            rename,
        ),
        // "`start_download` -> `spawn_fetch_jobs`", "`tx_view` renamed to
        // `extract_tx_view`", "`on_edited` should be `handle_edit_result`"
        rule(
            "en.command.becomes",
            Cat::Command,
            vec![C(Cat::Mention), C(Cat::Becomes), Class(CODE)],
            rename,
        ),
        // "rename `x` to `y` for me", "... Could you do that, please?"
        rule(
            "en.utterance.follow_up",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::FollowUp)],
            command,
        ),
        rule(
            "en.utterance.mark_follow_up",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Mark), C(Cat::FollowUp)],
            command,
        ),
        // "..., can you do it?", "... and update the callers too"
        rule(
            "en.utterance.joined_follow_up",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Join), C(Cat::FollowUp)],
            command,
        ),
        // A command, with or without its final punctuation.
        Rule::new(
            RuleInfo::new("en.utterance.command", ROUND),
            Cat::Utterance,
            vec![C(Cat::Command)],
            S::Both(command, |v| Some(vec![v.clone()])),
        ),
        rule(
            "en.utterance.command_mark",
            Cat::Utterance,
            vec![C(Cat::Command), W("?")],
            command,
        ),
        rule(
            "en.utterance.command_stop",
            Cat::Utterance,
            vec![C(Cat::Command), W(".")],
            command,
        ),
        // "rename `x` to `y`, please", "`x` -> `y` pls"
        rule(
            "en.command.interjection_after",
            Cat::Command,
            vec![C(Cat::Command), C(Cat::Interjection)],
            command,
        ),
        rule(
            "en.command.comma_interjection",
            Cat::Command,
            vec![C(Cat::Command), W(","), C(Cat::Interjection)],
            command,
        ),
        // "I'd rename `x` to `y`."
        Rule::new(
            RuleInfo::new("en.reply.proposal", ROUND),
            Cat::Reply,
            vec![W("I'd"), C(Cat::Command), W(".")],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Proposal { action, .. }) => {
                        Some(vec![Sem::Utterance(Utterance::Command(action.clone()))])
                    }
                    _ => None,
                },
            ),
        ),
        word(
            RuleInfo::new("en.into.to", ROUND),
            Cat::Into,
            &["to"],
            blank(),
        ),
    ];
    let words = |cat, table: &[(&'static str, &'static [&'static str])]| {
        table
            .iter()
            .map(move |(id, words)| word(RuleInfo::new(id, ROUND), cat, words, blank()).read_only())
            .collect::<Vec<_>>()
    };
    rules.extend(words(Cat::Into, INTO));
    rules.extend(words(Cat::Becomes, BECOMES));
    rules.extend(words(Cat::Naming, NAMING));
    rules.extend(words(Cat::FollowUp, FOLLOW_UPS));
    rules.extend(words(Cat::Mark, MARKS));
    rules.extend([
        rule(
            "en.becomes.into",
            Cat::Becomes,
            vec![W("renamed"), C(Cat::Into)],
            |c| c.first().cloned(),
        ),
        // "could you do that", "pls rename", "do that for me, please?"
        rule(
            "en.follow_up.prompt",
            Cat::FollowUp,
            vec![C(Cat::Prompt), C(Cat::FollowUp)],
            |c| c.last().cloned(),
        ),
        rule(
            "en.follow_up.interjection",
            Cat::FollowUp,
            vec![C(Cat::FollowUp), C(Cat::Interjection)],
            |c| c.first().cloned(),
        ),
        rule(
            "en.follow_up.comma_interjection",
            Cat::FollowUp,
            vec![C(Cat::FollowUp), W(","), C(Cat::Interjection)],
            |c| c.first().cloned(),
        ),
        // "...? `regrid` is kinda confusing for me": a remark, as a sentence
        // of its own, so none of its words belong to the request.
        rule(
            "en.follow_up.remark",
            Cat::FollowUp,
            vec![C(Cat::Mark), C(Cat::Remark)],
            |_| Some(Sem::Quote(String::new())),
        ),
        rule(
            "en.follow_up.mark",
            Cat::FollowUp,
            vec![C(Cat::FollowUp), C(Cat::Mark)],
            |c| c.first().cloned(),
        ),
    ]);
    rules
}

/// Words saying what a thing is to become, besides "to".
const INTO: &[(&str, &[&str])] = &[
    ("en.into.into", &["into"]),
    ("en.into.arrow", &["-", ">"]),
    ("en.into.unicode_arrow", &["→"]),
    // "rename `x` to something like `y`": the name the user offers.
    ("en.into.something_like", &["to", "something", "like"]),
];

/// Words saying a thing is to be called something else, after it.
const BECOMES: &[(&str, &[&str])] = &[
    ("en.becomes.arrow", &["-", ">"]),
    ("en.becomes.unicode_arrow", &["→"]),
    ("en.becomes.should_be", &["should", "be"]),
];

/// Verbs giving a thing a name, before it and the name.
const NAMING: &[(&str, &[&str])] = &[
    ("en.naming.rename", &["rename"]),
    ("en.naming.call", &["call"]),
    ("en.naming.make", &["make"]),
];

/// Words asking again, or in more detail, for a change just asked for.
const FOLLOW_UPS: &[(&str, &[&str])] = &[
    ("en.follow_up.do_that", &["do", "that"]),
    ("en.follow_up.do_it", &["do", "it"]),
    (
        "en.follow_up.take_care_of_that",
        &["take", "care", "of", "that"],
    ),
    ("en.follow_up.make_that_change", &["make", "that", "change"]),
    ("en.follow_up.do_that_rename", &["do", "that", "rename"]),
    ("en.follow_up.rename", &["rename"]),
    ("en.follow_up.rename_it", &["rename", "it"]),
    ("en.follow_up.rename_all_uses", &["rename", "all", "uses"]),
    ("en.follow_up.for_me", &["for", "me"]),
    ("en.follow_up.everywhere", &["everywhere"]),
    (
        "en.follow_up.everywhere_used",
        &["everywhere", "it's", "used"],
    ),
    (
        "en.follow_up.update_callers",
        &["update", "the", "callers", "too"],
    ),
    (
        "en.follow_up.update_all_callers",
        &["update", "all", "the", "callers", "too"],
    ),
];

/// Marks ending a sentence inside a turn.
const MARKS: &[(&str, &[&str])] = &[
    ("en.mark.question", &["?"]),
    ("en.mark.stop", &["."]),
    ("en.mark.bang", &["!"]),
];
