//! How the English grammar reads tokens that are not fixed words: names,
//! numbers, and the items and quotes the system speaks.

use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;

use glassmind_grammar::{Lexicon, RuleInfo};
use glassmind_meaning::{Labelled, Name, Ontology, Thing};

use crate::sem::Sem;
use crate::tokens::{Shape, Token};

/// Token class: a name the user typed, to be looked up in the world.
pub const NAME: &str = "name";
/// Token class: a name written as code, in backticks or shaped like an
/// identifier (`spawn_wave`). A plain word may be English the system does
/// not know, so it is never one: a new name must not be a guess.
pub const CODE: &str = "code";
/// Token class: a plain word the system does not know, not written as
/// code: what a name of the user's own ("the history sheet") is made of.
pub const WORD: &str = "word";
/// Token class: a plain word the system does not know, in lower case: a
/// noun of the user's own ("the test helper `x`"), never a type's name,
/// which is capitalized.
pub const LOWER: &str = "lower";
/// Token class: a plain word the system knows, read as the thing going by
/// that name where a name can stand: "a field named calls", "which
/// functions call forget". A word that is a noun phrase by itself ("it")
/// is never one.
pub const SAID: &str = "said";
/// Token class: a plain word the system knows, capitalized the way type
/// names are and more than one letter long, read as the thing going by
/// that name: "a field typed Handle", "whose type is or contains Change".
pub const CAPITALIZED: &str = "capitalized";
/// Token class: a number.
pub const NUMBER: &str = "number";
/// Token class: any word, name or number in a remark, whose content the
/// system need not understand. Punctuation that ends a remark is not one;
/// brackets, written inside a type or a key ("returns Option<u64>", "for
/// the ] and [ thing"), are.
pub const TEXT: &str = "text";
/// Token class: a plain word comparing, as "-er" words do: what a remark
/// after a request says the change makes better ("..., cleaner",
/// "..., shorter").
pub const COMPARATIVE: &str = "comparative";
/// Token class: an item the system speaks of. Never read from input.
pub const ITEM: &str = "item";
/// Token class: where an item the system speaks of is. Never read from input.
pub const PLACE: &str = "place";
/// Token class: a piece of the user's input the system quotes back.
pub const QUOTE: &str = "quote";

pub struct EnglishLexicon<O> {
    /// Every word some rule uses. A plain word that is one of these is read
    /// as that word, never as a name; quote it in backticks to name it.
    pub known: BTreeSet<&'static str>,
    /// Known words that are a noun phrase by themselves, pointing at a
    /// thing ("it", "that"): never read as the name of one.
    pub standing: BTreeSet<&'static str>,
    /// Mistyped words, each read as the word meant (see `spelling`).
    pub spellings: BTreeMap<&'static str, (RuleInfo, &'static str)>,
    /// Whether mistyped words may be read as known words (see `typos`).
    pub typos: bool,
    pub ontology: PhantomData<O>,
}

impl<O> EnglishLexicon<O> {
    /// The spelling rule that corrects `token`, and the word meant.
    pub fn respelled(&self, token: &Token) -> Option<(RuleInfo, &'static str)> {
        (token.shape == Shape::Word)
            .then(|| self.spellings.get(token.norm.as_str()).copied())
            .flatten()
    }
}

impl<O: Ontology> Lexicon<Sem<O>> for EnglishLexicon<O> {
    type Token = Token;

    fn norm<'t>(&self, token: &'t Token) -> &'t str {
        self.respelled(token)
            .map_or(&token.norm, |(_, meant)| meant)
    }

    fn read_class(&self, class: &'static str, token: &Token) -> Vec<Sem<O>> {
        match (class, token.shape) {
            (NAME | CODE, Shape::Code) => vec![Sem::Name(Name(token.text.clone()))],
            (NAME | WORD, Shape::Word) if !self.known.contains(self.norm(token)) => {
                vec![Sem::Name(Name(token.text.clone()))]
            }
            (LOWER, Shape::Word)
                if token.text.starts_with(char::is_lowercase)
                    && !self.known.contains(self.norm(token)) =>
            {
                vec![Sem::Quote(token.text.clone())]
            }
            (SAID, Shape::Word)
                if self.known.contains(self.norm(token))
                    && !self.standing.contains(self.norm(token)) =>
            {
                vec![Sem::Thing(Thing::Named(Name(token.text.clone())))]
            }
            (CAPITALIZED, Shape::Word)
                if token.text.chars().nth(1).is_some()
                    && token.text.starts_with(char::is_uppercase)
                    && self.known.contains(self.norm(token)) =>
            {
                vec![Sem::Thing(Thing::Named(Name(token.text.clone())))]
            }
            (TEXT, shape) if shape != Shape::Punct || token.text.contains(['<', '>', '[', ']']) => {
                vec![Sem::Quote(token.text.clone())]
            }
            (COMPARATIVE, Shape::Word) if token.norm.ends_with("er") => {
                vec![Sem::Quote(token.text.clone())]
            }
            (NUMBER, Shape::Number) => token.text.parse().map(Sem::Count).into_iter().collect(),
            _ => Vec::new(),
        }
    }

    fn corrections(&self, token: &Token) -> Vec<(&'static str, u32)> {
        if !self.typos || token.shape != Shape::Word || self.known.contains(self.norm(token)) {
            return Vec::new();
        }
        crate::typos::candidates(&token.norm, self.known.iter().copied())
    }

    fn speak_class(&self, class: &'static str, value: &Sem<O>) -> Option<String> {
        match (class, value) {
            (NAME | CODE, Sem::Name(Name(n))) => Some(format!("`{n}`")),
            (ITEM, Sem::Item(item)) => Some(format!("`{}`", item.label())),
            (PLACE, Sem::Item(item)) => Some(item.place()),
            (QUOTE, Sem::Quote(q)) => Some(format!("`{q}`")),
            (NUMBER, Sem::Count(n)) => Some(n.to_string()),
            _ => None,
        }
    }
}
