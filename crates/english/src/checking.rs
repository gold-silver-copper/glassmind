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
use crate::lexicon::{LOWER, NAME, NUMBER, WORD};
use crate::locate::fillers;
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

/// The ways of asking for a check to be made to pass, and of naming the
/// check by what is wrong with it.
fn passing<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| read_only_rule(id, 29, lhs, rhs, build);
    let command = |id, rhs| read_only_rule(id, 29, Command, rhs, repair);
    let mut rules = vec![
        // "the failing tests", "failing tests", "the broken build": a check
        // named by its not passing.
        rule(
            "en.check_phrase.ailing",
            CheckPhrase,
            vec![C(Ailing), C(CheckName)],
            last,
        ),
        // "the failing tests pass", "they pass".
        rule(
            "en.pass_clause.pass",
            PassClause,
            vec![C(CheckPhrase), W("pass")],
            first,
        ),
        // "make failing tests pass in heck", "get failing tests green in
        // heck": the source they are in; "get the build working again".
        rule(
            "en.pass_clause.in",
            PassClause,
            vec![C(PassClause), W("in"), Class(NAME)],
            first,
        ),
        rule(
            "en.pass.in",
            Pass,
            vec![C(Pass), W("in"), Class(NAME)],
            first,
        ),
        rule("en.pass.again", Pass, vec![C(Pass), W("again")], first),
        // "make the failing tests pass", "fix so the tests pass", "fix
        // things so the failing tests pass", "get the failing tests to
        // pass", "get failing tests green".
        command("en.command.make_pass", vec![W("make"), C(PassClause)]),
        command("en.command.fix_so", vec![W("fix"), C(FixSo), C(PassClause)]),
        command(
            "en.command.get_passing",
            vec![W("get"), C(CheckPhrase), C(Pass)],
        ),
    ];
    rules.extend(fillers(
        29,
        &[
            (
                Ailing,
                &[
                    ("en.ailing.failing", "failing"),
                    ("en.ailing.the_failing", "the failing"),
                    ("en.ailing.broken", "broken"),
                    ("en.ailing.the_broken", "the broken"),
                ],
            ),
            (
                Pass,
                &[
                    ("en.pass.passing", "passing"),
                    ("en.pass.to_pass", "to pass"),
                    ("en.pass.green", "green"),
                    ("en.pass.working", "working"),
                ],
            ),
            (
                FixSo,
                &[
                    ("en.fix_so.so", "so"),
                    ("en.fix_so.things", "things so"),
                    ("en.fix_so.it", "it so"),
                ],
            ),
        ],
    ));
    rules
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
        // longer what the request's "it" refers back to. "some tests are
        // failing in siphasher, for example `x`. could you make the
        // failing tests pass?".
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
    rules.extend(outcomes());
    rules.extend(passing());
    rules
}

const GROWN: u32 = 29;

/// Whether the check comes out well.
fn passes<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c.first()? {
        Sem::Check(check) => Some(Sem::Question(Question::Passes(check.clone()))),
        _ => None,
    }
}

/// What comes out badly in the check.
fn problems<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c.first()? {
        Sem::Check(check) => Some(Sem::Question(Question::Problems(check.clone()))),
        _ => None,
    }
}

/// Ways of asking how checks come out, from round 29: "are the tests
/// passing at the moment, or which ones are failing?", "tests all passs?",
/// "can you tell me whether the test suite currently passes", "run teh
/// tests pls, what fails", "If any of them fail, which ones are they?".
fn outcomes<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| read_only_rule(id, GROWN, lhs, rhs, build);
    let mut rules = fillers(
        GROWN,
        &[
            (
                Passing,
                &[
                    ("en.passing.pass", "pass"),
                    ("en.passing.passes", "passes"),
                    ("en.passing.passing", "passing"),
                    ("en.passing.green", "green"),
                ],
            ),
            (
                Failing,
                &[
                    ("en.failing.fail", "fail"),
                    ("en.failing.fails", "fails"),
                    ("en.failing.failing", "failing"),
                    ("en.failing.red", "red"),
                ],
            ),
            (
                Whether,
                &[
                    ("en.whether.do", "do"),
                    ("en.whether.are", "are"),
                    ("en.whether.if", "if"),
                    ("en.whether.whether", "whether"),
                ],
            ),
            (
                Portion,
                &[
                    ("en.portion.which", "which"),
                    ("en.portion.which_ones", "which ones"),
                    ("en.portion.what", "what"),
                    ("en.portion.some", "some"),
                    ("en.portion.some_of_them", "some of them"),
                    ("en.portion.any", "any"),
                    ("en.portion.anything", "anything"),
                ],
            ),
            // "Please let me know if any are failing."
            (Prompt, &[("en.prompt.let_me_know", "let me know")]),
            // "If any of them fail, which ones are they?"
            (
                AskAgain,
                &[("en.ask_again.which_ones_are_they", "which ones are they")],
            ),
        ],
    );
    // "are failing", "currently passes", "failing rn", "passing in dunce":
    // with the verb's helper, or placed in time or in the crate, the verb
    // says the same; so does "all pass", said of every part.
    for (cat, [are, before, after]) in [
        (
            Passing,
            [
                "en.passing.are",
                "en.passing.set_before",
                "en.passing.set_after",
            ],
        ),
        (
            Failing,
            [
                "en.failing.are",
                "en.failing.set_before",
                "en.failing.set_after",
            ],
        ),
    ] {
        rules.extend([
            rule(are, cat, vec![W("are"), C(cat)], last),
            rule(before, cat, vec![C(Setting), C(cat)], last),
            rule(after, cat, vec![C(cat), C(Setting)], first),
        ]);
    }
    rules.push(rule(
        "en.passing.all",
        Passing,
        vec![W("all"), C(Passing)],
        last,
    ));
    rules.extend([
        // "right now", "at the moment", "rn": when the check is run.
        rule("en.setting.span", Setting, vec![C(Span)], first),
        // "in dunce": the crate the check is run on, the one talked about.
        rule("en.setting.in", Setting, vec![W("in"), Class(WORD)], first),
        // "the dunce tests", "the unicase test suite": a crate's check.
        rule(
            "en.check_name.of",
            CheckName,
            vec![Class(WORD), C(CheckName)],
            last,
        ),
        // "all the tests".
        rule(
            "en.check_phrase.all",
            CheckPhrase,
            vec![W("all"), C(CheckPhrase)],
            last,
        ),
        // "the tests pass", "any tests failing".
        rule(
            "en.outcome.passing",
            Outcome,
            vec![C(CheckPhrase), C(Passing)],
            passes,
        ),
        rule(
            "en.outcome.failing",
            Outcome,
            vec![C(CheckPhrase), C(Failing)],
            problems,
        ),
        // "siphasher tests passing?", "any tests fial rn": asked by
        // saying it.
        rule("en.question.outcome", Question, vec![C(Outcome)], first),
        // "do the tests pass", "are the tests in dunce passing", "whether
        // the test suite currently passes".
        rule(
            "en.question.whether",
            Question,
            vec![C(Whether), C(Outcome)],
            last,
        ),
        // "which tests are currently failing", "what tests fail".
        rule(
            "en.question.which_failing",
            Question,
            vec![C(Portion), C(Outcome)],
            |c| match c.last()? {
                q @ Sem::Question(Question::Problems(_)) => Some(q.clone()),
                _ => None,
            },
        ),
        // "run the tests", "check the dunce test suite": running a check
        // asks how it comes out.
        rule(
            "en.question.run",
            Question,
            vec![W("run"), C(CheckPhrase)],
            passes,
        ),
        rule(
            "en.question.check",
            Question,
            vec![W("check"), C(CheckPhrase)],
            passes,
        ),
        // "Do all the tests pass, or do any fail?", "..., and tell me
        // which tests fail": the same check asked of again.
        rule(
            "en.ask_again.check",
            AskAgain,
            vec![C(Question)],
            |c: &[Sem<O>]| match c.first()? {
                Sem::Question(Question::Passes(check) | Question::Problems(check)) => {
                    Some(Sem::Check(check.clone()))
                }
                _ => None,
            },
        ),
        // "or which ones are failing", "what fails", "are some of them
        // failing", "if any are failing".
        rule(
            "en.ask_again.failing",
            AskAgain,
            vec![C(Portion), C(Failing)],
            first,
        ),
        rule(
            "en.ask_again.whether_failing",
            AskAgain,
            vec![C(Whether), C(Portion), C(Failing)],
            first,
        ),
        // "If not, I'd like to know which ones are failing.": asked again
        // on a condition, or framed.
        rule(
            "en.ask_again.remark",
            AskAgain,
            vec![C(Remark), C(Break), C(AskAgain)],
            last,
        ),
        rule(
            "en.ask_again.prompt",
            AskAgain,
            vec![C(Prompt), C(AskAgain)],
            last,
        ),
    ]);
    rules
}
