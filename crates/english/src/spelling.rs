//! Words as users often mistype them, each read by the lexicon as the word
//! meant wherever that word is read, and credited to the readings that use
//! it: "whcih fields does `S` hav" reads as "which fields does `S` have".
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
///
/// Round 22, from questions about what calls a function: "who cals
/// `x`", "list callers fo `x`", "anythign call `x`", "who calls `x`,
/// need to knwo". Also from questions about what a function calls:
/// "what funtions `limit` call", "which functinos signature call", "fns
/// calld by room_message", "show calles of `x`", "what funcs does find
/// call internaly", and after asking what calls it: "which fns it caal",
/// "which functions it calsl", "what does it call itslef".
///
/// Round 23, from questions about structs with a field of some type:
/// "which structs hold a Sender in tehir fields"; and from telling the
/// system a name of the user's own: "im gonna call `Wound` the hole", "ill
/// call Confirm the gate", "im calling it word list frm here". "im" and
/// "ill" are "i'm" and "i'll" without the apostrophe.
///
/// Round 25, from putting right what was just asked: "sorry ment
/// `back_plated`", "oops i ment org_people_say_how_many_are_left_out", "sry
/// meant `Overlay`". "sry" is "sorry" typed short, as "thx" is "thanks".
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
            ("en.spelling.fucntions", "fucntions", "functions"),
            ("en.spelling.functons", "functons", "functions"),
            ("en.spelling.functoins", "functoins", "functions"),
            ("en.spelling.funcitons", "funcitons", "functions"),
            ("en.spelling.fucntion", "fucntion", "function"),
            ("en.spelling.defiend", "defiend", "defined"),
            ("en.spelling.defnied", "defnied", "defined"),
            ("en.spelling.mnay", "mnay", "many"),
            ("en.how_many.mistyped.mny", "mny", "many"),
            // "where's notified definde", "wehre's `with_file`", "locate
            // `middle_of` definiton", "loaction of `x`".
            ("en.defined.mistyped.definde", "definde", "defined"),
            ("en.defined.mistyped.deifned", "deifned", "defined"),
            ("en.defined.mistyped.definded", "definded", "defined"),
            ("en.whence.mistyped.wehre_s", "wehre's", "where's"),
            (
                "en.definition.mistyped.definiton",
                "definiton",
                "definition",
            ),
            ("en.location.mistyped.loaction", "loaction", "location"),
            // "how many funtcions call `x`", "how many fns calll `x`",
            // "which fns `x` callls", "how many fns callling `x`".
            ("en.rust.noun.mistyped.funtcions", "funtcions", "functions"),
            ("en.rust.verb.mistyped.calll", "calll", "call"),
            ("en.rust.verb.mistyped.callls", "callls", "calls"),
            ("en.rust.verb.mistyped.callling", "callling", "calling"),
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
    (
        22,
        &[
            ("en.spelling.cals", "cals", "calls"),
            ("en.spelling.fo", "fo", "of"),
            ("en.spelling.anythign", "anythign", "anything"),
            ("en.spelling.funtions", "funtions", "functions"),
            ("en.spelling.functinos", "functinos", "functions"),
            ("en.spelling.calld", "calld", "called"),
            ("en.spelling.calles", "calles", "callees"),
            ("en.spelling.caal", "caal", "call"),
            ("en.spelling.calsl", "calsl", "calls"),
            ("en.spelling.itslef", "itslef", "itself"),
            ("en.spelling.internaly", "internaly", "internally"),
            // From questions about structs with a field of some name:
            // "structs containg field reason", "what structs have a field
            // namde value", "what structs got a `dead` fild", "what strcuts
            // have field named lists".
            ("en.mistyped.containg", "containg", "containing"),
            ("en.mistyped.namde", "namde", "named"),
            ("en.rust.mistyped.fild", "fild", "field"),
            ("en.rust.mistyped.strucs", "strucs", "structs"),
            ("en.rust.mistyped.strucst", "strucst", "structs"),
            ("en.rust.mistyped.strutcs", "strutcs", "structs"),
            ("en.rust.mistyped.strcuts", "strcuts", "structs"),
        ],
    ),
    (
        23,
        &[
            ("en.spelling.tehir", "tehir", "their"),
            ("en.spelling.im", "im", "i'm"),
            ("en.spelling.ill", "ill", "i'll"),
            ("en.spelling.frm", "frm", "from"),
        ],
    ),
    (
        25,
        &[
            ("en.spelling.ment", "ment", "meant"),
            ("en.spelling.sry", "sry", "sorry"),
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
