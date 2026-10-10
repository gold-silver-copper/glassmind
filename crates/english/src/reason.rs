//! Asking why: "why do you want to change that?", "reason for that
//! change?", "can you explain why you want to make that change?". The
//! question is about the action the system was last asked for, so it names
//! nothing. The system's answer: "Because you asked me to rename `x` to `y`."

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Reply, Utterance};

use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 6;

/// Words for the action the system took up.
const DEEDS: &[(&str, &[&str])] = &[
    ("en.deed.that", &["that"]),
    ("en.deed.it", &["it"]),
    ("en.deed.this", &["this"]),
    ("en.deed.that_change", &["that", "change"]),
    ("en.deed.this_change", &["this", "change"]),
    ("en.deed.the_change", &["the", "change"]),
    ("en.deed.the_thing", &["the", "thing"]),
    ("en.deed.that_rename", &["that", "rename"]),
    ("en.deed.the_rename", &["the", "rename"]),
];

/// Words saying the system put the action forward.
const PROPOSED: &[(&str, &[&str])] = &[
    ("en.proposed.you_proposed", &["you", "proposed"]),
    (
        "en.proposed.you_just_proposed",
        &["you", "just", "proposed"],
    ),
    ("en.proposed.you_suggested", &["you", "suggested"]),
    (
        "en.proposed.you_just_suggested",
        &["you", "just", "suggested"],
    ),
];

/// Words saying the system wants something, as a question asks it ("do
/// you want") or as a sloppy one does ("you want", "it wants"). Users speak
/// of the system as "you" or "it".
const WISHES: &[(&str, &[&str])] = &[
    ("en.wish.do_you_want", &["do", "you", "want"]),
    ("en.wish.you_want", &["you", "want"]),
    ("en.wish.would_you_want", &["would", "you", "want"]),
    ("en.wish.does_it_want", &["does", "it", "want"]),
    ("en.wish.it_wants", &["it", "wants"]),
    ("en.wish.would_it_want", &["would", "it", "want"]),
];

/// Verbs carrying out an action on what they are said of: "change that",
/// "make that change", "rename it", "do that".
const CARRY: &[(&str, &str)] = &[
    ("en.carrying.change", "change"),
    ("en.carrying.make", "make"),
    ("en.carrying.rename", "rename"),
    ("en.carrying.do", "do"),
];

/// Nouns for why something is done, with what they may open with.
const REASONS: &[(&str, &[&str])] = &[
    ("en.reason.reason", &["reason"]),
    ("en.reason.reasoning", &["reasoning"]),
    ("en.reason.the_reason", &["the", "reason"]),
    ("en.reason.the_reasoning", &["the", "reasoning"]),
    ("en.reason.your_reason", &["your", "reason"]),
    ("en.reason.your_reasoning", &["your", "reasoning"]),
];

/// Words softening a "why" question, besides the hedges of other questions.
const HEDGES: &[(&str, &[&str])] = &[("en.hedge.though", &["though"]), ("en.hedge.tho", &["tho"])];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let blank = || Sem::Quote(String::new());
    let why = || S::Const(Sem::Utterance(Utterance::AskReason));
    let rule = |id, lhs, rhs, sem| Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, sem).read_only();
    let first = || S::Build(|c: &[Sem<O>]| c.first().cloned());
    let mut rules = vec![
        // "why?", "why tho": why the system did what it just did.
        rule("en.why.why", Cat::Why, vec![W("why")], why()),
        // "why do you want to change that", "why it wants to make that change"
        rule(
            "en.why.want_to",
            Cat::Why,
            vec![W("why"), C(Cat::Wish), W("to"), C(Cat::Carrying)],
            why(),
        ),
        // "why do you want that", "why do you want that changed"
        rule(
            "en.why.want",
            Cat::Why,
            vec![W("why"), C(Cat::Wish), C(Cat::Deed)],
            why(),
        ),
        rule(
            "en.why.want_changed",
            Cat::Why,
            vec![W("why"), C(Cat::Wish), C(Cat::Deed), W("changed")],
            why(),
        ),
        // "why change that", "why that change"
        rule(
            "en.why.carrying",
            Cat::Why,
            vec![W("why"), C(Cat::Carrying)],
            why(),
        ),
        rule("en.why.deed", Cat::Why, vec![W("why"), C(Cat::Deed)], why()),
        // "reasoning behind that change", "what's the reason for this
        // change", "walk me through your reasoning"
        rule("en.why.reason", Cat::Why, vec![C(Cat::Reason)], why()),
        rule(
            "en.why.what_is_reason",
            Cat::Why,
            vec![C(Cat::WhatIs), C(Cat::Reason)],
            why(),
        ),
        // "can you explain why ...", "I'd like to know why ..."
        rule(
            "en.why.prompt",
            Cat::Why,
            vec![C(Cat::Prompt), C(Cat::Why)],
            S::Build(|c: &[Sem<O>]| c.last().cloned()),
        ),
        // "why do you want to change that, exactly", "why change it tho"
        rule(
            "en.why.hedge",
            Cat::Why,
            vec![C(Cat::Why), C(Cat::Hedge)],
            first(),
        ),
        rule(
            "en.why.comma_hedge",
            Cat::Why,
            vec![C(Cat::Why), W(","), C(Cat::Hedge)],
            first(),
        ),
        // "why do you want to change that, the thing you just proposed"
        rule(
            "en.why.deed_again",
            Cat::Why,
            vec![C(Cat::Why), W(","), C(Cat::Deed)],
            first(),
        ),
        // "the change you just proposed", "what you just proposed"
        rule(
            "en.deed.proposed",
            Cat::Deed,
            vec![C(Cat::Deed), C(Cat::Proposed)],
            first(),
        ),
        rule(
            "en.deed.what_proposed",
            Cat::Deed,
            vec![W("what"), C(Cat::Proposed)],
            S::Const(blank()),
        ),
        // "change it like that"
        rule(
            "en.carrying.like_that",
            Cat::Carrying,
            vec![C(Cat::Carrying), W("like"), W("that")],
            first(),
        ),
        // A "why" question, with or without its question mark.
        rule(
            "en.utterance.why_mark",
            Cat::Utterance,
            vec![C(Cat::Why), W("?")],
            first(),
        ),
        rule(
            "en.utterance.why",
            Cat::Utterance,
            vec![C(Cat::Why)],
            first(),
        ),
        // "Because you asked me to rename `x` to `y`."
        Rule::new(
            RuleInfo::new("en.reply.reason", ROUND),
            Cat::Reply,
            vec![
                W("because"),
                W("you"),
                W("asked"),
                W("me"),
                W("to"),
                C(Cat::Command),
                W("."),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Reason { action, .. }) => {
                        Some(vec![Sem::Utterance(Utterance::Command(action.clone()))])
                    }
                    _ => None,
                },
            ),
        ),
    ];
    // "reason for that change", "reasoning behind that change"
    for (id, ground) in [("en.reason.for", "for"), ("en.reason.behind", "behind")] {
        rules.push(rule(
            id,
            Cat::Reason,
            vec![C(Cat::Reason), W(ground), C(Cat::Deed)],
            first(),
        ));
    }
    rules.extend(
        CARRY
            .iter()
            .map(|(id, verb)| rule(id, Cat::Carrying, vec![W(verb), C(Cat::Deed)], first())),
    );
    for (table, cat) in [
        (DEEDS, Cat::Deed),
        (PROPOSED, Cat::Proposed),
        (WISHES, Cat::Wish),
        (REASONS, Cat::Reason),
        (HEDGES, Cat::Hedge),
    ] {
        rules.extend(
            table
                .iter()
                .map(|(id, words)| word(RuleInfo::new(id, ROUND), cat, words, blank()).read_only()),
        );
    }
    rules
}
