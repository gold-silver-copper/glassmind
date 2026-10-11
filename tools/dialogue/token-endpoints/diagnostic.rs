//! Offline frozen-reference diagnostic. Input is exclusively speaker/utterance.
use glassmind_dialogue::{
    adapter::Schema,
    multi::{MultiModel, MultiSession},
    tracker::{InputTurn, Speaker},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{fs::File, io::{BufReader, BufWriter, Write}, path::Path};

fn read<T: DeserializeOwned>(path: &str) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
}
#[derive(Serialize)]
struct Proposal {
    conversation: usize,
    turn: usize,
    service: String,
    slot: String,
    value: String,
    sources: Vec<(String, String)>,
    punctuation_value: Option<String>,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [model_path, schema_path, input_path, output_path] = args.as_slice() else {
        return Err("usage: dialogue-token-endpoint-trace MODEL SCHEMA INPUT OUTPUT".into());
    };
    let model: MultiModel = read(model_path)?;
    let schemas: Vec<Schema> = read(schema_path)?;
    model.validate(&schemas)?;
    let conversations: Vec<Vec<InputTurn>> = read(input_path)?;
    let mut out = BufWriter::new(File::create(Path::new(output_path))?);
    let mut total = 0usize;
    for (conversation, turns) in conversations.iter().enumerate() {
        let mut session = MultiSession::default();
        let mut history = Vec::new();
        for (turn, input) in turns.iter().enumerate() {
            let prior = session.states.clone();
            model.advance(&mut session, input, &schemas);
            if matches!(input.speaker, Speaker::User) {
                for schema in &schemas {
                    if !session.active_services.contains(&schema.service_name) {
                        continue;
                    }
                    let mut state = prior.get(&schema.service_name).cloned().unwrap_or_default();
                    let service = model.services.get(&schema.service_name).ok_or("missing service")?;
                    service.advance(&mut state, input, schema, true);
                    for slot in &schema.slots {
                        if state.slots.contains_key(&slot.name) {
                            continue;
                        }
                        let Some(p) = model.references.as_ref().and_then(|r| r.propose(
                            &schema.service_name, &slot.name, &prior, &history, &schemas,
                        )) else { continue };
                        let correction = model.boundaries.as_ref().and_then(|b| b.propose(&p.value, &history));
                        let expected = correction.as_ref().map_or(p.value.as_str(), |c| c.value.as_str());
                        let actual = session.states.get(&schema.service_name)
                            .and_then(|s| s.slots.get(&slot.name)).ok_or("missing replayed reference")?;
                        if actual.text() != expected {
                            return Err("diagnostic reference disagrees with actual frozen inference".into());
                        }
                        serde_json::to_writer(&mut out, &Proposal {
                            conversation, turn, service: schema.service_name.clone(), slot: slot.name.clone(),
                            value: p.value, sources: p.sources,
                            punctuation_value: correction.map(|c| c.value),
                        })?;
                        writeln!(out)?;
                        total += 1;
                    }
                }
            }
            history.push(input.utterance.clone());
        }
    }
    out.flush()?;
    eprintln!("{} conversations, {total} verified actual reference proposals", conversations.len());
    Ok(())
}
