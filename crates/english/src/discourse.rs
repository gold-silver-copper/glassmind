//! Words around a request that ask nothing themselves: interjections that
//! open or close a turn ("please", "pls") and prompts that frame it ("can
//! you", "tell me", "I'd like to know"). The request means the same with or
//! without them. The system never says them.

use glassmind_grammar::{Rule, Sym};
use glassmind_meaning::Ontology;

use crate::aside::filler;
use crate::asking::read_only_rule;
use crate::remark::request;
use crate::sem::{Cat, Sem};

use Cat::*;
use Sym::{Cat as C, Word as W};

const ROUND: u32 = 2;

/// Words that open or close a turn, and the round each came in: "please",
/// and in quick requests "pls", "plz".
const INTERJECTIONS: &[(&str, u32, &str)] = &[
    ("en.interjection.please", ROUND, "please"),
    ("en.interjection.pls", 5, "pls"),
    ("en.interjection.plz", 5, "plz"),
];

/// Words that frame a request, and the round each came in. They follow
/// each other freely: "can you" "tell me", "I'd like to" "see".
const PROMPTS: &[(&str, u32, &str)] = &[
    ("en.prompt.can_you", ROUND, "can you"),
    ("en.prompt.could_you", ROUND, "could you"),
    ("en.prompt.tell_me", ROUND, "tell me"),
    ("en.prompt.show_me", ROUND, "show me"),
    ("en.prompt.show", ROUND, "show"),
    ("en.prompt.list", ROUND, "list"),
    ("en.prompt.like_to", ROUND, "i'd like to"),
    ("en.prompt.trying_to", ROUND, "i'm trying to"),
    ("en.prompt.know", ROUND, "know"),
    ("en.prompt.see", ROUND, "see"),
    ("en.prompt.figure_out", ROUND, "figure out"),
    // Asking leave for a change, or saying it is wanted: "can we", "would
    // it be ok to", "I'd like `x` renamed ...", "want `x` renamed ...".
    ("en.prompt.can_we", 5, "can we"),
    ("en.prompt.could_we", 5, "could we"),
    ("en.prompt.can_i", 5, "can i"),
    ("en.prompt.would_it_be_ok_to", 5, "would it be ok to"),
    ("en.prompt.i_d_like", 5, "i'd like"),
    ("en.prompt.want", 5, "want"),
    // "maybe rename it to `log_fuzz_step`": a request put forward gently.
    ("en.prompt.maybe", 11, "maybe"),
    // "can someone explain what `x` does": asked of anyone listening.
    ("en.prompt.can_someone", 14, "can someone"),
    // "I'm thinking `x` should be renamed to `y`"
    ("en.prompt.i_m_thinking", 11, "i'm thinking"),
    // "I'm curious how many ...", "I'm wondering how many ...", "Trying
    // to figure out how many ...": the speaker's wish to know, or "I'm
    // trying to" said without its subject.
    ("en.prompt.i_m_curious", 11, "i'm curious"),
    ("en.prompt.i_m_wondering", 11, "i'm wondering"),
    ("en.prompt.bare_trying_to", 11, "trying to"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let utterance = |id, round, rhs| read_only_rule(id, round, Utterance, rhs, request);
    let mut rules: Vec<_> = INTERJECTIONS
        .iter()
        .map(|&(id, round, words)| filler(id, round, Interjection, words))
        .chain(
            PROMPTS
                .iter()
                .map(|&(id, round, words)| filler(id, round, Prompt, words)),
        )
        .collect();
    rules.extend([
        // "Please, which functions does `fence` call?"
        utterance(
            "en.utterance.interjection_before",
            ROUND,
            vec![C(Interjection), C(Utterance)],
        ),
        // "Could you list the functions that `paced` calls please?"
        utterance(
            "en.utterance.interjection_after",
            ROUND,
            vec![C(Utterance), C(Interjection)],
        ),
        // "Forget that, please."
        utterance(
            "en.utterance.interjection_after_comma",
            6,
            vec![C(Utterance), W(","), C(Interjection)],
        ),
        // "what u keeping in ur context rn, show me", "what u rember of
        // this convo so far, show pls": the prompt said after the request.
        utterance(
            "en.utterance.prompt_after",
            19,
            vec![C(Utterance), W(","), C(Prompt)],
        ),
        // "fields on `alice` struct??": the mark doubled asks no more.
        utterance(
            "en.utterance.question_marks",
            14,
            vec![C(Utterance), W("? ?")],
        ),
        // "Please rename the field `x` to `y`, since it's a flag."
        utterance("en.utterance.stop", 5, vec![C(Utterance), W(".")]),
        // "Can you tell me which functions `pubkey` calls?" ("I'd like to
        // know which functions `among` calls." is read by
        // `en.utterance.question_stop` in locate.rs.)
        utterance("en.utterance.prompt", ROUND, vec![C(Prompt), C(Utterance)]),
    ]);
    rules
}
