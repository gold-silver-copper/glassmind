//! Official SGD train/dev adapter. No system action or state annotations are used.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Schema {
    pub service_name: String,
    pub slots: Vec<SlotSchema>,
    pub intents: Vec<IntentSchema>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlotSchema {
    pub name: String,
    pub is_categorical: bool,
    pub possible_values: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntentSchema {
    pub name: String,
    pub required_slots: Vec<String>,
    pub optional_slots: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Dialogue {
    pub dialogue_id: String,
    pub services: Vec<String>,
    pub turns: Vec<AnnotatedTurn>,
}

#[derive(Debug, Deserialize)]
pub struct AnnotatedTurn {
    pub speaker: String,
    pub utterance: String,
    #[serde(default)]
    pub frames: Vec<Frame>,
}

#[derive(Debug, Deserialize)]
pub struct Frame {
    pub service: String,
    #[serde(default)]
    pub slots: Vec<Span>,
    pub state: Option<GoldState>,
}

#[derive(Debug, Deserialize)]
pub struct Span {
    pub slot: String,
    pub start: usize,
    pub exclusive_end: usize,
}

#[derive(Debug, Deserialize)]
pub struct GoldState {
    pub active_intent: String,
    pub slot_values: BTreeMap<String, Vec<String>>,
}
