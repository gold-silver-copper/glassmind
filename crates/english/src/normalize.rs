//! The normalizer: reading the way people actually type a request as the
//! way the grammar says it. When nothing parses as typed, its learned edit
//! model (`models/normalizer.tsv`, counted from what people typed against
//! what the grammar says for the same request, on turns answered right)
//! proposes small edits: leave out a word people add ("pls", "quick"),
//! put back a word they drop ("does", "is"), or read a word as the one it
//! usually stands for ("fns" as "functions"). The grammar decides: only
//! edited inputs that parse are kept, the cheapest first. Code names are
//! never edited.

use std::sync::OnceLock;

use glassmind_models::EditModel;
use glassmind_models::edits::START;
use glassmind_rules::{HAND, RuleInfo};

use crate::tokens::{Shape, Token};

/// Reading an input that does not parse as typed through the normalizer's edits.
pub const NORMALIZER: RuleInfo = RuleInfo::new("en.normalizer.edits", HAND);

/// The learned model, read once: the one built in, or for experiments the
/// file GLASSMIND_NORMALIZER names.
pub fn model() -> &'static EditModel {
    static MODEL: OnceLock<EditModel> = OnceLock::new();
    MODEL.get_or_init(|| {
        let text =
            std::env::var_os("GLASSMIND_NORMALIZER").and_then(|p| std::fs::read_to_string(p).ok());
        EditModel::load(
            text.as_deref()
                .unwrap_or(include_str!("../models/normalizer.tsv")),
        )
    })
}

/// How the model sees a token.
pub fn model_word(t: &Token) -> String {
    word(t)
}

fn word(t: &Token) -> String {
    match t.shape {
        Shape::Code => "<name>".to_owned(),
        Shape::Number => "<number>".to_owned(),
        _ => t.norm.clone(),
    }
}

/// The most any one edit of a reading answered without asking may cost, in
/// bits (on the dev set, readings within it were all right).
pub const SURE_EDIT_BITS: f64 = 3.0;

/// The most an edit may cost, in bits, and how often an edit must have been
/// seen to be proposed.
const MOST_BITS: f64 = 5.0;
const LEAST_SEEN: u32 = 3;

/// One edit to the tokens.
#[derive(Debug, Clone)]
enum Step {
    Remove(usize),
    Insert(usize, String),
    Replace(usize, String),
}

fn made(text: &str, at: &Token) -> Token {
    Token {
        text: text.to_owned(),
        norm: text.to_owned(),
        shape: Shape::Word,
        start: at.start,
        end: at.start,
    }
}

/// Applies steps (at positions in the original tokens) in reverse order.
fn apply(tokens: &[Token], steps: &[&Step]) -> Vec<Token> {
    let mut out: Vec<Token> = tokens.to_vec();
    let mut sorted: Vec<&&Step> = steps.iter().collect();
    sorted.sort_by_key(|s| {
        std::cmp::Reverse(match s {
            Step::Remove(i) | Step::Insert(i, _) | Step::Replace(i, _) => *i,
        })
    });
    for step in sorted {
        match step {
            Step::Remove(i) => {
                if *i < out.len() {
                    out.remove(*i);
                }
            }
            Step::Insert(i, w) => {
                let anchor = out.get(*i).or(out.last()).cloned();
                if let Some(anchor) = anchor {
                    out.insert((*i).min(out.len()), made(w, &anchor));
                }
            }
            Step::Replace(i, w) => {
                if let Some(t) = out.get_mut(*i) {
                    *t = Token {
                        text: w.clone(),
                        norm: w.clone(),
                        shape: Shape::Word,
                        start: t.start,
                        end: t.end,
                    };
                }
            }
        }
    }
    out
}

/// Edited versions of `tokens`, cheapest first, with their cost in bits:
/// every single edit the model finds likely, then pairs of the likeliest.
pub fn candidates(tokens: &[Token], known: &dyn Fn(&str) -> bool) -> Vec<(Vec<Token>, f64)> {
    let m = model();
    let mut singles: Vec<(Step, f64)> = Vec::new();
    for (i, t) in tokens.iter().enumerate() {
        let w = word(t);
        if t.shape != Shape::Code && m.added.get(&w).is_some_and(|n| *n >= LEAST_SEEN) {
            singles.push((Step::Remove(i), m.added_cost(&w)));
        }
        for (v, n) in m.swaps_for(&w, LEAST_SEEN) {
            if t.shape != Shape::Code && n >= LEAST_SEEN && known(v) {
                singles.push((Step::Replace(i, v.to_owned()), m.swapped_cost(&w, v)));
            }
        }
    }
    for i in 0..=tokens.len() {
        let after = if i == 0 {
            START.to_owned()
        } else {
            tokens.get(i - 1).map(word).unwrap_or_default()
        };
        for (a, w, _) in m.droppable(LEAST_SEEN) {
            if a == after && known(w) {
                singles.push((Step::Insert(i, w.to_owned()), m.dropped_cost(a, w)));
            }
        }
    }
    singles.retain(|(_, c)| *c <= MOST_BITS);
    singles.sort_by(|a, b| a.1.total_cmp(&b.1));
    singles.truncate(40);
    let mut out: Vec<(Vec<Token>, f64)> = singles
        .iter()
        .map(|(s, c)| (apply(tokens, &[s]), *c))
        .collect();
    let best: Vec<&(Step, f64)> = singles.iter().take(12).collect();
    for (k, (a, ca)) in best.iter().enumerate() {
        for (b, cb) in best.iter().skip(k + 1) {
            let pos = |s: &Step| match s {
                Step::Remove(i) | Step::Insert(i, _) | Step::Replace(i, _) => *i,
            };
            if pos(a) != pos(b) {
                out.push((apply(tokens, &[a, b]), ca + cb));
            }
        }
    }
    out.sort_by(|a, b| a.1.total_cmp(&b.1));
    out
}
