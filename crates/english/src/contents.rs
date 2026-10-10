//! Asking what a thing holds: "what fields are on the `Menu` struct?",
//! "what's inside `NewDm`?", "what are the fields of `Week`?", "`Conv`
//! fields", "`S` struct fields", "which fields make up `S`", "what fields
//! has `Lamp` got?", and the same asked about a thing set off before
//! ("`Note` struct - what fields", "I'm looking at `InGrid`, which fields
//! does it have?").
//!
//! The things asked for are often described by where they are, so these
//! shapes build a [`Cat::Description`] and ask for what it describes.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Condition, Description, Ontology, Position, Question, Thing};

use crate::asking::role_of;
use crate::sem::{Cat, Number, Sem, VerbForm};
use crate::word;

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 7;

/// The condition that the described thing fills `position` of the relation
/// among the children, with the thing among them on the other side.
fn related<O: Ontology>(c: &[Sem<O>], position: Position) -> Option<Condition<O>> {
    let relation = c.iter().find_map(|v| match v {
        Sem::Relation { relation, .. } => Some(relation.clone()),
        _ => None,
    })?;
    let other = c.iter().find_map(|v| match v {
        Sem::Thing(t) => Some(t.clone()),
        _ => None,
    })?;
    Some(Condition::Related {
        relation,
        position,
        other,
    })
}

/// Things of a kind standing in a relation to a thing: "fields (are) on
/// `S`", the kind, the relation and the thing among the children.
fn placed_kind<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let kind = c.iter().find_map(|v| match v {
        Sem::Kind { kind, .. } => Some(kind.clone()),
        _ => None,
    })?;
    Some(Sem::Description(Description {
        kind: Some(kind),
        conditions: vec![related(c, Position::First)?],
    }))
}

/// "is on `x`": the subject stands in the preposition's relation to `x`.
fn placed<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    Some(Sem::Condition {
        condition: related(c, Position::First)?,
        number: Number::Plural,
    })
}

/// Asks for every thing fitting the description among the children.
fn identify<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find_map(|v| match v {
        Sem::Description(d) => Some(Sem::Question(Question::Identify(d.clone()))),
        _ => None,
    })
}
/// "what's inside `Bank`, which fields it has": a question for what stands
/// in a relation to a thing, then for those of one kind, about the same
/// thing. The second asks what both ask.
fn narrowed<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let (Some(Sem::Question(Question::Identify(wide))), Some(Sem::Question(narrow))) =
        (c.first(), c.last())
    else {
        return None;
    };
    let [Condition::Related { other, .. }] = wide.conditions.as_slice() else {
        return None;
    };
    match narrow.map_things(&mut |t| match t {
        Thing::Previous => other.clone(),
        t => t.clone(),
    }) {
        Question::Identify(d) if wide.kind.is_none() && d.conditions == wide.conditions => {
            Some(Sem::Question(Question::Identify(d)))
        }
        _ => None,
    }
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let mut rules = vec![
        // "what fields on `memo_data`", "which fields are in `S`": a
        // question word before a description asks for what it describes.
        rule(
            "en.question.what_description",
            Cat::Question,
            vec![W("what"), C(Cat::Description)],
            identify,
        ),
        rule(
            "en.question.which_description",
            Cat::Question,
            vec![W("which"), C(Cat::Description)],
            identify,
        ),
        // "what are the fields of `S`", "what are fields inside `S`",
        // "what's the fields of `S`"
        rule(
            "en.question.what_are_description",
            Cat::Question,
            vec![W("what"), W("are"), C(Cat::Description)],
            identify,
        ),
        rule(
            "en.question.what_is_description",
            Cat::Question,
            vec![C(Cat::WhatIs), C(Cat::Description)],
            identify,
        ),
        // "fields are on `S`", "fields are defined on the `S` struct"
        rule(
            "en.description.are_preposition",
            Cat::Description,
            vec![
                C(Cat::Noun),
                W("are"),
                C(Cat::Preposition),
                C(Cat::NounPhrase),
            ],
            placed_kind,
        ),
        rule(
            "en.description.are_defined_preposition",
            Cat::Description,
            vec![
                C(Cat::Noun),
                W("are"),
                C(Cat::Defined),
                C(Cat::Preposition),
                C(Cat::NounPhrase),
            ],
            placed_kind,
        ),
        // "`S` struct fields", "end_speed fields", "hint_labels struct
        // members": the parts of a thing, named after it.
        rule(
            "en.description.parts",
            Cat::Description,
            vec![C(Cat::NounPhrase), C(Cat::Part)],
            |c: &[Sem<O>]| match c {
                [
                    Sem::Thing(other),
                    Sem::Holder {
                        kind,
                        relation,
                        position,
                    },
                ] => Some(Sem::Description(Description {
                    kind: Some(kind.clone()),
                    conditions: vec![Condition::Related {
                        relation: relation.clone(),
                        position: *position,
                        other: other.clone(),
                    }],
                })),
                _ => None,
            },
        ),
        // "I'd like to know what fields `S` has. Can you list them for me?",
        // "could you list them": asking again for what was just asked.
        rule(
            "en.ask_again.stop",
            Cat::AskAgain,
            vec![W("."), C(Cat::AskAgain)],
            |c| c.first().cloned(),
        ),
        rule(
            "en.ask_again.for_me",
            Cat::AskAgain,
            vec![C(Cat::AskAgain), W("for"), W("me")],
            |c| c.first().cloned(),
        ),
        rule(
            "en.ask_again.could_you_list_them",
            Cat::AskAgain,
            vec![W("could"), W("you"), W("list"), W("them")],
            |_| Some(Sem::Quote(String::new())),
        ),
        // "what's in `RepoId`", "what is inside `Bank`", "what's in it":
        // what the thing holds.
        rule(
            "en.question.what_is_inside",
            Cat::Question,
            vec![C(Cat::WhatIs), C(Cat::Inside), C(Cat::NounPhrase)],
            |c| {
                Some(Sem::Question(Question::Identify(Description {
                    kind: None,
                    conditions: vec![related(c, Position::Second)?],
                })))
            },
        ),
        // "its fields"
        rule(
            "en.description.its",
            Cat::Description,
            vec![W("its"), C(Cat::RoleNoun)],
            |c| role_of(c.first()?, &Sem::Thing(Thing::Previous)),
        ),
        rule(
            "en.question.narrowed",
            Cat::Question,
            vec![C(Cat::Question), C(Cat::Join), C(Cat::Question)],
            narrowed,
        ),
        // "what fields has the `Footer` struct got": the perfect, with the
        // verb's plain form.
        rule(
            "en.restriction.has_got",
            Cat::Restriction,
            vec![W("has"), C(Cat::NounPhrase), C(Cat::Verb)],
            |c| match c {
                [_, Sem::Relation { form, .. }] if *form == VerbForm::Plain => {
                    Some(Sem::Condition {
                        condition: related(c, Position::Second)?,
                        number: Number::Plural,
                    })
                }
                _ => None,
            },
        ),
        // "I'm looking at `InGrid`, which fields does it have?": what the
        // user is looking at, set off before the question about it.
        word(
            RuleInfo::new("en.prompt.looking_at", ROUND),
            Cat::Prompt,
            &["i'm", "looking", "at"],
            Sem::Quote(String::new()),
        )
        .read_only(),
        // "what are the fields of `Week`": "what is" asked of many.
        word(
            RuleInfo::new("en.what_is.what_are", ROUND),
            Cat::WhatIs,
            &["what", "are"],
            Sem::Quote(String::new()),
        )
        .read_only(),
    ];
    // "`Note` struct - what fields", "Identity - which fields": a role
    // alone, asked of the thing set off before it.
    for (id, which) in [
        ("en.question.set_off_what_role", "what"),
        ("en.question.set_off_which_role", "which"),
    ] {
        rules.push(rule(
            id,
            Cat::Question,
            vec![C(Cat::Mention), C(Cat::Break), W(which), C(Cat::RoleNoun)],
            |c| match c {
                [Sem::Mention { thing, .. }, _, role] => {
                    identify(&[role_of(role, &Sem::Thing(thing.clone()))?])
                }
                _ => None,
            },
        ));
    }
    // "(fields) are on `x`", "is in `x`", "are defined on `x`"
    for (id, be) in [
        ("en.verb_phrase.are_preposition", "are"),
        ("en.verb_phrase.is_preposition", "is"),
    ] {
        rules.push(rule(
            id,
            Cat::VerbPhrase,
            vec![W(be), C(Cat::Preposition), C(Cat::NounPhrase)],
            placed,
        ));
    }
    for (id, be) in [
        ("en.verb_phrase.are_defined_preposition", "are"),
        ("en.verb_phrase.is_defined_preposition", "is"),
    ] {
        rules.push(rule(
            id,
            Cat::VerbPhrase,
            vec![
                W(be),
                C(Cat::Defined),
                C(Cat::Preposition),
                C(Cat::NounPhrase),
            ],
            placed,
        ));
    }
    rules
}
