//! Mentioning a thing: by name, with the kind the user says it is ("the
//! `X` struct", "fn `x`"), or by referring back to it ("it"). A thing set
//! off before a question ("`x` - what does it do") is what its "it" refers
//! back to, and a question asked twice ("what does `x` do, and what is it
//! for?") asks once.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Question, Thing, Utterance};

use crate::sem::{Cat, Number, Sem};

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 3;

/// The mention among the children, with the kind among them.
fn of_kind<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
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
                |c| match c {
                    [Sem::Thing(thing)] => Some(Sem::Mention {
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
