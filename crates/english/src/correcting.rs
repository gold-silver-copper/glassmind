//! Putting right what was just asked: "sorry, I meant `y`", "oops, meant
//! `y`", "I actually meant `y`, not that one". The request is the one made
//! before, about the thing now said instead.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Utterance};

use crate::aside::filler;
use crate::first;
use crate::sem::{Cat, Sem};

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
        filler("en.meant.meant", ROUND, Cat::Meant, "meant"),
        // "I meant", "actually meant", "I actually meant".
        rule(
            "en.meant.i",
            Cat::Meant,
            vec![W("i"), C(Cat::Meant)],
            S::Build(first),
        ),
        rule(
            "en.meant.hedged",
            Cat::Meant,
            vec![C(Cat::Hedge), C(Cat::Meant)],
            S::Build(first),
        ),
        // "I meant `y`", "meant the `Y` struct": the request before, about
        // the thing mentioned.
        rule(
            "en.utterance.meant",
            Cat::Utterance,
            vec![C(Cat::Meant), C(Cat::Mention)],
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
            Cat::Utterance,
            vec![C(Cat::Utterance), W(","), W("not"), C(Cat::NounPhrase)],
            S::Build(correction),
        ),
    ]
}
