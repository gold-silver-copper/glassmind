use clap::{Parser, Subcommand};
use glassmind_dialogue::adapter::{Dialogue, Schema};
use glassmind_dialogue::tracker::{DialogueModel, InputTurn, SessionState, Speaker};
use serde::{Serialize, de::DeserializeOwned};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

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
        #[arg(long, default_value_t = 12)]
        epochs: usize,
    },
    Eval {
        #[arg(long)]
        data: PathBuf,
        #[arg(long)]
        schema: PathBuf,
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        no_carry: bool,
    },
    /// Predict an annotation-free JSON array of turns as a new session.
    Predict {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        schema: PathBuf,
        #[arg(long)]
        model: PathBuf,
    },
}

fn read<T: DeserializeOwned>(p: &PathBuf) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_reader(BufReader::new(File::open(p)?))?)
}
fn write<T: Serialize>(p: &PathBuf, data: &T) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer(BufWriter::new(File::create(p)?), data)?;
    Ok(())
}
fn schema(p: &PathBuf) -> Result<Schema, Box<dyn std::error::Error>> {
    let mut schemas: Vec<Schema> = read(p)?;
    if schemas.len() != 1 {
        return Err("provide exactly one service schema".into());
    }
    schemas.pop().ok_or_else(|| "empty schema".into())
}

#[derive(Serialize)]
struct Prediction {
    dialogue_id: String,
    turn_index: usize,
    service: String,
    state: PredictedState,
}
#[derive(Serialize)]
struct PredictedState {
    active_intent: String,
    slot_values: std::collections::BTreeMap<String, Vec<String>>,
}
fn predicted(state: &SessionState) -> PredictedState {
    PredictedState {
        active_intent: state.active_intent.clone().unwrap_or_else(|| "NONE".into()),
        slot_values: state
            .slots
            .iter()
            .map(|(s, v)| (s.clone(), vec![v.text().into()]))
            .collect(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::Train {
            data,
            schema: path,
            output,
            epochs,
        } => {
            let s = schema(&path)?;
            let dialogues: Vec<Dialogue> = read(&data)?;
            let model = DialogueModel::train(&dialogues, &s, epochs)?;
            model.validate(&s)?;
            write(&output, &model)?;
            println!(
                "{}",
                serde_json::json!({"training_dialogues":model.training_dialogues,"training_frames":model.training_frames,"span_features":model.spans.feature_count()})
            );
        }
        Command::Eval {
            data,
            schema: path,
            model,
            output,
            limit,
            no_carry,
        } => {
            let s = schema(&path)?;
            let model: DialogueModel = read(&model)?;
            model.validate(&s)?;
            let mut dialogues: Vec<Dialogue> = read(&data)?;
            dialogues.sort_by(|a, b| a.dialogue_id.cmp(&b.dialogue_id));
            let mut predictions = Vec::new();
            let mut exact = 0;
            let mut intents = 0;
            let mut count = 0;
            for dialogue in dialogues.iter().take(limit.unwrap_or(usize::MAX)) {
                // Restriction is declared in advance; no gold frame chooses runtime routing.
                if dialogue.services != vec![s.service_name.clone()] {
                    return Err("evaluation requires the declared single-service slice".into());
                }
                count += 1;
                let mut session = SessionState::default();
                for (index, turn) in dialogue.turns.iter().enumerate() {
                    let input = InputTurn {
                        speaker: match turn.speaker.as_str() {
                            "USER" => Speaker::User,
                            "SYSTEM" => Speaker::System,
                            _ => return Err("invalid speaker".into()),
                        },
                        utterance: turn.utterance.clone(),
                    };
                    model.advance(&mut session, &input, &s, !no_carry);
                    if matches!(input.speaker, Speaker::System) {
                        continue;
                    }
                    let state = predicted(&session);
                    // Gold annotations are first consulted AFTER producing the prediction.
                    let gold = turn
                        .frames
                        .iter()
                        .find(|f| f.service == s.service_name)
                        .and_then(|f| f.state.as_ref())
                        .ok_or("missing gold frame")?;
                    intents += usize::from(gold.active_intent == state.active_intent);
                    let correct = s.slots.iter().all(|slot| {
                        match (
                            gold.slot_values.get(&slot.name),
                            state.slot_values.get(&slot.name).and_then(|v| v.first()),
                        ) {
                            (None, None) => true,
                            (Some(values), Some(v)) => {
                                if slot.is_categorical {
                                    values.first().is_some_and(|x| x.eq_ignore_ascii_case(v))
                                } else {
                                    values.contains(v)
                                }
                            }
                            _ => false,
                        }
                    });
                    exact += usize::from(correct);
                    predictions.push(Prediction {
                        dialogue_id: dialogue.dialogue_id.clone(),
                        turn_index: index,
                        service: s.service_name.clone(),
                        state,
                    });
                }
            }
            write(&output, &predictions)?;
            println!(
                "{}",
                serde_json::json!({"dialogues":count,"frames":predictions.len(),"strict_joint_correct":exact,"intent_correct":intents,"carry":!no_carry})
            );
        }
        Command::Predict {
            input,
            schema: path,
            model,
        } => {
            let s = schema(&path)?;
            let model: DialogueModel = read(&model)?;
            model.validate(&s)?;
            let turns: Vec<InputTurn> = read(&input)?;
            let mut session = SessionState::default();
            for turn in turns {
                model.advance(&mut session, &turn, &s, true);
                if matches!(turn.speaker, Speaker::User) {
                    println!("{}", serde_json::to_string(&predicted(&session))?);
                }
            }
        }
    }
    Ok(())
}
