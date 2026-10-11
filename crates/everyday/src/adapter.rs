//! JSONL benchmark adapter. An OOS refusal is right, an in-scope refusal is
//! separate, and an accepted wrong label (including accepted OOS) is wrong.
use crate::model::Prediction;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{self, BufRead};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Example {
    pub text: String,
    pub label: String,
}

pub fn read_examples(reader: impl BufRead) -> io::Result<Vec<Example>> {
    reader
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let row: Example = serde_json::from_str(&line?).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("row {}: {e}", i + 1))
            })?;
            if row.text.trim().is_empty() || row.label.trim().is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("empty text or label at row {}", i + 1),
                ));
            }
            Ok(row)
        })
        .collect()
}

/// Stable stratified sample: the first n original-order rows of every label.
/// This includes OOS and never changes with a model or its predictions.
pub fn fixed_sample(rows: &[Example], per_label: usize) -> Vec<Example> {
    let mut counts = BTreeMap::<&str, usize>::new();
    rows.iter()
        .filter(|row| {
            let count = counts.entry(&row.label).or_default();
            *count += 1;
            *count <= per_label
        })
        .cloned()
        .collect()
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Totals {
    pub total: usize,
    pub right: usize,
    pub refusal: usize,
    pub wrong: usize,
    pub in_scope: usize,
    pub in_scope_right: usize,
    pub oos: usize,
    pub oos_right_refusal: usize,
    pub raw_top1_in_scope_right: usize,
}

impl Totals {
    pub fn observe(&mut self, gold: &str, prediction: &Prediction) {
        self.total += 1;
        if gold == "oos" {
            self.oos += 1;
            if prediction.accepted {
                self.wrong += 1;
            } else {
                self.right += 1;
                self.oos_right_refusal += 1;
            }
        } else {
            self.in_scope += 1;
            self.raw_top1_in_scope_right += usize::from(prediction.label == gold);
            if !prediction.accepted {
                self.refusal += 1;
            } else if prediction.label == gold {
                self.right += 1;
                self.in_scope_right += 1;
            } else {
                self.wrong += 1;
            }
        }
    }
}
