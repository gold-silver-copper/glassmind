//! Remarks: what users say about themselves around a request ("I'm
//! reworking the follow feature a bit."). A remark asks nothing, so the
//! request means the same with or without it. A greeting ("Hey, ...")
//! asks nothing either. The system never says one.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::Ontology;

use crate::lexicon::TEXT;
use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 1;
const GREETING_ROUND: u32 = 2;

/// The request a remark goes with.
fn request<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find(|v| matches!(v, Sem::Utterance(_))).cloned()
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let remark_value = || Sem::Quote(String::new());
    let remark = || S::Const(remark_value());
    let utterance = |id, round, rhs| {
        Rule::new(
            RuleInfo::new(id, round),
            Cat::Utterance,
            rhs,
            S::Build(request),
        )
    };
    // A word that greets the system, before the request it opens.
    let greeting = |id, w| {
        word(
            RuleInfo::new(id, GREETING_ROUND),
            Cat::Greeting,
            &[w],
            remark_value(),
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
            ROUND,
            vec![C(Cat::Remark), W("."), C(Cat::Utterance)],
        ),
        // "I'm looking into validation a bit, which functions call `problems`?"
        utterance(
            "en.utterance.remark_comma",
            ROUND,
            vec![C(Cat::Remark), W(","), C(Cat::Utterance)],
        ),
        // "Which functions call `x`? i'm trying to understand where it's hooked in"
        utterance(
            "en.utterance.remark_after",
            ROUND,
            vec![C(Cat::Utterance), C(Cat::Remark)],
        ),
        // "Which functions call `inner`? I'm trying to understand how it's used."
        utterance(
            "en.utterance.remark_after_stop",
            ROUND,
            vec![C(Cat::Utterance), C(Cat::Remark), W(".")],
        ),
        // "Hey, which functions call `show_with`?": a greeting opening a
        // request.
        utterance(
            "en.utterance.greeting",
            GREETING_ROUND,
            vec![C(Cat::Greeting), W(","), C(Cat::Utterance)],
        ),
        // "Hi! Which functions call `run_menu_row`?"
        utterance(
            "en.utterance.greeting_exclaim",
            GREETING_ROUND,
            vec![C(Cat::Greeting), W("!"), C(Cat::Utterance)],
        ),
        // "Hey, quick question: ...", "hey, could you rename ..."
        greeting("en.greeting.hey", "hey"),
        // "Hi, what functions does `local_times_in_explicit_zones` call?"
        greeting("en.greeting.hi", "hi"),
    ]
}
