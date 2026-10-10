//! Asking what a thing holds: "what fields are on the `Menu` struct?",
//! "what's inside `NewDm`?", "what are the fields of `Week`?", "`Conv`
//! fields", "`S` struct fields", "which fields make up `S`", "what fields
//! has `Lamp` got?", and the same asked about a thing set off before
//! ("`Note` struct - what fields", "I'm looking at `InGrid`, which fields
//! does it have?").
//!
//! The things asked for are often described by where they are, so these
//! shapes build a [`Cat::Description`] and ask for what it describes.

use glassmind_grammar::{Rule, Sym};
use glassmind_meaning::{Condition, Description, Ontology, Position, Question, Thing};

use crate::aside::filler;
use crate::asking::read_only_rule;
use crate::asking::role_of;
use crate::sem::{Cat, Number, Sem};

use Cat::*;
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
    let rule = |id, lhs, rhs, build| read_only_rule(id, ROUND, lhs, rhs, build);
    // A question asking for what the description among the children
    // describes.
    let question = |id, rhs| rule(id, Question, rhs, identify);
    let mut rules = vec![
        // "what fields on `memo_data`", "which fields are in `S`": a
        // question word before a description asks for what it describes.
        question(
            "en.question.what_description",
            vec![W("what"), C(Description)],
        ),
        question(
            "en.question.which_description",
            vec![W("which"), C(Description)],
        ),
        // "what are the fields of `S`", "what's the fields of `S`", "what
        // is inside `S`" ("what are" is a `Cat::WhatIs`)
        question(
            "en.question.what_is_description",
            vec![C(WhatIs), C(Description)],
        ),
        // "fields are on `S`", "fields are defined on the `S` struct"
        rule(
            "en.description.are_preposition",
            Description,
            vec![C(Noun), W("are"), C(Preposition), C(NounPhrase)],
            placed_kind,
        ),
        rule(
            "en.description.are_defined_preposition",
            Description,
            vec![C(Noun), W("are"), C(Defined), C(Preposition), C(NounPhrase)],
            placed_kind,
        ),
        // "could you list them": asking again for what was just answered.
        filler(
            "en.ask_again.could_you_list_them",
            ROUND,
            AskAgain,
            "could you list them",
        ),
        // "what's in `RepoId`", "what is inside `Bank`", "what's in it":
        // what the thing holds.
        rule(
            "en.question.what_is_inside",
            Question,
            vec![C(WhatIs), C(Inside), C(NounPhrase)],
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
            Description,
            vec![W("its"), C(RoleNoun)],
            |c| role_of(c.first()?, &Sem::Thing(Thing::Previous)),
        ),
        // "I'm looking at `InGrid`, which fields does it have?": what the
        // user is looking at, set off before the question about it.
        filler("en.prompt.looking_at", ROUND, Prompt, "i'm looking at"),
        // "what are the fields of `Week`": "what is" asked of many.
        filler("en.what_is.what_are", ROUND, WhatIs, "what are"),
    ];
    // "`Note` struct - what fields", "Identity - which fields": a role
    // alone, asked of the thing set off before it; also with no mark
    // between, "index_of what fields", "refresh_search struct which
    // fields", "doing struct what fields".
    for (id, round, rhs) in [
        (
            "en.question.set_off_what_role",
            ROUND,
            vec![C(Mention), C(Break), W("what"), C(RoleNoun)],
        ),
        (
            "en.question.set_off_which_role",
            ROUND,
            vec![C(Mention), C(Break), W("which"), C(RoleNoun)],
        ),
        // "doing struct what fields": the same, of a thing described.
        (
            "en.question.unmarked_what_role_described",
            26,
            vec![C(NounPhrase), W("what"), C(RoleNoun)],
        ),
        (
            "en.question.unmarked_which_role_described",
            26,
            vec![C(NounPhrase), W("which"), C(RoleNoun)],
        ),
    ] {
        rules.push(read_only_rule(id, round, Question, rhs, |c| match c {
            [Sem::Mention { thing, .. } | Sem::Thing(thing), .., role] => {
                identify(&[role_of(role, &Sem::Thing(thing.clone()))?])
            }
            _ => None,
        }));
    }
    // "(field) is in `x`"
    rules.push(rule(
        "en.verb_phrase.is_preposition",
        VerbPhrase,
        vec![W("is"), C(Preposition), C(NounPhrase)],
        placed,
    ));
    rules
}
