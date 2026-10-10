//! Asking how many things fit a description: "how many functions call
//! `x`?", "number of callers of `x`", "how many callers does `x` have",
//! "`x` caller count", and saying the answer: "3 functions call `x`."

use glassmind_grammar::{Rule, RuleInfo, Sym};
use glassmind_meaning::{
    Condition, Description, Ontology, Position, Question, Reply, Thing, Utterance,
};

use crate::aside::filler;
use crate::asking::read_only_rule;
use crate::asking::role_of;
use crate::blank;
use crate::first;
use crate::having::anything;
use crate::last;
use crate::lexicon::NUMBER;
use crate::locate::fillers;
use crate::sem::{Cat, Number, Sem, VerbForm};
use crate::spoken;
use crate::word;

use Cat::*;
use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 3;

/// Asks how many things the children describe: a description, a kind and
/// a condition, a role and the other argument, or a count question as it is.
/// Words that mean nothing by themselves ("how many", "number") are skipped.
fn count<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let c: Vec<_> = c.iter().filter(|v| **v != blank()).cloned().collect();
    let description = match c.as_slice() {
        [Sem::Description(d)] | [Sem::Question(Question::Count(d))] => d.clone(),
        [Sem::Kind { kind, .. }, Sem::Condition { condition, .. }]
        | [Sem::Condition { condition, .. }, Sem::Kind { kind, .. }] => Description {
            kind: Some(kind.clone()),
            conditions: vec![condition.clone()],
        },
        [role @ Sem::Role { .. }, other]
        | [other, role @ Sem::Role { .. }, ..]
        | [other, .., role @ Sem::Role { .. }] => match role_of(role, other)? {
            Sem::Description(d) => d,
            _ => return None,
        },
        _ => return None,
    };
    Some(Sem::Question(Question::Count(description)))
}

/// The kind and the condition on the first argument of a description of
/// things of a kind, as a verb phrase speaks it, agreeing with `number`.
fn kind_and_condition<O: Ontology>(d: &Description<O>, number: Number) -> Option<Vec<Sem<O>>> {
    match d.conditions.as_slice() {
        [
            condition @ Condition::Related {
                position: Position::First,
                ..
            },
        ] => Some(vec![
            Sem::Kind {
                kind: d.kind.clone()?,
                number,
            },
            Sem::Condition {
                condition: condition.clone(),
                number,
            },
        ]),
        _ => None,
    }
}

/// The description among the children (a noun and what restricts it), if
/// it describes things to count.
fn described<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match count(c)? {
        Sem::Question(Question::Count(d)) => Some(Sem::Description(d)),
        _ => None,
    }
}

/// The request before, asked again as how many things it asks for: of the
/// kind said after, if one is ("is there many functions that call `x`?
/// like how many exactly", "is there anything calling `x`? how many
/// functions", "how many fns call `x`, can you count them?").
fn recount<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let [
        Sem::Utterance(Utterance::Ask(Question::Identify(d) | Question::Count(d))),
        said,
    ] = c
    else {
        return None;
    };
    let mut d = d.clone();
    if let Sem::Kind { kind, .. } = said {
        if d.kind.as_ref().is_some_and(|k| k != kind) {
            return None;
        }
        d.kind = Some(kind.clone());
    }
    // "is it used in lot of places?": what it undergoes must be what the things
    // asked about do to the thing they were asked about.
    if let Sem::Condition { condition, .. } = said {
        d.conditions
            .iter()
            .find(|c| c.map_things(&mut |_| Thing::Previous) == *condition)?;
    }
    Some(Sem::Utterance(Utterance::Ask(Question::Count(d))))
}

/// Words before a participle that keep its meaning, and the round each
/// came in: "are calling `x`", "end up calling `x`", "ends up calling `x`",
/// "anything that ends up calling it".
const ASPECTS: &[(&str, u32, &str)] = &[
    ("en.verb_phrase.are_participial", ROUND, "are"),
    ("en.verb_phrase.is_participial", ROUND, "is"),
    ("en.verb_phrase.end_up", ROUND, "end up"),
    ("en.verb_phrase.ends_up", 9, "ends up"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| read_only_rule(id, ROUND, lhs, rhs, build);
    let mut rules = vec![
        // "how many functions call `x`": how the system asks it, too.
        spoken(
            "en.question.how_many",
            ROUND,
            Question,
            vec![C(HowMany), C(Noun), C(VerbPhrase)],
            count,
            |v| match v {
                Sem::Question(Question::Count(d)) => {
                    Some([vec![blank()], kind_and_condition(d, Number::Plural)?].concat())
                }
                _ => None,
            },
        ),
        // "how many callers of `x`", "how many fns calling `x`"
        rule(
            "en.question.how_many_described",
            Question,
            vec![C(HowMany), C(Description)],
            count,
        ),
        // "how many places call `x`", "how many things call it": things of
        // any kind.
        read_only_rule(
            "en.question.how_many_ones",
            11,
            Question,
            vec![C(HowMany), C(Ones), C(VerbPhrase)],
            |c| count(&[anything(c)?]),
        ),
        // "how many call it", "how many are calling it in total?": things
        // of any kind, with the noun left out.
        read_only_rule(
            "en.question.how_many_bare",
            27,
            Question,
            vec![C(HowMany), C(VerbPhrase)],
            |c| count(&[anything(c)?]),
        ),
        // "how many callers `x`", with nothing between the role and the thing.
        read_only_rule(
            "en.question.how_many_role",
            11,
            Question,
            vec![C(HowMany), C(RoleNoun), C(NounPhrase)],
            count,
        ),
        // "how many callers does `x` have"
        rule(
            "en.question.how_many_have",
            Question,
            vec![C(HowMany), C(RoleNoun), W("does"), C(NounPhrase), W("have")],
            count,
        ),
        // "`x` — how many callers"
        rule(
            "en.question.topic_how_many",
            Question,
            vec![C(NounPhrase), C(Dash), C(HowMany), C(RoleNoun)],
            count,
        ),
        // "number of callers of `x`", "count of fns calling `x`", "# of callers for `x`"
        rule(
            "en.question.amount_of",
            Question,
            vec![C(Amount), W("of"), C(Description)],
            count,
        ),
        // "`x` caller count"
        rule(
            "en.question.role_amount",
            Question,
            vec![C(NounPhrase), C(RoleNoun), C(Amount)],
            count,
        ),
        // "count callers of `x`", "can you count how many functions call `x`"
        rule(
            "en.question.count_described",
            Question,
            vec![W("count"), C(Description)],
            count,
        ),
        // A count question as it is.
        rule(
            "en.question.count_how_many",
            Question,
            vec![W("count"), C(Question)],
            count,
        ),
        // "fns calling `x`", "funcs that call `x`"
        rule(
            "en.description.participial",
            Description,
            vec![C(Noun), C(Participial)],
            described,
        ),
        rule(
            "en.description.that_verb_phrase",
            Description,
            vec![C(Noun), W("that"), C(VerbPhrase)],
            described,
        ),
        // "calling `x`"
        rule(
            "en.participial.object",
            Participial,
            vec![C(Verb), C(NounPhrase)],
            |c: &[Sem<O>]| match c {
                [
                    Sem::Relation {
                        relation,
                        form: VerbForm::Progressive,
                    },
                    Sem::Thing(other),
                ] => Some(Sem::Condition {
                    condition: Condition::Related {
                        relation: relation.clone(),
                        position: Position::First,
                        other: other.clone(),
                    },
                    number: Number::Plural,
                }),
                _ => None,
            },
        ),
        // "how many": how the system says it, too.
        word(
            RuleInfo::new("en.how_many", 18),
            HowMany,
            "how many",
            blank(),
        ),
        // "number": how the system says it, too.
        word(
            RuleInfo::new("en.amount.number", ROUND),
            Amount,
            "number",
            blank(),
        ),
    ];
    // Nouns for how many things there are, and dashes setting a topic
    // before a question about it.
    rules.extend(fillers(
        ROUND,
        &[
            (
                Amount,
                &[("en.amount.count", "count"), ("en.amount.hash", "#")],
            ),
            (Dash, &[("en.dash.em", "—"), ("en.dash.hyphen", "-")]),
        ],
    ));
    rules.extend([
        // "3 functions call `x`.", "1 function calls `x`."
        spoken(
            "en.reply.count",
            ROUND,
            Reply,
            vec![Class(NUMBER), C(Noun), C(VerbPhrase), W(".")],
            |_| None,
            |v| match v {
                Sem::Reply(Reply::Count { description, count }) => {
                    let number = if *count == 1 {
                        Number::Singular
                    } else {
                        Number::Plural
                    };
                    Some(
                        [
                            vec![Sem::Count(*count)],
                            kind_and_condition(description, number)?,
                        ]
                        .concat(),
                    )
                }
                _ => None,
            },
        ),
        // "the number is 3.", when the things counted have no kind.
        spoken(
            "en.reply.count_only",
            ROUND,
            Reply,
            vec![W("the"), C(Amount), W("is"), Class(NUMBER), W(".")],
            |_| None,
            |v| match v {
                Sem::Reply(Reply::Count { count, .. }) => Some(vec![blank(), Sem::Count(*count)]),
                _ => None,
            },
        ),
    ]);
    // Asking again how many of the things just asked about there are.
    let again = |id, round, rhs, build| read_only_rule(id, round, Recount, rhs, build);
    rules.extend([
        read_only_rule(
            "en.utterance.recount",
            11,
            Utterance,
            vec![C(Utterance), C(Recount)],
            recount,
        ),
        // "how many functions": of that kind.
        again("en.recount.kind", 11, vec![C(Recount), C(Noun)], last),
        // "fn count only": a count of things of that kind; "or exact number
        // if you can": the count itself.
        again(
            "en.recount.kind_amount",
            14,
            vec![C(Noun), C(Amount)],
            first,
        ),
        again(
            "en.recount.exact_amount",
            14,
            vec![W("exact"), C(Amount)],
            |_| Some(blank()),
        ),
        // ", can you count them", "like how many"
        again("en.recount.prompt", 11, vec![C(Prompt), C(Recount)], last),
        again("en.recount.joined", 11, vec![C(Join), C(Recount)], last),
        // "how many functions call it, roughly? or exactly if you can": how
        // close a count the speaker wants (not just any hedge: "like even if it's wrapped" asks no
        // count).
        again(
            "en.recount.or_exactly",
            27,
            vec![C(Join), W("exactly")],
            |_| Some(blank()),
        ),
        // "how many exactly", "count them for me?"
        again("en.recount.hedged", 11, vec![C(Recount), C(Hedge)], first),
        // "how many functions call `x`, is it used in lot of places?":
        // how widely it is used asks how many things use it.
        again(
            "en.recount.undergoes_widely",
            15,
            vec![W("is"), C(Undergoes), C(Widely)],
            first,
        ),
    ]);
    rules.extend(fillers(
        11,
        &[
            (
                Recount,
                &[
                    ("en.recount.count_them", "count them"),
                    ("en.recount.how_many", "how many"),
                ],
            ),
            // "num of funcs calling `x`"
            (Amount, &[("en.amount.num", "num")]),
        ],
    ));
    // "is it used in lot of places", with the article dropped
    rules.push(filler(
        "en.widely.in_lot_of_places",
        15,
        Widely,
        "in lot of places",
    ));
    // "`refilter` called from how many fns", "`x` called by how many fns"
    rules.extend(
        [
            ("en.question.undergoes_from_how_many", 11, "from"),
            ("en.question.undergoes_by_how_many", 14, "by"),
        ]
        .map(|(id, round, agent)| {
            read_only_rule(
                id,
                round,
                Question,
                vec![C(Undergoes), W(agent), C(HowMany), C(Noun)],
                count,
            )
        }),
    );
    // "how many callers `x` has", "`x` how many callers it got", "`x`, how
    // many callers it have": the question in the order of a statement.
    rules.extend(
        [
            ("en.question.how_many_role_has", 14, "has"),
            ("en.question.how_many_role_have", 14, "have"),
            ("en.question.how_many_role_got", 14, "got"),
        ]
        .map(|(id, round, verb)| {
            read_only_rule(
                id,
                round,
                Question,
                vec![C(HowMany), C(RoleNoun), C(NounPhrase), W(verb)],
                count,
            )
        }),
    );
    rules.extend(ASPECTS.iter().map(|&(id, round, before)| {
        read_only_rule(
            id,
            round,
            VerbPhrase,
            vec![W(before), C(Participial)],
            first,
        )
    }));
    rules
}
