use clap::{Parser, Subcommand};
use glassmind_everyday::{Example, IntentModel, TrainOptions};
use glassmind_facts::{EntityId, Store};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Match positive conditions whose recorded intervals strictly contain a day.
    MatchAt {
        #[arg(long)]
        index: PathBuf,
        #[arg(long)]
        temporal: PathBuf,
        #[arg(long)]
        date: String,
        question: String,
    },
    /// Evidence that one recorded bounded interval strictly contains a calendar day.
    VerifyAt {
        #[arg(long)]
        index: PathBuf,
        #[arg(long)]
        temporal: PathBuf,
        #[arg(long)]
        date: String,
        question: String,
    },
    /// Match two or three positive shared-subject conditions in this snapshot.
    Match {
        #[arg(long)]
        index: PathBuf,
        question: String,
    },
    /// Check an explicit positive relation against this finite local snapshot.
    Verify {
        #[arg(long)]
        index: PathBuf,
        question: String,
    },
    Train {
        #[arg(long)]
        train: PathBuf,
        #[arg(long)]
        model: PathBuf,
    },
    Score {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        index: PathBuf,
        #[arg(long)]
        limit: Option<usize>,
    },
    Ask {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        index: PathBuf,
        question: String,
    },
}
struct Row {
    subject: String,
    relation: String,
    object: String,
    question: String,
}
fn rows(path: &PathBuf) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
    fs::read_to_string(path)?
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let mut parts = line.splitn(4, '\t');
            let subject = parts.next().ok_or("missing subject")?.to_owned();
            let relation = parts.next().ok_or("missing relation")?.to_owned();
            let object = parts.next().ok_or("missing object")?.to_owned();
            let question = parts
                .next()
                .ok_or_else(|| format!("missing question on row {}", i + 1))?
                .to_owned();
            Ok(Row {
                subject,
                relation,
                object,
                question,
            })
        })
        .collect()
}
fn load(
    model: &PathBuf,
    index: &PathBuf,
) -> Result<(IntentModel, Store), Box<dyn std::error::Error>> {
    let model: IntentModel = serde_json::from_slice(&fs::read(model)?)?;
    let store: Store = serde_json::from_slice(&fs::read(index)?)?;
    model.validate()?;
    store.validate()?;
    Ok((model, store))
}
#[derive(Default, Serialize)]
struct Totals {
    total: usize,
    right: usize,
    refused: usize,
    wrong: usize,
    entity_linked_correct: usize,
    relation_top1_correct: usize,
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Args::parse().command {
        Command::MatchAt {
            index,
            temporal,
            date,
            question,
        } => {
            use glassmind_facts::temporal::{
                MAX_TEMPORAL_BYTES, MAX_TEMPORAL_INDEX_BYTES, TemporalIndex,
            };
            fn bounded(path: PathBuf, cap: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
                use std::io::Read;
                let mut bytes = Vec::new();
                fs::File::open(path)?
                    .take((cap + 1) as u64)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > cap {
                    return Err("input byte cap exceeded".into());
                }
                Ok(bytes)
            }
            let store = TemporalIndex::from_bytes(
                &bounded(index, MAX_TEMPORAL_INDEX_BYTES)?,
                &bounded(temporal, MAX_TEMPORAL_BYTES)?,
            )?;
            println!(
                "{}",
                serde_json::to_string(&store.match_at_str(&date, &question))?
            );
        }
        Command::VerifyAt {
            index,
            temporal,
            date,
            question,
        } => {
            use glassmind_facts::temporal::{
                MAX_TEMPORAL_BYTES, MAX_TEMPORAL_INDEX_BYTES, TemporalIndex,
            };
            fn bounded(path: PathBuf, cap: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
                use std::io::Read;
                let mut bytes = Vec::new();
                fs::File::open(path)?
                    .take((cap + 1) as u64)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > cap {
                    return Err("input byte cap exceeded".into());
                }
                Ok(bytes)
            }
            let store = TemporalIndex::from_bytes(
                &bounded(index, MAX_TEMPORAL_INDEX_BYTES)?,
                &bounded(temporal, MAX_TEMPORAL_BYTES)?,
            )?;
            println!(
                "{}",
                serde_json::to_string(&store.verify_at_str(&date, &question))?
            );
        }
        Command::Match { index, question } => {
            let bytes = fs::read(index)?;
            let store: Store = serde_json::from_slice(&bytes)?;
            store.validate()?;
            #[derive(Serialize)]
            struct Reply {
                #[serde(flatten)]
                result: glassmind_facts::MatchResult,
                index_sha256: String,
            }
            println!(
                "{}",
                serde_json::to_string(&Reply {
                    result: store.match_question(&question),
                    index_sha256: format!("{:x}", Sha256::digest(&bytes)),
                })?
            );
        }
        Command::Verify { index, question } => {
            let store: Store = serde_json::from_slice(&fs::read(index)?)?;
            store.validate()?;
            println!("{}", serde_json::to_string(&store.verify(&question))?);
        }
        Command::Train { train, model } => {
            let rows = rows(&train)?;
            // Deterministic training-only calibration, no validation examples.
            let (calibration, fit): (Vec<_>, Vec<_>) =
                rows.into_iter().enumerate().partition(|(i, _)| i % 10 == 0);
            let examples = |rows: Vec<(usize, Row)>| {
                rows.into_iter()
                    .map(|(_, r)| Example {
                        text: r.question,
                        label: r.relation,
                    })
                    .collect::<Vec<_>>()
            };
            let fit = examples(fit);
            let calibration = examples(calibration);
            let mut trained = IntentModel::train(
                &fit,
                TrainOptions {
                    epochs: 12,
                    char_weight: 0.15,
                    min_df: 2,
                },
            )?;
            let totals = trained.calibrate(&calibration, 2.0)?;
            fs::write(model, serde_json::to_vec(&trained)?)?;
            println!("{}", serde_json::to_string(&totals)?);
        }
        Command::Score {
            input,
            model,
            index,
            limit,
        } => {
            let (model, store) = load(&model, &index)?;
            let mut totals = Totals::default();
            for row in rows(&input)?.into_iter().take(limit.unwrap_or(usize::MAX)) {
                let answer = store.answer(&model, &row.question);
                totals.total += 1;
                totals.entity_linked_correct +=
                    usize::from(answer.entity.as_ref().is_some_and(|id| id.0 == row.subject));
                totals.relation_top1_correct +=
                    usize::from(model.predict(&row.question).label == row.relation);
                // Multiple answers count wrong, even if one matches the benchmark answer.
                if answer.answers.len() == 1 && answer.answers.contains(&EntityId(row.object)) {
                    totals.right += 1;
                } else if answer.refusal.is_some() {
                    totals.refused += 1;
                } else {
                    totals.wrong += 1;
                }
            }
            println!("{}", serde_json::to_string(&totals)?);
        }
        Command::Ask {
            model,
            index,
            question,
        } => {
            let (model, store) = load(&model, &index)?;
            println!(
                "{}",
                serde_json::to_string(&store.answer(&model, &question))?
            );
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
