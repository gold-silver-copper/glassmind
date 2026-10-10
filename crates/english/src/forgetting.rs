//! Taking back what was said: "forget that", "never mind that last
//! question", "scratch that", and the system's reply, "OK, I've forgotten
//! that.".

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Reply, Utterance};

use crate::aside::filler;
use crate::blank;
use crate::phrase;
use crate::sem::{Cat, Sem};

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 6;

/// The words of a request to forget, each meaning nothing by itself.
const WORDS: &[(&str, Cat, &str)] = &[
    // Verbs asking the system to drop what was said, alone or before what
    // they drop: "forget (that)", "never mind (that)", "scratch that".
    ("en.dismiss.forget", Cat::Dismiss, "forget"),
    ("en.dismiss.never_mind", Cat::Dismiss, "never mind"),
    ("en.dismiss.nevermind", Cat::Dismiss, "nevermind"),
    ("en.dismiss.nvm", Cat::Dismiss, "nvm"),
    ("en.dismiss.scratch", Cat::Dismiss, "scratch"),
    // Words pointing back at what was said: "that", "it".
    ("en.said.that", Cat::Said, "that"),
    ("en.said.it", Cat::Said, "it"),
    // Nouns for something said before: "one", "thing", "question".
    ("en.earlier.one", Cat::Earlier, "one"),
    ("en.earlier.thing", Cat::Earlier, "thing"),
    ("en.earlier.question", Cat::Earlier, "question"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, sem| Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, sem).read_only();
    let blank_of = |id, lhs, rhs| rule(id, lhs, rhs, S::Const(blank()));
    let forget = |id, rhs| {
        rule(
            id,
            Cat::Utterance,
            rhs,
            S::Const(Sem::Utterance(Utterance::Forget)),
        )
    };
    let mut rules: Vec<_> = WORDS
        .iter()
        .map(|&(id, cat, words)| filler(id, ROUND, cat, words))
        .collect();
    rules.extend([
        // "that one", "that last question": what was said, by when.
        blank_of(
            "en.said.that_one",
            Cat::Said,
            vec![W("that"), C(Cat::Earlier)],
        ),
        blank_of(
            "en.earlier.last",
            Cat::Earlier,
            vec![W("last"), C(Cat::Earlier)],
        ),
        // "never mind", "nvm": what to drop goes without saying.
        forget("en.utterance.dismiss", vec![C(Cat::Dismiss)]),
        // "forget that", "never mind that last question", "scratch that".
        forget(
            "en.utterance.dismiss_said",
            vec![C(Cat::Dismiss), C(Cat::Said)],
        ),
        // "OK, I've forgotten that."
        Rule::new(
            RuleInfo::new("en.reply.forgotten", ROUND),
            Cat::Reply,
            phrase("ok , I've forgotten that ."),
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Forgotten) => Some(Vec::new()),
                    _ => None,
                },
            ),
        ),
    ]);
    rules
}
