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
use crate::{blank, first, last};

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 7;

/// The words of a request to recall, each meaning nothing by itself.
const WORDS: &[(&str, Cat, &str)] = &[
    // The hearer before a progressive verb, as a statement or a question
    // puts it: "you're keeping", "are you holding".
    ("en.you_are.you_re", Cat::YouAre, "you're"),
    ("en.you_are.you_are", Cat::YouAre, "you are"),
    ("en.you_are.are_you", Cat::YouAre, "are you"),
    // The hearer before a plain verb: "you remember", "do you remember".
    ("en.you_do.you", Cat::YouDo, "you"),
    ("en.you_do.do_you", Cat::YouDo, "do you"),
    // Verbs of holding in mind, as progressives: "keeping", "keepin",
    // "holding", "remembering".
    ("en.keeping.keeping", Cat::Keeping, "keeping"),
    ("en.keeping.keepin", Cat::Keeping, "keepin"),
    ("en.keeping.holding", Cat::Keeping, "holding"),
    ("en.keeping.remembering", Cat::Keeping, "remembering"),
    // Verbs of holding in mind, plain: "remember", "keep".
    ("en.keep.remember", Cat::Keep, "remember"),
    ("en.keep.keep", Cat::Keep, "keep"),
    // The hearer having something, said whole: "you've got".
    ("en.holder.you_ve_got", Cat::Rememberer, "you've got"),
    // Nouns for where the system holds what was said.
    ("en.memory.context", Cat::Memory, "context"),
    // Words for when the system holds it: "right now", "so far", "rn".
    ("en.span.right_now", Cat::Span, "right now"),
    ("en.span.rn", Cat::Span, "rn"),
    ("en.span.at_the_moment", Cat::Span, "at the moment"),
    ("en.span.so_far", Cat::Span, "so far"),
    ("en.span.at_this_point", Cat::Span, "at this point"),
    // Prepositions before the conversation: "of", "from", "about", "in".
    ("en.within.of", Cat::Among, "of"),
    ("en.within.from", Cat::Among, "from"),
    ("en.within.about", Cat::Among, "about"),
    ("en.within.in", Cat::Among, "in"),
    // The conversation being had.
    ("en.talk.this_conversation", Cat::Talk, "this conversation"),
    ("en.talk.our_conversation", Cat::Talk, "our conversation"),
    ("en.talk.this_convo", Cat::Talk, "this convo"),
    // Words saying the clause after them restates the one before.
    ("en.gloss.meaning", Cat::Gloss, "meaning"),
    ("en.gloss.i_e", Cat::Gloss, "i.e ."),
    // Verbs asking the system to give out what it holds, besides "show"
    // and "tell me": "dump what you've got in context".
    ("en.prompt.dump", Cat::Prompt, "dump"),
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
        empty(
            "en.holder.you_are",
            Cat::Rememberer,
            vec![C(Cat::YouAre), C(Cat::Keeping)],
        ),
        empty(
            "en.holder.you_do",
            Cat::Rememberer,
            vec![C(Cat::YouDo), C(Cat::Keep)],
        ),
        // "you're currently keeping", "you currently remember"
        empty(
            "en.keeping.currently",
            Cat::Keeping,
            vec![W("currently"), C(Cat::Keeping)],
        ),
        empty(
            "en.keep.currently",
            Cat::Keep,
            vec![W("currently"), C(Cat::Keep)],
        ),
        // "your context"
        empty(
            "en.memory.your",
            Cat::Memory,
            vec![W("your"), C(Cat::Memory)],
        ),
        // "from this conversation", "at this point in our conversation"
        empty("en.span.talk", Cat::Span, vec![C(Cat::Among), C(Cat::Talk)]),
        // "what you're keeping", "what do you remember"
        rule(
            "en.recall.what",
            Cat::Recall,
            vec![W("what"), C(Cat::Rememberer)],
            recall(),
        ),
        // "what's in your context"
        rule(
            "en.recall.what_is_in",
            Cat::Recall,
            vec![C(Cat::WhatIs), W("in"), C(Cat::Memory)],
            recall(),
        ),
        // "what you're keeping in your context", "... right now", "... of
        // this conversation so far"
        as_first(
            "en.recall.in",
            Cat::Recall,
            vec![C(Cat::Recall), W("in"), C(Cat::Memory)],
        ),
        as_first(
            "en.recall.span",
            Cat::Recall,
            vec![C(Cat::Recall), C(Cat::Span)],
        ),
        // "meaning what you remember of this conversation", "like what's in
        // your context"
        rule(
            "en.recall.gloss",
            Cat::Recall,
            vec![C(Cat::Gloss), C(Cat::Recall)],
            S::Build(last),
        ),
        // "what you're keeping in your context, please"
        as_first(
            "en.recall.please",
            Cat::Recall,
            vec![C(Cat::Recall), W(","), W("please")],
        ),
        // Asked, with or without its mark; "can you show me ..." and "I'd
        // like to see ..." are prompts before it.
        as_first("en.utterance.recall", Cat::Utterance, vec![C(Cat::Recall)]),
        as_first(
            "en.utterance.recall_mark",
            Cat::Utterance,
            vec![C(Cat::Recall), C(Cat::Mark)],
        ),
        // "So far we have talked about `a` and `b`."
        Rule::new(
            RuleInfo::new("en.reply.recall", ROUND),
            Cat::Reply,
            [
                phrase("so far we have talked about"),
                vec![C(Cat::ItemList), W(".")],
            ]
            .concat(),
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Recall(items)) if !items.is_empty() => {
                        Some(vec![Sem::Items(items.clone())])
                    }
                    _ => None,
                },
            ),
        ),
        // "We have talked about nothing so far."
        Rule::new(
            RuleInfo::new("en.reply.recall_nothing", ROUND),
            Cat::Reply,
            phrase("we have talked about nothing so far ."),
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Recall(items)) if items.is_empty() => Some(Vec::new()),
                    _ => None,
                },
            ),
        ),
    ];
    rules.extend(
        WORDS
            .iter()
            .map(|&(id, cat, words)| filler(id, ROUND, cat, words)),
    );
    rules
}
