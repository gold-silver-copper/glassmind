//! Words as users often mistype them, each read by the lexicon as the word
//! meant wherever that word is read, and credited to the readings that use
//! it: "wich fields does `S` hav" reads as "which fields does `S` have".
//! Each is one fixed word, mistyped the way people often mistype it, with
//! its own ID and switch; it stands for the word meant everywhere that word
//! is read, never as a name or a guess, and the system never says it. Only
//! plain words are corrected: code and quoted names are not, so quote a
//! mistyping in backticks to name it.

use std::collections::BTreeMap;

use glassmind_rules::{RuleInfo, Switches};

/// A rule's ID, the word as typed and the word meant.
type Spelling = (&'static str, &'static str, &'static str);

/// For each round, the rule, the word as typed and the word meant.
///
/// Most mistyped words need no entry: `typos` reads a plain word the grammar
/// does not know as a known word a slip or two away. These are the ones it
/// cannot read, or not reliably: words typed short ("u", "r", "im", "sry",
/// "smth", "w/", where "w/" is "with" shortened and the tokenizer keeps the
/// slash with the letter), slips into a word of two letters ("doo", "fo"),
/// slips as near to another known word ("hav": "has" or "have"; "wnat":
/// "want" or "what"), and slips that are also read as a name, where the
/// reading of the name could otherwise win ("which structs hold a Sender in
/// tehir fields", "what structs hav a field thats `x` or contians it").
///
/// Round 17: "what is_general doo", "PullRequestState whats it fro".
///
/// Round 18: "what fields random_nonce hav", "list teh fields of segment
/// pls", "where Behind defiend", "how many fns calll `x`", "wehre's
/// `with_file`", "roughly how mnay callers".
///
/// Round 19, asking what the system holds of the conversation: "show me
/// what u rember of this convo", "show what ur keepign in context"; asking
/// why a change was proposed: "why u wnat to change that"; and about
/// structs with a field of some type: "structs w/ a field thats Kind or
/// wraps it", "what structs have tht", "... or smth containg it".
///
/// Round 22: "list callers fo `x`", "structs containg field reason",
/// "what structs got a `dead` fild", "what strucs have field named lists".
///
/// Round 23: "which structs hold a Sender in tehir fields", "im gonna call
/// `Wound` the hole", "ill call Confirm the gate". "im" and "ill" are
/// "i'm" and "i'll" without the apostrophe.
///
/// Round 25: "sry meant `Overlay`", "sry" being "sorry" typed short.
const SPELLINGS: &[(u32, &[Spelling])] = &[
    (
        17,
        &[
            ("en.doing.mistyped.doo", "doo", "do"),
            ("en.use.mistyped.fro", "fro", "for"),
        ],
    ),
    (
        18,
        &[
            ("en.spelling.wich", "wich", "which"),
            ("en.spelling.waht", "waht", "what"),
            ("en.spelling.whta", "whta", "what"),
            ("en.spelling.wat", "wat", "what"),
            ("en.spelling.r", "r", "are"),
            ("en.spelling.hav", "hav", "have"),
            ("en.spelling.teh", "teh", "the"),
            ("en.spelling.defiend", "defiend", "defined"),
            ("en.spelling.mnay", "mnay", "many"),
            ("en.how_many.mistyped.mny", "mny", "many"),
            ("en.defined.mistyped.definde", "definde", "defined"),
            ("en.rust.verb.mistyped.calll", "calll", "call"),
            ("en.spelling.rememebr", "rememebr", "remember"),
            ("en.spelling.wehre", "wehre", "where"),
            ("en.spelling.contians", "contians", "contains"),
        ],
    ),
    (
        19,
        &[
            ("en.spelling.u", "u", "you"),
            ("en.spelling.keepign", "keepign", "keeping"),
            ("en.spelling.rember", "rember", "remember"),
            ("en.spelling.wnat", "wnat", "want"),
            ("en.spelling.w_slash", "w/", "with"),
            ("en.spelling.thats", "thats", "that's"),
            ("en.spelling.tht", "tht", "that"),
            ("en.spelling.smth", "smth", "something"),
        ],
    ),
    (
        22,
        &[
            ("en.spelling.fo", "fo", "of"),
            ("en.mistyped.containg", "containg", "containing"),
            ("en.rust.mistyped.fild", "fild", "field"),
            ("en.rust.mistyped.strucs", "strucs", "structs"),
            ("en.rust.mistyped.strucst", "strucst", "structs"),
        ],
    ),
    (
        23,
        &[
            ("en.spelling.tehir", "tehir", "their"),
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
