//! The request-category tagger: reading a request whose words, as typed and
//! through the normalizer, fit no sentence the grammar knows ("callers
//! blob_reader pls"). Its learned model (`models/intents.tsv`, from turns
//! answered right) says which canonical request the words most likely
//! stand for; the names typed fill its slots in order. The grammar still
//! decides: only a filled template that parses is read, and it is never
//! answered without asking unless the model is sure.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use glassmind_models::IntentModel;
use glassmind_models::intents::slot;
use glassmind_rules::{HAND, RuleInfo};

use crate::tokens::{Shape, Token};

/// Reading a request through the request-category tagger.
pub const INTENTS: RuleInfo = RuleInfo::new("en.intents.template", HAND);

/// The learned model, read once: the one built in, or for experiments the
/// file GLASSMIND_INTENTS names.
pub fn model() -> &'static IntentModel {
    static MODEL: OnceLock<IntentModel> = OnceLock::new();
    MODEL.get_or_init(|| {
        let text =
            std::env::var_os("GLASSMIND_INTENTS").and_then(|p| std::fs::read_to_string(p).ok());
        IntentModel::load(
            text.as_deref()
                .unwrap_or(include_str!("../models/intents.tsv")),
        )
    })
}

/// The names in `tokens` (code, numbers, and plain words the grammar does
/// not know), in order, and the other words. An unknown word one slip from
/// a word the model `m` knows is a mistyped word, not a name: read as that
/// word when there is one such word, and left out when there are several.
pub fn split(
    tokens: &[Token],
    known: &dyn Fn(&str) -> bool,
    m: &IntentModel,
) -> (Vec<Token>, BTreeSet<String>) {
    let mut names = Vec::new();
    let mut words = BTreeSet::new();
    for t in tokens {
        match t.shape {
            Shape::Code | Shape::Number => names.push(t.clone()),
            Shape::Word if !known(&t.norm) => {
                let near: Vec<&String> = if t.norm.chars().count() >= 4 {
                    m.seen
                        .keys()
                        .filter(|w| crate::typos::slips(&t.norm, w, 1).is_some())
                        .collect()
                } else {
                    Vec::new()
                };
                match near.as_slice() {
                    [] => names.push(t.clone()),
                    [w] => {
                        words.insert((*w).clone());
                    }
                    _ => {}
                }
            }
            Shape::Word => {
                words.insert(t.norm.clone());
            }
            Shape::Punct => {}
        }
    }
    (names, words)
}

/// The template's text with the names typed in its slots.
pub fn fill(template: &str, names: &[Token]) -> String {
    template
        .split(' ')
        .map(|w| {
            names
                .iter()
                .enumerate()
                .find(|(i, _)| slot(*i) == w)
                .map_or_else(
                    || w.to_owned(),
                    |(_, t)| match t.shape {
                        Shape::Code => format!("`{}`", t.text),
                        _ => t.text.clone(),
                    },
                )
        })
        .collect::<Vec<_>>()
        .join(" ")
}
