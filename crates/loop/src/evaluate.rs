//! Building, checking and scoring one version of glassmind.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use glassmind_answer::Outcome;

use crate::exec::{Run, cargo};
use crate::score::{self, Label, Verdict};
use crate::state::{Baseline, Config, Paths, Scores};
use crate::{agent, read_jsonl, size};

/// Outcomes and scores of one version.
#[derive(Debug, Clone, Default)]
pub struct Evaluated {
    pub scores: Scores,
    pub train: Vec<Outcome>,
    pub heldout: Vec<Outcome>,
}

/// The labelled data sets.
pub struct Data {
    pub train: Vec<Label>,
    pub heldout: Vec<Label>,
    pub valid: Vec<Label>,
}

impl Data {
    pub fn load(config: &Config) -> Result<Data, String> {
        Ok(Data {
            train: read_jsonl(&config.train.join("labels.jsonl"))?,
            heldout: read_jsonl(&config.heldout.join("labels.jsonl"))?,
            valid: match &config.valid {
                Some(v) => read_jsonl(&v.join("labels.jsonl"))?,
                None => Vec::new(),
            },
        })
    }
}

fn tail(s: &str, lines: usize) -> String {
    let all: Vec<&str> = s.lines().collect();
    all.get(all.len().saturating_sub(lines)..)
        .unwrap_or_default()
        .join("\n")
}

/// Builds the workspace and runs clippy, the tests, the layer check and the
/// rule check. `Err` says which failed and why.
pub fn checks(dir: &Path, target: &Path) -> Result<(), String> {
    let steps: [(&str, Vec<&str>); 4] = [
        ("build", vec!["build", "--release", "--workspace"]),
        (
            "clippy",
            vec![
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        ),
        ("tests", vec!["test", "--workspace"]),
        ("layers", vec!["run", "-q", "-p", "glassmind-depcheck"]),
    ];
    for (name, args) in steps {
        let out = cargo(dir, target, &args)?;
        if !out.ok {
            return Err(format!("{name} failed:\n{}", tail(&out.stderr, 40)));
        }
    }
    let out = Run::new(
        &binary(target).to_string_lossy(),
        &["rules".to_owned()],
        dir,
    )
    .go()?;
    if !out.ok || out.stderr.contains("problem:") {
        return Err(format!("rule check failed:\n{}", tail(&out.stderr, 20)));
    }
    Ok(())
}

pub fn binary(target: &Path) -> PathBuf {
    target.join("release").join("glassmind")
}

/// Runs `glassmind eval` on one data set.
pub fn eval_set(
    config: &Config,
    paths: &Paths,
    dir: &Path,
    target: &Path,
    sources: &Path,
    data: &Path,
    disable: &[String],
) -> Result<Vec<Outcome>, String> {
    let mut args = vec![
        "eval".to_owned(),
        "--sources".to_owned(),
        sources.to_string_lossy().into_owned(),
        "--inputs".to_owned(),
        data.join("inputs.jsonl").to_string_lossy().into_owned(),
    ];
    if !disable.is_empty() {
        args.push("--disable".to_owned());
        args.push(disable.join(","));
    }
    // The candidate's code reads held-out inputs, so it runs where it can
    // write nothing.
    let held_out =
        (sources == config.heldout_sources.as_path()).then_some(config.heldout.as_path());
    let places = check_places_for(held_out);
    let scratch = paths.root.join("eval-tmp");
    std::fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
    let mut sb = agent::sandbox(config, paths, Vec::new(), Some(scratch));
    if let Some(s) = &mut sb {
        s.hidden
            .retain(|h| h != &config.heldout && h != &config.heldout_sources && h != &paths.root);
        // Coding tasks build and test their sources: builds and results go
        // to the shared check places, the only places it may write.
        s.writable = vec![
            places.target.clone().unwrap_or_default(),
            places.cache.clone().unwrap_or_default(),
            scratch_of(&places),
        ];
    }
    let out = Run::new(&binary(target).to_string_lossy(), &args, dir)
        .env(
            "GLASSMIND_CARGO_TARGET",
            &places.target.clone().unwrap_or_default().to_string_lossy(),
        )
        .env(
            "GLASSMIND_CHECK_CACHE",
            &places.cache.clone().unwrap_or_default().to_string_lossy(),
        )
        .env("GLASSMIND_SCRATCH", &scratch_of(&places).to_string_lossy())
        .timeout(Duration::from_secs(3600))
        .sandbox(sb.as_ref())
        .go()?;
    if !out.ok {
        return Err(format!("eval failed:\n{}", tail(&out.stderr, 30)));
    }
    let outcomes: Vec<Outcome> = out
        .stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).map_err(|e| format!("bad outcome: {e}")))
        .collect::<Result<_, _>>()?;
    let labels: Vec<Label> = read_jsonl(&data.join("labels.jsonl")).unwrap_or_default();
    Ok(crate::verify::remedies(&places, &labels, outcomes, sources))
}

/// Where checks keep builds and results. Held-out sources get their own,
/// inside the held-out directory agents cannot see, since the results hold
/// what the compiler said about them.
pub fn check_places_for(held_out: Option<&Path>) -> glassmind_world_rust::checks::Places {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let base = match held_out {
        Some(dir) => dir.join("check"),
        None => home.join(".cache/glassmind/check"),
    };
    let places = glassmind_world_rust::checks::Places {
        target: Some(base.join("target")),
        cache: Some(base.join("results")),
    };
    for dir in [&places.target, &places.cache].into_iter().flatten() {
        let _ = std::fs::create_dir_all(dir);
    }
    places
}

/// Where scratch copies for trying changes go: beside the check results.
fn scratch_of(places: &glassmind_world_rust::checks::Places) -> PathBuf {
    let dir = places
        .cache
        .as_ref()
        .and_then(|c| c.parent())
        .map(|p| p.join("scratch"))
        .unwrap_or_default();
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Where checks on training sources keep builds and results.
pub fn check_places() -> glassmind_world_rust::checks::Places {
    check_places_for(None)
}

/// Scores `dir`'s glassmind (already built into `target`) on every data set.
pub fn evaluate(
    config: &Config,
    paths: &Paths,
    data: &Data,
    dir: &Path,
    target: &Path,
    disable: &[String],
) -> Result<Evaluated, String> {
    let train = eval_set(
        config,
        paths,
        dir,
        target,
        &config.sources,
        &config.train,
        disable,
    )?;
    let heldout = eval_set(
        config,
        paths,
        dir,
        target,
        &config.heldout_sources,
        &config.heldout,
        disable,
    )?;
    let valid = match &config.valid {
        Some(v) => eval_set(
            config,
            paths,
            dir,
            target,
            &config.heldout_sources,
            v,
            disable,
        )?,
        None => Vec::new(),
    };
    let mut families: BTreeMap<String, crate::state::Family> = BTreeMap::new();
    for (labels, outcomes, held) in [
        (&data.train, &train, false),
        (&data.heldout, &heldout, true),
    ] {
        let report = score::report(labels, outcomes);
        for label in labels {
            let f = families.entry(family(&label.kind).to_owned()).or_default();
            let right = report
                .turns
                .get(&label.turn_id)
                .is_some_and(|s| s.verdict == Verdict::Right);
            if held {
                f.heldout_turns += 1;
                f.heldout_right += usize::from(right);
            } else {
                f.train_turns += 1;
                f.train_right += usize::from(right);
            }
        }
    }
    let t = score::report(&data.train, &train).total;
    let h = score::report(&data.heldout, &heldout).total;
    let v = score::report(&data.valid, &valid).total;
    let rules = Run::new(
        &binary(target).to_string_lossy(),
        &["rules".to_owned()],
        dir,
    )
    .go()
    .map(|o| o.stdout.lines().count())
    .unwrap_or(0);
    Ok(Evaluated {
        scores: Scores {
            train_cost: t.cost,
            train_right: t.right,
            train_wrong: t.wrong,
            train_turns: t.turns,
            heldout_cost: h.cost,
            heldout_right: h.right,
            heldout_wrong: h.wrong,
            heldout_turns: h.turns,
            valid_cost: v.cost,
            valid_right: v.right,
            valid_turns: v.turns,
            size: size::mind_size(dir).1,
            rules,
            families,
        },
        train,
        heldout,
    })
}

/// The task family of a label's kind: coding tasks for the harness, or
/// questions and requests about code (`qa`).
pub fn family(kind: &str) -> &'static str {
    let kind = kind.rsplit(':').next().unwrap_or(kind);
    if matches!(
        kind,
        "build_status" | "fix_build" | "test_status" | "fix_tests"
    ) {
        "harness"
    } else {
        "qa"
    }
}

/// Per-turn verdicts.
pub fn verdicts(labels: &[Label], outcomes: &[Outcome]) -> BTreeMap<String, (Verdict, f64)> {
    score::report(labels, outcomes)
        .turns
        .into_iter()
        .map(|(id, s)| (id, (s.verdict, s.cost)))
        .collect()
}

/// Whether a candidate may replace the baseline, by the acceptance rules
/// (other than the build checks and the reviewer).
pub fn decide(
    config: &Config,
    data: &Data,
    before: &Baseline,
    after: &Evaluated,
) -> Result<(), String> {
    let b = verdicts(&data.train, &before.train);
    let a = verdicts(&data.train, &after.train);
    let broken: Vec<&String> = b
        .iter()
        .filter(|(id, (v, _))| {
            matches!(v, Verdict::Right | Verdict::Refused)
                && a.get(*id).is_some_and(|(va, _)| *va == Verdict::Wrong)
        })
        .map(|(id, _)| id)
        .collect();
    if !broken.is_empty() {
        let shown: Vec<&str> = broken.iter().take(8).map(|s| s.as_str()).collect();
        return Err(format!(
            "{} training turns that were right or honestly refused became wrong: {}",
            broken.len(),
            shown.join(", ")
        ));
    }
    let wordless = |outcomes: &[Outcome]| score::report(&data.train, outcomes).total.wordless;
    let (wb, wa) = (wordless(&before.train), wordless(&after.train));
    if wa > wb {
        return Err(format!(
            "{} more training replies have no words ({wb} -> {wa})",
            wa - wb
        ));
    }
    let tb = before.scores.total(config.size_price);
    let ta = after.scores.total(config.size_price);
    if ta >= tb - 1e-9 {
        return Err(format!(
            "total did not drop: {ta:.2} (train {:.1} + size {}) vs {tb:.2} (train {:.1} + size {})",
            after.scores.train_cost,
            after.scores.size,
            before.scores.train_cost,
            before.scores.size
        ));
    }
    if after.scores.heldout_cost > before.scores.heldout_cost + 1e-9 {
        return Err(format!(
            "held-out score dropped: cost {:.1} vs {:.1}",
            after.scores.heldout_cost, before.scores.heldout_cost
        ));
    }
    Ok(())
}
