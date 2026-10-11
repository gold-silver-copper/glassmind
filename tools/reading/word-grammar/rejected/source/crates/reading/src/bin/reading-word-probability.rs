// SPDX-License-Identifier: MIT OR Apache-2.0
//! Ownership-local TRAIN and fixed-ranking adapter. No generation interface.
// Original scorer is included unchanged; training/persistence APIs are unused here.
#[allow(dead_code)]
#[path = "../probability.rs"]
mod probability;
#[path = "../word_probability.rs"]
mod word_probability;

use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use word_probability::{Model, tokenize};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Train {
        #[arg(long, required = true, num_args = 1..)]
        input: Vec<PathBuf>,
        #[arg(long)]
        model: PathBuf,
    },
    Evaluate {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        input: PathBuf,
    },
    Score {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        text: String,
    },
}
#[derive(Deserialize)]
struct Pair {
    #[serde(default)]
    id: serde_json::Value,
    group: String,
    good: String,
    bad: String,
}
fn preference(good: f64, bad: f64) -> &'static str {
    if (good - bad).abs() <= 1e-9 {
        "tie"
    } else if good < bad {
        "good"
    } else {
        "bad"
    }
}
fn documents(paths: &[PathBuf], mut f: impl FnMut(&str) -> io::Result<()>) -> io::Result<()> {
    for path in paths {
        let mut reader = BufReader::new(File::open(path)?);
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }
            f(&line)?;
        }
    }
    Ok(())
}
fn emit(out: &mut impl Write, value: &serde_json::Value) -> io::Result<()> {
    serde_json::to_writer(&mut *out, value)?;
    writeln!(out)
}
fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = BufWriter::new(io::stdout().lock());
    match args.command {
        Command::Train { input, model } => {
            let mut frequencies = HashMap::<String, u64>::new();
            documents(&input, |line| {
                for token in tokenize(line) {
                    *frequencies.entry(token).or_default() += 1;
                }
                Ok(())
            })?;
            let mut trained = Model::from_frequencies(frequencies)?;
            documents(&input, |line| trained.observe(line))?;
            trained.finish()?;
            trained.save(BufWriter::new(
                File::options().write(true).create_new(true).open(model)?,
            ))?;
            emit(
                &mut out,
                &serde_json::json!({"documents":trained.documents,"tokens":trained.tokens,
                "vocabulary":trained.vocabulary_size(),"entries":trained.entries(),"order":3}),
            )?;
        }
        Command::Evaluate {
            model,
            baseline,
            input,
        } => {
            let model = Model::load(BufReader::new(File::open(model)?))?;
            let baseline = probability::Model::load(BufReader::new(File::open(baseline)?))?;
            for (index, line) in BufReader::new(File::open(input)?).lines().enumerate() {
                let row: Pair =
                    serde_json::from_str(&line?).map_err(|e| format!("row {}: {e}", index + 1))?;
                let bg = baseline.bits(row.good.as_bytes(), true);
                let bb = baseline.bits(row.bad.as_bytes(), true);
                let cg = model.bits(&row.good);
                let cb = model.bits(&row.bad);
                if ![bg, bb, cg, cb].iter().all(|v| v.is_finite()) {
                    return Err("nonfinite cost".into());
                }
                emit(
                    &mut out,
                    &serde_json::json!({"row":index+1,"id":row.id,"group":row.group,
                    "baseline_good_bits":bg,"baseline_bad_bits":bb,"candidate_good_token_bits":cg,
                    "candidate_bad_token_bits":cb,"baseline_choice":preference(bg,bb),"candidate_choice":preference(cg,cb)}),
                )?;
            }
        }
        Command::Score { model, text } => {
            let model = Model::load(BufReader::new(File::open(model)?))?;
            emit(
                &mut out,
                &serde_json::json!({"token_bits_including_eos":model.bits(&text)}),
            )?;
        }
    }
    out.flush()?;
    Ok(())
}
fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            ExitCode::FAILURE
        }
    }
}
