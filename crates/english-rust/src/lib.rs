//! The English words for the Rust world's kinds and relations.
//!
//! A world never adds grammar, and the English crate knows no world; this
//! bridge adds one rule per word form, each reading as a Rust concept.

use glassmind_english::{Cat, Number, Sem, VerbForm, word};
use glassmind_grammar::Rule;
use glassmind_rules::{RuleInfo, SEED};
use glassmind_world_rust::{Rust, RustKind, RustRelation};

fn kind(
    id: &'static str,
    round: u32,
    form: &'static str,
    kind: RustKind,
    number: Number,
) -> Rule<Cat, Sem<Rust>> {
    word(
        RuleInfo::new(id, round),
        Cat::Noun,
        &[form],
        Sem::Kind { kind, number },
    )
}

fn verb(
    id: &'static str,
    round: u32,
    form: &'static str,
    relation: RustRelation,
    verb_form: VerbForm,
) -> Rule<Cat, Sem<Rust>> {
    word(
        RuleInfo::new(id, round),
        Cat::Verb,
        &[form],
        Sem::Relation {
            relation,
            form: verb_form,
        },
    )
}

/// Every English word for a Rust concept.
pub fn words() -> Vec<Rule<Cat, Sem<Rust>>> {
    use Number::*;
    use RustKind::*;
    use RustRelation::*;
    use VerbForm::*;
    vec![
        kind(
            "en.rust.noun.function",
            SEED,
            "function",
            Function,
            Singular,
        ),
        kind(
            "en.rust.noun.functions",
            SEED,
            "functions",
            Function,
            Plural,
        ),
        verb("en.rust.verb.calls", SEED, "calls", Calls, ThirdSingular),
        verb("en.rust.verb.call", SEED, "call", Calls, Plain),
    ]
}

/// All English rules for talking about Rust: the language's own and the words for the world.
pub fn rules() -> Vec<Rule<Cat, Sem<Rust>>> {
    let mut rules = glassmind_english::rules();
    rules.extend(words());
    rules
}
