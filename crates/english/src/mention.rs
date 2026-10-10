//! Mentioning a thing: by name, with the kind the user says it is ("the
//! `X` struct", "fn `x`"), or by referring back to it ("it"). A thing set
//! off before a question ("`x` - what does it do") is what its "it" refers
//! back to, and a question asked twice ("what does `x` do, and what is it
//! for?") asks once.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{
    Condition, Description, Name, Ontology, Position, Question, Thing, Utterance,
};

use crate::contents::narrowed;
use crate::lexicon::{CODE, NAME, WORD};
use crate::locate::fillers;
use crate::sem::{Cat, Number, Sem};
use crate::{blank, first, word};

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 3;

/// The mention among the children, with the kind among them.
pub(crate) fn of_kind<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Mention { thing, kind: None }, Sem::Kind { kind, .. }]
        | [Sem::Kind { kind, .. }, Sem::Mention { thing, kind: None }] => Some(Sem::Mention {
            thing: thing.clone(),
            kind: Some(kind.clone()),
        }),
        _ => None,
    }
}

/// The question `later` asks when what it refers back to is the thing
/// `earlier` asks about, taken to be of the kind `earlier` says.
fn refer_back<O: Ontology>(later: &Question<O>, earlier: &Question<O>) -> Option<Question<O>> {
    let (thing, kind) = match earlier {
        Question::Describe { thing, kind } | Question::Locate { thing, kind } => (thing, kind),
        Question::Identify(_) | Question::Count(_) => return None,
    };
    let bound = later.map_things(&mut |t| match t {
        Thing::Previous => thing.clone(),
        other => other.clone(),
    });
    Some(match bound {
        Question::Describe { thing, kind: None } => Question::Describe {
            thing,
            kind: kind.clone(),
        },
        Question::Locate { thing, kind: None } => Question::Locate {
            thing,
            kind: kind.clone(),
        },
        other => other,
    })
}

/// "`x` on the `App` struct": the thing with the mentioned name that stands
/// in the preposition's relation to the other thing, of the kind said.
pub(crate) fn related<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let [
        Sem::Mention {
            thing: Thing::Named(name),
            kind,
        },
        Sem::Relation { relation, .. },
        Sem::Mention { thing, kind: other },
    ] = c
    else {
        return None;
    };
    let called = |name: &Name, kind: &Option<O::Kind>, more| {
        Thing::Fitting(Description {
            kind: kind.clone(),
            conditions: [vec![Condition::Called(name.clone())], more].concat(),
        })
    };
    let other = match (thing, other) {
        (Thing::Named(n), Some(_)) => called(n, other, Vec::new()),
        _ => thing.clone(),
    };
    let related = Condition::Related {
        relation: relation.clone(),
        position: Position::First,
        other,
    };
    Some(Sem::Mention {
        thing: called(name, kind, vec![related]),
        kind: kind.clone(),
    })
}

/// The thing with the name, of a kind left open.
pub(crate) fn named<O: Ontology>(v: Option<&Sem<O>>) -> Option<Sem<O>> {
    match v? {
        Sem::Name(name) => Some(Sem::Mention {
            thing: Thing::Named(name.clone()),
            kind: None,
        }),
        _ => None,
    }
}

/// "`x` - what does it do", "`x` where is it": the question about the
/// thing set off before it.
fn set_off<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Mention { thing, kind }, .., Sem::Question(later)]
            if later.things().contains(&&Thing::Previous) =>
        {
            let earlier = Question::Describe {
                thing: thing.clone(),
                kind: kind.clone(),
            };
            refer_back(later, &earlier).map(Sem::Question)
        }
        _ => None,
    }
}

/// "what does `x` do, and what is it for?": the first question, when the
/// second asks the same about what it refers back to; or the second, when
/// it narrows the first to things of a kind ([`narrowed`]).
fn restated<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let ([first, second] | [first, _, second]) = c else {
        return None;
    };
    let same = |a: &Question<O>, b: &Question<O>| refer_back(b, a).as_ref() == Some(a);
    match (first, second) {
        (Sem::Question(a), Sem::Question(b))
        | (Sem::Utterance(Utterance::Ask(a)), Sem::Utterance(Utterance::Ask(b)))
            if same(a, b) =>
        {
            Some(first.clone())
        }
        // "why do you want to change that? what's the reason?", "never
        // mind, forget that"
        (Sem::Utterance(a), Sem::Utterance(b)) if a == b => Some(first.clone()),
        (Sem::Question(a), Sem::Question(b)) => narrowed(a, b).map(Sem::Question),
        // "is there anything that calls `x`? like how many functions call
        // it": the count of the same things.
        (Sem::Utterance(Utterance::Ask(a)), Sem::Utterance(Utterance::Ask(b))) => match (a, b) {
            (Question::Identify(a), Question::Count(b)) => counted(a, b).map(Question::Count),
            _ => None,
        }
        .or_else(|| narrowed(a, b))
        .map(|q| Sem::Utterance(Utterance::Ask(q))),
        _ => None,
    }
}

/// `count`, if it counts the things `which` asks for, its "it" being the
/// one thing `which` refers to, whatever kind either says they are.
fn counted<O: Ontology>(which: &Description<O>, count: &Description<O>) -> Option<Description<O>> {
    let things = which.things();
    let [thing] = things.as_slice() else {
        return None;
    };
    let bound = count.map_things(&mut |t| match t {
        Thing::Previous => (*thing).clone(),
        other => other.clone(),
    });
    (bound.conditions == which.conditions).then_some(bound)
}

/// Words and marks that only join or set off phrases, by category.
const FILLERS: &[(Cat, &[(&str, &str)])] = &[
    // Words joining two ways of asking one question.
    (
        Cat::Join,
        &[
            ("en.join.comma", ","),
            ("en.join.and", "and"),
            ("en.join.or", "or"),
            ("en.join.like", "like"),
        ],
    ),
    // Marks setting a thing off before a question about it.
    (
        Cat::Break,
        &[
            ("en.break.dash", "-"),
            ("en.break.em_dash", "—"),
            ("en.break.comma", ","),
        ],
    ),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, round, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, round), lhs, rhs, S::Build(build)).read_only()
    };
    let mut rules = vec![
        // "`x`": a thing mentioned by name, of a kind left open.
        Rule::new(
            RuleInfo::new("en.mention.thing", ROUND),
            Cat::Mention,
            vec![C(Cat::NounPhrase)],
            S::Both(
                // A description ("a `Gate` field") points at no one thing.
                |c| match c {
                    [Sem::Thing(thing)] if !matches!(thing, Thing::Fitting(_)) => {
                        Some(Sem::Mention {
                            thing: thing.clone(),
                            kind: None,
                        })
                    }
                    _ => None,
                },
                |v| match v {
                    Sem::Mention { thing, kind: None } => Some(vec![Sem::Thing(thing.clone())]),
                    _ => None,
                },
            ),
        ),
        // "the `X` struct": how the system says it, too.
        Rule::new(
            RuleInfo::new("en.mention.the_kind", ROUND),
            Cat::Mention,
            vec![W("the"), C(Cat::Mention), C(Cat::Noun)],
            S::Both(of_kind, |v: &Sem<O>| match v {
                Sem::Mention {
                    thing,
                    kind: Some(kind),
                } => Some(vec![
                    Sem::Mention {
                        thing: thing.clone(),
                        kind: None,
                    },
                    Sem::Kind {
                        kind: kind.clone(),
                        number: Number::Singular,
                    },
                ]),
                _ => None,
            }),
        ),
        // "`indexed` fn", "`Fixed` struct"
        rule(
            "en.mention.kind",
            ROUND,
            Cat::Mention,
            vec![C(Cat::Mention), C(Cat::Noun)],
            of_kind,
        ),
        // "fn star", "the function `hm`"
        rule(
            "en.mention.kind_first",
            ROUND,
            Cat::Mention,
            vec![C(Cat::Noun), C(Cat::Mention)],
            of_kind,
        ),
        // "the `x`", "the Walker"
        rule(
            "en.mention.the",
            ROUND,
            Cat::Mention,
            vec![W("the"), C(Cat::Mention)],
            first,
        ),
        // "`save_filter` in `App`", "the `retry` method on `Chat`", "the
        // `sealed` field of the `Builder` struct"
        rule(
            "en.mention.preposition",
            5,
            Cat::Mention,
            vec![C(Cat::Mention), C(Cat::Preposition), C(Cat::Mention)],
            related,
        ),
        // "which functions call `accent` on Theme": the thing so placed,
        // as the object of a verb.
        rule(
            "en.noun_phrase.preposition",
            9,
            Cat::NounPhrase,
            vec![C(Cat::Mention), C(Cat::Preposition), C(Cat::Mention)],
            |c| match related(c)? {
                Sem::Mention { thing, .. } => Some(Sem::Thing(thing)),
                _ => None,
            },
        ),
        // "...": a trailing-off that sets what came before apart.
        word(
            RuleInfo::new("en.break.dots", 5),
            Cat::Break,
            ". . .",
            blank(),
        )
        .read_only(),
        // "..": the same, typed short.
        word(
            RuleInfo::new("en.break.two_dots", 11),
            Cat::Break,
            ". .",
            blank(),
        )
        .read_only(),
        // "the `move_covers` system": a thing named as code, with a noun
        // of the user's own for its kind, which says nothing the system can
        // check.
        rule(
            "en.mention.own_noun_after",
            11,
            Cat::Mention,
            vec![W("the"), Class(CODE), Class(WORD)],
            |c| named(c.first()),
        ),
        // "the name `job_git`": the thing that has the name.
        rule(
            "en.mention.the_name",
            5,
            Cat::Mention,
            vec![W("the"), C(Cat::NameNoun), C(Cat::Mention)],
            |c| c.last().cloned(),
        ),
        // "that function": the thing said before, of a kind.
        rule(
            "en.mention.that_kind",
            5,
            Cat::Mention,
            vec![W("that"), C(Cat::Noun)],
            |c| match c {
                [Sem::Kind { kind, .. }] => Some(Sem::Mention {
                    thing: Thing::Previous,
                    kind: Some(kind.clone()),
                }),
                _ => None,
            },
        ),
        // "which fields does that struct have": the thing said before, as
        // the object of a verb.
        rule(
            "en.noun_phrase.that_kind",
            7,
            Cat::NounPhrase,
            vec![W("that"), C(Cat::Noun)],
            |_| Some(Sem::Thing(Thing::Previous)),
        ),
        // "job_page - what does it do", "static_record, what's it for"
        rule(
            "en.question.set_off",
            ROUND,
            Cat::Question,
            vec![C(Cat::Mention), C(Cat::Break), C(Cat::Question)],
            set_off,
        ),
        // "is_rate_limit where is it", "SimPlugin where it live": the same,
        // with no mark between.
        rule(
            "en.question.set_off_unmarked",
            13,
            Cat::Question,
            vec![C(Cat::Mention), C(Cat::Question)],
            set_off,
        ),
        // "what does `x` do, and what is it for", "what is the Walker for,
        // like what does it do", "what's inside `Bank`, which fields it has"
        rule(
            "en.question.restated",
            ROUND,
            Cat::Question,
            vec![C(Cat::Question), C(Cat::Join), C(Cat::Question)],
            restated,
        ),
        // "what is `fit_s` for? what does it do?", "I'd like to know what
        // `hm` does. Could you explain it, please?"
        rule(
            "en.utterance.restated",
            ROUND,
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Utterance)],
            restated,
        ),
        // "what does mark_dead do exactly? like what is it for"
        rule(
            "en.utterance.restated_joined",
            ROUND,
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Join), C(Cat::Utterance)],
            restated,
        ),
        // ", and", ", or", ", like"
        rule(
            "en.join.comma_join",
            ROUND,
            Cat::Join,
            vec![W(","), C(Cat::Join)],
            first,
        ),
    ];
    // Words for the thing said before: "it", and "who calls that",
    // "where's that defined", "which functions call that one?".
    rules.extend(
        [
            ("en.noun_phrase.it", ROUND, "it"),
            ("en.noun_phrase.that", 9, "that"),
            ("en.noun_phrase.that_one", 9, "that one"),
        ]
        .map(|(id, round, words)| {
            word(
                RuleInfo::new(id, round),
                Cat::NounPhrase,
                words,
                Sem::Thing(Thing::Previous),
            )
            .read_only()
        }),
    );
    // "functions call the `index_bits`": a name with an article before it.
    rules.push(rule(
        "en.noun_phrase.the_name",
        11,
        Cat::NounPhrase,
        vec![W("the"), Class(NAME)],
        |c| match c {
            [Sem::Name(n)] => Some(Sem::Thing(Thing::Named(n.clone()))),
            _ => None,
        },
    ));
    rules.extend(fillers(ROUND, FILLERS));
    rules
}
