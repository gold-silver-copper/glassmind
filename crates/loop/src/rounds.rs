//! The loop itself: growth and compression rounds, each candidate change
//! applied on top of the accepted version, checked, scored, reviewed, and
//! committed only if it is better.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::agent::{self, AgentRun};
use crate::evaluate::{self, Data, Evaluated};
use crate::exec::git;
use crate::state::{self, Attempts, Baseline, Config, Entry, Paths, Tokens};
use crate::{compress, growth};

/// Paths a loop change may touch; anything else an agent leaves behind
/// (scratch outputs) is not part of the change. The loop itself, the
/// answer format and the fixture labels are the measuring instruments, so
/// agents cannot change them.
const CHANGE_PATHS: &[&str] = &[
    "crates",
    "tools/depcheck",
    "Cargo.toml",
    "Cargo.lock",
    "fixtures",
];
const FIXED_PATHS: &[&str] = &[":!crates/loop", ":!crates/answer", ":!fixtures/data"];

fn log(paths: &Paths, line: &str) {
    use std::io::Write;
    let _ = std::fs::create_dir_all(paths.logs());
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths.logs().join("loop.log"))
    {
        let _ = writeln!(f, "[{}] {line}", state::now());
    }
    eprintln!("{line}");
}

fn worktree(config: &Config, paths: &Paths, name: &str, commit: &str) -> Result<PathBuf, String> {
    let dir = paths.worktrees().join(name);
    if dir.exists() {
        let _ = git(
            &config.repo,
            &["worktree", "remove", "--force", &dir.to_string_lossy()],
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
    std::fs::create_dir_all(paths.worktrees()).map_err(|e| e.to_string())?;
    git(
        &config.repo,
        &[
            "worktree",
            "add",
            "--detach",
            &dir.to_string_lossy(),
            commit,
        ],
    )?;
    Ok(dir)
}

fn drop_worktree(config: &Config, dir: &Path) {
    let _ = git(
        &config.repo,
        &["worktree", "remove", "--force", &dir.to_string_lossy()],
    );
    let _ = std::fs::remove_dir_all(dir);
}

fn head(config: &Config) -> Result<String, String> {
    git(
        &config.repo,
        &["rev-parse", &format!("refs/heads/{}", config.branch)],
    )
}

/// The agent's change, as a patch against the commit it started from.
fn patch_of(dir: &Path, base: &str) -> Result<String, String> {
    let mut add = vec!["add", "-A", "--"];
    add.extend(CHANGE_PATHS.iter().filter(|p| dir.join(p).exists()));
    add.extend(FIXED_PATHS);
    git(dir, &add)?;
    let out = crate::exec::Run::new(
        "git",
        &[
            "diff".to_owned(),
            "--cached".to_owned(),
            "--binary".to_owned(),
            base.to_owned(),
        ],
        dir,
    )
    .go()?;
    Ok(out.stdout)
}

/// Establishes the baseline from the branch head, if there is none yet.
pub fn baseline(config: &Config, paths: &Paths, data: &Data) -> Result<Baseline, String> {
    if paths.baseline().exists() {
        return state::read_json(&paths.baseline());
    }
    let commit = head(config)?;
    log(paths, &format!("establishing the baseline at {commit}"));
    let dir = worktree(config, paths, "baseline", &commit)?;
    let target = paths.eval_target();
    let result = evaluate::checks(&dir, &target)
        .and_then(|()| evaluate::evaluate(config, paths, data, &dir, &target, &[]));
    drop_worktree(config, &dir);
    let ev = result?;
    let b = Baseline {
        commit: commit.clone(),
        round: 0,
        scores: ev.scores.clone(),
        train: ev.train,
        heldout: ev.heldout,
    };
    state::write_json(&paths.baseline(), &b)?;
    state::append_history(
        paths,
        &Entry {
            round: 0,
            kind: "baseline".into(),
            group: "seed".into(),
            accepted: true,
            reason: "the seed".into(),
            scores: ev.scores,
            commit,
            unix_time: state::now(),
            ..Entry::default()
        },
    )?;
    Ok(b)
}

/// Scores the version at `commit` again: after the training data changed
/// (`kind` "data") or after changes made by hand outside the loop (`kind`
/// "manual"), so later changes are judged against it.
fn rescore(
    config: &Config,
    paths: &Paths,
    data: &Data,
    base: &Baseline,
    commit: &str,
    kind: &str,
    why: &str,
) -> Result<Baseline, String> {
    log(
        paths,
        &format!(
            "{why}; scoring {commit} again ({} training turns before)",
            base.train.len()
        ),
    );
    let dir = worktree(config, paths, "rescore", commit)?;
    let target = paths.eval_target();
    let built = crate::exec::cargo(&dir, &target, &["build", "--release", "-p", "glassmind"]);
    let result = match built {
        Ok(o) if o.ok => evaluate::evaluate(config, paths, data, &dir, &target, &[]),
        Ok(o) => Err(format!("build failed: {}", o.stderr)),
        Err(e) => Err(e),
    };
    drop_worktree(config, &dir);
    let ev = result?;
    let round = state::history(paths)
        .iter()
        .map(|e| e.round)
        .max()
        .unwrap_or(0);
    let b = Baseline {
        commit: commit.to_owned(),
        round,
        scores: ev.scores.clone(),
        train: ev.train,
        heldout: ev.heldout,
    };
    state::write_json(&paths.baseline(), &b)?;
    state::append_history(
        paths,
        &Entry {
            round,
            kind: kind.into(),
            group: why.into(),
            accepted: true,
            reason: why.into(),
            scores: ev.scores,
            tokens_cumulative: cumulative(paths),
            commit: commit.to_owned(),
            unix_time: state::now(),
            ..Entry::default()
        },
    )?;
    let _ = std::fs::remove_file(paths.attempts());
    Ok(b)
}

/// Whether the validation score has not improved for `patience` rounds
/// (the held-out score decides nothing, not even when to stop).
pub fn stale(history: &[Entry], patience: u32, round: u32) -> bool {
    // A change made by hand can change how cost is counted, so improvement
    // is measured from the last one on.
    let since = history
        .iter()
        .rposition(|e| e.kind == "manual" || e.kind == "data")
        .unwrap_or(0);
    let mut best = f64::INFINITY;
    let mut best_round = 0;
    for e in history.iter().skip(since).filter(|e| e.accepted) {
        if e.scores.valid_cost < best - 1e-9 {
            best = e.scores.valid_cost;
            best_round = e.round;
        }
    }
    round > best_round + patience
}

/// Merges the upstream branch's new commits into the loop's branch, if
/// there are any and they merge cleanly; the merged subject line.
fn pick_up(config: &Config) -> Result<Option<String>, String> {
    let Some((repo, branch)) = &config.upstream else {
        return Ok(None);
    };
    git(
        &config.repo,
        &["fetch", "-q", &repo.to_string_lossy(), branch],
    )?;
    let theirs = git(&config.repo, &["rev-parse", "FETCH_HEAD"])?;
    let ours = head(config)?;
    let merged = git(
        &config.repo,
        &["merge-base", "--is-ancestor", &theirs, &ours],
    )
    .is_ok();
    if merged {
        return Ok(None);
    }
    let subject = git(&config.repo, &["log", "-1", "--format=%s", &theirs])?;
    let new = if git(
        &config.repo,
        &["merge-base", "--is-ancestor", &ours, &theirs],
    )
    .is_ok()
    {
        theirs
    } else {
        // Both moved: a merge commit, made in a scratch worktree.
        let dir = config.repo.join(".git").join("pick-up");
        let _ = git(
            &config.repo,
            &["worktree", "remove", "--force", &dir.to_string_lossy()],
        );
        git(
            &config.repo,
            &[
                "worktree",
                "add",
                "-q",
                "--detach",
                &dir.to_string_lossy(),
                &ours,
            ],
        )?;
        let made = git(&dir, &["merge", "-q", "--no-edit", &theirs])
            .and_then(|_| git(&dir, &["rev-parse", "HEAD"]));
        let _ = git(
            &config.repo,
            &["worktree", "remove", "--force", &dir.to_string_lossy()],
        );
        made.map_err(|e| format!("merging {theirs} into {ours}: {e}"))?
    };
    git(
        &config.repo,
        &[
            "update-ref",
            &format!("refs/heads/{}", config.branch),
            &new,
            &ours,
        ],
    )?;
    Ok(Some(subject))
}

/// Runs rounds until stopped (a `STOP` file in the state directory, or no
/// validation improvement for `patience` rounds), or just one with `once`.
pub fn run(paths: &Paths, once: bool) -> Result<(), String> {
    let config: Config = state::read_json(&paths.config())?;
    let data = Data::load(&config)?;
    let mut base = baseline(&config, paths, &data)?;
    let at = head(&config)?;
    if base.train.len() != data.train.len() {
        let why = format!("more training data: {} turns", data.train.len());
        base = rescore(&config, paths, &data, &base, &at, "data", &why)?;
    } else if base.commit != at {
        let subject = git(&config.repo, &["log", "-1", "--format=%s", &at]).unwrap_or_default();
        let why = format!("changed by hand: {subject}");
        base = rescore(&config, paths, &data, &base, &at, "manual", &why)?;
    }
    loop {
        if paths.stop_file().exists() {
            log(paths, "stop file found; stopping");
            break;
        }
        // Changes made by hand meanwhile: merged, scored, and on.
        match pick_up(&config) {
            Ok(Some(subject)) => {
                let at = head(&config)?;
                let why = format!("changed by hand: {subject}");
                base = rescore(&config, paths, &data, &base, &at, "manual", &why)?;
            }
            Ok(None) => {}
            Err(e) => log(paths, &format!("could not take up hand changes: {e}")),
        }
        let history = state::history(paths);
        let round = history.iter().map(|e| e.round).max().unwrap_or(0) + 1;
        if stale(&history, config.patience, round) {
            log(
                paths,
                &format!(
                    "no validation improvement for {} rounds; stopping",
                    config.patience
                ),
            );
            break;
        }
        let compression = config.growth_per_compression > 0
            && u64::from(round) % (u64::from(config.growth_per_compression) + 1) == 0;
        log(
            paths,
            &format!(
                "round {round}: {}",
                if compression { "compression" } else { "growth" }
            ),
        );
        let result = if compression {
            compression_round(&config, paths, &data, &mut base, round)
        } else {
            growth_round(&config, paths, &data, &mut base, round)
        };
        if let Err(e) = result {
            log(paths, &format!("round {round} failed: {e}"));
            state::append_history(
                paths,
                &Entry {
                    round,
                    kind: if compression { "compression" } else { "growth" }.into(),
                    group: "(round)".into(),
                    reason: e,
                    scores: base.scores.clone(),
                    tokens_cumulative: cumulative(paths),
                    unix_time: state::now(),
                    ..Entry::default()
                },
            )?;
        }
        if once {
            break;
        }
    }
    Ok(())
}

fn cumulative(paths: &Paths) -> u64 {
    state::history(paths).iter().map(|e| e.tokens.total()).sum()
}

/// The diff of the most recent accepted change, for agents to learn from.
fn worked_example(config: &Config, paths: &Paths, kind: &str) -> Option<String> {
    let entry = state::history(paths)
        .into_iter()
        .rev()
        .find(|e| e.accepted && e.kind == kind && !e.commit.is_empty())?;
    let diff = git(
        &config.repo,
        &["show", "--format=%s%n%n%b", &entry.commit, "--", "crates"],
    )
    .ok()?;
    let lines: Vec<&str> = diff.lines().take(260).collect();
    Some(lines.join("\n"))
}

fn growth_round(
    config: &Config,
    paths: &Paths,
    data: &Data,
    base: &mut Baseline,
    round: u32,
) -> Result<(), String> {
    let verdicts = evaluate::verdicts(&data.train, &base.train);
    let mut attempts: Attempts = state::read_json(&paths.attempts()).unwrap_or_default();
    // Groups agents keep failing at cool down; groups that mostly overlap
    // one already chosen wait for a later round, since their fixes would collide.
    let mut chosen: Vec<growth::Group> = Vec::new();
    let mut taken: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let focused: Vec<crate::score::Label> = data
        .train
        .iter()
        .filter(|l| {
            config.focus.is_empty() || config.focus.iter().any(|f| f == evaluate::family(&l.kind))
        })
        .cloned()
        .collect();
    for g in growth::groups(&focused, &base.train, &verdicts) {
        if chosen.len() >= config.max_agents {
            break;
        }
        let cooled = attempts
            .groups
            .get(&g.key)
            .is_some_and(|(failed, size)| *failed >= 3 && *size == g.turns.len());
        let overlap = g.turns.iter().filter(|t| taken.contains(*t)).count();
        if cooled || overlap * 2 > g.turns.len() {
            continue;
        }
        taken.extend(g.turns.iter().cloned());
        chosen.push(g);
    }
    if chosen.is_empty() {
        return Err("no failure groups left to work on".into());
    }
    let example = worked_example(config, paths, "growth");
    let start = base.commit.clone();
    let mut jobs = Vec::new();
    for (i, g) in chosen.iter().enumerate() {
        let slot = paths.slot(i);
        let _ = std::fs::remove_dir_all(slot.join("data"));
        std::fs::create_dir_all(&slot).map_err(|e| e.to_string())?;
        let brief = growth::write_brief(
            &slot,
            config,
            round,
            g,
            &data.train,
            &base.train,
            example.as_deref(),
        )?;
        let wt = worktree(config, paths, &format!("r{round}-{i}"), &start)?;
        log(
            paths,
            &format!(
                "  agent {i}: {} ({} turns, cost {:.1})",
                g.title,
                g.turns.len(),
                g.cost
            ),
        );
        jobs.push((g.clone(), slot, wt, brief));
    }
    let runs: Vec<Result<AgentRun, String>> = std::thread::scope(|s| {
        let handles: Vec<_> = jobs
            .iter()
            .map(|(_, slot, wt, brief)| {
                s.spawn(|| agent::run_agent(config, paths, slot, wt, brief))
            })
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| Err("agent thread panicked".into()))
            })
            .collect()
    });
    for ((group, _slot, wt, _), run) in jobs.iter().zip(runs) {
        let mut agent_ok = false;
        let (mut tokens, summary, patch) = match run {
            Ok(r) => {
                agent_ok = r.ok;
                let _ = std::fs::write(
                    paths
                        .logs()
                        .join(format!("r{round}-{}.log", sanitize(&group.key))),
                    &r.log,
                );
                (r.tokens, r.summary, patch_of(wt, &start))
            }
            Err(e) => (Tokens::default(), String::new(), Err(e)),
        };
        let outcome = match patch {
            Ok(p) if p.trim().is_empty() => Err(if agent_ok {
                "the agent made no change".to_owned()
            } else {
                "the agent failed or timed out without a change".to_owned()
            }),
            Ok(p) => consider(
                config,
                paths,
                data,
                base,
                round,
                "growth",
                &group.title,
                &summary,
                &p,
                &mut tokens,
            ),
            Err(e) => Err(e),
        };
        record(paths, base, round, "growth", &group.title, tokens, &outcome)?;
        let entry = attempts
            .groups
            .entry(group.key.clone())
            .or_insert((0, group.turns.len()));
        if outcome.is_ok() {
            *entry = (0, group.turns.len());
        } else {
            *entry = (entry.0 + 1, group.turns.len());
        }
        drop_worktree(config, wt);
    }
    state::write_json(&paths.attempts(), &attempts)
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

fn record(
    paths: &Paths,
    base: &Baseline,
    round: u32,
    kind: &str,
    group: &str,
    tokens: Tokens,
    outcome: &Result<Evaluated, String>,
) -> Result<(), String> {
    let previous = cumulative(paths);
    let (accepted, reason, scores) = match outcome {
        Ok(ev) => (true, "accepted".to_owned(), ev.scores.clone()),
        Err(e) => (false, e.clone(), base.scores.clone()),
    };
    log(
        paths,
        &format!(
            "  {kind} `{group}`: {} (train {:.1}, held-out {:.1}, size {}) {}",
            if accepted { "ACCEPTED" } else { "rejected" },
            scores.train_cost,
            scores.heldout_cost,
            scores.size,
            if accepted {
                String::new()
            } else {
                reason.lines().next().unwrap_or_default().to_owned()
            }
        ),
    );
    state::append_history(
        paths,
        &Entry {
            round,
            kind: kind.into(),
            group: group.into(),
            accepted,
            reason,
            tokens,
            tokens_cumulative: previous + tokens.total(),
            scores,
            commit: if accepted {
                base.commit.clone()
            } else {
                String::new()
            },
            unix_time: state::now(),
        },
    )
}

/// Applies a patch on top of the accepted version, checks, scores and
/// reviews it, and commits it if it is accepted. On success the baseline is
/// the new version.
#[allow(clippy::too_many_arguments)]
fn consider(
    config: &Config,
    paths: &Paths,
    data: &Data,
    base: &mut Baseline,
    round: u32,
    kind: &str,
    group: &str,
    summary: &str,
    patch: &str,
    tokens: &mut Tokens,
) -> Result<Evaluated, String> {
    let at = head(config)?;
    let dir = worktree(config, paths, "apply", &at)?;
    let result = (|| {
        let patch = unique_rule_ids(&dir, patch, round);
        let patch_file = paths.root.join("candidate.patch");
        std::fs::write(&patch_file, &patch).map_err(|e| e.to_string())?;
        let applied = git(
            &dir,
            &[
                "apply",
                "--3way",
                "--whitespace=nowarn",
                &patch_file.to_string_lossy(),
            ],
        );
        if let Err(e) = applied {
            merge(config, paths, &dir, summary, tokens).map_err(|m| {
                format!("the change does not apply to the latest accepted version: {e}\n{m}")
            })?;
        }
        let target = paths.eval_target();
        evaluate::checks(&dir, &target)?;
        let ev = evaluate::evaluate(config, paths, data, &dir, &target, &[])?;
        evaluate::decide(config, data, base, &ev)?;
        if config.reviewer {
            review(config, paths, &dir)?;
        }
        let mut message = String::new();
        let _ = writeln!(message, "loop round {round} ({kind}): {group}\n");
        let _ = writeln!(message, "{}\n", summary.trim());
        let _ = writeln!(message, "Loop-Round: {round}");
        let _ = writeln!(
            message,
            "Train-Cost: {:.1} -> {:.1}",
            base.scores.train_cost, ev.scores.train_cost
        );
        let _ = writeln!(
            message,
            "Heldout-Cost: {:.1} -> {:.1}",
            base.scores.heldout_cost, ev.scores.heldout_cost
        );
        let _ = writeln!(message, "Size: {} -> {}", base.scores.size, ev.scores.size);
        git(&dir, &["add", "-A", "--"])?;
        git(&dir, &["commit", "-q", "-m", &message])?;
        let new = git(&dir, &["rev-parse", "HEAD"])?;
        git(
            &config.repo,
            &[
                "update-ref",
                &format!("refs/heads/{}", config.branch),
                &new,
                &at,
            ],
        )?;
        *base = Baseline {
            commit: new,
            round,
            scores: ev.scores.clone(),
            train: ev.train.clone(),
            heldout: ev.heldout.clone(),
        };
        state::write_json(&paths.baseline(), base)?;
        Ok(ev)
    })();
    drop_worktree(config, &dir);
    result
}

/// The rule IDs given to `RuleInfo::new` on the lines of `text`.
fn rule_ids<'t>(lines: impl Iterator<Item = &'t str>) -> BTreeSet<String> {
    const MARK: &str = "RuleInfo::new(\"";
    lines
        .flat_map(|l| {
            l.match_indices(MARK)
                .map(move |(i, _)| l.get(i + MARK.len()..))
        })
        .flatten()
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_owned)
        .collect()
}

/// The patch with every rule it adds under an ID the accepted version in
/// `dir` already has renamed (`id` becomes `id.r<round><n>`): agents working
/// in parallel pick the same names.
fn unique_rule_ids(dir: &Path, patch: &str, round: u32) -> String {
    let added = rule_ids(
        patch
            .lines()
            .filter(|l| l.starts_with('+') && !l.starts_with("+++")),
    );
    let removed = rule_ids(
        patch
            .lines()
            .filter(|l| l.starts_with('-') && !l.starts_with("---")),
    );
    let existing = git(dir, &["grep", "-h", "RuleInfo::new(\"", "--", "crates"])
        .map(|out| rule_ids(out.lines()))
        .unwrap_or_default();
    let mut patch = patch.to_owned();
    for (n, id) in added
        .iter()
        .filter(|id| existing.contains(*id) && !removed.contains(*id))
        .enumerate()
    {
        let (old, new) = (format!("\"{id}\""), format!("\"{id}.r{round}{n}\""));
        patch = patch
            .lines()
            .map(|l| {
                if l.starts_with('+') && l.contains("RuleInfo::new(") {
                    l.replace(&old, &new)
                } else {
                    l.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
    }
    patch
}

const MERGE: &str = "You are merging one change into glassmind, a symbolic English-understanding system whose \
knowledge is Rust source (read docs/growing.md). The change was written against an older version; other changes \
were accepted since, and applying it left git conflict markers (<<<<<<< ======= >>>>>>>) in these files: {files}. \
Resolve every conflict so that BOTH the already accepted code and the change work together: keep every rule from \
both sides (rule IDs must stay unique), merge enum variants and match arms, and fix anything else that no longer \
fits. Do not add new features. Then make sure `cargo build --release -j 1 --workspace`, `cargo clippy --workspace \
--all-targets -j 1 -- -D warnings` pass in capped scopes with TMPDIR set. Leave the result \
uncommitted. What the change was for: {summary}";

/// Resolves the conflicts a patch left in `dir` with a merging agent.
fn merge(
    config: &Config,
    paths: &Paths,
    dir: &Path,
    summary: &str,
    tokens: &mut Tokens,
) -> Result<(), String> {
    let files = git(dir, &["diff", "--name-only", "--diff-filter=U"])?;
    if files.trim().is_empty() {
        return Err("nothing to merge".into());
    }
    let prompt = MERGE
        .replace("{files}", &files.lines().collect::<Vec<_>>().join(", "))
        .replace("{summary}", summary.trim());
    let run = agent::model_agent(
        config,
        paths,
        &paths.slot(98),
        dir,
        &prompt,
        None,
        Vec::new(),
    )?;
    tokens.add(&run.tokens);
    let markers = crate::exec::Run::new(
        "git",
        &[
            "grep".to_owned(),
            "-l".to_owned(),
            "-E".to_owned(),
            "^(<<<<<<<|>>>>>>>) ".to_owned(),
        ],
        dir,
    )
    .go()?;
    if !markers.stdout.trim().is_empty() {
        return Err(format!(
            "conflict markers remain in {}",
            markers.stdout.trim()
        ));
    }
    git(dir, &["add", "-A", "--"])?;
    Ok(())
}

const REVIEW: &str = "You review one change to glassmind, an English-understanding system whose knowledge is \
readable Rust source. Read docs/growing.md. Run `git diff HEAD` to see the change (it is uncommitted). Any method \
is allowed (fuzzy matching, learned models, outside data, new formalisms). VETO the change only if it clearly does \
one of these: (1) fakes answers instead of computing them from the world (for example a table of particular \
training sentences or task names with their answers); (2) reads anything about the held-out data or the scorer's \
labels; (3) puts knowledge where only one request can use it while presenting it as general. Do not veto for \
method, style, size or doubts about usefulness: the scorer judges those. \
Reply with only JSON: {\"veto\": true|false, \"reason\": \"...\"}";

fn review(config: &Config, paths: &Paths, dir: &Path) -> Result<(), String> {
    // No verdict is not an approval: an empty or unreadable reply is asked
    // again once, then the change is rejected.
    for _ in 0..2 {
        let run = agent::model_agent(
            config,
            paths,
            &paths.slot(99),
            dir,
            REVIEW,
            Some("Read,Grep,Glob,Bash"),
            Vec::new(),
        )?;
        let text = run.summary;
        let _ = std::fs::write(paths.logs().join("last-review.txt"), &text);
        let verdict: Option<serde_json::Value> = text
            .find('{')
            .zip(text.rfind('}'))
            .and_then(|(a, b)| text.get(a..=b))
            .and_then(|json| serde_json::from_str(json).ok());
        match verdict
            .as_ref()
            .and_then(|v| v.get("veto"))
            .and_then(serde_json::Value::as_bool)
        {
            Some(false) => return Ok(()),
            Some(true) => {
                let reason = verdict
                    .as_ref()
                    .and_then(|v| v.get("reason"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("vetoed");
                return Err(format!("reviewer veto: {reason}"));
            }
            None => continue,
        }
    }
    Err("the reviewer gave no readable verdict".into())
}

fn compression_round(
    config: &Config,
    paths: &Paths,
    data: &Data,
    base: &mut Baseline,
    round: u32,
) -> Result<(), String> {
    let start = base.commit.clone();
    let dir = worktree(config, paths, "ablate", &start)?;
    let target = paths.eval_target();
    let found = compress::find(config, paths, data, base, &dir, &target);
    drop_worktree(config, &dir);
    let found = found?;
    log(
        paths,
        &format!(
            "  ablation: {} rules removable, {} single-use",
            found.removable.len(),
            found.single_use.len()
        ),
    );
    let slot = paths.slot(0);
    std::fs::create_dir_all(&slot).map_err(|e| e.to_string())?;
    let brief = compress::write_brief(&slot, config, round, &found, base)?;
    let wt = worktree(config, paths, &format!("r{round}-compress"), &start)?;
    let run = agent::run_agent(config, paths, &slot, &wt, &brief);
    let (mut tokens, summary, patch) = match run {
        Ok(r) => {
            let _ = std::fs::write(paths.logs().join(format!("r{round}-compress.log")), &r.log);
            (r.tokens, r.summary, patch_of(&wt, &start))
        }
        Err(e) => (Tokens::default(), String::new(), Err(e)),
    };
    let outcome = match patch {
        Ok(p) if p.trim().is_empty() => Err("the agent made no change".to_owned()),
        Ok(p) => consider(
            config,
            paths,
            data,
            base,
            round,
            "compression",
            "compression",
            &summary,
            &p,
            &mut tokens,
        ),
        Err(e) => Err(e),
    };
    record(
        paths,
        base,
        round,
        "compression",
        "compression",
        tokens,
        &outcome,
    )?;
    drop_worktree(config, &wt);
    Ok(())
}

/// Rule usage on the training set: for each rule ID, the turns it took part
/// in that were answered right.
pub fn usage(data: &Data, base: &Baseline) -> BTreeMap<String, Vec<String>> {
    let verdicts = evaluate::verdicts(&data.train, &base.train);
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for o in &base.train {
        if verdicts
            .get(&o.turn_id)
            .is_some_and(|(v, _)| *v == crate::score::Verdict::Right)
        {
            for r in &o.rules {
                out.entry(r.clone()).or_default().push(o.turn_id.clone());
            }
        }
    }
    out
}
