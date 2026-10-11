use glassmind_dialogue::{
    adapter::Schema,
    multi::{MultiModel, MultiSession},
    tracker::{InputTurn, Speaker},
};
use serde_json::json;
use std::{fs::File, io::{BufReader, BufWriter}, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [model_path, schema_path, input_path, output_path] = args.as_slice() else {
        return Err("expected MODEL SCHEMA INPUT OUTPUT".into());
    };
    let model: MultiModel = serde_json::from_reader(BufReader::new(File::open(model_path)?))?;
    let schemas: Vec<Schema> = serde_json::from_reader(BufReader::new(File::open(schema_path)?))?;
    model.validate(&schemas)?;
    let conversations: Vec<Vec<InputTurn>> = serde_json::from_reader(BufReader::new(File::open(input_path)?))?;
    let offers = model.offers.as_ref().ok_or("missing approved offers model")?;
    let mut rows = Vec::new();
    for (conversation_index, turns) in conversations.iter().enumerate() {
        let mut session = MultiSession::default();
        let mut sources: Vec<(usize, &str)> = Vec::new();
        for (turn_index, turn) in turns.iter().enumerate() {
            model.advance(&mut session, turn, &schemas);
            if matches!(turn.speaker, Speaker::System) {
                sources.push((turn_index, &turn.utterance));
                continue;
            }
            for schema in &schemas {
                let service = &schema.service_name;
                if !session.active_services.contains(service) { continue; }
                for slot in &schema.slots {
                    if slot.is_categorical || session.states.get(service).is_some_and(|s| s.slots.contains_key(&slot.name)) {
                        continue;
                    }
                    let proposals: Vec<_> = sources.iter().rev().take(32)
                        .filter_map(|&(index, text)| offers.propose(service, &slot.name, (index, text), &turn.utterance, &schemas))
                        .collect();
                    if !proposals.is_empty() {
                        rows.push(json!({"conversation_index": conversation_index, "turn_index": turn_index,
                            "service": service, "slot": slot.name, "proposals": proposals}));
                    }
                }
            }
        }
    }
    serde_json::to_writer(BufWriter::new(File::create(Path::new(output_path))?), &rows)?;
    Ok(())
}
