//! The loop's state on disk: configuration, the accepted baseline and the
//! history of every round. Everything needed to resume after an interruption.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use glassmind_answer::Outcome;
use serde::{Deserialize, Serialize};

/// How the loop runs. Written once by `glassmind-loop init`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The loop's own clone of the glassmind repository.
    pub repo: PathBuf,
    /// The branch accepted changes are committed to.
    pub branch: String,
    /// Training sources (each source under its name).
    pub sources: PathBuf,
    /// Held-out sources. Never shown to agents.
    pub heldout_sources: PathBuf,
    /// Directories holding `inputs.jsonl` and `labels.jsonl`.
    pub train: PathBuf,
    pub heldout: PathBuf,
    /// Training generators on held-out repos: guards acceptance (a change
    /// may not make it worse) and decides when to stop. The held-out set
    /// decides nothing.
    pub valid: Option<PathBuf>,
    /// More directories agents never see: everything else held out (its
    /// generator prompts, dev sets, outside benchmarks' held-out splits).
    #[serde(default)]
    pub private: Vec<PathBuf>,
    /// Failure units per syntax token of mind code.
    pub size_price: f64,
    /// Agents running at once.
    pub max_agents: usize,
    pub model: String,
    /// `claude` for Claude Code agents, or a command that edits the
    /// worktree given as its first argument, with the brief as its second
    /// (used by CI's stub agent).
    pub agent: String,
    /// Run agents and evaluations inside bubblewrap, hiding the held-out
    /// data, the user's code and the user's Claude Code history.
    pub sandbox: bool,
    /// Growth rounds before each compression round.
    pub growth_per_compression: u32,
    /// Stop after this many rounds without a validation improvement.
    pub patience: u32,
    /// Ask an LLM reviewer to veto changes that put language knowledge in
    /// as special cases or knowledge in the wrong layer.
    pub reviewer: bool,
    /// Minutes an agent may run.
    pub agent_minutes: u64,
}

/// Token use of agent sessions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Tokens {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub cost_usd: f64,
}

impl Tokens {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_write
    }

    pub fn add(&mut self, o: &Tokens) {
        self.input += o.input;
        self.output += o.output;
        self.cache_read += o.cache_read;
        self.cache_write += o.cache_write;
        self.cost_usd += o.cost_usd;
    }
}

/// Scores of one evaluated version of glassmind.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Scores {
    pub train_cost: f64,
    pub train_right: usize,
    pub train_wrong: usize,
    pub train_turns: usize,
    pub heldout_cost: f64,
    pub heldout_right: usize,
    pub heldout_wrong: usize,
    pub heldout_turns: usize,
    pub valid_cost: f64,
    pub valid_right: usize,
    pub valid_turns: usize,
    pub size: u64,
    pub rules: usize,
    /// Right answers and turns per task family (`qa`, `harness`), for
    /// training and held-out, so each family keeps its own history.
    #[serde(default)]
    pub families: BTreeMap<String, Family>,
}

/// One task family's numbers on training and held-out turns.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Family {
    pub train_right: usize,
    pub train_turns: usize,
    pub heldout_right: usize,
    pub heldout_turns: usize,
}

impl Scores {
    /// What the loop minimizes: training failures plus the price of size.
    pub fn total(&self, size_price: f64) -> f64 {
        self.train_cost + size_price * self.size as f64
    }
}

/// The currently accepted version.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Baseline {
    pub commit: String,
    pub round: u32,
    pub scores: Scores,
    pub train: Vec<Outcome>,
    pub heldout: Vec<Outcome>,
}

/// One line of the history: one candidate change, or one round marker.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Entry {
    pub round: u32,
    /// `baseline`, `growth` or `compression`.
    pub kind: String,
    pub group: String,
    pub accepted: bool,
    pub reason: String,
    pub tokens: Tokens,
    /// Tokens spent by all agents so far, including this entry.
    pub tokens_cumulative: u64,
    /// Scores of the candidate (or of the baseline, for a baseline entry).
    pub scores: Scores,
    pub commit: String,
    pub unix_time: u64,
}

/// Failed attempts at a failure group, to cool down groups agents cannot fix.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Attempts {
    pub groups: BTreeMap<String, (u32, usize)>,
}

pub struct Paths {
    pub root: PathBuf,
}

impl Paths {
    /// The state directory, created if missing, as an absolute path (git
    /// worktrees and child processes need absolute paths).
    pub fn new(root: &Path) -> Self {
        let _ = std::fs::create_dir_all(root);
        Paths {
            root: std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf()),
        }
    }
    pub fn config(&self) -> PathBuf {
        self.root.join("config.json")
    }
    pub fn baseline(&self) -> PathBuf {
        self.root.join("baseline.json")
    }
    pub fn history(&self) -> PathBuf {
        self.root.join("history.jsonl")
    }
    pub fn attempts(&self) -> PathBuf {
        self.root.join("attempts.json")
    }
    pub fn stop_file(&self) -> PathBuf {
        self.root.join("STOP")
    }
    pub fn worktrees(&self) -> PathBuf {
        self.root.join("worktrees")
    }
    /// Private directory of agent slot `n`: brief, data, config, tmp.
    pub fn slot(&self, n: usize) -> PathBuf {
        self.root.join("slots").join(n.to_string())
    }
    pub fn eval_target(&self) -> PathBuf {
        self.root.join("target-eval")
    }
    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("cannot parse {}: {e}", path.display()))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn append_history(paths: &Paths, entry: &Entry) -> Result<(), String> {
    use std::io::Write;
    let line = serde_json::to_string(entry).map_err(|e| e.to_string())?;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths.history())
        .map_err(|e| e.to_string())?;
    writeln!(f, "{line}").map_err(|e| e.to_string())
}

pub fn history(paths: &Paths) -> Vec<Entry> {
    crate::read_jsonl(&paths.history()).unwrap_or_default()
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
