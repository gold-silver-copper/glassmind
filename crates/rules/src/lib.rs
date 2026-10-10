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

/// The mark for knowledge written by hand after the loop started (fixes to
/// the engine's own behavior, not grown by a loop round).
pub const HAND: u32 = u32::MAX;

/// Says who added a rule: `seed`, `round N`, or `by hand`.
pub fn provenance(round: u32) -> String {
    match round {
        SEED => "seed".to_owned(),
        HAND => "by hand".to_owned(),
        n => format!("round {n}"),
    }
}

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
    pub fn without(ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Switches {
            disabled: ids.into_iter().map(Into::into).collect(),
        }
    }

    pub fn on(&self, id: &str) -> bool {
        !self.disabled.contains(id)
    }
}

/// Every rule ID that appears more than once in `rules`, for start-up checks.
pub fn duplicate_ids<'a>(rules: impl IntoIterator<Item = &'a RuleInfo>) -> Vec<&'static str> {
    let mut seen = BTreeSet::new();
    let duplicates: BTreeSet<_> = rules
        .into_iter()
        .map(|r| r.id)
        .filter(|id| !seen.insert(*id))
        .collect();
    duplicates.into_iter().collect()
}
