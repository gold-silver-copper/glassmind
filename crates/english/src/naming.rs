//! Describing things by their names: "a field named `done`", "a `tabs`
//! field", "the field `search`", "`merge_state` as a field".
//!
//! A noun with a name beside it is any thing of the noun's kind going by
//! that name. Many things may share a name, so it is a condition on what
//! is described, not a reference to one thing.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Condition, Description, Ontology, Thing};

use crate::lexicon::SAID;
use crate::renaming::heard;
use crate::sem::{Cat, Number, Sem};

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
        Rule::new(
            RuleInfo::new("en.qualifier.named", ROUND),
            Cat::Qualifier,
            vec![W("named"), C(Cat::NounPhrase)],
            S::Both(named, |v| match v {
                Sem::Condition {
                    condition: Condition::Named(name),
                    ..
                } => Some(vec![Sem::Thing(Thing::Named(name.clone()))]),
                _ => None,
            }),
        ),
        // "a field called `footer`"
        heard(
            "en.qualifier.called",
            Cat::Qualifier,
            vec![W("called"), C(Cat::NounPhrase)],
            named,
        ),
        // "(a field) named like that": goes by the name of the thing said
        // before, as "named that" does.
        Rule::new(
            RuleInfo::new("en.qualifier.named_like", 13),
            Cat::Qualifier,
            vec![W("named"), W("like"), C(Cat::NounPhrase)],
            S::Build(named),
        )
        .read_only(),
        // "(a field) with that name": goes by the name of the thing said
        // before.
        Rule::new(
            RuleInfo::new("en.qualifier.with_that_name", 13),
            Cat::Qualifier,
            vec![W("with"), W("that"), C(Cat::NameNoun)],
            S::Build(|_| named(&[Sem::Thing(Thing::Previous)])),
        )
        .read_only(),
        // "a `push_head` field", "`seam_m` field"
        heard(
            "en.noun_phrase.name_kind",
            Cat::NounPhrase,
            vec![C(Cat::NounPhrase), C(Cat::Noun)],
            kind_named,
        ),
        // "field `width`", "a field `eye_off`"
        heard(
            "en.noun_phrase.kind_name",
            Cat::NounPhrase,
            vec![C(Cat::Noun), C(Cat::NounPhrase)],
            kind_named,
        ),
        // "`merge_state` as a field"
        heard(
            "en.noun_phrase.as_a",
            Cat::NounPhrase,
            vec![C(Cat::NounPhrase), W("as"), W("a"), C(Cat::Noun)],
            kind_named,
        ),
        // "the field `search`", "the `saved` field": a thing described,
        // which "the" leaves as it is.
        heard(
            "en.noun_phrase.the_fitting",
            Cat::NounPhrase,
            vec![W("the"), C(Cat::NounPhrase)],
            |c| match c {
                [t @ Sem::Thing(Thing::Fitting(_))] => Some(t.clone()),
                _ => None,
            },
        ),
        // "which structs have field named calls?", "which structs have
        // field far": a word after "named" or the kind is the name, even
        // one the system knows as English.
        Rule::new(
            RuleInfo::new("en.qualifier.named_word", 15),
            Cat::Qualifier,
            vec![W("named"), Class(SAID)],
            S::Build(named),
        )
        .read_only(),
        Rule::new(
            RuleInfo::new("en.noun_phrase.kind_word", 15),
            Cat::NounPhrase,
            vec![C(Cat::Noun), Class(SAID)],
            S::Build(kind_named),
        )
        .read_only(),
    ]
}
