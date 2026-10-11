//! Totals-only reading and grammatical-preference baselines. No text model yet.
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use glassmind_english::English;
use glassmind_meaning::interface::{Language, Understanding};
use glassmind_rules::Switches;
use glassmind_world_rust::Rust;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Args {
    /// JSONL rows: {"text":...} or {"good":...,"bad":...}, optional group.
    #[arg(long)]
    input: PathBuf,
    /// Longer inputs count as uncovered/tied, and are reported separately.
    #[arg(long, default_value_t = 48)]
    max_tokens: usize,
}

#[derive(Deserialize)]
struct Item {
    #[serde(default)]
    group: String,
    text: Option<String>,
    good: Option<String>,
    bad: Option<String>,
}

#[derive(Default, Serialize)]
struct Totals {
    sentences: usize,
    parsed: usize,
    /// Parsing into a meaning does not establish that meaning's correctness.
    pairs: usize,
    preferred_good: usize,
    preferred_bad: usize,
    tied: usize,
    too_long_sentences: usize,
}

fn parses(english: &English<Rust>, text: &str, max: usize, totals: &mut Totals) -> bool {
    if english.tokens(text).len() > max {
        totals.too_long_sentences += 1;
        return false;
    }
    matches!(english.understand(text), Understanding::Readings(readings)
        if readings.iter().any(|r| r.edits == 0 && r.idle == 0 && r.discarded.is_empty()))
}

fn run(args: Args) -> Result<(), String> {
    // A corrected sentence or a guessed code-request template is not evidence
    // for the grammaticality of the original sentence.
    let switches = Switches::without([
        "en.normalizer.edits",
        "en.intents.template",
        "en.typos.edit",
    ]);
    let english = English::<Rust>::new(glassmind_english_rust::rules(), &switches);
    let file = std::fs::File::open(&args.input).map_err(|e| e.to_string())?;
    let mut totals = BTreeMap::<String, Totals>::new();
    for (n, line) in std::io::BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let row: Item =
            serde_json::from_str(&line).map_err(|_| format!("invalid input row {}", n + 1))?;
        let total = totals.entry(row.group).or_default();
        match (row.text, row.good, row.bad) {
            (Some(text), None, None) => {
                total.sentences += 1;
                total.parsed += usize::from(parses(&english, &text, args.max_tokens, total));
            }
            (None, Some(good), Some(bad)) => {
                total.pairs += 1;
                let before = total.too_long_sentences;
                let g = parses(&english, &good, args.max_tokens, total);
                let b = parses(&english, &bad, args.max_tokens, total);
                if total.too_long_sentences > before {
                    total.tied += 1;
                } else {
                    match (g, b) {
                        (true, false) => total.preferred_good += 1,
                        (false, true) => total.preferred_bad += 1,
                        _ => total.tied += 1,
                    }
                }
            }
            _ => return Err(format!("row {} needs text or a good/bad pair", n + 1)),
        }
    }
    serde_json::to_writer_pretty(std::io::stdout(), &totals).map_err(|e| e.to_string())?;
    writeln!(std::io::stdout()).map_err(|e| e.to_string())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "{e}");
            ExitCode::FAILURE
        }
    }
}
