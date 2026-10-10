//! Words around a request that ask nothing themselves: interjections that
//! open or close a turn ("hey,", "um,", "quick question:", "thanks!") and
//! prompts that frame it ("can you", "tell me", "I'd like to know"). The
//! request means the same with or without them. The system never says them.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::Ontology;

use crate::sem::{Cat, Sem};

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 2;

/// The request the words go with.
fn request<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find(|v| matches!(v, Sem::Utterance(_))).cloned()
}

/// The question the words go with.
fn question<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find(|v| matches!(v, Sem::Question(_))).cloned()
}

/// Words that open or close a turn.
const INTERJECTIONS: &[(&str, &[&str])] = &[
    ("en.interjection.hey", &["hey"]),
    ("en.interjection.hi", &["hi"]),
    ("en.interjection.hello", &["hello"]),
    ("en.interjection.um", &["um"]),
    ("en.interjection.hmm", &["hmm"]),
    ("en.interjection.ok", &["ok"]),
    ("en.interjection.okay", &["okay"]),
    ("en.interjection.so", &["so"]),
    ("en.interjection.please", &["please"]),
    ("en.interjection.thanks", &["thanks"]),
    ("en.interjection.quick_question", &["quick", "question"]),
];

/// Punctuation that may follow an interjection: "hey,", "quick question:",
/// "thanks!".
const INTERJECTION_MARKS: &[(&str, &str)] = &[
    ("en.interjection.comma", ","),
    ("en.interjection.colon", ":"),
    ("en.interjection.bang", "!"),
    ("en.interjection.stop", "."),
];

/// Words that frame a request: "can you", "tell me", "I'd like to know".
/// They follow each other freely: "can you" "tell me", "I'd like to" "see".
const PROMPTS: &[(&str, &[&str])] = &[
    ("en.prompt.can_you", &["can", "you"]),
    ("en.prompt.could_you", &["could", "you"]),
    ("en.prompt.please", &["please"]),
    ("en.prompt.tell_me", &["tell", "me"]),
    ("en.prompt.show_me", &["show", "me"]),
    ("en.prompt.show", &["show"]),
    ("en.prompt.list", &["list"]),
    ("en.prompt.like_to", &["i'd", "like", "to"]),
    ("en.prompt.trying_to", &["i'm", "trying", "to"]),
    ("en.prompt.know", &["know"]),
    ("en.prompt.see", &["see"]),
    ("en.prompt.figure_out", &["figure", "out"]),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let words = |cat, table: &[(&'static str, &'static [&'static str])]| {
        table
            .iter()
            .map(move |(id, words)| {
                Rule::new(
                    RuleInfo::new(id, ROUND),
                    cat,
                    words.iter().map(|w| W(w)).collect(),
                    S::Const(Sem::Quote(String::new())),
                )
                .read_only()
            })
            .collect::<Vec<_>>()
    };
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let mut rules = words(Cat::Interjection, INTERJECTIONS);
    rules.extend(words(Cat::Prompt, PROMPTS));
    rules.extend(INTERJECTION_MARKS.iter().map(|(id, mark)| {
        rule(
            id,
            Cat::Interjection,
            vec![C(Cat::Interjection), W(mark)],
            |c: &[Sem<O>]| c.first().cloned(),
        )
    }));
    rules.extend([
        // "Hey, which functions does `fence` call?"
        rule(
            "en.utterance.interjection_before",
            Cat::Utterance,
            vec![C(Cat::Interjection), C(Cat::Utterance)],
            request,
        ),
        // "Could you list the functions that `paced` calls? Thanks!"
        rule(
            "en.utterance.interjection_after",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Interjection)],
            request,
        ),
        // "which functions `open_key` calls, please"
        rule(
            "en.question.interjection_after",
            Cat::Question,
            vec![C(Cat::Question), W(","), C(Cat::Interjection)],
            question,
        ),
        // "I'd like to know which functions `among` calls." is read by
        // `en.utterance.question_stop` in locate.rs.
        // "Can you tell me which functions `pubkey` calls?"
        rule(
            "en.utterance.prompt",
            Cat::Utterance,
            vec![C(Cat::Prompt), C(Cat::Utterance)],
            request,
        ),
    ]);
    rules
}
