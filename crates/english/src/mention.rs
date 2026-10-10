//! Mentioning a thing: by name, with the kind the user says it is ("the
//! `X` struct", "fn `x`"), or by referring back to it ("it"). A thing set
//! off before a question ("`x` - what does it do") is what its "it" refers
//! back to, and a question asked twice ("what does `x` do, and what is it
//! for?") asks once.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{
    Condition, Description, Name, Ontology, Position, Question, Thing, Utterance,
};

use crate::sem::{Cat, Number, Sem};

use Sym::{Cat as C, Word as W};

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
        other => other,
    })
}

/// "`x` on the `App` struct": the thing with the mentioned name that stands
/// in the preposition's relation to the other thing, of the kind said.
fn related<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let called = |name: &Name, kind: &Option<O::Kind>, more| {
        Thing::Fitting(Description {
            kind: kind.clone(),
            conditions: [vec![Condition::Called(name.clone())], more].concat(),
        })
    };
    match c {
        [
            Sem::Mention {
                thing: Thing::Named(name),
                kind,
            },
            Sem::Relation { relation, .. },
            Sem::Mention { thing, kind: other },
        ] => {
            let other = match (thing, other) {
                (Thing::Named(n), Some(_)) => called(n, other, Vec::new()),
                (thing, _) => thing.clone(),
            };
            Some(Sem::Mention {
                thing: called(
                    name,
                    kind,
                    vec![Condition::Related {
                        relation: relation.clone(),
                        position: Position::First,
                        other,
                    }],
                ),
                kind: kind.clone(),
            })
        }
        _ => None,
    }
}

/// "`x` - what does it do": the question about the thing set off before it.
fn set_off<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Mention { thing, kind }, _, Sem::Question(later)]
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
/// second asks the same about what it refers back to.
fn restated<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let (first, second) = match c {
        [first, second] | [first, _, second] => (first, second),
        _ => return None,
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
        _ => None,
    }
}

/// Words joining two ways of asking one question.
const JOINS: &[(&str, &str)] = &[
    ("en.join.comma", ","),
    ("en.join.and", "and"),
    ("en.join.or", "or"),
    ("en.join.like", "like"),
];

/// Marks setting a thing off before a question about it.
const BREAKS: &[(&str, &str)] = &[
    ("en.break.dash", "-"),
    ("en.break.em_dash", "—"),
    ("en.break.comma", ","),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let blank = || S::Const(Sem::Quote(String::new()));
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
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
            Cat::Mention,
            vec![C(Cat::Mention), C(Cat::Noun)],
            of_kind,
        ),
        // "fn star", "the function `hm`"
        rule(
            "en.mention.kind_first",
            Cat::Mention,
            vec![C(Cat::Noun), C(Cat::Mention)],
            of_kind,
        ),
        // "the `x`", "the Walker"
        rule(
            "en.mention.the",
            Cat::Mention,
            vec![W("the"), C(Cat::Mention)],
            |c| c.first().cloned(),
        ),
        // "`save_filter` in `App`", "the `retry` method on `Chat`", "the
        // `sealed` field of the `Builder` struct"
        Rule::new(
            RuleInfo::new("en.mention.preposition", 5),
            Cat::Mention,
            vec![C(Cat::Mention), C(Cat::Preposition), C(Cat::Mention)],
            S::Build(related),
        )
        .read_only(),
        // "...": a trailing-off that sets what came before apart.
        Rule::new(
            RuleInfo::new("en.break.dots", 5),
            Cat::Break,
            vec![W("."), W("."), W(".")],
            blank(),
        )
        .read_only(),
        // "the name `job_git`": the thing that has the name.
        Rule::new(
            RuleInfo::new("en.mention.the_name", 5),
            Cat::Mention,
            vec![W("the"), C(Cat::NameNoun), C(Cat::Mention)],
            S::Build(|c| c.last().cloned()),
        )
        .read_only(),
        // "that function": the thing said before, of a kind.
        Rule::new(
            RuleInfo::new("en.mention.that_kind", 5),
            Cat::Mention,
            vec![W("that"), C(Cat::Noun)],
            S::Build(|c: &[Sem<O>]| match c {
                [Sem::Kind { kind, .. }] => Some(Sem::Mention {
                    thing: Thing::Previous,
                    kind: Some(kind.clone()),
                }),
                _ => None,
            }),
        )
        .read_only(),
        // "which fields does that struct have": the thing said before, as
        // the object of a verb.
        Rule::new(
            RuleInfo::new("en.noun_phrase.that_kind", 7),
            Cat::NounPhrase,
            vec![W("that"), C(Cat::Noun)],
            S::Const(Sem::Thing(Thing::Previous)),
        )
        .read_only(),
        // "it": the thing said before.
        Rule::new(
            RuleInfo::new("en.noun_phrase.it", ROUND),
            Cat::NounPhrase,
            vec![W("it")],
            S::Const(Sem::Thing(Thing::Previous)),
        )
        .read_only(),
        // "job_page - what does it do", "static_record, what's it for"
        rule(
            "en.question.set_off",
            Cat::Question,
            vec![C(Cat::Mention), C(Cat::Break), C(Cat::Question)],
            set_off,
        ),
        // "what does `x` do, and what is it for", "what is the Walker for,
        // like what does it do"
        rule(
            "en.question.restated",
            Cat::Question,
            vec![C(Cat::Question), C(Cat::Join), C(Cat::Question)],
            restated,
        ),
        // "what is `fit_s` for? what does it do?", "I'd like to know what
        // `hm` does. Could you explain it, please?"
        rule(
            "en.utterance.restated",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Utterance)],
            restated,
        ),
        // "what does mark_dead do exactly? like what is it for"
        rule(
            "en.utterance.restated_joined",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Join), C(Cat::Utterance)],
            restated,
        ),
        // ", and", ", or", ", like"
        rule(
            "en.join.comma_join",
            Cat::Join,
            vec![W(","), C(Cat::Join)],
            |c| c.first().cloned(),
        ),
    ];
    for (table, cat) in [(JOINS, Cat::Join), (BREAKS, Cat::Break)] {
        rules.extend(table.iter().map(|(id, mark)| {
            Rule::new(RuleInfo::new(id, ROUND), cat, vec![W(mark)], blank()).read_only()
        }));
    }
    rules
}
