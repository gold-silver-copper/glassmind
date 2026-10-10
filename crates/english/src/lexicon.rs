//! How the English grammar reads tokens that are not fixed words: names,
//! numbers, and the items and quotes the system speaks.

use std::collections::BTreeSet;
use std::marker::PhantomData;

use glassmind_grammar::Lexicon;
use glassmind_meaning::{Labelled, Name, Ontology};

use crate::sem::Sem;
use crate::tokens::{Shape, Token};

/// Token class: a name the user typed, to be looked up in the world.
pub const NAME: &str = "name";
/// Token class: a name written as code, in backticks or shaped like an
/// identifier (`spawn_wave`). A plain word may be English the system does
/// not know, so it is never one: a new name must not be a guess.
pub const CODE: &str = "code";
/// Token class: a number.
pub const NUMBER: &str = "number";
/// Token class: any word, name or number in a remark, whose content the
/// system need not understand. Punctuation ends a remark, so it is not one.
pub const TEXT: &str = "text";
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
    pub ontology: PhantomData<O>,
}

impl<O: Ontology> Lexicon<Sem<O>> for EnglishLexicon<O> {
    type Token = Token;

    fn norm<'t>(&self, token: &'t Token) -> &'t str {
        &token.norm
    }

    fn read_class(&self, class: &'static str, token: &Token) -> Vec<Sem<O>> {
        match (class, token.shape) {
            (NAME | CODE, Shape::Code) => vec![Sem::Name(Name(token.text.clone()))],
            (NAME, Shape::Word) if !self.known.contains(token.norm.as_str()) => {
                vec![Sem::Name(Name(token.text.clone()))]
            }
            (TEXT, shape) if shape != Shape::Punct => vec![Sem::Quote(token.text.clone())],
            (NUMBER, Shape::Number) => token.text.parse().map(Sem::Count).into_iter().collect(),
            _ => Vec::new(),
        }
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
