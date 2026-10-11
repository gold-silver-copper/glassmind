use crate::adapter::{Dialogue, Schema};
use glassmind_everyday::{Example, IntentModel, TrainOptions};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The runtime boundary deliberately cannot represent oracle frames or actions.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputTurn {
    pub speaker: Speaker,
    pub utterance: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Speaker {
    User,
    System,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SlotValue {
    Categorical(String),
    Text(String),
    DontCare,
}

impl SlotValue {
    pub fn text(&self) -> &str {
        match self {
            Self::Categorical(v) | Self::Text(v) => v,
            Self::DontCare => "dontcare",
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SessionState {
    pub active_intent: Option<String>,
    pub slots: BTreeMap<String, SlotValue>,
    previous_system: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DialogueModel {
    pub version: u32,
    pub service: String,
    pub intent: IntentModel,
    pub categorical: BTreeMap<String, IntentModel>,
    pub spans: IntentModel,
    pub training_dialogues: usize,
    pub training_frames: usize,
}

#[derive(Debug)]
struct Token {
    text: String,
    start: usize,
    end: usize,
}

fn tokens(text: &str) -> Vec<Token> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut start = None;
    for (i, c) in chars
        .iter()
        .copied()
        .chain(std::iter::once(' '))
        .enumerate()
    {
        if c.is_alphanumeric() || c == '\'' || c == '-' {
            start.get_or_insert(i);
        } else if let Some(begin) = start.take() {
            out.push(Token {
                text: chars.get(begin..i).unwrap_or_default().iter().collect(),
                start: begin,
                end: i,
            });
        }
    }
    out
}

fn context(system: &str, user: &str, filled: bool) -> String {
    let system = tokens(system)
        .iter()
        .map(|t| format!("sys{}", t.text))
        .collect::<Vec<_>>()
        .join(" ");
    format!("{user} {system} filled{filled}")
}

fn token_context(words: &[Token], i: usize, system: &str) -> String {
    let mut features = Vec::new();
    for offset in -2isize..=2 {
        if let Some(j) = i.checked_add_signed(offset)
            && let Some(t) = words.get(j)
        {
            features.push(format!("p{}x{}", offset + 2, t.text));
            if offset == 0 {
                features.push(format!(
                    "capital{}",
                    t.text.chars().next().is_some_and(char::is_uppercase)
                ));
                features.push(format!(
                    "numeric{}",
                    t.text.chars().all(|c| c.is_ascii_digit())
                ));
            }
        }
    }
    features.push(format!("first{} last{}", i == 0, i + 1 == words.len()));
    for t in tokens(system) {
        features.push(format!("sys{}", t.text));
    }
    features.join(" ")
}

impl DialogueModel {
    pub fn train(dialogues: &[Dialogue], schema: &Schema, epochs: usize) -> Result<Self, String> {
        let mut intent_rows = Vec::new();
        let mut categorical = BTreeMap::<String, Vec<Example>>::new();
        let mut span_rows = Vec::new();
        let input_slots: BTreeSet<_> = schema
            .intents
            .iter()
            .flat_map(|i| i.required_slots.iter().chain(i.optional_slots.keys()))
            .collect();
        for s in &schema.slots {
            if s.is_categorical && input_slots.contains(&s.name) {
                categorical.insert(s.name.clone(), Vec::new());
            }
        }
        for dialogue in dialogues {
            let mut system = String::new();
            let mut previous = BTreeMap::<String, Vec<String>>::new();
            let mut previous_intent = "NONE".to_owned();
            for turn in &dialogue.turns {
                if turn.speaker == "SYSTEM" {
                    system.clone_from(&turn.utterance);
                    continue;
                }
                let Some(frame) = turn
                    .frames
                    .iter()
                    .find(|f| f.service == schema.service_name)
                else {
                    continue;
                };
                let Some(gold) = &frame.state else { continue };
                intent_rows.push(Example {
                    text: format!(
                        "{} priorintent{previous_intent}",
                        context(&system, &turn.utterance, false)
                    ),
                    label: if gold.active_intent == previous_intent {
                        "KEEP".into()
                    } else {
                        format!("SET:{}", gold.active_intent)
                    },
                });
                previous_intent.clone_from(&gold.active_intent);
                for (slot, rows) in &mut categorical {
                    let old = previous.get(slot).and_then(|v| v.first());
                    let new = gold.slot_values.get(slot).and_then(|v| v.first());
                    let label = if new == old {
                        "KEEP".to_owned()
                    } else {
                        new.map_or_else(|| "DELETE".to_owned(), |v| format!("SET:{v}"))
                    };
                    rows.push(Example {
                        text: context(&system, &turn.utterance, old.is_some()),
                        label,
                    });
                }
                let words = tokens(&turn.utterance);
                for (i, word) in words.iter().enumerate() {
                    let span = frame.slots.iter().find(|s| {
                        input_slots.contains(&s.slot)
                            && word.start >= s.start
                            && word.end <= s.exclusive_end
                    });
                    let label = span.map_or_else(
                        || "O".to_owned(),
                        |s| {
                            format!(
                                "{}:{}",
                                if word.start == s.start { "B" } else { "I" },
                                s.slot
                            )
                        },
                    );
                    span_rows.push(Example {
                        text: token_context(&words, i, &system),
                        label,
                    });
                }
                previous.clone_from(&gold.slot_values);
            }
        }
        let options = TrainOptions {
            epochs,
            char_weight: 0.15,
            min_df: 2,
        };
        let intent = IntentModel::train(&intent_rows, options.clone())?;
        let mut slot_models = BTreeMap::new();
        for (name, rows) in categorical {
            eprintln!("training categorical {name}: {} rows", rows.len());
            slot_models.insert(name, IntentModel::train(&rows, options.clone())?);
        }
        eprintln!("training span proposals: {} tokens", span_rows.len());
        let spans = IntentModel::train(&span_rows, options)?;
        Ok(Self {
            version: 2,
            service: schema.service_name.clone(),
            intent,
            categorical: slot_models,
            spans,
            training_dialogues: dialogues.len(),
            training_frames: intent_rows.len(),
        })
    }

    pub fn validate(&self, schema: &Schema) -> Result<(), String> {
        if self.version != 2 || self.service != schema.service_name {
            return Err("model/schema mismatch".into());
        }
        self.intent.validate()?;
        self.spans.validate()?;
        for model in self.categorical.values() {
            model.validate()?;
        }
        Ok(())
    }

    /// Only the current utterance, schema, and this session's predicted state enter here.
    pub fn advance(
        &self,
        state: &mut SessionState,
        turn: &InputTurn,
        schema: &Schema,
        carry: bool,
    ) {
        if matches!(turn.speaker, Speaker::System) {
            state.previous_system.clone_from(&turn.utterance);
            return;
        }
        if !carry {
            state.slots.clear();
            state.active_intent = None;
        }
        let proposed = self
            .intent
            .predict(&format!(
                "{} priorintent{}",
                context(&state.previous_system, &turn.utterance, false),
                state.active_intent.as_deref().unwrap_or("NONE")
            ))
            .label;
        if let Some(intent) = proposed.strip_prefix("SET:") {
            if intent == "NONE" {
                state.active_intent = None;
            } else if schema.intents.iter().any(|i| i.name == intent) {
                state.active_intent = Some(intent.into());
            }
        }
        for (slot, model) in &self.categorical {
            let prediction = model.predict(&context(
                &state.previous_system,
                &turn.utterance,
                state.slots.contains_key(slot),
            ));
            if prediction.label == "DELETE" {
                state.slots.remove(slot);
            }
            if let Some(value) = prediction.label.strip_prefix("SET:") {
                if value == "dontcare" {
                    state.slots.insert(slot.clone(), SlotValue::DontCare);
                } else if schema.slots.iter().any(|s| {
                    s.name == *slot
                        && s.is_categorical
                        && s.possible_values.iter().any(|v| v == value)
                }) {
                    state
                        .slots
                        .insert(slot.clone(), SlotValue::Categorical(value.into()));
                }
            }
        }
        let words = tokens(&turn.utterance);
        let chars: Vec<char> = turn.utterance.chars().collect();
        let mut spans: Vec<(String, usize, usize)> = Vec::new();
        for (i, word) in words.iter().enumerate() {
            let p = self
                .spans
                .predict(&token_context(&words, i, &state.previous_system));
            if let Some((tag, slot)) = p.label.split_once(':') {
                if !schema
                    .slots
                    .iter()
                    .any(|s| s.name == slot && !s.is_categorical)
                {
                    continue;
                }
                if tag == "I"
                    && let Some((last_slot, _, end)) = spans.last_mut()
                    && last_slot == slot
                {
                    *end = word.end;
                    continue;
                }
                spans.push((slot.to_owned(), word.start, word.end));
            }
        }
        // Longest proposed span per slot avoids replacing an address by a stray token.
        spans.sort_by_key(|(_, start, end)| end - start);
        for (slot, start, end) in spans {
            if let Some(value) = chars.get(start..end) {
                state
                    .slots
                    .insert(slot, SlotValue::Text(value.iter().collect()));
            }
        }
    }
}
