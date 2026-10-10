//! Asking how many things fit a description: "how many functions call
//! `x`?", "number of callers of `x`", "how many callers does `x` have",
//! "`x` caller count", and saying the answer: "3 functions call `x`."

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Condition, Description, Ontology, Position, Question, Reply};

use crate::asking::role_of;
use crate::lexicon::NUMBER;
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

/// Words before a participle that keep its meaning: "are calling `x`",
/// "end up calling `x`".
const ASPECTS: &[(&str, &[&str])] = &[
    ("en.verb_phrase.are_participial", &["are"]),
    ("en.verb_phrase.is_participial", &["is"]),
    ("en.verb_phrase.end_up", &["end", "up"]),
    ("en.verb_phrase.ends_up", &["ends", "up"]),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let words = |id, cat, words: &[&'static str]| {
        word(
            RuleInfo::new(id, ROUND),
            cat,
            words,
            Sem::Quote(String::new()),
        )
        .read_only()
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
        rule(
            "en.question.count_how_many",
            Cat::Question,
            vec![W("count"), C(Cat::Question)],
            |c| match c {
                [q @ Sem::Question(Question::Count(_))] => Some(q.clone()),
                _ => None,
            },
        ),
        // "can you count how many functions call `x` for me?"
        rule(
            "en.question.for_me",
            Cat::Question,
            vec![C(Cat::Question), W("for"), W("me")],
            |c| c.first().cloned(),
        ),
        // "fns calling `x`", "funcs that call `x`"
        rule(
            "en.description.participial",
            Cat::Description,
            vec![C(Cat::Noun), C(Cat::Participial)],
            |c| match count(c)? {
                Sem::Question(Question::Count(d)) => Some(Sem::Description(d)),
                _ => None,
            },
        ),
        rule(
            "en.description.that_verb_phrase",
            Cat::Description,
            vec![C(Cat::Noun), W("that"), C(Cat::VerbPhrase)],
            |c| match count(c)? {
                Sem::Question(Question::Count(d)) => Some(Sem::Description(d)),
                _ => None,
            },
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
        word(
            RuleInfo::new("en.amount.number", ROUND),
            Cat::Amount,
            &["number"],
            Sem::Quote(String::new()),
        ),
        words("en.amount.count", Cat::Amount, &["count"]),
        words("en.amount.hash", Cat::Amount, &["#"]),
        words("en.dash.em", Cat::Dash, &["—"]),
        words("en.dash.en", Cat::Dash, &["–"]),
        words("en.dash.hyphen", Cat::Dash, &["-"]),
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
                        let mut c = vec![Sem::Count(*count)];
                        c.extend(kind_and_condition(description, number)?);
                        Some(c)
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
                        Some(vec![Sem::Quote(String::new()), Sem::Count(*count)])
                    }
                    _ => None,
                },
            ),
        ),
    ];
    rules.extend(ASPECTS.iter().map(|(id, before)| {
        let mut rhs: Vec<_> = before.iter().map(|w| W(w)).collect();
        rhs.push(C(Cat::Participial));
        rule(id, Cat::VerbPhrase, rhs, |c: &[Sem<O>]| c.first().cloned())
    }));
    rules
}
