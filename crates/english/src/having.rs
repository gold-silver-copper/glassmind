//! Describing things by what they have: "which structs have a field whose
//! type is or contains `Arc`?", "structs w/ a field of type `Tab`", "fields
//! typed `Art`".
//!
//! A noun with a qualifier ("a field whose type is `Arc`") is any thing
//! fitting a description, so it can stand as the object of a verb. A
//! qualifier names one side of a relation with a role noun ("type") and
//! says what fills it.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Condition, Ontology, Position, Thing};

use crate::aside::filler;
use crate::asking::{identify, meeting, read_only_rule, related, same, words_then};
use crate::sem::{Cat, Number, Sem};
use crate::{blank, last, word};

use Cat::{
    Again, Alternative, AskAgain, Description, Hedge, Holder, Includes, Noun, NounPhrase,
    Preposition, Prompt, Qualifier, Question, RoleAdjective, RoleNoun, Utterance, VerbPhrase,
    Within,
};
use Sym::{Cat as C, Word as W};

const ROUND: u32 = 3;

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
        ] if !matches!(again, Sem::Thing(t) if t != other) => Some(q.clone()),
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

/// "a field that is `x`", "a `x` field": any thing of a holder's kind
/// holding `x`, which comes after or before the holder.
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
        ] if matches!(other, Thing::Named(_) | Thing::Item(_)) => Some(Sem::Thing(Thing::Fitting(
            meeting(Some(kind), related(relation, flip(*position), other)),
        ))),
        _ => None,
    }
}

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
    ("en.within.somewhere_in_role", "somewhere in"),
];

/// Questions after a question that ask again for what it asks about.
const ASK_AGAIN: &[(&str, &str)] = &[
    ("en.ask_again.which_ones", "which ones"),
    ("en.ask_again.are_there_any", "are there any"),
    ("en.ask_again.is_there_any", "is there any"),
    ("en.ask_again.can_you_list_them", "can you list them"),
];

/// Determiners before a role noun, which name the same role: "a type",
/// "the type", "its type".
const ROLE_DETERMINERS: &[(&str, &str)] = &[
    ("en.role_noun.a", "a"),
    ("en.role_noun.the", "the"),
    ("en.role_noun.its", "its"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| read_only_rule(id, ROUND, lhs, rhs, build);
    let mut rules: Vec<Rule<Cat, Sem<O>>> = vec![
        // "a field whose type is or contains `x`": how the system says it.
        Rule::new(
            RuleInfo::new("en.noun_phrase.qualified", ROUND),
            NounPhrase,
            words_then("a", &[Noun, Qualifier]),
            S::Both(fitting, |v| match v {
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
            }),
        ),
        // "fields typed `Art`", "have field whose type is `Gone`".
        rule(
            "en.noun_phrase.bare_qualified",
            NounPhrase,
            vec![C(Noun), C(Qualifier)],
            fitting,
        ),
        // "contains a `Key`", "an `Arc`".
        rule(
            "en.noun_phrase.a",
            NounPhrase,
            words_then("a", &[NounPhrase]),
            last,
        ),
        rule(
            "en.noun_phrase.an",
            NounPhrase,
            words_then("an", &[NounPhrase]),
            last,
        ),
        // "whose type is or contains `x`"
        Rule::new(
            RuleInfo::new("en.qualifier.whose", ROUND),
            Qualifier,
            words_then("whose", &[RoleNoun, Includes, NounPhrase]),
            S::Both(qualified, unqualified),
        ),
        // "of type `x`", "with type `x`", "with a type of `x`", "typed `x`".
        rule(
            "en.qualifier.of",
            Qualifier,
            words_then("of", &[RoleNoun, NounPhrase]),
            qualified,
        ),
        rule(
            "en.qualifier.with",
            Qualifier,
            words_then("with", &[RoleNoun, NounPhrase]),
            qualified,
        ),
        rule(
            "en.qualifier.with_role_of",
            Qualifier,
            vec![W("with"), C(RoleNoun), W("of"), C(NounPhrase)],
            qualified,
        ),
        // "with type that is or contains `x`"
        rule(
            "en.qualifier.with_role_that",
            Qualifier,
            vec![
                W("with"),
                C(RoleNoun),
                W("that"),
                C(Includes),
                C(NounPhrase),
            ],
            qualified,
        ),
        rule(
            "en.qualifier.adjective",
            Qualifier,
            vec![C(RoleAdjective), C(NounPhrase)],
            qualified,
        ),
        // A qualifier naming no role, after a holder: "(a field) that is
        // `Key`", "containing `Instant`", "that's a `u16`".
        rule(
            "en.qualifier.that",
            Qualifier,
            words_then("that", &[Includes, NounPhrase]),
            last,
        ),
        rule(
            "en.qualifier.that_s",
            Qualifier,
            words_then("that's", &[NounPhrase]),
            last,
        ),
        rule(
            "en.qualifier.includes",
            Qualifier,
            vec![C(Includes), C(NounPhrase)],
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
            same,
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
        // ", or contains it", "(or containing it)", "or contains it somehow"
        rule(
            "en.alternative.comma",
            Alternative,
            words_then(",", &[Alternative]),
            last,
        ),
        rule(
            "en.alternative.aside",
            Alternative,
            vec![W("("), C(Alternative), W(")")],
            same,
        ),
        // The thing again: "it", "one", or its name.
        filler("en.again.it", ROUND, Again, "it"),
        filler("en.again.one", ROUND, Again, "one"),
        rule("en.again.name", Again, vec![C(NounPhrase)], last),
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
        // "any structs with fields typed `Commit`?", "are there structs
        // that ...": asking whether there are some asks for them.
        rule(
            "en.description.any",
            Description,
            words_then("any", &[Description]),
            last,
        ),
        rule(
            "en.description.is_there",
            Description,
            words_then("is there", &[Description]),
            last,
        ),
        rule(
            "en.description.are_there",
            Description,
            words_then("are there", &[Description]),
            last,
        ),
        // "are there structs that ...? which ones?", "..., can you list
        // them?": asking again for the things just asked about.
        read_only_rule(
            "en.utterance.ask_again",
            5,
            Utterance,
            vec![C(Utterance), C(AskAgain)],
            same,
        ),
        read_only_rule(
            "en.ask_again.comma",
            5,
            AskAgain,
            words_then(",", &[AskAgain]),
            last,
        ),
        read_only_rule(
            "en.ask_again.question_mark",
            5,
            AskAgain,
            vec![C(AskAgain), W("?")],
            same,
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
            vec![C(Noun), W("in"), W("the"), W("codebase")],
            same,
        ),
        // "which struct(s) have ...": a noun that may be one or many.
        read_only_rule(
            "en.noun.maybe_plural",
            5,
            Noun,
            vec![C(Noun), W("("), W("s"), W(")")],
            |c| match c {
                [Sem::Kind { kind, .. }] => Some(Sem::Kind {
                    kind: kind.clone(),
                    number: Number::Plural,
                }),
                _ => None,
            },
        ),
        // "which structs have a `jobs` field in them": "them" is the
        // subject again, so it says nothing more.
        read_only_rule(
            "en.verb_phrase.in_them",
            5,
            VerbPhrase,
            vec![C(VerbPhrase), W("in"), W("them")],
            same,
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
    ] {
        rules.extend(
            table
                .iter()
                .map(|(id, words)| filler(id, round, cat, words)),
        );
    }
    for (table, lhs) in [(ROLE_DETERMINERS, RoleNoun), (WITHIN_ROLE, Within)] {
        rules.extend(
            table
                .iter()
                .map(|(id, words)| rule(id, lhs, words_then(words, &[RoleNoun]), last)),
        );
    }
    rules
}
