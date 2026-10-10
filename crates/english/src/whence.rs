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

use Sym::{Cat as C, Word as W};

const ROUND: u32 = 9;

/// The words of a question about where from, each meaning nothing by
/// itself.
const WORDS: &[(&str, Cat, &str)] = &[
    // Words asking where, with the auxiliary of a question: "where is `x`
    // called", "where does `x` get used".
    ("en.whence.where", Cat::Whence, "where"),
    ("en.whence.where_is", Cat::Whence, "where is"),
    ("en.whence.where_s", Cat::Whence, "where's"),
    ("en.whence.where_does", Cat::Whence, "where does"),
    // Words before a participle making it passive: "is called", "gets
    // used", "getting called".
    ("en.becoming.is", Cat::Becoming, "is"),
    ("en.becoming.gets", Cat::Becoming, "gets"),
    ("en.becoming.get", Cat::Becoming, "get"),
    ("en.becoming.getting", Cat::Becoming, "getting"),
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
        // "from where `x` gets called", "from where does `x` get used"
        rule(
            "en.whence.from",
            Cat::Whence,
            vec![W("from"), C(Cat::Whence)],
            last,
        ),
        // "called", "gets called", "is getting called"
        rule("en.undergo.verb", Cat::Undergo, vec![C(Cat::Verb)], last),
        rule(
            "en.undergo.auxiliary",
            Cat::Undergo,
            vec![C(Cat::Becoming), C(Cat::Undergo)],
            last,
        ),
        // "`x` gets called"
        rule(
            "en.undergoes.passive",
            Cat::Undergoes,
            vec![C(Cat::NounPhrase), C(Cat::Undergo)],
            undergoes,
        ),
        // "where is `x` called", "where `x` gets called from"
        rule(
            "en.question.whence",
            Cat::Question,
            vec![C(Cat::Whence), C(Cat::Undergoes)],
            |c| identify(c.get(1..)?),
        ),
        rule(
            "en.question.whence_from",
            Cat::Question,
            vec![C(Cat::Whence), C(Cat::Undergoes), W("from")],
            |c| identify(c.get(1..2)?),
        ),
    ]);
    rules
}
