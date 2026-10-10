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

use glassmind_grammar::{Rule, RuleInfo, Sym};
use glassmind_meaning::{Action, Condition, Description, Name, Ontology, Thing, Utterance};

use crate::aside::filler;
use crate::asking::read_only_rule;
use crate::lexicon::{CODE, NAME, TEXT};
use crate::sem::{Cat, Sem};
use crate::spoken;
use crate::{blank, first, word};

use Cat::*;
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
    read_only_rule(id, ROUND, lhs, rhs, build)
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
        spoken(
            "en.command.rename_verb",
            ROUND,
            Command,
            vec![C(Rename), C(Mention), C(Into), Class(NAME)],
            rename,
            |v| match v {
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
            },
        ),
        // "Builder.cur_len -> chunk_bytes"
        heard(
            "en.command.arrow",
            Command,
            vec![C(Mention), C(Arrow), Class(NAME)],
            rename,
        ),
        // "rename it", "pls rename": a rename to a name not said here.
        heard(
            "en.command.rename_only",
            Command,
            vec![C(Rename), C(Mention)],
            rename,
        ),
        heard("en.command.rename_bare", Command, vec![C(Rename)], rename),
        // "`filename` field should be `file_name`", "should be called ..."
        heard(
            "en.command.should_be",
            Command,
            vec![C(Mention), C(ShouldBe), C(NewName)],
            rename,
        ),
        // The new name: "`y`", "something like `y`".
        heard("en.new_name.name", NewName, vec![Class(NAME)], first),
        // "the field": the thing said before, of that kind.
        heard(
            "en.mention.the_kind_only",
            Mention,
            vec![W("the"), C(Noun)],
            previous,
        ),
        // "the field name `fg`", "name `x`": the thing, by its name.
        heard(
            "en.mention.kind_name",
            Mention,
            vec![C(Noun), W("name"), C(Mention)],
            crate::mention::of_kind,
        ),
        // "-> rename to `url_slug`"
        filler("en.break.arrow", ROUND, Break, "- >"),
        // "`x` -> `y`, rename it", "`x` should be called `y`, could you
        // rename it?"
        heard(
            "en.utterance.confirmed",
            Utterance,
            vec![C(Utterance), W(","), C(Utterance)],
            confirmed,
        ),
        // "the field `tag` is pretty generic, could you rename it? Can you
        // do that, is that possible to do?"
        heard(
            "en.utterance.proxy_comma",
            Utterance,
            vec![C(Utterance), W(","), C(Errand)],
            request,
        ),
        // "call it `y`", "call it `y` instead"
        heard(
            "en.command.call",
            Command,
            vec![W("call"), C(Mention), Class(NAME)],
            rename,
        ),
        // "I'd like the `x` field renamed to `y`"
        heard(
            "en.command.like_renamed",
            Command,
            vec![
                W("i'd like"),
                C(Mention),
                W("renamed"),
                C(Into),
                Class(NAME),
            ],
            rename,
        ),
        // "`x` on `S`, rename it to `y`", "`S.x` — call it `y` instead"
        heard(
            "en.command.about",
            Command,
            vec![C(Topic), C(Break), C(Command)],
            about,
        ),
        heard(
            "en.command.about_bare",
            Command,
            vec![C(Topic), C(Command)],
            about,
        ),
        // "the `x` field on `S` is a bit vague to me, can you rename it to
        // `y`?", "... isn't very descriptive. Can you rename it to `y`?"
        heard(
            "en.utterance.about",
            Utterance,
            vec![C(Topic), C(Break), C(Utterance)],
            about,
        ),
        heard(
            "en.utterance.about_stop",
            Utterance,
            vec![C(Topic), W("."), C(Utterance)],
            about,
        ),
        // A topic: a thing mentioned, perhaps with what the user thinks of
        // it, up to the next punctuation: "`x` is confusing".
        heard("en.topic.mention", Topic, vec![C(Mention)], first),
        heard("en.topic.more", Topic, vec![C(Topic), Class(TEXT)], first),
        // "the field `x` in `S` is also used for calf it seems, so name is
        // a bit misleading"
        heard(
            "en.topic.comma_more",
            Topic,
            vec![C(Topic), W(","), Class(TEXT)],
            first,
        ),
        // "in `Chat` there is field `dm_news`", "in `S` the field `x`"
        heard(
            "en.topic.there_is",
            Topic,
            vec![C(Belonging), C(Mention), W("there is"), C(Mention)],
            belonging,
        ),
        heard(
            "en.topic.in_owner",
            Topic,
            vec![C(Belonging), C(Mention), C(Mention)],
            belonging,
        ),
        // "App::title_shown -> rename to window_title"
        heard(
            "en.command.about_arrow",
            Command,
            vec![C(Topic), C(Arrow), C(Command)],
            about,
        ),
        // "Config field saved_max_mb", "Worker recheck field": a part
        // named with its owner.
        heard(
            "en.mention.owner_part",
            Mention,
            vec![C(Mention), C(Part), C(Mention)],
            belonging,
        ),
        // "App's `on_mouse`", "Builder's seen_urls": a part named after
        // its owner's "'s".
        read_only_rule(
            "en.mention.possessive",
            21,
            Mention,
            vec![C(Mention), C(Possessive), C(Mention)],
            |c| match c {
                [owner, role, part] => belonging(&[role.clone(), owner.clone(), part.clone()]),
                _ => None,
            },
        ),
        heard(
            "en.mention.owner_part_after",
            Mention,
            vec![C(Mention), C(Mention), C(Part)],
            belonging,
        ),
        // "I'd like to rename `x` to `y`. Can you do that for me?"
        heard(
            "en.utterance.errand",
            Utterance,
            vec![C(Utterance), C(Errand)],
            request,
        ),
        heard(
            "en.command.errand",
            Command,
            vec![C(Command), W(","), C(Errand)],
            first,
        ),
        heard(
            "en.errand.prompt",
            Errand,
            vec![C(Prompt), C(Errand)],
            first,
        ),
    ];
    for (table, cat) in [
        (RENAME, Rename),
        (ARROWS, Arrow),
        (PROMPTS, Prompt),
        (ERRANDS, Errand),
    ] {
        rules.extend(blanks(cat, table));
    }
    let later = |id, lhs, rhs, build| read_only_rule(id, 11, lhs, rhs, build);
    rules.extend([
        // "rename `padded` to something clearer": a rename whose new name
        // is left for later.
        later(
            "en.command.rename_vague",
            Command,
            vec![C(Rename), C(Mention), C(Into), C(Vague)],
            rename,
        ),
        filler("en.vague.something", 11, Vague, "something"),
        later("en.vague.more", Vague, vec![C(Vague), Class(TEXT)], first),
        // "... to something clearer? maybe `inset_by_padding`", "... more
        // clear? I think `load_job_files` fits better"
        later(
            "en.utterance.offered",
            Utterance,
            vec![C(Utterance), C(Mark), C(Offer)],
            offered,
        ),
        // The name offered behind the words that frame a request ("maybe",
        // "I think").
        read_only_rule(
            "en.offer.prompt",
            12,
            Offer,
            vec![C(Prompt), C(NewName)],
            crate::last,
        ),
        later("en.offer.more", Offer, vec![C(Offer), Class(TEXT)], first),
        // "can we rename it into `y` so it's more clear what it is?": a
        // remark after a request to change something, which a question's
        // words may not be.
        later(
            "en.utterance.command_remark_mark",
            Utterance,
            vec![C(Command), C(Remark), W("?")],
            first,
        ),
        // "I think `x` should be called `y`. Could you rename that
        // function, please?": the same rename asked for again in a
        // sentence of its own.
        later(
            "en.utterance.confirmed_stop",
            Utterance,
            vec![C(Utterance), W("."), C(Utterance)],
            confirmed,
        ),
        // "`thread_found` should be renamed to `y`"
        later(
            "en.becomes.should_be_renamed",
            Becomes,
            vec![W("should be renamed"), C(Into)],
            first,
        ),
        // "Could you handle that for me?"
        filler("en.errand.handle_that", 11, Errand, "handle that"),
        // "the function `wakes` — the name is kinda unclear to me, ...
        // right? could you rename it": a topic the user is unsure of.
        later(
            "en.utterance.about_question",
            Utterance,
            vec![C(Topic), W("?"), C(Utterance)],
            about,
        ),
        // "so there is this fn `check_kit`, can we rename it": the thing
        // pointed out before the request about it.
        later(
            "en.topic.there_is_this",
            Topic,
            vec![W("there is this"), C(Mention)],
            first,
        ),
        // "what is `digit` even... ok anyway, could you rename it": the
        // thing wondered about, left aside for the request.
        later(
            "en.topic.what_is_even",
            Topic,
            vec![C(WhatIs), C(Mention), C(Hedge)],
            |c| c.get(1).cloned(),
        ),
    ]);
    let grown = |id, lhs, rhs, build| read_only_rule(id, 21, lhs, rhs, build);
    rules.extend([
        // "rename `x` to something better, maybe `y`", "rename `x` to
        // smth better pls, `y`": the name offered after a comma.
        grown(
            "en.utterance.offered_comma",
            Utterance,
            vec![C(Utterance), W(","), C(Offer)],
            offered,
        ),
        // "..., `snap_to_play_zoom`": a name offered with no words framing
        // it, written as code so it is no guess.
        grown("en.offer.code", Offer, vec![Class(CODE)], first),
        // "`run_menu_item` to `activate_menu_row`, thx": the thing and the
        // name it is to have, with no verb.
        grown(
            "en.command.mention_to",
            Command,
            vec![C(Mention), W("to"), Class(CODE)],
            rename,
        ),
        // "want it called `y` instead"
        grown(
            "en.command.want_called",
            Command,
            vec![W("want"), C(Mention), W("called"), Class(CODE)],
            rename,
        ),
        // "`cached_data` would be clearer as `cache_lookup`": what the
        // thing would be better as.
        grown(
            "en.becomes.would_be_as",
            Becomes,
            vec![W("would be"), Class(TEXT), W("as")],
            first,
        ),
    ]);
    // Words saying what a thing ought to be called.
    rules.push(filler(
        "en.should_be.called",
        ROUND,
        ShouldBe,
        "should be called",
    ));
    for (table, cat) in [(COMMAND_AFTER, Command), (ERRAND_AFTER, Errand)] {
        rules.extend(
            table
                .iter()
                .map(|(id, words)| heard(id, cat, vec![C(cat), W(words)], first)),
        );
    }
    rules
}
