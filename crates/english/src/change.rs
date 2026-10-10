//! Asking for a change: "rename to `y`", "`x` -> `y`", "make it `y`", with
//! follow-ups such as "for me" or "update the callers too", and the system's
//! proposal, "I'd rename `x` to `y`." (the rename itself is spoken by
//! `en.command.rename_verb` in [`crate::renaming`]).
//!
//! Follow-ups, marks and prompts all read as blank values, so a follow-up
//! carries the value of its first part and adds nothing to the request.

use glassmind_grammar::{Rule, RuleInfo, Sym};
use glassmind_meaning::{Ontology, Reply, Utterance};

use crate::aside::filler;
use crate::lexicon::CODE;
use crate::renaming::{Phrases, blanks, heard, rename, request};
use crate::sem::{Cat, Sem};
use crate::spoken;
use crate::{blank, first, word};

use Cat::*;
use Sym::{Cat as C, Class, Word as W};

const ROUND: u32 = 5;

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let mut rules = vec![
        // "rename to `y`", after what is to be renamed was said.
        heard(
            "en.command.rename_to",
            Command,
            vec![W("rename"), C(Into), Class(CODE)],
            rename,
        ),
        // "rename it `submenu_modal`", "call it `vault_hand_placement`"
        heard(
            "en.command.naming",
            Command,
            vec![C(Naming), C(Mention), Class(CODE)],
            rename,
        ),
        // "`start_download` -> `spawn_fetch_jobs`", "`tx_view` renamed to
        // `extract_tx_view`", "`on_edited` should be `handle_edit_result`"
        heard(
            "en.command.becomes",
            Command,
            vec![C(Mention), C(Becomes), Class(CODE)],
            rename,
        ),
        // "rename `x` to `y` for me", "... Could you do that, please?"
        heard(
            "en.utterance.follow_up",
            Utterance,
            vec![C(Utterance), C(FollowUp)],
            request,
        ),
        // "..., can you do it?", "... and update the callers too"
        heard(
            "en.utterance.joined_follow_up",
            Utterance,
            vec![C(Utterance), C(Join), C(FollowUp)],
            request,
        ),
        // A command, with or without its final punctuation.
        spoken(
            "en.utterance.command",
            ROUND,
            Utterance,
            vec![C(Command)],
            first,
            |v| Some(vec![v.clone()]),
        ),
        heard(
            "en.utterance.command_mark",
            Utterance,
            vec![C(Command), W("?")],
            first,
        ),
        // "I'd rename `x` to `y`."
        spoken(
            "en.reply.proposal",
            ROUND,
            Reply,
            vec![W("I'd"), C(Command), W(".")],
            |_| None,
            |v| match v {
                Sem::Reply(Reply::Proposal { action, .. }) => {
                    Some(vec![Sem::Utterance(Utterance::Command(action.clone()))])
                }
                _ => None,
            },
        ),
        word(RuleInfo::new("en.into.to", ROUND), Into, "to", blank()),
    ];
    for (cat, table) in [
        (Into, INTO),
        (Becomes, BECOMES),
        (Naming, NAMING),
        (FollowUp, FOLLOW_UPS),
        (Mark, MARKS),
    ] {
        rules.extend(blanks(cat, table).map(Rule::read_only));
    }
    rules.extend([
        heard(
            "en.becomes.into",
            Becomes,
            vec![W("renamed"), C(Into)],
            first,
        ),
        // "could you do that", "pls rename", "do that for me, please?"
        heard(
            "en.follow_up.prompt",
            FollowUp,
            vec![C(Prompt), C(FollowUp)],
            first,
        ),
        heard(
            "en.follow_up.comma_interjection",
            FollowUp,
            vec![C(FollowUp), W(","), C(Interjection)],
            first,
        ),
        // "...? `regrid` is kinda confusing for me": a remark, as a sentence
        // of its own, so none of its words belong to the request.
        heard(
            "en.follow_up.remark",
            FollowUp,
            vec![C(Mark), C(Remark)],
            first,
        ),
        // "rename `x` -> `y`, w/ all callers", "..., rename all uses"
        filler("en.follow_up.all_uses", 21, FollowUp, "all uses"),
        filler(
            "en.follow_up.with_all_callers",
            21,
            FollowUp,
            "with all callers",
        ),
        heard(
            "en.follow_up.mark",
            FollowUp,
            vec![C(FollowUp), C(Mark)],
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
const BECOMES: Phrases = &[("en.becomes.should_be", "should be")];

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
];

/// Marks ending a sentence inside a turn.
const MARKS: Phrases = &[("en.mark.question", "?"), ("en.mark.bang", "!")];
