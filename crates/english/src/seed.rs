//! The hand-written seed of the English grammar (round 0).
//!
//! Just enough to ask which things stand in a relation to a named thing
//! ("what calls `spawn_wave`?", "which functions call `spawn_wave`?") and to
//! say the answer, plus the replies for each kind of trouble. Everything
//! else in the English grammar is grown by the loop.

use glassmind_grammar::{Rule, RuleInfo, SEED, Sem as S, Sym};
use glassmind_meaning::{
    Assumption, Condition, Description, Extent, Ontology, Position, Question, Reply, Thing,
    Trouble, Utterance,
};

use crate::lexicon::{ITEM, NAME, NUMBER, QUOTE};
use crate::sem::{Cat, Number, Sem, VerbForm};

use Sym::{Cat as C, Class, Word as W};

fn number_of(count: usize) -> Number {
    if count == 1 {
        Number::Singular
    } else {
        Number::Plural
    }
}

fn form_for(number: Number) -> VerbForm {
    match number {
        Number::Singular => VerbForm::ThirdSingular,
        Number::Plural => VerbForm::Plain,
    }
}

/// The one condition of a description, if it has exactly one.
fn only_condition<O: Ontology>(d: &Description<O>) -> Option<&Condition<O>> {
    match d.conditions.as_slice() {
        [c] => Some(c),
        _ => None,
    }
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    vec![
        // A question, with or without its question mark.
        Rule::new(
            RuleInfo::new("en.utterance.question_mark", SEED),
            Cat::Utterance,
            vec![C(Cat::Question), W("?")],
            S::Both(ask, unask),
        ),
        Rule::new(
            RuleInfo::new("en.utterance.question", SEED),
            Cat::Utterance,
            vec![C(Cat::Question)],
            S::Both(ask, unask),
        )
        .read_only(),
        // "what calls `x`": identify things of any kind meeting a condition.
        Rule::new(
            RuleInfo::new("en.question.what", SEED),
            Cat::Question,
            vec![W("what"), C(Cat::VerbPhrase)],
            S::Both(
                |c| match c {
                    [Sem::Condition { condition, .. }] => {
                        Some(Sem::Question(Question::Identify(Description {
                            kind: None,
                            conditions: vec![condition.clone()],
                        })))
                    }
                    _ => None,
                },
                |v| match v {
                    Sem::Question(Question::Identify(d)) if d.kind.is_none() => {
                        let condition = only_condition(d)?.clone();
                        Some(vec![Sem::Condition {
                            condition,
                            number: Number::Singular,
                        }])
                    }
                    _ => None,
                },
            ),
        ),
        // "which functions call `x`": identify things of a kind meeting a condition.
        Rule::new(
            RuleInfo::new("en.question.which", SEED),
            Cat::Question,
            vec![W("which"), C(Cat::Noun), C(Cat::VerbPhrase)],
            S::Both(
                |c| match c {
                    [Sem::Kind { kind, .. }, Sem::Condition { condition, .. }] => {
                        Some(Sem::Question(Question::Identify(Description {
                            kind: Some(kind.clone()),
                            conditions: vec![condition.clone()],
                        })))
                    }
                    _ => None,
                },
                |v| match v {
                    Sem::Question(Question::Identify(d)) => {
                        let kind = d.kind.clone()?;
                        let condition = only_condition(d)?.clone();
                        Some(vec![
                            Sem::Kind {
                                kind,
                                number: Number::Plural,
                            },
                            Sem::Condition {
                                condition,
                                number: Number::Plural,
                            },
                        ])
                    }
                    _ => None,
                },
            ),
        ),
        // "calls `x`": the subject is the first argument of the verb's relation.
        Rule::new(
            RuleInfo::new("en.verb_phrase.object", SEED),
            Cat::VerbPhrase,
            vec![C(Cat::Verb), C(Cat::NounPhrase)],
            S::Both(
                |c| match c {
                    [Sem::Relation { relation, form }, Sem::Thing(other)] => Some(Sem::Condition {
                        condition: Condition::Related {
                            relation: relation.clone(),
                            position: Position::First,
                            other: other.clone(),
                        },
                        number: form.agrees_with(),
                    }),
                    _ => None,
                },
                |v| match v {
                    Sem::Condition {
                        condition:
                            Condition::Related {
                                relation,
                                position: Position::First,
                                other,
                            },
                        number,
                    } => Some(vec![
                        Sem::Relation {
                            relation: relation.clone(),
                            form: form_for(*number),
                        },
                        Sem::Thing(other.clone()),
                    ]),
                    _ => None,
                },
            ),
        ),
        // A thing named by the user.
        Rule::new(
            RuleInfo::new("en.noun_phrase.name", SEED),
            Cat::NounPhrase,
            vec![Class(NAME)],
            S::Both(
                |c| match c {
                    [Sem::Name(n)] => Some(Sem::Thing(Thing::Named(n.clone()))),
                    _ => None,
                },
                |v| match v {
                    Sem::Thing(Thing::Named(n)) => Some(vec![Sem::Name(n.clone())]),
                    _ => None,
                },
            ),
        ),
        // A resolved item, as the system speaks of it.
        Rule::new(
            RuleInfo::new("en.noun_phrase.item", SEED),
            Cat::NounPhrase,
            vec![Class(ITEM)],
            S::Both(
                |c| match c {
                    [Sem::Item(i)] => Some(Sem::Thing(Thing::Item(i.clone()))),
                    _ => None,
                },
                |v| match v {
                    Sem::Thing(Thing::Item(i)) => Some(vec![Sem::Item(i.clone())]),
                    _ => None,
                },
            ),
        ),
        // "`a` and `b` call `x`."
        Rule::new(
            RuleInfo::new("en.statement.list", SEED),
            Cat::Statement,
            vec![C(Cat::ItemList), C(Cat::VerbPhrase)],
            S::Both(
                |c| match c {
                    [Sem::Items(members), Sem::Condition { condition, .. }] => {
                        Some(Sem::Extent(Extent {
                            description: Description {
                                kind: None,
                                conditions: vec![condition.clone()],
                            },
                            members: members.clone(),
                        }))
                    }
                    _ => None,
                },
                |v| match v {
                    Sem::Extent(e) if e.description.kind.is_none() && !e.members.is_empty() => {
                        let condition = only_condition(&e.description)?.clone();
                        Some(vec![
                            Sem::Items(e.members.clone()),
                            Sem::Condition {
                                condition,
                                number: number_of(e.members.len()),
                            },
                        ])
                    }
                    _ => None,
                },
            ),
        ),
        // "2 functions call `x`: `a` and `b`."
        Rule::new(
            RuleInfo::new("en.statement.counted", SEED),
            Cat::Statement,
            vec![
                Class(NUMBER),
                C(Cat::Noun),
                C(Cat::VerbPhrase),
                W(":"),
                C(Cat::ItemList),
            ],
            S::Both(
                |c| match c {
                    [
                        Sem::Count(n),
                        Sem::Kind { kind, .. },
                        Sem::Condition { condition, .. },
                        Sem::Items(members),
                    ] if usize::try_from(*n).ok() == Some(members.len()) => {
                        Some(Sem::Extent(Extent {
                            description: Description {
                                kind: Some(kind.clone()),
                                conditions: vec![condition.clone()],
                            },
                            members: members.clone(),
                        }))
                    }
                    _ => None,
                },
                |v| match v {
                    Sem::Extent(e) if !e.members.is_empty() => {
                        let kind = e.description.kind.clone()?;
                        let condition = only_condition(&e.description)?.clone();
                        let number = number_of(e.members.len());
                        Some(vec![
                            Sem::Count(u64::try_from(e.members.len()).ok()?),
                            Sem::Kind { kind, number },
                            Sem::Condition { condition, number },
                            Sem::Items(e.members.clone()),
                        ])
                    }
                    _ => None,
                },
            ),
        ),
        // "nothing calls `x`."
        Rule::new(
            RuleInfo::new("en.statement.nothing", SEED),
            Cat::Statement,
            vec![W("nothing"), C(Cat::VerbPhrase)],
            S::Both(
                |c| match c {
                    [Sem::Condition { condition, .. }] => Some(Sem::Extent(Extent {
                        description: Description {
                            kind: None,
                            conditions: vec![condition.clone()],
                        },
                        members: Vec::new(),
                    })),
                    _ => None,
                },
                |v| match v {
                    Sem::Extent(e) if e.description.kind.is_none() && e.members.is_empty() => {
                        Some(vec![Sem::Condition {
                            condition: only_condition(&e.description)?.clone(),
                            number: Number::Singular,
                        }])
                    }
                    _ => None,
                },
            ),
        ),
        // "no functions call `x`."
        Rule::new(
            RuleInfo::new("en.statement.no_kind", SEED),
            Cat::Statement,
            vec![W("no"), C(Cat::Noun), C(Cat::VerbPhrase)],
            S::Both(
                |c| match c {
                    [Sem::Kind { kind, .. }, Sem::Condition { condition, .. }] => {
                        Some(Sem::Extent(Extent {
                            description: Description {
                                kind: Some(kind.clone()),
                                conditions: vec![condition.clone()],
                            },
                            members: Vec::new(),
                        }))
                    }
                    _ => None,
                },
                |v| match v {
                    Sem::Extent(e) if e.members.is_empty() => Some(vec![
                        Sem::Kind {
                            kind: e.description.kind.clone()?,
                            number: Number::Plural,
                        },
                        Sem::Condition {
                            condition: only_condition(&e.description)?.clone(),
                            number: Number::Plural,
                        },
                    ]),
                    _ => None,
                },
            ),
        ),
        // Lists of items: "`a`", "`a` and `b`", "`a`, `b` and `c`".
        Rule::new(
            RuleInfo::new("en.item_list.one", SEED),
            Cat::ItemList,
            vec![Class(ITEM)],
            S::Both(
                |c| match c {
                    [Sem::Item(i)] => Some(Sem::Items(vec![i.clone()])),
                    _ => None,
                },
                |v| match v {
                    Sem::Items(items) => match items.as_slice() {
                        [i] => Some(vec![Sem::Item(i.clone())]),
                        _ => None,
                    },
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.item_list.two", SEED),
            Cat::ItemList,
            vec![Class(ITEM), W("and"), Class(ITEM)],
            S::Both(
                |c| match c {
                    [Sem::Item(a), Sem::Item(b)] => Some(Sem::Items(vec![a.clone(), b.clone()])),
                    _ => None,
                },
                |v| match v {
                    Sem::Items(items) => match items.as_slice() {
                        [a, b] => Some(vec![Sem::Item(a.clone()), Sem::Item(b.clone())]),
                        _ => None,
                    },
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.item_list.more", SEED),
            Cat::ItemList,
            vec![Class(ITEM), W(","), C(Cat::ItemList)],
            S::Both(
                |c| match c {
                    [Sem::Item(first), Sem::Items(rest)] if rest.len() >= 2 => {
                        let mut items = vec![first.clone()];
                        items.extend(rest.iter().cloned());
                        Some(Sem::Items(items))
                    }
                    _ => None,
                },
                |v| match v {
                    Sem::Items(items) if items.len() >= 3 => {
                        let (first, rest) = items.split_first()?;
                        Some(vec![Sem::Item(first.clone()), Sem::Items(rest.to_vec())])
                    }
                    _ => None,
                },
            ),
        ),
        // Replies.
        Rule::new(
            RuleInfo::new("en.reply.extent", SEED),
            Cat::Reply,
            vec![C(Cat::Statement), W(".")],
            S::Both(
                |c| match c {
                    [Sem::Extent(e)] => Some(Sem::Reply(Reply::Extent(e.clone()))),
                    _ => None,
                },
                |v| match v {
                    Sem::Reply(Reply::Extent(e)) => Some(vec![Sem::Extent(e.clone())]),
                    _ => None,
                },
            ),
        ),
        // "By `x` I assume you mean `Foo::x`. ..." or "I take that to mean: ...? ..."
        Rule::new(
            RuleInfo::new("en.reply.assuming", SEED),
            Cat::Reply,
            vec![C(Cat::Assumption), C(Cat::Reply)],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Assuming { assumptions, reply }) => {
                        let (first, rest) = assumptions.split_first()?;
                        let rest = if rest.is_empty() {
                            (**reply).clone()
                        } else {
                            Reply::Assuming {
                                assumptions: rest.to_vec(),
                                reply: reply.clone(),
                            }
                        };
                        Some(vec![Sem::Assumption(first.clone()), Sem::Reply(rest)])
                    }
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.assumption.means", SEED),
            Cat::Assumption,
            vec![
                W("by"),
                Class(NAME),
                W("I"),
                W("assume"),
                W("you"),
                W("mean"),
                Class(ITEM),
                W("."),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Assumption(Assumption::Means { name, item }) => {
                        Some(vec![Sem::Name(name.clone()), Sem::Item(item.clone())])
                    }
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.assumption.reading", SEED),
            Cat::Assumption,
            vec![
                W("I"),
                W("take"),
                W("that"),
                W("to"),
                W("mean"),
                W(":"),
                C(Cat::Utterance),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Assumption(Assumption::Reading { meaning }) => {
                        Some(vec![Sem::Utterance(meaning.clone())])
                    }
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.reply.unknown_word", SEED),
            Cat::Reply,
            vec![
                W("I"),
                W("don't"),
                W("know"),
                W("the"),
                W("word"),
                Class(QUOTE),
                W("."),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Trouble(Trouble::UnknownWord { word, .. })) => {
                        Some(vec![Sem::Quote(word.clone())])
                    }
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.reply.unknown_shape", SEED),
            Cat::Reply,
            vec![
                W("I"),
                W("can't"),
                W("follow"),
                W("the"),
                W("sentence"),
                W("from"),
                W("the"),
                W("marked"),
                W("spot"),
                W("."),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Trouble(Trouble::UnknownShape { .. })) => Some(Vec::new()),
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.reply.no_such_name", SEED),
            Cat::Reply,
            vec![W("nothing"), W("is"), W("called"), Class(NAME), W(".")],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Trouble(Trouble::NoSuchName { name })) => {
                        Some(vec![Sem::Name(name.clone())])
                    }
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.reply.wrong_kind", SEED),
            Cat::Reply,
            vec![
                Class(ITEM),
                W("is"),
                W("the"),
                W("wrong"),
                W("kind"),
                W("of"),
                W("thing"),
                W("for"),
                W("that"),
                W("."),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Trouble(Trouble::WrongKind { item })) => {
                        Some(vec![Sem::Item(item.clone())])
                    }
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.reply.ambiguous", SEED),
            Cat::Reply,
            vec![
                W("I"),
                W("can't"),
                W("tell"),
                W("which"),
                W("one"),
                W("you"),
                W("mean"),
                W(":"),
                C(Cat::ItemList),
                W("."),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Trouble(Trouble::Ambiguous { candidates })) => {
                        Some(vec![Sem::Items(candidates.clone())])
                    }
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.reply.unsupported", SEED),
            Cat::Reply,
            vec![
                W("I"),
                W("understand"),
                W(","),
                W("but"),
                W("I"),
                W("can't"),
                W("do"),
                W("that"),
                W("here"),
                W("."),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Trouble(Trouble::Unsupported)) => Some(Vec::new()),
                    _ => None,
                },
            ),
        ),
        Rule::new(
            RuleInfo::new("en.reply.too_ambiguous", SEED),
            Cat::Reply,
            vec![
                W("that"),
                W("has"),
                W("too"),
                W("many"),
                W("readings"),
                W("for"),
                W("me"),
                W("."),
            ],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Trouble(Trouble::TooAmbiguous)) => Some(Vec::new()),
                    _ => None,
                },
            ),
        ),
    ]
}

fn ask<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Question(q)] => Some(Sem::Utterance(Utterance::Ask(q.clone()))),
        _ => None,
    }
}

fn unask<O: Ontology>(v: &Sem<O>) -> Option<Vec<Sem<O>>> {
    match v {
        Sem::Utterance(Utterance::Ask(q)) => Some(vec![Sem::Question(q.clone())]),
        _ => None,
    }
}
