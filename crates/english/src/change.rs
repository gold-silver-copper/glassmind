//! Asking for a change: "rename to `y`", "`x` -> `y`", "make it `y`", with
//! follow-ups such as "for me" or "update the callers too", and the system's
//! proposal, "I'd rename `x` to `y`." (the rename itself is spoken by
//! `en.command.rename_verb` in [`crate::renaming`]).
//!
//! Follow-ups, marks and prompts all read as blank values, so a follow-up
//! carries the value of its first part and adds nothing to the request.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::{Ontology, Reply, Utterance};

use crate::lexicon::CODE;
use crate::renaming::{Phrases, blanks, heard, rename, request};
use crate::sem::{Cat, Sem};
use crate::{blank, first, word};

use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 5;

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let mut rules = vec![
        // "rename to `y`", after what is to be renamed was said.
        heard(
            "en.command.rename_to",
            Cat::Command,
            vec![W("rename"), C(Cat::Into), Class(CODE)],
            rename,
        ),
        // "rename it `submenu_modal`", "call it `vault_hand_placement`"
        heard(
            "en.command.naming",
            Cat::Command,
            vec![C(Cat::Naming), C(Cat::Mention), Class(CODE)],
            rename,
        ),
        // "`start_download` -> `spawn_fetch_jobs`", "`tx_view` renamed to
        // `extract_tx_view`", "`on_edited` should be `handle_edit_result`"
        heard(
            "en.command.becomes",
            Cat::Command,
            vec![C(Cat::Mention), C(Cat::Becomes), Class(CODE)],
            rename,
        ),
        // "rename `x` to `y` for me", "... Could you do that, please?"
        heard(
            "en.utterance.follow_up",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::FollowUp)],
            request,
        ),
        heard(
            "en.utterance.mark_follow_up",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Mark), C(Cat::FollowUp)],
            request,
        ),
        // "..., can you do it?", "... and update the callers too"
        heard(
            "en.utterance.joined_follow_up",
            Cat::Utterance,
            vec![C(Cat::Utterance), C(Cat::Join), C(Cat::FollowUp)],
            request,
        ),
        // A command, with or without its final punctuation.
        Rule::new(
            RuleInfo::new("en.utterance.command", ROUND),
            Cat::Utterance,
            vec![C(Cat::Command)],
            S::Both(first, |v| Some(vec![v.clone()])),
        ),
        heard(
            "en.utterance.command_mark",
            Cat::Utterance,
            vec![C(Cat::Command), W("?")],
            first,
        ),
        // "I'd rename `x` to `y`."
        Rule::new(
            RuleInfo::new("en.reply.proposal", ROUND),
            Cat::Reply,
            vec![W("I'd"), C(Cat::Command), W(".")],
            S::Both(
                |_| None,
                |v| match v {
                    Sem::Reply(Reply::Proposal { action, .. }) => {
                        Some(vec![Sem::Utterance(Utterance::Command(action.clone()))])
                    }
                    _ => None,
                },
            ),
        ),
        word(RuleInfo::new("en.into.to", ROUND), Cat::Into, "to", blank()),
    ];
    for (cat, table) in [
        (Cat::Into, INTO),
        (Cat::Becomes, BECOMES),
        (Cat::Naming, NAMING),
        (Cat::FollowUp, FOLLOW_UPS),
        (Cat::Mark, MARKS),
    ] {
        rules.extend(blanks(cat, table).map(Rule::read_only));
    }
    rules.extend([
        heard(
            "en.becomes.into",
            Cat::Becomes,
            vec![W("renamed"), C(Cat::Into)],
            first,
        ),
        // "could you do that", "pls rename", "do that for me, please?"
        heard(
            "en.follow_up.prompt",
            Cat::FollowUp,
            vec![C(Cat::Prompt), C(Cat::FollowUp)],
            first,
        ),
        heard(
            "en.follow_up.comma_interjection",
            Cat::FollowUp,
            vec![C(Cat::FollowUp), W(","), C(Cat::Interjection)],
            first,
        ),
        // "...? `regrid` is kinda confusing for me": a remark, as a sentence
        // of its own, so none of its words belong to the request.
        heard(
            "en.follow_up.remark",
            Cat::FollowUp,
            vec![C(Cat::Mark), C(Cat::Remark)],
            first,
        ),
        heard(
            "en.follow_up.mark",
            Cat::FollowUp,
            vec![C(Cat::FollowUp), C(Cat::Mark)],
            first,
        ),
    ]);
    rules
}

/// Words saying what a thing is to become, besides "to".
const INTO: Phrases = &[
    ("en.into.into", "into"),
    ("en.into.arrow", "- >"),
    ("en.into.unicode_arrow", "→"),
    // "rename `x` to something like `y`": the name the user offers.
    ("en.into.something_like", "to something like"),
];

/// Words saying a thing is to be called something else, after it.
const BECOMES: Phrases = &[
    ("en.becomes.arrow", "- >"),
    ("en.becomes.should_be", "should be"),
];

/// Verbs giving a thing a name, before it and the name.
const NAMING: Phrases = &[("en.naming.rename", "rename"), ("en.naming.make", "make")];

/// Words asking again, or in more detail, for a change just asked for.
const FOLLOW_UPS: Phrases = &[
    ("en.follow_up.make_that_change", "make that change"),
    ("en.follow_up.rename", "rename"),
    ("en.follow_up.rename_it", "rename it"),
    ("en.follow_up.for_me", "for me"),
    ("en.follow_up.everywhere", "everywhere"),
    ("en.follow_up.everywhere_used", "everywhere it's used"),
    ("en.follow_up.update_callers", "update the callers too"),
    (
        "en.follow_up.update_all_callers",
        "update all the callers too",
    ),
];

/// Marks ending a sentence inside a turn.
const MARKS: Phrases = &[("en.mark.question", "?"), ("en.mark.bang", "!")];
