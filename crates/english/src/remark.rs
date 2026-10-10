//! Remarks: what users say about themselves around a request ("I'm
//! reworking the follow feature a bit."). A remark asks nothing, so the
//! request means the same with or without it. A greeting ("Hey, ...")
//! asks nothing either. The system never says one.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Thing};

use crate::lexicon::TEXT;
use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 1;
const GREETING_ROUND: u32 = 2;

/// The request a remark goes with. A remark about a thing before it ("`x`
/// is a bit vague. Can you rename it?") is what the request's "it" refers
/// back to. One after it is not taken as a remark: its words may be the
/// rest of the request ("where's `x` getting called from").
pub(crate) fn request<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let mut topic = None;
    for v in c {
        match (v, topic) {
            (Sem::Mention { thing, .. }, _) => topic = Some(thing),
            (Sem::Utterance(u), Some(topic)) => {
                return Some(Sem::Utterance(u.map_things(&mut |t| {
                    if *t == Thing::Previous {
                        topic.clone()
                    } else {
                        t.clone()
                    }
                })));
            }
            (Sem::Utterance(_), None) => {
                return c
                    .iter()
                    .all(|v| !matches!(v, Sem::Mention { .. }))
                    .then(|| v.clone());
            }
            _ => {}
        }
    }
    None
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
    let mut rules = vec![
        // "I'm looking at the voxel code": the speaker's own situation, up
        // to the next punctuation.
        Rule::new(
            RuleInfo::new("en.remark.i_am", ROUND),
            Cat::Remark,
            vec![W("i'm"), Class(TEXT)],
            remark(),
        ),
        // "I keep seeing it around", "i couldn't find it": the same, said
        // with a plain subject.
        Rule::new(
            RuleInfo::new("en.remark.i", 2),
            Cat::Remark,
            vec![W("i"), Class(TEXT)],
            remark(),
        ),
        // "can't find it": the speaker's trouble, without a subject.
        Rule::new(
            RuleInfo::new("en.remark.cannot", 2),
            Cat::Remark,
            vec![W("can't"), Class(TEXT)],
            remark(),
        ),
        // "kick_mps is kind of cryptic for me": what the speaker thinks of
        // a thing, which a later "it" refers to.
        Rule::new(
            RuleInfo::new("en.remark.mention_is", 5),
            Cat::Remark,
            vec![C(Cat::Mention), W("is"), Class(TEXT)],
            S::Build(|c| c.first().cloned()),
        ),
        // "still finding my way around here", "still new to this code": the
        // speaker's situation, said without a subject.
        Rule::new(
            RuleInfo::new("en.remark.still", 5),
            Cat::Remark,
            vec![W("still"), Class(TEXT)],
            remark(),
        ),
        // Each further word of the remark: "... the voxel code".
        Rule::new(
            RuleInfo::new("en.remark.more", ROUND),
            Cat::Remark,
            vec![C(Cat::Remark), Class(TEXT)],
            S::Build(|c| c.first().cloned()),
        ),
        // "`tick` on `App` is a bit cryptic": a remark about a thing.
        Rule::new(
            RuleInfo::new("en.remark.about", 5),
            Cat::Remark,
            vec![C(Cat::Mention), Class(TEXT)],
            S::Build(|c| c.first().cloned()),
        ),
        // "since `lost` on its own is pretty vague", "if that's ok": the
        // speaker's reasons and conditions.
        Rule::new(
            RuleInfo::new("en.remark.since", 5),
            Cat::Remark,
            vec![W("since"), Class(TEXT)],
            remark(),
        ),
        Rule::new(
            RuleInfo::new("en.remark.if", 5),
            Cat::Remark,
            vec![W("if"), Class(TEXT)],
            remark(),
        ),
        word(
            RuleInfo::new("en.name_noun.name", 5),
            Cat::NameNoun,
            &["name"],
            remark_value(),
        ),
        word(
            RuleInfo::new("en.name_noun.current_name", 5),
            Cat::NameNoun,
            &["current", "name"],
            remark_value(),
        ),
        word(
            RuleInfo::new("en.name_noun.plain_name", 5),
            Cat::NameNoun,
            &["plain", "name"],
            remark_value(),
        ),
        // "so it's clear what it returns": the speaker's purpose.
        Rule::new(
            RuleInfo::new("en.remark.so_it_is", 5),
            Cat::Remark,
            vec![W("so"), W("it's"), Class(TEXT)],
            remark(),
        ),
        // "maybe it's a better name", "just so it's more obvious"
        Rule::new(
            RuleInfo::new("en.remark.maybe", 5),
            Cat::Remark,
            vec![W("maybe"), Class(TEXT)],
            remark(),
        ),
        Rule::new(
            RuleInfo::new("en.remark.just", 5),
            Cat::Remark,
            vec![W("just"), Class(TEXT)],
            remark(),
        ),
        // "the current name is a bit vague", "the plain name confused me":
        // a remark about the name being changed.
        Rule::new(
            RuleInfo::new("en.remark.the_name", 5),
            Cat::Remark,
            vec![W("the"), C(Cat::NameNoun), Class(TEXT)],
            remark(),
        ),
        // "`App::submenu`, rename it `submenu_modal`", "`x` — can you
        // rename it to `y`?": a thing set off before a request about it.
        utterance(
            "en.utterance.set_off",
            5,
            vec![C(Cat::Mention), C(Cat::Break), C(Cat::Utterance)],
        ),
        // "I'm looking at the voxel code. How many functions call `carve`?"
        utterance(
            "en.utterance.remark_stop",
            ROUND,
            vec![C(Cat::Remark), W("."), C(Cat::Utterance)],
        ),
        // "I'm looking into validation a bit, which functions call
        // `problems`?", "I'm trying to clean up a bit in Editor... could you
        // rename ..."
        utterance(
            "en.utterance.remark_break",
            5,
            vec![C(Cat::Remark), C(Cat::Break), C(Cat::Utterance)],
        ),
        // "I'm working on the UI side and I'd like to know ...", "I'm
        // trying to find where `session` is stored — which structs ..."
        utterance(
            "en.utterance.remark_and",
            5,
            vec![C(Cat::Remark), W("and"), C(Cat::Utterance)],
        ),
        utterance(
            "en.utterance.remark_dash",
            5,
            vec![C(Cat::Remark), W("—"), C(Cat::Utterance)],
        ),
        // "Which functions call `x`? i'm trying to understand where it's hooked in"
        utterance(
            "en.utterance.remark_after",
            ROUND,
            vec![C(Cat::Utterance), C(Cat::Remark)],
        ),
        // "where is the `downloaded` defined, I can't find it"
        Rule::new(
            RuleInfo::new("en.utterance.remark_after_comma", 2),
            Cat::Utterance,
            vec![C(Cat::Utterance), W(","), C(Cat::Remark)],
            S::Build(request),
        ),
        // "Please rename `x` to `y`, since that describes it better."
        utterance(
            "en.utterance.remark_after_comma_stop",
            5,
            vec![C(Cat::Utterance), W(","), C(Cat::Remark), W(".")],
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
        // "I'm going through the chat code, and I'd like ..."
        Rule::new(
            RuleInfo::new("en.utterance.remark_comma_and", 5),
            Cat::Utterance,
            vec![C(Cat::Remark), W(","), W("and"), C(Cat::Utterance)],
            S::Build(request),
        ),
        // "Hey, quick question: ...", "hey, could you rename ..."
        greeting("en.greeting.hey", "hey"),
        // "Hi, what functions does `local_times_in_explicit_zones` call?"
        greeting("en.greeting.hi", "hi"),
    ];
    // "the old name confused me a bit", "the current name is kinda
    // confusing": what the speaker thinks of the name being changed.
    for (id, subject) in [
        ("en.remark.the_old_name", "old"),
        ("en.remark.the_current_name", "current"),
    ] {
        rules.push(Rule::new(
            RuleInfo::new(id, 5),
            Cat::Remark,
            vec![W("the"), W(subject), W("name"), Class(TEXT)],
            remark(),
        ));
    }
    // Other ways a remark opens: with another subject ("it's more clear",
    // "the name is a bit vague"), or without one ("seems more clear to me",
    // "took me a while to get").
    let openers = OPENERS.iter().map(|o| (o, 5));
    rules.extend(
        openers
            .chain(MORE_OPENERS.iter().map(|o| (o, 6)))
            .map(|((id, w), round)| {
                Rule::new(
                    RuleInfo::new(id, round),
                    Cat::Remark,
                    vec![W(w), Class(TEXT)],
                    remark(),
                )
            }),
    );
    // What the request is for: "so it's clearer".
    rules.push(Rule::new(
        RuleInfo::new("en.remark.so", 5),
        Cat::Remark,
        vec![W("so"), C(Cat::Remark)],
        remark(),
    ));
    rules
}

/// Words a remark can open with, besides those above.
const OPENERS: &[(&str, &str)] = &[
    ("en.remark.it", "it"),
    ("en.remark.it_s", "it's"),
    ("en.remark.its", "its"),
    ("en.remark.that_s", "that's"),
    ("en.remark.the", "the"),
    ("en.remark.seems", "seems"),
    ("en.remark.reads", "reads"),
    ("en.remark.took", "took"),
    ("en.remark.think", "think"),
];

/// Words a remark can open with after a question about the system's
/// reasons: "not sure I get it", "not sure I follow".
const MORE_OPENERS: &[(&str, &str)] = &[("en.remark.not", "not")];
