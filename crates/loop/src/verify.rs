//! Judging remedies by running them: for a turn whose expected outcome is
//! "repaired" (any change that makes a check pass), the changes glassmind
//! offered are applied to a scratch copy of the task's source and the
//! check is run. A remedy that works is right; one that does not is a
//! broken edit, which costs like a wrong answer.
//!
//! The scorer's own code, apart from everything agents may change: it
//! copies, edits and runs cargo itself, and judges by cargo's exit status
//! alone.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use glassmind_answer::{Change, Outcome, Response};

use crate::exec::Run;
use crate::score::Label;

/// Where verification builds and copies go: places agents cannot see.
#[derive(Debug, Clone)]
pub struct Places {
    pub target: PathBuf,
    pub scratch: PathBuf,
}

impl Places {
    pub fn under(dir: &Path) -> Places {
        let places = Places {
            target: dir.join("target"),
            scratch: dir.join("scratch"),
        };
        let _ = std::fs::create_dir_all(&places.target);
        let _ = std::fs::create_dir_all(&places.scratch);
        places
    }
}

/// The cargo command line for a check, or `None` for an unknown check.
fn check_args(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "build" => Some(&["check", "-j", "4", "--all-targets", "-q"]),
        "test" => Some(&["test", "-j", "4", "-q", "--", "--test-threads", "4"]),
        "lint" => Some(&[
            "clippy",
            "-j",
            "4",
            "--all-targets",
            "-q",
            "--",
            "-D",
            "warnings",
        ]),
        _ => None,
    }
}

/// Copies the files under `from` over `to`, leaving out `target` and
/// hidden entries.
fn overlay(from: &Path, to: &Path) -> Result<(), String> {
    let entries = std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') || name == "target" {
            continue;
        }
        let path = entry.path();
        let into = to.join(&name);
        if path.is_dir() {
            std::fs::create_dir_all(&into).map_err(|e| e.to_string())?;
            overlay(&path, &into)?;
        } else {
            std::fs::copy(&path, &into).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Applies changes to the copy at `root`: each replaces `old` at its line
/// and column with `new`, and all of them must match.
fn apply(root: &Path, changes: &[&Change]) -> Result<(), String> {
    let mut by_file: BTreeMap<&str, Vec<&Change>> = BTreeMap::new();
    for c in changes {
        by_file.entry(c.file.as_str()).or_default().push(c);
    }
    for (file, mut changes) in by_file {
        if file.split('/').any(|p| p == "..") {
            return Err(format!("{file}: outside the source"));
        }
        let path = root.join(file);
        let mut text = std::fs::read_to_string(&path).map_err(|e| format!("{file}: {e}"))?;
        // From the end, so earlier offsets stay right.
        changes.sort_by_key(|c| std::cmp::Reverse((c.line, c.column)));
        for c in changes {
            let line_start = if c.line <= 1 {
                0
            } else {
                text.match_indices('\n')
                    .nth(c.line as usize - 2)
                    .map(|(i, _)| i + 1)
                    .ok_or_else(|| format!("{file}: no line {}", c.line))?
            };
            let at = text
                .get(line_start..)
                .and_then(|rest| {
                    rest.char_indices()
                        .map(|(i, _)| i)
                        .chain(std::iter::once(rest.len()))
                        .nth(c.column as usize)
                })
                .map(|i| line_start + i)
                .ok_or_else(|| format!("{file}:{}: no column {}", c.line, c.column))?;
            if !text.get(at..).is_some_and(|rest| rest.starts_with(&c.old)) {
                return Err(format!("{file}:{}: `{}` is not there", c.line, c.old));
            }
            text.replace_range(at..at + c.old.len(), &c.new);
        }
        std::fs::write(&path, text).map_err(|e| format!("{file}: {e}"))?;
    }
    Ok(())
}

/// Whether applying `changes` to the label's source makes `check` pass
/// (with the label's hidden files copied in, the tests).
fn works(places: &Places, sources: &Path, label: &Label, check: &str, changes: &[&Change]) -> bool {
    let source = label.source.as_str();
    let check = if label.hidden.is_some() {
        "test"
    } else {
        check
    };
    let (Some(args), false) = (check_args(check), source.contains(['/', '.'])) else {
        return false;
    };
    let copy = places.scratch.join(source);
    let _ = std::fs::remove_dir_all(&copy);
    let ready = std::fs::create_dir_all(&copy)
        .map_err(|e| e.to_string())
        .and_then(|()| overlay(&sources.join(source), &copy))
        .and_then(|()| apply(&copy, changes))
        .and_then(|()| match &label.hidden {
            Some(hidden) => overlay(&sources.join(hidden), &copy),
            None => Ok(()),
        });
    let passed = ready.is_ok()
        && Run::new(
            "cargo",
            &args.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>(),
            &copy,
        )
        .env(
            "CARGO_TARGET_DIR",
            &places.target.join(source).to_string_lossy(),
        )
        .env("TMPDIR", &places.scratch.to_string_lossy())
        .timeout(Duration::from_secs(600))
        .memory("6G")
        .go()
        .is_ok_and(|out| out.ok);
    let _ = std::fs::remove_dir_all(&copy);
    passed
}

/// The outcomes, with every remedy (or edit) for a "repaired" turn
/// replaced by `Repaired` when it works (so it matches the label) and left
/// as it is when it does not (so it is judged wrong).
pub fn remedies(
    places: &Places,
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
            let (Some(label), Response::Remedy { changes, .. } | Response::Edit { changes }) =
                (expected.get(o.turn_id.as_str()), &o.response)
            else {
                return o;
            };
            let Response::Repaired { check } = &label.expected else {
                return o;
            };
            let mine: Vec<&Change> = changes
                .iter()
                .filter(|c| c.source == label.source)
                .collect();
            if !changes.is_empty()
                && mine.len() == changes.len()
                && works(places, sources, label, check, &mine)
            {
                o.response = Response::Repaired {
                    check: check.clone(),
                };
            }
            o
        })
        .collect()
}
