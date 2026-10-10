//! Putting right what was just asked: "sorry, I meant `y`", "oops, meant
//! `y`", "I actually meant `y`, not that one". The request is the one made
//! before, about the thing now said instead.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Utterance};

use crate::aside::filler;
use crate::first;
use crate::sem::{Cat, Sem};

use Cat::*;
use Sym::{Cat as C, Word as W};

const ROUND: u32 = 10;

/// The correction among the children, with what it sets aside after it.
fn correction<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    first(c).filter(|v| matches!(v, Sem::Utterance(Utterance::Correct(_))))
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, sem| Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, sem).read_only();
    vec![
        // "meant": what the speaker had in mind.
        filler("en.meant.meant", ROUND, Meant, "meant"),
        // "actually meant", "I actually meant" ("I meant" is in
        // [`crate::around`]).
        rule(
            "en.meant.hedged",
            Meant,
            vec![C(Hedge), C(Meant)],
            S::Build(first),
        ),
        // "I meant `y`", "meant the `Y` struct": the request before, about
        // the thing mentioned.
        rule(
            "en.utterance.meant",
            Utterance,
            vec![C(Meant), C(Mention)],
            S::Build(|c| match c {
                [_, Sem::Mention { thing, .. }] => {
                    Some(Sem::Utterance(Utterance::Correct(thing.clone())))
                }
                _ => None,
            }),
        ),
        // "I meant `y`, not that one": the correction, with the thing it
        // sets aside.
        rule(
            "en.utterance.meant_comma_not",
            Utterance,
            vec![C(Utterance), W(", not"), C(NounPhrase)],
            S::Build(correction),
        ),
    ]
}
