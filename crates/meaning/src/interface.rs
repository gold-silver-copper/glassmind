//! What a language and a world must provide, stated in meanings.
//!
//! The dialogue layer (`glassmind-mind`) talks to any language and any world
//! through these traits, so it never depends on either. A language turns text
//! into meanings and meanings into text; a world answers meanings.

use glassmind_rules::RuleInfo;

use crate::{Name, Ontology, Question, Reply, Trouble, Utterance};

/// One stretch of the user's input that the language treats as a token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenSpan {
    /// Byte offsets into the input.
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// One way of understanding the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading<O: Ontology> {
    pub meaning: Utterance<O>,
    /// Every rule that built this reading.
    pub rules: Vec<RuleInfo>,
    /// The parse tree, written out for `--explain`.
    pub parse: String,
}

/// What the language made of the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Understanding<O: Ontology> {
    /// Every reading the grammar allows; never empty.
    Readings(Vec<Reading<O>>),
    /// No reading; `trouble` says where and why.
    Failed { trouble: Trouble<O> },
}

/// A reply put into words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spoken {
    pub text: String,
    /// Every rule used to build the text.
    pub rules: Vec<RuleInfo>,
}

/// Something that may come next in an unfinished input.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Next {
    /// This exact word.
    Word(String),
    /// The name of something in the world.
    Name,
    /// A number.
    Number,
}

/// A human language.
pub trait Language<O: Ontology> {
    /// Splits input into tokens, the only place raw text is read.
    fn tokens(&self, text: &str) -> Vec<TokenSpan>;
    /// Every reading of the input.
    fn understand(&self, text: &str) -> Understanding<O>;
    /// Puts a reply into words. `None` if the language has no way to say it.
    fn say(&self, reply: &Reply<O>) -> Option<Spoken>;
    /// Says what may follow the complete tokens of `prefix`.
    fn next(&self, prefix: &str) -> Vec<Next>;
}

/// A world: the data glassmind can see.
pub trait World<O: Ontology> {
    /// Every item with this name. Pushes the world rules it used onto `fired`.
    fn lookup(&self, name: &Name, fired: &mut Vec<RuleInfo>) -> Vec<O::Item>;
    /// Answers a question whose things are all resolved to items. Pushes the
    /// world rules it used onto `fired`.
    fn answer(
        &self,
        question: &Question<O>,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<Reply<O>, Trouble<O>>;
    /// The names of all items, for completion, sorted and deduplicated.
    fn names(&self) -> Vec<String>;
}
