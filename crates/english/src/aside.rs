//! Asides: the words users put around a request that ask nothing ("Hey,
//! quick question: ...", "um, ...", "... Thanks!"). The request means the
//! same with or without them. The system never says one.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::Ontology;

use crate::blank;
use crate::remark::request;
use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 2;

/// A fixed phrase that means nothing by itself (an aside, a prompt, the
/// "that" of "forget that"), read but never said.
pub(crate) fn filler<O: Ontology>(
    id: &'static str,
    round: u32,
    cat: Cat,
    words: &'static str,
) -> Rule<Cat, Sem<O>> {
    word(RuleInfo::new(id, round), cat, words, blank()).read_only()
}

/// Greetings, hesitations and thanks, as users open or close a request,
/// and the round each came in.
const ASIDES: &[(&str, u32, &str)] = &[
    ("en.aside.ok", ROUND, "ok"),
    ("en.aside.so", ROUND, "so"),
    ("en.aside.um", ROUND, "um"),
    ("en.aside.uh", ROUND, "uh"),
    ("en.aside.hmm", ROUND, "hmm"),
    ("en.aside.hm", ROUND, "hm"),
    ("en.aside.oh", ROUND, "oh"),
    ("en.aside.sorry", ROUND, "sorry"),
    ("en.aside.thanks", ROUND, "thanks"),
    ("en.aside.quick_question", ROUND, "quick question"),
    ("en.aside.quick_one", ROUND, "quick one"),
    // "Quick request: please rename ...", "... Thanks a lot!"
    ("en.aside.quick_request", 5, "quick request"),
    ("en.aside.thanks_a_lot", 5, "thanks a lot"),
    // "Actually, please forget that."
    ("en.aside.actually", 6, "actually"),
    ("en.aside.thanks_in_advance", 5, "thanks in advance"),
    // "wait, why ...", "ok but why ...": a pause before a question about
    // what was just said.
    ("en.aside.wait", 6, "wait"),
    ("en.aside.but", 6, "but"),
    // "Quick heads-up: I'm going to call ...", "btw I'll call ...", "...,
    // fyi"
    ("en.aside.quick_heads_up", 6, "quick heads-up"),
    ("en.aside.quick_thing", 6, "quick thing"),
    ("en.aside.btw", 6, "btw"),
    ("en.aside.fyi", 6, "fyi"),
    // "And which functions call it?", "ok and where is that one defined?",
    // "Okay, and ...": a turn going on from the one before.
    ("en.aside.and", 9, "and"),
    ("en.aside.okay", 9, "okay"),
    // "Got it. Where is that function defined?", "Great, thanks.", "Makes
    // sense, thanks.": the last reply taken in.
    ("en.aside.got_it", 9, "got it"),
    ("en.aside.great", 9, "great"),
    ("en.aside.makes_sense", 9, "makes sense"),
    // "How many functions call it, by the way?"
    ("en.aside.by_the_way", 9, "by the way"),
];

/// The punctuation after an aside: "..., ok?" (an aside asking whether
/// what was said is fine), and what ends one: "hey,", "hi!", "quick
/// question:", "sorry, quick one -", "Thanks.".
const MARKS: &[(&str, u32, &str)] = &[
    ("en.aside.question", 6, "?"),
    ("en.aside.comma", ROUND, ","),
    ("en.aside.bang", ROUND, "!"),
    ("en.aside.colon", ROUND, ":"),
    ("en.aside.stop", ROUND, "."),
    ("en.aside.dash", ROUND, "-"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let utterance = |id, round, rhs| {
        Rule::new(
            RuleInfo::new(id, round),
            Cat::Utterance,
            rhs,
            S::Build(request),
        )
        .read_only()
    };
    let mut rules: Vec<_> = ASIDES
        .iter()
        .map(|&(id, round, words)| filler(id, round, Cat::Aside, words))
        .collect();
    rules.extend(MARKS.iter().map(|&(id, round, mark)| {
        Rule::new(
            RuleInfo::new(id, round),
            Cat::Aside,
            vec![C(Cat::Aside), W(mark)],
            S::Const(blank()),
        )
        .read_only()
    }));
    rules.extend([
        // "Please forget that, thanks."
        utterance(
            "en.utterance.comma_aside",
            6,
            vec![C(Cat::Utterance), W(","), C(Cat::Aside)],
        ),
        // "Hey, where is `x` defined?" and "Where is `x` defined? Thanks!"
        utterance(
            "en.utterance.aside_before",
            ROUND,
            vec![C(Cat::Aside), C(Cat::Utterance)],
        ),
        utterance(
            "en.utterance.aside_after",
            ROUND,
            vec![C(Cat::Utterance), C(Cat::Aside)],
        ),
    ]);
    rules
}
