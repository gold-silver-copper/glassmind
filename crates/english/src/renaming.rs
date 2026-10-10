//! Asking for a new name: "rename the field `x` of `S` to `y`", "`S.x` ->
//! `y`", "the `x` field on `S` is confusing, can you rename it to `y`?",
//! and the system's answer, "Renaming `S::x` to `y` changes 3 places.".
//!
//! A thing can be said to belong to another ("`x` on `S`"); the word for
//! belonging names the relation, so it comes from a bridge crate.
//!
//! A rename may leave the new name unsaid ("rename it", "can you rename
//! that field?"); said together with a request that gives it ("`x` -> `y`,
//! rename it"), the request is made once.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Action, Condition, Description, Name, Ontology, Thing, Utterance};

use crate::lexicon::{NAME, TEXT};
use crate::phrase;
use crate::sem::{Cat, Sem};
use crate::{blank, first, word};

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 5;

/// A rename of the thing to the name, if one was said, as a command.
fn renaming<O: Ontology>(thing: Thing<O>, kind: Option<O::Kind>, name: Option<Name>) -> Sem<O> {
    Sem::Utterance(Utterance::Command(Action::Rename { thing, kind, name }))
}

/// Renaming the thing mentioned among the children (or else the thing said
/// before: "rename to `y`") to the name among them, if there is one.
pub(crate) fn rename<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let (thing, kind) = c
        .iter()
        .find_map(|v| match v {
            Sem::Mention { thing, kind } => Some((thing.clone(), kind.clone())),
            _ => None,
        })
        .unwrap_or((Thing::Previous, None));
    let name = c.iter().rev().find_map(|v| match v {
        Sem::Name(n) => Some(n.clone()),
        _ => None,
    });
    Some(renaming(thing, kind, name))
}

/// "the field `x` of `S`", "`S` field `x`", "in `S` there is field `x`":
/// the thing mentioned, which also stands in the relation that belonging
/// (or being a part) names to its owner. The kind said stays with the
/// mention, wherever in the phrase it was said.
fn belonging<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let (part, kind, relation, position, owner) = match c {
        [part, Sem::Role { relation, position }, owner]
        | [Sem::Role { relation, position }, owner, part] => {
            (part, None, relation, position, owner)
        }
        [
            owner,
            Sem::Holder {
                kind,
                relation,
                position,
            },
            part,
        ]
        | [
            owner,
            part,
            Sem::Holder {
                kind,
                relation,
                position,
            },
        ] => (part, Some(kind.clone()), relation, position, owner),
        _ => return None,
    };
    let (
        Sem::Mention { thing, kind: said },
        Sem::Mention {
            thing: owner,
            kind: owner_kind,
        },
    ) = (part, owner)
    else {
        return None;
    };
    (*thing != Thing::Previous).then(|| Sem::Mention {
        thing: Thing::Fitting(Description {
            kind: None,
            conditions: vec![
                Condition::Is(thing.clone()),
                Condition::Related {
                    relation: relation.clone(),
                    position: *position,
                    // The owner, as the thing that is it and of the kind
                    // said with it, if one was.
                    other: match owner_kind {
                        Some(kind) => Thing::Fitting(Description {
                            kind: Some(kind.clone()),
                            conditions: vec![Condition::Is(owner.clone())],
                        }),
                        None => owner.clone(),
                    },
                },
            ],
        }),
        kind: kind.or(said.clone()),
    })
}

/// "`x` on `S`, rename it to `y`": the request, with what it refers back to
/// taken to be the topic set up before it.
fn about<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let [
        Sem::Mention { thing, kind },
        ..,
        Sem::Utterance(Utterance::Command(Action::Rename {
            thing: Thing::Previous,
            kind: said,
            name,
        })),
    ] = c
    else {
        return None;
    };
    Some(renaming(
        thing.clone(),
        said.clone().or(kind.clone()),
        name.clone(),
    ))
}

/// "`x` field -> `y`, rename it": the first request, when the second asks
/// for the same rename of the same thing, or of what it refers back to.
/// The second may give the name the first left unsaid: "rename `x`, call
/// it `y`".
fn confirmed<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let [
        Sem::Utterance(Utterance::Command(Action::Rename { thing, kind, name })),
        ..,
        Sem::Utterance(Utterance::Command(Action::Rename {
            thing: again,
            kind: kind_again,
            name: name_again,
        })),
    ] = c
    else {
        return None;
    };
    ((*again == Thing::Previous || again == thing)
        && (kind_again.is_none() || kind.is_none() || kind_again == kind)
        && (name_again.is_none() || name.is_none() || name_again == name))
        .then(|| {
            renaming(
                thing.clone(),
                kind.clone().or_else(|| kind_again.clone()),
                name.clone().or_else(|| name_again.clone()),
            )
        })
}

/// "rename `x` to something clearer? maybe `y`": the request, with the name
/// offered after it, if it left the name unsaid.
fn offered<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let [
        Sem::Utterance(Utterance::Command(Action::Rename {
            thing,
            kind,
            name: None,
        })),
        ..,
        Sem::Name(name),
    ] = c
    else {
        return None;
    };
    Some(renaming(thing.clone(), kind.clone(), Some(name.clone())))
}

/// "the field": the thing said before, of the kind named.
fn previous<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    match c {
        [Sem::Kind { kind, .. }] => Some(Sem::Mention {
            thing: Thing::Previous,
            kind: Some(kind.clone()),
        }),
        _ => None,
    }
}

/// The request among the children (rejecting an utterance that is not one).
pub(crate) fn request<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter()
        .find(|v| matches!(v, Sem::Utterance(Utterance::Command(_))))
        .cloned()
}

/// A rule of round 5 that is read but never said.
pub(crate) fn heard<O: Ontology>(
    id: &'static str,
    lhs: Cat,
    rhs: Vec<Sym<Cat>>,
    build: fn(&[Sem<O>]) -> Option<Sem<O>>,
) -> Rule<Cat, Sem<O>> {
    Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
}

/// Rules reading each phrase of a table of (id, phrase) as a blank `cat`,
/// the first of them how the system says it.
pub(crate) fn blanks<O: Ontology>(
    cat: Cat,
    table: Phrases,
) -> impl Iterator<Item = Rule<Cat, Sem<O>>> {
    table
        .iter()
        .map(move |(id, words)| word(RuleInfo::new(id, ROUND), cat, words, blank()))
}

/// A table of rule IDs and the phrases they read.
pub(crate) type Phrases = &'static [(&'static str, &'static str)];

/// Verbs asking for a new name. The first is how the system says it.
const RENAME: Phrases = &[
    ("en.rename.rename", "rename"),
    ("en.rename.renaming", "renaming"),
    ("en.rename.change", "change"),
];

/// Arrows from an old name to a new one: "`S.x` -> `y`".
const ARROWS: Phrases = &[("en.arrow.ascii", "- >"), ("en.arrow.unicode", "→")];

/// Words that frame a request to change something: "would you mind",
/// "is it possible to", "I think".
const PROMPTS: Phrases = &[
    ("en.prompt.would_you", "would you"),
    ("en.prompt.would_you_mind", "would you mind"),
    ("en.prompt.i_think", "i think"),
];

/// Words after a request to change something that leave it as it was:
/// "please", "pls", ", update all usages", "instead".
const COMMAND_AFTER: Phrases = &[
    ("en.command.please", "please"),
    ("en.command.comma_please", ", please"),
    ("en.command.pls", "pls"),
    ("en.command.update_usages", ", update usages"),
    ("en.command.update_all_usages", ", update all usages"),
    ("en.command.update_all_uses", ", update all uses"),
    ("en.command.rename_it", ", rename it"),
    ("en.command.instead", "instead"),
    ("en.command.for_me", "for me"),
];

/// Ways of asking for what was just requested to be carried out.
const ERRANDS: Phrases = &[
    ("en.errand.do_that", "do that"),
    ("en.errand.do_it", "do it"),
    ("en.errand.take_care_of_that", "take care of that"),
    ("en.proxy.doing_that", "doing that"),
    ("en.proxy.possible_to_do", "is that possible to do"),
    ("en.proxy.ok_to_do", "is that ok to do"),
];

/// Words after an errand that leave it as it was: ", please", "for me", "?".
const ERRAND_AFTER: Phrases = &[
    ("en.errand.comma_please", ", please"),
    ("en.errand.for_me", "for me"),
    ("en.errand.question_mark", "?"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let mut rules = vec![
        // "rename the field `x` to `y`": how the system says it, too.
        Rule::new(
            RuleInfo::new("en.command.rename_verb", ROUND),
            Cat::Command,
            vec![C(Cat::Rename), C(Cat::Mention), C(Cat::Into), Class(NAME)],
            S::Both(rename, |v| match v {
                Sem::Utterance(Utterance::Command(Action::Rename {
                    thing,
                    kind,
                    name: Some(name),
                })) => Some(vec![
                    blank(),
                    Sem::Mention {
                        thing: thing.clone(),
                        kind: kind.clone(),
                    },
                    blank(),
                    Sem::Name(name.clone()),
                ]),
                _ => None,
            }),
        ),
        // "Builder.cur_len -> chunk_bytes"
        heard(
            "en.command.arrow",
            Cat::Command,
            vec![C(Cat::Mention), C(Cat::Arrow), Class(NAME)],
            rename,
        ),
        // "rename it", "pls rename": a rename to a name not said here.
        heard(
            "en.command.rename_only",
            Cat::Command,
            vec![C(Cat::Rename), C(Cat::Mention)],
            rename,
        ),
        heard(
            "en.command.rename_bare",
            Cat::Command,
            vec![C(Cat::Rename)],
            rename,
        ),
        // "`filename` field should be `file_name`", "should be called ..."
        heard(
            "en.command.should_be",
            Cat::Command,
            vec![C(Cat::Mention), C(Cat::ShouldBe), C(Cat::NewName)],
            rename,
        ),
        // The new name: "`y`", "something like `y`".
        heard("en.new_name.name", Cat::NewName, vec![Class(NAME)], first),
        // "the field": the thing said before, of that kind.
        heard(
            "en.mention.the_kind_only",
            Cat::Mention,
            vec![W("the"), C(Cat::Noun)],
            previous,
        ),
        // "the field name `fg`", "name `x`": the thing, by its name.
        heard(
            "en.mention.kind_name",
            Cat::Mention,
            vec![C(Cat::Noun), W("name"), C(Cat::Mention)],
            crate::mention::of_kind,
        ),
        heard(
            "en.mention.name_first",
            Cat::Mention,
            vec![W("name"), C(Cat::Mention)],
            first,
        ),
        // "-> rename to `url_slug`"
        word(
            RuleInfo::new("en.break.arrow", ROUND),
            Cat::Break,
            "- >",
            blank(),
        )
        .read_only(),
        // "`x` -> `y`, rename it", "`x` should be called `y`, could you
        // rename it?"
        heard(
            "en.utterance.confirmed",
            Cat::Utterance,
            vec![C(Cat::Utterance), W(","), C(Cat::Utterance)],
            confirmed,
        ),
        // "the field `tag` is pretty generic, could you rename it? Can you
        // do that, is that possible to do?"
        heard(
            "en.utterance.proxy_comma",
            Cat::Utterance,
            vec![C(Cat::Utterance), W(","), C(Cat::Errand)],
            request,
        ),
        // "call it `y`", "call it `y` instead"
        heard(
            "en.command.call",
            Cat::Command,
            vec![W("call"), C(Cat::Mention), Class(NAME)],
            rename,
        ),
        // "I'd like the `x` field renamed to `y`"
        heard(
            "en.command.like_renamed",
            Cat::Command,
            vec![
                W("i'd"),
                W("like"),
                C(Cat::Mention),
                W("renamed"),
                C(Cat::Into),
                Class(NAME),
            ],
            rename,
        ),
        // "`x` on `S`, rename it to `y`", "`S.x` — call it `y` instead"
        heard(
            "en.command.about",
            Cat::Command,
            vec![C(Cat::Topic), C(Cat::Break), C(Cat::Command)],
            about,
        ),
        heard(
            "en.command.about_bare",
            Cat::Command,
            vec![C(Cat::Topic), C(Cat::Command)],
            about,
        ),
        // "the `x` field on `S` is a bit vague to me, can you rename it to
        // `y`?", "... isn't very descriptive. Can you rename it to `y`?"
        heard(
            "en.utterance.about",
            Cat::Utterance,
            vec![C(Cat::Topic), C(Cat::Break), C(Cat::Utterance)],
            about,
        ),
        heard(
            "en.utterance.about_stop",
            Cat::Utterance,
            vec![C(Cat::Topic), W("."), C(Cat::Utterance)],
            about,
        ),
        // A topic: a thing mentioned, perhaps with what the user thinks of
        // it, up to the next punctuation: "`x` is confusing".
        heard("en.topic.mention", Cat::Topic, vec![C(Cat::Mention)], first),
        heard(
            "en.topic.more",
            Cat::Topic,
            vec![C(Cat::Topic), Class(TEXT)],
            first,
        ),
        // "the field `x` in `S` is also used for calf it seems, so name is
        // a bit misleading"
        heard(
            "en.topic.comma_more",
            Cat::Topic,
            vec![C(Cat::Topic), W(","), Class(TEXT)],
            first,
        ),
        // "in `Chat` there is field `dm_news`", "in `S` the field `x`"
        heard(
            "en.topic.there_is",
            Cat::Topic,
            vec![
                C(Cat::Belonging),
                C(Cat::Mention),
                W("there"),
                W("is"),
                C(Cat::Mention),
            ],
            belonging,
        ),
        heard(
            "en.topic.in_owner",
            Cat::Topic,
            vec![C(Cat::Belonging), C(Cat::Mention), C(Cat::Mention)],
            belonging,
        ),
        // "App::title_shown -> rename to window_title"
        heard(
            "en.command.about_arrow",
            Cat::Command,
            vec![C(Cat::Topic), C(Cat::Arrow), C(Cat::Command)],
            about,
        ),
        // "Config field saved_max_mb", "Worker recheck field": a part
        // named with its owner.
        heard(
            "en.mention.owner_part",
            Cat::Mention,
            vec![C(Cat::Mention), C(Cat::Part), C(Cat::Mention)],
            belonging,
        ),
        heard(
            "en.mention.owner_part_after",
            Cat::Mention,
            vec![C(Cat::Mention), C(Cat::Mention), C(Cat::Part)],
            belonging,
        ),
        // "its field `x`": the thing itself.
        heard(
            "en.mention.its",
            Cat::Mention,
            vec![W("its"), C(Cat::Mention)],
            first,
        ),
        // "I'd like to rename `x` to `y`. Can you do that for me?"
        heard(
            "en.utterance.errand",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Errand)],
            request,
        ),
        heard(
            "en.command.errand",
            Cat::Command,
            vec![C(Cat::Command), W(","), C(Cat::Errand)],
            first,
        ),
        heard(
            "en.errand.prompt",
            Cat::Errand,
            vec![C(Cat::Prompt), C(Cat::Errand)],
            first,
        ),
    ];
    for (table, cat) in [
        (RENAME, Cat::Rename),
        (ARROWS, Cat::Arrow),
        (PROMPTS, Cat::Prompt),
        (ERRANDS, Cat::Errand),
    ] {
        rules.extend(blanks(cat, table));
    }
    let later = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, 11), lhs, rhs, S::Build(build)).read_only()
    };
    rules.extend([
        // "rename `padded` to something clearer": a rename whose new name
        // is left for later.
        later(
            "en.command.rename_vague",
            Cat::Command,
            vec![C(Cat::Rename), C(Cat::Mention), C(Cat::Into), C(Cat::Vague)],
            rename,
        ),
        word(
            RuleInfo::new("en.vague.something", 11),
            Cat::Vague,
            "something",
            blank(),
        )
        .read_only(),
        later(
            "en.vague.more",
            Cat::Vague,
            vec![C(Cat::Vague), Class(TEXT)],
            first,
        ),
        // "... to something clearer? maybe `inset_by_padding`", "... more
        // clear? I think `load_job_files` fits better"
        later(
            "en.utterance.offered",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Mark), C(Cat::Offer)],
            offered,
        ),
        // The name offered behind the words that frame a request ("maybe",
        // "I think").
        Rule::new(
            RuleInfo::new("en.offer.prompt", 12),
            Cat::Offer,
            vec![C(Cat::Prompt), C(Cat::NewName)],
            S::Build(crate::last),
        )
        .read_only(),
        later(
            "en.offer.more",
            Cat::Offer,
            vec![C(Cat::Offer), Class(TEXT)],
            first,
        ),
        // "could you rename it to `open_tab_at` maybe?"
        later(
            "en.command.maybe",
            Cat::Command,
            vec![C(Cat::Command), W("maybe")],
            first,
        ),
        // "can we rename it into `y` so it's more clear what it is?": a
        // remark after a request to change something, which a question's
        // words may not be.
        later(
            "en.utterance.command_remark_mark",
            Cat::Utterance,
            vec![C(Cat::Command), C(Cat::Remark), W("?")],
            first,
        ),
        // "I think `x` should be called `y`. Could you rename that
        // function, please?": the same rename asked for again in a
        // sentence of its own.
        later(
            "en.utterance.confirmed_stop",
            Cat::Utterance,
            vec![C(Cat::Utterance), W("."), C(Cat::Utterance)],
            confirmed,
        ),
        // "`thread_found` should be renamed to `y`"
        later(
            "en.becomes.should_be_renamed",
            Cat::Becomes,
            vec![W("should"), W("be"), W("renamed"), C(Cat::Into)],
            first,
        ),
        // "Could you handle that for me?"
        word(
            RuleInfo::new("en.errand.handle_that", 11),
            Cat::Errand,
            "handle that",
            blank(),
        )
        .read_only(),
        // "the function `wakes` — the name is kinda unclear to me, ...
        // right? could you rename it": a topic the user is unsure of.
        later(
            "en.utterance.about_question",
            Cat::Utterance,
            vec![C(Cat::Topic), W("?"), C(Cat::Utterance)],
            about,
        ),
        // "so there is this fn `check_kit`, can we rename it": the thing
        // pointed out before the request about it.
        later(
            "en.topic.there_is_this",
            Cat::Topic,
            vec![W("there"), W("is"), W("this"), C(Cat::Mention)],
            first,
        ),
        // "what is `digit` even... ok anyway, could you rename it": the
        // thing wondered about, left aside for the request.
        later(
            "en.topic.what_is_even",
            Cat::Topic,
            vec![C(Cat::WhatIs), C(Cat::Mention), C(Cat::Hedge)],
            |c| c.get(1).cloned(),
        ),
        // "change name of `rounded` to `to_ellipse`", "change `load_older`
        // name to ...": the thing whose name is meant.
        later(
            "en.mention.name_of",
            Cat::Mention,
            vec![W("name"), W("of"), C(Cat::Mention)],
            first,
        ),
        later(
            "en.mention.name_after",
            Cat::Mention,
            vec![C(Cat::Mention), W("name")],
            first,
        ),
    ]);
    // Words saying what a thing ought to be called.
    rules.push(
        word(
            RuleInfo::new("en.should_be.called", ROUND),
            Cat::ShouldBe,
            "should be called",
            blank(),
        )
        .read_only(),
    );
    for (table, cat) in [(COMMAND_AFTER, Cat::Command), (ERRAND_AFTER, Cat::Errand)] {
        rules.extend(table.iter().map(|(id, words)| {
            let mut rhs = vec![C(cat)];
            rhs.extend(phrase(words));
            heard(id, cat, rhs, first)
        }));
    }
    rules
}
