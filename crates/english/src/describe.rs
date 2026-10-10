//! Asking what a thing is and what it is for: "what does `x` do?", "what
//! is `x` for?", "what's `x` testing", "purpose of `x`", "explain the `X`
//! struct", and the system's answer, "`x` is the function defined at
//! src/x.rs:12.".

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Question, Reply, Thing};

use crate::lexicon::{ITEM, PLACE};
use crate::sem::{Cat, Number, Sem};
use crate::word;
use crate::{blank, first, last};

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 3;

/// What the thing mentioned among the children is.
fn describe<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find_map(|v| match v {
        Sem::Mention { thing, kind } => Some(Sem::Question(Question::Describe {
            thing: thing.clone(),
            kind: kind.clone(),
        })),
        _ => None,
    })
}

/// Verbs for what a thing does when it runs. The first is how the system
/// says it.
const DOING: &[(&str, &str)] = &[
    ("en.doing.do", "do"),
    ("en.doing.does", "does"),
    ("en.doing.doing", "doing"),
    ("en.doing.check", "check"),
    ("en.doing.checks", "checks"),
    ("en.doing.checking", "checking"),
    ("en.doing.checking_for", "checking for"),
    ("en.doing.test", "test"),
    ("en.doing.testing", "testing"),
    ("en.doing.testing_for", "testing for"),
    ("en.doing.represents", "represents"),
];

/// What a thing serves: "what is `x` for", "used for", "about".
const USE: &[(&str, &str)] = &[
    ("en.use.for", "for"),
    ("en.use.used_for", "used for"),
    ("en.use.about", "about"),
];

/// Nouns for what a thing is for.
const PURPOSE: &[(&str, &str)] = &[("en.purpose.purpose", "purpose")];

/// Words that soften or sharpen a question: "what is `x` even for", "what
/// does `x` do exactly", "explain `x` quickly".
const HEDGES: &[(&str, &str)] = &[
    ("en.hedge.even", "even"),
    ("en.hedge.actually", "actually"),
    ("en.hedge.exactly", "exactly"),
    ("en.hedge.basically", "basically"),
    ("en.hedge.quickly", "quickly"),
    ("en.hedge.a_bit", "a bit"),
];

/// "what is", "what's", "whats".
const WHAT_IS: &[(&str, &str)] = &[
    ("en.what_is.what_is", "what is"),
    ("en.what_is.what_s", "what's"),
    ("en.what_is.whats", "whats"),
];

/// Words that frame a request to be told about something: "can you
/// explain what `x` does", "I'm trying to understand what `x` does".
const PROMPTS: &[(&str, &str)] = &[
    ("en.prompt.explain", "explain"),
    ("en.prompt.walk_me_through", "walk me through"),
    ("en.prompt.understand", "understand"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    // A question asking what the thing mentioned is.
    let question = |id, rhs| rule(id, Cat::Question, rhs, describe);
    let mut rules = vec![
        // "what does `x` do": how the system asks it, too.
        Rule::new(
            RuleInfo::new("en.question.what_does_do", ROUND),
            Cat::Question,
            vec![W("what"), W("does"), C(Cat::Mention), C(Cat::Doing)],
            S::Both(describe, |v| match v {
                Sem::Question(Question::Describe { thing, kind }) => Some(vec![
                    Sem::Mention {
                        thing: thing.clone(),
                        kind: kind.clone(),
                    },
                    blank(),
                ]),
                _ => None,
            }),
        ),
        // "what's `seed_at` do", "what is `x` doing", "what's `x` testing"
        question(
            "en.question.what_is_doing",
            vec![C(Cat::WhatIs), C(Cat::Mention), C(Cat::Doing)],
        ),
        // "what is `x` for", "what's `x` even for", "what is `x` used for"
        question(
            "en.question.what_is_for",
            vec![C(Cat::WhatIs), C(Cat::Mention), C(Cat::Use)],
        ),
        // "what is `Found`", "LastReviewQuery - what is it"
        question("en.question.what_is", vec![C(Cat::WhatIs), C(Cat::Mention)]),
        // In embedded questions: "what `x` does", "what `Signature` represents".
        question(
            "en.question.what_doing",
            vec![W("what"), C(Cat::Mention), C(Cat::Doing)],
        ),
        // "`tabs_has` does what": asked in the order of a statement.
        Rule::new(
            RuleInfo::new("en.question.does_what", 10),
            Cat::Question,
            vec![C(Cat::Mention), C(Cat::Doing), W("what")],
            S::Build(describe),
        )
        .read_only(),
        // "what `x` is for", "what the `Notice` struct is used for", "what
        // `x` is checking"
        question(
            "en.question.what_x_is_for",
            vec![W("what"), C(Cat::Mention), W("is"), C(Cat::Use)],
        ),
        question(
            "en.question.what_x_is_doing",
            vec![W("what"), C(Cat::Mention), W("is"), C(Cat::Doing)],
        ),
        // "what it's for", "what it's used for": what the thing said
        // before is.
        rule(
            "en.question.what_it_s_for",
            Cat::Question,
            vec![W("what"), W("it's"), C(Cat::Use)],
            |_| {
                describe(&[Sem::Mention {
                    thing: Thing::Previous,
                    kind: None,
                }])
            },
        ),
        // "purpose of `boards_plain_and_wrapped`", "what is the purpose of `x`"
        question(
            "en.question.purpose_of",
            vec![C(Cat::Purpose), W("of"), C(Cat::Mention)],
        ),
        // "`x` purpose", "utc_times fn - purpose?"
        question(
            "en.question.purpose",
            vec![C(Cat::Mention), C(Cat::Purpose)],
        ),
        question(
            "en.question.set_off_purpose",
            vec![C(Cat::Mention), C(Cat::Break), C(Cat::Purpose)],
        ),
        // "explain message_rows", "explain the Thread struct"
        question("en.question.explain", vec![W("explain"), C(Cat::Mention)]),
        // "what does `x` do exactly", "so what is `x` for, basically?"
        rule(
            "en.question.hedged",
            Cat::Question,
            vec![C(Cat::Question), C(Cat::Hedge)],
            first,
        ),
        rule(
            "en.question.comma_hedged",
            Cat::Question,
            vec![C(Cat::Question), W(","), C(Cat::Hedge)],
            first,
        ),
        // "`x` is the function defined at src/x.rs:12."
        Rule::new(
            RuleInfo::new("en.reply.account", ROUND),
            Cat::Reply,
            vec![
                Class(ITEM),
                W("is"),
                W("the"),
                C(Cat::Noun),
                W("defined"),
                W("at"),
                Class(PLACE),
                W("."),
            ],
            S::Both(
                |_| None,
                |v: &Sem<O>| match v {
                    Sem::Reply(Reply::Account { item, kind }) => Some(vec![
                        Sem::Item(item.clone()),
                        Sem::Kind {
                            kind: kind.clone(),
                            number: Number::Singular,
                        },
                        Sem::Item(item.clone()),
                    ]),
                    _ => None,
                },
            ),
        ),
    ];
    // "what does `x` actually do", "what is `x` even for"
    for (id, cat) in [("en.doing.hedged", Cat::Doing), ("en.use.hedged", Cat::Use)] {
        rules.push(rule(id, cat, vec![C(Cat::Hedge), C(cat)], last));
    }
    for (table, cat) in [
        (DOING, Cat::Doing),
        (USE, Cat::Use),
        (PURPOSE, Cat::Purpose),
        (HEDGES, Cat::Hedge),
        (WHAT_IS, Cat::WhatIs),
        (PROMPTS, Cat::Prompt),
    ] {
        rules.extend(
            table
                .iter()
                .map(|&(id, words)| word(RuleInfo::new(id, ROUND), cat, words, blank())),
        );
    }
    rules
}
