//! Asides: the words users put around a request that ask nothing ("Hey,
//! quick question: ...", "um, ...", "... Thanks!"). The request means the
//! same with or without them. The system never says one.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::Ontology;

use crate::remark::request;
use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 2;

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let aside = || S::Const(Sem::Quote(String::new()));
    // Greetings, hesitations and thanks, as users open or close a request.
    let aside_word = |(id, words, round): (&'static str, &'static [&'static str], u32)| {
        word(
            RuleInfo::new(id, round),
            Cat::Aside,
            words,
            Sem::Quote(String::new()),
        )
        .read_only()
    };
    let mut rules: Vec<Rule<Cat, Sem<O>>> = [
        ("en.aside.hey", &["hey"][..]),
        ("en.aside.hi", &["hi"]),
        ("en.aside.hello", &["hello"]),
        ("en.aside.ok", &["ok"]),
        ("en.aside.okay", &["okay"]),
        ("en.aside.so", &["so"]),
        ("en.aside.um", &["um"]),
        ("en.aside.uh", &["uh"]),
        ("en.aside.hmm", &["hmm"]),
        ("en.aside.hm", &["hm"]),
        ("en.aside.oh", &["oh"]),
        ("en.aside.ah", &["ah"]),
        ("en.aside.sorry", &["sorry"]),
        ("en.aside.thanks", &["thanks"]),
        ("en.aside.thank_you", &["thank", "you"]),
        ("en.aside.quick_question", &["quick", "question"]),
        ("en.aside.quick_one", &["quick", "one"]),
    ]
    .into_iter()
    .map(|(id, words)| aside_word((id, words, ROUND)))
    .collect();
    // "Quick request: please rename ...", "... Thanks a lot!"
    rules.push(aside_word((
        "en.aside.quick_request",
        &["quick", "request"],
        5,
    )));
    rules.push(aside_word((
        "en.aside.thanks_a_lot",
        &["thanks", "a", "lot"],
        5,
    )));
    rules.push(aside_word((
        "en.aside.thanks_in_advance",
        &["thanks", "in", "advance"],
        5,
    )));
    // The punctuation that ends an aside: "hey,", "hi!", "quick question:",
    // "sorry, quick one -", "Thanks.".
    for (id, mark) in [
        ("en.aside.comma", ","),
        ("en.aside.bang", "!"),
        ("en.aside.colon", ":"),
        ("en.aside.stop", "."),
        ("en.aside.dash", "-"),
        ("en.aside.em_dash", "—"),
    ] {
        rules.push(
            Rule::new(
                RuleInfo::new(id, ROUND),
                Cat::Aside,
                vec![C(Cat::Aside), W(mark)],
                aside(),
            )
            .read_only(),
        );
    }
    // "Hey, where is `x` defined?" and "Where is `x` defined? Thanks!"
    for (id, rhs) in [
        (
            "en.utterance.aside_before",
            vec![C(Cat::Aside), C(Cat::Utterance)],
        ),
        (
            "en.utterance.aside_after",
            vec![C(Cat::Utterance), C(Cat::Aside)],
        ),
    ] {
        rules.push(
            Rule::new(
                RuleInfo::new(id, ROUND),
                Cat::Utterance,
                rhs,
                S::Build(request),
            )
            .read_only(),
        );
    }
    rules
}
