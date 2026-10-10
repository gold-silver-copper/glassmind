//! Asking why: "why do you want to change that?", "reason for that
//! change?", "can you explain why you want to make that change?". The
//! question is about the action the system was last asked for, so it names
//! nothing. The system's answer: "Because you asked me to rename `x` to `y`."

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Reply, Utterance};

use crate::aside::filler;
use crate::phrase;
use crate::sem::{Cat, Sem};
use crate::{blank, first, last};

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 6;

/// The words of a "why" question, each meaning nothing by itself.
const WORDS: &[(&str, Cat, &str)] = &[
    // Words for the action the system took up.
    ("en.deed.that", Cat::Deed, "that"),
    ("en.deed.it", Cat::Deed, "it"),
    ("en.deed.that_change", Cat::Deed, "that change"),
    ("en.deed.this_change", Cat::Deed, "this change"),
    ("en.deed.the_change", Cat::Deed, "the change"),
    ("en.deed.the_thing", Cat::Deed, "the thing"),
    // Words saying the system put the action forward.
    (
        "en.proposed.you_just_proposed",
        Cat::Proposed,
        "you just proposed",
    ),
    // Words saying the system wants something, as a question asks it ("do
    // you want") or as a sloppy one does ("you want", "it wants"). Users
    // speak of the system as "you" or "it".
    ("en.wish.do_you_want", Cat::Wish, "do you want"),
    ("en.wish.you_want", Cat::Wish, "you want"),
    ("en.wish.does_it_want", Cat::Wish, "does it want"),
    ("en.wish.it_wants", Cat::Wish, "it wants"),
    ("en.wish.would_it_want", Cat::Wish, "would it want"),
    // Nouns for why something is done, with what they may open with.
    ("en.reason.reason", Cat::Reason, "reason"),
    ("en.reason.reasoning", Cat::Reason, "reasoning"),
    ("en.reason.the_reason", Cat::Reason, "the reason"),
    ("en.reason.the_reasoning", Cat::Reason, "the reasoning"),
    ("en.reason.your_reasoning", Cat::Reason, "your reasoning"),
    // Words softening a "why" question, besides the hedges of other
    // questions.
    ("en.hedge.though", Cat::Hedge, "though"),
    ("en.hedge.tho", Cat::Hedge, "tho"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, sem| Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, sem).read_only();
    // A "why" question about the action the system took up.
    let why = |id, rhs| {
        rule(
            id,
            Cat::Why,
            rhs,
            S::Const(Sem::Utterance(Utterance::AskReason)),
        )
    };
    // A phrase meaning what its first part means.
    let as_first = |id, lhs, rhs| rule(id, lhs, rhs, S::Build(first));
    let mut rules = vec![
        // "why?", "why tho": why the system did what it just did.
        why("en.why.why", vec![W("why")]),
        // "why do you want to change that", "why it wants to make that change"
        why(
            "en.why.want_to",
            vec![W("why"), C(Cat::Wish), W("to"), C(Cat::Carrying)],
        ),
        // "why do you want that", "why do you want that changed"
        why("en.why.want", vec![W("why"), C(Cat::Wish), C(Cat::Deed)]),
        why(
            "en.why.want_changed",
            vec![W("why"), C(Cat::Wish), C(Cat::Deed), W("changed")],
        ),
        // "why change that", "why that change"
        why("en.why.carrying", vec![W("why"), C(Cat::Carrying)]),
        why("en.why.deed", vec![W("why"), C(Cat::Deed)]),
        // "reasoning behind that change", "what's the reason for this
        // change", "walk me through your reasoning"
        why("en.why.reason", vec![C(Cat::Reason)]),
        why(
            "en.why.what_is_reason",
            vec![C(Cat::WhatIs), C(Cat::Reason)],
        ),
        // "can you explain why ...", "I'd like to know why ..."
        rule(
            "en.why.prompt",
            Cat::Why,
            vec![C(Cat::Prompt), C(Cat::Why)],
            S::Build(last),
        ),
        // "why do you want to change that, exactly", "why change it tho"
        as_first("en.why.hedge", Cat::Why, vec![C(Cat::Why), C(Cat::Hedge)]),
        as_first(
            "en.why.comma_hedge",
            Cat::Why,
            vec![C(Cat::Why), W(","), C(Cat::Hedge)],
        ),
        // "why do you want to change that, the thing you just proposed"
        as_first(
            "en.why.deed_again",
            Cat::Why,
            vec![C(Cat::Why), W(","), C(Cat::Deed)],
        ),
        // "the change you just proposed", "what you just proposed"
        as_first(
            "en.deed.proposed",
            Cat::Deed,
            vec![C(Cat::Deed), C(Cat::Proposed)],
        ),
        rule(
            "en.deed.what_proposed",
            Cat::Deed,
            vec![W("what"), C(Cat::Proposed)],
            S::Const(blank()),
        ),
        // "change it like that"
        as_first(
            "en.carrying.like_that",
            Cat::Carrying,
            vec![C(Cat::Carrying), W("like"), W("that")],
        ),
        // A "why" question, with or without its question mark.
        as_first(
            "en.utterance.why_mark",
            Cat::Utterance,
            vec![C(Cat::Why), W("?")],
        ),
        as_first("en.utterance.why", Cat::Utterance, vec![C(Cat::Why)]),
        // "Because you asked me to rename `x` to `y`."
        Rule::new(
            RuleInfo::new("en.reply.reason", ROUND),
            Cat::Reply,
            [
                phrase("because you asked me to"),
                vec![C(Cat::Command), W(".")],
            ]
            .concat(),
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
        rules.push(as_first(
            id,
            Cat::Reason,
            vec![C(Cat::Reason), W(ground), C(Cat::Deed)],
        ));
    }
    // Verbs carrying out an action on what they are said of: "change that",
    // "make that change".
    for (id, verb) in [
        ("en.carrying.change", "change"),
        ("en.carrying.make", "make"),
    ] {
        rules.push(as_first(id, Cat::Carrying, vec![W(verb), C(Cat::Deed)]));
    }
    rules.extend(
        WORDS
            .iter()
            .map(|&(id, cat, words)| filler(id, ROUND, cat, words)),
    );
    rules
}
