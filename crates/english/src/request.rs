//! Polite requests: "can you tell me which functions call `x`?" asks the
//! embedded question itself. The opening asks nothing of its own, so the
//! question means the same with or without it. The system never says one.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::Ontology;

use crate::sem::{Cat, Sem};

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 2;

/// The question a request goes with.
fn question<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find(|v| matches!(v, Sem::Question(_))).cloned()
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let opening = || S::Const(Sem::Quote(String::new()));
    let asks = |id, rhs| {
        Rule::new(
            RuleInfo::new(id, ROUND),
            Cat::Question,
            rhs,
            S::Build(question),
        )
    };
    vec![
        // "Can you ...": asking whether the hearer can do something, which
        // asks them to do it.
        Rule::new(
            RuleInfo::new("en.request.can_you", ROUND),
            Cat::Request,
            vec![W("can"), W("you")],
            opening(),
        ),
        // "Can you please ...".
        Rule::new(
            RuleInfo::new("en.request.please", ROUND),
            Cat::Request,
            vec![C(Cat::Request), W("please")],
            opening(),
        ),
        // "Can you tell me which functions call `data_dir`?"
        asks(
            "en.question.tell_me",
            vec![C(Cat::Request), W("tell"), W("me"), C(Cat::Question)],
        ),
        // "Can you show me which functions call `go_to_file`?"
        asks(
            "en.question.show_me",
            vec![C(Cat::Request), W("show"), W("me"), C(Cat::Question)],
        ),
        // "... which functions call `merge_box`, please?"
        asks(
            "en.question.please",
            vec![C(Cat::Question), W(","), W("please")],
        ),
    ]
}
