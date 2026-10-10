//! Remarks: what users say about themselves around a request ("I'm
//! reworking the follow feature a bit."). A remark asks nothing, so the
//! request means the same with or without it. The system never says one.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::Ontology;

use crate::lexicon::TEXT;
use crate::sem::{Cat, Sem};

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 1;

/// The request a remark goes with.
fn request<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find(|v| matches!(v, Sem::Utterance(_))).cloned()
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let remark = || S::Const(Sem::Quote(String::new()));
    let utterance = |id, rhs| {
        Rule::new(
            RuleInfo::new(id, ROUND),
            Cat::Utterance,
            rhs,
            S::Build(request),
        )
    };
    vec![
        // "I'm looking at the voxel code": the speaker's own situation, up
        // to the next punctuation.
        Rule::new(
            RuleInfo::new("en.remark.i_am", ROUND),
            Cat::Remark,
            vec![W("i'm"), Class(TEXT)],
            remark(),
        ),
        // Each further word of the remark: "... the voxel code".
        Rule::new(
            RuleInfo::new("en.remark.more", ROUND),
            Cat::Remark,
            vec![C(Cat::Remark), Class(TEXT)],
            remark(),
        ),
        // "I'm looking at the voxel code. How many functions call `carve`?"
        utterance(
            "en.utterance.remark_stop",
            vec![C(Cat::Remark), W("."), C(Cat::Utterance)],
        ),
        // "I'm looking into validation a bit, which functions call `problems`?"
        utterance(
            "en.utterance.remark_comma",
            vec![C(Cat::Remark), W(","), C(Cat::Utterance)],
        ),
        // "Which functions call `x`? i'm trying to understand where it's hooked in"
        utterance(
            "en.utterance.remark_after",
            vec![C(Cat::Utterance), C(Cat::Remark)],
        ),
        // "Which functions call `inner`? I'm trying to understand how it's used."
        utterance(
            "en.utterance.remark_after_stop",
            vec![C(Cat::Utterance), C(Cat::Remark), W(".")],
        ),
    ]
}
