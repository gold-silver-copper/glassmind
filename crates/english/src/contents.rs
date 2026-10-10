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

use crate::aside::filler;
use crate::asking::role_of;
use crate::first;
use crate::sem::{Cat, Number, Sem};

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
/// "what's inside `Bank`, which fields it has", "is there anything that
/// calls `x`? which functions call it": a question for what stands in a
/// relation to a thing, then for the same about the same thing, with a
/// kind said in one of them (or asking how many: "like how many functions
/// call it"). Together they ask the later question, of the kind said.
pub(crate) fn narrowed<O: Ontology>(
    earlier: &Question<O>,
    later: &Question<O>,
) -> Option<Question<O>> {
    let Question::Identify(wide) = earlier else {
        return None;
    };
    let [Condition::Related { other, .. }] = wide.conditions.as_slice() else {
        return None;
    };
    // "it" is the thing the earlier question is about, or else the one
    // thing named deep in what it describes: "where is `x` used as a field
    // type? which structs have a field that contains it?"
    let bind = |it: &Thing<O>| {
        let later = later.map_things(&mut |t| match t {
            Thing::Previous => it.clone(),
            t => t.clone(),
        });
        let fits = matches!(&later, Question::Identify(d) | Question::Count(d)
            if d.conditions == wide.conditions && (wide.kind.is_none() || d.kind.is_none()));
        fits.then_some(later)
    };
    let later = bind(other).or_else(|| match other.things().as_slice() {
        [thing] => bind(thing),
        _ => None,
    })?;
    let (Question::Identify(d) | Question::Count(d)) = &later else {
        return None;
    };
    let d = Description {
        kind: d.kind.clone().or_else(|| wide.kind.clone()),
        conditions: d.conditions.clone(),
    };
    Some(match later {
        Question::Count(_) => Question::Count(d),
        _ => Question::Identify(d),
    })
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    // A question asking for what the description among the children
    // describes.
    let question = |id, rhs| rule(id, Cat::Question, rhs, identify);
    let mut rules = vec![
        // "what fields on `memo_data`", "which fields are in `S`": a
        // question word before a description asks for what it describes.
        question(
            "en.question.what_description",
            vec![W("what"), C(Cat::Description)],
        ),
        question(
            "en.question.which_description",
            vec![W("which"), C(Cat::Description)],
        ),
        // "what are the fields of `S`", "what's the fields of `S`", "what
        // is inside `S`" ("what are" is a `Cat::WhatIs`)
        question(
            "en.question.what_is_description",
            vec![C(Cat::WhatIs), C(Cat::Description)],
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
        rule(
            "en.ask_again.for_me",
            Cat::AskAgain,
            vec![C(Cat::AskAgain), W("for"), W("me")],
            first,
        ),
        // "could you list them": asking again for what was just answered.
        filler(
            "en.ask_again.could_you_list_them",
            ROUND,
            Cat::AskAgain,
            "could you list them",
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
        // "I'm looking at `InGrid`, which fields does it have?": what the
        // user is looking at, set off before the question about it.
        filler("en.prompt.looking_at", ROUND, Cat::Prompt, "i'm looking at"),
        // "what are the fields of `Week`": "what is" asked of many.
        filler("en.what_is.what_are", ROUND, Cat::WhatIs, "what are"),
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
    // "(field) is in `x`"
    rules.push(rule(
        "en.verb_phrase.is_preposition",
        Cat::VerbPhrase,
        vec![W("is"), C(Cat::Preposition), C(Cat::NounPhrase)],
        placed,
    ));
    rules
}
