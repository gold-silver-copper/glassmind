//! Mentioning a thing: by name, with the kind the user says it is ("the
//! `X` struct", "fn `x`"), or by referring back to it ("it"). A thing set
//! off before a question ("`x` - what does it do") is what its "it" refers
//! back to, and a question asked twice ("what does `x` do, and what is it
//! for?") asks once.

use glassmind_grammar::{Rule, RuleInfo, Sym};
use glassmind_meaning::{
    Condition, Description, Name, Ontology, Position, Question, Thing, Utterance,
};

use crate::aside::filler;
use crate::asking::read_only_rule;
use crate::contents::narrowed;
use crate::lexicon::{CAPITALIZED, CODE, LOWER, NAME, WORD};
use crate::locate::fillers;
use crate::sem::{Cat, Number, Sem};
use crate::spoken;
use crate::{first, word};

use Cat::*;
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
    // "field credentials, which structs have it": asking which things
    // stand in a relation to a thing named with its kind is asking about
    // every thing of that kind going by that name.
    let thing = match (later, thing, kind) {
        (Question::Identify(_) | Question::Count(_), Thing::Named(name), Some(_)) => {
            Thing::Fitting(Description {
                kind: kind.clone(),
                conditions: vec![Condition::Named(name.clone())],
            })
        }
        _ => thing.clone(),
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

/// A thing named by a known word, mentioned as an owner.
fn owner<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c.last()? {
        Sem::Thing(thing) => Some(Sem::Mention {
            thing: thing.clone(),
            kind: None,
        }),
        _ => None,
    }
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
        Join,
        &[
            ("en.join.comma", ","),
            ("en.join.and", "and"),
            ("en.join.or", "or"),
            ("en.join.like", "like"),
        ],
    ),
    // Marks setting a thing off before a question about it.
    (
        Break,
        &[
            ("en.break.dash", "-"),
            ("en.break.em_dash", "—"),
            ("en.break.comma", ","),
        ],
    ),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, round, lhs, rhs, build| read_only_rule(id, round, lhs, rhs, build);
    let mut rules = vec![
        // "`x`": a thing mentioned by name, of a kind left open.
        spoken(
            "en.mention.thing",
            ROUND,
            Mention,
            vec![C(NounPhrase)], // A description ("a `Gate` field") points at no one thing.
            |c| match c {
                [Sem::Thing(thing)] if !matches!(thing, Thing::Fitting(_)) => Some(Sem::Mention {
                    thing: thing.clone(),
                    kind: None,
                }),
                _ => None,
            },
            |v| match v {
                Sem::Mention { thing, kind: None } => Some(vec![Sem::Thing(thing.clone())]),
                _ => None,
            },
        ),
        // "the `X` struct": how the system says it, too.
        spoken(
            "en.mention.the_kind",
            ROUND,
            Mention,
            vec![W("the"), C(Mention), C(Noun)],
            of_kind,
            |v: &Sem<O>| match v {
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
            },
        ),
        // "`indexed` fn", "`Fixed` struct"
        rule(
            "en.mention.kind",
            ROUND,
            Mention,
            vec![C(Mention), C(Noun)],
            of_kind,
        ),
        // "fn star", "the function `hm`"
        rule(
            "en.mention.kind_first",
            ROUND,
            Mention,
            vec![C(Noun), C(Mention)],
            of_kind,
        ),
        // "refer to struct Store as ...", "the struct Handle": a thing of
        // the kind said, named by a known word capitalized the way type
        // names are.
        rule(
            "en.mention.kind_capitalized",
            23,
            Mention,
            vec![C(Noun), Class(CAPITALIZED)],
            |c| of_kind(&[c.first()?.clone(), owner(c)?]),
        ),
        // "`save_filter` in `App`", "the `retry` method on `Chat`", "the
        // `sealed` field of the `Builder` struct", "rename state_path in
        // Store to ..."
        rule(
            "en.mention.preposition",
            5,
            Mention,
            vec![C(Mention), C(Preposition), C(Owner)],
            related,
        ),
        rule("en.owner.mention", 22, Owner, vec![C(Mention)], first),
        // "rename state_path in Store to ...", "calling `http_put` on the
        // Store": the owner named by a known word, capitalized the way
        // type names are.
        rule(
            "en.owner.capitalized",
            22,
            Owner,
            vec![Class(CAPITALIZED)],
            owner,
        ),
        rule(
            "en.owner.the_capitalized",
            22,
            Owner,
            vec![W("the"), Class(CAPITALIZED)],
            owner,
        ),
        // "which functions call `accent` on Theme", "what does Chat's
        // `paste` call", "which funcs does Chat retry invoke": a thing
        // mentioned by what it is to another, as a verb's subject or object.
        rule(
            "en.noun_phrase.described",
            22,
            NounPhrase,
            vec![C(Mention)],
            |c| match c {
                [
                    Sem::Mention {
                        thing: thing @ Thing::Fitting(_),
                        ..
                    },
                ] => Some(Sem::Thing(thing.clone())),
                _ => None,
            },
        ),
        // "...": a trailing-off that sets what came before apart.
        filler("en.break.dots", 5, Break, ". . ."),
        // "..": the same, typed short.
        filler("en.break.two_dots", 11, Break, ". ."),
        // "the `move_covers` system": a thing named as code, with a noun
        // of the user's own for its kind, which says nothing the system can
        // check.
        rule(
            "en.mention.own_noun_after",
            11,
            Mention,
            vec![W("the"), Class(CODE), Class(WORD)],
            |c| named(c.first()),
        ),
        // "the test helper `fake_list`": the same, with the user's own noun
        // before the name, after a kind.
        rule(
            "en.mention.kind_own_noun_before",
            21,
            Mention,
            vec![W("the"), C(Noun), Class(LOWER), C(Mention)],
            |c| match c {
                [kind, _, mention] => of_kind(&[kind.clone(), mention.clone()]),
                _ => None,
            },
        ),
        // "the name `job_git`": the thing that has the name.
        rule(
            "en.mention.the_name",
            5,
            Mention,
            vec![W("the"), C(NameNoun), C(Mention)],
            |c| c.last().cloned(),
        ),
        // "that function": the thing said before, of a kind.
        rule(
            "en.mention.that_kind",
            5,
            Mention,
            vec![W("that"), C(Noun)],
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
            NounPhrase,
            vec![W("that"), C(Noun)],
            |_| Some(Sem::Thing(Thing::Previous)),
        ),
        // "what's inside `x`, like which fields the struct has?": the
        // thing said before, named again by its kind.
        rule(
            "en.noun_phrase.the_kind",
            14,
            NounPhrase,
            vec![W("the"), C(Noun)],
            |_| Some(Sem::Thing(Thing::Previous)),
        ),
        // "job_page - what does it do", "static_record, what's it for"
        rule(
            "en.question.set_off",
            ROUND,
            Question,
            vec![C(Mention), C(Break), C(Question)],
            set_off,
        ),
        // "is_rate_limit where is it", "SimPlugin where it live": the same,
        // with no mark between.
        rule(
            "en.question.set_off_unmarked",
            13,
            Question,
            vec![C(Mention), C(Question)],
            set_off,
        ),
        // "what does `x` do, and what is it for", "what is the Walker for,
        // like what does it do", "what's inside `Bank`, which fields it has"
        rule(
            "en.question.restated",
            ROUND,
            Question,
            vec![C(Question), C(Join), C(Question)],
            restated,
        ),
        // "what is `fit_s` for? what does it do?", "I'd like to know what
        // `hm` does. Could you explain it, please?"
        rule(
            "en.utterance.restated",
            ROUND,
            Utterance,
            vec![C(Utterance), C(Utterance)],
            restated,
        ),
        // "what does mark_dead do exactly? like what is it for"
        rule(
            "en.utterance.restated_joined",
            ROUND,
            Utterance,
            vec![C(Utterance), C(Join), C(Utterance)],
            restated,
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
                NounPhrase,
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
        NounPhrase,
        vec![W("the"), Class(NAME)],
        |c| match c {
            [Sem::Name(n)] => Some(Sem::Thing(Thing::Named(n.clone()))),
            _ => None,
        },
    ));
    rules.extend(fillers(ROUND, FILLERS));
    // "what `x` do / whats it for": two ways of asking, split by a slash.
    rules.extend(fillers(14, &[(Join, &[("en.join.slash", "/")])]));
    // "which functions call `step_match`, in the Chat": the thing so
    // placed, set off by a comma, as the object of a verb.
    rules.push(rule(
        "en.noun_phrase.preposition_set_off",
        22,
        NounPhrase,
        vec![C(Mention), W(","), C(Preposition), C(Owner)],
        |c| match related(c)? {
            Sem::Mention { thing, .. } => Some(Sem::Thing(thing)),
            _ => None,
        },
    ));
    rules
}
