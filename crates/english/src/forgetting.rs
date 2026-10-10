//! Taking back what was said: "forget that", "never mind that last
//! question", "scratch that", and the system's reply, "OK, I've forgotten
//! that.".

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Reply, Utterance};

use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 6;

/// Verbs asking the system to drop what was said, alone or before what
/// they drop: "forget (that)", "never mind (that)", "scratch that".
const DISMISS: &[(&str, &[&str])] = &[
    ("en.dismiss.forget", &["forget"]),
    ("en.dismiss.never_mind", &["never", "mind"]),
    ("en.dismiss.nevermind", &["nevermind"]),
    ("en.dismiss.nvm", &["nvm"]),
    ("en.dismiss.scratch", &["scratch"]),
    ("en.dismiss.ignore", &["ignore"]),
    ("en.dismiss.disregard", &["disregard"]),
];

/// Words pointing back at what was said: "that", "it", "this".
const SAID: &[(&str, &[&str])] = &[
    ("en.said.that", &["that"]),
    ("en.said.it", &["it"]),
    ("en.said.this", &["this"]),
];

/// Nouns for something said before, and words placing it: "one",
/// "question", "last thing".
const EARLIER: &[(&str, &[&str])] = &[
    ("en.earlier.one", &["one"]),
    ("en.earlier.thing", &["thing"]),
    ("en.earlier.question", &["question"]),
    ("en.earlier.request", &["request"]),
    ("en.earlier.message", &["message"]),
];

/// Two ways of asking to forget, said one after the other, ask it once.
fn forget_again<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter()
        .filter(|v| matches!(v, Sem::Utterance(_)))
        .all(|v| *v == Sem::Utterance(Utterance::Forget))
        .then_some(Sem::Utterance(Utterance::Forget))
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let blank = || Sem::Quote(String::new());
    let forget = || S::Const(Sem::Utterance(Utterance::Forget));
    let rule = |id, lhs, rhs, sem| Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, sem).read_only();
    let mut rules = Vec::new();
    for (table, cat) in [
        (DISMISS, Cat::Dismiss),
        (SAID, Cat::Said),
        (EARLIER, Cat::Earlier),
    ] {
        rules.extend(
            table
                .iter()
                .map(|(id, words)| word(RuleInfo::new(id, ROUND), cat, words, blank()).read_only()),
        );
    }
    rules.extend([
        // "that one", "that last question", "the last one", "my last
        // request": what was said, by when.
        rule(
            "en.said.that_one",
            Cat::Said,
            vec![W("that"), C(Cat::Earlier)],
            S::Const(blank()),
        ),
        rule(
            "en.said.the",
            Cat::Said,
            vec![W("the"), C(Cat::Earlier)],
            S::Const(blank()),
        ),
        rule(
            "en.said.my",
            Cat::Said,
            vec![W("my"), C(Cat::Earlier)],
            S::Const(blank()),
        ),
        rule(
            "en.earlier.last",
            Cat::Earlier,
            vec![W("last"), C(Cat::Earlier)],
            S::Const(blank()),
        ),
        rule(
            "en.earlier.previous",
            Cat::Earlier,
            vec![W("previous"), C(Cat::Earlier)],
            S::Const(blank()),
        ),
        // "never mind", "nvm": what to drop goes without saying.
        rule(
            "en.utterance.dismiss",
            Cat::Utterance,
            vec![C(Cat::Dismiss)],
            forget(),
        ),
        // "forget that", "never mind that last question", "scratch that".
        rule(
            "en.utterance.dismiss_said",
            Cat::Utterance,
            vec![C(Cat::Dismiss), C(Cat::Said)],
            forget(),
        ),
        // "never mind, forget that", "forget that, nevermind", "Never mind
        // that. Please forget it."
        rule(
            "en.utterance.forget_again",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Utterance)],
            S::Build(forget_again),
        ),
        rule(
            "en.utterance.forget_again_comma",
            Cat::Utterance,
            vec![C(Cat::Utterance), W(","), C(Cat::Utterance)],
            S::Build(forget_again),
        ),
        // "OK, I've forgotten that."
        Rule::new(
            RuleInfo::new("en.reply.forgotten", ROUND),
            Cat::Reply,
            vec![
                W("ok"),
                W(","),
                W("I've"),
                W("forgotten"),
                W("that"),
                W("."),
            ],
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
