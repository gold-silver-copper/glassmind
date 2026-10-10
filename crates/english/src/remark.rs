//! Remarks: what users say about themselves around a request ("I'm
//! reworking the follow feature a bit."). A remark asks nothing, so the
//! request means the same with or without it. A greeting ("Hey, ...")
//! asks nothing either. The system never says one.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Thing};

use crate::asking::read_only_rule;
use crate::lexicon::{CODE, COMPARATIVE, TEXT};
use crate::mention::named;
use crate::sem::{Cat, Sem};
use crate::{blank, first, last, word};

use Cat::*;
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
    let remark_value = || blank();
    // A remark opening with fixed words, then any text up to the next
    // punctuation.
    let opener = |&(id, round, words): &(&'static str, u32, &'static str)| {
        Rule::new(
            RuleInfo::new(id, round),
            Remark,
            vec![W(words), Class(TEXT)],
            S::Const(remark_value()),
        )
    };
    // A remark keeping the thing it is about, which a later "it" refers to.
    let about = |id, rhs| read_only_rule(id, 5, Remark, rhs, first);
    let utterance = |id, round, rhs| read_only_rule(id, round, Utterance, rhs, request);
    let after_comma = |id, round, rhs| read_only_rule(id, round, Utterance, rhs, first);
    // A word that greets the system, before the request it opens.
    let greeting = |id, w| {
        word(
            RuleInfo::new(id, GREETING_ROUND),
            Greeting,
            w,
            remark_value(),
        )
    };
    let mut rules: Vec<_> = OPENERS.iter().map(opener).collect();
    rules.extend([
        // "kick_mps is kind of cryptic for me": what the speaker thinks of
        // a thing, which a later "it" refers to.
        about(
            "en.remark.mention_is",
            vec![C(Mention), W("is"), Class(TEXT)],
        ),
        // Each further word of the remark: "... the voxel code".
        read_only_rule(
            "en.remark.more",
            ROUND,
            Remark,
            vec![C(Remark), Class(TEXT)],
            first,
        ),
        // "`tick` on `App` is a bit cryptic": a remark about a thing.
        about("en.remark.about", vec![C(Mention), Class(TEXT)]),
        // "trying to understand scroll_by", "so the Store has this
        // `write_json`": a remark ending with the thing it is about, named
        // as code.
        read_only_rule(
            "en.remark.ending_about",
            9,
            Remark,
            vec![C(Remark), Class(CODE)],
            |c| named(c.last()),
        ),
        // "I keep seeing Readied around the code": a remark on where the
        // speaker came across a thing, which a later "it" refers to.
        read_only_rule(
            "en.remark.seeing",
            15,
            Remark,
            vec![C(Remark), W("seeing"), C(Mention)],
            last,
        ),
        // "seems it just fires one round at a body, nothing gets shot down
        // really? can we rename it ...": a remark going on past a comma,
        // up to the question mark.
        read_only_rule(
            "en.musing.comma",
            27,
            Musing,
            vec![C(Remark), W(","), Class(TEXT)],
            first,
        ),
        read_only_rule(
            "en.musing.more",
            27,
            Musing,
            vec![C(Musing), Class(TEXT)],
            first,
        ),
        utterance(
            "en.utterance.musing_question",
            27,
            vec![C(Musing), W("?"), C(Utterance)],
        ),
        // "rename fn `x` to `y`, cleaner": what the change makes better,
        // said with no subject or verb.
        read_only_rule(
            "en.remark.comparative",
            27,
            Remark,
            vec![Class(COMPARATIVE)],
            first,
        ),
        // "the name": what a remark about a thing's name calls it.
        word(
            RuleInfo::new("en.name_noun.name", 5),
            NameNoun,
            "name",
            remark_value(),
        ),
        // "Which functions call `x`? i'm trying to understand where it's hooked in"
        utterance(
            "en.utterance.remark_after",
            ROUND,
            vec![C(Utterance), C(Remark)],
        ),
        // "where is the `downloaded` defined, I can't find it". Set off by
        // the comma, a remark may also be about a thing: "please rename
        // `tick` to `x`, tick is kind of vague for me".
        after_comma(
            "en.utterance.remark_after_comma",
            2,
            vec![C(Utterance), W(","), C(Remark)],
        ),
        // "Please rename `x` to `y`, since that describes it better."
        after_comma(
            "en.utterance.remark_after_comma_stop",
            5,
            vec![C(Utterance), W(","), C(Remark), W(".")],
        ),
        // "um, which functions call `x`, if any?"
        after_comma(
            "en.utterance.remark_after_comma_mark",
            22,
            vec![C(Utterance), W(","), C(Remark), C(Mark)],
        ),
        // "Which functions call `inner`? I'm trying to understand how it's used."
        utterance(
            "en.utterance.remark_after_stop",
            ROUND,
            vec![C(Utterance), C(Remark), W(".")],
        ),
        // "what is `x` even for? is it a test?": a remark the speaker is
        // unsure of, after the request.
        utterance(
            "en.utterance.remark_after_mark",
            27,
            vec![C(Utterance), C(Remark), C(Mark)],
        ),
        // "the fn yes() -> bool one": a function's signature in a remark,
        // its arrow two marks.
        read_only_rule(
            "en.remark.arrow",
            27,
            Remark,
            vec![C(Remark), W("- >")],
            first,
        ),
        // "...? I'd like the drawing functions to be named a bit more
        // consistently.", "which functions call `x`? I'd like to check
        // before changing it": what the user wishes for, after the request.
        // Said before one, its words may be the request itself ("I'd like
        // to know what `x` does. Could you explain it?").
        utterance(
            "en.utterance.wish_after",
            11,
            vec![C(Utterance), C(Wishing)],
        ),
        Rule::new(
            RuleInfo::new("en.wishing.i_d_like", 11),
            Wishing,
            vec![W("i'd like"), Class(TEXT)],
            S::Const(remark_value()),
        ),
        Rule::new(
            RuleInfo::new("en.wishing.more", 11),
            Wishing,
            vec![C(Wishing), Class(TEXT)],
            S::Const(remark_value()),
        ),
        // "Hey, quick question: ...", "hey, could you rename ..."
        greeting("en.greeting.hey", "hey"),
        // "Hi, what functions does `local_times_in_explicit_zones` call?"
        greeting("en.greeting.hi", "hi"),
        // What the request is for: "so it's clearer".
        Rule::new(
            RuleInfo::new("en.remark.so", 5),
            Remark,
            vec![W("so"), C(Remark)],
            S::Const(remark_value()),
        ),
    ]);
    // A thing, remark or greeting before the request it goes with, and
    // what separates them.
    rules.extend(
        [
            // "`App::submenu`, rename it `submenu_modal`", "`x` — can you
            // rename it to `y`?": a thing set off before a request about it.
            ("en.utterance.set_off", 5, Mention, C(Break)),
            // "I'm looking at the voxel code. How many functions call
            // `carve`?"
            ("en.utterance.remark_stop", ROUND, Remark, W(".")),
            // "the function trace_step, i think it just logs a line per
            // step? maybe rename it to ...": a remark the user is unsure of.
            ("en.utterance.remark_question", 11, Remark, W("?")),
            // "I'm looking into validation a bit, which functions call
            // `problems`?", "I'm trying to clean up a bit in Editor... could
            // you rename ..."
            ("en.utterance.remark_break", 5, Remark, C(Break)),
            // "I'm working on the UI side and I'd like to know ...", "I'm
            // trying to find where `session` is stored — which structs ..."
            ("en.utterance.remark_and", 5, Remark, W("and")),
            ("en.utterance.remark_dash", 5, Remark, W("—")),
            // "Hey, which functions call `show_with`?": a greeting opening a
            // request.
            ("en.utterance.greeting", GREETING_ROUND, Greeting, W(",")),
            // "Hi! Which functions call `run_menu_row`?"
            (
                "en.utterance.greeting_exclaim",
                GREETING_ROUND,
                Greeting,
                W("!"),
            ),
        ]
        .map(|(id, round, lead, between)| {
            utterance(id, round, vec![C(lead), between, C(Utterance)])
        }),
    );
    // "so in batch_responses, what does it end up calling?": the thing the
    // request is about, set off with the place the user looks in.
    rules.push(
        utterance(
            "en.utterance.set_off_in",
            13,
            vec![W("in"), C(Mention), C(Break), C(Utterance)],
        )
        .read_only(),
    );
    // "Small thing, in `WireDay` I'd like the field `x` to be called `y`":
    // the same, with nothing setting it off.
    rules.push(utterance(
        "en.utterance.set_off_in_bare",
        21,
        vec![W("in"), C(Mention), C(Utterance)],
    ));
    rules
}

/// The words a remark opens with, and the round each came in.
const OPENERS: &[(&str, u32, &str)] = &[
    // "I'm looking at the voxel code": the speaker's own situation.
    ("en.remark.i_am", ROUND, "i'm"),
    // "I keep seeing it around", "i couldn't find it": the same, said with
    // a plain subject.
    ("en.remark.i", 2, "i"),
    // "maybe it's better name"
    ("en.remark.maybe", 11, "maybe"),
    // "name sounds like a getter to me", "would be easier for me to read"
    ("en.remark.name", 11, "name"),
    ("en.remark.would", 11, "would"),
    // "can't find it": the speaker's trouble, without a subject.
    ("en.remark.cannot", 2, "can't"),
    // "cant find it": the same, typed without the apostrophe.
    ("en.remark.cant", 10, "cant"),
    // "still finding my way around here", "still new to this code": the
    // speaker's situation, said without a subject.
    ("en.remark.still", 5, "still"),
    // "since `lost` on its own is pretty vague", "if that's ok": the
    // speaker's reasons and conditions.
    ("en.remark.since", 5, "since"),
    ("en.remark.if", 5, "if"),
    // "so it's clear what it returns": the speaker's purpose.
    ("en.remark.so_it_is", 5, "so it's"),
    // "just so it's more obvious"
    ("en.remark.just", 5, "just"),
    // "the old name confused me a bit", "the current name is kinda
    // confusing": what the speaker thinks of the name being changed.
    ("en.remark.the_old_name", 5, "the old name"),
    ("en.remark.the_current_name", 5, "the current name"),
    // Another subject ("it's more clear", "the name is a bit vague"), or
    // none ("seems more clear to me", "took me a while to get").
    ("en.remark.it", 5, "it"),
    ("en.remark.it_s", 5, "it's"),
    ("en.remark.its", 5, "its"),
    ("en.remark.that_s", 5, "that's"),
    ("en.remark.the", 5, "the"),
    ("en.remark.seems", 5, "seems"),
    ("en.remark.reads", 5, "reads"),
    ("en.remark.took", 5, "took"),
    ("en.remark.think", 5, "think"),
    // After a question about the system's reasons ("not sure I get it",
    // "not sure I follow"), or after a statement ("hope that's fine",
    // "easier for me to remember").
    ("en.remark.not", 6, "not"),
    ("en.remark.hope", 6, "hope"),
    ("en.remark.easier", 6, "easier"),
    // After a question about what a thing holds: what the speaker is after
    // ("trying to understand the struct", "want to see what it holds"), an
    // apology ("sorry, still new to this code"), an aside ("kind of a long
    // name for a struct lol") or the question again in other words ("like
    // what does the struct hold").
    ("en.remark.trying", 7, "trying"),
    ("en.remark.want", 7, "want"),
    ("en.remark.sorry", 7, "sorry"),
    ("en.remark.kind", 7, "kind"),
    ("en.remark.like", 7, "like"),
    // "where is `ask` defined? keep seeing it around", "saw it somewhere
    // and can't find it again": where the speaker came across the thing.
    ("en.remark.keep", 13, "keep"),
    ("en.remark.saw", 13, "saw"),
    // "how many functions call `screen`? kinda lost with this one": how
    // the speaker feels about the code.
    ("en.remark.kinda", 14, "kinda"),
    // "callers of `App::modal`, need the full list", "who calls `x`, need
    // the callers": what the speaker needs, said without a subject.
    ("en.remark.need", 19, "need"),
    // "current name kinda akward", "current name dosnt say waht it does":
    // what the speaker thinks of the name, without "the".
    ("en.remark.current_name", 21, "current name"),
    // "..., current name kinda misleadng", "..., too long, make it `y`":
    // what the speaker thinks of the name being changed.
    ("en.remark.current", 21, "current"),
    ("en.remark.too", 21, "too"),
    // "where is `x` defined? someone mentioned it to me": where the
    // speaker heard of the thing.
    ("en.remark.someone", 25, "someone"),
    // "what does `x` do, quick sumary": how the speaker wants the answer.
    ("en.remark.quick", 25, "quick"),
    // "what fields does `x` have, is it like a wrapper struct around u32?",
    // "what fields `x` has as a struct": what the speaker takes the thing
    // to be.
    ("en.remark.is_it_like", 26, "is it like"),
    ("en.remark.as", 26, "as"),
    // "so there's this `x`, what does it do": a thing the speaker points
    // out.
    ("en.remark.there_s", 27, "there's"),
    // "what is `x` even for? is it a test?": what the speaker guesses the
    // thing is.
    ("en.remark.is_it_a", 27, "is it a"),
    ("en.remark.is_it_an", 27, "is it an"),
];
