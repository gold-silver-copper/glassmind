//! Checking, fixing and agreeing (written by hand to seed the harness;
//! the loop grows the other ways of saying these): "does the build pass",
//! "are there problems with the tests", "fix the build", "yes", "no", and
//! the replies: what a check found, the changes offered to fix it, what
//! changed once they were made.
//!
//! The words for the checks themselves ("build", "tests") come from a
//! bridge crate, as words of category `CheckName`.

use glassmind_grammar::{Rule, Sym};
use glassmind_meaning::{Action, Ontology, Question, Reply, Trouble, Utterance};
use glassmind_rules::HAND;

use crate::asking::read_only_rule;
use crate::lexicon::{LOWER, NAME, NUMBER};
use crate::sem::{Cat, Sem};
use crate::{blank, first, last, spoken};

use Cat::*;
use Sym::{Cat as C, Class, Word as W};

/// A reply rule: the system says it, the user never does.
fn said<O: Ontology>(
    id: &'static str,
    cat: Cat,
    rhs: Vec<Sym<Cat>>,
    speak: fn(&Sem<O>) -> Option<Vec<Sem<O>>>,
) -> Rule<Cat, Sem<O>> {
    spoken(id, HAND, cat, rhs, |_| None, speak)
}

/// A request to make `check` pass, from the first check among `c`.
fn repair<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find_map(|v| match v {
        Sem::Check(check) => Some(Sem::Utterance(Utterance::Command(Action::Repair(
            check.clone(),
        )))),
        _ => None,
    })
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let mut rules: Vec<Rule<Cat, Sem<O>>> = vec![
        // "the build", "the tests".
        spoken(
            "en.check_phrase.the",
            HAND,
            CheckPhrase,
            vec![W("the"), C(CheckName)],
            |c| c.first().cloned(),
            |v| Some(vec![v.clone()]),
        ),
        // "does the build pass", "do the tests pass".
        spoken(
            "en.question.passes",
            HAND,
            Question,
            vec![W("does"), C(CheckPhrase), W("pass")],
            |c| match c {
                [Sem::Check(check)] => Some(Sem::Question(Question::Passes(check.clone()))),
                _ => None,
            },
            |v| match v {
                Sem::Question(Question::Passes(check)) => Some(vec![Sem::Check(check.clone())]),
                _ => None,
            },
        ),
        Rule::new(
            glassmind_grammar::RuleInfo::new("en.question.passes_do", HAND),
            Question,
            vec![W("do"), C(CheckPhrase), W("pass")],
            glassmind_grammar::Sem::Build(|c: &[Sem<O>]| match c {
                [Sem::Check(check)] => Some(Sem::Question(Question::Passes(check.clone()))),
                _ => None,
            }),
        )
        .read_only(),
        // "are there problems with the build".
        spoken(
            "en.question.problems",
            HAND,
            Question,
            vec![
                W("are"),
                W("there"),
                W("problems"),
                W("with"),
                C(CheckPhrase),
            ],
            |c| match c {
                [Sem::Check(check)] => Some(Sem::Question(Question::Problems(check.clone()))),
                _ => None,
            },
            |v| match v {
                Sem::Question(Question::Problems(check)) => Some(vec![Sem::Check(check.clone())]),
                _ => None,
            },
        ),
        // "fix the build".
        spoken(
            "en.command.repair",
            HAND,
            Command,
            vec![W("fix"), C(CheckPhrase)],
            |c| match c {
                [Sem::Check(check)] => Some(Sem::Utterance(Utterance::Command(Action::Repair(
                    check.clone(),
                )))),
                _ => None,
            },
            |v| match v {
                Sem::Utterance(Utterance::Command(Action::Repair(check))) => {
                    Some(vec![Sem::Check(check.clone())])
                }
                _ => None,
            },
        ),
        // "yes", "no": to what was proposed.
        spoken(
            "en.utterance.yes",
            HAND,
            Utterance,
            vec![W("yes")],
            |_| Some(Sem::Utterance(Utterance::Consent(true))),
            |v| matches!(v, Sem::Utterance(Utterance::Consent(true))).then(Vec::new),
        ),
        spoken(
            "en.utterance.no",
            HAND,
            Utterance,
            vec![W("no")],
            |_| Some(Sem::Utterance(Utterance::Consent(false))),
            |v| matches!(v, Sem::Utterance(Utterance::Consent(false))).then(Vec::new),
        ),
        // "No problems with the build."
        said(
            "en.reply.checked_clean",
            Reply,
            vec![W("no problems with"), C(CheckPhrase), W(".")],
            |v| match v {
                Sem::Reply(Reply::Checked { check, problems }) if problems.is_empty() => {
                    Some(vec![Sem::Check(check.clone())])
                }
                _ => None,
            },
        ),
        // "I found 2 problems with the build: `E0425` and `E0596`."
        said(
            "en.reply.checked_problems",
            Reply,
            vec![
                W("I found"),
                Class(NUMBER),
                W("problems with"),
                C(CheckPhrase),
                W(":"),
                C(ItemList),
                W("."),
            ],
            |v| match v {
                Sem::Reply(Reply::Checked { check, problems }) if problems.len() > 1 => Some(vec![
                    Sem::Count(u64::try_from(problems.len()).ok()?),
                    Sem::Check(check.clone()),
                    Sem::Items(problems.clone()),
                ]),
                _ => None,
            },
        ),
        said(
            "en.reply.checked_problem",
            Reply,
            vec![
                W("I found one problem with"),
                C(CheckPhrase),
                W(":"),
                C(ItemList),
                W("."),
            ],
            |v| match v {
                Sem::Reply(Reply::Checked { check, problems }) if problems.len() == 1 => {
                    Some(vec![
                        Sem::Check(check.clone()),
                        Sem::Items(problems.clone()),
                    ])
                }
                _ => None,
            },
        ),
        // "I'd make 2 changes to fix the build." (the changes are listed by the harness)
        said(
            "en.reply.remedy",
            Reply,
            vec![
                W("I'd make"),
                Class(NUMBER),
                W("changes to fix"),
                C(CheckPhrase),
                W("."),
            ],
            |v| match v {
                Sem::Reply(Reply::Remedy {
                    check,
                    changes,
                    left,
                    ..
                }) if changes.len() > 1 && left.is_empty() => Some(vec![
                    Sem::Count(u64::try_from(changes.len()).ok()?),
                    Sem::Check(check.clone()),
                ]),
                _ => None,
            },
        ),
        said(
            "en.reply.remedy_one",
            Reply,
            vec![W("I'd make one change to fix"), C(CheckPhrase), W(".")],
            |v| match v {
                Sem::Reply(Reply::Remedy {
                    check,
                    changes,
                    left,
                    ..
                }) if changes.len() == 1 && left.is_empty() => {
                    Some(vec![Sem::Check(check.clone())])
                }
                _ => None,
            },
        ),
        // "I'd make 2 changes; they leave 1 problems with the build."
        said(
            "en.reply.remedy_partial",
            Reply,
            vec![
                W("I'd make"),
                Class(NUMBER),
                W("changes , but some problems with"),
                C(CheckPhrase),
                W("would remain ."),
            ],
            |v| match v {
                Sem::Reply(Reply::Remedy {
                    check,
                    changes,
                    left,
                    ..
                }) if changes.len() > 1 && !left.is_empty() => Some(vec![
                    Sem::Count(u64::try_from(changes.len()).ok()?),
                    Sem::Check(check.clone()),
                ]),
                _ => None,
            },
        ),
        said(
            "en.reply.remedy_partial_one",
            Reply,
            vec![
                W("I'd make one change , but some problems with"),
                C(CheckPhrase),
                W("would remain ."),
            ],
            |v| match v {
                Sem::Reply(Reply::Remedy {
                    check,
                    changes,
                    left,
                    ..
                }) if changes.len() == 1 && !left.is_empty() => {
                    Some(vec![Sem::Check(check.clone())])
                }
                _ => None,
            },
        ),
        // "I don't know how to fix the build."
        said(
            "en.reply.no_remedy",
            Reply,
            vec![W("I don't know how to fix"), C(CheckPhrase), W(".")],
            |v| match v {
                Sem::Reply(Reply::Remedy { check, changes, .. }) if changes.is_empty() => {
                    Some(vec![Sem::Check(check.clone())])
                }
                _ => None,
            },
        ),
        // "I made 2 changes. No problems with the build."
        said(
            "en.reply.applied",
            Reply,
            vec![W("I made"), Class(NUMBER), W("changes ."), C(Reply)],
            |v| match v {
                Sem::Reply(Reply::Applied {
                    changes,
                    check: Some(check),
                    problems,
                }) if *changes != 1 => Some(vec![
                    Sem::Count(u64::try_from(*changes).ok()?),
                    Sem::Reply(Reply::Checked {
                        check: check.clone(),
                        problems: problems.clone(),
                    }),
                ]),
                _ => None,
            },
        ),
        said(
            "en.reply.applied_one",
            Reply,
            vec![W("I made one change ."), C(Reply)],
            |v| match v {
                Sem::Reply(Reply::Applied {
                    changes: 1,
                    check: Some(check),
                    problems,
                }) => Some(vec![Sem::Reply(Reply::Checked {
                    check: check.clone(),
                    problems: problems.clone(),
                })]),
                _ => None,
            },
        ),
        said(
            "en.reply.declined",
            Reply,
            vec![W("OK , I changed nothing .")],
            |v| matches!(v, Sem::Reply(Reply::Declined)).then(Vec::new),
        ),
        said(
            "en.reply.nothing_pending",
            Reply,
            vec![W("there is nothing waiting for a yes or no .")],
            |v| matches!(v, Sem::Reply(Reply::Trouble(Trouble::NothingPending))).then(Vec::new),
        ),
    ];
    let rule = |id, lhs, rhs, build| read_only_rule(id, 29, lhs, rhs, build);
    rules.extend([
        // "fix build", "fix compile errors", "fix the build issues", "fix
        // the heck compile errors": the check, with what is wrong with it
        // in a noun of the user's own, and the crate it is checked in.
        rule(
            "en.check_phrase.bare",
            CheckPhrase,
            vec![C(CheckName)],
            first,
        ),
        rule(
            "en.check_phrase.errors",
            CheckPhrase,
            vec![C(CheckPhrase), Class(LOWER)],
            first,
        ),
        rule(
            "en.check_phrase.named",
            CheckPhrase,
            vec![W("the"), Class(NAME), C(CheckName)],
            last,
        ),
        // "fix it so it compiles".
        rule(
            "en.command.repair_it_so",
            Command,
            vec![W("fix it so it"), C(CheckName)],
            repair,
        ),
        // "get the build working", each perhaps "again".
        rule(
            "en.command.get_working",
            Command,
            vec![W("get"), C(CheckPhrase), W("working")],
            repair,
        ),
        rule(
            "en.command.again",
            Command,
            vec![C(Command), W("again")],
            first,
        ),
        // "get it compiling", "get strsim compiling".
        rule(
            "en.command.get_going",
            Command,
            vec![W("get"), C(Mention), C(CheckName)],
            repair,
        ),
        // "build broken", "the dunce build is broken", "heck doesn't
        // compile": a remark naming a check, which a fix after it fixes.
        rule("en.remark.check", Remark, vec![C(CheckPhrase)], first),
        rule(
            "en.remark.then_check",
            Remark,
            vec![C(Remark), C(CheckName)],
            last,
        ),
        // "The dunce build is broken. I'm seeing E0425, "cannot find value
        // `x`". Could you fix it?": what a tool said goes on past a comma,
        // before the request. Gone on so long, what it was about is no
        // longer what the request's "it" refers back to.
        rule(
            "en.utterance.musing_stop",
            Utterance,
            vec![C(Musing), W("."), C(Utterance)],
            last,
        ),
        // "build fails ..., can you fix", "build broke, ... . pls fix",
        // "the build is broken, can you fix it?".
        rule("en.fix.bare", Fix, vec![W("fix")], |_| Some(blank())),
        rule("en.fix.it", Fix, vec![W("fix it")], |_| Some(blank())),
        rule("en.fix.mark", Fix, vec![C(Fix), C(Mark)], first),
        rule("en.fix.prompt", Fix, vec![C(Prompt), C(Fix)], last),
        rule(
            "en.fix.interjection",
            Fix,
            vec![C(Interjection), C(Fix)],
            last,
        ),
        rule(
            "en.utterance.failing_fix",
            Utterance,
            vec![C(Remark), C(Break), C(Fix)],
            repair,
        ),
        rule(
            "en.utterance.failing_stop_fix",
            Utterance,
            vec![C(Remark), W("."), C(Fix)],
            repair,
        ),
        rule(
            "en.utterance.musing_stop_fix",
            Utterance,
            vec![C(Musing), W("."), C(Fix)],
            repair,
        ),
    ]);
    rules
}
