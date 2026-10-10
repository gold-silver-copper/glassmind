//! Asking why: "why do you want to change that?", "reason for that
//! change?", "can you explain why you want to make that change?". The
//! question is about the action the system was last asked for, so it names
//! nothing. The system's answer: "Because you asked me to rename `x` to `y`."

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Reply, Utterance};

use crate::aside::filler;
use crate::sem::{Cat, Sem};
use crate::spoken;
use crate::{blank, first, last};

use Cat::*;
use Sym::{Cat as C, Word as W};

const ROUND: u32 = 6;

/// The words of a "why" question, each meaning nothing by itself.
const WORDS: &[(&str, Cat, &str)] = &[
    // Words for the action the system took up.
    ("en.deed.that", Deed, "that"),
    ("en.deed.it", Deed, "it"),
    ("en.deed.that_change", Deed, "that change"),
    ("en.deed.this_change", Deed, "this change"),
    ("en.deed.the_change", Deed, "the change"),
    // Words saying the system put the action forward.
    (
        "en.proposed.you_just_proposed",
        Proposed,
        "you just proposed",
    ),
    // Words saying the system wants something, as a question asks it ("do
    // you want") or as a sloppy one does ("you want", "it wants"). Users
    // speak of the system as "you" or "it".
    ("en.wish.do_you_want", Wish, "do you want"),
    ("en.wish.you_want", Wish, "you want"),
    ("en.wish.does_it_want", Wish, "does it want"),
    ("en.wish.it_wants", Wish, "it wants"),
    // Nouns for why something is done, with what they may open with.
    ("en.reason.reason", Reason, "reason"),
    ("en.reason.reasoning", Reason, "reasoning"),
    ("en.reason.the_reason", Reason, "the reason"),
    ("en.reason.the_reasoning", Reason, "the reasoning"),
    ("en.reason.your_reasoning", Reason, "your reasoning"),
    // Words softening a "why" question, besides the hedges of other
    // questions.
    ("en.hedge.though", Hedge, "though"),
    ("en.hedge.tho", Hedge, "tho"),
];

/// Words of a "why" question from round 19: "why you want to change that
/// one?".
const WORDS_19: &[(&str, Cat, &str)] = &[("en.deed.that_one", Deed, "that one")];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule_in =
        |id, round, lhs, rhs, sem| Rule::new(RuleInfo::new(id, round), lhs, rhs, sem).read_only();
    let rule = |id, lhs, rhs, sem| rule_in(id, ROUND, lhs, rhs, sem);
    // A "why" question about the action the system took up.
    let why_in = |id, round, rhs| {
        rule_in(
            id,
            round,
            Why,
            rhs,
            S::Const(Sem::Utterance(Utterance::AskReason)),
        )
    };
    let why = |id, rhs| why_in(id, ROUND, rhs);
    // A phrase meaning what its first part means.
    let as_first = |id, lhs, rhs| rule(id, lhs, rhs, S::Build(first));
    let mut rules = vec![
        // "why?", "why tho": why the system did what it just did.
        why("en.why.why", vec![W("why")]),
        // "why do you want to change that", "why it wants to make that change"
        why(
            "en.why.want_to",
            vec![W("why"), C(Wish), W("to"), C(Carrying)],
        ),
        // "why u want chnage that", "why u wnat change that": the "to" left
        // out.
        why_in("en.why.want_bare", 19, vec![W("why"), C(Wish), C(Carrying)]),
        // "why does that change need to happen?"
        why_in(
            "en.why.need_to_happen",
            19,
            vec![W("why does"), C(Deed), W("need to happen")],
        ),
        // "why do you want that", "why do you want that changed"
        why("en.why.want", vec![W("why"), C(Wish), C(Deed)]),
        // "why change that", "why that change"
        why("en.why.carrying", vec![W("why"), C(Carrying)]),
        why("en.why.deed", vec![W("why"), C(Deed)]),
        // "reasoning behind that change", "what's the reason for this
        // change", "walk me through your reasoning"
        why("en.why.reason", vec![C(Reason)]),
        why("en.why.what_is_reason", vec![C(WhatIs), C(Reason)]),
        // "can you explain why ...", "I'd like to know why ..."
        rule(
            "en.why.prompt",
            Why,
            vec![C(Prompt), C(Why)],
            S::Build(last),
        ),
        // "why do you want to change that, exactly", "why change it tho"
        as_first("en.why.hedge", Why, vec![C(Why), C(Hedge)]),
        as_first("en.why.comma_hedge", Why, vec![C(Why), W(","), C(Hedge)]),
        // "why do you want to change that, the thing you just proposed"
        as_first("en.why.deed_again", Why, vec![C(Why), W(","), C(Deed)]),
        // "the change you just proposed", "what you just proposed"
        as_first("en.deed.proposed", Deed, vec![C(Deed), C(Proposed)]),
        rule(
            "en.deed.what_proposed",
            Deed,
            vec![W("what"), C(Proposed)],
            S::Const(blank()),
        ),
        // A "why" question, with or without its question mark.
        as_first("en.utterance.why_mark", Utterance, vec![C(Why), W("?")]),
        as_first("en.utterance.why", Utterance, vec![C(Why)]),
        // "Because you asked me to rename `x` to `y`."
        spoken(
            "en.reply.reason",
            ROUND,
            Reply,
            vec![W("because you asked me to"), C(Command), W(".")],
            |_| None,
            |v| match v {
                Sem::Reply(Reply::Reason { action, .. }) => {
                    Some(vec![Sem::Utterance(Utterance::Command(action.clone()))])
                }
                _ => None,
            },
        ),
    ];
    // "reason for that change", "reasoning behind that change"
    for (id, ground) in [("en.reason.for", "for"), ("en.reason.behind", "behind")] {
        rules.push(as_first(id, Reason, vec![C(Reason), W(ground), C(Deed)]));
    }
    // Verbs carrying out an action on what they are said of: "change that",
    // "make that change".
    for (id, verb) in [
        ("en.carrying.change", "change"),
        ("en.carrying.make", "make"),
    ] {
        rules.push(as_first(id, Carrying, vec![W(verb), C(Deed)]));
    }
    rules.extend(
        WORDS
            .iter()
            .map(|&(id, cat, words)| filler(id, ROUND, cat, words))
            .chain(
                WORDS_19
                    .iter()
                    .map(|&(id, cat, words)| filler(id, 19, cat, words)),
            ),
    );
    rules
}
