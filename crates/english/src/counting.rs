//! Asking how many things fit a description: "how many functions call
//! `x`?", "number of callers of `x`", "how many callers does `x` have",
//! "`x` caller count", and saying the answer: "3 functions call `x`."

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Condition, Description, Ontology, Position, Question, Reply};

use crate::asking::role_of;
use crate::blank;
use crate::first;
use crate::lexicon::NUMBER;
use crate::locate::fillers;
use crate::phrase;
use crate::sem::{Cat, Number, Sem, VerbForm};
use crate::word;

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 3;

/// Asks how many things the children describe: a description, a kind and
/// a condition, a role and the other argument, or a count question as it is.
fn count<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let description = match c {
        [Sem::Description(d)] | [Sem::Question(Question::Count(d))] => d.clone(),
        [Sem::Kind { kind, .. }, Sem::Condition { condition, .. }] => Description {
            kind: Some(kind.clone()),
            conditions: vec![condition.clone()],
        },
        [role @ Sem::Role { .. }, other]
        | [other, role @ Sem::Role { .. }, ..]
        | [other, .., role @ Sem::Role { .. }] => match role_of(role, other)? {
            Sem::Description(d) => d,
            _ => return None,
        },
        _ => return None,
    };
    Some(Sem::Question(Question::Count(description)))
}

/// The kind and the condition on the first argument of a description of
/// things of a kind, as a verb phrase speaks it, agreeing with `number`.
fn kind_and_condition<O: Ontology>(d: &Description<O>, number: Number) -> Option<Vec<Sem<O>>> {
    match d.conditions.as_slice() {
        [
            condition @ Condition::Related {
                position: Position::First,
                ..
            },
        ] => Some(vec![
            Sem::Kind {
                kind: d.kind.clone()?,
                number,
            },
            Sem::Condition {
                condition: condition.clone(),
                number,
            },
        ]),
        _ => None,
    }
}

/// The description among the children (a noun and what restricts it), if
/// it describes things to count.
fn described<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match count(c)? {
        Sem::Question(Question::Count(d)) => Some(Sem::Description(d)),
        _ => None,
    }
}

/// Words before a participle that keep its meaning, and the round each
/// came in: "are calling `x`", "end up calling `x`", "ends up calling `x`",
/// "anything that ends up calling it".
const ASPECTS: &[(&str, u32, &str)] = &[
    ("en.verb_phrase.are_participial", ROUND, "are"),
    ("en.verb_phrase.is_participial", ROUND, "is"),
    ("en.verb_phrase.end_up", ROUND, "end up"),
    ("en.verb_phrase.ends_up", 9, "ends up"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let mut rules = vec![
        // "how many functions call `x`": how the system asks it, too.
        Rule::new(
            RuleInfo::new("en.question.how_many", ROUND),
            Cat::Question,
            vec![W("how"), W("many"), C(Cat::Noun), C(Cat::VerbPhrase)],
            S::Both(count, |v| match v {
                Sem::Question(Question::Count(d)) => kind_and_condition(d, Number::Plural),
                _ => None,
            }),
        ),
        // "how many callers of `x`", "how many fns calling `x`"
        rule(
            "en.question.how_many_described",
            Cat::Question,
            vec![W("how"), W("many"), C(Cat::Description)],
            count,
        ),
        // "how many callers does `x` have"
        rule(
            "en.question.how_many_have",
            Cat::Question,
            vec![
                W("how"),
                W("many"),
                C(Cat::RoleNoun),
                W("does"),
                C(Cat::NounPhrase),
                W("have"),
            ],
            count,
        ),
        // "`x` — how many callers"
        rule(
            "en.question.topic_how_many",
            Cat::Question,
            vec![
                C(Cat::NounPhrase),
                C(Cat::Dash),
                W("how"),
                W("many"),
                C(Cat::RoleNoun),
            ],
            count,
        ),
        // "number of callers of `x`", "count of fns calling `x`", "# of callers for `x`"
        rule(
            "en.question.amount_of",
            Cat::Question,
            vec![C(Cat::Amount), W("of"), C(Cat::Description)],
            |c| count(c.get(1..)?),
        ),
        // "`x` caller count"
        rule(
            "en.question.role_amount",
            Cat::Question,
            vec![C(Cat::NounPhrase), C(Cat::RoleNoun), C(Cat::Amount)],
            count,
        ),
        // "count callers of `x`", "can you count how many functions call `x`"
        rule(
            "en.question.count_described",
            Cat::Question,
            vec![W("count"), C(Cat::Description)],
            count,
        ),
        // A count question as it is.
        rule(
            "en.question.count_how_many",
            Cat::Question,
            vec![W("count"), C(Cat::Question)],
            count,
        ),
        // "can you count how many functions call `x` for me?"
        rule(
            "en.question.for_me",
            Cat::Question,
            vec![C(Cat::Question), W("for"), W("me")],
            first,
        ),
        // "fns calling `x`", "funcs that call `x`"
        rule(
            "en.description.participial",
            Cat::Description,
            vec![C(Cat::Noun), C(Cat::Participial)],
            described,
        ),
        rule(
            "en.description.that_verb_phrase",
            Cat::Description,
            vec![C(Cat::Noun), W("that"), C(Cat::VerbPhrase)],
            described,
        ),
        // "calling `x`"
        rule(
            "en.participial.object",
            Cat::Participial,
            vec![C(Cat::Verb), C(Cat::NounPhrase)],
            |c: &[Sem<O>]| match c {
                [
                    Sem::Relation {
                        relation,
                        form: VerbForm::Progressive,
                    },
                    Sem::Thing(other),
                ] => Some(Sem::Condition {
                    condition: Condition::Related {
                        relation: relation.clone(),
                        position: Position::First,
                        other: other.clone(),
                    },
                    number: Number::Plural,
                }),
                _ => None,
            },
        ),
        // "number": how the system says it, too.
        word(
            RuleInfo::new("en.amount.number", ROUND),
            Cat::Amount,
            "number",
            blank(),
        ),
    ];
    // Nouns for how many things there are, and dashes setting a topic
    // before a question about it.
    rules.extend(fillers(
        ROUND,
        &[
            (
                Cat::Amount,
                &[("en.amount.count", "count"), ("en.amount.hash", "#")],
            ),
            (Cat::Dash, &[("en.dash.em", "—"), ("en.dash.hyphen", "-")]),
        ],
    ));
    rules.extend([
        // "3 functions call `x`.", "1 function calls `x`."
        Rule::new(
            RuleInfo::new("en.reply.count", ROUND),
            Cat::Reply,
            vec![Class(NUMBER), C(Cat::Noun), C(Cat::VerbPhrase), W(".")],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Count { description, count }) => {
                        let number = if *count == 1 {
                            Number::Singular
                        } else {
                            Number::Plural
                        };
                        Some(
                            [
                                vec![Sem::Count(*count)],
                                kind_and_condition(description, number)?,
                            ]
                            .concat(),
                        )
                    }
                    _ => None,
                },
            ),
        ),
        // "the number is 3.", when the things counted have no kind.
        Rule::new(
            RuleInfo::new("en.reply.count_only", ROUND),
            Cat::Reply,
            vec![W("the"), C(Cat::Amount), W("is"), Class(NUMBER), W(".")],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Count { count, .. }) => {
                        Some(vec![blank(), Sem::Count(*count)])
                    }
                    _ => None,
                },
            ),
        ),
    ]);
    rules.extend(ASPECTS.iter().map(|&(id, round, before)| {
        let mut rhs = phrase(before);
        rhs.push(C(Cat::Participial));
        Rule::new(
            RuleInfo::new(id, round),
            Cat::VerbPhrase,
            rhs,
            S::Build(first),
        )
        .read_only()
    }));
    rules
}
