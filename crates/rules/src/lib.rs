//! What every piece of glassmind's knowledge has in common: a stable ID, the
//! loop round that added it, and a switch to turn it off at runtime.
//!
//! Grammar rules, words, reference preferences, world relations and reply
//! patterns are all rules in this sense. The loop's compression rounds switch
//! rules off one at a time by ID to find the ones nothing needs, and
//! `--explain` shows the round of every rule that fired.

use std::collections::BTreeSet;

/// The loop round that marks hand-written seed knowledge.
pub const SEED: u32 = 0;

/// Identity and provenance of one rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleInfo {
    /// Stable, unique, dotted ID such as `en.question.what_verb`.
    pub id: &'static str,
    /// The loop round that added the rule; [`SEED`] for the hand-written seed.
    pub round: u32,
}

impl RuleInfo {
    pub const fn new(id: &'static str, round: u32) -> Self {
        RuleInfo { id, round }
    }
}

/// Which rules are switched off for this run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Switches {
    disabled: BTreeSet<String>,
}

impl Switches {
    /// Every rule on.
    pub fn all_on() -> Self {
        Switches::default()
    }

    /// Every rule on except those listed.
    pub fn without<I, S>(ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Switches {
            disabled: ids.into_iter().map(Into::into).collect(),
        }
    }

    pub fn on(&self, id: &str) -> bool {
        !self.disabled.contains(id)
    }

    pub fn disabled(&self) -> impl Iterator<Item = &str> {
        self.disabled.iter().map(String::as_str)
    }
}

/// Every rule ID that appears more than once in `rules`, for start-up checks.
pub fn duplicate_ids<'a>(rules: impl IntoIterator<Item = &'a RuleInfo>) -> Vec<&'static str> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for rule in rules {
        if !seen.insert(rule.id) {
            duplicates.insert(rule.id);
        }
    }
    duplicates.into_iter().collect()
}
