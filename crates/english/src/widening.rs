//! Widening remarks: after a question about things holding a thing, users
//! say the thing counts when it is held inside another ("which structs have
//! a `Gate` field, incl wrapped types", "... in a field? also counts if
//! it's inside some other type, like Option or whatever", "structs w/
//! fields typed `Message`, incl Vec<Message> etc", "directly or nested").
//!
//! A held thing is held wherever it is written in what holds it, so the
//! remark asks for nothing the question does not already ask: it keeps the
//! question as it is. It only goes with a question about things holding
//! things fitting a description, which is what it speaks of. The system
//! never says one.

use glassmind_grammar::{Rule, Sym};
use glassmind_meaning::{Condition, Ontology, Question, Thing, Utterance};

use crate::asking::{read_only_rule, same};
use crate::lexicon::NAME;
use crate::locate::fillers;
use crate::sem::{Cat, Sem};

use Cat::{
    Counted, Directly, Enclosure, Hedge, Includes, Inclusion, Noun, NounPhrase, Prompt, RoleNoun,
    Widening, Wrapped, Wrapping,
};
use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 9;

/// The question among the children, if it asks for things holding
/// something that fits a description ("structs with a field of type `x`"):
/// what a widening remark after it speaks of.
fn widened<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let first = c.first()?;
    let (Sem::Question(Question::Identify(d) | Question::Count(d))
    | Sem::Utterance(Utterance::Ask(Question::Identify(d) | Question::Count(d)))) = first
    else {
        return None;
    };
    d.conditions
        .iter()
        .any(|c| {
            matches!(
                c,
                Condition::Related {
                    other: Thing::Fitting(_),
                    ..
                }
            )
        })
        .then(|| first.clone())
}

/// A question with a widening remark after it: "which structs have a
/// `Gate` field, incl wrapped types", "... a field with `usize` in the type
/// (directly or inside like Vec or Option)?", "... in it? like
/// Vec<DiscussionComment> also counts".
const WIDENED: &[(&str, Cat, &[Sym<Cat>])] = &[
    (
        "en.question.widened",
        Cat::Question,
        &[C(Cat::Question), C(Widening)],
    ),
    (
        "en.question.widened_comma",
        Cat::Question,
        &[C(Cat::Question), W(","), C(Widening)],
    ),
    (
        "en.utterance.widened",
        Cat::Utterance,
        &[C(Cat::Utterance), C(Widening)],
    ),
    (
        "en.utterance.widened_comma",
        Cat::Utterance,
        &[C(Cat::Utterance), W(","), C(Widening)],
    ),
];

/// The parts of a widening remark, each meaning no more than its first
/// part does.
const PARTS: &[(&str, Cat, &[Sym<Cat>])] = &[
    // "(direct or nested)"
    (
        "en.widening.aside",
        Widening,
        &[W("("), C(Widening), W(")")],
    ),
    // "wrapped", "nested in generics", "inside some other type".
    ("en.widening.wrapping", Widening, &[C(Wrapping)]),
    // "incl wrapped", "also ones where ...", "like even if ...".
    (
        "en.widening.inclusion",
        Widening,
        &[C(Inclusion), C(Widening)],
    ),
    // "incl Vec<Message> etc", "include containers like Vec".
    (
        "en.widening.inclusion_enclosure",
        Widening,
        &[C(Inclusion), C(Enclosure)],
    ),
    // "Option<u16> etc count too", "Just Keypair or wrapped, both count".
    (
        "en.widening.enclosure_counted",
        Widening,
        &[C(Enclosure), C(Counted)],
    ),
    (
        "en.widening.comma_counted",
        Widening,
        &[C(Widening), W(","), C(Counted)],
    ),
    ("en.widening.etc", Widening, &[C(Widening), W("etc")]),
    // "incl. wrapped, Option<Art> etc", "... inside some other type, like
    // Option or whatever".
    (
        "en.widening.comma_enclosure",
        Widening,
        &[C(Widening), W(","), C(Enclosure)],
    ),
    // "directly or wrapped", "direct or nested".
    (
        "en.widening.directly_or",
        Widening,
        &[C(Directly), W("or"), C(Widening)],
    ),
    // "if it's wrapped in other type", "when it's wrapped in something",
    // "ones where it's inside some other type", "fields where `Post` is
    // wrapped in something else", "ones where the type just contains Key".
    ("en.widening.if", Widening, &[W("if"), C(Wrapped)]),
    ("en.widening.when", Widening, &[W("when"), C(Wrapped)]),
    (
        "en.widening.ones_where",
        Widening,
        &[W("ones"), W("where"), C(Wrapped)],
    ),
    (
        "en.widening.kind_where",
        Widening,
        &[C(Noun), W("where"), C(Wrapped)],
    ),
    ("en.wrapped.it_s", Wrapped, &[W("it's"), C(Widening)]),
    (
        "en.wrapped.is",
        Wrapped,
        &[C(NounPhrase), W("is"), C(Widening)],
    ),
    (
        "en.wrapped.role_includes",
        Wrapped,
        &[C(RoleNoun), C(Includes), C(NounPhrase)],
    ),
    // "wrapped in another type", "inside a Vec or something", "nested like
    // Vec<Arm>".
    (
        "en.wrapping.enclosure",
        Wrapping,
        &[C(Wrapping), C(Enclosure)],
    ),
    // What a thing is inside: a type by name ("Vec", "`Option<u64>`",
    // "Option<...>"), any type ("another type", "some other type"), or
    // several ("Option/Vec etc", "a Vec or something").
    ("en.enclosure.name", Enclosure, &[Class(NAME)]),
    ("en.enclosure.role", Enclosure, &[C(RoleNoun)]),
    (
        "en.enclosure.generic",
        Enclosure,
        &[C(Enclosure), W("<"), C(Enclosure), W(">")],
    ),
    (
        "en.enclosure.generic_elided",
        Enclosure,
        &[C(Enclosure), W("<"), W("."), W("."), W("."), W(">")],
    ),
    ("en.enclosure.etc", Enclosure, &[C(Enclosure), W("etc")]),
    ("en.enclosure.like", Enclosure, &[W("like"), C(Enclosure)]),
    (
        "en.enclosure.or",
        Enclosure,
        &[C(Enclosure), W("or"), C(Enclosure)],
    ),
    (
        "en.enclosure.slash",
        Enclosure,
        &[C(Enclosure), W("/"), C(Enclosure)],
    ),
    ("en.enclosure.a", Enclosure, &[W("a"), C(Enclosure)]),
    ("en.enclosure.an", Enclosure, &[W("an"), C(Enclosure)]),
    (
        "en.enclosure.another",
        Enclosure,
        &[W("another"), C(Enclosure)],
    ),
    ("en.enclosure.other", Enclosure, &[W("other"), C(Enclosure)]),
    ("en.enclosure.some", Enclosure, &[W("some"), C(Enclosure)]),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let mut rules: Vec<Rule<Cat, Sem<O>>> = Vec::new();
    for (table, build) in [(WIDENED, widened as fn(&[Sem<O>]) -> _), (PARTS, same)] {
        rules.extend(
            table
                .iter()
                .map(|&(id, lhs, rhs)| read_only_rule(id, ROUND, lhs, rhs.to_vec(), build)),
        );
    }
    // "the type just contains Key"
    rules.push(read_only_rule(
        "en.includes.hedged",
        ROUND,
        Includes,
        vec![C(Hedge), C(Includes)],
        crate::last,
    ));
    rules.extend(fillers(
        ROUND,
        &[
            (
                Inclusion,
                &[
                    ("en.inclusion.including", "including"),
                    ("en.inclusion.incl", "incl"),
                    ("en.inclusion.incl_stop", "incl ."),
                    ("en.inclusion.include", "include"),
                    ("en.inclusion.also", "also"),
                    ("en.inclusion.also_counts", "also counts"),
                    ("en.inclusion.counting", "counting"),
                    ("en.inclusion.even", "even"),
                    ("en.inclusion.like", "like"),
                ],
            ),
            (
                Wrapping,
                &[
                    ("en.wrapping.wrapped", "wrapped"),
                    ("en.wrapping.nested", "nested"),
                    ("en.wrapping.boxed", "boxed"),
                    ("en.wrapping.inside", "inside"),
                    ("en.wrapping.in", "in"),
                ],
            ),
            (
                Enclosure,
                &[
                    ("en.enclosure.something", "something"),
                    ("en.enclosure.something_else", "something else"),
                    ("en.enclosure.whatever", "whatever"),
                ],
            ),
            (
                Counted,
                &[
                    ("en.counted.count", "count"),
                    ("en.counted.count_too", "count too"),
                    ("en.counted.also_counts", "also counts"),
                    ("en.counted.both_count", "both count"),
                ],
            ),
            (
                Directly,
                &[
                    ("en.directly.directly", "directly"),
                    ("en.directly.direct", "direct"),
                ],
            ),
            (Hedge, &[("en.hedge.just", "just")]),
            // "Is there a way to see which structs ...", "is there a list of
            // structs that ..."
            (
                Prompt,
                &[
                    ("en.prompt.is_there_a_way_to", "is there a way to"),
                    ("en.prompt.is_there_a_list_of", "is there a list of"),
                ],
            ),
            // "I'm trying to figure out where BoardConfig is held: which
            // structs ..."
            (Cat::Break, &[("en.break.colon", ":")]),
        ],
    ));
    rules
}
