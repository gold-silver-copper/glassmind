//! `glassmind-loop`: the improvement loop and its measuring tools.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use glassmind_answer::Outcome;
use serde::de::DeserializeOwned;

/// Prints a line to stdout, ignoring a closed pipe (`println!` would panic).
macro_rules! say {
    ($($t:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stdout(), $($t)*);
    }};
}

mod agent;
mod chart;
mod compress;
mod evaluate;
mod exec;
mod growth;
mod rounds;
mod score;
mod size;
mod state;
mod verify;

#[derive(Parser)]
#[command(
    name = "glassmind-loop",
    about = "Grow glassmind against an automatic scorer"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Command {
    /// Score outcomes (from `glassmind eval`) against labels.
    Score {
        #[arg(long)]
        labels: PathBuf,
        #[arg(long)]
        outcomes: PathBuf,
        /// The sources the outcomes are about: remedies for coding tasks
        /// are applied to scratch copies of them and checked.
        #[arg(long)]
        sources: Option<PathBuf>,
        /// Print the full report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Count the mind crates' size in syntax tokens.
    Size {
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Show which turns changed verdict between two outcome files.
    Compare {
        #[arg(long)]
        labels: PathBuf,
        #[arg(long)]
        before: PathBuf,
        #[arg(long)]
        after: PathBuf,
    },
    /// Write a loop state directory's configuration.
    Init {
        /// The state directory.
        #[arg(long)]
        state: PathBuf,
        /// The loop's own clone of the repository.
        #[arg(long)]
        repo: PathBuf,
        #[arg(long, default_value = "loop")]
        branch: String,
        #[arg(long)]
        sources: PathBuf,
        #[arg(long)]
        heldout_sources: PathBuf,
        #[arg(long)]
        train: PathBuf,
        #[arg(long)]
        heldout: PathBuf,
        #[arg(long)]
        valid: Option<PathBuf>,
        /// More directories agents never see (everything else held out).
        #[arg(long)]
        private: Vec<PathBuf>,
        #[arg(long, default_value_t = 0.01)]
        size_price: f64,
        #[arg(long, default_value_t = 4)]
        max_agents: usize,
        #[arg(long, default_value = "gpt-6-astra")]
        model: String,
        /// `codex` (ChatGPT login, fast tier), or a stub `CMD WORKTREE BRIEF`.
        #[arg(long, default_value = "codex")]
        agent: String,
        #[arg(long)]
        no_sandbox: bool,
        #[arg(long, default_value_t = 0)]
        growth_per_compression: u32,
        #[arg(long, default_value_t = 16)]
        patience: u32,
        #[arg(long)]
        no_reviewer: bool,
        #[arg(long, default_value_t = 60)]
        agent_minutes: u64,
    },
    /// Run the loop (resumable: it continues from the state directory).
    Run {
        #[arg(long)]
        state: PathBuf,
        /// Run a single round and stop.
        #[arg(long)]
        once: bool,
    },
    /// Print a summary of the loop's history.
    Status {
        #[arg(long)]
        state: PathBuf,
    },
    /// Write the main result chart (SVG) from the loop's history.
    Chart {
        #[arg(long)]
        state: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

/// Reads a JSON-lines file.
pub fn read_jsonl<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, l)| {
            serde_json::from_str(l).map_err(|e| format!("{}:{}: {e}", path.display(), i + 1))
        })
        .collect()
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result: Result<(), String> = match cli.command {
        Command::Score {
            labels,
            outcomes,
            sources,
            json,
        } => (|| {
            let labels: Vec<score::Label> = read_jsonl(&labels)?;
            let mut outcomes: Vec<Outcome> = read_jsonl(&outcomes)?;
            if let Some(sources) = sources {
                let home = std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_default();
                let places = verify::Places::under(&home.join(".cache/glassmind/score-check"));
                outcomes = verify::remedies(&places, &labels, outcomes, &sources);
            }
            let report = score::report(&labels, &outcomes);
            if json {
                say!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
                );
            } else {
                print_totals("total", &report.total);
                for (name, t) in &report.by_split {
                    print_totals(&format!("split {name}"), t);
                }
                for (name, t) in &report.by_generator {
                    print_totals(&format!("generator {name}"), t);
                }
                for (name, t) in &report.by_kind {
                    print_totals(&format!("kind {name}"), t);
                }
            }
            Ok(())
        })(),
        Command::Compare {
            labels,
            before,
            after,
        } => (|| -> Result<(), String> {
            let labels: Vec<score::Label> = read_jsonl(&labels)?;
            let b = evaluate::verdicts(&labels, &read_jsonl::<Outcome>(&before)?);
            let a = evaluate::verdicts(&labels, &read_jsonl::<Outcome>(&after)?);
            let (mut better, mut worse) = (0, 0);
            for (id, (vb, cb)) in &b {
                let Some((va, ca)) = a.get(id) else { continue };
                if (ca - cb).abs() > 1e-9 {
                    if ca < cb {
                        better += 1;
                    } else {
                        worse += 1;
                    }
                    say!(
                        "{} {id}: {vb:?} {cb:.2} -> {va:?} {ca:.2}",
                        if ca < cb { "better" } else { "WORSE " }
                    );
                }
            }
            let tb: f64 = b.values().map(|(_, c)| c).sum();
            let ta: f64 = a.values().map(|(_, c)| c).sum();
            say!("{better} turns better, {worse} worse; cost {tb:.1} -> {ta:.1}");
            Ok(())
        })(),
        Command::Init {
            state: dir,
            repo,
            branch,
            sources,
            heldout_sources,
            train,
            heldout,
            valid,
            private,
            size_price,
            max_agents,
            model,
            agent,
            no_sandbox,
            growth_per_compression,
            patience,
            no_reviewer,
            agent_minutes,
        } => {
            let abs = |p: PathBuf| std::fs::canonicalize(&p).unwrap_or(p);
            let config = state::Config {
                repo: abs(repo),
                branch,
                sources: abs(sources),
                heldout_sources: abs(heldout_sources),
                train: abs(train),
                heldout: abs(heldout),
                valid: valid.map(abs),
                private: private.into_iter().map(abs).collect(),
                focus: Vec::new(),
                upstream: None,
                eval_jobs: 8,
                size_price,
                max_agents: max_agents.min(4),
                model,
                agent,
                sandbox: !no_sandbox,
                growth_per_compression,
                patience,
                reviewer: !no_reviewer,
                agent_minutes,
            };
            state::write_json(&state::Paths::new(&dir).config(), &config)
        }
        Command::Run { state: dir, once } => rounds::run(&state::Paths::new(&dir), once),
        Command::Status { state: dir } => {
            let paths = state::Paths::new(&dir);
            for e in state::history(&paths) {
                say!(
                    "round {:>3} {:<11} {:<8} train {:>8.1} ({:>4}/{:<4} right) held-out {:>7.1} ({:>3}/{:<3}) size {:>6} tokens {:>10}  {}",
                    e.round,
                    e.kind,
                    if e.accepted { "ACCEPT" } else { "reject" },
                    e.scores.train_cost,
                    e.scores.train_right,
                    e.scores.train_turns,
                    e.scores.heldout_cost,
                    e.scores.heldout_right,
                    e.scores.heldout_turns,
                    e.scores.size,
                    e.tokens_cumulative,
                    if e.accepted {
                        e.group.clone()
                    } else {
                        format!(
                            "{}: {}",
                            e.group,
                            e.reason.lines().next().unwrap_or_default()
                        )
                    }
                );
            }
            Ok(())
        }
        Command::Chart { state: dir, out } => chart::write(&state::Paths::new(&dir), &out),
        Command::Size { repo } => {
            let (per, total) = size::mind_size(&repo);
            for (krate, n) in per {
                say!("{n:>8}  {krate}");
            }
            say!("{total:>8}  total");
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("glassmind-loop: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_totals(name: &str, t: &score::Totals) {
    say!(
        "{name:<36} {:>5} turns  {:>5} right  {:>5} refused  {:>4} wrong  {:>4} wordless  cost {:>8.1}  ({:.1}% right)",
        t.turns,
        t.right,
        t.refused,
        t.wrong,
        t.wordless,
        t.cost,
        100.0 * t.accuracy()
    );
}
