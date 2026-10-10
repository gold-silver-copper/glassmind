//! Asking where a thing is: "where is `x` defined?", "can you tell me
//! where `x` is defined?", "find the definition of `x`", "`x` defined
//! where", and the system's answer, "`x` is defined at src/x.rs:12.".

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Question, Reply, Utterance};

use crate::lexicon::{ITEM, PLACE};
use crate::sem::{Cat, Number, Sem};
use crate::word;

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 2;

/// The question among the children, if it asks where something is.
fn located<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter()
        .find(|v| matches!(v, Sem::Question(Question::Locate { .. })))
        .cloned()
}

/// The question among the children, whatever it asks.
fn question<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find(|v| matches!(v, Sem::Question(_))).cloned()
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

/// Where the thing is, given that it is of the kind among the children.
fn of_kind<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let kind = c.iter().find_map(|v| match v {
        Sem::Kind { kind, .. } => Some(kind.clone()),
        _ => None,
    });
    c.iter().find_map(|v| match v {
        Sem::Question(Question::Locate { thing, kind: None }) => {
            Some(Sem::Question(Question::Locate {
                thing: thing.clone(),
                kind: kind.clone(),
            }))
        }
        _ => None,
    })
}

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let blank = || Sem::Quote(String::new());
    let words = |id, cat, words: &[&'static str]| {
        word(RuleInfo::new(id, ROUND), cat, words, blank()).read_only()
    };
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    vec![
        // "where is `x` defined": how the system asks it, too.
        Rule::new(
            RuleInfo::new("en.question.where_defined", ROUND),
            Cat::Question,
            vec![W("where"), W("is"), C(Cat::Target), C(Cat::Defined)],
            S::Both(located, |v| {
                Some(vec![v.clone(), Sem::Quote(String::new())])
            }),
        ),
        // "where's `x` defined"
        rule(
            "en.question.where_s_defined",
            Cat::Question,
            vec![W("where's"), C(Cat::Target), C(Cat::Defined)],
            located,
        ),
        // "where is `x`", "where's the definition of `x`"
        rule(
            "en.question.where_is",
            Cat::Question,
            vec![W("where"), W("is"), C(Cat::Target)],
            located,
        ),
        rule(
            "en.question.where_s",
            Cat::Question,
            vec![W("where's"), C(Cat::Target)],
            located,
        ),
        // "where does `x` live"
        rule(
            "en.question.where_live",
            Cat::Question,
            vec![W("where"), W("does"), C(Cat::Target), W("live")],
            located,
        ),
        // "where does `x` get defined"
        rule(
            "en.question.where_get_defined",
            Cat::Question,
            vec![
                W("where"),
                W("does"),
                C(Cat::Target),
                W("get"),
                C(Cat::Defined),
            ],
            located,
        ),
        // "`x` defined where", "Sub declared where"
        rule(
            "en.question.defined_where",
            Cat::Question,
            vec![C(Cat::Target), C(Cat::Defined), W("where")],
            located,
        ),
        // "find def of `x`", "point me to `x`", "jump to `x` def"
        rule(
            "en.question.seek",
            Cat::Question,
            vec![C(Cat::Seek), C(Cat::Target)],
            located,
        ),
        // "where can I find the definition of `x`"
        rule(
            "en.question.where_can_i",
            Cat::Question,
            vec![W("where"), W("can"), W("i"), C(Cat::Seek), C(Cat::Target)],
            located,
        ),
        // "loc of `set_theme_color`", "`Cmd` defn location"
        rule(
            "en.question.location_of",
            Cat::Question,
            vec![C(Cat::Location), W("of"), C(Cat::Target)],
            located,
        ),
        rule(
            "en.question.target_location",
            Cat::Question,
            vec![C(Cat::Target), C(Cat::Location)],
            located,
        ),
        // "can you tell me where `x` is defined", "I'd like to know where ..."
        rule(
            "en.question.inquiry",
            Cat::Question,
            vec![C(Cat::Inquiry), C(Cat::Indirect)],
            question,
        ),
        // "where `x` is defined"
        rule(
            "en.indirect.where_defined",
            Cat::Indirect,
            vec![W("where"), C(Cat::Target), W("is"), C(Cat::Defined)],
            located,
        ),
        // Where the user wants it looked for: "where is `x` defined in the
        // code", "where does `x` live in the codebase", "where's `x` defined at".
        rule(
            "en.question.in_the_code",
            Cat::Question,
            vec![C(Cat::Question), W("in"), W("the"), W("code")],
            located,
        ),
        rule(
            "en.question.in_the_codebase",
            Cat::Question,
            vec![C(Cat::Question), W("in"), W("the"), W("codebase")],
            located,
        ),
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
        Rule::new(
            RuleInfo::new("en.utterance.question_stop", ROUND),
            Cat::Utterance,
            vec![C(Cat::Question), W(".")],
            S::Build(|c| match c {
                [Sem::Question(q)] => Some(Sem::Utterance(Utterance::Ask(q.clone()))),
                _ => None,
            }),
        )
        .read_only(),
        // The thing itself, "the `x`", "the `x` struct", "the function
        // `x`", its definition: "the definition of `x`", "`x` def".
        Rule::new(
            RuleInfo::new("en.target.thing", ROUND),
            Cat::Target,
            vec![C(Cat::NounPhrase)],
            S::Both(locate, |v| match v {
                Sem::Question(Question::Locate { thing, kind: None }) => {
                    Some(vec![Sem::Thing(thing.clone())])
                }
                _ => None,
            }),
        ),
        Rule::new(
            RuleInfo::new("en.target.the_kind", ROUND),
            Cat::Target,
            vec![W("the"), C(Cat::Target), C(Cat::Noun)],
            S::Both(of_kind, |v| match v {
                Sem::Question(Question::Locate {
                    thing,
                    kind: Some(kind),
                }) => Some(vec![
                    Sem::Question(Question::Locate {
                        thing: thing.clone(),
                        kind: None,
                    }),
                    Sem::Kind {
                        kind: kind.clone(),
                        number: Number::Singular,
                    },
                ]),
                _ => None,
            }),
        ),
        rule(
            "en.target.kind",
            Cat::Target,
            vec![C(Cat::Target), C(Cat::Noun)],
            of_kind,
        ),
        rule(
            "en.target.the_kind_first",
            Cat::Target,
            vec![W("the"), C(Cat::Noun), C(Cat::Target)],
            of_kind,
        ),
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
        // "Inquiry": a modal question "can you tell me", or the speaker's
        // wish "I'd like to know", "I'm trying to figure out".
        rule(
            "en.inquiry.modal",
            Cat::Inquiry,
            vec![C(Cat::Modal), W("you"), C(Cat::Tell)],
            |_| Some(Sem::Quote(String::new())),
        ),
        rule("en.inquiry.tell", Cat::Inquiry, vec![C(Cat::Tell)], |_| {
            Some(Sem::Quote(String::new()))
        }),
        words(
            "en.inquiry.do_you_know",
            Cat::Inquiry,
            &["do", "you", "know"],
        ),
        words(
            "en.inquiry.like_to_know",
            Cat::Inquiry,
            &["i'd", "like", "to", "know"],
        ),
        words(
            "en.inquiry.figure_out",
            Cat::Inquiry,
            &["i'm", "trying", "to", "figure", "out"],
        ),
        words(
            "en.inquiry.trying_to_find",
            Cat::Inquiry,
            &["i'm", "trying", "to", "find"],
        ),
        words(
            "en.inquiry.looking_for",
            Cat::Inquiry,
            &["i'm", "looking", "for"],
        ),
        words("en.modal.can", Cat::Modal, &["can"]),
        words("en.modal.could", Cat::Modal, &["could"]),
        words("en.tell.tell_me", Cat::Tell, &["tell", "me"]),
        words("en.tell.show_me", Cat::Tell, &["show", "me"]),
        words("en.tell.point_me_to", Cat::Tell, &["point", "me", "to"]),
        words("en.seek.find", Cat::Seek, &["find"]),
        words("en.seek.locate", Cat::Seek, &["locate"]),
        words("en.seek.point_me_to", Cat::Seek, &["point", "me", "to"]),
        words("en.seek.jump_to", Cat::Seek, &["jump", "to"]),
        word(
            RuleInfo::new("en.defined.defined", ROUND),
            Cat::Defined,
            &["defined"],
            blank(),
        ),
        words("en.defined.declared", Cat::Defined, &["declared"]),
        words("en.defined.located", Cat::Defined, &["located"]),
        words("en.definition.definition", Cat::Definition, &["definition"]),
        words("en.definition.def", Cat::Definition, &["def"]),
        words("en.definition.defn", Cat::Definition, &["defn"]),
        words(
            "en.definition.declaration",
            Cat::Definition,
            &["declaration"],
        ),
        words("en.location.location", Cat::Location, &["location"]),
        words("en.location.loc", Cat::Location, &["loc"]),
        // "`x` is defined at src/x.rs:12."
        Rule::new(
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
                |v| match v {
                    Sem::Reply(Reply::Location(item)) => Some(vec![
                        Sem::Item(item.clone()),
                        Sem::Quote(String::new()),
                        Sem::Item(item.clone()),
                    ]),
                    _ => None,
                },
            ),
        ),
    ]
}
