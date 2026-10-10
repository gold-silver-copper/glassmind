//! Asides: the words users put around a request that ask nothing ("Hey,
//! quick question: ...", "um, ...", "... Thanks!"). The request means the
//! same with or without them. The system never says one.

use glassmind_grammar::{Rule, RuleInfo, Sym};
use glassmind_meaning::Ontology;

use crate::asking::read_only_rule;
use crate::blank;
use crate::remark::request;
use crate::sem::{Cat, Sem};
use crate::word;

use Cat::*;
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
    // "Out of curiosity, could you show me what you're keeping ..."
    ("en.aside.out_of_curiosity", 19, "out of curiosity"),
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
    // "Oops, sorry, I meant `x`.", "ah sorry, ...", "no, I meant ...": a
    // slip owned before it is put right.
    ("en.aside.oops", 10, "oops"),
    ("en.aside.ah", 10, "ah"),
    ("en.aside.no", 10, "no"),
    // "ok anyway, could you rename it ...": going back to the request
    // after a digression; "..., thx".
    ("en.aside.anyway", 11, "anyway"),
    ("en.aside.thx", 11, "thx"),
    // "Ok so, to keep things short, I'll refer to ...": why the user is
    // about to say something.
    ("en.aside.to_keep_things_short", 23, "to keep things short"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let utterance = |id, round, rhs| read_only_rule(id, round, Utterance, rhs, request);
    let mut rules: Vec<_> = ASIDES
        .iter()
        .map(|&(id, round, words)| filler(id, round, Aside, words))
        .collect();
    rules.extend([
        // "Please forget that, thanks."
        utterance(
            "en.utterance.comma_aside",
            6,
            vec![C(Utterance), W(","), C(Aside)],
        ),
        // "Hey, where is `x` defined?" and "Where is `x` defined? Thanks!"
        utterance(
            "en.utterance.aside_before",
            ROUND,
            vec![C(Aside), C(Utterance)],
        ),
        utterance(
            "en.utterance.aside_after",
            ROUND,
            vec![C(Utterance), C(Aside)],
        ),
    ]);
    rules
}
