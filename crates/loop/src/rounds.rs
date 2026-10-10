//! The loop itself: growth and compression rounds, each candidate change
//! applied on top of the accepted version, checked, scored, reviewed, and
//! committed only if it is better.

use std::collections::BTreeMap;
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

/// Whether the held-out score has not improved for `patience` rounds.
pub fn stale(history: &[Entry], patience: u32, round: u32) -> bool {
    // A change made by hand can change how cost is counted, so improvement
    // is measured from the last one on.
    let since = history
        .iter()
        .rposition(|e| e.kind == "manual")
        .unwrap_or(0);
    let mut best = f64::INFINITY;
    let mut best_round = 0;
    for e in history.iter().skip(since).filter(|e| e.accepted) {
        if e.scores.heldout_cost < best - 1e-9 {
            best = e.scores.heldout_cost;
            best_round = e.round;
        }
    }
    round > best_round + patience
}

/// Runs rounds until stopped (a `STOP` file in the state directory, or no
/// held-out improvement for `patience` rounds), or just one with `once`.
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
        let history = state::history(paths);
        let round = history.iter().map(|e| e.round).max().unwrap_or(0) + 1;
        if stale(&history, config.patience, round) {
            log(
                paths,
                &format!(
                    "no held-out improvement for {} rounds; stopping",
                    config.patience
                ),
            );
            break;
        }
        let compression = round % (config.growth_per_compression + 1) == 0;
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
    for g in growth::groups(&data.train, &base.train, &verdicts) {
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
        let patch_file = paths.root.join("candidate.patch");
        std::fs::write(&patch_file, patch).map_err(|e| e.to_string())?;
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

const MERGE: &str = "You are merging one change into glassmind, a symbolic English-understanding system whose \
knowledge is Rust source (read docs/growing.md). The change was written against an older version; other changes \
were accepted since, and applying it left git conflict markers (<<<<<<< ======= >>>>>>>) in these files: {files}. \
Resolve every conflict so that BOTH the already accepted code and the change work together: keep every rule from \
both sides (rule IDs must stay unique), merge enum variants and match arms, and fix anything else that no longer \
fits. Do not add new features. Then make sure `cargo build --release -j 8 --workspace`, `cargo clippy --workspace \
--all-targets -j 8 -- -D warnings` and `cargo test --workspace -j 8 -- --test-threads 8` pass. Leave the result \
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
    let run = agent::claude(
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
    let slot = paths.slot(99);
    let run = agent::claude(
        config,
        paths,
        &slot,
        dir,
        REVIEW,
        Some("Read,Grep,Glob,Bash"),
        Vec::new(),
    )?;
    let text = run.summary;
    let start = text.find('{').unwrap_or(0);
    let verdict: serde_json::Value = serde_json::from_str(
        text.get(start..)
            .unwrap_or_default()
            .trim_end_matches('`')
            .trim(),
    )
    .unwrap_or(serde_json::Value::Null);
    let _ = std::fs::write(paths.logs().join("last-review.txt"), &text);
    if verdict.get("veto").and_then(serde_json::Value::as_bool) == Some(true) {
        let reason = verdict
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("vetoed");
        return Err(format!("reviewer veto: {reason}"));
    }
    Ok(())
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
