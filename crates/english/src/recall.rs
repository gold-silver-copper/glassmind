//! Asking what the system holds of the conversation: "can you show me what
//! you're keeping in your context right now?", "what do you remember of
//! this convo so far", "dump what you've got in context". The request is
//! about the conversation itself, so it names nothing. The system's answer:
//! "So far we have talked about `A` and `B`."

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Reply, Utterance};

use crate::aside::filler;
use crate::phrase;
use crate::sem::{Cat, Sem};
use crate::spoken;
use crate::{blank, first, last};

use Cat::*;
use Sym::{Cat as C, Word as W};

const ROUND: u32 = 7;

/// The words of a request to recall, each meaning nothing by itself.
const WORDS: &[(&str, Cat, &str)] = &[
    // The hearer before a progressive verb, as a statement or a question
    // puts it: "you're keeping", "are you holding".
    ("en.you_are.you_re", YouAre, "you're"),
    ("en.you_are.you_are", YouAre, "you are"),
    ("en.you_are.are_you", YouAre, "are you"),
    // The hearer before a plain verb: "you remember", "do you remember".
    ("en.you_do.you", YouDo, "you"),
    ("en.you_do.do_you", YouDo, "do you"),
    // Verbs of holding in mind, as progressives: "keeping", "keepin",
    // "holding", "remembering".
    ("en.keeping.keeping", Keeping, "keeping"),
    ("en.keeping.keepin", Keeping, "keepin"),
    ("en.keeping.holding", Keeping, "holding"),
    ("en.keeping.remembering", Keeping, "remembering"),
    // Verbs of holding in mind, plain: "remember", "keep".
    ("en.keep.remember", Keep, "remember"),
    ("en.keep.keep", Keep, "keep"),
    // The hearer having something, said whole: "you've got".
    ("en.holder.you_ve_got", Rememberer, "you've got"),
    // Nouns for where the system holds what was said.
    ("en.memory.context", Memory, "context"),
    // Words for when the system holds it: "right now", "so far", "rn".
    ("en.span.right_now", Span, "right now"),
    ("en.span.rn", Span, "rn"),
    ("en.span.at_the_moment", Span, "at the moment"),
    ("en.span.so_far", Span, "so far"),
    ("en.span.at_this_point", Span, "at this point"),
    // Prepositions before the conversation: "of", "from", "about", "in".
    ("en.within.of", Among, "of"),
    ("en.within.from", Among, "from"),
    ("en.within.about", Among, "about"),
    ("en.within.in", Among, "in"),
    // The conversation being had.
    ("en.talk.this_conversation", Talk, "this conversation"),
    ("en.talk.our_conversation", Talk, "our conversation"),
    ("en.talk.this_convo", Talk, "this convo"),
    // Words saying the clause after them restates the one before.
    ("en.gloss.meaning", Gloss, "meaning"),
    ("en.gloss.i_e", Gloss, "i.e ."),
    // Verbs asking the system to give out what it holds, besides "show"
    // and "tell me": "dump what you've got in context".
    ("en.prompt.dump", Prompt, "dump"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let recall = || S::Const(Sem::Utterance(Utterance::AskRecall));
    let rule = |id, lhs, rhs, sem| Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, sem).read_only();
    // A phrase of a request to recall, meaning nothing by itself.
    let empty = |id, lhs, rhs| rule(id, lhs, rhs, S::Const(blank()));
    // A phrase meaning what its first part means.
    let as_first = |id, lhs, rhs| rule(id, lhs, rhs, S::Build(first));
    let mut rules = vec![
        // "you're keeping", "are you holding", "do you remember"
        empty("en.holder.you_are", Rememberer, vec![C(YouAre), C(Keeping)]),
        empty("en.holder.you_do", Rememberer, vec![C(YouDo), C(Keep)]),
        // "from this conversation", "at this point in our conversation"
        empty("en.span.talk", Span, vec![C(Among), C(Talk)]),
        // "what you're keeping", "what do you remember"
        rule(
            "en.recall.what",
            Recall,
            vec![W("what"), C(Rememberer)],
            recall(),
        ),
        // "what's in your context"
        rule(
            "en.recall.what_is_in",
            Recall,
            vec![C(WhatIs), W("in"), C(Memory)],
            recall(),
        ),
        // "what you're keeping in your context", "... right now", "... of
        // this conversation so far"
        as_first("en.recall.in", Recall, vec![C(Recall), W("in"), C(Memory)]),
        as_first("en.recall.span", Recall, vec![C(Recall), C(Span)]),
        // "meaning what you remember of this conversation", "like what's in
        // your context"
        rule(
            "en.recall.gloss",
            Recall,
            vec![C(Gloss), C(Recall)],
            S::Build(last),
        ),
        // Asked, with or without its mark; "can you show me ..." and "I'd
        // like to see ..." are prompts before it.
        as_first("en.utterance.recall", Utterance, vec![C(Recall)]),
        as_first(
            "en.utterance.recall_mark",
            Utterance,
            vec![C(Recall), C(Mark)],
        ),
        // "So far we have talked about `a` and `b`."
        spoken(
            "en.reply.recall",
            ROUND,
            Reply,
            vec![W("so far we have talked about"), C(ItemList), W(".")],
            |_| None,
            |v| match v {
                Sem::Reply(Reply::Recall(items)) if !items.is_empty() => {
                    Some(vec![Sem::Items(items.clone())])
                }
                _ => None,
            },
        ),
        // "We have talked about nothing so far."
        spoken(
            "en.reply.recall_nothing",
            ROUND,
            Reply,
            phrase("we have talked about nothing so far ."),
            |_| None,
            |v| match v {
                Sem::Reply(Reply::Recall(items)) if items.is_empty() => Some(Vec::new()),
                _ => None,
            },
        ),
    ];
    rules.extend(
        WORDS
            .iter()
            .map(|&(id, cat, words)| filler(id, ROUND, cat, words)),
    );
    rules
}
