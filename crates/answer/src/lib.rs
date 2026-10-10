//! The machine-readable answer format.
//!
//! Three programs speak it: glassmind writes a [`Response`] for every turn it
//! answers, the oracle writes the expected [`Response`] for every task it
//! computes, and the loop's scorer compares the two. Nothing here knows how an
//! answer is computed; it only fixes what an answer looks like, so that the
//! oracle can stay independent of glassmind's own code.
//!
//! The exact meaning of each response kind is specified in `docs/tasks.md`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// The kinds of item an answer can name. A deliberately small set: the
/// answer format only needs to tell items apart, not describe them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// A free function, an associated function or a method.
    Function,
    Struct,
    Enum,
    Trait,
    /// A named field of a struct.
    Field,
    /// A problem a tool reported (a compiler error or a lint): `name` is its
    /// code, such as `E0425`.
    Problem,
    /// A failing test: `name` is its path; `file` is empty and `line` 0.
    Test,
}

/// One item in one source, identified by where its name is written.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ItemKey {
    /// The source (repository) name the conversation was given.
    pub source: String,
    /// Path of the file, relative to the source root, with `/` separators.
    pub file: String,
    /// 1-based line of the item's name.
    pub line: u32,
    pub name: String,
    pub kind: ItemKind,
}

/// One textual replacement of an identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Change {
    pub source: String,
    pub file: String,
    /// 1-based line.
    pub line: u32,
    /// 0-based column, in characters, where the old identifier starts.
    pub column: u32,
    pub old: String,
    pub new: String,
}

/// Why a request was not carried out.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Refusal {
    /// A word the system does not know.
    UnknownWord { word: String },
    /// Every word is known but the sentence shape is not.
    UnknownShape,
    /// The request names something that does not exist in the source.
    NoSuchName { name: String },
    /// The request asks for something the named thing cannot have or do,
    /// such as the fields of a function.
    WrongKind,
    /// Asked to apply or decline a proposal when none was waiting.
    NothingPending,
    /// A reference has several candidates and nothing decides between them.
    Ambiguous,
    /// Understood, but the system cannot do it.
    Unsupported,
}

/// A conversational act that is not an answer about the source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "act", rename_all = "snake_case")]
pub enum MetaAct {
    /// Displayed the conversation context.
    ShowContext,
    /// Removed something from the context.
    Forget,
    /// Explained why an earlier action was taken; `turn` is the 0-based index
    /// of the turn that asked for that action.
    ExplainAction { turn: u32 },
    /// Took note of a fact the user stated.
    Noted,
    /// The user declined a proposal; nothing was changed.
    Declined,
}

/// What the system did with one turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// A set of items: the answer to "which", "what" and "where" questions.
    Items { items: BTreeSet<ItemKey> },
    /// A number: the answer to "how many" questions.
    Count { count: u64 },
    /// A description of one item: the answer to "what does X do" or "what is X".
    Describe { item: ItemKey },
    /// A proposed edit, as the full set of replacements it would make.
    Edit { changes: BTreeSet<Change> },
    /// A question back to the user: which of these was meant.
    Ask { candidates: BTreeSet<ItemKey> },
    Refuse {
        #[serde(flatten)]
        refusal: Refusal,
    },
    Meta {
        #[serde(flatten)]
        act: MetaAct,
    },
    /// What a check found: `check` is `build`, `test` or `lint`; `passed`
    /// when `problems` is empty.
    Checked {
        check: String,
        passed: bool,
        problems: BTreeSet<ItemKey>,
    },
    /// Changes offered to make a check pass (empty if the system knows none).
    Remedy {
        check: String,
        changes: BTreeSet<Change>,
    },
    /// Expected only, never given: any changes that make `check` pass on
    /// the task's source are right. The scorer applies the given remedy to
    /// a scratch copy and runs the check to judge it.
    Repaired { check: String },
}

/// How far the system got before it stopped. Used to price refusals: a
/// request that got further costs less.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    /// Stopped at a word it does not know.
    Words,
    /// Knew the words, but not the sentence.
    Sentence,
    /// Understood the sentence, but could not resolve what it refers to.
    Reference,
    /// Understood and resolved, but could not carry it out.
    Action,
    /// Carried it out.
    Done,
}

/// The full record of one turn, as written by `glassmind eval`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub turn_id: String,
    pub response: Response,
    pub stage: Stage,
    /// How many input tokens were understood before stopping, out of `tokens`.
    pub understood: u32,
    pub tokens: u32,
    /// IDs of every rule that took part in the chosen reading and the reply.
    pub rules: Vec<String>,
    /// The reply as the user would see it.
    pub text: String,
    /// What the system says for the reading it chose, before names were
    /// looked up: the canonical form of the request, for learning how
    /// people's phrasing differs from it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<String>,
}

/// One conversation as given to `glassmind eval`: sources and user turns only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationInput {
    pub id: String,
    /// Source names; `glassmind eval --sources <dir>` finds each under `<dir>/<name>`.
    pub sources: Vec<String>,
    pub turns: Vec<TurnInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnInput {
    pub id: String,
    pub text: String,
}

impl Response {
    /// The response kind's name, used to group results.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Response::Items { .. } => "items",
            Response::Count { .. } => "count",
            Response::Describe { .. } => "describe",
            Response::Edit { .. } => "edit",
            Response::Ask { .. } => "ask",
            Response::Refuse { .. } => "refuse",
            Response::Meta { .. } => "meta",
            Response::Checked { .. } => "checked",
            Response::Remedy { .. } => "remedy",
            Response::Repaired { .. } => "repaired",
        }
    }
}
