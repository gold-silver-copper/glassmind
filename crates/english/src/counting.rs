//! Asking how many things fit a description: "how many functions call
//! `x`?", "number of callers of `x`", "how many callers does `x` have",
//! "`x` caller count", and saying the answer: "3 functions call `x`."

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{
    Condition, Description, Ontology, Position, Question, Reply, Thing, Utterance,
};

use crate::asking::role_of;
use crate::blank;
use crate::first;
use crate::having::anything;
use crate::last;
use crate::lexicon::NUMBER;
use crate::locate::fillers;
use crate::phrase;
use crate::sem::{Cat, Number, Sem, VerbForm};
use crate::word;

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 3;

/// Asks how many things the children describe: a description, a kind and
/// a condition, a role and the other argument, or a count question as it is.
fn count<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let description = match c {
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
    // "is it used a lot?": what it undergoes must be what the things
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
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let mut rules = vec![
        // "how many functions call `x`": how the system asks it, too.
        Rule::new(
            RuleInfo::new("en.question.how_many", ROUND),
            Cat::Question,
            vec![W("how"), W("many"), C(Cat::Noun), C(Cat::VerbPhrase)],
            S::Both(count, |v| match v {
                Sem::Question(Question::Count(d)) => kind_and_condition(d, Number::Plural),
                _ => None,
            }),
        ),
        // "how many callers of `x`", "how many fns calling `x`"
        rule(
            "en.question.how_many_described",
            Cat::Question,
            vec![W("how"), W("many"), C(Cat::Description)],
            count,
        ),
        // "how many places call `x`", "how many things call it": things of
        // any kind.
        Rule::new(
            RuleInfo::new("en.question.how_many_ones", 11),
            Cat::Question,
            vec![W("how"), W("many"), C(Cat::Ones), C(Cat::VerbPhrase)],
            S::Build(|c| count(&[anything(c)?])),
        )
        .read_only(),
        // "how many callers `x`", with nothing between the role and the thing.
        Rule::new(
            RuleInfo::new("en.question.how_many_role", 11),
            Cat::Question,
            vec![W("how"), W("many"), C(Cat::RoleNoun), C(Cat::NounPhrase)],
            S::Build(count),
        )
        .read_only(),
        // "how many callers does `x` have"
        rule(
            "en.question.how_many_have",
            Cat::Question,
            vec![
                W("how"),
                W("many"),
                C(Cat::RoleNoun),
                W("does"),
                C(Cat::NounPhrase),
                W("have"),
            ],
            count,
        ),
        // "`x` — how many callers"
        rule(
            "en.question.topic_how_many",
            Cat::Question,
            vec![
                C(Cat::NounPhrase),
                C(Cat::Dash),
                W("how"),
                W("many"),
                C(Cat::RoleNoun),
            ],
            count,
        ),
        // "number of callers of `x`", "count of fns calling `x`", "# of callers for `x`"
        rule(
            "en.question.amount_of",
            Cat::Question,
            vec![C(Cat::Amount), W("of"), C(Cat::Description)],
            |c| count(c.get(1..)?),
        ),
        // "`x` caller count"
        rule(
            "en.question.role_amount",
            Cat::Question,
            vec![C(Cat::NounPhrase), C(Cat::RoleNoun), C(Cat::Amount)],
            count,
        ),
        // "count callers of `x`", "can you count how many functions call `x`"
        rule(
            "en.question.count_described",
            Cat::Question,
            vec![W("count"), C(Cat::Description)],
            count,
        ),
        // A count question as it is.
        rule(
            "en.question.count_how_many",
            Cat::Question,
            vec![W("count"), C(Cat::Question)],
            count,
        ),
        // "can you count how many functions call `x` for me?"
        rule(
            "en.question.for_me",
            Cat::Question,
            vec![C(Cat::Question), W("for"), W("me")],
            first,
        ),
        // "fns calling `x`", "funcs that call `x`"
        rule(
            "en.description.participial",
            Cat::Description,
            vec![C(Cat::Noun), C(Cat::Participial)],
            described,
        ),
        rule(
            "en.description.that_verb_phrase",
            Cat::Description,
            vec![C(Cat::Noun), W("that"), C(Cat::VerbPhrase)],
            described,
        ),
        // "calling `x`"
        rule(
            "en.participial.object",
            Cat::Participial,
            vec![C(Cat::Verb), C(Cat::NounPhrase)],
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
        // "number": how the system says it, too.
        word(
            RuleInfo::new("en.amount.number", ROUND),
            Cat::Amount,
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
                Cat::Amount,
                &[("en.amount.count", "count"), ("en.amount.hash", "#")],
            ),
            (Cat::Dash, &[("en.dash.em", "—"), ("en.dash.hyphen", "-")]),
        ],
    ));
    rules.extend([
        // "3 functions call `x`.", "1 function calls `x`."
        Rule::new(
            RuleInfo::new("en.reply.count", ROUND),
            Cat::Reply,
            vec![Class(NUMBER), C(Cat::Noun), C(Cat::VerbPhrase), W(".")],
            S::Both(
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
        ),
        // "the number is 3.", when the things counted have no kind.
        Rule::new(
            RuleInfo::new("en.reply.count_only", ROUND),
            Cat::Reply,
            vec![W("the"), C(Cat::Amount), W("is"), Class(NUMBER), W(".")],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Count { count, .. }) => {
                        Some(vec![blank(), Sem::Count(*count)])
                    }
                    _ => None,
                },
            ),
        ),
    ]);
    // Asking again how many of the things just asked about there are.
    let again = |id, round, rhs, build| {
        Rule::new(RuleInfo::new(id, round), Cat::Recount, rhs, S::Build(build)).read_only()
    };
    rules.extend([
        Rule::new(
            RuleInfo::new("en.utterance.recount", 11),
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Recount)],
            S::Build(recount),
        )
        .read_only(),
        // "how many functions": of that kind.
        again(
            "en.recount.kind",
            11,
            vec![C(Cat::Recount), C(Cat::Noun)],
            last,
        ),
        // "fn count only": a count of things of that kind; "or exact number
        // if you can": the count itself.
        again(
            "en.recount.kind_amount",
            14,
            vec![C(Cat::Noun), C(Cat::Amount)],
            first,
        ),
        again(
            "en.recount.exact_amount",
            14,
            vec![W("exact"), C(Cat::Amount)],
            |_| Some(blank()),
        ),
        // ", can you count them", "like how many"
        again(
            "en.recount.prompt",
            11,
            vec![C(Cat::Prompt), C(Cat::Recount)],
            last,
        ),
        again(
            "en.recount.joined",
            11,
            vec![C(Cat::Join), C(Cat::Recount)],
            last,
        ),
        // "how many exactly", "count them for me?"
        again(
            "en.recount.hedged",
            11,
            vec![C(Cat::Recount), C(Cat::Hedge)],
            first,
        ),
        again(
            "en.recount.for_me",
            11,
            vec![C(Cat::Recount), W("for"), W("me")],
            first,
        ),
        again("en.recount.mark", 11, vec![C(Cat::Recount), W("?")], first),
        // "how many functions call `x`, is it used in a lot of places?":
        // how widely it is used asks how many things use it.
        again(
            "en.recount.undergoes_widely",
            15,
            vec![W("is"), C(Cat::Undergoes), C(Cat::Widely)],
            first,
        ),
        // "..., roughly is it used a lot?"
        again(
            "en.recount.hedged_before",
            15,
            vec![C(Cat::Hedge), C(Cat::Recount)],
            last,
        ),
    ]);
    rules.extend(fillers(
        11,
        &[
            (
                Cat::Recount,
                &[
                    ("en.recount.count_them", "count them"),
                    ("en.recount.how_many", "how many"),
                ],
            ),
            // "num of funcs calling `x`"
            (Cat::Amount, &[("en.amount.num", "num")]),
        ],
    ));
    rules.extend(fillers(
        15,
        &[(
            Cat::Widely,
            &[
                ("en.widely.a_lot", "a lot"),
                ("en.widely.in_a_lot_of_places", "in a lot of places"),
                // with the article dropped
                ("en.widely.in_lot_of_places", "in lot of places"),
            ],
        )],
    ));
    // "`refilter` called from how many fns", "`x` called by how many fns"
    rules.extend(
        [
            ("en.question.undergoes_from_how_many", 11, "from"),
            ("en.question.undergoes_by_how_many", 14, "by"),
        ]
        .map(|(id, round, agent)| {
            Rule::new(
                RuleInfo::new(id, round),
                Cat::Question,
                vec![
                    C(Cat::Undergoes),
                    W(agent),
                    W("how"),
                    W("many"),
                    C(Cat::Noun),
                ],
                S::Build(count),
            )
            .read_only()
        }),
    );
    // "how many callers `x` has", "`x` how many callers it got", "`x`, how
    // many callers it have": the question in the order of a statement.
    rules.extend(
        [
            ("en.question.how_many_role_has", "has"),
            ("en.question.how_many_role_have", "have"),
            ("en.question.how_many_role_got", "got"),
        ]
        .map(|(id, verb)| {
            Rule::new(
                RuleInfo::new(id, 14),
                Cat::Question,
                vec![
                    W("how"),
                    W("many"),
                    C(Cat::RoleNoun),
                    C(Cat::NounPhrase),
                    W(verb),
                ],
                S::Build(count),
            )
            .read_only()
        }),
    );
    rules.extend(ASPECTS.iter().map(|&(id, round, before)| {
        let mut rhs = phrase(before);
        rhs.push(C(Cat::Participial));
        Rule::new(
            RuleInfo::new(id, round),
            Cat::VerbPhrase,
            rhs,
            S::Build(first),
        )
        .read_only()
    }));
    rules
}
