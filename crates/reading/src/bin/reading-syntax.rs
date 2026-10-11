// SPDX-License-Identifier: MIT
//! Standalone TRAIN adapter and forms/text-only syntax inference. No QA actions.
use clap::{Parser, Subcommand};
use glassmind_reading::syntax::{Input, Model, TrainingSentence, VERSION, train};
use serde::Serialize;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
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
        train: PathBuf,
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        report: PathBuf,
    },
    Predict {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}
#[derive(Default, Serialize)]
struct Rows {
    integer: usize,
    range: usize,
    decimal: usize,
}
fn corpus(path: PathBuf) -> Result<(Vec<TrainingSentence>, Rows), String> {
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if f.metadata().map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
        return Err("TRAIN exceeds32MiB".into());
    }
    let mut out = Vec::new();
    let mut counts = Rows::default();
    let empty = || TrainingSentence {
        forms: Vec::new(),
        upos: Vec::new(),
        heads: Vec::new(),
        labels: Vec::new(),
    };
    let mut s = empty();
    for line in BufReader::new(f).lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            if !s.forms.is_empty() {
                out.push(s);
                s = empty();
            }
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [
            id,
            form,
            _lemma,
            pos,
            _xpos,
            _features,
            head,
            label,
            _deps,
            _misc,
        ] = fields.as_slice()
        else {
            return Err("expected ten CoNLL-U columns".into());
        };
        if id.contains('-') {
            counts.range += 1;
            continue;
        }
        if id.contains('.') {
            counts.decimal += 1;
            continue;
        }
        if id.parse::<usize>().map_err(|e| e.to_string())? != s.forms.len() + 1 {
            return Err("noncontiguous integer IDs".into());
        }
        s.forms.push((*form).into());
        s.upos.push((*pos).into());
        s.heads.push(
            head.parse()
                .map_err(|e: std::num::ParseIntError| e.to_string())?,
        );
        s.labels.push((*label).into());
        counts.integer += 1;
    }
    if !s.forms.is_empty() {
        out.push(s);
    }
    Ok((out, counts))
}
fn run() -> Result<(), String> {
    match Args::parse().command {
        Command::Train {
            train: source,
            model,
            report,
        } => {
            let (data, rows) = corpus(source)?;
            let (m, r) = train(&data)?;
            let mut writer =
                BufWriter::new(std::fs::File::create(model).map_err(|e| e.to_string())?);
            m.save(&mut writer)?;
            writer.flush().map_err(|e| e.to_string())?;
            std::fs::write(
                report,
                serde_json::to_vec_pretty(
                    &serde_json::json!({"version":VERSION,"training":r,"conllu_rows":rows}),
                )
                .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }
        Command::Predict {
            model,
            input,
            output,
        } => {
            let model = Model::load(BufReader::new(
                std::fs::File::open(model).map_err(|e| e.to_string())?,
            ))?;
            let mut reader = BufReader::new(std::fs::File::open(input).map_err(|e| e.to_string())?);
            let mut writer =
                BufWriter::new(std::fs::File::create(output).map_err(|e| e.to_string())?);
            loop {
                let mut line = String::new();
                let n = reader
                    .by_ref()
                    .take(128 * 1024 + 1)
                    .read_line(&mut line)
                    .map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                if n > 128 * 1024 {
                    return Err("JSONL row exceeds128KiB".into());
                }
                let result = serde_json::from_str::<Input>(&line)
                    .map_err(|e| e.to_string())
                    .and_then(|i| model.predict(i));
                let value = match result {
                    Ok(p) => serde_json::to_value(p).map_err(|e| e.to_string())?,
                    Err(reason) => {
                        serde_json::json!({"status":"refused","model_version":VERSION,"reason":reason})
                    }
                };
                serde_json::to_writer(&mut writer, &value).map_err(|e| e.to_string())?;
                writeln!(writer).map_err(|e| e.to_string())?;
            }
            writer.flush().map_err(|e| e.to_string())?;
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
