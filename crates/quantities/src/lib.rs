//! Deterministic, bounded arithmetic with explicit refusals and typed execution.
pub mod calendar;
pub mod inventory;
pub mod number_words;
mod parser;
mod percent;
pub mod rational;
pub mod units;

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Answer {
    Answer {
        value: String,
        unit: Option<String>,
        method: String,
    },
    Refusal {
        reason: String,
    },
}

/// Parse a fully consumed supported question, then execute checked operations.
pub fn solve(question: &str) -> Answer {
    match parser::solve(question) {
        Ok((value, unit, method)) => Answer::Answer {
            value,
            unit,
            method: method.into(),
        },
        Err(reason) => Answer::Refusal { reason },
    }
}
