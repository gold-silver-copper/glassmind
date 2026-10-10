//! Training glassmind's small models from what it already gets right.

use std::collections::BTreeMap;
use std::path::Path;

use glassmind_answer::{ConversationInput, Outcome, Response};
use glassmind_english::{Shape, Token};
use glassmind_models::EditModel;
use serde::Deserialize;

/// The parts of a label the trainer needs.
#[derive(Deserialize)]
struct Label {
    turn_id: String,
    expected: Response,
}

fn read_lines<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).map_err(|e| format!("{}: {e}", path.display())))
        .collect()
}

/// A token as the normalizer sees it: code names and numbers stand for
/// their kind, words for their lowercase form.
pub fn model_word(t: &Token) -> String {
    match t.shape {
        Shape::Code => "<name>".to_owned(),
        Shape::Number => "<number>".to_owned(),
        _ => t.norm.clone(),
    }
}

/// Learns the normalizer's edit model from every turn answered right:
/// what was typed, against what the grammar says for the same request.
pub fn normalizer(
    inputs: &Path,
    outcomes: &Path,
    labels: &Path,
    out: &Path,
) -> Result<String, String> {
    let inputs: Vec<ConversationInput> = read_lines(inputs)?;
    let outcomes: Vec<Outcome> = read_lines(outcomes)?;
    let labels: Vec<Label> = read_lines(labels)?;
    let expected: BTreeMap<&str, &Response> = labels
        .iter()
        .map(|l| (l.turn_id.as_str(), &l.expected))
        .collect();
    let texts: BTreeMap<&str, &str> = inputs
        .iter()
        .flat_map(|c| c.turns.iter().map(|t| (t.id.as_str(), t.text.as_str())))
        .collect();
    let english = glassmind_english::English::<glassmind_world_rust::Rust>::new(
        glassmind_english_rust::rules(),
        &glassmind_rules::Switches::all_on(),
    );
    let words =
        |text: &str| -> Vec<String> { english.tokens_of(text).iter().map(model_word).collect() };
    let mut pairs = Vec::new();
    for o in &outcomes {
        let (Some(canonical), Some(exp), Some(text)) = (
            &o.canonical,
            expected.get(o.turn_id.as_str()),
            texts.get(o.turn_id.as_str()),
        ) else {
            continue;
        };
        if matches!(exp, Response::Repaired { .. }) || &&o.response != exp {
            continue;
        }
        let canonical = canonical.trim_end_matches(['?', '.']).to_owned();
        pairs.push((words(text), words(&canonical)));
    }
    let mut model = EditModel::learn(&pairs);
    // Trust readings that cost no more than 95 in 100 readings known to be right.
    let mut costs: Vec<f64> = pairs
        .iter()
        .map(|(t, c)| model.channel_cost(t, c))
        .collect();
    costs.sort_by(f64::total_cmp);
    model.trusted_bits = costs.get(costs.len() * 95 / 100).copied().unwrap_or(0.0);
    std::fs::write(out, model.dump())
        .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    let top = |m: &BTreeMap<String, u32>| {
        let mut v: Vec<(&String, &u32)> = m.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1));
        v.iter()
            .take(12)
            .map(|(w, n)| format!("{w}:{n}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut dropped: Vec<(String, u32)> = model
        .dropped
        .iter()
        .map(|((a, w), n)| (format!("{a}+{w}"), *n))
        .collect();
    dropped.sort_by_key(|d| std::cmp::Reverse(d.1));
    let mut swapped: Vec<(String, u32)> = model
        .swapped
        .iter()
        .map(|((t, c), n)| (format!("{t}->{c}"), *n))
        .collect();
    swapped.sort_by_key(|d| std::cmp::Reverse(d.1));
    let list = |v: &[(String, u32)]| {
        v.iter()
            .take(12)
            .map(|(w, n)| format!("{w}:{n}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    Ok(format!(
        "learned from {} pairs; trusted up to {:.1} bits\nadded:   {}\ndropped: {}\nswapped: {}",
        pairs.len(),
        model.trusted_bits,
        top(&model.added),
        list(&dropped),
        list(&swapped)
    ))
}
