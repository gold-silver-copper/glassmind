//! Asking where a thing undergoes a relation from: "where is `x` called
//! from?", "where `x` gets used", "where's `x` getting called from", "from
//! where `x` gets called". The places asked for are the things on the
//! other side of the relation, so it asks what calls `x`. The system never
//! says one.

use glassmind_grammar::{Rule, Sym};
use glassmind_meaning::{Ontology, Position};

use crate::aside::filler;
use crate::asking::{identify, read_only_rule, related};
use crate::last;
use crate::sem::{Cat, Number, Sem, VerbForm};

use Cat::*;
use Sym::{Cat as C, Word as W};

const ROUND: u32 = 9;

/// The words of a question about where from, each meaning nothing by
/// itself.
const WORDS: &[(&str, Cat, &str)] = &[
    // Words asking where, with the auxiliary of a question: "where does
    // `x` get used" ("where is" is in [`crate::locate`]).
    ("en.whence.where", Where, "where"),
    ("en.whence.where_s", Where, "where's"),
    ("en.whence.where_does", Where, "where does"),
    // Words before a participle making it passive: "is called", "gets
    // used", "getting called".
    ("en.becoming.is", Becoming, "is"),
    ("en.becoming.gets", Becoming, "gets"),
    ("en.becoming.get", Becoming, "get"),
    ("en.becoming.getting", Becoming, "getting"),
];

/// "`x` gets called": the things asked about stand first in the relation,
/// with `x` second.
fn undergoes<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [
            Sem::Thing(other),
            Sem::Relation {
                relation,
                form: VerbForm::Participle,
            },
        ] => Some(Sem::Condition {
            condition: related(relation, Position::First, other),
            number: Number::Plural,
        }),
        _ => None,
    }
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| read_only_rule(id, ROUND, lhs, rhs, build);
    let mut rules: Vec<_> = WORDS
        .iter()
        .map(|&(id, cat, words)| filler(id, ROUND, cat, words))
        .collect();
    rules.extend([
        // "called", "gets called", "is getting called"
        rule("en.undergo.verb", Undergo, vec![C(Verb)], last),
        rule(
            "en.undergo.auxiliary",
            Undergo,
            vec![C(Becoming), C(Undergo)],
            last,
        ),
        // "`x` gets called"
        rule(
            "en.undergoes.passive",
            Undergoes,
            vec![C(NounPhrase), C(Undergo)],
            undergoes,
        ),
        // "where is `x` called", "where `x` gets called from"
        rule(
            "en.question.whence",
            Question,
            vec![C(Where), C(Undergoes)],
            |c| identify(c.get(1..)?),
        ),
        rule(
            "en.question.whence_from",
            Question,
            vec![C(Where), C(Undergoes), W("from")],
            |c| identify(c.get(1..2)?),
        ),
        // "`x` called from where", "`x` gets used where": the question
        // word in the place it asks about.
        read_only_rule(
            "en.question.whence_echo",
            22,
            Question,
            vec![C(Undergoes), C(Where)],
            |c| identify(c.get(..1)?),
        ),
    ]);
    // "`x` called by what fns", "`press` called by whcih functions": the
    // doer asked for in its place, with its kind.
    for (id, which) in [
        ("en.question.undergoes_by_which", "which"),
        ("en.question.undergoes_by_what", "what"),
    ] {
        rules.push(read_only_rule(
            id,
            22,
            Question,
            vec![C(Undergoes), W("by"), W(which), C(Noun)],
            |c| identify(&[c.last()?.clone(), c.first()?.clone()]),
        ));
    }
    rules
}
