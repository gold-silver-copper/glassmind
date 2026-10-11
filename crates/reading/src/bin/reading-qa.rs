// SPDX-License-Identifier: MIT
//! Offline trainer and strict context/question-only batch inference adapter.
use clap::{Parser, Subcommand};
use glassmind_reading::qa::{self, Input, Model, TrainingRow};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Train {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        model: PathBuf,
    },
    Predict {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        model: Option<PathBuf>,
    },
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Args::parse().command {
        Command::Train { input, model } => {
            let rows: Vec<TrainingRow> = BufReader::new(File::open(input)?)
                .lines()
                .map(|l| Ok(serde_json::from_str(&l?)?))
                .collect::<Result<_, Box<dyn std::error::Error>>>()?;
            let m = Model::train(&rows);
            m.validate()?;
            serde_json::to_writer(BufWriter::new(File::create(model)?), &m)?;
        }
        Command::Predict { input, model } => {
            let model: Option<Model> = model
                .map(|p| -> Result<_, Box<dyn std::error::Error>> {
                    Ok(serde_json::from_reader(BufReader::new(File::open(p)?))?)
                })
                .transpose()?;
            if let Some(m) = &model {
                m.validate()?;
            }
            let mut out = BufWriter::new(std::io::stdout().lock());
            for line in BufReader::new(File::open(input)?).lines() {
                let row: Input = serde_json::from_str(&line?)?;
                let result = model.as_ref().map_or_else(
                    || qa::baseline(&row.context, &row.question),
                    |m| m.predict(&row.context, &row.question),
                );
                serde_json::to_writer(&mut out, &result)?;
                writeln!(out)?;
            }
        }
    }
    Ok(())
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            std::process::ExitCode::FAILURE
        }
    }
}
