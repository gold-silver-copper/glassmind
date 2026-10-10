//! A request-category model: which canonical request a bag of words most
//! likely stands for, when the words as typed do not parse at all
//! ("callers blob_reader pls", "fields Found what r they").
//!
//! Learned from turns answered right: the words typed, against the
//! canonical form of the same request with its names numbered in the order
//! they were typed (`which functions call <0>`), a template. It is a
//! Bernoulli naive Bayes classifier over which words appear, so a word the
//! template's requests always have and this one lacks counts against it
//! too. It only proposes: the template, with the names put back, must
//! still parse, and the names must still resolve.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// The slot for the `i`th name typed.
pub fn slot(i: usize) -> String {
    format!("<{i}>")
}

/// Counts of templates and of the words that came with each.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IntentModel {
    /// How many examples had each template (its tokens, space-separated).
    pub templates: BTreeMap<String, u32>,
    /// How many examples of a template had a word (each word once).
    pub words: BTreeMap<(String, String), u32>,
    /// How many examples had each word, over all templates.
    pub seen: BTreeMap<String, u32>,
    pub examples: u32,
    /// The least probability a template may have to be answered without
    /// asking, and to be offered as "did you mean".
    pub sure: f64,
    pub offer: f64,
}

/// Words seen fewer times than this are left out of the model.
const LEAST_SEEN: u32 = 2;

/// Smoothing for a word's chance in a template.
const ALPHA: f64 = 0.5;

/// How many names a template takes.
pub fn slots_of(template: &str) -> usize {
    template
        .split(' ')
        .filter(|t| t.starts_with('<') && t.ends_with('>') && t.len() > 2)
        .collect::<BTreeSet<_>>()
        .len()
}

impl IntentModel {
    /// Learns from (words typed, template) examples.
    pub fn learn(examples: &[(BTreeSet<String>, String)]) -> IntentModel {
        let mut m = IntentModel::default();
        for (_, template) in examples {
            *m.templates.entry(template.clone()).or_default() += 1;
            m.examples += 1;
        }
        let mut seen: BTreeMap<String, u32> = BTreeMap::new();
        for (words, _) in examples {
            for w in words {
                *seen.entry(w.clone()).or_default() += 1;
            }
        }
        m.seen = seen.into_iter().filter(|(_, n)| *n >= LEAST_SEEN).collect();
        for (words, template) in examples {
            for w in words.iter().filter(|w| m.seen.contains_key(*w)) {
                *m.words.entry((template.clone(), w.clone())).or_default() += 1;
            }
        }
        m
    }

    /// Whether the model knows a word.
    pub fn knows(&self, word: &str) -> bool {
        self.seen.contains_key(word)
    }

    /// The templates taking `names` names, likeliest first, with their
    /// probability given which known words appear in `words`.
    pub fn rank(&self, words: &BTreeSet<String>, names: usize) -> Vec<(&str, f64)> {
        let present: BTreeSet<&str> = words
            .iter()
            .filter(|w| self.knows(w))
            .map(String::as_str)
            .collect();
        let mut scored: Vec<(&str, f64)> = self
            .templates
            .iter()
            .filter(|(t, _)| slots_of(t) == names)
            .map(|(t, &n)| {
                let n = f64::from(n);
                let mut score = (n / f64::from(self.examples.max(1))).ln();
                for w in self.seen.keys() {
                    let k = self
                        .words
                        .get(&(t.clone(), w.clone()))
                        .copied()
                        .unwrap_or(0);
                    let p = (f64::from(k) + ALPHA) / (n + 2.0 * ALPHA);
                    score += if present.contains(w.as_str()) {
                        p.ln()
                    } else {
                        (1.0 - p).ln()
                    };
                }
                (t.as_str(), score)
            })
            .collect();
        let top = scored.iter().map(|s| s.1).fold(f64::NEG_INFINITY, f64::max);
        let total: f64 = scored.iter().map(|s| (s.1 - top).exp()).sum();
        for s in &mut scored {
            s.1 = (s.1 - top).exp() / total;
        }
        scored.sort_by(|a, b| b.1.total_cmp(&a.1));
        scored
    }

    /// Whether every word in `words` has come with `template` before: a
    /// word it has never come with is a sign of more being asked than the
    /// template says ("what does `x` do, and which functions call it").
    pub fn explains(&self, template: &str, words: &BTreeSet<String>) -> bool {
        words.iter().all(|w| {
            self.words
                .get(&(template.to_owned(), w.clone()))
                .is_some_and(|n| *n > 0)
        })
    }

    /// The model as plain text: one line per count, readable and loadable.
    pub fn dump(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "examples\t{}", self.examples);
        let _ = writeln!(s, "sure\t{}", self.sure);
        let _ = writeln!(s, "offer\t{}", self.offer);
        for (t, n) in &self.templates {
            let _ = writeln!(s, "template\t{t}\t{n}");
        }
        for (w, n) in &self.seen {
            let _ = writeln!(s, "seen\t{w}\t{n}");
        }
        for ((t, w), n) in &self.words {
            let _ = writeln!(s, "word\t{t}\t{w}\t{n}");
        }
        s
    }

    /// Reads a dump; lines it does not understand (and `#` comments) are
    /// skipped.
    pub fn load(text: &str) -> IntentModel {
        let mut m = IntentModel::default();
        for line in text.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            let num = |i: usize| f.get(i).and_then(|n| n.parse::<u32>().ok());
            match (f.first().copied(), f.len()) {
                (Some("examples"), 2) => m.examples = num(1).unwrap_or(0),
                (Some("sure"), 2) => m.sure = f.get(1).and_then(|n| n.parse().ok()).unwrap_or(1.0),
                (Some("offer"), 2) => {
                    m.offer = f.get(1).and_then(|n| n.parse().ok()).unwrap_or(1.0)
                }
                (Some("template"), 3) => {
                    if let (Some(t), Some(n)) = (f.get(1), num(2)) {
                        m.templates.insert((*t).to_owned(), n);
                    }
                }
                (Some("seen"), 3) => {
                    if let (Some(w), Some(n)) = (f.get(1), num(2)) {
                        m.seen.insert((*w).to_owned(), n);
                    }
                }
                (Some("word"), 4) => {
                    if let (Some(t), Some(w), Some(n)) = (f.get(1), f.get(2), num(3)) {
                        m.words.insert(((*t).to_owned(), (*w).to_owned()), n);
                    }
                }
                _ => {}
            }
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bag(s: &str) -> BTreeSet<String> {
        s.split(' ').map(str::to_owned).collect()
    }

    #[test]
    fn learns_which_request_words_stand_for_and_survives_its_dump() {
        let ex = |w: &str, t: &str| (bag(w), t.to_owned());
        let m = IntentModel::learn(&[
            ex("who calls", "which functions call <0>"),
            ex("callers of", "which functions call <0>"),
            ex("callers", "which functions call <0>"),
            ex("how many callers", "how many functions call <0>"),
            ex("how many calls", "how many functions call <0>"),
            ex("fields of", "what fields does <0> have"),
            ex("fields", "what fields does <0> have"),
        ]);
        let best = |w: &str| m.rank(&bag(w), 1).first().map(|r| r.0.to_owned());
        assert_eq!(
            best("callers pls").as_deref(),
            Some("which functions call <0>")
        );
        assert_eq!(
            best("how many callers").as_deref(),
            Some("how many functions call <0>")
        );
        assert!(m.rank(&bag("callers"), 2).is_empty());
        assert!(!m.explains("what fields does <0> have", &bag("fields callers")));
        assert_eq!(IntentModel::load(&m.dump()), m);
    }
}
