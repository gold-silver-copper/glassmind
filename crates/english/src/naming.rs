//! Describing things by their names: "a field named `done`", "a `tabs`
//! field", "the field `search`", "`merge_state` as a field".
//!
//! A noun with a name beside it is any thing of the noun's kind going by
//! that name. Many things may share a name, so it is a condition on what
//! is described, not a reference to one thing.

use glassmind_grammar::{Rule, Sym};
use glassmind_meaning::{Condition, Description, Ontology, Thing};

use crate::asking::read_only_rule;
use crate::lexicon::SAID;
use crate::renaming::heard;
use crate::seed::object;
use crate::sem::{Cat, Number, Sem};
use crate::spoken;

use Cat::*;
use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 5;

/// "named `x`": the described thing goes by the name; "named that": by
/// the name of the thing said before.
fn named<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let condition = match c.last()? {
        Sem::Thing(Thing::Named(name)) => Condition::Named(name.clone()),
        Sem::Thing(Thing::Previous) => Condition::NamedLike(Thing::Previous),
        _ => return None,
    };
    Some(Sem::Condition {
        condition,
        number: Number::Plural,
    })
}

/// "a `x` field", "field `x`": any thing of the kind going by the name.
fn kind_named<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Thing(Thing::Named(name)), Sem::Kind { kind, .. }]
        | [Sem::Kind { kind, .. }, Sem::Thing(Thing::Named(name))] => {
            Some(Sem::Thing(Thing::Fitting(Description {
                kind: Some(kind.clone()),
                conditions: vec![Condition::Named(name.clone())],
            })))
        }
        _ => None,
    }
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    vec![
        // "(a field) named `done`": how the system says it.
        spoken(
            "en.qualifier.named",
            ROUND,
            Qualifier,
            vec![W("named"), C(NounPhrase)],
            named,
            |v| match v {
                Sem::Condition {
                    condition: Condition::Named(name),
                    ..
                } => Some(vec![Sem::Thing(Thing::Named(name.clone()))]),
                _ => None,
            },
        ),
        // "a field called `footer`"
        heard(
            "en.qualifier.called",
            Qualifier,
            vec![W("called"), C(NounPhrase)],
            named,
        ),
        // "(a field) named like that": goes by the name of the thing said
        // before, as "named that" does.
        read_only_rule(
            "en.qualifier.named_like",
            13,
            Qualifier,
            vec![W("named like"), C(NounPhrase)],
            named,
        ),
        // "(a field) with that name": goes by the name of the thing said
        // before.
        read_only_rule(
            "en.qualifier.with_that_name",
            13,
            Qualifier,
            vec![W("with that"), C(NameNoun)],
            |_| named(&[Sem::Thing(Thing::Previous)]),
        ),
        // "a `push_head` field", "`seam_m` field"
        heard(
            "en.noun_phrase.name_kind",
            NounPhrase,
            vec![C(NounPhrase), C(Noun)],
            kind_named,
        ),
        // "field `width`", "a field `eye_off`"
        heard(
            "en.noun_phrase.kind_name",
            NounPhrase,
            vec![C(Noun), C(NounPhrase)],
            kind_named,
        ),
        // "`merge_state` as a field"
        heard(
            "en.noun_phrase.as_a",
            NounPhrase,
            vec![C(NounPhrase), W("as a"), C(Noun)],
            kind_named,
        ),
        // "what structs have `selected` as field"
        read_only_rule(
            "en.noun_phrase.as_bare",
            17,
            NounPhrase,
            vec![C(NounPhrase), W("as"), C(Noun)],
            kind_named,
        ),
        // "the field `search`", "the `saved` field": a thing described,
        // which "the" leaves as it is.
        heard(
            "en.noun_phrase.the_fitting",
            NounPhrase,
            vec![W("the"), C(NounPhrase)],
            |c| match c {
                [t @ Sem::Thing(Thing::Fitting(_))] => Some(t.clone()),
                _ => None,
            },
        ),
        // "which structs have field named calls?", "which structs have
        // field far": a word after "named" or the kind is the name, even
        // one the system knows as English.
        read_only_rule(
            "en.qualifier.named_word",
            15,
            Qualifier,
            vec![W("named"), Class(SAID)],
            named,
        ),
        read_only_rule(
            "en.noun_phrase.kind_word",
            15,
            NounPhrase,
            vec![C(Noun), Class(SAID)],
            kind_named,
        ),
        // "fields on call struct": the same, with the kind after the word.
        read_only_rule(
            "en.noun_phrase.word_kind",
            26,
            NounPhrase,
            vec![Class(SAID), C(Noun)],
            kind_named,
        ),
        // "whcih functions call forget", "who calls forget": the word
        // after a verb is the thing it names, even one the system knows
        // as English.
        read_only_rule(
            "en.verb_phrase.object_word",
            22,
            VerbPhrase,
            vec![C(Verb), Class(SAID)],
            object,
        ),
    ]
}
