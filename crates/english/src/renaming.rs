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
use glassmind_meaning::{Action, Condition, Description, Ontology, Reply, Thing, Utterance};

use crate::lexicon::{ITEM, NAME, NUMBER, TEXT};
use crate::sem::{Cat, Sem};
use crate::word;

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 5;

/// Renaming the thing mentioned among the children (or else the thing said
/// before: "rename to `y`") to the name among them, if there is one.
fn rename<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
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
    Some(Sem::Utterance(Utterance::Command(Action::Rename {
        thing,
        kind,
        name,
    })))
}

/// The thing a mention speaks of, as a thing: with its kind, if it has one,
/// as the thing that is it and of that kind.
fn as_thing<O: Ontology>(thing: &Thing<O>, kind: &Option<O::Kind>) -> Thing<O> {
    match kind {
        Some(kind) => Thing::Fitting(Description {
            kind: Some(kind.clone()),
            conditions: vec![Condition::Is(thing.clone())],
        }),
        None => thing.clone(),
    }
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
    match (part, owner) {
        (
            Sem::Mention { thing, kind: said },
            Sem::Mention {
                thing: owner,
                kind: owner_kind,
            },
        ) if *thing != Thing::Previous => Some(Sem::Mention {
            thing: Thing::Fitting(Description {
                kind: None,
                conditions: vec![
                    Condition::Is(thing.clone()),
                    Condition::Related {
                        relation: relation.clone(),
                        position: *position,
                        other: as_thing(owner, owner_kind),
                    },
                ],
            }),
            kind: kind.or(said.clone()),
        }),
        _ => None,
    }
}

/// "`x` on `S`, rename it to `y`": the request, with what it refers back to
/// taken to be the topic set up before it.
fn about<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let (Some(Sem::Mention { thing, kind }), Some(later)) = (c.first(), c.last()) else {
        return None;
    };
    match later {
        Sem::Utterance(Utterance::Command(Action::Rename {
            thing: Thing::Previous,
            kind: said,
            name,
        })) => Some(Sem::Utterance(Utterance::Command(Action::Rename {
            thing: thing.clone(),
            kind: said.clone().or(kind.clone()),
            name: name.clone(),
        }))),
        _ => None,
    }
}

/// "`x` field -> `y`, rename it": the first request, when the second asks
/// for the same rename of the same thing, or of what it refers back to.
fn confirmed<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    let (Some(first), Some(second)) = (c.first(), c.last()) else {
        return None;
    };
    match (first, second) {
        (
            Sem::Utterance(Utterance::Command(Action::Rename { thing, kind, name })),
            Sem::Utterance(Utterance::Command(Action::Rename {
                thing: again,
                kind: kind_again,
                name: name_again,
            })),
        ) if (*again == Thing::Previous || again == thing)
            && (kind_again.is_none() || kind.is_none() || kind_again == kind)
            && (name_again.is_none() || name_again == name) =>
        {
            Some(Sem::Utterance(Utterance::Command(Action::Rename {
                thing: thing.clone(),
                kind: kind.clone().or_else(|| kind_again.clone()),
                name: name.clone(),
            })))
        }
        _ => None,
    }
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

/// The thing mentioned among the children.
fn mentioned<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter().find(|v| matches!(v, Sem::Mention { .. })).cloned()
}

/// The request among the children.
fn request<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.iter()
        .find(|v| matches!(v, Sem::Utterance(Utterance::Command(_))))
        .cloned()
}

/// The value of a rule's first child.
fn first<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.first().cloned()
}

/// Verbs asking for a new name. The first is how the system says it.
const RENAME: &[(&str, &[&str])] = &[
    ("en.rename.rename", &["rename"]),
    ("en.rename.renaming", &["renaming"]),
    ("en.rename.change", &["change"]),
];

/// Arrows from an old name to a new one: "`S.x` -> `y`".
const ARROWS: &[(&str, &[&str])] = &[
    ("en.arrow.ascii", &["-", ">"]),
    ("en.arrow.unicode", &["→"]),
];

/// Words that frame a request to change something: "would you mind",
/// "is it possible to", "I think".
const PROMPTS: &[(&str, &[&str])] = &[
    ("en.prompt.would_you", &["would", "you"]),
    ("en.prompt.would_you_mind", &["would", "you", "mind"]),
    (
        "en.prompt.is_it_possible_to",
        &["is", "it", "possible", "to"],
    ),
    ("en.prompt.i_think", &["i", "think"]),
];

/// Words saying what a thing ought to be called: "should be called".
const SHOULD_BE: &[(&str, &[&str])] = &[
    ("en.should_be.should_be", &["should", "be"]),
    ("en.should_be.called", &["should", "be", "called"]),
    ("en.should_be.named", &["should", "be", "named"]),
];

/// Words after a request to change something, or after an errand, that
/// leave it as it was: "please", "pls", "everywhere", "instead".
const AFTER: &[(&str, &str, &[&str])] = &[
    ("en.command.please", "en.errand.please", &["please"]),
    (
        "en.command.comma_please",
        "en.errand.comma_please",
        &[",", "please"],
    ),
    ("en.command.pls", "en.errand.pls", &["pls"]),
    ("en.command.plz", "en.errand.plz", &["plz"]),
    (
        "en.command.everywhere",
        "en.errand.everywhere",
        &["everywhere"],
    ),
    (
        "en.command.everywhere_it_s_used",
        "en.errand.everywhere_it_s_used",
        &["everywhere", "it's", "used"],
    ),
    (
        "en.command.everywhere_its_used",
        "en.errand.everywhere_its_used",
        &[",", "everywhere", "its", "used"],
    ),
    (
        "en.command.update_usages",
        "en.errand.update_usages",
        &[",", "update", "usages"],
    ),
    (
        "en.command.update_all_usages",
        "en.errand.update_all_usages",
        &[",", "update", "all", "usages"],
    ),
    (
        "en.command.update_all_uses",
        "en.errand.update_all_uses",
        &[",", "update", "all", "uses"],
    ),
    (
        "en.command.rename_it",
        "en.errand.rename_it",
        &[",", "rename", "it"],
    ),
    (
        "en.command.if_that_s_ok",
        "en.errand.if_that_s_ok",
        &[",", "if", "that's", "ok"],
    ),
    ("en.command.instead", "en.errand.instead", &["instead"]),
];

/// Ways of asking for what was just requested to be carried out.
const ERRANDS: &[(&str, &[&str])] = &[
    ("en.errand.do_that", &["do", "that"]),
    ("en.errand.do_it", &["do", "it"]),
    (
        "en.errand.take_care_of_that",
        &["take", "care", "of", "that"],
    ),
    ("en.proxy.doing_that", &["doing", "that"]),
    (
        "en.proxy.possible_to_do",
        &["is", "that", "possible", "to", "do"],
    ),
    ("en.proxy.ok_to_do", &["is", "that", "ok", "to", "do"]),
];

/// Words after an errand that leave it as it was, besides those after a
/// request: "for me", "?".
const ERRAND_AFTER: &[(&str, &[&str])] = &[
    ("en.errand.for_me", &["for", "me"]),
    ("en.errand.question_mark", &["?"]),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let blank = || S::Const(Sem::Quote(String::new()));
    let rule = |id, lhs, rhs, build| {
        Rule::new(RuleInfo::new(id, ROUND), lhs, rhs, S::Build(build)).read_only()
    };
    let mut rules = vec![
        // "rename the field `x` to `y`"
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
                    Sem::Quote(String::new()),
                    Sem::Mention {
                        thing: thing.clone(),
                        kind: kind.clone(),
                    },
                    Sem::Quote(String::new()),
                    Sem::Name(name.clone()),
                ]),
                _ => None,
            }),
        ),
        // "Builder.cur_len -> chunk_bytes"
        rule(
            "en.command.arrow",
            Cat::Command,
            vec![C(Cat::Mention), C(Cat::Arrow), Class(NAME)],
            rename,
        ),
        // "rename to `y`" after the thing was set up: "`x` on `S`, rename to `y`"
        rule(
            "en.command.rename_previous",
            Cat::Command,
            vec![C(Cat::Rename), C(Cat::Into), Class(NAME)],
            rename,
        ),
        // "rename it", "pls rename": a rename to a name not said here.
        rule(
            "en.command.rename_only",
            Cat::Command,
            vec![C(Cat::Rename), C(Cat::Mention)],
            rename,
        ),
        rule(
            "en.command.rename_bare",
            Cat::Command,
            vec![C(Cat::Rename)],
            rename,
        ),
        // "the field `version` renamed to `schema_version`"
        rule(
            "en.command.renamed",
            Cat::Command,
            vec![C(Cat::Mention), W("renamed"), C(Cat::Into), C(Cat::NewName)],
            rename,
        ),
        // "`filename` field should be `file_name`", "should be called ..."
        rule(
            "en.command.should_be",
            Cat::Command,
            vec![C(Cat::Mention), C(Cat::ShouldBe), C(Cat::NewName)],
            rename,
        ),
        // "make it `commands`"
        rule(
            "en.command.make",
            Cat::Command,
            vec![W("make"), C(Cat::Mention), C(Cat::NewName)],
            rename,
        ),
        // "... to `y` for me"
        rule(
            "en.command.for_me",
            Cat::Command,
            vec![C(Cat::Command), W("for"), W("me")],
            request,
        ),
        // The new name: "`y`", "something like `y`".
        rule("en.new_name.name", Cat::NewName, vec![Class(NAME)], first),
        rule(
            "en.new_name.something_like",
            Cat::NewName,
            vec![W("something"), W("like"), C(Cat::NewName)],
            |c| c.last().cloned(),
        ),
        // "the field": the thing said before, of that kind.
        rule(
            "en.mention.the_kind_only",
            Cat::Mention,
            vec![W("the"), C(Cat::Noun)],
            previous,
        ),
        // "the field name `fg`", "name `x`": the thing, by its name.
        rule(
            "en.mention.kind_name",
            Cat::Mention,
            vec![C(Cat::Noun), W("name"), C(Cat::Mention)],
            crate::mention::of_kind,
        ),
        rule(
            "en.mention.name_first",
            Cat::Mention,
            vec![W("name"), C(Cat::Mention)],
            mentioned,
        ),
        // "-> rename to `url_slug`"
        rule("en.break.arrow", Cat::Break, vec![W("-"), W(">")], |_| {
            Some(Sem::Quote(String::new()))
        }),
        // "`x` -> `y`, rename it", "`x` should be called `y`, could you
        // rename it?"
        rule(
            "en.utterance.confirmed",
            Cat::Utterance,
            vec![C(Cat::Utterance), W(","), C(Cat::Utterance)],
            confirmed,
        ),
        rule(
            "en.utterance.confirmed_again",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Utterance)],
            confirmed,
        ),
        // "the field `tag` is pretty generic, could you rename it? Can you
        // do that, is that possible to do?"
        rule(
            "en.utterance.proxy_comma",
            Cat::Utterance,
            vec![C(Cat::Utterance), W(","), C(Cat::Errand)],
            request,
        ),
        // "..., can you do that, thanks?"
        rule(
            "en.proxy.interjection",
            Cat::Errand,
            vec![C(Cat::Errand), W(","), C(Cat::Interjection)],
            first,
        ),
        // "call it `y`", "call it `y` instead"
        rule(
            "en.command.call",
            Cat::Command,
            vec![W("call"), C(Cat::Mention), Class(NAME)],
            rename,
        ),
        // "I'd like the `x` field renamed to `y`"
        rule(
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
        // "the field `x` of `S`", "`x` on the `S` struct", "`x` in `S`"
        rule(
            "en.mention.belonging",
            Cat::Mention,
            vec![C(Cat::Mention), C(Cat::Belonging), C(Cat::Mention)],
            belonging,
        ),
        // "`x` on `S`, rename it to `y`", "`S.x` — call it `y` instead"
        rule(
            "en.command.about",
            Cat::Command,
            vec![C(Cat::Topic), C(Cat::Break), C(Cat::Command)],
            about,
        ),
        rule(
            "en.command.about_bare",
            Cat::Command,
            vec![C(Cat::Topic), C(Cat::Command)],
            about,
        ),
        // "the `x` field on `S` is a bit vague to me, can you rename it to
        // `y`?", "... isn't very descriptive. Can you rename it to `y`?"
        rule(
            "en.utterance.about",
            Cat::Utterance,
            vec![C(Cat::Topic), C(Cat::Break), C(Cat::Utterance)],
            about,
        ),
        rule(
            "en.utterance.about_stop",
            Cat::Utterance,
            vec![C(Cat::Topic), W("."), C(Cat::Utterance)],
            about,
        ),
        // A topic: a thing mentioned, perhaps with what the user thinks of
        // it, up to the next punctuation: "`x` is confusing".
        rule("en.topic.mention", Cat::Topic, vec![C(Cat::Mention)], first),
        rule(
            "en.topic.more",
            Cat::Topic,
            vec![C(Cat::Topic), Class(TEXT)],
            first,
        ),
        // "the field `x` in `S` is also used for calf it seems, so name is
        // a bit misleading"
        rule(
            "en.topic.comma_more",
            Cat::Topic,
            vec![C(Cat::Topic), W(","), Class(TEXT)],
            first,
        ),
        // "in `Chat` there is field `dm_news`", "in `S` the field `x`"
        rule(
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
        rule(
            "en.topic.in_owner",
            Cat::Topic,
            vec![C(Cat::Belonging), C(Cat::Mention), C(Cat::Mention)],
            belonging,
        ),
        // "App::title_shown -> rename to window_title"
        rule(
            "en.command.about_arrow",
            Cat::Command,
            vec![C(Cat::Topic), C(Cat::Arrow), C(Cat::Command)],
            about,
        ),
        // "`x` on `S` sounds like a bool to me... could you rename it?"
        Rule::new(
            RuleInfo::new("en.break.ellipsis", ROUND),
            Cat::Break,
            vec![W("."), W("."), W(".")],
            blank(),
        )
        .read_only(),
        // "Config field saved_max_mb", "Worker recheck field": a part
        // named with its owner.
        rule(
            "en.mention.owner_part",
            Cat::Mention,
            vec![C(Cat::Mention), C(Cat::Part), C(Cat::Mention)],
            belonging,
        ),
        rule(
            "en.mention.owner_part_after",
            Cat::Mention,
            vec![C(Cat::Mention), C(Cat::Mention), C(Cat::Part)],
            belonging,
        ),
        // "its field `x`": the thing itself.
        rule(
            "en.mention.its",
            Cat::Mention,
            vec![W("its"), C(Cat::Mention)],
            first,
        ),
        // "I'd like to rename `x` to `y`. Can you do that for me?"
        rule(
            "en.utterance.errand",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Errand)],
            request,
        ),
        rule(
            "en.command.errand",
            Cat::Command,
            vec![C(Cat::Command), W(","), C(Cat::Errand)],
            request,
        ),
        rule(
            "en.errand.prompt",
            Cat::Errand,
            vec![C(Cat::Prompt), C(Cat::Errand)],
            first,
        ),
        // "Renaming `S::x` to `y` changes 3 places."
        Rule::new(
            RuleInfo::new("en.reply.renaming", ROUND),
            Cat::Reply,
            vec![
                W("renaming"),
                Class(ITEM),
                W("to"),
                Class(NAME),
                W("changes"),
                Class(NUMBER),
                W("places"),
                W("."),
            ],
            S::Both(
                |_| None,
                |v: &Sem<O>| match v {
                    Sem::Reply(Reply::Proposal {
                        action:
                            Action::Rename {
                                thing: Thing::Item(item),
                                name: Some(name),
                                ..
                            },
                        spots,
                    }) => Some(vec![
                        Sem::Item(item.clone()),
                        Sem::Name(name.clone()),
                        Sem::Count(u64::try_from(spots.len()).ok()?),
                    ]),
                    _ => None,
                },
            ),
        ),
    ];
    for (table, cat) in [
        (RENAME, Cat::Rename),
        (ARROWS, Cat::Arrow),
        (PROMPTS, Cat::Prompt),
        (ERRANDS, Cat::Errand),
    ] {
        rules.extend(table.iter().map(|(id, words)| {
            Rule::new(
                RuleInfo::new(id, ROUND),
                cat,
                words.iter().map(|w| W(w)).collect(),
                blank(),
            )
        }));
    }
    rules.extend(SHOULD_BE.iter().map(|(id, words)| {
        word(
            RuleInfo::new(id, ROUND),
            Cat::ShouldBe,
            words,
            Sem::Quote(String::new()),
        )
        .read_only()
    }));
    for (command, errand, words) in AFTER {
        for (id, cat) in [(*command, Cat::Command), (*errand, Cat::Errand)] {
            let mut rhs = vec![C(cat)];
            rhs.extend(words.iter().map(|w| W(w)));
            rules.push(rule(id, cat, rhs, first));
        }
    }
    rules.extend(ERRAND_AFTER.iter().map(|(id, words)| {
        let mut rhs = vec![C(Cat::Errand)];
        rhs.extend(words.iter().map(|w| W(w)));
        rule(id, Cat::Errand, rhs, first)
    }));
    rules
}
