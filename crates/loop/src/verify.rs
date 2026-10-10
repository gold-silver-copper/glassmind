//! Judging remedies by running them: for a turn whose expected outcome is
//! "repaired" (any change that makes a check pass), the changes glassmind
//! offered are applied to a scratch copy of the task's source and the
//! check is run. A remedy that works is right; one that does not is a
//! broken edit, which costs like a wrong answer.

use std::collections::BTreeMap;
use std::path::Path;

use glassmind_answer::{Outcome, Response};
use glassmind_world_rust::checks::{self, RustChange, RustCheck};

use crate::score::Label;

fn check_of(name: &str) -> Option<RustCheck> {
    match name {
        "build" => Some(RustCheck::Build),
        "test" => Some(RustCheck::Test),
        "lint" => Some(RustCheck::Lint),
        _ => None,
    }
}

/// Whether applying `changes` to the source makes `check` pass.
fn works(
    places: &checks::Places,
    sources: &Path,
    source: &str,
    check: RustCheck,
    changes: &[RustChange],
) -> bool {
    let scratch = checks::scratch(&format!("verify-{source}"));
    let roots = BTreeMap::from([(source.to_owned(), scratch.clone())]);
    let passed = checks::copy_tree(&sources.join(source), &scratch)
        .and_then(|()| checks::apply(changes, &roots))
        .and_then(|_| checks::run_in(places, check, source, &scratch))
        .is_ok_and(|found| found.is_empty());
    let _ = std::fs::remove_dir_all(&scratch);
    passed
}

/// The outcomes, with every remedy for a "repaired" turn replaced by
/// `Repaired` when it works (so it matches the label) and left as it is
/// when it does not (so it is judged wrong).
pub fn remedies(
    places: &checks::Places,
    labels: &[Label],
    outcomes: Vec<Outcome>,
    sources: &Path,
) -> Vec<Outcome> {
    let expected: BTreeMap<&str, &Label> = labels
        .iter()
        .filter(|l| matches!(l.expected, Response::Repaired { .. }))
        .map(|l| (l.turn_id.as_str(), l))
        .collect();
    outcomes
        .into_iter()
        .map(|mut o| {
            let Some(label) = expected.get(o.turn_id.as_str()) else {
                return o;
            };
            if let Response::Remedy { check, changes } = &o.response
                && !changes.is_empty()
                && let Some(c) = check_of(check)
            {
                let changes: Vec<RustChange> = changes
                    .iter()
                    .map(|c| RustChange {
                        source: c.source.clone(),
                        file: c.file.clone(),
                        line: c.line,
                        column: c.column,
                        old: c.old.clone(),
                        new: c.new.clone(),
                    })
                    .collect();
                if works(places, sources, &label.source, c, &changes) {
                    o.response = Response::Repaired {
                        check: check.clone(),
                    };
                }
            }
            o
        })
        .collect()
}
