//! Asking what a thing is and what it is for: "what does `x` do?", "what
//! is `x` for?", "what's `x` testing", "purpose of `x`", "explain the `X`
//! struct", and the system's answer, "`x` is the function defined at
//! src/x.rs:12.".

use glassmind_grammar::{Rule, RuleInfo, Sym};
use glassmind_meaning::{Ontology, Question, Reply, Thing};

use crate::aside::filler;
use crate::asking::read_only_rule;
use crate::lexicon::{ITEM, PLACE};
use crate::locate::fillers;
use crate::mention::related;
use crate::sem::{Cat, Number, Sem};
use crate::spoken;
use crate::word;
use crate::{blank, first, last};

use Cat::*;
use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 3;
const NOW: u32 = 14;

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

/// "what does `unblock` do on `Chat`", "what is `x` for, in the `Chat`":
/// the question about the thing, the one with its name that stands so to
/// the other thing.
fn placed<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let [
        Sem::Question(Question::Describe {
            thing: thing @ Thing::Named(_),
            kind,
        }),
        ..,
        relation,
        other,
    ] = c
    else {
        return None;
    };
    let mention = Sem::Mention {
        thing: thing.clone(),
        kind: kind.clone(),
    };
    describe(&[related(&[mention, relation.clone(), other.clone()])?])
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

/// More such words, from round 14: "what does it represent", "could you
/// explain me what `x` is for".
const LATER: &[(Cat, &[(&str, &str)])] = &[
    (Doing, &[("en.doing.represent", "represent")]),
    (Prompt, &[("en.prompt.explain_me", "explain me")]),
];

/// Asking what the thing said before is.
fn previous_described<O: Ontology>(_: &[Sem<O>]) -> Option<Sem<O>> {
    describe(&[Sem::Mention {
        thing: Thing::Previous,
        kind: None,
    }])
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| read_only_rule(id, ROUND, lhs, rhs, build);
    // A question asking what the thing mentioned is.
    let question = |id, rhs| rule(id, Question, rhs, describe);
    let mut rules = vec![
        // "what does `x` do": how the system asks it, too.
        spoken(
            "en.question.what_does_do",
            ROUND,
            Question,
            vec![W("what does"), C(Mention), C(Doing)],
            describe,
            |v| match v {
                Sem::Question(Question::Describe { thing, kind }) => Some(vec![
                    Sem::Mention {
                        thing: thing.clone(),
                        kind: kind.clone(),
                    },
                    blank(),
                ]),
                _ => None,
            },
        ),
        // "what's `seed_at` do", "what is `x` doing", "what's `x` testing"
        question(
            "en.question.what_is_doing",
            vec![C(WhatIs), C(Mention), C(Doing)],
        ),
        // "what is `x` for", "what's `x` even for", "what is `x` used for"
        question(
            "en.question.what_is_for",
            vec![C(WhatIs), C(Mention), C(Use)],
        ),
        // "what is `Found`", "LastReviewQuery - what is it"
        question("en.question.what_is", vec![C(WhatIs), C(Mention)]),
        // In embedded questions: "what `x` does", "what `Signature` represents".
        question(
            "en.question.what_doing",
            vec![W("what"), C(Mention), C(Doing)],
        ),
        // "`tabs_has` does what": asked in the order of a statement.
        read_only_rule(
            "en.question.does_what",
            10,
            Question,
            vec![C(Mention), C(Doing), W("what")],
            describe,
        ),
        // "what `x` is for", "what the `Notice` struct is used for", "what
        // `x` is checking"
        question(
            "en.question.what_x_is_for",
            vec![W("what"), C(Mention), W("is"), C(Use)],
        ),
        question(
            "en.question.what_x_is_doing",
            vec![W("what"), C(Mention), W("is"), C(Doing)],
        ),
        // "what it's for", "what it's used for": what the thing said
        // before is.
        rule(
            "en.question.what_it_s_for",
            Question,
            vec![W("what it's"), C(Use)],
            previous_described,
        ),
        // "what `run_angles` do, what its for": the same, the apostrophe
        // left out.
        read_only_rule(
            "en.question.what_its_for",
            18,
            Question,
            vec![W("what its"), C(Use)],
            previous_described,
        ),
        // "purpose of `boards_plain_and_wrapped`", "what is the purpose of `x`"
        question(
            "en.question.purpose_of",
            vec![C(Purpose), W("of"), C(Mention)],
        ),
        // "`x` purpose", "utc_times fn - purpose?"
        question("en.question.purpose", vec![C(Mention), C(Purpose)]),
        question(
            "en.question.set_off_purpose",
            vec![C(Mention), C(Break), C(Purpose)],
        ),
        // "explain message_rows", "explain the Thread struct"
        question("en.question.explain", vec![W("explain"), C(Mention)]),
        // "what does `x` do exactly", "so what is `x` for, basically?"
        rule(
            "en.question.hedged",
            Question,
            vec![C(Question), C(Hedge)],
            first,
        ),
        rule(
            "en.question.comma_hedged",
            Question,
            vec![C(Question), W(","), C(Hedge)],
            first,
        ),
        // "`x` is the function defined at src/x.rs:12."
        spoken(
            "en.reply.account",
            ROUND,
            Reply,
            vec![
                Class(ITEM),
                W("is the"),
                C(Noun),
                W("defined at"),
                Class(PLACE),
                W("."),
            ],
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
    ];
    let now = |id, rhs, build| read_only_rule(id, NOW, Question, rhs, build);
    rules.extend([
        // "what Mark for", "what `RepoSort` used for", "what `cut` fn for":
        // asked with no "is"; and "`x` struct, what it for".
        now(
            "en.question.what_for",
            vec![W("what"), C(Mention), C(Use)],
            describe,
        ),
        // "describe unread_counter"
        now(
            "en.question.describe",
            vec![W("describe"), C(Mention)],
            describe,
        ),
        // "what does `unblock` do on `Chat`", "what `filterable` does in
        // `Menu`": the place said after the question.
        now(
            "en.question.placed",
            vec![C(Question), C(Preposition), C(Mention)],
            placed,
        ),
        now(
            "en.question.placed_after_comma",
            vec![C(Question), W(","), C(Preposition), C(Mention)],
            placed,
        ),
        // "what `x` does, can you explain?": framed by more than one
        // prompt, said after.
        now(
            "en.question.prompts_after",
            vec![C(Question), W(","), C(Prompt), C(Prompt)],
            first,
        ),
    ]);
    rules.extend(fillers(NOW, LATER));
    // In embedded questions: "can you tell me what `x` is", "what
    // `RepositoryWithPendingReview` is, what it's for".
    rules.push(read_only_rule(
        "en.question.what_x_is",
        15,
        Question,
        vec![W("what"), C(Mention), W("is")],
        describe,
    ));
    // "how many functions call `x`, roughly?"
    rules.push(filler("en.hedge.roughly", 11, Hedge, "roughly"));
    // "what does `armed` even mean"
    rules.push(filler("en.doing.mean", 21, Doing, "mean"));
    // "what does `x` do, explain me": the frame said after.
    rules.push(read_only_rule(
        "en.question.prompt_after",
        11,
        Question,
        vec![C(Question), W(","), C(Prompt)],
        first,
    ));
    // "what does `x` actually do", "what is `x` even for"
    for (id, cat) in [("en.doing.hedged", Doing), ("en.use.hedged", Use)] {
        rules.push(rule(id, cat, vec![C(Hedge), C(cat)], last));
    }
    for (table, cat) in [
        (DOING, Doing),
        (USE, Use),
        (PURPOSE, Purpose),
        (HEDGES, Hedge),
        (WHAT_IS, WhatIs),
        (PROMPTS, Prompt),
    ] {
        rules.extend(
            table
                .iter()
                .map(|&(id, words)| word(RuleInfo::new(id, ROUND), cat, words, blank())),
        );
    }
    rules
}
