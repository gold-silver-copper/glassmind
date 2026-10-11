//! Standalone multi-service API; legacy command semantics remain unchanged.
use clap::{Parser, Subcommand};
use glassmind_dialogue::{
    adapter::{Dialogue, Schema},
    multi::{MultiModel, MultiSession},
    tracker::{DialogueModel, InputTurn, Speaker},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Train {
        #[arg(long)]
        data: PathBuf,
        #[arg(long)]
        schema: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        frozen: Vec<PathBuf>,
        #[arg(long, default_value_t = 12)]
        epochs: usize,
        #[arg(long, default_value_t = 12)]
        router_epochs: usize,
        #[arg(long)]
        independent_router: bool,
    },
    /// Batch of independent conversations, each an array of speaker/utterance turns.
    Predict {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        schema: PathBuf,
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}
fn read<T: DeserializeOwned>(p: &PathBuf) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_reader(BufReader::new(File::open(p)?))?)
}
fn write<T: Serialize>(p: &PathBuf, value: &T) -> Result<(), Box<dyn std::error::Error>> {
    Ok(serde_json::to_writer(
        BufWriter::new(File::create(p)?),
        value,
    )?)
}
#[derive(Serialize)]
struct Snapshot {
    turn_index: usize,
    active_services: BTreeSet<String>,
    frames: Vec<Frame>,
}
#[derive(Serialize)]
struct Frame {
    service: String,
    state: State,
}
#[derive(Serialize)]
struct State {
    active_intent: String,
    slot_values: BTreeMap<String, Vec<String>>,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::Train {
            data,
            schema,
            output,
            frozen,
            epochs,
            router_epochs,
            independent_router,
        } => {
            let schemas: Vec<Schema> = read(&schema)?;
            let dialogues: Vec<Dialogue> = read(&data)?;
            let mut services = BTreeMap::new();
            for path in frozen {
                let m: DialogueModel = read(&path)?;
                services.insert(m.service.clone(), m);
            }
            for schema in &schemas {
                if !services.contains_key(&schema.service_name) {
                    eprintln!("training {}", schema.service_name);
                    services.insert(
                        schema.service_name.clone(),
                        DialogueModel::train(&dialogues, schema, epochs)?,
                    );
                }
            }
            let selected = schemas.iter().map(|s| s.service_name.clone()).collect();
            let router = MultiModel::train_router(&dialogues, &selected, router_epochs)?;
            let independent_router = if independent_router {
                MultiModel::train_independent_router(&dialogues, &selected, router_epochs)?
            } else {
                BTreeMap::new()
            };
            let model = MultiModel {
                version: 1,
                router,
                independent_router,
                references: None,
                services,
            };
            model.validate(&schemas)?;
            write(&output, &model)?;
        }
        Command::Predict {
            input,
            schema,
            model,
            output,
        } => {
            let schemas: Vec<Schema> = read(&schema)?;
            let model: MultiModel = read(&model)?;
            model.validate(&schemas)?;
            let conversations: Vec<Vec<InputTurn>> = read(&input)?;
            let mut results = Vec::new();
            for conversation in conversations {
                let mut session = MultiSession::default();
                let mut predictions = Vec::new();
                for (turn_index, turn) in conversation.iter().enumerate() {
                    model.advance(&mut session, turn, &schemas);
                    if matches!(turn.speaker, Speaker::User) {
                        let frames = session
                            .states
                            .iter()
                            .map(|(service, state)| Frame {
                                service: service.clone(),
                                state: State {
                                    active_intent: state
                                        .active_intent
                                        .clone()
                                        .unwrap_or_else(|| "NONE".into()),
                                    slot_values: state
                                        .slots
                                        .iter()
                                        .map(|(s, v)| (s.clone(), vec![v.text().into()]))
                                        .collect(),
                                },
                            })
                            .collect();
                        predictions.push(Snapshot {
                            turn_index,
                            active_services: session.active_services.clone(),
                            frames,
                        });
                    }
                }
                results.push(predictions);
            }
            write(&output, &results)?;
        }
    }
    Ok(())
}
