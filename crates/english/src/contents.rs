//! Asking what a thing is made of: "what fields are on the `S` struct?",
//! "what are the fields of `S`", "what fields in `S`", "`S` struct fields",
//! "which fields make up `S`".
//!
//! The things asked for are described by where they are, so these shapes
//! build a [`Cat::Description`] and ask for what it describes.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Condition, Description, Ontology, Position, Question};

use crate::sem::{Cat, Sem};

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 7;

/// Things of a kind standing in a relation to a thing: "fields (are) on
/// `S`", the kind, the relation and the thing among the children.
fn placed<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let kind = c.iter().find_map(|v| match v {
        Sem::Kind { kind, .. } => Some(kind.clone()),
        _ => None,
    })?;
    let relation = c.iter().find_map(|v| match v {
        Sem::Relation { relation, .. } => Some(relation.clone()),
        _ => None,
    })?;
    let other = c.iter().find_map(|v| match v {
        Sem::Thing(t) => Some(t.clone()),
        _ => None,
    })?;
    Some(Sem::Description(Description {
        kind: Some(kind),
        conditions: vec![Condition::Related {
            relation,
            position: Position::First,
            other,
        }],
    }))
}

/// Asks to identify what the description among the children describes.
fn identify<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find_map(|v| match v {
        Sem::Description(d) => Some(Sem::Question(Question::Identify(d.clone()))),
        _ => None,
    })
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    vec![
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
            placed,
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
            placed,
        ),
        // "`S` struct fields", "end_speed fields", "hint_labels struct
        // members": the parts of a thing, named after it.
        rule(
            "en.description.parts",
            Cat::Description,
            vec![C(Cat::NounPhrase), C(Cat::Part)],
            |c| match c {
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
    ]
}
