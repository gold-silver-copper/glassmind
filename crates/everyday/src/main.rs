use clap::{Parser, Subcommand};
use glassmind_everyday::{Conversation, Example, IntentModel, Totals, TrainOptions, adapter};
use serde::Serialize;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, IsTerminal, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(about = "Learned everyday intents and typed chat memory; no runtime LLM")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Train {
        #[arg(long)]
        train: PathBuf,
        #[arg(long)]
        dev: PathBuf,
        #[arg(long)]
        model: PathBuf,
        #[arg(long, default_value_t = 20)]
        epochs: usize,
        #[arg(long, default_value_t = 0.3)]
        char_weight: f32,
        #[arg(long, default_value_t = 2.0)]
        wrong_cost: f32,
        /// Fixed first N rows per label for calibration (0 = all validation).
        #[arg(long, default_value_t = 3)]
        per_label: usize,
    },
    Calibrate {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        dev: PathBuf,
        #[arg(long, default_value_t = 2.0)]
        wrong_cost: f32,
    },
    Eval {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        dev: PathBuf,
        /// Fixed first N rows per label (0 = all validation).
        #[arg(long, default_value_t = 3)]
        per_label: usize,
    },
    Intent {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        explain: bool,
        text: Vec<String>,
    },
    Chat {
        #[arg(long)]
        model: Option<PathBuf>,
        /// Optional conversation state, persisted after each turn.
        #[arg(long)]
        session: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

fn read_data(path: &Path) -> Result<Vec<Example>, Box<dyn std::error::Error>> {
    // The CLI deliberately names only train/dev. Split isolation is guaranteed
    // by the fetcher; custom JSONL callers are responsible for their own data.
    Ok(adapter::read_examples(BufReader::new(File::open(path)?))?)
}

fn load_model(path: &Path) -> Result<IntentModel, Box<dyn std::error::Error>> {
    let model: IntentModel = serde_json::from_reader(BufReader::new(File::open(path)?))?;
    model.validate()?;
    Ok(model)
}

fn save<T: Serialize>(path: &Path, value: &T) -> Result<(), Box<dyn std::error::Error>> {
    let temporary = path.with_extension(format!("{}.pending", std::process::id()));
    let mut writer = BufWriter::new(File::create(&temporary)?);
    serde_json::to_writer(&mut writer, value)?;
    writer.flush()?;
    fs::rename(temporary, path)?;
    Ok(())
}

fn output(value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer_pretty(io::stdout().lock(), value)?;
    println!();
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::Train {
            train,
            dev,
            model,
            epochs,
            char_weight,
            wrong_cost,
            per_label,
        } => {
            let training = read_data(&train)?;
            let validation = read_data(&dev)?;
            let validation = if per_label == 0 {
                validation
            } else {
                adapter::fixed_sample(&validation, per_label)
            };
            let mut fitted = IntentModel::train(
                &training,
                TrainOptions {
                    epochs,
                    char_weight,
                    min_df: 2,
                },
            )?;
            let totals = fitted.calibrate(&validation, wrong_cost)?;
            save(&model, &fitted)?;
            output(
                &serde_json::json!({ "training_rows": fitted.training_rows, "labels": fitted.labels.len(), "features": fitted.feature_count(), "policy": fitted.policy, "dev_selection": totals }),
            )?;
        }
        Command::Calibrate {
            model,
            dev,
            wrong_cost,
        } => {
            let mut fitted = load_model(&model)?;
            let totals = fitted.calibrate(&read_data(&dev)?, wrong_cost)?;
            save(&model, &fitted)?;
            output(&serde_json::json!({ "policy": fitted.policy, "dev_selection": totals }))?;
        }
        Command::Eval {
            model,
            dev,
            per_label,
        } => {
            let fitted = load_model(&model)?;
            let rows = read_data(&dev)?;
            let rows = if per_label == 0 {
                rows
            } else {
                adapter::fixed_sample(&rows, per_label)
            };
            let mut totals = Totals::default();
            for row in rows {
                totals.observe(&row.label, &fitted.predict(&row.text));
            }
            output(&totals)?;
        }
        Command::Intent {
            model,
            explain,
            text,
        } => {
            if text.is_empty() {
                return Err("supply request text".into());
            }
            let fitted = load_model(&model)?;
            let text = text.join(" ");
            output(
                &serde_json::json!({"prediction": fitted.predict(&text), "contributions": if explain { fitted.explain(&text, 20) } else { vec![] }}),
            )?;
        }
        Command::Chat {
            model,
            session,
            json,
        } => {
            let model = model.as_deref().map(load_model).transpose()?;
            let mut conversation: Conversation = match session.as_deref() {
                Some(path) if path.exists() => {
                    serde_json::from_reader(BufReader::new(File::open(path)?))?
                }
                _ => Conversation::default(),
            };
            let interactive = io::stdin().is_terminal();
            if interactive {
                eprintln!("Everyday chat. /quit to leave; /reset to clear session memory.");
            }
            let mut lines = io::stdin().lock().lines();
            loop {
                if interactive {
                    print!("> ");
                    io::stdout().flush()?;
                }
                let Some(line) = lines.next() else {
                    break;
                };
                let line = line?;
                if line.trim() == "/quit" {
                    break;
                }
                if line.trim().is_empty() {
                    continue;
                }
                if line.len() > 8192 {
                    eprintln!("Please keep each turn below 8192 bytes.");
                    continue;
                }
                if line.trim() == "/reset" {
                    conversation = Conversation::default();
                    if let Some(path) = &session {
                        save(path, &conversation)?;
                    }
                    if json {
                        println!("{{\"kind\":\"reset\"}}");
                    } else {
                        println!("Memory cleared.");
                    }
                    continue;
                }
                let reply = conversation.respond(&line, model.as_ref());
                if json {
                    serde_json::to_writer(io::stdout().lock(), &reply)?;
                    println!();
                } else {
                    println!("{}", reply.text());
                }
                if let Some(path) = &session {
                    save(path, &conversation)?;
                }
            }
        }
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
