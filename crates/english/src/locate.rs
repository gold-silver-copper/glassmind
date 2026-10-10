//! Asking where a thing is: "where is `x` defined?", "can you tell me
//! where `x` is defined?", "find the definition of `x`", "`x` defined
//! where", and the system's answer, "`x` is defined at src/x.rs:12.".

use glassmind_grammar::{Rule, RuleInfo, Sym};
use glassmind_meaning::{Ontology, Question, Reply, Thing, Utterance};

use crate::aside::filler;
use crate::asking::read_only_rule;
use crate::blank;
use crate::first;
use crate::lexicon::{ITEM, PLACE};
use crate::mention::related;
use crate::sem::{Cat, Sem};
use crate::spoken;
use crate::word;

use Cat::*;
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
            phrases
                .iter()
                .map(move |&(id, words)| filler(id, round, cat, words))
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

/// Where the thing said before is.
fn previous<O: Ontology>(_: &[Sem<O>]) -> Option<Sem<O>> {
    Some(Sem::Question(Question::Locate {
        thing: Thing::Previous,
        kind: None,
    }))
}

/// Where the thing among the children is, of those in the relation to
/// the other thing after it.
fn placed<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [thing, .., relation, other] => {
            match related(&[thing.clone(), relation.clone(), other.clone()])? {
                Sem::Mention { thing, kind } => {
                    Some(Sem::Question(Question::Locate { thing, kind }))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Words that only shape a question about where a thing is, by category.
const FILLERS: &[(Cat, &[(&str, &str)])] = &[
    // The speaker's wish to know: "I'd like to know", "I'm trying to
    // figure out".
    (
        Inquiry,
        &[
            ("en.inquiry.like_to_know", "i'd like to know"),
            ("en.inquiry.figure_out", "i'm trying to figure out"),
            ("en.inquiry.trying_to_find", "i'm trying to find"),
            ("en.inquiry.looking_for", "i'm looking for"),
        ],
    ),
    (
        Modal,
        &[("en.modal.can", "can"), ("en.modal.could", "could")],
    ),
    (
        Tell,
        &[
            ("en.tell.tell_me", "tell me"),
            ("en.tell.point_me_to", "point me to"),
        ],
    ),
    (
        Seek,
        &[
            ("en.seek.find", "find"),
            ("en.seek.locate", "locate"),
            ("en.seek.point_me_to", "point me to"),
            ("en.seek.jump_to", "jump to"),
        ],
    ),
    // "where does `x` live": where it is defined.
    (
        Defined,
        &[
            ("en.defined.declared", "declared"),
            ("en.defined.live", "live"),
        ],
    ),
    (
        Codebase,
        &[
            ("en.codebase.code", "code"),
            ("en.codebase.the_code", "the code"),
            ("en.codebase.the_codebase", "the codebase"),
        ],
    ),
    (
        Definition,
        &[
            ("en.definition.definition", "definition"),
            ("en.definition.def", "def"),
            ("en.definition.defn", "defn"),
        ],
    ),
    (
        Location,
        &[
            ("en.location.location", "location"),
            ("en.location.loc", "loc"),
        ],
    ),
];

/// More such words, from round 13.
const LATER_FILLERS: &[(Cat, &[(&str, &str)])] = &[
    // "do you know where `x` is defined?", "..., do you know where?"
    (Inquiry, &[("en.inquiry.do_you_know", "do you know")]),
    // "..., can you point me?", "could you point me there?"
    (
        Tell,
        &[
            ("en.tell.point_me", "point me"),
            ("en.tell.point_me_there", "point me there"),
        ],
    ),
    // "in which file?", "which file defines `x`"
    (Location, &[("en.location.file", "file")]),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let rule = |id, round, lhs, rhs, build| read_only_rule(id, round, lhs, rhs, build);
    // "where is `x` defined": how the system asks it, too. Also "where's
    // `x` defined", "where `x` defined", "where does `x` live", "where's it
    // live", "where air_n get defined", "where parses_urls fn live".
    let mut rules = vec![spoken(
        "en.question.where_defined",
        ROUND,
        Question,
        vec![C(Where), C(Target), C(Defined)],
        located,
        |v| Some(vec![blank(), v.clone(), blank()]),
    )];
    rules.extend(
        [
            // "where is `x`", "where's the definition of `x`", "where `x`"
            ("en.question.where", vec![C(Where), C(Target)]),
            // "`x` defined where", "Sub declared where", "license_info
            // defined wehre"
            (
                "en.question.defined_where",
                vec![C(Target), C(Defined), C(Where)],
            ),
            // "find def of `x`", "point me to `x`", "jump to `x` def"
            ("en.question.seek", vec![C(Seek), C(Target)]),
            // "where can I find the definition of `x`"
            (
                "en.question.where_can_i",
                vec![W("where can i"), C(Seek), C(Target)],
            ),
            // "loc of `set_theme_color`", "`Cmd` defn location"
            (
                "en.question.location_of",
                vec![C(Location), W("of"), C(Target)],
            ),
            ("en.question.target_location", vec![C(Target), C(Location)]),
        ]
        .map(|(id, rhs)| rule(id, ROUND, Question, rhs, located)),
    );
    // "where does `x` get defined", "where air_n get defined"
    rules.push(rule(
        "en.defined.get",
        ROUND,
        Defined,
        vec![W("get"), C(Defined)],
        nothing,
    ));
    rules.extend([
        // "def of scroll_by", "settings_changed definition": the
        // definition alone, asked for.
        rule(
            "en.question.definition_of",
            13,
            Question,
            vec![C(Definition), W("of"), C(NounPhrase)],
            locate,
        ),
        // "kept_glare definiton?": the same, the noun after.
        rule(
            "en.question.definition",
            25,
            Question,
            vec![C(NounPhrase), C(Definition)],
            locate,
        ),
        // "which file defines Network"
        rule(
            "en.question.which_file_defines",
            13,
            Question,
            vec![W("which"), C(Location), W("defines"), C(Target)],
            located,
        ),
        // "Where's `x` defined, in which file?", "protocol_name — which
        // file": the file of the thing said before.
        rule(
            "en.question.which_file",
            13,
            Question,
            vec![W("which"), C(Location)],
            previous,
        ),
        rule(
            "en.question.in_which_file",
            13,
            Question,
            vec![W("in which"), C(Location)],
            previous,
        ),
        // "..., do you know where?": where the thing said before is.
        rule(
            "en.indirect.where",
            13,
            Indirect,
            vec![W("where")],
            previous,
        ),
        // "I'm trying to figure out where `x` is defined, can you point
        // me?": the question, asked again.
        rule(
            "en.question.inquiry_again",
            13,
            Question,
            vec![C(Question), W(","), C(Inquiry)],
            first,
        ),
        // "Where is `set_focus` defined on `Chat`?": where the thing so
        // placed is.
        rule(
            "en.question.where_defined_on",
            13,
            Question,
            vec![
                W("where is"),
                C(Mention),
                C(Defined),
                C(Preposition),
                C(Mention),
            ],
            placed,
        ),
        // "where `mount_m` is defined, on GearLook i think": the place
        // said after the clause.
        rule(
            "en.indirect.where_defined_on",
            15,
            Indirect,
            vec![
                W("where"),
                C(Mention),
                W("is"),
                C(Defined),
                W(","),
                C(Preposition),
                C(Mention),
            ],
            placed,
        ),
    ]);
    rules.extend([
        // "can you tell me where `x` is defined", "I'd like to know where
        // ...", "I'm trying to find where `x` is used as a field type", "I'm
        // looking for structs that have `probe` field", "i'm trying to
        // figure out which structs have a field called `on`"
        rule(
            "en.question.inquiry_question",
            13,
            Question,
            vec![C(Inquiry), C(Question)],
            crate::last,
        ),
        // "where `x` is defined"
        rule(
            "en.indirect.where_defined",
            ROUND,
            Indirect,
            vec![W("where"), C(Target), W("is"), C(Defined)],
            located,
        ),
    ]);
    // Where the user wants it looked for: "where is `x` defined in the
    // code", "where does `x` live in the codebase", "what is `x` used for in
    // the code", "where is it defined in code?".
    rules.push(rule(
        "en.question.in_the_code",
        ROUND,
        Question,
        vec![C(Question), W("in"), C(Codebase)],
        first,
    ));
    // "whats Topic for in thsi codebase": the code the user is working
    // in, pointed at.
    rules.push(filler(
        "en.codebase.this_codebase",
        26,
        Codebase,
        "this codebase",
    ));
    rules.extend([
        // "where `parse_remote` lives", "where it lives": where the thing
        // is, as a clause.
        rule(
            "en.indirect.where_lives",
            10,
            Indirect,
            vec![W("where"), C(Target), W("lives")],
            located,
        ),
        // "..., do you know where it is?": where the thing is, with
        // nothing more said of it.
        rule(
            "en.indirect.where_is",
            25,
            Indirect,
            vec![W("where"), C(Target), W("is")],
            located,
        ),
        // "where it's defined": where the thing said before is.
        rule(
            "en.indirect.where_it_s_defined",
            9,
            Indirect,
            vec![W("where it's"), C(Defined)],
            previous,
        ),
        // "`LookFelt` - where defined": where the thing just set off is,
        // with nothing more said of it.
        rule(
            "en.question.where_defined_bare",
            10,
            Question,
            vec![W("where"), C(Defined)],
            previous,
        ),
        // "and where it is defined?": asked in the order of a clause.
        rule(
            "en.question.indirect",
            9,
            Question,
            vec![C(Indirect)],
            crate::last,
        ),
        // "where's `x` defined at"
        rule(
            "en.question.at",
            ROUND,
            Question,
            vec![C(Question), W("at")],
            located,
        ),
        // "where is `x` defined, please" is read by `en.question.please`
        // in request.rs.
        // A request for information that ends like a statement: "I'd like
        // to know where `x` is defined."
        rule(
            "en.utterance.question_stop",
            ROUND,
            Utterance,
            vec![C(Question), W(".")],
            |c| match c {
                [Sem::Question(q)] => Some(Sem::Utterance(Utterance::Ask(q.clone()))),
                _ => None,
            },
        ),
        // The thing itself, as mentioned: "the `x`", "the `x` struct", "the
        // function `x`"; or its definition: "the definition of `x`", "`x` def".
        spoken(
            "en.target.mention",
            3,
            Target,
            vec![C(Mention)],
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
        // "the definition of `x`"
        rule(
            "en.target.the",
            ROUND,
            Target,
            vec![W("the"), C(Target)],
            located,
        ),
        rule(
            "en.target.definition_of",
            ROUND,
            Target,
            vec![C(Definition), W("of"), C(NounPhrase)],
            locate,
        ),
        rule(
            "en.target.definition",
            ROUND,
            Target,
            vec![C(NounPhrase), C(Definition)],
            locate,
        ),
        // "Inquiry": a modal question "can you tell me"; the speaker's wish ("I'd like to know") is in `FILLERS`.
        rule(
            "en.inquiry.modal",
            ROUND,
            Inquiry,
            vec![C(Modal), W("you"), C(Tell)],
            nothing,
        ),
        // "where is `x` defined", "where is `x` called": how the system
        // opens the question, too. The other openings are in
        // [`crate::whence`].
        word(
            RuleInfo::new("en.whence.where_is", 9),
            Where,
            "where is",
            blank(),
        ),
        // "defined": how the system says it, too.
        word(
            RuleInfo::new("en.defined.defined", ROUND),
            Defined,
            "defined",
            blank(),
        ),
    ]);
    rules.extend(fillers(ROUND, FILLERS));
    rules.extend(fillers(13, LATER_FILLERS));
    // "`x` is defined at src/x.rs:12."
    rules.push(spoken(
        "en.reply.location",
        ROUND,
        Reply,
        vec![
            Class(ITEM),
            W("is"),
            C(Defined),
            W("at"),
            Class(PLACE),
            W("."),
        ],
        |_| None,
        |v: &Sem<O>| match v {
            Sem::Reply(Reply::Location(item)) => Some(vec![
                Sem::Item(item.clone()),
                blank(),
                Sem::Item(item.clone()),
            ]),
            _ => None,
        },
    ));
    rules
}
