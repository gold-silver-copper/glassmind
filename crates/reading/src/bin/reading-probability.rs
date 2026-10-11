//! Separate fixed probability training/evaluation; the parse-only CLI is unchanged.
#[path = "../probability.rs"]
mod probability;

use clap::{Parser, Subcommand};
use probability::Model;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Train the fixed byte 6-gram and its unigram baseline on LF-delimited files.
    Train {
        #[arg(long, required = true, num_args = 1..)]
        input: Vec<PathBuf>,
        #[arg(long)]
        model: PathBuf,
    },
    /// Compare fixed unigram/six-gram probabilities on all supplied JSONL rows.
    Evaluate {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        input: PathBuf,
    },
    /// Score one UTF-8 string with both orders; emits bits including EOS.
    Score {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        text: String,
    },
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
struct Metrics {
    good: u64,
    bad: u64,
    ties: u64,
    bits: f64,
    bits_per_byte: Option<f64>,
}
#[derive(Default, Serialize)]
struct Totals {
    pairs: u64,
    documents: u64,
    bytes: u64,
    unigram: Metrics,
    sixgram: Metrics,
    /// Rows: unigram good/bad/tie; columns: sixgram good/bad/tie.
    transitions: [[u64; 3]; 3],
    documents_lower_bits: u64,
    documents_higher_bits: u64,
    documents_equal_bits: u64,
}

fn preference(metrics: &mut Metrics, good: f64, bad: f64) -> usize {
    if (good - bad).abs() <= 1e-9 {
        metrics.ties += 1;
        2
    } else if good < bad {
        metrics.good += 1;
        0
    } else {
        metrics.bad += 1;
        1
    }
}

impl Totals {
    fn add(&mut self, row: &Item, model: &Model) -> Result<(), String> {
        match (&row.text, &row.good, &row.bad) {
            (Some(text), None, None) => {
                self.documents += 1;
                self.bytes += text.len() as u64;
                let u = model.bits(text.as_bytes(), false);
                let s = model.bits(text.as_bytes(), true);
                self.unigram.bits += u;
                self.sixgram.bits += s;
                if (u - s).abs() <= 1e-9 {
                    self.documents_equal_bits += 1;
                } else if s < u {
                    self.documents_lower_bits += 1;
                } else {
                    self.documents_higher_bits += 1;
                }
            }
            (None, Some(good), Some(bad)) => {
                self.pairs += 1;
                let u = preference(
                    &mut self.unigram,
                    model.bits(good.as_bytes(), false),
                    model.bits(bad.as_bytes(), false),
                );
                let s = preference(
                    &mut self.sixgram,
                    model.bits(good.as_bytes(), true),
                    model.bits(bad.as_bytes(), true),
                );
                if let Some(cell) = self.transitions.get_mut(u).and_then(|r| r.get_mut(s)) {
                    *cell += 1;
                }
            }
            _ => return Err("expected exactly text or good/bad pair".into()),
        }
        if self.bytes > 0 {
            self.unigram.bits_per_byte = Some(self.unigram.bits / self.bytes as f64);
            self.sixgram.bits_per_byte = Some(self.sixgram.bits / self.bytes as f64);
        }
        Ok(())
    }
}

fn json(value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = io::stdout().lock();
    serde_json::to_writer_pretty(&mut output, value)?;
    writeln!(output)?;
    Ok(())
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        Command::Train { input, model } => {
            let mut trained = Model::default();
            for path in input {
                let mut reader = BufReader::new(File::open(&path)?);
                let mut line = Vec::new();
                loop {
                    line.clear();
                    if reader.read_until(b'\n', &mut line)? == 0 {
                        break;
                    }
                    if line.last() == Some(&b'\n') {
                        line.pop();
                        if line.last() == Some(&b'\r') {
                            line.pop();
                        }
                    }
                    trained.observe(&line);
                }
                eprintln!(
                    "trained {}: {} documents, {} bytes, {} entries cumulative",
                    path.display(),
                    trained.documents,
                    trained.bytes,
                    trained.entries()
                );
            }
            trained.finish()?;
            trained.save(BufWriter::new(File::create(model)?))?;
            json(
                &serde_json::json!({"documents":trained.documents,"bytes":trained.bytes,"entries":trained.entries(),"order":6}),
            )
        }
        Command::Evaluate { model, input } => {
            let model = Model::load(BufReader::new(File::open(model)?))?;
            let mut total = Totals::default();
            let mut groups = BTreeMap::<String, Totals>::new();
            for (index, line) in BufReader::new(File::open(input)?).lines().enumerate() {
                let row: Item =
                    serde_json::from_str(&line?).map_err(|e| format!("row {}: {e}", index + 1))?;
                total.add(&row, &model)?;
                groups
                    .entry(row.group.clone())
                    .or_default()
                    .add(&row, &model)?;
            }
            json(
                &serde_json::json!({"total":total,"groups":groups,"policy":"raw UTF-8; BOS context only; EOS scored; pair total bits; document bits / text bytes"}),
            )
        }
        Command::Score { model, text } => {
            let model = Model::load(BufReader::new(File::open(model)?))?;
            json(
                &serde_json::json!({"bytes":text.len(),"unigram_bits":model.bits(text.as_bytes(),false),"sixgram_bits":model.bits(text.as_bytes(),true)}),
            )
        }
    }
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            ExitCode::FAILURE
        }
    }
}
