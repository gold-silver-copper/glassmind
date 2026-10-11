//! An edit model over token sequences: how people's text differs from a
//! canonical form of the same request. Learned by aligning pairs (what was
//! typed, what the grammar says for the same meaning) and counting the
//! edits: words people add ("pls", "quick"), words they drop ("does",
//! "the"), and words they use in place of others. Its costs are negative
//! log-probabilities, so a likely edit is cheap.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// One difference between typed and canonical text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Edit {
    /// A word the person typed that the canonical form does not have.
    Added(String),
    /// A word the canonical form has, after `after`, that the person left out.
    Dropped { after: String, word: String },
    /// A word typed where the canonical form has another.
    Swapped { typed: String, canonical: String },
}

/// Counts of edits, and of how often each word appeared at all.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditModel {
    pub added: BTreeMap<String, u32>,
    pub dropped: BTreeMap<(String, String), u32>,
    pub swapped: BTreeMap<(String, String), u32>,
    /// How often each word was typed, and how often each appeared in a
    /// canonical form: the denominators.
    pub typed: BTreeMap<String, u32>,
    pub canonical: BTreeMap<String, u32>,
    pub pairs: u32,
    /// The most a reading may cost (see [`EditModel::channel_cost`]) to be
    /// trusted: set from the costs of readings known to be right.
    pub trusted_bits: f64,
    /// The most any one edit in a trusted reading may cost: a reading that
    /// needs one edit people almost never make is not trusted.
    pub trusted_edit_bits: f64,
}

/// The start-of-text marker used as `after` for a word dropped at the start.
pub const START: &str = "^";

/// Aligns typed and canonical tokens with the fewest edits (adding,
/// dropping or swapping one word each), returning the edits.
pub fn align(typed: &[String], canonical: &[String]) -> Vec<Edit> {
    let (n, m) = (typed.len(), canonical.len());
    // cost[i][j]: edits to turn typed[..i] into canonical[..j].
    let mut cost = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in cost.iter_mut().enumerate() {
        if let Some(c) = row.first_mut() {
            *c = i;
        }
    }
    if let Some(row) = cost.first_mut() {
        for (j, c) in row.iter_mut().enumerate() {
            *c = j;
        }
    }
    let at = |cost: &Vec<Vec<usize>>, i: usize, j: usize| {
        cost.get(i)
            .and_then(|r| r.get(j))
            .copied()
            .unwrap_or(usize::MAX / 2)
    };
    for i in 1..=n {
        for j in 1..=m {
            let same = typed.get(i - 1) == canonical.get(j - 1);
            let best = (at(&cost, i - 1, j - 1) + usize::from(!same))
                .min(at(&cost, i - 1, j) + 1)
                .min(at(&cost, i, j - 1) + 1);
            if let Some(c) = cost.get_mut(i).and_then(|r| r.get_mut(j)) {
                *c = best;
            }
        }
    }
    // Walk back from the end, preferring matches, then swaps.
    let mut out = Vec::new();
    let (mut i, mut j) = (n, m);
    while i > 0 || j > 0 {
        let here = at(&cost, i, j);
        let same = i > 0 && j > 0 && typed.get(i - 1) == canonical.get(j - 1);
        if i > 0 && j > 0 && here == at(&cost, i - 1, j - 1) + usize::from(!same) {
            if !same {
                out.push(Edit::Swapped {
                    typed: typed.get(i - 1).cloned().unwrap_or_default(),
                    canonical: canonical.get(j - 1).cloned().unwrap_or_default(),
                });
            }
            i -= 1;
            j -= 1;
        } else if i > 0 && here == at(&cost, i - 1, j) + 1 {
            out.push(Edit::Added(typed.get(i - 1).cloned().unwrap_or_default()));
            i -= 1;
        } else {
            let after = if j >= 2 {
                canonical.get(j - 2).cloned().unwrap_or_default()
            } else {
                START.to_owned()
            };
            out.push(Edit::Dropped {
                after,
                word: canonical.get(j - 1).cloned().unwrap_or_default(),
            });
            j = j.saturating_sub(1);
        }
    }
    out.reverse();
    out
}

fn bump<K: Ord>(map: &mut BTreeMap<K, u32>, key: K) {
    *map.entry(key).or_default() += 1;
}

impl EditModel {
    /// Learns from (typed, canonical) pairs.
    pub fn learn(pairs: &[(Vec<String>, Vec<String>)]) -> EditModel {
        let mut m = EditModel::default();
        for (typed, canonical) in pairs {
            m.pairs += 1;
            for w in typed {
                bump(&mut m.typed, w.clone());
            }
            for w in canonical {
                bump(&mut m.canonical, w.clone());
            }
            for edit in align(typed, canonical) {
                match edit {
                    Edit::Added(w) => bump(&mut m.added, w),
                    Edit::Dropped { after, word } => bump(&mut m.dropped, (after, word)),
                    Edit::Swapped { typed, canonical } => bump(&mut m.swapped, (typed, canonical)),
                }
            }
        }
        m
    }

    /// -log2 of an add-one-smoothed rate `count` out of `of`.
    fn bits(count: u32, of: u32) -> f64 {
        -((f64::from(count) + 0.5) / (f64::from(of) + 1.0)).log2()
    }

    /// The cost of reading a typed `word` as something to leave out.
    pub fn added_cost(&self, word: &str) -> f64 {
        let count = self.added.get(word).copied().unwrap_or(0);
        Self::bits(count, self.typed.get(word).copied().unwrap_or(0))
    }

    /// The cost of supposing `word` was left out after `after`.
    pub fn dropped_cost(&self, after: &str, word: &str) -> f64 {
        let count = self
            .dropped
            .get(&(after.to_owned(), word.to_owned()))
            .copied()
            .unwrap_or(0);
        Self::bits(
            count,
            self.canonical.get(word).copied().unwrap_or(0).max(count),
        )
    }

    /// The cost of reading a typed word as `canonical`.
    pub fn swapped_cost(&self, typed: &str, canonical: &str) -> f64 {
        let count = self
            .swapped
            .get(&(typed.to_owned(), canonical.to_owned()))
            .copied()
            .unwrap_or(0);
        Self::bits(
            count,
            self.typed.get(typed).copied().unwrap_or(0).max(count),
        )
    }

    /// What it costs, in bits, to have typed `typed` when meaning what
    /// `canonical` says: the sum of the costs of the edits that align them.
    /// A reading whose canonical form is far from what was typed is a
    /// guess the words do not support.
    pub fn channel_cost(&self, typed: &[String], canonical: &[String]) -> f64 {
        align(typed, canonical)
            .iter()
            .map(|e| match e {
                Edit::Added(w) => self.added_cost(w),
                Edit::Dropped { after, word } => self.dropped_cost(after, word),
                Edit::Swapped { typed, canonical } => self.swapped_cost(typed, canonical),
            })
            .sum()
    }

    /// The cost of the least likely edit between `typed` and `canonical`.
    pub fn worst_edit(&self, typed: &[String], canonical: &[String]) -> f64 {
        self.edit_costs(typed, canonical)
            .into_iter()
            .fold(0.0, f64::max)
    }

    fn edit_costs(&self, typed: &[String], canonical: &[String]) -> Vec<f64> {
        align(typed, canonical)
            .iter()
            .map(|e| match e {
                Edit::Added(w) => self.added_cost(w),
                Edit::Dropped { after, word } => self.dropped_cost(after, word),
                Edit::Swapped { typed, canonical } => self.swapped_cost(typed, canonical),
            })
            .collect()
    }

    /// Whether a reading whose canonical form is `canonical` is trusted for
    /// what was `typed`: close enough in all, and with no unlikely edit.
    pub fn trusts(&self, typed: &[String], canonical: &[String]) -> bool {
        let costs: Vec<f64> = align(typed, canonical)
            .iter()
            .map(|e| match e {
                Edit::Added(w) => self.added_cost(w),
                Edit::Dropped { after, word } => self.dropped_cost(after, word),
                Edit::Swapped { typed, canonical } => self.swapped_cost(typed, canonical),
            })
            .collect();
        let worst = costs.iter().copied().fold(0.0, f64::max);
        costs.iter().sum::<f64>() <= self.trusted_bits
            && (self.trusted_edit_bits <= 0.0 || worst <= self.trusted_edit_bits)
    }

    /// Words worth supposing left out, most often first: (after, word, count).
    pub fn droppable(&self, min: u32) -> Vec<(&str, &str, u32)> {
        let mut out: Vec<(&str, &str, u32)> = self
            .dropped
            .iter()
            .filter(|(_, n)| **n >= min)
            .map(|((a, w), n)| (a.as_str(), w.as_str(), *n))
            .collect();
        out.sort_by(|a, b| b.2.cmp(&a.2).then(a.cmp(b)));
        out
    }

    /// What a typed word is often written in place of: (canonical, count).
    pub fn swaps_for(&self, typed: &str, min: u32) -> Vec<(&str, u32)> {
        self.swapped
            .range((typed.to_owned(), String::new())..)
            .take_while(|((t, _), _)| t == typed)
            .filter(|(_, n)| **n >= min)
            .map(|((_, c), n)| (c.as_str(), *n))
            .collect()
    }

    /// The model as text: one edit per line with its count, then the word
    /// counts. This is the model; [`EditModel::load`] reads it back.
    pub fn dump(&self) -> String {
        let mut out = format!(
            "pairs\t{}\ntrusted_bits\t{}\ntrusted_edit_bits\t{}\n",
            self.pairs, self.trusted_bits, self.trusted_edit_bits
        );
        for (w, n) in &self.added {
            let _ = writeln!(out, "added\t{w}\t{n}");
        }
        for ((a, w), n) in &self.dropped {
            let _ = writeln!(out, "dropped\t{a}\t{w}\t{n}");
        }
        for ((t, c), n) in &self.swapped {
            let _ = writeln!(out, "swapped\t{t}\t{c}\t{n}");
        }
        for (w, n) in &self.typed {
            let _ = writeln!(out, "typed\t{w}\t{n}");
        }
        for (w, n) in &self.canonical {
            let _ = writeln!(out, "canonical\t{w}\t{n}");
        }
        out
    }

    /// Reads a model written by [`EditModel::dump`]. Lines it does not
    /// understand are skipped.
    pub fn load(text: &str) -> EditModel {
        let mut m = EditModel::default();
        for line in text.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            let n = |s: Option<&&str>| s.and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
            match f.as_slice() {
                ["pairs", c] => m.pairs = n(Some(c)),
                ["trusted_bits", b] => m.trusted_bits = b.parse().unwrap_or(0.0),
                ["trusted_edit_bits", b] => m.trusted_edit_bits = b.parse().unwrap_or(0.0),
                ["added", w, c] => {
                    m.added.insert((*w).to_owned(), n(Some(c)));
                }
                ["dropped", a, w, c] => {
                    m.dropped
                        .insert(((*a).to_owned(), (*w).to_owned()), n(Some(c)));
                }
                ["swapped", t, w, c] => {
                    m.swapped
                        .insert(((*t).to_owned(), (*w).to_owned()), n(Some(c)));
                }
                ["typed", w, c] => {
                    m.typed.insert((*w).to_owned(), n(Some(c)));
                }
                ["canonical", w, c] => {
                    m.canonical.insert((*w).to_owned(), n(Some(c)));
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

    fn words(s: &str) -> Vec<String> {
        s.split(' ').map(str::to_owned).collect()
    }

    #[test]
    fn learns_what_people_add_drop_and_swap() {
        let pairs = [
            (
                words("pls what fields Player have"),
                words("what fields does Player have"),
            ),
            (words("pls who calls tick"), words("what calls tick")),
        ];
        let m = EditModel::learn(&pairs);
        assert_eq!(m.added.get("pls"), Some(&2));
        assert_eq!(
            m.dropped.get(&("fields".to_owned(), "does".to_owned())),
            Some(&1)
        );
        assert_eq!(
            m.swapped.get(&("who".to_owned(), "what".to_owned())),
            Some(&1)
        );
        assert!(m.added_cost("pls") < m.added_cost("Player"));
        assert_eq!(EditModel::load(&m.dump()), m);
    }
}
