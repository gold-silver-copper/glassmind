//! Asking where a thing is: "where is `x` defined?", "can you tell me
//! where `x` is defined?", "find the definition of `x`", "`x` defined
//! where", and the system's answer, "`x` is defined at src/x.rs:12.".

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Question, Reply, Thing, Utterance};

use crate::blank;
use crate::first;
use crate::lexicon::{ITEM, PLACE};
use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 2;

/// Nothing: the value of words that only shape a sentence.
pub(crate) fn nothing<O: Ontology>(_: &[Sem<O>]) -> Option<Sem<O>> {
    Some(blank())
}

/// Read-only rules reading each fixed phrase of `table` (category, then
/// id and words for each phrase) as a value that means nothing by itself.
pub(crate) fn fillers<O: Ontology>(
    round: u32,
    table: &[(Cat, &[(&'static str, &'static str)])],
) -> Vec<Rule<Cat, Sem<O>>> {
    table
        .iter()
        .flat_map(|&(cat, phrases)| {
            phrases.iter().map(move |&(id, words)| {
                word(RuleInfo::new(id, round), cat, words, blank()).read_only()
            })
        })
        .collect()
}

/// The question among the children, if it asks where something is.
fn located<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter()
        .find(|v| matches!(v, Sem::Question(Question::Locate { .. })))
        .cloned()
}

/// Where the thing among the children is.
fn locate<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find_map(|v| match v {
        Sem::Thing(thing) => Some(Sem::Question(Question::Locate {
            thing: thing.clone(),
            kind: None,
        })),
        _ => None,
    })
}

/// Words that only shape a question about where a thing is, by category.
const FILLERS: &[(Cat, &[(&str, &str)])] = &[
    // The speaker's wish to know: "I'd like to know", "I'm trying to
    // figure out".
    (
        Cat::Inquiry,
        &[
            ("en.inquiry.do_you_know", "do you know"),
            ("en.inquiry.like_to_know", "i'd like to know"),
            ("en.inquiry.figure_out", "i'm trying to figure out"),
            ("en.inquiry.trying_to_find", "i'm trying to find"),
            ("en.inquiry.looking_for", "i'm looking for"),
        ],
    ),
    (
        Cat::Modal,
        &[("en.modal.can", "can"), ("en.modal.could", "could")],
    ),
    (
        Cat::Tell,
        &[
            ("en.tell.tell_me", "tell me"),
            ("en.tell.show_me", "show me"),
            ("en.tell.point_me_to", "point me to"),
        ],
    ),
    (
        Cat::Seek,
        &[
            ("en.seek.find", "find"),
            ("en.seek.locate", "locate"),
            ("en.seek.point_me_to", "point me to"),
            ("en.seek.jump_to", "jump to"),
        ],
    ),
    (Cat::Defined, &[("en.defined.declared", "declared")]),
    (
        Cat::Definition,
        &[
            ("en.definition.definition", "definition"),
            ("en.definition.def", "def"),
            ("en.definition.defn", "defn"),
        ],
    ),
    (
        Cat::Location,
        &[
            ("en.location.location", "location"),
            ("en.location.loc", "loc"),
        ],
    ),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    // "where is `x` defined": how the system asks it, too.
    let mut rules = vec![Rule::new(
        RuleInfo::new("en.question.where_defined", ROUND),
        Cat::Question,
        vec![W("where"), W("is"), C(Cat::Target), C(Cat::Defined)],
        S::Both(located, |v| Some(vec![v.clone(), blank()])),
    )];
    rules.extend(
        [
            // "where's `x` defined"
            (
                "en.question.where_s_defined",
                vec![W("where's"), C(Cat::Target), C(Cat::Defined)],
            ),
            // "where is `x`", "where's the definition of `x`"
            (
                "en.question.where_is",
                vec![W("where"), W("is"), C(Cat::Target)],
            ),
            ("en.question.where_s", vec![W("where's"), C(Cat::Target)]),
            // "where does `x` live"
            (
                "en.question.where_live",
                vec![W("where"), W("does"), C(Cat::Target), W("live")],
            ),
            // "where does `x` get defined"
            (
                "en.question.where_get_defined",
                vec![
                    W("where"),
                    W("does"),
                    C(Cat::Target),
                    W("get"),
                    C(Cat::Defined),
                ],
            ),
            // "`x` defined where", "Sub declared where"
            (
                "en.question.defined_where",
                vec![C(Cat::Target), C(Cat::Defined), W("where")],
            ),
            // "find def of `x`", "point me to `x`", "jump to `x` def"
            ("en.question.seek", vec![C(Cat::Seek), C(Cat::Target)]),
            // "where can I find the definition of `x`"
            (
                "en.question.where_can_i",
                vec![W("where"), W("can"), W("i"), C(Cat::Seek), C(Cat::Target)],
            ),
            // "loc of `set_theme_color`", "`Cmd` defn location"
            (
                "en.question.location_of",
                vec![C(Cat::Location), W("of"), C(Cat::Target)],
            ),
            (
                "en.question.target_location",
                vec![C(Cat::Target), C(Cat::Location)],
            ),
        ]
        .map(|(id, rhs)| rule(id, Cat::Question, rhs, located)),
    );
    rules.extend([
        // "can you tell me where `x` is defined", "I'd like to know where ..."
        rule(
            "en.question.inquiry",
            Cat::Question,
            vec![C(Cat::Inquiry), C(Cat::Indirect)],
            located,
        ),
        // "where `x` is defined"
        rule(
            "en.indirect.where_defined",
            Cat::Indirect,
            vec![W("where"), C(Cat::Target), W("is"), C(Cat::Defined)],
            located,
        ),
    ]);
    // Where the user wants it looked for: "where is `x` defined in the
    // code", "where does `x` live in the codebase", "what is `x` used for in
    // the code".
    rules.extend(
        [
            ("en.question.in_the_code", "code"),
            ("en.question.in_the_codebase", "codebase"),
        ]
        .map(|(id, place)| {
            rule(
                id,
                Cat::Question,
                vec![C(Cat::Question), W("in"), W("the"), W(place)],
                first,
            )
        }),
    );
    rules.extend([
        // "where's it live", "where's that live"
        Rule::new(
            RuleInfo::new("en.question.where_s_live", 9),
            Cat::Question,
            vec![W("where's"), C(Cat::Target), W("live")],
            S::Build(located),
        )
        .read_only(),
        // "where `parse_remote` lives", "where it lives": where the thing
        // is, as a clause.
        Rule::new(
            RuleInfo::new("en.indirect.where_lives", 10),
            Cat::Indirect,
            vec![W("where"), C(Cat::Target), W("lives")],
            S::Build(located),
        )
        .read_only(),
        // "where it's defined": where the thing said before is.
        Rule::new(
            RuleInfo::new("en.indirect.where_it_s_defined", 9),
            Cat::Indirect,
            vec![W("where"), W("it's"), C(Cat::Defined)],
            S::Build(|_| {
                Some(Sem::Question(Question::Locate {
                    thing: Thing::Previous,
                    kind: None,
                }))
            }),
        )
        .read_only(),
        // "`LookFelt` - where defined": where the thing just set off is,
        // with nothing more said of it.
        Rule::new(
            RuleInfo::new("en.question.where_defined_bare", 10),
            Cat::Question,
            vec![W("where"), C(Cat::Defined)],
            S::Build(|_| {
                Some(Sem::Question(Question::Locate {
                    thing: Thing::Previous,
                    kind: None,
                }))
            }),
        )
        .read_only(),
        // "and where it is defined?": asked in the order of a clause.
        Rule::new(
            RuleInfo::new("en.question.indirect", 9),
            Cat::Question,
            vec![C(Cat::Indirect)],
            S::Build(located),
        )
        .read_only(),
        // "where's `x` defined at"
        rule(
            "en.question.at",
            Cat::Question,
            vec![C(Cat::Question), W("at")],
            located,
        ),
        // "where is `x` defined, please" is read by `en.question.please`
        // in request.rs.
        // A request for information that ends like a statement: "I'd like
        // to know where `x` is defined."
        rule(
            "en.utterance.question_stop",
            Cat::Utterance,
            vec![C(Cat::Question), W(".")],
            |c| match c {
                [Sem::Question(q)] => Some(Sem::Utterance(Utterance::Ask(q.clone()))),
                _ => None,
            },
        ),
        // The thing itself, as mentioned: "the `x`", "the `x` struct", "the
        // function `x`"; or its definition: "the definition of `x`", "`x` def".
        Rule::new(
            RuleInfo::new("en.target.mention", 3),
            Cat::Target,
            vec![C(Cat::Mention)],
            S::Both(
                |c| match c {
                    [Sem::Mention { thing, kind }] => Some(Sem::Question(Question::Locate {
                        thing: thing.clone(),
                        kind: kind.clone(),
                    })),
                    _ => None,
                },
                |v| match v {
                    Sem::Question(Question::Locate { thing, kind }) => Some(vec![Sem::Mention {
                        thing: thing.clone(),
                        kind: kind.clone(),
                    }]),
                    _ => None,
                },
            ),
        ),
        // "the definition of `x`"
        rule(
            "en.target.the",
            Cat::Target,
            vec![W("the"), C(Cat::Target)],
            located,
        ),
        rule(
            "en.target.definition_of",
            Cat::Target,
            vec![C(Cat::Definition), W("of"), C(Cat::NounPhrase)],
            locate,
        ),
        rule(
            "en.target.definition",
            Cat::Target,
            vec![C(Cat::NounPhrase), C(Cat::Definition)],
            locate,
        ),
        // "Inquiry": a modal question "can you tell me", or a bare "tell
        // me"; the speaker's wish ("I'd like to know") is in `FILLERS`.
        rule(
            "en.inquiry.modal",
            Cat::Inquiry,
            vec![C(Cat::Modal), W("you"), C(Cat::Tell)],
            nothing,
        ),
        rule("en.inquiry.tell", Cat::Inquiry, vec![C(Cat::Tell)], nothing),
        // "defined": how the system says it, too.
        word(
            RuleInfo::new("en.defined.defined", ROUND),
            Cat::Defined,
            "defined",
            blank(),
        ),
    ]);
    rules.extend(fillers(ROUND, FILLERS));
    // The same wish said without a subject: "trying to find where `x` is
    // defined".
    rules.extend(fillers(
        10,
        &[(
            Cat::Inquiry,
            &[
                ("en.inquiry.bare_trying_to_find", "trying to find"),
                ("en.inquiry.bare_figure_out", "trying to figure out"),
            ],
        )],
    ));
    // "`x` is defined at src/x.rs:12."
    rules.push(Rule::new(
        RuleInfo::new("en.reply.location", ROUND),
        Cat::Reply,
        vec![
            Class(ITEM),
            W("is"),
            C(Cat::Defined),
            W("at"),
            Class(PLACE),
            W("."),
        ],
        S::Both(
            |_| None,
            |v: &Sem<O>| match v {
                Sem::Reply(Reply::Location(item)) => Some(vec![
                    Sem::Item(item.clone()),
                    blank(),
                    Sem::Item(item.clone()),
                ]),
                _ => None,
            },
        ),
    ));
    rules
}
