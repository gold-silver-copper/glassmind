//! Things described without a noun for their kind: "which ones call it?",
//! "is there anything that ends up calling it?", "which functions are
//! those that call it?", "which ones are they, the ones calling it?". The
//! description says what is asked for; the word in the noun's place
//! leaves the kind open.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::Ontology;

use crate::asking::{described, identify, words_then};
use crate::last;
use crate::locate::fillers;
use crate::sem::{Cat, Sem};

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
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let mut rules = vec![
        // "ones calling `x`", "anything that calls it"
        rule(
            "en.description.ones_participial",
            Cat::Description,
            vec![C(Cat::Ones), C(Cat::Participial)],
            described_rest,
        ),
        rule(
            "en.description.ones_that",
            Cat::Description,
            vec![C(Cat::Ones), W("that"), C(Cat::VerbPhrase)],
            described_rest,
        ),
        // "they, the functions that call it": the description they stand for.
        rule(
            "en.description.ones_apposed",
            Cat::Description,
            vec![C(Cat::Ones), W(","), C(Cat::Description)],
            last,
        ),
        // "which ones call it?"
        rule(
            "en.question.which_ones",
            Cat::Question,
            vec![W("which"), C(Cat::Ones), C(Cat::VerbPhrase)],
            identify_rest,
        ),
        // "which ones are they, the ones calling it?", "which functions are
        // those that call it?"
        rule(
            "en.question.which_ones_are",
            Cat::Question,
            vec![W("which"), C(Cat::Ones), W("are"), C(Cat::Description)],
            identify_rest,
        ),
        rule(
            "en.question.which_kind_are",
            Cat::Question,
            vec![W("which"), C(Cat::Noun), W("are"), C(Cat::Description)],
            identify_rest,
        ),
        // "is there anything that calls it?"
        rule(
            "en.question.is_there",
            Cat::Question,
            words_then("is there", &[Cat::Description]),
            identify,
        ),
    ];
    rules.extend(fillers(
        ROUND,
        &[(
            Cat::Ones,
            &[
                ("en.ones.ones", "ones"),
                ("en.ones.those", "those"),
                ("en.ones.they", "they"),
                ("en.ones.anything", "anything"),
            ],
        )],
    ));
    // "how many places call it", "is there many things that end up
    // calling `x`"
    rules.extend(fillers(
        11,
        &[(
            Cat::Ones,
            &[("en.ones.places", "places"), ("en.ones.things", "things")],
        )],
    ));
    rules
}
