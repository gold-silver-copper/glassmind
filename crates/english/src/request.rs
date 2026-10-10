//! Polite requests: "can you tell me which functions call `x`?" asks the
//! embedded question itself. The opening asks nothing of its own, so the
//! question means the same with or without it. The system never says one.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::Ontology;

use crate::sem::{Cat, Sem};
use crate::{blank, word};

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 2;

/// The question a request goes with.
fn question<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find(|v| matches!(v, Sem::Question(_))).cloned()
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let asks = |id, rhs| {
        Rule::new(
            RuleInfo::new(id, ROUND),
            Cat::Question,
            rhs,
            S::Build(question),
        )
    };
    let mut rules = vec![
        // "Can you ...": asking whether the hearer can do something, which
        // asks them to do it. The opening asks nothing of its own.
        word(
            RuleInfo::new("en.request.can_you", ROUND),
            Cat::Request,
            "can you",
            blank(),
        ),
    ];
    // "Can you tell me which functions call `data_dir`?", "Can you show me
    // which functions call `go_to_file`?"
    for (id, verb) in [
        ("en.question.tell_me", "tell"),
        ("en.question.show_me", "show"),
    ] {
        rules.push(asks(
            id,
            vec![C(Cat::Request), W(verb), W("me"), C(Cat::Question)],
        ));
    }
    // "... which functions call `merge_box`, please?"
    rules.push(asks(
        "en.question.please",
        vec![C(Cat::Question), W(","), W("please")],
    ));
    rules
}
