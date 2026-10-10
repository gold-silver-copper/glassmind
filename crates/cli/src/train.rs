//! Training glassmind's small models from what it already gets right.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use glassmind_answer::{ConversationInput, Outcome, Response};
use glassmind_english::{Shape, Token};
use glassmind_models::{EditModel, IntentModel};
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

/// Every turn answered right (other than coding fixes, which have no
/// single right wording): what was typed, and what the grammar says for the
/// reading chosen. What each small model learns from.
fn answered_right(
    inputs: &Path,
    outcomes: &Path,
    labels: &Path,
) -> Result<Vec<(String, String)>, String> {
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
    Ok(outcomes
        .iter()
        .filter_map(|o| {
            let canonical = o.canonical.as_ref()?;
            let exp = expected.get(o.turn_id.as_str())?;
            let text = texts.get(o.turn_id.as_str())?;
            (!matches!(exp, Response::Repaired { .. }) && &&o.response == exp)
                .then(|| ((*text).to_owned(), canonical.clone()))
        })
        .collect())
}

fn english() -> glassmind_english::English<glassmind_world_rust::Rust> {
    glassmind_english::English::new(
        glassmind_english_rust::rules(),
        &glassmind_rules::Switches::all_on(),
    )
}

/// Learns the normalizer's edit model from every turn answered right:
/// what was typed, against what the grammar says for the same request.
pub fn normalizer(
    inputs: &Path,
    outcomes: &Path,
    labels: &Path,
    out: &Path,
) -> Result<String, String> {
    let english = english();
    let words =
        |text: &str| -> Vec<String> { english.tokens_of(text).iter().map(model_word).collect() };
    let pairs: Vec<(Vec<String>, Vec<String>)> = answered_right(inputs, outcomes, labels)?
        .iter()
        .map(|(text, canonical)| (words(text), words(canonical.trim_end_matches(['?', '.']))))
        .collect();
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

/// Learns the request-category tagger from every turn answered right: the
/// words typed, against the canonical request with its names as slots.
/// Twice: the second time, mistyped words one slip from a word the first
/// model knows are read as that word, as they are when it is used.
pub fn intents(
    inputs: &Path,
    outcomes: &Path,
    labels: &Path,
    out: &Path,
) -> Result<String, String> {
    let english = english();
    let right = answered_right(inputs, outcomes, labels)?;
    let learn = |m: &IntentModel| -> Vec<(BTreeSet<String>, String)> {
        right
            .iter()
            .filter_map(|(text, canonical)| english.intent_example(text, canonical, m))
            .collect()
    };
    let first = IntentModel::learn(&learn(&IntentModel::default()));
    let examples = learn(&first);
    // Held out: every fifth example, judged by a model learned from the rest.
    let (learning, held) = glassmind_models::report::hold_out(&examples, 5);
    let trial = IntentModel::learn(&learning);
    let answers: Vec<(f64, bool)> = held
        .iter()
        .filter_map(|(words, t)| {
            let best = trial
                .rank(words, glassmind_models::intents::slots_of(t))
                .into_iter()
                .next()?;
            Some((best.1, best.0 == t))
        })
        .collect();
    let calibration = glassmind_models::Calibration::of(&answers);
    let mut model = IntentModel::learn(&examples);
    // The thresholds stay those of the model being replaced, which were
    // calibrated on the dev set; a new model is answered by nothing until
    // it is calibrated.
    let before = std::fs::read_to_string(out).unwrap_or_default();
    let old = IntentModel::load(&before);
    (model.sure, model.offer) = if before.is_empty() {
        (1.01, 0.5)
    } else {
        (old.sure, old.offer)
    };
    let notes: String = before
        .lines()
        .filter(|l| l.starts_with("# calibrated"))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(
        out,
        format!("{notes}{}{}", calibration.table("# "), model.dump()),
    )
    .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    Ok(format!(
        "learned from {} examples of {} turns answered right; {} templates, {} words and pairs\n{}",
        examples.len(),
        right.len(),
        model.templates.len(),
        model.seen.len(),
        calibration.table(""),
    ))
}
