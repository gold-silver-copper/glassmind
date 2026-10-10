//! Words as users often mistype them, each read as the word meant wherever
//! that word is read. A `Mistyped` rule's words are the word as typed, then
//! the word meant; it is never a phrase, so the system never says one. A
//! mistyping is a known word, so it is never read as a name; quote it in
//! backticks to name it. The words for a world's kinds are mistyped in its
//! bridge crate.
//!
//! A `SPELLINGS` entry is read the same way by the lexicon, credited to
//! the readings that use it: "whcih fields does `S` hav" reads as "which
//! fields does `S` have". Each is one fixed word, mistyped the
//! way people often mistype it, with its own ID and switch; it stands for
//! the word meant everywhere that word is read, never as a name or a
//! guess. Only plain words are corrected: code and quoted names are not.

use std::collections::BTreeMap;

use glassmind_grammar::Rule;
use glassmind_meaning::Ontology;
use glassmind_rules::{RuleInfo, Switches};

use crate::locate::fillers;
use crate::sem::{Cat, Sem};

/// Mistypings from round 18's questions about structs with a field of some
/// name: "whcih structs have field `area`", "which structs hav `kinds`
/// field", "what structs have teh field more_wanted", "structs contianing
/// field reason", "what structs have a field caled value".
const MISTYPED: &[(Cat, &[(&str, &str)])] = &[(
    Cat::Mistyped,
    &[
        ("en.mistyped.whcih", "whcih which"),
        ("en.mistyped.whihc", "whihc which"),
        ("en.mistyped.wich", "wich which"),
        ("en.mistyped.hav", "hav have"),
        ("en.mistyped.hve", "hve have"),
        ("en.mistyped.hvae", "hvae have"),
        ("en.mistyped.contianing", "contianing containing"),
        ("en.mistyped.containg", "containg containing"),
        ("en.mistyped.caled", "caled called"),
        ("en.mistyped.namde", "namde named"),
        ("en.mistyped.teh", "teh the"),
    ],
)];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    fillers(18, MISTYPED)
}

/// A rule's ID, the word as typed and the word meant.
type Spelling = (&'static str, &'static str, &'static str);

/// For each round, the rule, the word as typed and the word meant.
///
/// Round 17: "what is_general doo", "what does lead_up acutally do",
/// "what HudRounds for exacly", "PullRequestState whats it fro", "waht's
/// with_saved for", "what long_words_break_hard tesst", "actualy scratch
/// that, forget it".
///
/// Round 18: "what feilds pr_base have", "list fileds of
/// draw_add_filter struct pls", "whcih fields export_thread have", "what
/// fields random_nonce hav", "tag_transfer hav what fields", "what fields
/// draw_board_images struct hsa", "list teh fields of segment pls",
/// "draw_menu fields, quick lsit", "slab_interval memebers", "what does
/// thread_marks contian, field-wise", "nvm forget taht", "nvm forgte that".
///
/// Round 19, asking what the system holds of the conversation: "show me
/// what u rember of this convo", "show what ur keepign in context", "what
/// u remebering of this conversation", "what u keeping in contxt right
/// now". "u" is "you" typed short, as in "can u rename ..." and "why u
/// wnat to chagne that". Also asking why a change was proposed: "why u
/// wnat to change that", "why u want to chagne that", "reason for the
/// cahnge?".
///
/// Round 19, from questions about structs with a field of some type:
/// "structs w/ a field thats Kind or wraps it", "field type Milestone or
/// contains it, what structs have tht", "... or smth containg it", "a field
/// typed Handle (or wraping one)", "or that conatins it", "or contaning
/// it", "whose type is or incldues `x`", "directly or nestd". "w/" is not
/// mistyped but shortened, the way "with" is often written; the tokenizer
/// keeps the slash with the letter.
///
/// Round 21, from requests to rename a function: "rename `x` to `y`,
/// thnks", "`x` shoud be called `y`", "pls rename evrywhere", "want it
/// called `y` isntead".
const SPELLINGS: &[(u32, &[Spelling])] = &[
    (
        17,
        &[
            ("en.doing.mistyped.doo", "doo", "do"),
            ("en.doing.mistyped.doe", "doe", "does"),
            ("en.doing.mistyped.doess", "doess", "does"),
            ("en.doing.mistyped.tset", "tset", "test"),
            ("en.doing.mistyped.tesst", "tesst", "test"),
            ("en.doing.mistyped.chekcs", "chekcs", "checks"),
            ("en.doing.mistyped.chekcing", "chekcing", "checking"),
            ("en.use.mistyped.fro", "fro", "for"),
            ("en.hedge.mistyped.actualy", "actualy", "actually"),
            ("en.hedge.mistyped.acutally", "acutally", "actually"),
            ("en.hedge.mistyped.exaclty", "exaclty", "exactly"),
            ("en.hedge.mistyped.exacly", "exacly", "exactly"),
            ("en.what_is.mistyped.waht_s", "waht's", "what's"),
            ("en.what_is.mistyped.whta_s", "whta's", "what's"),
        ],
    ),
    (
        18,
        &[
            ("en.spelling.feilds", "feilds", "fields"),
            ("en.spelling.fileds", "fileds", "fields"),
            ("en.spelling.feild", "feild", "field"),
            ("en.spelling.fiedl", "fiedl", "field"),
            ("en.spelling.whcih", "whcih", "which"),
            ("en.spelling.whihc", "whihc", "which"),
            ("en.spelling.wich", "wich", "which"),
            ("en.spelling.waht", "waht", "what"),
            ("en.spelling.whta", "whta", "what"),
            ("en.spelling.wat", "wat", "what"),
            ("en.spelling.r", "r", "are"),
            ("en.spelling.tehy", "tehy", "they"),
            ("en.spelling.hav", "hav", "have"),
            ("en.spelling.hve", "hve", "have"),
            ("en.spelling.hvae", "hvae", "have"),
            ("en.spelling.hsa", "hsa", "has"),
            ("en.spelling.teh", "teh", "the"),
            ("en.spelling.taht", "taht", "that"),
            ("en.spelling.lsit", "lsit", "list"),
            ("en.spelling.fucntions", "fucntions", "functions"),
            ("en.spelling.functons", "functons", "functions"),
            ("en.spelling.functoins", "functoins", "functions"),
            ("en.spelling.funcitons", "funcitons", "functions"),
            ("en.spelling.fucntion", "fucntion", "function"),
            ("en.spelling.defiend", "defiend", "defined"),
            ("en.spelling.defnied", "defnied", "defined"),
            ("en.spelling.mnay", "mnay", "many"),
            ("en.spelling.renmae", "renmae", "rename"),
            ("en.spelling.chnage", "chnage", "change"),
            ("en.spelling.rememebr", "rememebr", "remember"),
            ("en.spelling.remeber", "remeber", "remember"),
            ("en.spelling.insdie", "insdie", "inside"),
            ("en.spelling.wher", "wher", "where"),
            ("en.spelling.whre", "whre", "where"),
            ("en.spelling.wehre", "wehre", "where"),
            ("en.spelling.caled", "caled", "called"),
            ("en.spelling.memebers", "memebers", "members"),
            ("en.spelling.contian", "contian", "contain"),
            ("en.spelling.contians", "contians", "contains"),
            ("en.spelling.contianing", "contianing", "containing"),
            ("en.spelling.forgte", "forgte", "forget"),
            ("en.spelling.forgett", "forgett", "forget"),
        ],
    ),
    (
        19,
        &[
            ("en.spelling.u", "u", "you"),
            ("en.spelling.keepng", "keepng", "keeping"),
            ("en.spelling.keepign", "keepign", "keeping"),
            ("en.spelling.rember", "rember", "remember"),
            ("en.spelling.rememeber", "rememeber", "remember"),
            ("en.spelling.remmeber", "remmeber", "remember"),
            ("en.spelling.remebering", "remebering", "remembering"),
            ("en.spelling.contxt", "contxt", "context"),
            ("en.spelling.wnat", "wnat", "want"),
            ("en.spelling.chagne", "chagne", "change"),
            ("en.spelling.cahnge", "cahnge", "change"),
            ("en.spelling.w_slash", "w/", "with"),
            ("en.spelling.thats", "thats", "that's"),
            ("en.spelling.tht", "tht", "that"),
            ("en.spelling.smth", "smth", "something"),
            ("en.spelling.wraping", "wraping", "wrapping"),
            ("en.spelling.conatins", "conatins", "contains"),
            ("en.spelling.contaning", "contaning", "containing"),
            ("en.spelling.incldues", "incldues", "includes"),
            ("en.spelling.nestd", "nestd", "nested"),
        ],
    ),
    (
        21,
        &[
            ("en.spelling.thnks", "thnks", "thanks"),
            ("en.spelling.shoud", "shoud", "should"),
            ("en.spelling.evrywhere", "evrywhere", "everywhere"),
            ("en.spelling.isntead", "isntead", "instead"),
        ],
    ),
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
