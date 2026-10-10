//! Asking for the objects of a relation: "what does `x` call?", "which
//! functions does `x` call?", "fns called by `x`", "callees of `x`", and
//! saying the answer: "`x` calls `a` and `b`."
//!
//! The thing asked about is the relation's second argument, so the clause
//! that restricts it has a gap where its object would be (a [`Cat::Restriction`]).

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Condition, Description, Extent, Ontology, Position, Question, Thing};

use crate::first;
use crate::sem::{Cat, Number, Sem, VerbForm};
use crate::spoken;

use Cat::*;
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
pub(crate) fn described<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
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

/// Words before the doer of a passive: "called by `x`", "called from
/// inside `x`", "called in `x`".
const AGENTS: &[(&str, &str)] = &[
    ("en.passive.by", "by"),
    ("en.passive.from", "from"),
    ("en.passive.in", "in"),
    ("en.passive.inside", "inside"),
    ("en.passive.from_inside", "from inside"),
];

/// Words before a passive, and the round each came in: "which functions
/// are called by `x`", "what functions get called from `x`", "what
/// functions are getting called from `x`".
const PASSIVE_AUX: &[(&str, u32, &str)] = &[
    ("en.restriction.are_passive", ROUND, "are"),
    ("en.restriction.get_passive", ROUND, "get"),
    ("en.restriction.gets_passive", ROUND, "gets"),
    ("en.restriction.are_getting_passive", 13, "are getting"),
];

/// Words between a role noun and the other argument: "callees of `x`",
/// "callees for `x`".
const ROLE_OF: &[(&str, u32, &str)] = &[
    ("en.description.role_of", ROUND, "of"),
    ("en.description.role_for", ROUND, "for"),
    // "calls made by `x`", "calls made inside `x`"
    ("en.description.role_by", 13, "by"),
    ("en.description.role_inside", 13, "inside"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| read_only_rule(id, ROUND, lhs, rhs, build);
    let mut rules: Vec<Rule<Cat, Sem<O>>> = vec![
        // "does `x` call": the question order, which the system also speaks.
        spoken(
            "en.restriction.does",
            ROUND,
            Restriction,
            words_then("does", &[NounPhrase, Verb]),
            |c| restrict(c, VerbForm::Plain),
            |v| subject_verb(v, VerbForm::Plain),
        ),
        // "`x` calls", in embedded and casual questions: "which functions
        // `x` calls", "`x` calls what".
        rule(
            "en.restriction.declarative",
            Restriction,
            vec![C(NounPhrase), C(Verb)],
            |c| restrict(c, VerbForm::ThirdSingular),
        ),
        // "fns called by `x`".
        rule(
            "en.restriction.passive",
            Restriction,
            vec![C(Passive)],
            first,
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
        // "what does `x` call"
        spoken(
            "en.question.what_restricted",
            ROUND,
            Question,
            words_then("what", &[Restriction]),
            identify,
            |v| match v {
                Sem::Question(Question::Identify(d)) if d.kind.is_none() => {
                    Some(vec![object_condition(d)?])
                }
                _ => None,
            },
        ),
        // "which functions does `x` call"
        spoken(
            "en.question.which_restricted",
            ROUND,
            Question,
            words_then("which", &[Noun, Restriction]),
            identify,
            |v| match v {
                Sem::Question(Question::Identify(d)) => Some(vec![
                    Sem::Kind {
                        kind: d.kind.clone()?,
                        number: Number::Plural,
                    },
                    object_condition(d)?,
                ]),
                _ => None,
            },
        ),
        // "what functions does `x` call"
        rule(
            "en.question.what_kind_restricted",
            Question,
            words_then("what", &[Noun, Restriction]),
            identify,
        ),
        // "`x` calls what", "`x` calls which fns": the question word in the
        // object's place.
        rule(
            "en.question.echo_what",
            Question,
            vec![C(Restriction), W("what")],
            identify,
        ),
        rule(
            "en.question.echo_which",
            Question,
            vec![C(Restriction), W("which"), C(Noun)],
            |c| match c {
                [condition, kind] => identify(&[kind.clone(), condition.clone()]),
                _ => None,
            },
        ),
        // A description alone asks for what it describes: "fns called by `x`".
        rule(
            "en.question.description",
            Question,
            vec![C(Description)],
            identify,
        ),
        // "fns called by `x`", "funcs `x` calls", "functions that `x` calls".
        rule(
            "en.description.restricted",
            Description,
            vec![C(Noun), C(Restriction)],
            described,
        ),
        // "`pick_row` callees"
        rule(
            "en.description.name_role",
            Description,
            vec![C(NounPhrase), C(RoleNoun)],
            role_and_thing,
        ),
        // "Stock's fields", "`x`'s callers"
        read_only_rule(
            "en.description.possessive_role",
            21,
            Description,
            vec![C(NounPhrase), W("'s"), C(RoleNoun)],
            role_and_thing,
        ),
        // "`x` calls `a` and `b`."
        spoken(
            "en.statement.object_list",
            ROUND,
            Statement,
            vec![C(NounPhrase), C(Verb), C(ItemList)],
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
                    let mut said =
                        subject_verb(&object_condition(&e.description)?, VerbForm::ThirdSingular)?;
                    said.push(Sem::Items(e.members.clone()));
                    Some(said)
                }
                _ => None,
            },
        ),
        // "`x` calls nothing."
        spoken(
            "en.statement.object_nothing",
            ROUND,
            Statement,
            vec![C(NounPhrase), C(Verb), W("nothing")],
            |_| None,
            |v| match v {
                Sem::Extent(e) if e.members.is_empty() => {
                    subject_verb(&object_condition(&e.description)?, VerbForm::ThirdSingular)
                }
                _ => None,
            },
        ),
    ];
    rules.extend(AGENTS.iter().map(|(id, words)| {
        let rhs = vec![C(Verb), W(words), C(NounPhrase)];
        rule(id, Passive, rhs, |c| restrict(c, VerbForm::Participle))
    }));
    rules.extend(PASSIVE_AUX.iter().map(|&(id, round, words)| {
        read_only_rule(id, round, Restriction, words_then(words, &[Passive]), first)
    }));
    // The subject and the verb in casual and embedded questions: "what fns
    // `x` call", "which functions `x` ends up calling", "which functions
    // `x` is calling", "what does it end up calling?".
    let progressive: fn(&[Sem<O>]) -> Option<Sem<O>> = |c| restrict(c, VerbForm::Progressive);
    for (id, rhs, build) in [
        (
            "en.restriction.plain",
            vec![C(NounPhrase), C(Verb)],
            (|c| restrict(c, VerbForm::Plain)) as fn(&[Sem<O>]) -> Option<Sem<O>>,
        ),
        (
            "en.restriction.declarative_progressive",
            vec![C(NounPhrase), W("is"), C(Verb)],
            progressive,
        ),
        (
            "en.restriction.ends_up",
            vec![C(NounPhrase), W("ends up"), C(Verb)],
            progressive,
        ),
        (
            "en.restriction.does_end_up",
            vec![W("does"), C(NounPhrase), W("end up"), C(Verb)],
            progressive,
        ),
        // "what are the functions it's calling?": "it is" run together.
        ("en.restriction.it_s", vec![W("it's"), C(Verb)], |c| {
            restrict(
                &[Sem::Thing(Thing::Previous), c.first()?.clone()],
                VerbForm::Progressive,
            )
        }),
    ] {
        rules.push(read_only_rule(id, 13, Restriction, rhs, build));
    }
    rules.extend(ROLE_OF.iter().map(|&(id, round, word)| {
        read_only_rule(
            id,
            round,
            Description,
            vec![C(RoleNoun), W(word), C(NounPhrase)],
            role_and_thing,
        )
    }));
    rules.extend([
        // "what's `settings_menu` calling"
        read_only_rule(
            "en.question.what_is_progressive",
            13,
            Question,
            vec![C(WhatIs), C(NounPhrase), C(Verb)],
            |c| identify(&[restrict(c.get(1..)?, VerbForm::Progressive)?]),
        ),
        // "with_query -> callees"
        read_only_rule(
            "en.description.name_arrow_role",
            13,
            Description,
            vec![C(NounPhrase), C(Arrow), C(RoleNoun)],
            |c| match c {
                [other, _, role] => role_of(role, other),
                _ => None,
            },
        ),
    ]);
    rules
}
