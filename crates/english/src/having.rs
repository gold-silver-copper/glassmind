//! Describing things by what they have: "which structs have a field whose
//! type is or contains `Arc`?", "structs w/ a field of type `Tab`", "fields
//! typed `Art`".
//!
//! A noun with a qualifier ("a field whose type is `Arc`") is any thing
//! fitting a description, so it can stand as the object of a verb. A
//! qualifier names one side of a relation with a role noun ("type") and
//! says what fills it.

use glassmind_grammar::{Rule, RuleInfo, Sym};
use glassmind_meaning::{self as meaning, Condition, Ontology, Position, Thing};

use crate::aside::filler;
use crate::asking::{identify, meeting, read_only_rule, related, words_then};
use crate::lexicon::CAPITALIZED;
use crate::sem::{Cat, Number, Sem};
use crate::{blank, first, last, spoken, word};

use Cat::*;
use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 3;
const GROWN: u32 = 9;
const LATER: u32 = 11;
const NOW: u32 = 13;
const ROUND_14: u32 = 14;

/// The request asked again: as it was, or, when asked again for things of
/// a kind ("which functions?"), for those of that kind among what it asks.
/// Asked again by a role ("who uses `x`, callers only"), it must already ask
/// for the things in that role.
fn again<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    use meaning::{Question::Identify, Utterance::Ask};
    match c {
        [Sem::Utterance(Ask(Identify(d))), Sem::Kind { kind, .. }]
            if d.kind.as_ref().is_none_or(|k| k == kind) =>
        {
            let mut d = d.clone();
            d.kind = Some(kind.clone());
            Some(Sem::Utterance(Ask(Identify(d))))
        }
        [_, Sem::Kind { .. }] => None,
        [
            Sem::Utterance(Ask(Identify(d))),
            Sem::Role { relation, position },
        ] => d
            .conditions
            .iter()
            .any(|c| {
                matches!(c, Condition::Related { relation: r, position: p, .. }
                    if r == relation && p == position)
            })
            .then(|| c.first().cloned())
            .flatten(),
        [_, Sem::Role { .. }] => None,
        _ => c.first().cloned(),
    }
}

/// Things of any kind meeting the condition last among the children.
pub(crate) fn anything<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c.last()? {
        Sem::Condition { condition, .. } => {
            Some(Sem::Description(meeting(None, condition.clone())))
        }
        _ => None,
    }
}

/// The other side of a two-place relation.
fn flip(position: Position) -> Position {
    match position {
        Position::First => Position::Second,
        Position::Second => Position::First,
    }
}

/// "fields in `S`": things of the kind standing in the preposition's
/// relation to the thing.
fn placed<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [
            Sem::Kind { kind, .. },
            Sem::Relation { relation, .. },
            Sem::Thing(other),
        ] => Some(Sem::Description(meeting(
            Some(kind),
            related(relation, Position::First, other),
        ))),
        _ => None,
    }
}

/// "whose type is `x`", "of type `x`", "typed `x`": the described thing's
/// role (the first child) is filled by `x` (the last), so it fills the
/// other side of the role's relation.
fn qualified<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Role { relation, position }, .., Sem::Thing(other)] => Some(Sem::Condition {
            condition: related(relation, flip(*position), other),
            number: Number::Plural,
        }),
        _ => None,
    }
}

/// The role, the wording and the thing a qualifier speaks.
fn unqualified<O: Ontology>(v: &Sem<O>) -> Option<Vec<Sem<O>>> {
    match v {
        Sem::Condition {
            condition:
                Condition::Related {
                    relation,
                    position,
                    other,
                },
            ..
        } => Some(vec![
            Sem::Role {
                relation: relation.clone(),
                position: flip(*position),
            },
            blank(),
            Sem::Thing(other.clone()),
        ]),
        _ => None,
    }
}

/// Whether what is said again speaks only of the given things, by name or
/// as "it", wherever it names them: "or a field that contains it".
fn speaks_of<O: Ontology>(again: &Sem<O>, things: &[&Thing<O>]) -> bool {
    match again {
        Sem::Thing(t) => t
            .things()
            .iter()
            .all(|t| **t == Thing::Previous || things.contains(t)),
        _ => true,
    }
}

/// The qualifier among the children, if what else they say agrees with it:
/// an alternative must speak of the same thing, by name or as "it".
fn agreeing<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [
            q @ (Sem::Condition {
                condition: Condition::Related { other, .. },
                ..
            }
            | Sem::Thing(other)),
            again,
        ] if speaks_of(again, &other.things()) => Some(q.clone()),
        _ => None,
    }
}

/// Any thing of a kind meeting a condition: "a field whose type is `x`".
fn fitting<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Kind { kind, .. }, Sem::Condition { condition, .. }] => Some(Sem::Thing(
            Thing::Fitting(meeting(Some(kind), condition.clone())),
        )),
        _ => None,
    }
}

/// "a field that is `x`", "a `x` field", "a field that contains it": any
/// thing of a holder's kind holding `x`, which comes after or before the
/// holder.
fn held<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [
            Sem::Holder {
                kind,
                relation,
                position,
            },
            Sem::Thing(other),
        ]
        | [
            Sem::Thing(other),
            Sem::Holder {
                kind,
                relation,
                position,
            },
        ] if matches!(other, Thing::Named(_) | Thing::Item(_) | Thing::Previous) => {
            Some(Sem::Thing(Thing::Fitting(meeting(
                Some(kind),
                related(relation, flip(*position), other),
            ))))
        }
        _ => None,
    }
}

/// "with `x` in its type", "with `x` type", "with `x` in it": a thing
/// inside what fills a role (said after it), or inside the holder itself.
fn holding<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [.., thing, role @ Sem::Role { .. }] => qualified(&[role.clone(), thing.clone()]),
        [thing @ Sem::Thing(_), _] => Some(thing.clone()),
        _ => None,
    }
}

/// The thing a qualifier speaks of: "(or) with a type that contains it".
fn spoken_of<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [
            Sem::Condition {
                condition: Condition::Related { other, .. },
                ..
            }
            | Sem::Thing(other),
        ] => Some(Sem::Thing(other.clone())),
        _ => None,
    }
}

/// Whether a holder and a role noun after it name the same side of the
/// same relation: "field type".
fn holder_role<O: Ontology>(c: &[Sem<O>]) -> bool {
    matches!(c, [
        Sem::Holder { relation, position, .. },
        Sem::Role { relation: r, position: p },
    ] if relation == r && position == p)
}

/// A question again with another way of putting what it asks about, said
/// after it ("which structs have a field of type `Session`? or something
/// containing it"): the alternative must speak of a thing the question
/// does, by name or as "it".
fn put_again<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Utterance(u), again] if speaks_of(again, &u.things()) => {
            Some(Sem::Utterance(u.clone()))
        }
        _ => None,
    }
}

/// "a field of type `x`, what structs have that": the question for the
/// things related to any thing fitting a description set off before it,
/// which its "that" refers back to. A thing described is no one thing, so
/// only a question for things related to it asks about it.
fn set_off_fitting<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [
            Sem::Thing(fitting @ Thing::Fitting(_)),
            ..,
            Sem::Question(later @ (meaning::Question::Identify(_) | meaning::Question::Count(_))),
        ] if later.things().contains(&&Thing::Previous) => {
            Some(Sem::Question(later.map_things(&mut |t| match t {
                Thing::Previous => fitting.clone(),
                other => other.clone(),
            })))
        }
        _ => None,
    }
}

/// "where is `x` used as a field type": the things having a part (a field)
/// whose role (its type) `x` fills.
fn used_as<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [
            Sem::Thing(thing),
            Sem::Holder {
                kind,
                relation,
                position,
            },
            Sem::Role {
                relation: role,
                position: filled,
            },
        ] => {
            let part = meeting(Some(kind), related(role, flip(*filled), thing));
            Some(Sem::Question(meaning::Question::Identify(meeting(
                None,
                related(relation, flip(*position), &Thing::Fitting(part)),
            ))))
        }
        _ => None,
    }
}

/// "`tree_scroll` field": what has a part going by the name.
fn part_holders<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [
            Sem::Thing(Thing::Named(name)),
            Sem::Holder {
                kind,
                relation,
                position,
            },
        ] => Some(Sem::Description(meeting(
            None,
            related(
                relation,
                flip(*position),
                &Thing::Fitting(meeting(Some(kind), Condition::Named(name.clone()))),
            ),
        ))),
        _ => None,
    }
}

/// Words between a thing and a holder it is in: "a `LineCache` in a
/// field", "`Poster` as field type", "a Spot somewhere in their fields".
const LODGINGS: &[(&str, &str)] = &[
    ("en.noun_phrase.in_holder", "in"),
    ("en.noun_phrase.as_holder", "as"),
    ("en.noun_phrase.somewhere_in_holder", "somewhere in"),
];

/// Ways of saying a role's filler is or includes a thing. A field's type
/// "contains" `Arc` when `Arc` is written anywhere in it, so "is or
/// contains" and "contains" say the same. The system says "is or contains"
/// (`en.includes.is_or_contains`, made in [`rules`]); these it only reads.
const INCLUDES: &[(&str, &str)] = &[
    ("en.includes.is", "is"),
    ("en.includes.contains", "contains"),
    ("en.includes.includes", "includes"),
    ("en.includes.wraps", "wraps"),
    ("en.includes.containing", "containing"),
    ("en.includes.wrapping", "wrapping"),
    ("en.includes.holding", "holding"),
    ("en.includes.is_or_includes", "is or includes"),
    ("en.includes.is_or_wraps", "is or wraps"),
    ("en.includes.is_or_contains_aside", "is ( or contains )"),
    ("en.includes.is_comma_or_contains", "is , or contains ,"),
];

/// More ways of saying a role's filler includes a thing: "whose field
/// types include `Sort`", "a field with a type that is, or includes, `x`",
/// "whose type has `GistFile` in it", "fields that use `PostText`".
const INCLUDES_LATER: &[(&str, &str)] = &[
    ("en.includes.include", "include"),
    ("en.includes.is_comma_or_includes", "is , or includes ,"),
    ("en.includes.has", "has"),
    ("en.includes.use", "use"),
];

/// Ways of saying a role's filler includes a thing, from round 19: "a
/// field that uses `QuerySet`".
const INCLUDES_19: &[(&str, &str)] = &[("en.includes.uses", "uses")];

/// Words saying where in a thing another is, after a qualifier: "or wraps
/// it somehow", "or contains u32 in it", "with type IRect in it".
const WITHIN: &[(&str, &str)] = &[
    ("en.within.somehow", "somehow"),
    ("en.within.somewhere", "somewhere"),
    ("en.within.in_it", "in it"),
    ("en.within.inside_it", "inside it"),
    ("en.within.inside", "inside"),
];

/// Words before a role noun saying a thing is inside what fills the role:
/// "in its type", "somewhere in the type".
const WITHIN_ROLE: &[(&str, &str)] = &[
    ("en.within.in_role", "in"),
    ("en.within.inside_role", "inside"),
];

/// Questions after a question that ask again for what it asks about.
const ASK_AGAIN: &[(&str, &str)] = &[
    ("en.ask_again.which_ones", "which ones"),
    ("en.ask_again.are_there_any", "are there any"),
    ("en.ask_again.is_there_any", "is there any"),
    ("en.ask_again.can_you_list_them", "can you list them"),
];

/// More questions asking again: "... like &str or Box<str>. which ones
/// are there?"
const ASK_AGAIN_LATER: &[(&str, &str)] =
    &[("en.ask_again.which_ones_are_there", "which ones are there")];

/// Asking again for the things just asked about, from round 18:
/// "`thread_every` fields, what are they", "fields on `read` struct, what
/// are they".
const ASK_AGAIN_18: &[(&str, &str)] = &[("en.ask_again.what_are_they", "what are they")];

/// Words for some thing of any kind: "anything", "something".
const ANYTHING: &[(&str, &str)] = &[
    ("en.anything.anything", "anything"),
    ("en.anything.something", "something"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| read_only_rule(id, ROUND, lhs, rhs, build);
    let mut rules: Vec<Rule<Cat, Sem<O>>> = vec![
        // "a field whose type is or contains `x`": how the system says it.
        spoken(
            "en.noun_phrase.qualified",
            ROUND,
            NounPhrase,
            words_then("a", &[Noun, Qualifier]),
            fitting,
            |v| match v {
                Sem::Thing(Thing::Fitting(d)) => match d.conditions.as_slice() {
                    [condition] => Some(vec![
                        Sem::Kind {
                            kind: d.kind.clone()?,
                            number: Number::Singular,
                        },
                        Sem::Condition {
                            condition: condition.clone(),
                            number: Number::Singular,
                        },
                    ]),
                    _ => None,
                },
                _ => None,
            },
        ),
        // "fields typed `Art`", "have field whose type is `Gone`".
        rule(
            "en.noun_phrase.bare_qualified",
            NounPhrase,
            vec![C(Noun), C(Qualifier)],
            fitting,
        ),
        // "whose type is or contains `x`"
        spoken(
            "en.qualifier.whose",
            ROUND,
            Qualifier,
            words_then("whose", &[RoleNoun, Includes, Filling]),
            qualified,
            unqualified,
        ),
        // "of type `x`", "with type `x`", "with a type of `x`", "typed `x`".
        rule(
            "en.qualifier.of",
            Qualifier,
            words_then("of", &[RoleNoun, Filling]),
            qualified,
        ),
        rule(
            "en.qualifier.with",
            Qualifier,
            words_then("with", &[RoleNoun, Filling]),
            qualified,
        ),
        rule(
            "en.qualifier.with_role_of",
            Qualifier,
            vec![W("with"), C(RoleNoun), W("of"), C(Filling)],
            qualified,
        ),
        // "with type that is or contains `x`"
        rule(
            "en.qualifier.with_role_that",
            Qualifier,
            vec![W("with"), C(RoleNoun), W("that"), C(Includes), C(Filling)],
            qualified,
        ),
        rule(
            "en.qualifier.adjective",
            Qualifier,
            vec![C(RoleAdjective), C(Filling)],
            qualified,
        ),
        // A qualifier naming no role, after a holder: "(a field) that is
        // `Key`", "containing `Instant`", "that's a `u16`".
        rule(
            "en.qualifier.that",
            Qualifier,
            words_then("that", &[Includes, Filling]),
            last,
        ),
        rule(
            "en.qualifier.that_s",
            Qualifier,
            words_then("that's", &[Filling]),
            last,
        ),
        rule(
            "en.qualifier.includes",
            Qualifier,
            vec![C(Includes), C(Filling)],
            last,
        ),
        // "a field that is `Key`", "fields containing `Signature`"
        rule(
            "en.noun_phrase.holder",
            NounPhrase,
            vec![C(Holder), C(Qualifier)],
            held,
        ),
        // "a `DraftSide` field", "a BigInt field"
        rule(
            "en.noun_phrase.compound",
            NounPhrase,
            vec![C(NounPhrase), C(Holder)],
            held,
        ),
        // "with type IRect in it"
        rule(
            "en.qualifier.within",
            Qualifier,
            vec![C(Qualifier), C(Within)],
            first,
        ),
        // "whose type is `Tab` or contains it"
        rule(
            "en.qualifier.alternative",
            Qualifier,
            vec![C(Qualifier), C(Alternative)],
            agreeing,
        ),
        // "or contains it", "or wraps a `Notify`", "or has Arc somewhere in
        // the type", "or something wrapping it", "or a type that contains it".
        rule(
            "en.alternative.or",
            Alternative,
            words_then("or", &[Includes, Again]),
            last,
        ),
        rule(
            "en.alternative.or_has",
            Alternative,
            words_then("or has", &[Again]),
            last,
        ),
        rule(
            "en.alternative.or_something",
            Alternative,
            words_then("or something", &[Includes, Again]),
            last,
        ),
        rule(
            "en.alternative.or_something_that",
            Alternative,
            words_then("or something that", &[Includes, Again]),
            last,
        ),
        rule(
            "en.alternative.or_role_that",
            Alternative,
            vec![W("or"), C(RoleNoun), W("that"), C(Includes), C(Again)],
            last,
        ),
        rule(
            "en.alternative.aside",
            Alternative,
            vec![W("("), C(Alternative), W(")")],
            first,
        ),
        // The thing again: "one", or "it" or its name.
        filler("en.again.one", ROUND, Again, "one"),
        rule("en.again.name", Again, vec![C(Filling)], last),
        // "which structs have a field ...": also "what structs", "who has".
        rule(
            "en.question.what_kind",
            Question,
            words_then("what", &[Noun, VerbPhrase]),
            identify,
        ),
        rule(
            "en.question.who",
            Question,
            words_then("who", &[VerbPhrase]),
            identify,
        ),
        rule(
            "en.description.preposition",
            Description,
            vec![C(Noun), C(Preposition), C(NounPhrase)],
            placed,
        ),
        // "what fields are in the slot?": asking for the things so placed.
        read_only_rule(
            "en.question.what_are_preposition",
            7,
            Question,
            vec![W("what"), C(Noun), W("are"), C(Preposition), C(NounPhrase)],
            |c| identify(&[placed(c)?]),
        ),
        // "is there anything that ends up calling `x`", "anything calling
        // `x`": something doing it, of any kind.
        read_only_rule(
            "en.description.anything_that",
            9,
            Description,
            vec![C(Anything), W("that"), C(VerbPhrase)],
            anything,
        ),
        read_only_rule(
            "en.description.anything_doing",
            9,
            Description,
            vec![C(Anything), C(Participial)],
            anything,
        ),
        // "anything call `x`, which fns", "anythign call `x`": the question
        // whether something does, said without its "does".
        read_only_rule(
            "en.description.anything_bare",
            22,
            Description,
            vec![C(Anything), C(VerbPhrase)],
            anything,
        ),
        // "does anything call `x`": asking whether something does asks
        // for what does.
        read_only_rule(
            "en.question.does_anything",
            9,
            Question,
            words_then("does", &[Anything, VerbPhrase]),
            |c| identify(&[anything(c)?]),
        ),
        // "are there structs that ...? which ones?", "..., can you list
        // them?": asking again for the things just asked about.
        read_only_rule(
            "en.utterance.ask_again",
            5,
            Utterance,
            vec![C(Utterance), C(AskAgain)],
            again,
        ),
        // ", which ones?", "like which functions"
        read_only_rule(
            "en.ask_again.joined",
            9,
            AskAgain,
            vec![C(Join), C(AskAgain)],
            last,
        ),
        // "which functions?", "which functions do": asking again for the
        // things just asked about, of the kind they are.
        read_only_rule(
            "en.ask_again.which_kind",
            9,
            AskAgain,
            words_then("which", &[Noun]),
            first,
        ),
        read_only_rule(
            "en.ask_again.which_kind_do",
            9,
            AskAgain,
            vec![W("which"), C(Noun), W("do")],
            first,
        ),
        // "which structs actually have ...": a hedge changes nothing asked.
        read_only_rule(
            "en.verb_phrase.hedged",
            5,
            VerbPhrase,
            vec![C(Hedge), C(VerbPhrase)],
            last,
        ),
        // "which structs in the codebase have ...": every thing asked about
        // is in the code, so saying so restricts nothing.
        read_only_rule(
            "en.noun.in_the_codebase",
            5,
            Noun,
            vec![C(Noun), W("in"), C(Codebase)],
            first,
        ),
        // "which struct(s) have ...": a noun that may be one or many.
        read_only_rule(
            "en.noun.maybe_plural",
            5,
            Noun,
            vec![C(Noun), W("( s )")],
            |c| match c {
                [Sem::Kind { kind, .. }] => Some(Sem::Kind {
                    kind: kind.clone(),
                    number: Number::Plural,
                }),
                _ => None,
            },
        ),
        // "find the structs that ..."
        filler("en.prompt.find", ROUND, Prompt, "find"),
        // How the system says a role's filler is or includes a thing.
        word(
            RuleInfo::new("en.includes.is_or_contains", ROUND),
            Includes,
            "is or contains",
            blank(),
        ),
    ];
    for (table, cat, round) in [
        (INCLUDES, Includes, ROUND),
        (WITHIN, Within, ROUND),
        (ASK_AGAIN, AskAgain, 5),
        (ANYTHING, Anything, 9),
        (ASK_AGAIN_LATER, AskAgain, LATER),
        (INCLUDES_LATER, Includes, LATER),
        (ASK_AGAIN_18, AskAgain, 18),
        (INCLUDES_19, Includes, 19),
    ] {
        rules.extend(
            table
                .iter()
                .map(|(id, words)| filler(id, round, cat, words)),
        );
    }
    let grown = |id, lhs, rhs, build| read_only_rule(id, GROWN, lhs, rhs, build);
    rules.extend([
        // "typed as `HashSet`"
        grown(
            "en.qualifier.adjective_as",
            Qualifier,
            vec![C(RoleAdjective), W("as"), C(Filling)],
            qualified,
        ),
        // "where the type is `Does`", "(or) where the type contains it"
        grown(
            "en.qualifier.where",
            Qualifier,
            words_then("where", &[RoleNoun, Includes, Filling]),
            qualified,
        ),
        // "with `FileKind` in its type", "with PrChangedFile in it", "with
        // `SavedPost` type (in it)"
        grown(
            "en.qualifier.with_within",
            Qualifier,
            words_then("with", &[NounPhrase, Within]),
            holding,
        ),
        grown(
            "en.qualifier.with_then_role",
            Qualifier,
            words_then("with", &[NounPhrase, RoleNoun]),
            holding,
        ),
        // "(a field) that is `Saved` type", "that is type `Load`"
        grown(
            "en.qualifier.that_then_role",
            Qualifier,
            words_then("that", &[Includes, NounPhrase, RoleNoun]),
            holding,
        ),
        grown(
            "en.qualifier.that_role",
            Qualifier,
            words_then("that", &[Includes, RoleNoun, Filling]),
            |c| qualified(c.get(1..)?),
        ),
        // "(or contains it, like Option<...>)"
        grown(
            "en.alternative.widened",
            Alternative,
            vec![C(Alternative), W(","), C(Widening)],
            first,
        ),
        // "or with a type that includes `IRect`", "or that contains Key
        // somehow", "or where the type contains it"
        grown(
            "en.alternative.qualifier",
            Alternative,
            words_then("or", &[Qualifier]),
            spoken_of,
        ),
        // "a field type", "in a field type"
        grown(
            "en.holder.role",
            Holder,
            vec![C(Holder), C(RoleNoun)],
            |c| holder_role(c).then(|| c.first().cloned()).flatten(),
        ),
        // "which structs have `Poster` as field type? or field type that
        // contains Poster somewhere in it"
        grown(
            "en.utterance.put_again",
            Utterance,
            vec![C(Utterance), C(Alternative)],
            put_again,
        ),
        // "who has a `Target` field? structs only", "who has a url field,
        // structs only": asked again of things of that kind.
        grown(
            "en.ask_again.only",
            AskAgain,
            vec![C(Noun), W("only")],
            first,
        ),
    ]);
    rules.extend(LODGINGS.iter().map(|(id, words)| {
        grown(
            id,
            NounPhrase,
            vec![C(NounPhrase), W(words), C(Holder)],
            held,
        )
    }));
    let later = |id, lhs, rhs, build| read_only_rule(id, LATER, lhs, rhs, build);
    rules.extend([
        // "fields of `RefCompare`", "field of ImageSearch": a holder of the
        // thing, said after it.
        later(
            "en.noun_phrase.holder_of",
            NounPhrase,
            vec![C(Holder), W("of"), C(NounPhrase)],
            held,
        ),
        // "a struct field": a holder said with the kind it is in, which is
        // the same holder.
        later("en.holder.kind", Holder, vec![C(Noun), C(Holder)], last),
        // "(fields) use `Tab` type"
        later(
            "en.qualifier.includes_then_role",
            Qualifier,
            vec![C(Includes), C(NounPhrase), C(RoleNoun)],
            holding,
        ),
        // "or a field that contains it", "or field type that contains
        // Poster somewhere in it": another thing holding it.
        later(
            "en.alternative.or_noun_phrase",
            Alternative,
            words_then("or", &[NounPhrase]),
            last,
        ),
        // "or one whose type contains it"
        later(
            "en.alternative.or_one",
            Alternative,
            words_then("or one", &[Qualifier]),
            spoken_of,
        ),
        // "or with a type that contains one", "or a type containing it"
        later(
            "en.alternative.or_with_role_that",
            Alternative,
            vec![W("or with"), C(RoleNoun), W("that"), C(Includes), C(Again)],
            last,
        ),
        later(
            "en.alternative.or_role_includes",
            Alternative,
            vec![W("or"), C(RoleNoun), C(Includes), C(Again)],
            last,
        ),
        // "use TimeZone anywhere in the type"
        later(
            "en.within.anywhere_in_role",
            Within,
            words_then("anywhere in", &[RoleNoun]),
            last,
        ),
        // "where is `ImageSearch` used as field type", "RangeSource used as
        // field type where": what
        // has a part whose role `x` fills.
        later(
            "en.question.where_used_as",
            Question,
            vec![
                W("where is"),
                C(NounPhrase),
                W("used as"),
                C(Part),
                C(RoleNoun),
            ],
            used_as,
        ),
        later(
            "en.question.used_as_where",
            Question,
            vec![
                C(NounPhrase),
                W("used as"),
                C(Part),
                C(RoleNoun),
                W("where"),
            ],
            used_as,
        ),
    ]);
    let now = |id, lhs, rhs, build| read_only_rule(id, NOW, lhs, rhs, build);
    rules.extend([
        // "`tree_scroll` field": what has a part going by the name, which
        // "- what structs" asks again of things of a kind.
        now(
            "en.description.part_holders",
            Description,
            vec![C(NounPhrase), C(Part)],
            part_holders,
        ),
        // "— which structs", "- what structs": asked again, set off.
        now(
            "en.ask_again.set_off",
            AskAgain,
            vec![C(Break), C(AskAgain)],
            last,
        ),
        // "what structs", asked again after a question or a thing set off.
        now(
            "en.ask_again.what_kind",
            AskAgain,
            words_then("what", &[Noun]),
            first,
        ),
    ]);
    // "is there any struct that has a field `eye_off`? what structs is it
    // in": asking again which holders the thing asked about is in.
    rules.push(read_only_rule(
        "en.ask_again.is_it_in",
        17,
        AskAgain,
        vec![C(AskAgain), W("is it in")],
        first,
    ));
    rules.extend(
        [
            ("en.ask_again.which_are_they", "which are they"),
            (
                "en.ask_again.is_there_a_list_somewhere",
                "is there a list somewhere",
            ),
        ]
        .map(|(id, words)| filler(id, NOW, AskAgain, words)),
    );
    // "`bore_height` struct fields - list pls": asked again, to be listed.
    rules.push(filler("en.ask_again.list", ROUND_14, AskAgain, "list"));
    // "structs that have a `seen` field, which", "any structs w/ field
    // named real? which": "which" alone asks again.
    rules.push(filler("en.ask_again.which", 18, AskAgain, "which"));
    let round_14 = |id, lhs, rhs, build| read_only_rule(id, ROUND_14, lhs, rhs, build);
    rules.extend([
        // "whats inside `overboard`, fields", "what does `x` hold,
        // field-wise": a kind alone, set off after what was asked, asks it
        // again of that kind.
        round_14("en.ask_again.kind", AskAgain, vec![C(Join), C(Noun)], last),
        // "the struct fields of `person_items`": a role said with the kind
        // it is in, which is the same role.
        round_14(
            "en.role_noun.kind",
            RoleNoun,
            vec![C(Noun), C(RoleNoun)],
            last,
        ),
    ]);
    // "a struct field": a part said with the kind it is in, the same part.
    rules.push(read_only_rule(
        "en.part.kind",
        15,
        Part,
        vec![C(Noun), C(Part)],
        last,
    ));
    // "where's `handgun_draw_m` declared as a struct field", "where's `x`
    // used as a struct field": what has a part going by the name.
    for (id, verb) in [
        ("en.question.where_s_declared_as", C(Defined)),
        ("en.question.where_s_used_as", W("used")),
    ] {
        let rhs = vec![W("where's"), C(NounPhrase), verb, W("as"), C(Part)];
        rules.push(read_only_rule(id, 15, Question, rhs, |c| {
            identify(&[part_holders(&[c.first()?.clone(), c.last()?.clone()])?])
        }));
    }
    // "which struct has `engine_inertia` field in it": "in it" says no
    // more than the verb did.
    rules.push(read_only_rule(
        "en.verb_phrase.within",
        17,
        VerbPhrase,
        vec![C(VerbPhrase), C(Within)],
        first,
    ));
    rules.extend(
        [("en.hedge.only", "only"), ("en.hedge.like", "like")]
            .map(|(id, words)| filler(id, LATER, Hedge, words)),
    );
    let round_19 = |id, lhs, rhs, build| read_only_rule(id, 19, lhs, rhs, build);
    rules.extend([
        // What fills a role is a thing: "whose type is or contains `x`".
        spoken(
            "en.filling.noun_phrase",
            19,
            Filling,
            vec![C(NounPhrase)],
            first,
            |v| Some(vec![v.clone()]),
        ),
        // "a field typed Handle", "a field thats Kind or wraps it": a type
        // named by a word the system knows as English, capitalized.
        round_19(
            "en.filling.capitalized",
            Filling,
            vec![Class(CAPITALIZED)],
            first,
        ),
        // "(structs that have a) field type `Reveal`": a role and what
        // fills it, after the holder, with nothing between.
        round_19(
            "en.qualifier.role",
            Qualifier,
            vec![C(RoleNoun), C(Filling)],
            qualified,
        ),
        // "(`MergeMethod` as field type,) or inside one", "or insdie it":
        // the thing again, wrapped in another.
        round_19(
            "en.alternative.or_wrapping",
            Alternative,
            vec![W("or"), C(Wrapping), C(Again)],
            last,
        ),
        // "field type `Milestone` or contains it, what structs have tht"
        round_19(
            "en.question.set_off_described",
            Question,
            vec![C(NounPhrase), C(Break), C(Question)],
            set_off_fitting,
        ),
        // "structs with a `WireRelease` field in them": "in them" says no
        // more than the preposition did.
        round_19(
            "en.description.within",
            Description,
            vec![C(Description), C(Within)],
            first,
        ),
    ]);
    rules.push(filler("en.within.in_them", 19, Within, "in them"));
    // "any structs w/ field typed `x`, list them pls"
    rules.push(filler("en.ask_again.list_them", 19, AskAgain, "list them"));
    // "who uses `wrap_lines`, callers only": the role of the things asked
    // for, said again.
    rules.extend([
        read_only_rule("en.ask_again.role", 22, AskAgain, vec![C(RoleNoun)], first),
        read_only_rule(
            "en.ask_again.hedged",
            22,
            AskAgain,
            vec![C(AskAgain), C(Hedge)],
            first,
        ),
    ]);
    rules.extend(
        WITHIN_ROLE
            .iter()
            .map(|(id, words)| rule(id, Within, words_then(words, &[RoleNoun]), last)),
    );
    rules
}
