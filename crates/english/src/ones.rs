//! Things described without a noun for their kind: "which ones call it?",
//! "is there anything that ends up calling it?", "which functions are
//! those that call it?", "which ones are they, the ones calling it?". The
//! description says what is asked for; the word in the noun's place
//! leaves the kind open.

use glassmind_grammar::{Rule, Sym};
use glassmind_meaning::Ontology;

use crate::asking::{described, identify, read_only_rule, words_then};
use crate::last;
use crate::locate::fillers;
use crate::sem::{Cat, Sem};

use Cat::*;
use Sym::{Cat as C, Word as W};

const ROUND: u32 = 9;

/// Asks for what the children after the first describe.
fn identify_rest<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    identify(c.get(1..)?)
}

/// What the children after the first describe.
fn described_rest<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    described(c.get(1..)?)
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| read_only_rule(id, ROUND, lhs, rhs, build);
    let mut rules = vec![
        // "ones calling `x`", "anything that calls it"
        rule(
            "en.description.ones_participial",
            Description,
            vec![C(Ones), C(Participial)],
            described_rest,
        ),
        rule(
            "en.description.ones_that",
            Description,
            vec![C(Ones), W("that"), C(VerbPhrase)],
            described_rest,
        ),
        // "everything `shoved` invokes", "the ones `x` calls"
        read_only_rule(
            "en.description.ones_restricted",
            13,
            Description,
            vec![C(Ones), C(Restriction)],
            described_rest,
        ),
        // "they, the functions that call it": the description they stand for.
        rule(
            "en.description.ones_apposed",
            Description,
            vec![C(Ones), W(","), C(Description)],
            last,
        ),
        // "which ones call it?"
        rule(
            "en.question.which_ones",
            Question,
            vec![W("which"), C(Ones), C(VerbPhrase)],
            identify_rest,
        ),
        // "which ones are they, the ones calling it?", "which functions are
        // those that call it?"
        rule(
            "en.question.which_ones_are",
            Question,
            vec![W("which"), C(Ones), W("are"), C(Description)],
            identify_rest,
        ),
        rule(
            "en.question.which_kind_are",
            Question,
            vec![W("which"), C(Noun), W("are"), C(Description)],
            identify_rest,
        ),
        // "is there anything that calls it?"
        rule(
            "en.question.is_there",
            Question,
            words_then("is there", &[Description]),
            identify,
        ),
    ];
    rules.extend(fillers(
        ROUND,
        &[(
            Ones,
            &[
                ("en.ones.ones", "ones"),
                ("en.ones.those", "those"),
                ("en.ones.they", "they"),
            ],
        )],
    ));
    // "how many places call it", "is there many things that end up
    // calling `x`"
    rules.extend(fillers(
        11,
        &[(
            Ones,
            &[("en.ones.places", "places"), ("en.ones.things", "things")],
        )],
    ));
    rules.extend(fillers(
        13,
        &[(Ones, &[("en.ones.everything", "everything")])],
    ));
    rules
}
