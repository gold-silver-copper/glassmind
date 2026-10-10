//! The hand-written seed of the English grammar (round 0).
//!
//! Just enough to ask which things stand in a relation to a named thing
//! ("what calls `spawn_wave`?", "which functions call `spawn_wave`?") and to
//! say the answer, plus the replies for each kind of trouble. Everything
//! else in the English grammar is grown by the loop.

use glassmind_grammar::{Rule, SEED, Sym};
use glassmind_meaning::{
    Assumption, Condition, Description, Extent, Ontology, Position, Question, Reply, Thing,
    Trouble, Utterance,
};

use crate::lexicon::{ITEM, NAME, NUMBER};
use crate::sem::{Cat, Number, Sem, VerbForm};
use crate::spoken;

use Cat::*;
use Sym::{Cat as C, Class, Word as W};

type Read<O> = fn(&[Sem<O>]) -> Option<Sem<O>>;
type Speak<O> = fn(&Sem<O>) -> Option<Vec<Sem<O>>>;

/// A seed rule that reads its children with `read` and speaks with `speak`.
fn seed<O: Ontology>(
    id: &'static str,
    cat: Cat,
    rhs: Vec<Sym<Cat>>,
    read: Read<O>,
    speak: Speak<O>,
) -> Rule<Cat, Sem<O>> {
    spoken(id, SEED, cat, rhs, read, speak)
}

/// A seed rule only the system says: a reply the user never types, so it
/// reads nothing.
fn said<O: Ontology>(
    id: &'static str,
    cat: Cat,
    rhs: Vec<Sym<Cat>>,
    speak: Speak<O>,
) -> Rule<Cat, Sem<O>> {
    seed(id, cat, rhs, |_| None, speak)
}

fn number_of(count: usize) -> Number {
    if count == 1 {
        Number::Singular
    } else {
        Number::Plural
    }
}

/// The description a rule's children say: a condition on things of any
/// kind ("calls `x`"), or a kind and a condition ("functions" "call `x`").
fn described<O: Ontology>(c: &[Sem<O>]) -> Option<Description<O>> {
    let (kind, condition) = match c {
        [Sem::Condition { condition, .. }] => (None, condition),
        [Sem::Kind { kind, .. }, Sem::Condition { condition, .. }] => (Some(kind), condition),
        _ => return None,
    };
    Some(Description {
        kind: kind.cloned(),
        conditions: vec![condition.clone()],
    })
}

/// The children that say a description of one condition, agreeing with
/// `number`: its kind and condition if `kinded`, else just its condition
/// (then it must have no kind).
fn undescribed<O: Ontology>(
    d: &Description<O>,
    kinded: bool,
    number: Number,
) -> Option<Vec<Sem<O>>> {
    let [condition] = d.conditions.as_slice() else {
        return None;
    };
    let condition = Sem::Condition {
        condition: condition.clone(),
        number,
    };
    match &d.kind {
        Some(kind) if kinded => Some(vec![
            Sem::Kind {
                kind: kind.clone(),
                number,
            },
            condition,
        ]),
        None if !kinded => Some(vec![condition]),
        _ => None,
    }
}

/// "calls `x`": the subject is the first argument of the verb's relation.
pub(crate) fn object<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Relation { relation, form }, Sem::Thing(other)] => Some(Sem::Condition {
            condition: Condition::Related {
                relation: relation.clone(),
                position: Position::First,
                other: other.clone(),
            },
            number: form.agrees_with(),
        }),
        _ => None,
    }
}

/// Asks which things fit the description the children say.
fn identify<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    Some(Sem::Question(Question::Identify(described(c)?)))
}

/// The children that ask which things fit a description, as `undescribed`.
fn unidentify<O: Ontology>(v: &Sem<O>, kinded: bool, number: Number) -> Option<Vec<Sem<O>>> {
    match v {
        Sem::Question(Question::Identify(d)) => undescribed(d, kinded, number),
        _ => None,
    }
}

/// Reads a statement of which things fit a description: the members listed
/// before it ("`a` and `b` call `x`"), counted around it ("2 functions call
/// `x`: `a` and `b`"), or none ("nothing calls `x`").
fn extent<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let (description, members) = match c {
        [Sem::Items(members), rest @ ..] => (described(rest)?, members.clone()),
        [Sem::Count(n), rest @ .., Sem::Items(members)]
            if usize::try_from(*n).ok() == Some(members.len()) =>
        {
            (described(rest)?, members.clone())
        }
        _ => (described(c)?, Vec::new()),
    };
    Some(Sem::Extent(Extent {
        description,
        members,
    }))
}

/// Items listed one by one: "`a`", "`a` and `b`".
fn list<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter()
        .map(|s| match s {
            Sem::Item(i) => Some(i.clone()),
            _ => None,
        })
        .collect::<Option<_>>()
        .map(Sem::Items)
}

/// Exactly `n` items, to be listed one by one.
fn unlist<O: Ontology>(v: &Sem<O>, n: usize) -> Option<Vec<Sem<O>>> {
    match v {
        Sem::Items(items) if items.len() == n => {
            Some(items.iter().cloned().map(Sem::Item).collect())
        }
        _ => None,
    }
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    vec![
        // A question, with or without its question mark.
        seed(
            "en.utterance.question_mark",
            Utterance,
            vec![C(Question), W("?")],
            ask,
            unask,
        ),
        seed(
            "en.utterance.question",
            Utterance,
            vec![C(Question)],
            ask,
            unask,
        )
        .read_only(),
        // "what calls `x`": identify things of any kind meeting a condition.
        seed(
            "en.question.what",
            Question,
            vec![W("what"), C(VerbPhrase)],
            identify,
            |v| unidentify(v, false, Number::Singular),
        ),
        // "which functions call `x`": identify things of a kind meeting a condition.
        seed(
            "en.question.which",
            Question,
            vec![W("which"), C(Noun), C(VerbPhrase)],
            identify,
            |v| unidentify(v, true, Number::Plural),
        ),
        // "calls `x`": the subject is the first argument of the verb's relation.
        seed(
            "en.verb_phrase.object",
            VerbPhrase,
            vec![C(Verb), C(NounPhrase)],
            object,
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
                        form: match number {
                            Number::Singular => VerbForm::ThirdSingular,
                            Number::Plural => VerbForm::Plain,
                        },
                    },
                    Sem::Thing(other.clone()),
                ]),
                _ => None,
            },
        ),
        // A thing named by the user.
        seed(
            "en.noun_phrase.name",
            NounPhrase,
            vec![Class(NAME)],
            |c| match c {
                [Sem::Name(n)] => Some(Sem::Thing(Thing::Named(n.clone()))),
                _ => None,
            },
            |v| match v {
                Sem::Thing(Thing::Named(n)) => Some(vec![Sem::Name(n.clone())]),
                _ => None,
            },
        ),
        // A resolved item, as the system speaks of it.
        seed(
            "en.noun_phrase.item",
            NounPhrase,
            vec![Class(ITEM)],
            |c| match c {
                [Sem::Item(i)] => Some(Sem::Thing(Thing::Item(i.clone()))),
                _ => None,
            },
            |v| match v {
                Sem::Thing(Thing::Item(i)) => Some(vec![Sem::Item(i.clone())]),
                _ => None,
            },
        ),
        // "`a` and `b` call `x`."
        seed(
            "en.statement.list",
            Statement,
            vec![C(ItemList), C(VerbPhrase)],
            extent,
            |v| match v {
                Sem::Extent(e) if !e.members.is_empty() => {
                    let number = number_of(e.members.len());
                    let said = undescribed(&e.description, false, number)?;
                    Some([vec![Sem::Items(e.members.clone())], said].concat())
                }
                _ => None,
            },
        ),
        // "2 functions call `x`: `a` and `b`."
        seed(
            "en.statement.counted",
            Statement,
            vec![Class(NUMBER), C(Noun), C(VerbPhrase), W(":"), C(ItemList)],
            extent,
            |v| match v {
                Sem::Extent(e) if !e.members.is_empty() => {
                    let count = Sem::Count(u64::try_from(e.members.len()).ok()?);
                    let number = number_of(e.members.len());
                    let said = undescribed(&e.description, true, number)?;
                    Some([vec![count], said, vec![Sem::Items(e.members.clone())]].concat())
                }
                _ => None,
            },
        ),
        // "nothing calls `x`."
        seed(
            "en.statement.nothing",
            Statement,
            vec![W("nothing"), C(VerbPhrase)],
            extent,
            |v| match v {
                Sem::Extent(e) if e.members.is_empty() => {
                    undescribed(&e.description, false, Number::Singular)
                }
                _ => None,
            },
        ),
        // "no functions call `x`."
        seed(
            "en.statement.no_kind",
            Statement,
            vec![W("no"), C(Noun), C(VerbPhrase)],
            extent,
            |v| match v {
                Sem::Extent(e) if e.members.is_empty() => {
                    undescribed(&e.description, true, Number::Plural)
                }
                _ => None,
            },
        ),
        // Lists of items: "`a`", "`a` and `b`", "`a`, `b` and `c`".
        seed("en.item_list.one", ItemList, vec![Class(ITEM)], list, |v| {
            unlist(v, 1)
        }),
        seed(
            "en.item_list.two",
            ItemList,
            vec![Class(ITEM), W("and"), Class(ITEM)],
            list,
            |v| unlist(v, 2),
        ),
        seed(
            "en.item_list.more",
            ItemList,
            vec![Class(ITEM), W(","), C(ItemList)],
            |c| match c {
                [Sem::Item(first), Sem::Items(rest)] if rest.len() >= 2 => {
                    Some(Sem::Items([vec![first.clone()], rest.clone()].concat()))
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
        // Replies.
        seed(
            "en.reply.extent",
            Reply,
            vec![C(Statement), W(".")],
            |c| match c {
                [Sem::Extent(e)] => Some(Sem::Reply(Reply::Extent(e.clone()))),
                _ => None,
            },
            |v| match v {
                Sem::Reply(Reply::Extent(e)) => Some(vec![Sem::Extent(e.clone())]),
                _ => None,
            },
        ),
        // "By `x` I assume you mean `Foo::x`. ..." or "I take that to mean: ...? ..."
        said(
            "en.reply.assuming",
            Reply,
            vec![C(Assumption), C(Reply)],
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
        said(
            "en.assumption.means",
            Assumption,
            vec![
                W("by"),
                Class(NAME),
                W("I assume you mean"),
                Class(ITEM),
                W("."),
            ],
            |v| match v {
                Sem::Assumption(Assumption::Means { name, item }) => {
                    Some(vec![Sem::Name(name.clone()), Sem::Item(item.clone())])
                }
                _ => None,
            },
        ),
        said(
            "en.assumption.reading",
            Assumption,
            vec![W("I take that to mean :"), C(Utterance)],
            |v| match v {
                Sem::Assumption(Assumption::Reading { meaning }) => {
                    Some(vec![Sem::Utterance(meaning.clone())])
                }
                _ => None,
            },
        ),
        said(
            "en.reply.no_such_name",
            Reply,
            vec![W("nothing is called"), Class(NAME), W(".")],
            |v| match v {
                Sem::Reply(Reply::Trouble(Trouble::NoSuchName { name })) => {
                    Some(vec![Sem::Name(name.clone())])
                }
                _ => None,
            },
        ),
        said(
            "en.reply.wrong_kind",
            Reply,
            vec![Class(ITEM), W("is the wrong kind of thing for that .")],
            |v| match v {
                Sem::Reply(Reply::Trouble(Trouble::WrongKind { item })) => {
                    Some(vec![Sem::Item(item.clone())])
                }
                _ => None,
            },
        ),
        said(
            "en.reply.ambiguous",
            Reply,
            vec![W("I can't tell which one you mean :"), C(ItemList), W(".")],
            |v| match v {
                Sem::Reply(Reply::Trouble(Trouble::Ambiguous { candidates })) => {
                    Some(vec![Sem::Items(candidates.clone())])
                }
                _ => None,
            },
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
