//! The request-category tagger: reading a request whose words, as typed and
//! through the normalizer, fit no sentence the grammar knows ("callers
//! blob_reader pls"). Its learned model (`models/intents.tsv`, from turns
//! answered right) says which canonical request the words most likely
//! stand for; the names typed fill its slots in order. The grammar still
//! decides: only a filled template that parses is read, and it is never
//! answered without asking unless the model is sure.

use std::collections::{BTreeMap, BTreeSet};
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

/// The word bridge (`models/synonyms.tsv`, from Wikidata's programming
/// concepts and WordNet; see `tools/datagen/wordnet_bridge.py`): words the
/// model never saw, and the words it knows that mean the same. Read once,
/// or for experiments from the file GLASSMIND_SYNONYMS names.
fn bridge() -> &'static BTreeMap<String, Vec<String>> {
    static BRIDGE: OnceLock<BTreeMap<String, Vec<String>>> = OnceLock::new();
    BRIDGE.get_or_init(|| {
        let text =
            std::env::var_os("GLASSMIND_SYNONYMS").and_then(|p| std::fs::read_to_string(p).ok());
        text.as_deref()
            .unwrap_or(include_str!("../models/synonyms.tsv"))
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| l.split_once('\t'))
            .map(|(w, to)| (w.to_owned(), to.split(',').map(str::to_owned).collect()))
            .collect()
    })
}

/// What stands for a name in the model's word pairs.
pub const NAME: &str = "<n>";

/// The names in `tokens` (code, numbers, and plain words the grammar does
/// not know), in order, and the other words, with each pair of neighbours
/// ("does <n>", "<n> call": the order says which way a request goes) and
/// the first word after `^`. An unknown word one slip from a word the model
/// `m` knows is a mistyped word, not a name: read as that word when there is
/// one such word, and left out when there are several.
pub fn split(
    tokens: &[Token],
    known: &dyn Fn(&str) -> bool,
    m: &IntentModel,
) -> (Vec<Token>, BTreeSet<String>) {
    let mut names = Vec::new();
    let mut words = BTreeSet::new();
    let mut sequence = vec!["^".to_owned()];
    for t in tokens {
        let bridged: Vec<&String> = match t.shape {
            Shape::Word if !m.knows(&t.norm) => bridge()
                .get(&t.norm)
                .into_iter()
                .flatten()
                .filter(|w| m.knows(w))
                .collect(),
            _ => Vec::new(),
        };
        let word = match t.shape {
            Shape::Code | Shape::Number => {
                names.push(t.clone());
                Some(NAME.to_owned())
            }
            // A word the model never saw that means one it knows
            // ("invoked": "called").
            Shape::Word if !bridged.is_empty() => {
                words.extend(bridged.iter().map(|w| (*w).clone()));
                bridged.first().map(|w| (*w).clone())
            }
            Shape::Word if !known(&t.norm) => {
                let near: Vec<&String> = if t.norm.chars().count() >= 4 {
                    m.seen
                        .keys()
                        .filter(|w| !w.contains(' '))
                        .filter(|w| crate::typos::slips(&t.norm, w, 1).is_some())
                        .collect()
                } else {
                    Vec::new()
                };
                match near.as_slice() {
                    [] => {
                        names.push(t.clone());
                        Some(NAME.to_owned())
                    }
                    [w] => Some((*w).clone()),
                    _ => None,
                }
            }
            Shape::Word => Some(t.norm.clone()),
            Shape::Punct => None,
        };
        if let Some(word) = word {
            if word != NAME {
                words.insert(word.clone());
            }
            sequence.push(word);
        }
    }
    for pair in sequence.windows(2) {
        if let [a, b] = pair {
            words.insert(format!("{a} {b}"));
        }
    }
    (names, words)
}

/// The most ways of choosing names [`choices`] gives.
const MOST_CHOICES: usize = 40;

/// The ways of choosing which of `names` are the request's names, up to
/// `most` of them, in the order typed, each with the words left out: code
/// and numbers are always names; a plain word the grammar does not know may
/// be a name ("fields of Found") or a word it was never taught.
pub fn choices(names: &[Token], most: usize) -> Vec<(Vec<Token>, Vec<String>)> {
    let fixed = names
        .iter()
        .filter(|t| matches!(t.shape, Shape::Code | Shape::Number))
        .count();
    let mut out = Vec::new();
    // Every subset of the plain words, as a bit mask over their positions.
    let plain: Vec<usize> = (0..names.len())
        .filter(|&i| names.get(i).is_some_and(|t| t.shape == Shape::Word))
        .collect();
    if plain.len() > 16 || fixed > most {
        return out;
    }
    let mut masks: Vec<u32> = (0..1u32 << plain.len())
        .filter(|m| fixed + m.count_ones() as usize <= most)
        .collect();
    // Fewest words left out first.
    masks.sort_by_key(|m| std::cmp::Reverse(m.count_ones()));
    for mask in masks.into_iter().take(MOST_CHOICES) {
        let left: Vec<usize> = plain
            .iter()
            .enumerate()
            .filter(|(bit, _)| mask & (1 << bit) == 0)
            .map(|(_, &i)| i)
            .collect();
        let chosen = names
            .iter()
            .enumerate()
            .filter(|(i, _)| !left.contains(i))
            .map(|(_, t)| t.clone())
            .collect();
        let dropped = left
            .iter()
            .filter_map(|&i| names.get(i))
            .map(|t| t.text.clone())
            .collect();
        out.push((chosen, dropped));
    }
    out
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
