//! Compression rounds: find rules nothing needs, rules that fire for only one
//! request, and near-duplicates, and brief an agent to delete, merge,
//! generalize and rename them.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::evaluate::{self, Data};
use crate::exec::{Run, cargo};
use crate::rounds::usage;
use crate::score::Verdict;
use crate::state::{Baseline, Config, Paths};

/// What ablation found.
#[derive(Debug, Clone, Default)]
pub struct Found {
    /// Rules that can be switched off with nothing getting worse.
    pub removable: Vec<String>,
    /// Rules that took part in exactly one right answer (memorizing).
    pub single_use: Vec<(String, String)>,
    /// Every rule: ID, round, uses in right answers, and what it reads.
    pub table: Vec<(String, u32, usize, String)>,
}

/// Whether switching rules off left every training turn at most as costly
/// and the held-out cost no higher.
fn no_worse(data: &Data, base: &Baseline, ev: &evaluate::Evaluated) -> bool {
    train_no_worse(data, base, &ev.train)
        && ev.scores.heldout_cost <= base.scores.heldout_cost + 1e-9
}

/// Whether every training turn is at most as costly as in the baseline.
fn train_no_worse(data: &Data, base: &Baseline, train: &[glassmind_answer::Outcome]) -> bool {
    let before = evaluate::verdicts(&data.train, &base.train);
    let after = evaluate::verdicts(&data.train, train);
    let worse = before.iter().any(|(id, (vb, cb))| {
        after.get(id).is_some_and(|(va, ca)| {
            *ca > cb + 1e-9 || (*va == Verdict::Wrong && *vb != Verdict::Wrong)
        })
    });
    !worse
}

/// Builds the accepted version in `dir` and switches rules off one by one.
pub fn find(
    config: &Config,
    paths: &Paths,
    data: &Data,
    base: &Baseline,
    dir: &Path,
    target: &Path,
) -> Result<Found, String> {
    let out = cargo(dir, target, &["build", "--release", "-p", "glassmind"])?;
    if !out.ok {
        return Err(format!("build failed: {}", out.stderr));
    }
    let listed = Run::new(
        &evaluate::binary(target).to_string_lossy(),
        &["rules".to_owned(), "--detail".to_owned()],
        dir,
    )
    .go()?;
    let used = usage(data, base);
    let mut found = Found::default();
    for line in listed.stdout.lines() {
        let mut parts = line.splitn(3, '\t');
        let (Some(id), Some(round)) = (parts.next(), parts.next()) else {
            continue;
        };
        let detail = parts.next().unwrap_or_default().to_owned();
        let uses = used.get(id).map_or(0, Vec::len);
        found
            .table
            .push((id.to_owned(), round.parse().unwrap_or(0), uses, detail));
        if uses == 1
            && let Some(turn) = used.get(id).and_then(|v| v.first())
        {
            found.single_use.push((id.to_owned(), turn.clone()));
        }
    }
    // Only rules used by at most one right answer can possibly be removed.
    let candidates: Vec<String> = found
        .table
        .iter()
        .filter(|(_, _, uses, _)| *uses <= 1)
        .map(|(id, ..)| id.clone())
        .collect();
    // Each rule is tried on its own, eight evaluations at a time.
    for chunk in candidates.chunks(8) {
        let results: Vec<Result<bool, String>> = std::thread::scope(|s| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|id| {
                    // Training only here; the held-out set is checked on the
                    // rules that pass, together, below.
                    s.spawn(move || {
                        evaluate::eval_set(
                            config,
                            paths,
                            dir,
                            target,
                            &config.sources,
                            &config.train,
                            std::slice::from_ref(id),
                        )
                        .map(|train| train_no_worse(data, base, &train))
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| {
                    h.join()
                        .unwrap_or_else(|_| Err("ablation thread panicked".into()))
                })
                .collect()
        });
        for (id, ok) in chunk.iter().zip(results) {
            if ok? {
                found.removable.push(id.clone());
            }
        }
    }
    // Check that they can all go together; if not, keep a greedy subset.
    if found.removable.len() > 1 {
        let ev = evaluate::evaluate(config, paths, data, dir, target, &found.removable)?;
        if !no_worse(data, base, &ev) {
            // Add them sixteen at a time, and one at a time only within a
            // group of sixteen that does not pass as a whole.
            let mut keep: Vec<String> = Vec::new();
            for chunk in found.removable.chunks(16) {
                let mut trial = keep.clone();
                trial.extend(chunk.iter().cloned());
                let ev = evaluate::evaluate(config, paths, data, dir, target, &trial)?;
                if no_worse(data, base, &ev) {
                    keep = trial;
                    continue;
                }
                for id in chunk {
                    let mut trial = keep.clone();
                    trial.push(id.clone());
                    let ev = evaluate::evaluate(config, paths, data, dir, target, &trial)?;
                    if no_worse(data, base, &ev) {
                        keep = trial;
                    }
                }
            }
            found.removable = keep;
        }
    }
    Ok(found)
}

pub fn write_brief(
    slot: &Path,
    config: &Config,
    round: u32,
    found: &Found,
    base: &Baseline,
) -> Result<PathBuf, String> {
    let mut b = String::new();
    let _ = writeln!(b, "# Loop brief: round {round}, compression\n");
    let _ = writeln!(
        b,
        "The codebase is not append-only. Make glassmind smaller and clearer without making it worse. \
         The loop accepts your change only if the training cost plus {} per syntax token of mind code goes \
         down, no training turn that was right or honestly refused becomes wrong, no more replies lose their \
         words, and the hidden held-out set does not get worse. Size counts syntax tokens of non-test code \
         (identifiers count 1 whatever their length; comments and docs are free), so good names and docs \
         cost nothing. Prefer one general rule over several phrase-specific ones.\n",
        config.size_price
    );
    let _ = writeln!(
        b,
        "## 1. Delete these rules\n\nSwitching each of these off (and all of them together) made nothing \
         worse on training or held-out data. Delete them from the source, and anything only they use:\n"
    );
    for id in &found.removable {
        let _ = writeln!(b, "- `{id}`");
    }
    if found.removable.is_empty() {
        let _ = writeln!(b, "(none this round)");
    }
    let _ = writeln!(
        b,
        "\n## 2. Generalize or merge\n\nThese rules took part in exactly one right answer, which may mean \
         they memorize one request. Where a more general rule (or an existing one, slightly widened) would \
         cover the same turn and others like it, replace them. Leave them if they are genuinely general.\n"
    );
    for (id, turn) in &found.single_use {
        let _ = writeln!(b, "- `{id}` (only in turn `{turn}`)");
    }
    let _ = writeln!(
        b,
        "\n## 3. Near-duplicates and names\n\nAll rules, with their round and how many right answers used them. \
         Merge rules that differ only in a detail a shared rule could carry; rename rules and types whose \
         names do not say what they do; write doc comments from actual uses where missing. Rule IDs of rules \
         you keep must stay the same (renaming an ID is fine only when you merge rules).\n"
    );
    let _ = writeln!(b, "| id | round | uses | reads |\n|---|---|---|---|");
    for (id, r, uses, detail) in &found.table {
        let _ = writeln!(
            b,
            "| `{id}` | {r} | {uses} | {} |",
            detail.replace('|', "\\|")
        );
    }
    let _ = writeln!(
        b,
        "\nThe training data is at `DATA={}` (`train-inputs.jsonl`, `train-labels.jsonl`, \
         `baseline-outcomes.jsonl`) with `SOURCES={}`; commands to check your work are in `docs/growing.md`.",
        slot.join("data").display(),
        config.sources.display()
    );
    let data = slot.join("data");
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    for name in ["inputs.jsonl", "labels.jsonl"] {
        std::fs::copy(config.train.join(name), data.join(format!("train-{name}")))
            .map_err(|e| e.to_string())?;
    }
    let outcomes: Vec<String> = base
        .train
        .iter()
        .filter_map(|o| serde_json::to_string(o).ok())
        .collect();
    std::fs::write(
        data.join("baseline-outcomes.jsonl"),
        outcomes.join("\n") + "\n",
    )
    .map_err(|e| e.to_string())?;
    let path = slot.join("brief.md");
    std::fs::write(&path, b).map_err(|e| e.to_string())?;
    Ok(path)
}
