//! Asking for the objects of a relation: "what does `x` call?", "which
//! functions does `x` call?", "fns called by `x`", "callees of `x`", and
//! saying the answer: "`x` calls `a` and `b`."
//!
//! The thing asked about is the relation's second argument, so the clause
//! that restricts it has a gap where its object would be (a [`Cat::Restriction`]).

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Condition, Description, Extent, Ontology, Position, Question, Thing};

use crate::sem::{Cat, Number, Sem, VerbForm};

use Cat::{ItemList, Noun, NounPhrase, Passive, Restriction, RoleNoun, Statement, Verb};
use Sym::{Cat as C, Word as W};

const ROUND: u32 = 2;

/// A rule's right-hand side: the words of a phrase ("from inside"), then
/// categories.
pub(crate) fn words_then(words: &'static str, cats: &[Cat]) -> Vec<Sym<Cat>> {
    words
        .split_whitespace()
        .map(W)
        .chain(cats.iter().copied().map(C))
        .collect()
}

/// A rule the system reads but never says, building its value with `build`.
pub(crate) fn read_only_rule<O: Ontology>(
    id: &'static str,
    round: u32,
    lhs: Cat,
    rhs: Vec<Sym<Cat>>,
    build: fn(&[Sem<O>]) -> Option<Sem<O>>,
) -> Rule<Cat, Sem<O>> {
    Rule::new(RuleInfo::new(id, round), lhs, rhs, S::Build(build)).read_only()
}

/// The condition that the thing described is the `position` argument of
/// `relation`, with `other` as the other argument.
pub(crate) fn related<O: Ontology>(
    relation: &O::Relation,
    position: Position,
    other: &Thing<O>,
) -> Condition<O> {
    Condition::Related {
        relation: relation.clone(),
        position,
        other: other.clone(),
    }
}

/// Things of a kind (or of any kind) meeting one condition.
pub(crate) fn meeting<O: Ontology>(
    kind: Option<&O::Kind>,
    condition: Condition<O>,
) -> Description<O> {
    Description {
        kind: kind.cloned(),
        conditions: vec![condition],
    }
}

/// The one condition of a description, if it is on the second argument of
/// a relation ("`x` calls …"), as the value of a restriction.
fn object_condition<O: Ontology>(d: &Description<O>) -> Option<Sem<O>> {
    match d.conditions.as_slice() {
        [
            condition @ Condition::Related {
                position: Position::Second,
                ..
            },
        ] => Some(Sem::Condition {
            condition: condition.clone(),
            number: Number::Plural,
        }),
        _ => None,
    }
}

/// The subject and verb (in the given form) that say a restriction's
/// condition on the second argument.
fn subject_verb<O: Ontology>(v: &Sem<O>, form: VerbForm) -> Option<Vec<Sem<O>>> {
    match v {
        Sem::Condition {
            condition:
                Condition::Related {
                    relation,
                    position: Position::Second,
                    other,
                },
            ..
        } => Some(vec![
            Sem::Thing(other.clone()),
            Sem::Relation {
                relation: relation.clone(),
                form,
            },
        ]),
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
            Some(Sem::Condition {
                condition: related(relation, Position::Second, other),
                number: Number::Plural,
            })
        }
        _ => None,
    }
}

/// Things of a kind (or of any kind) meeting one condition.
fn described<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let (kind, condition) = match c {
        [Sem::Kind { kind, .. }, Sem::Condition { condition, .. }] => (Some(kind), condition),
        [Sem::Condition { condition, .. }] => (None, condition),
        _ => return None,
    };
    Some(Sem::Description(meeting(kind, condition.clone())))
}

/// Asks to identify what a description describes, or the things (of a
/// kind) meeting a condition: "which structs have …", "who has …".
pub(crate) fn identify<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Description(d)] => Some(Sem::Question(Question::Identify(d.clone()))),
        _ => identify(&[described(c)?]),
    }
}

/// "callees of `x`": the things filling one side of a relation.
pub(crate) fn role_of<O: Ontology>(role: &Sem<O>, other: &Sem<O>) -> Option<Sem<O>> {
    match (role, other) {
        (Sem::Role { relation, position }, Sem::Thing(other)) => Some(Sem::Description(meeting(
            None,
            related(relation, *position, other),
        ))),
        _ => None,
    }
}

/// [`role_of`] for a role noun and a thing in either order: "callees of
/// `x`", "`x` callees".
fn role_and_thing<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [role @ Sem::Role { .. }, other] | [other, role @ Sem::Role { .. }] => role_of(role, other),
        _ => None,
    }
}

/// A rule passing its first child's value up unchanged.
pub(crate) fn same<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.first().cloned()
}

/// Words before the doer of a passive: "called by `x`", "called from
/// inside `x`", "called in `x`".
const AGENTS: &[(&str, &str)] = &[
    ("en.passive.by", "by"),
    ("en.passive.from", "from"),
    ("en.passive.in", "in"),
    ("en.passive.inside", "inside"),
    ("en.passive.from_inside", "from inside"),
];

/// Words after a verb that say the calls happen inside the subject: "what
/// does `x` call inside", "what `x` calls into".
const INSIDE: &[(&str, &str)] = &[
    ("en.restriction.into", "into"),
    ("en.restriction.inside", "inside"),
];

/// Words between a role noun and the other argument: "callees of `x`",
/// "callees for `x`".
const ROLE_OF: &[(&str, &str)] = &[
    ("en.description.role_of", "of"),
    ("en.description.role_for", "for"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| read_only_rule(id, ROUND, lhs, rhs, build);
    let mut rules: Vec<Rule<Cat, Sem<O>>> = vec![
        // "does `x` call": the question order, which the system also speaks.
        Rule::new(
            RuleInfo::new("en.restriction.does", ROUND),
            Restriction,
            words_then("does", &[NounPhrase, Verb]),
            S::Both(
                |c| restrict(c, VerbForm::Plain),
                |v| subject_verb(v, VerbForm::Plain),
            ),
        ),
        // "`x` calls", in embedded and casual questions: "which functions
        // `x` calls", "`x` calls what".
        rule(
            "en.restriction.declarative",
            Restriction,
            vec![C(NounPhrase), C(Verb)],
            |c| restrict(c, VerbForm::ThirdSingular),
        ),
        // "the functions that `x` calls".
        rule(
            "en.restriction.that",
            Restriction,
            words_then("that", &[Restriction]),
            same,
        ),
        // "fns called by `x`".
        rule(
            "en.restriction.passive",
            Restriction,
            vec![C(Passive)],
            same,
        ),
        // "what fields has `x` got": the perfect, with "got" as its verb.
        read_only_rule(
            "en.restriction.has",
            7,
            Restriction,
            words_then("has", &[NounPhrase, Verb]),
            |c| restrict(c, VerbForm::Plain),
        ),
        // "which functions is `x` calling"
        rule(
            "en.restriction.progressive",
            Restriction,
            words_then("is", &[NounPhrase, Verb]),
            |c| restrict(c, VerbForm::Progressive),
        ),
        // "which functions are called by `x`", "what functions get called from `x`".
        rule(
            "en.restriction.are_passive",
            Restriction,
            words_then("are", &[Passive]),
            same,
        ),
        rule(
            "en.restriction.get_passive",
            Restriction,
            words_then("get", &[Passive]),
            same,
        ),
        rule(
            "en.restriction.gets_passive",
            Restriction,
            words_then("gets", &[Passive]),
            same,
        ),
        // "what does `x` call"
        Rule::new(
            RuleInfo::new("en.question.what_restricted", ROUND),
            Cat::Question,
            words_then("what", &[Restriction]),
            S::Both(identify, |v| match v {
                Sem::Question(Question::Identify(d)) if d.kind.is_none() => {
                    Some(vec![object_condition(d)?])
                }
                _ => None,
            }),
        ),
        // "which functions does `x` call"
        Rule::new(
            RuleInfo::new("en.question.which_restricted", ROUND),
            Cat::Question,
            words_then("which", &[Noun, Restriction]),
            S::Both(identify, |v| match v {
                Sem::Question(Question::Identify(d)) => Some(vec![
                    Sem::Kind {
                        kind: d.kind.clone()?,
                        number: Number::Plural,
                    },
                    object_condition(d)?,
                ]),
                _ => None,
            }),
        ),
        // "what functions does `x` call"
        rule(
            "en.question.what_kind_restricted",
            Cat::Question,
            words_then("what", &[Noun, Restriction]),
            identify,
        ),
        // "`x` calls what", "`x` calls which fns": the question word in the
        // object's place.
        rule(
            "en.question.echo_what",
            Cat::Question,
            vec![C(Restriction), W("what")],
            identify,
        ),
        rule(
            "en.question.echo_which",
            Cat::Question,
            vec![C(Restriction), W("which"), C(Noun)],
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
            vec![C(Noun), C(Restriction)],
            described,
        ),
        // "the functions that `x` calls"
        rule(
            "en.description.the",
            Cat::Description,
            words_then("the", &[Cat::Description]),
            same,
        ),
        // "`pick_row` callees"
        rule(
            "en.description.name_role",
            Cat::Description,
            vec![C(NounPhrase), C(RoleNoun)],
            role_and_thing,
        ),
        // "`x` calls `a` and `b`."
        Rule::new(
            RuleInfo::new("en.statement.object_list", ROUND),
            Statement,
            vec![C(NounPhrase), C(Verb), C(ItemList)],
            S::Both(
                |c| match c {
                    [
                        Sem::Thing(other),
                        Sem::Relation { relation, .. },
                        Sem::Items(members),
                    ] => Some(Sem::Extent(Extent {
                        description: meeting(None, related(relation, Position::Second, other)),
                        members: members.clone(),
                    })),
                    _ => None,
                },
                |v| match v {
                    Sem::Extent(e) if !e.members.is_empty() => {
                        let mut said = subject_verb(
                            &object_condition(&e.description)?,
                            VerbForm::ThirdSingular,
                        )?;
                        said.push(Sem::Items(e.members.clone()));
                        Some(said)
                    }
                    _ => None,
                },
            ),
        ),
        // "`x` calls nothing."
        Rule::new(
            RuleInfo::new("en.statement.object_nothing", ROUND),
            Statement,
            vec![C(NounPhrase), C(Verb), W("nothing")],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Extent(e) if e.members.is_empty() => {
                        subject_verb(&object_condition(&e.description)?, VerbForm::ThirdSingular)
                    }
                    _ => None,
                },
            ),
        ),
    ];
    rules.extend(AGENTS.iter().map(|(id, words)| {
        let mut rhs = vec![C(Verb)];
        rhs.extend(words_then(words, &[NounPhrase]));
        rule(id, Passive, rhs, |c| restrict(c, VerbForm::Participle))
    }));
    rules.extend(
        INSIDE
            .iter()
            .map(|(id, word)| rule(id, Restriction, vec![C(Restriction), W(word)], same)),
    );
    rules.extend(ROLE_OF.iter().map(|(id, word)| {
        rule(
            id,
            Cat::Description,
            vec![C(RoleNoun), W(word), C(NounPhrase)],
            role_and_thing,
        )
    }));
    rules
}
