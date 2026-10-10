//! `glassmind-loop`: the improvement loop and its measuring tools.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use glassmind_answer::Outcome;
use serde::de::DeserializeOwned;

mod score;
mod size;

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
enum Command {
    /// Score outcomes (from `glassmind eval`) against labels.
    Score {
        #[arg(long)]
        labels: PathBuf,
        #[arg(long)]
        outcomes: PathBuf,
        /// Print the full report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Count the mind crates' size in syntax tokens.
    Size {
        #[arg(long, default_value = ".")]
        repo: PathBuf,
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
            json,
        } => (|| {
            let labels: Vec<score::Label> = read_jsonl(&labels)?;
            let outcomes: Vec<Outcome> = read_jsonl(&outcomes)?;
            let report = score::report(&labels, &outcomes);
            if json {
                println!(
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
        Command::Size { repo } => {
            let (per, total) = size::mind_size(&repo);
            for (krate, n) in per {
                println!("{n:>8}  {krate}");
            }
            println!("{total:>8}  total");
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
    println!(
        "{name:<36} {:>5} turns  {:>5} right  {:>5} refused  {:>4} wrong  cost {:>8.1}  ({:.1}% right)",
        t.turns,
        t.right,
        t.refused,
        t.wrong,
        t.cost,
        100.0 * t.accuracy()
    );
}
