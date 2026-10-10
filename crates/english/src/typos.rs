//! Reading mistyped words: a plain word the grammar does not know may be a
//! known word typed with one slip (two for long words): a letter dropped,
//! added, swapped with its neighbour, or replaced. The parser only tries
//! these when nothing reads as typed, and keeps the readings with fewest
//! slips, so a correction never wins over a reading that needs none.
//! Code and quoted names are never corrected.

use glassmind_rules::{HAND, RuleInfo};

/// Reading a mistyped word as a known word a slip or two away.
pub const TYPOS: RuleInfo = RuleInfo::new("en.typos.edit", HAND);

/// How many slips a word of this many letters may have.
fn allowed(len: usize) -> usize {
    match len {
        0..=2 => 0,
        3..=6 => 1,
        _ => 2,
    }
}

/// Slips between two words (letters dropped, added, replaced, or two
/// neighbours swapped), or `None` if more than `max`.
pub fn slips(a: &str, b: &str, max: usize) -> Option<usize> {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > max {
        return None;
    }
    // Optimal string alignment distance, three rows at a time.
    let mut before: Vec<usize> = Vec::new();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut row = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            let mut best = (prev.get(j + 1)? + 1)
                .min(row.get(j)? + 1)
                .min(prev.get(j)? + cost);
            if i > 0 && j > 0 && a.get(i - 1) == Some(cb) && b.get(j - 1) == Some(ca) {
                best = best.min(before.get(j - 1)? + 1);
            }
            *row.get_mut(j + 1)? = best;
        }
        if row.iter().min().is_some_and(|&m| m > max) {
            return None;
        }
        before = std::mem::replace(&mut prev, row);
    }
    prev.last().copied().filter(|&d| d <= max)
}

/// The known words `word` may be a mistyping of, with their slips.
pub fn candidates(
    word: &str,
    known: impl Iterator<Item = &'static str>,
) -> Vec<(&'static str, u32)> {
    let max = allowed(word.chars().count());
    if max == 0 {
        return Vec::new();
    }
    known
        .filter(|k| !k.contains(' ') && k.chars().count() >= 3)
        .filter_map(|k| slips(word, k, max).map(|d| (k, u32::try_from(d).unwrap_or(u32::MAX))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_slips() {
        assert_eq!(slips("wher", "where", 1), Some(1));
        assert_eq!(slips("teh", "the", 1), Some(1));
        assert_eq!(slips("defiend", "defined", 2), Some(1));
        assert_eq!(slips("callers", "callees", 1), Some(1));
        assert_eq!(slips("struct", "function", 2), None);
    }
}
