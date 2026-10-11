//! Words as people shorten them in chat ("u", "r", "smth", "w/", "im"),
//! each read by the lexicon as the word meant wherever that word is read,
//! and credited to the readings that use it. Each has its own ID and
//! switch; it stands for the word meant everywhere that word is read, never
//! as a name or a guess, and the system never says it. Mistyped words are
//! not listed here: `typos` reads a plain word the grammar does not know as
//! a known word a slip or two away, and mind prefers readings whose names
//! name something. Only plain words are read this way: code and quoted
//! names are not.

use std::collections::BTreeMap;

use glassmind_rules::{RuleInfo, Switches};

/// A rule's ID, the word as typed and the word meant.
type Spelling = (&'static str, &'static str, &'static str);

/// For each round, the rule, the word as typed and the word meant: words
/// typed short ("u", "r", "sry", "smth", "w/", where the tokenizer keeps
/// the slash with the letter, "tht", "thats", "mny") and contractions
/// without the apostrophe ("im", "ill").
const SPELLINGS: &[(u32, &[Spelling])] = &[
    (
        18,
        &[
            ("en.spelling.r", "r", "are"),
            ("en.how_many.mistyped.mny", "mny", "many"),
        ],
    ),
    (
        19,
        &[
            ("en.spelling.u", "u", "you"),
            ("en.spelling.w_slash", "w/", "with"),
            ("en.spelling.thats", "thats", "that's"),
            ("en.spelling.tht", "tht", "that"),
            ("en.spelling.smth", "smth", "something"),
        ],
    ),
    (
        23,
        &[
            ("en.spelling.im", "im", "i'm"),
            ("en.spelling.ill", "ill", "i'll"),
        ],
    ),
    (25, &[("en.spelling.sry", "sry", "sorry")]),
];

/// Every spelling: the word as typed, its rule and the word meant.
fn all() -> impl Iterator<Item = (&'static str, (RuleInfo, &'static str))> {
    SPELLINGS.iter().flat_map(|&(round, table)| {
        table
            .iter()
            .map(move |&(id, typed, meant)| (typed, (RuleInfo::new(id, round), meant)))
    })
}

/// Every spelling rule, for listing and switching off.
pub fn corrections() -> impl Iterator<Item = RuleInfo> {
    all().map(|(_, (rule, _))| rule)
}

/// The spellings switched on, by the word as typed.
pub fn spellings(switches: &Switches) -> BTreeMap<&'static str, (RuleInfo, &'static str)> {
    all()
        .filter(|(_, (rule, _))| switches.on(rule.id))
        .collect()
}
