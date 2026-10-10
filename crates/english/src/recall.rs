//! Asking what the system holds of the conversation: "can you show me what
//! you're keeping in your context right now?", "what do you remember of
//! this convo so far", "dump what you've got in context". The request is
//! about the conversation itself, so it names nothing. The system's answer:
//! "So far we have talked about `A` and `B`."

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Reply, Utterance};

use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 7;

/// The hearer before a progressive verb, as a statement or a question puts
/// it: "you're keeping", "are you holding".
const YOU_ARE: &[(&str, &[&str])] = &[
    ("en.you_are.you_re", &["you're"]),
    ("en.you_are.youre", &["youre"]),
    ("en.you_are.you_are", &["you", "are"]),
    ("en.you_are.are_you", &["are", "you"]),
];

/// The hearer before a plain verb: "you remember", "do you remember".
const YOU_DO: &[(&str, &[&str])] = &[
    ("en.you_do.you", &["you"]),
    ("en.you_do.do_you", &["do", "you"]),
];

/// Verbs of holding in mind, as progressives: "keeping", "keepin",
/// "holding", "remembering".
const KEEPING: &[(&str, &[&str])] = &[
    ("en.keeping.keeping", &["keeping"]),
    ("en.keeping.keepin", &["keepin"]),
    ("en.keeping.holding", &["holding"]),
    ("en.keeping.remembering", &["remembering"]),
];

/// Verbs of holding in mind, plain: "remember", "keep", "hold", "recall".
const KEEP: &[(&str, &[&str])] = &[
    ("en.keep.remember", &["remember"]),
    ("en.keep.recall", &["recall"]),
    ("en.keep.keep", &["keep"]),
    ("en.keep.hold", &["hold"]),
];

/// The hearer having something, said whole: "you've got", "have you got".
const HOLDERS: &[(&str, &[&str])] = &[
    ("en.holder.you_ve_got", &["you've", "got"]),
    ("en.holder.have_you_got", &["have", "you", "got"]),
];

/// Nouns for where the system holds what was said.
const MEMORY: &[(&str, &[&str])] = &[
    ("en.memory.context", &["context"]),
    ("en.memory.memory", &["memory"]),
];

/// Words for when the system holds it: "right now", "so far", "rn".
const SPANS: &[(&str, &[&str])] = &[
    ("en.span.right_now", &["right", "now"]),
    ("en.span.now", &["now"]),
    ("en.span.rn", &["rn"]),
    ("en.span.currently", &["currently"]),
    ("en.span.at_the_moment", &["at", "the", "moment"]),
    ("en.span.so_far", &["so", "far"]),
    ("en.span.at_this_point", &["at", "this", "point"]),
];

/// Prepositions before the conversation: "of", "from", "about", "in".
const WITHIN: &[(&str, &[&str])] = &[
    ("en.within.of", &["of"]),
    ("en.within.from", &["from"]),
    ("en.within.about", &["about"]),
    ("en.within.in", &["in"]),
];

/// The conversation being had.
const TALKS: &[(&str, &[&str])] = &[
    ("en.talk.this_conversation", &["this", "conversation"]),
    ("en.talk.our_conversation", &["our", "conversation"]),
    ("en.talk.the_conversation", &["the", "conversation"]),
    ("en.talk.this_convo", &["this", "convo"]),
    ("en.talk.our_convo", &["our", "convo"]),
];

/// Words saying the clause after them restates the one before.
const GLOSSES: &[(&str, &[&str])] = &[
    ("en.gloss.meaning", &["meaning"]),
    ("en.gloss.like", &["like"]),
    ("en.gloss.i_e", &["i.e", "."]),
    ("en.gloss.that_is", &["that", "is"]),
];

/// Verbs asking the system to give out what it holds, besides "show" and
/// "tell me": "dump what you've got in context".
const PROMPTS: &[(&str, &[&str])] = &[("en.prompt.dump", &["dump"])];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let blank = || Sem::Quote(String::new());
    let recall = || S::Const(Sem::Utterance(Utterance::AskRecall));
    let rule = |id, lhs, rhs, sem| Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, sem).read_only();
    let first = || S::Build(|c: &[Sem<O>]| c.first().cloned());
    let mut rules = vec![
        // "you're keeping", "are you holding", "do you remember"
        rule(
            "en.holder.you_are",
            Cat::Rememberer,
            vec![C(Cat::YouAre), C(Cat::Keeping)],
            S::Const(blank()),
        ),
        rule(
            "en.holder.you_do",
            Cat::Rememberer,
            vec![C(Cat::YouDo), C(Cat::Keep)],
            S::Const(blank()),
        ),
        // "you're currently keeping", "you currently remember"
        rule(
            "en.keeping.currently",
            Cat::Keeping,
            vec![W("currently"), C(Cat::Keeping)],
            S::Const(blank()),
        ),
        rule(
            "en.keep.currently",
            Cat::Keep,
            vec![W("currently"), C(Cat::Keep)],
            S::Const(blank()),
        ),
        // "your context"
        rule(
            "en.memory.your",
            Cat::Memory,
            vec![W("your"), C(Cat::Memory)],
            S::Const(blank()),
        ),
        // "from this conversation", "at this point in our conversation"
        rule(
            "en.span.talk",
            Cat::Span,
            vec![C(Cat::Among), C(Cat::Talk)],
            S::Const(blank()),
        ),
        rule(
            "en.span.span_talk",
            Cat::Span,
            vec![C(Cat::Span), C(Cat::Among), C(Cat::Talk)],
            S::Const(blank()),
        ),
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
        rule(
            "en.recall.in",
            Cat::Recall,
            vec![C(Cat::Recall), W("in"), C(Cat::Memory)],
            first(),
        ),
        rule(
            "en.recall.span",
            Cat::Recall,
            vec![C(Cat::Recall), C(Cat::Span)],
            first(),
        ),
        // "meaning what you remember of this conversation", "like what's in
        // your context"
        rule(
            "en.recall.gloss",
            Cat::Recall,
            vec![C(Cat::Gloss), C(Cat::Recall)],
            S::Build(|c: &[Sem<O>]| c.last().cloned()),
        ),
        // "what you're keeping in your context, please"
        rule(
            "en.recall.please",
            Cat::Recall,
            vec![C(Cat::Recall), W(","), W("please")],
            first(),
        ),
        // Asked, with or without its mark; "can you show me ..." and "I'd
        // like to see ..." are prompts before it.
        rule(
            "en.utterance.recall",
            Cat::Utterance,
            vec![C(Cat::Recall)],
            first(),
        ),
        rule(
            "en.utterance.recall_mark",
            Cat::Utterance,
            vec![C(Cat::Recall), C(Cat::Mark)],
            first(),
        ),
        // "So far we have talked about `a` and `b`."
        Rule::new(
            RuleInfo::new("en.reply.recall", ROUND),
            Cat::Reply,
            vec![
                W("so"),
                W("far"),
                W("we"),
                W("have"),
                W("talked"),
                W("about"),
                C(Cat::ItemList),
                W("."),
            ],
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
            vec![
                W("we"),
                W("have"),
                W("talked"),
                W("about"),
                W("nothing"),
                W("so"),
                W("far"),
                W("."),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Recall(items)) if items.is_empty() => Some(Vec::new()),
                    _ => None,
                },
            ),
        ),
    ];
    for (table, cat) in [
        (YOU_ARE, Cat::YouAre),
        (YOU_DO, Cat::YouDo),
        (KEEPING, Cat::Keeping),
        (KEEP, Cat::Keep),
        (HOLDERS, Cat::Rememberer),
        (MEMORY, Cat::Memory),
        (SPANS, Cat::Span),
        (WITHIN, Cat::Among),
        (TALKS, Cat::Talk),
        (GLOSSES, Cat::Gloss),
        (PROMPTS, Cat::Prompt),
    ] {
        rules.extend(
            table
                .iter()
                .map(|(id, words)| word(RuleInfo::new(id, ROUND), cat, words, blank()).read_only()),
        );
    }
    rules
}
