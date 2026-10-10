//! Asking for the objects of a relation: "what does `x` call?", "which
//! functions does `x` call?", "fns called by `x`", "callees of `x`", and
//! saying the answer: "`x` calls `a` and `b`."
//!
//! The thing asked about is the relation's second argument, so the clause
//! that restricts it has a gap where its object would be (a [`Cat::Restriction`]).

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Condition, Description, Extent, Ontology, Position, Question, Thing};

use crate::sem::{Cat, Number, Sem, VerbForm};

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 2;

/// "`x` calls …": the condition that `other` stands in `relation` to the
/// thing asked about.
fn object_of<O: Ontology>(relation: &O::Relation, other: &Thing<O>) -> Sem<O> {
    Sem::Condition {
        condition: Condition::Related {
            relation: relation.clone(),
            position: Position::Second,
            other: other.clone(),
        },
        number: Number::Plural,
    }
}

/// The relation and other argument of a condition on the second argument.
fn unobject<O: Ontology>(c: &Condition<O>) -> Option<(O::Relation, Thing<O>)> {
    match c {
        Condition::Related {
            relation,
            position: Position::Second,
            other,
        } => Some((relation.clone(), other.clone())),
        _ => None,
    }
}

/// The relation and other argument of a description by one condition on
/// the second argument.
fn described_object<O: Ontology>(d: &Description<O>) -> Option<(O::Relation, Thing<O>)> {
    match d.conditions.as_slice() {
        [c] => unobject(c),
        _ => None,
    }
}

/// A restriction from a subject and a verb in the given form.
fn restrict<O: Ontology>(c: &[Sem<O>], wanted: VerbForm) -> Option<Sem<O>> {
    match c {
        [Sem::Thing(other), Sem::Relation { relation, form }]
        | [Sem::Relation { relation, form }, Sem::Thing(other)]
            if *form == wanted =>
        {
            Some(object_of(relation, other))
        }
        _ => None,
    }
}

/// Things of a kind (or of any kind) meeting one condition.
fn described<O: Ontology>(kind: Option<&O::Kind>, condition: &Sem<O>) -> Option<Sem<O>> {
    match condition {
        Sem::Condition { condition, .. } => Some(Sem::Description(Description {
            kind: kind.cloned(),
            conditions: vec![condition.clone()],
        })),
        _ => None,
    }
}

/// Asks to identify what a description describes.
fn identify<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Description(d)] => Some(Sem::Question(Question::Identify(d.clone()))),
        [Sem::Kind { kind, .. }, condition] => identify(&[described(Some(kind), condition)?]),
        [condition] => identify(&[described(None, condition)?]),
        _ => None,
    }
}

/// "callees of `x`": the things filling one side of a relation.
pub(crate) fn role_of<O: Ontology>(role: &Sem<O>, other: &Sem<O>) -> Option<Sem<O>> {
    match (role, other) {
        (Sem::Role { relation, position }, Sem::Thing(other)) => {
            Some(Sem::Description(Description {
                kind: None,
                conditions: vec![Condition::Related {
                    relation: relation.clone(),
                    position: *position,
                    other: other.clone(),
                }],
            }))
        }
        _ => None,
    }
}

/// A rule passing its one child's value up unchanged.
fn same<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.first().cloned()
}

/// Words before the doer of a passive: "called by `x`", "called from
/// inside `x`", "called in `x`".
const AGENTS: &[(&str, &[&str])] = &[
    ("en.passive.by", &["by"]),
    ("en.passive.from", &["from"]),
    ("en.passive.in", &["in"]),
    ("en.passive.inside", &["inside"]),
    ("en.passive.from_inside", &["from", "inside"]),
    ("en.passive.within", &["within"]),
];

/// Words after a verb that say the calls happen inside the subject: "what
/// does `x` call inside", "what `x` calls into".
const INSIDE: &[(&str, &str)] = &[
    ("en.restriction.into", "into"),
    ("en.restriction.inside", "inside"),
    ("en.restriction.internally", "internally"),
];

/// Words between a role noun and the other argument: "callees of `x`",
/// "callees for `x`".
const ROLE_OF: &[(&str, &str)] = &[
    ("en.description.role_of", "of"),
    ("en.description.role_for", "for"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let mut rules: Vec<Rule<Cat, Sem<O>>> = vec![
        // "does `x` call": the question order, which the system also speaks.
        Rule::new(
            RuleInfo::new("en.restriction.does", ROUND),
            Cat::Restriction,
            vec![W("does"), C(Cat::NounPhrase), C(Cat::Verb)],
            S::Both(
                |c| restrict(c, VerbForm::Plain),
                |v| {
                    let (relation, other) = match v {
                        Sem::Condition { condition, .. } => unobject(condition)?,
                        _ => return None,
                    };
                    Some(vec![
                        Sem::Thing(other),
                        Sem::Relation {
                            relation,
                            form: VerbForm::Plain,
                        },
                    ])
                },
            ),
        ),
        // "`x` calls", in embedded and casual questions: "which functions
        // `x` calls", "`x` calls what".
        rule(
            "en.restriction.declarative",
            Cat::Restriction,
            vec![C(Cat::NounPhrase), C(Cat::Verb)],
            |c| restrict(c, VerbForm::ThirdSingular),
        ),
        // "the functions that `x` calls".
        rule(
            "en.restriction.that",
            Cat::Restriction,
            vec![W("that"), C(Cat::Restriction)],
            same,
        ),
        // "fns called by `x`".
        rule(
            "en.restriction.passive",
            Cat::Restriction,
            vec![C(Cat::Passive)],
            same,
        ),
        // "which functions is `x` calling"
        rule(
            "en.restriction.progressive",
            Cat::Restriction,
            vec![W("is"), C(Cat::NounPhrase), C(Cat::Verb)],
            |c| restrict(c, VerbForm::Progressive),
        ),
        // "which functions are called by `x`", "what functions get called from `x`".
        rule(
            "en.restriction.are_passive",
            Cat::Restriction,
            vec![W("are"), C(Cat::Passive)],
            same,
        ),
        rule(
            "en.restriction.get_passive",
            Cat::Restriction,
            vec![W("get"), C(Cat::Passive)],
            same,
        ),
        rule(
            "en.restriction.gets_passive",
            Cat::Restriction,
            vec![W("gets"), C(Cat::Passive)],
            same,
        ),
        // "what does `x` call"
        Rule::new(
            RuleInfo::new("en.question.what_restricted", ROUND),
            Cat::Question,
            vec![W("what"), C(Cat::Restriction)],
            S::Both(identify, |v| match v {
                Sem::Question(Question::Identify(d)) if d.kind.is_none() => {
                    let (relation, other) = described_object(d)?;
                    Some(vec![object_of(&relation, &other)])
                }
                _ => None,
            }),
        ),
        // "which functions does `x` call"
        Rule::new(
            RuleInfo::new("en.question.which_restricted", ROUND),
            Cat::Question,
            vec![W("which"), C(Cat::Noun), C(Cat::Restriction)],
            S::Both(identify, |v| match v {
                Sem::Question(Question::Identify(d)) => {
                    let kind = d.kind.clone()?;
                    let (relation, other) = described_object(d)?;
                    Some(vec![
                        Sem::Kind {
                            kind,
                            number: Number::Plural,
                        },
                        object_of(&relation, &other),
                    ])
                }
                _ => None,
            }),
        ),
        // "what functions does `x` call"
        rule(
            "en.question.what_kind_restricted",
            Cat::Question,
            vec![W("what"), C(Cat::Noun), C(Cat::Restriction)],
            identify,
        ),
        // "`x` calls what", "`x` calls which fns": the question word in the
        // object's place.
        rule(
            "en.question.echo_what",
            Cat::Question,
            vec![C(Cat::Restriction), W("what")],
            identify,
        ),
        rule(
            "en.question.echo_which",
            Cat::Question,
            vec![C(Cat::Restriction), W("which"), C(Cat::Noun)],
            |c| match c {
                [condition, kind] => identify(&[kind.clone(), condition.clone()]),
                _ => None,
            },
        ),
        // A description alone asks for what it describes: "fns called by `x`".
        rule(
            "en.question.description",
            Cat::Question,
            vec![C(Cat::Description)],
            identify,
        ),
        // "fns called by `x`", "funcs `x` calls", "functions that `x` calls".
        rule(
            "en.description.restricted",
            Cat::Description,
            vec![C(Cat::Noun), C(Cat::Restriction)],
            |c| match c {
                [Sem::Kind { kind, .. }, condition] => described(Some(kind), condition),
                _ => None,
            },
        ),
        // "the functions that `x` calls"
        rule(
            "en.description.the",
            Cat::Description,
            vec![W("the"), C(Cat::Description)],
            same,
        ),
        // "`pick_row` callees"
        rule(
            "en.description.name_role",
            Cat::Description,
            vec![C(Cat::NounPhrase), C(Cat::RoleNoun)],
            |c| match c {
                [other, role] => role_of(role, other),
                _ => None,
            },
        ),
        // "`x` calls `a` and `b`."
        Rule::new(
            RuleInfo::new("en.statement.object_list", ROUND),
            Cat::Statement,
            vec![C(Cat::NounPhrase), C(Cat::Verb), C(Cat::ItemList)],
            S::Both(
                |c| match c {
                    [
                        Sem::Thing(other),
                        Sem::Relation { relation, .. },
                        Sem::Items(members),
                    ] => Some(Sem::Extent(Extent {
                        description: Description {
                            kind: None,
                            conditions: vec![Condition::Related {
                                relation: relation.clone(),
                                position: Position::Second,
                                other: other.clone(),
                            }],
                        },
                        members: members.clone(),
                    })),
                    _ => None,
                },
                |v| match v {
                    Sem::Extent(e) if !e.members.is_empty() => {
                        let (relation, other) = described_object(&e.description)?;
                        Some(vec![
                            Sem::Thing(other),
                            Sem::Relation {
                                relation,
                                form: VerbForm::ThirdSingular,
                            },
                            Sem::Items(e.members.clone()),
                        ])
                    }
                    _ => None,
                },
            ),
        ),
        // "`x` calls nothing."
        Rule::new(
            RuleInfo::new("en.statement.object_nothing", ROUND),
            Cat::Statement,
            vec![C(Cat::NounPhrase), C(Cat::Verb), W("nothing")],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Extent(e) if e.members.is_empty() => {
                        let (relation, other) = described_object(&e.description)?;
                        Some(vec![
                            Sem::Thing(other),
                            Sem::Relation {
                                relation,
                                form: VerbForm::ThirdSingular,
                            },
                        ])
                    }
                    _ => None,
                },
            ),
        ),
    ];
    rules.extend(AGENTS.iter().map(|(id, words)| {
        let mut rhs = vec![C(Cat::Verb)];
        rhs.extend(words.iter().map(|w| W(w)));
        rhs.push(C(Cat::NounPhrase));
        rule(id, Cat::Passive, rhs, |c: &[Sem<O>]| {
            restrict(c, VerbForm::Participle)
        })
    }));
    rules.extend(INSIDE.iter().map(|(id, word)| {
        rule(
            id,
            Cat::Restriction,
            vec![C(Cat::Restriction), W(word)],
            same,
        )
    }));
    rules.extend(ROLE_OF.iter().map(|(id, word)| {
        rule(
            id,
            Cat::Description,
            vec![C(Cat::RoleNoun), W(word), C(Cat::NounPhrase)],
            |c: &[Sem<O>]| match c {
                [role, other] => role_of(role, other),
                _ => None,
            },
        )
    }));
    rules
}
