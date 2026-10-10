//! Describing things by what they have: "which structs have a field whose
//! type is or contains `Arc`?", "structs w/ a field of type `Tab`", "fields
//! typed `Art`".
//!
//! A noun with a qualifier ("a field whose type is `Arc`") is any thing
//! fitting a description, so it can stand as the object of a verb. A
//! qualifier names one side of a relation with a role noun ("type") and
//! says what fills it.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Condition, Description, Ontology, Position, Question, Thing};

use crate::sem::{Cat, Number, Sem};
use crate::word;

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 3;

/// The other side of a two-place relation.
fn flip(position: Position) -> Position {
    match position {
        Position::First => Position::Second,
        Position::Second => Position::First,
    }
}

/// "whose type is `x`": the described thing's role is filled by `x`, so it
/// fills the other side of the role's relation.
fn qualified<O: Ontology>(role: &Sem<O>, other: &Sem<O>) -> Option<Sem<O>> {
    match (role, other) {
        (Sem::Role { relation, position }, Sem::Thing(other)) => Some(Sem::Condition {
            condition: Condition::Related {
                relation: relation.clone(),
                position: flip(*position),
                other: other.clone(),
            },
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
            Sem::Quote(String::new()),
            Sem::Thing(other.clone()),
        ]),
        _ => None,
    }
}

/// The qualifier among the children, if what else they say agrees with it:
/// an alternative must speak of the same thing, by name or as "it".
fn agreeing<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [q, again] => {
            let other = match q {
                Sem::Condition {
                    condition: Condition::Related { other, .. },
                    ..
                }
                | Sem::Thing(other) => other,
                _ => return None,
            };
            match again {
                Sem::Thing(t) if t != other => None,
                _ => Some(q.clone()),
            }
        }
        _ => None,
    }
}

/// Any thing of a kind meeting a condition: "a field whose type is `x`".
fn fitting<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Kind { kind, .. }, Sem::Condition { condition, .. }] => {
            Some(Sem::Thing(Thing::Fitting(Description {
                kind: Some(kind.clone()),
                conditions: vec![condition.clone()],
            })))
        }
        _ => None,
    }
}

/// "a field that is `x`", "a `x` field": any thing of a holder's kind
/// holding `x`.
fn held<O: Ontology>(holder: &Sem<O>, other: &Sem<O>) -> Option<Sem<O>> {
    match (holder, other) {
        (
            Sem::Holder {
                kind,
                relation,
                position,
            },
            Sem::Thing(other @ (Thing::Named(_) | Thing::Item(_))),
        ) => Some(Sem::Thing(Thing::Fitting(Description {
            kind: Some(kind.clone()),
            conditions: vec![Condition::Related {
                relation: relation.clone(),
                position: flip(*position),
                other: other.clone(),
            }],
        }))),
        _ => None,
    }
}

/// The value of a rule's last child.
fn last<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.last().cloned()
}

/// Ways of saying a role's filler is or includes a thing. A field's type
/// "contains" `Arc` when `Arc` is written anywhere in it, so "is or
/// contains" and "contains" say the same. The first is how the system says it.
const INCLUDES: &[(&str, &[&str])] = &[
    ("en.includes.is_or_contains", &["is", "or", "contains"]),
    ("en.includes.is", &["is"]),
    ("en.includes.contains", &["contains"]),
    ("en.includes.includes", &["includes"]),
    ("en.includes.wraps", &["wraps"]),
    ("en.includes.containing", &["containing"]),
    ("en.includes.wrapping", &["wrapping"]),
    ("en.includes.holding", &["holding"]),
    ("en.includes.is_or_includes", &["is", "or", "includes"]),
    ("en.includes.is_or_wraps", &["is", "or", "wraps"]),
    (
        "en.includes.is_or_contains_aside",
        &["is", "(", "or", "contains", ")"],
    ),
    (
        "en.includes.is_comma_or_contains",
        &["is", ",", "or", "contains", ","],
    ),
];

/// Words saying where in a thing another is, after a qualifier: "or wraps
/// it somehow", "or contains u32 in it", "with type IRect in it".
const WITHIN: &[(&str, &[&str])] = &[
    ("en.within.somehow", &["somehow"]),
    ("en.within.somewhere", &["somewhere"]),
    ("en.within.in_it", &["in", "it"]),
    ("en.within.inside_it", &["inside", "it"]),
    ("en.within.inside", &["inside"]),
];

/// Words before a role noun saying a thing is inside what fills the role:
/// "in its type", "somewhere in the type".
const WITHIN_ROLE: &[(&str, &[&str])] = &[
    ("en.within.in_role", &["in"]),
    ("en.within.inside_role", &["inside"]),
    ("en.within.somewhere_in_role", &["somewhere", "in"]),
];

/// Questions after a question that ask again for what it asks about.
const ASK_AGAIN: &[(&str, &[&str])] = &[
    ("en.ask_again.which_ones", &["which", "ones"]),
    (
        "en.ask_again.which_ones_are_there",
        &["which", "ones", "are", "there"],
    ),
    (
        "en.ask_again.which_ones_have_it",
        &["which", "ones", "have", "it"],
    ),
    ("en.ask_again.are_there_any", &["are", "there", "any"]),
    ("en.ask_again.is_there_any", &["is", "there", "any"]),
    (
        "en.ask_again.can_you_list_them",
        &["can", "you", "list", "them"],
    ),
];

/// Determiners before a role noun, which name the same role: "a type",
/// "the type", "its type".
const ROLE_DETERMINERS: &[(&str, &str)] = &[
    ("en.role_noun.a", "a"),
    ("en.role_noun.the", "the"),
    ("en.role_noun.its", "its"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let blank = || Sem::Quote(String::new());
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let mut rules: Vec<Rule<Cat, Sem<O>>> = vec![
        // "a field whose type is or contains `x`": how the system says it.
        Rule::new(
            RuleInfo::new("en.noun_phrase.qualified", ROUND),
            Cat::NounPhrase,
            vec![W("a"), C(Cat::Noun), C(Cat::Qualifier)],
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
            Cat::NounPhrase,
            vec![C(Cat::Noun), C(Cat::Qualifier)],
            fitting,
        ),
        // "contains a `Key`", "an `Arc`".
        rule(
            "en.noun_phrase.a",
            Cat::NounPhrase,
            vec![W("a"), C(Cat::NounPhrase)],
            last,
        ),
        rule(
            "en.noun_phrase.an",
            Cat::NounPhrase,
            vec![W("an"), C(Cat::NounPhrase)],
            last,
        ),
        // "whose type is or contains `x`"
        Rule::new(
            RuleInfo::new("en.qualifier.whose", ROUND),
            Cat::Qualifier,
            vec![
                W("whose"),
                C(Cat::RoleNoun),
                C(Cat::Includes),
                C(Cat::NounPhrase),
            ],
            S::Both(
                |c| match c {
                    [role, _, other] => qualified(role, other),
                    _ => None,
                },
                unqualified,
            ),
        ),
        // "of type `x`", "with type `x`", "with a type of `x`", "typed `x`".
        rule(
            "en.qualifier.of",
            Cat::Qualifier,
            vec![W("of"), C(Cat::RoleNoun), C(Cat::NounPhrase)],
            |c| match c {
                [role, other] => qualified(role, other),
                _ => None,
            },
        ),
        rule(
            "en.qualifier.with",
            Cat::Qualifier,
            vec![W("with"), C(Cat::RoleNoun), C(Cat::NounPhrase)],
            |c| match c {
                [role, other] => qualified(role, other),
                _ => None,
            },
        ),
        rule(
            "en.qualifier.with_role_of",
            Cat::Qualifier,
            vec![W("with"), C(Cat::RoleNoun), W("of"), C(Cat::NounPhrase)],
            |c| match c {
                [role, other] => qualified(role, other),
                _ => None,
            },
        ),
        // "with type that is or contains `x`"
        rule(
            "en.qualifier.with_role_that",
            Cat::Qualifier,
            vec![
                W("with"),
                C(Cat::RoleNoun),
                W("that"),
                C(Cat::Includes),
                C(Cat::NounPhrase),
            ],
            |c| match c {
                [role, _, other] => qualified(role, other),
                _ => None,
            },
        ),
        rule(
            "en.qualifier.adjective",
            Cat::Qualifier,
            vec![C(Cat::RoleAdjective), C(Cat::NounPhrase)],
            |c| match c {
                [role, other] => qualified(role, other),
                _ => None,
            },
        ),
        // A qualifier naming no role, after a holder: "(a field) that is
        // `Key`", "containing `Instant`", "that's a `u16`".
        rule(
            "en.qualifier.that",
            Cat::Qualifier,
            vec![W("that"), C(Cat::Includes), C(Cat::NounPhrase)],
            last,
        ),
        rule(
            "en.qualifier.that_s",
            Cat::Qualifier,
            vec![W("that's"), C(Cat::NounPhrase)],
            last,
        ),
        rule(
            "en.qualifier.includes",
            Cat::Qualifier,
            vec![C(Cat::Includes), C(Cat::NounPhrase)],
            last,
        ),
        // "a field that is `Key`", "fields containing `Signature`"
        rule(
            "en.noun_phrase.holder",
            Cat::NounPhrase,
            vec![C(Cat::Holder), C(Cat::Qualifier)],
            |c| match c {
                [holder, other] => held(holder, other),
                _ => None,
            },
        ),
        // "a `DraftSide` field", "a BigInt field"
        rule(
            "en.noun_phrase.compound",
            Cat::NounPhrase,
            vec![C(Cat::NounPhrase), C(Cat::Holder)],
            |c| match c {
                [other, holder] => held(holder, other),
                _ => None,
            },
        ),
        // "with type IRect in it"
        rule(
            "en.qualifier.within",
            Cat::Qualifier,
            vec![C(Cat::Qualifier), C(Cat::Within)],
            |c| c.first().cloned(),
        ),
        // "whose type is `Tab` or contains it"
        rule(
            "en.qualifier.alternative",
            Cat::Qualifier,
            vec![C(Cat::Qualifier), C(Cat::Alternative)],
            agreeing,
        ),
        // "or contains it", "or wraps a `Notify`", "or has Arc somewhere in
        // the type", "or something wrapping it", "or a type that contains it".
        rule(
            "en.alternative.or",
            Cat::Alternative,
            vec![W("or"), C(Cat::Includes), C(Cat::Again)],
            last,
        ),
        rule(
            "en.alternative.or_has",
            Cat::Alternative,
            vec![W("or"), W("has"), C(Cat::Again)],
            last,
        ),
        rule(
            "en.alternative.or_something",
            Cat::Alternative,
            vec![W("or"), W("something"), C(Cat::Includes), C(Cat::Again)],
            last,
        ),
        rule(
            "en.alternative.or_something_that",
            Cat::Alternative,
            vec![
                W("or"),
                W("something"),
                W("that"),
                C(Cat::Includes),
                C(Cat::Again),
            ],
            last,
        ),
        rule(
            "en.alternative.or_role_that",
            Cat::Alternative,
            vec![
                W("or"),
                C(Cat::RoleNoun),
                W("that"),
                C(Cat::Includes),
                C(Cat::Again),
            ],
            last,
        ),
        // ", or contains it", "(or containing it)", "or contains it somehow"
        rule(
            "en.alternative.comma",
            Cat::Alternative,
            vec![W(","), C(Cat::Alternative)],
            last,
        ),
        rule(
            "en.alternative.aside",
            Cat::Alternative,
            vec![W("("), C(Cat::Alternative), W(")")],
            |c| c.first().cloned(),
        ),
        // The thing again: "it", "one", or its name.
        word(
            RuleInfo::new("en.again.it", ROUND),
            Cat::Again,
            &["it"],
            blank(),
        )
        .read_only(),
        word(
            RuleInfo::new("en.again.one", ROUND),
            Cat::Again,
            &["one"],
            blank(),
        )
        .read_only(),
        rule("en.again.name", Cat::Again, vec![C(Cat::NounPhrase)], last),
        // "which structs have a field ...": also "what structs", "who has".
        rule(
            "en.question.what_kind",
            Cat::Question,
            vec![W("what"), C(Cat::Noun), C(Cat::VerbPhrase)],
            |c| match c {
                [Sem::Kind { kind, .. }, Sem::Condition { condition, .. }] => {
                    Some(Sem::Question(Question::Identify(Description {
                        kind: Some(kind.clone()),
                        conditions: vec![condition.clone()],
                    })))
                }
                _ => None,
            },
        ),
        rule(
            "en.question.who",
            Cat::Question,
            vec![W("who"), C(Cat::VerbPhrase)],
            |c| match c {
                [Sem::Condition { condition, .. }] => {
                    Some(Sem::Question(Question::Identify(Description {
                        kind: None,
                        conditions: vec![condition.clone()],
                    })))
                }
                _ => None,
            },
        ),
        // "structs that have a field ...", "structs w/ a field ...".
        rule(
            "en.description.that",
            Cat::Description,
            vec![C(Cat::Noun), W("that"), C(Cat::VerbPhrase)],
            |c| match c {
                [Sem::Kind { kind, .. }, Sem::Condition { condition, .. }] => {
                    Some(Sem::Description(Description {
                        kind: Some(kind.clone()),
                        conditions: vec![condition.clone()],
                    }))
                }
                _ => None,
            },
        ),
        rule(
            "en.description.preposition",
            Cat::Description,
            vec![C(Cat::Noun), C(Cat::Preposition), C(Cat::NounPhrase)],
            |c| match c {
                [
                    Sem::Kind { kind, .. },
                    Sem::Relation { relation, .. },
                    Sem::Thing(other),
                ] => Some(Sem::Description(Description {
                    kind: Some(kind.clone()),
                    conditions: vec![Condition::Related {
                        relation: relation.clone(),
                        position: Position::First,
                        other: other.clone(),
                    }],
                })),
                _ => None,
            },
        ),
        // "any structs with fields typed `Commit`?", "are there structs
        // that ...": asking whether there are some asks for them.
        rule(
            "en.description.any",
            Cat::Description,
            vec![W("any"), C(Cat::Description)],
            last,
        ),
        rule(
            "en.description.is_there",
            Cat::Description,
            vec![W("is"), W("there"), C(Cat::Description)],
            last,
        ),
        rule(
            "en.description.are_there",
            Cat::Description,
            vec![W("are"), W("there"), C(Cat::Description)],
            last,
        ),
        // "are there structs that ...? which ones?", "..., can you list
        // them?": asking again for the things just asked about.
        Rule::new(
            RuleInfo::new("en.utterance.ask_again", 5),
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::AskAgain)],
            S::Build(|c| c.first().cloned()),
        )
        .read_only(),
        Rule::new(
            RuleInfo::new("en.ask_again.comma", 5),
            Cat::AskAgain,
            vec![W(","), C(Cat::AskAgain)],
            S::Build(last),
        )
        .read_only(),
        Rule::new(
            RuleInfo::new("en.ask_again.question_mark", 5),
            Cat::AskAgain,
            vec![C(Cat::AskAgain), W("?")],
            S::Build(|c| c.first().cloned()),
        )
        .read_only(),
        // "which structs actually have ...": a hedge changes nothing asked.
        Rule::new(
            RuleInfo::new("en.verb_phrase.hedged", 5),
            Cat::VerbPhrase,
            vec![C(Cat::Hedge), C(Cat::VerbPhrase)],
            S::Build(last),
        )
        .read_only(),
        // "which structs in the codebase have ...": every thing asked about
        // is in the code, so saying so restricts nothing.
        Rule::new(
            RuleInfo::new("en.noun.in_the_codebase", 5),
            Cat::Noun,
            vec![C(Cat::Noun), W("in"), W("the"), W("codebase")],
            S::Build(|c| c.first().cloned()),
        )
        .read_only(),
        // "which struct(s) have ...": a noun that may be one or many.
        Rule::new(
            RuleInfo::new("en.noun.maybe_plural", 5),
            Cat::Noun,
            vec![C(Cat::Noun), W("("), W("s"), W(")")],
            S::Build(|c: &[Sem<O>]| match c {
                [Sem::Kind { kind, .. }] => Some(Sem::Kind {
                    kind: kind.clone(),
                    number: Number::Plural,
                }),
                _ => None,
            }),
        )
        .read_only(),
        // "which structs have a `jobs` field in them": "them" is the
        // subject again, so it says nothing more.
        Rule::new(
            RuleInfo::new("en.verb_phrase.in_them", 5),
            Cat::VerbPhrase,
            vec![C(Cat::VerbPhrase), W("in"), W("them")],
            S::Build(|c| c.first().cloned()),
        )
        .read_only(),
        // "find the structs that ..."
        word(
            RuleInfo::new("en.prompt.find", ROUND),
            Cat::Prompt,
            &["find"],
            blank(),
        )
        .read_only(),
    ];
    let words = |table: &[(&'static str, &'static [&'static str])], cat, speak: bool| {
        table
            .iter()
            .enumerate()
            .map(|(i, (id, words))| {
                let r = word(
                    RuleInfo::new(id, ROUND),
                    cat,
                    words,
                    Sem::Quote(String::new()),
                );
                if speak && i == 0 { r } else { r.read_only() }
            })
            .collect::<Vec<_>>()
    };
    rules.extend(words(INCLUDES, Cat::Includes, true));
    rules.extend(words(WITHIN, Cat::Within, false));
    rules.extend(
        ASK_AGAIN
            .iter()
            .map(|(id, w)| word(RuleInfo::new(id, 5), Cat::AskAgain, w, blank()).read_only()),
    );
    rules.extend(
        ROLE_DETERMINERS
            .iter()
            .map(|(id, w)| rule(id, Cat::RoleNoun, vec![W(w), C(Cat::RoleNoun)], last)),
    );
    rules.extend(WITHIN_ROLE.iter().map(|(id, words)| {
        let mut rhs: Vec<Sym<Cat>> = words.iter().map(|w| W(w)).collect();
        rhs.push(C(Cat::RoleNoun));
        rule(id, Cat::Within, rhs, last)
    }));
    rules
}
