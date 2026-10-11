// SPDX-License-Identifier: MIT
//! Inspectable sparse linear span ranking with exact UTF-8 byte provenance.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const DIM: usize = 65536;
const MAX_TOKENS: usize = 2048;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub context: String,
    pub question: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl Span {
    pub fn text<'a>(&self, context: &'a str) -> Option<&'a str> {
        context.get(self.start..self.end)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Predicted,
    Refused,
}

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub status: Status,
    pub answer: String,
    /// Raw proposal retained for evaluation even when the policy refuses it.
    pub proposal: Option<Span>,
    pub evidence: Option<Span>,
    /// Rank score and margin are NOT probabilities or calibrated confidence.
    pub score: f32,
    pub margin: f32,
    pub reason: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Model {
    pub version: u32,
    weights: Vec<f32>,
    pub threshold: f32,
    pub fit_rows: usize,
    pub representable_rows: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainingRow {
    pub context: String,
    pub question: String,
    pub answer: Span,
}

#[derive(Clone)]
struct Token {
    span: Span,
    word: String,
    cap: bool,
    digit: bool,
}
fn tokens(text: &str) -> Vec<Token> {
    let mut result = Vec::new();
    let mut start = None;
    for (i, c) in text.char_indices().chain([(text.len(), ' ')]) {
        if c.is_alphanumeric() || ((c == '\'' || c == '’' || c == '-') && start.is_some()) {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start.take()
            && let Some(raw) = text.get(s..i)
        {
            result.push(Token {
                span: Span { start: s, end: i },
                word: raw.to_lowercase(),
                cap: raw.chars().next().is_some_and(char::is_uppercase),
                digit: raw.chars().any(char::is_numeric),
            });
        }
    }
    result
}
fn stop(w: &str) -> bool {
    matches!(
        w,
        "the"
            | "a"
            | "an"
            | "of"
            | "in"
            | "on"
            | "at"
            | "to"
            | "for"
            | "from"
            | "by"
            | "with"
            | "and"
            | "or"
            | "is"
            | "was"
            | "were"
            | "are"
            | "be"
            | "been"
            | "did"
            | "does"
            | "do"
            | "has"
            | "have"
            | "had"
            | "what"
            | "which"
            | "who"
            | "when"
            | "where"
            | "how"
            | "why"
            | "that"
            | "this"
            | "it"
            | "its"
            | "as"
            | "s"
    )
}
fn hash(parts: &[&str]) -> usize {
    let mut h = 14695981039346656037u64;
    for p in parts {
        for b in p.bytes().chain([255]) {
            h = (h ^ u64::from(b)).wrapping_mul(1099511628211);
        }
    }
    (h as usize) % DIM
}
type Features = Vec<(usize, f32)>;
fn add(f: &mut Features, parts: &[&str], value: f32) {
    if value != 0.0 {
        f.push((hash(parts), value));
    }
}
fn dot(w: &[f32], f: &Features) -> f32 {
    f.iter()
        .map(|(i, x)| w.get(*i).copied().unwrap_or(0.0) * x)
        .sum()
}

struct Candidate {
    span: Span,
    evidence: Span,
    features: Features,
    baseline: f32,
}
struct Prepared {
    candidates: Vec<Candidate>,
    supported: bool,
}
fn prepare(context: &str, question: &str) -> Prepared {
    let mut output = Prepared {
        candidates: Vec::new(),
        supported: false,
    };
    if context.len() > 65536 || question.len() > 2048 {
        return output;
    }
    let ts = tokens(context);
    let qs = tokens(question);
    if ts.len() > MAX_TOKENS || qs.is_empty() {
        return output;
    }
    let qwords: BTreeSet<&str> = qs
        .iter()
        .map(|t| t.word.as_str())
        .filter(|w| !stop(w))
        .collect();
    if qwords.is_empty() {
        return output;
    }
    let wh = qs.first().map_or("", |t| t.word.as_str());
    let qhead = qs
        .iter()
        .find(|t| !stop(&t.word))
        .map_or("", |t| t.word.as_str());
    let mut sentences = Vec::new();
    let mut s = 0;
    for (i, t) in ts.iter().enumerate() {
        let boundary = ts.get(i + 1).is_none_or(|next| {
            context
                .get(t.span.end..next.span.start)
                .unwrap_or("")
                .contains(['.', '?', '!', ';', '\n'])
        });
        if boundary {
            let slice = ts.get(s..=i).unwrap_or(&[]);
            let overlap = slice
                .iter()
                .filter(|t| qwords.contains(t.word.as_str()))
                .map(|t| &t.word)
                .collect::<BTreeSet<_>>()
                .len();
            sentences.push((s, i + 1, overlap as f32 / qwords.len() as f32));
            s = i + 1;
        }
    }
    sentences.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
    output.supported = sentences.first().is_some_and(|x| x.2 > 0.0);
    for (s, e, overlap) in sentences.into_iter().take(3) {
        for i in s..e {
            for j in i + 1..=(i + 10).min(e) {
                let span_tokens = ts.get(i..j).unwrap_or(&[]);
                let (Some(first), Some(last), Some(sent_first), Some(sent_last)) = (
                    span_tokens.first(),
                    span_tokens.last(),
                    ts.get(s),
                    ts.get(e.saturating_sub(1)),
                ) else {
                    continue;
                };
                let span = Span {
                    start: first.span.start,
                    end: last.span.end,
                };
                let evidence = Span {
                    start: sent_first.span.start,
                    end: sent_last.span.end,
                };
                let n = (j - i) as f32;
                let inside = span_tokens
                    .iter()
                    .filter(|t| qwords.contains(t.word.as_str()))
                    .count() as f32
                    / n;
                let caps = span_tokens.iter().filter(|t| t.cap).count() as f32 / n;
                let digit = span_tokens.iter().any(|t| t.digit) as u8 as f32;
                let left = i.checked_sub(1).and_then(|k| ts.get(k)).filter(|_| i > s);
                let right = ts.get(j).filter(|_| j < e);
                let leftword = left.map_or("<s>", |t| t.word.as_str());
                let rightword = right.map_or("</s>", |t| t.word.as_str());
                let near = ts
                    .get(s..e)
                    .unwrap_or(&[])
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| qwords.contains(t.word.as_str()))
                    .map(|(k, _)| {
                        let k = k + s;
                        if k < i {
                            1.0 / ((i - k) as f32)
                        } else if k >= j {
                            1.0 / ((k - j + 1) as f32)
                        } else {
                            0.0
                        }
                    })
                    .sum::<f32>();
                let fullcap =
                    caps == 1.0 && !left.is_some_and(|t| t.cap) && !right.is_some_and(|t| t.cap);
                let numeric_question = wh == "when"
                    || (wh == "how"
                        && qs.get(1).is_some_and(|t| {
                            matches!(t.word.as_str(), "many" | "much" | "long" | "old")
                        }));
                let baseline = 4.0 * overlap + near * 0.35 - 2.0 * inside - 0.12 * n
                    + 0.7 * (fullcap as u8 as f32)
                    + if numeric_question { 1.5 * digit } else { 0.0 };
                let mut f = Vec::with_capacity(70);
                for (name, value) in [
                    ("overlap", overlap),
                    ("near", near),
                    ("inside", inside),
                    ("length", n / 10.0),
                    ("caps", caps),
                    ("digit", digit),
                    ("fullcap", fullcap as u8 as f32),
                    ("firstcap", first.cap as u8 as f32),
                    ("lastcap", last.cap as u8 as f32),
                    ("startsent", (i == s) as u8 as f32),
                    ("endsent", (j == e) as u8 as f32),
                    ("leftcap", left.is_some_and(|t| t.cap) as u8 as f32),
                    ("rightcap", right.is_some_and(|t| t.cap) as u8 as f32),
                    ("firststop", stop(&first.word) as u8 as f32),
                    ("laststop", stop(&last.word) as u8 as f32),
                ] {
                    add(&mut f, &[name], value);
                    add(&mut f, &["wh", wh, name], value);
                    add(&mut f, &["head", qhead, name], value);
                }
                let length = (j - i).to_string();
                add(&mut f, &["n", &length], 1.0);
                add(&mut f, &["whn", wh, &length], 1.0);
                for (role, word) in [
                    ("left", leftword),
                    ("right", rightword),
                    ("first", first.word.as_str()),
                    ("last", last.word.as_str()),
                ] {
                    add(&mut f, &[role, word], 1.0);
                    add(&mut f, &["wh", wh, role, word], 1.0);
                }
                for (role, gap) in [
                    (
                        "before",
                        left.and_then(|t| context.get(t.span.end..span.start))
                            .unwrap_or(""),
                    ),
                    (
                        "after",
                        right
                            .and_then(|t| context.get(span.end..t.span.start))
                            .unwrap_or(""),
                    ),
                ] {
                    for c in gap.chars().filter(|c| !c.is_whitespace()).take(2) {
                        add(&mut f, &[role, &c.to_string()], 1.0);
                    }
                }
                output.candidates.push(Candidate {
                    span,
                    evidence,
                    features: f,
                    baseline,
                });
            }
        }
    }
    output
}
impl Model {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.weights.len() != DIM
            || self.weights.iter().any(|x| !x.is_finite())
            || !self.threshold.is_finite()
        {
            return Err("invalid QA model".into());
        }
        Ok(())
    }
    pub fn train(rows: &[TrainingRow]) -> Self {
        let mut model = Self {
            version: 1,
            weights: vec![0.0; DIM],
            threshold: f32::MAX,
            fit_rows: rows.len(),
            representable_rows: 0,
        };
        let mut sums = vec![0.0f64; DIM];
        let mut touched = vec![0usize; DIM];
        let mut step = 0usize;
        for epoch in 0..5 {
            let mut usable = 0;
            for row in rows {
                step += 1;
                let p = prepare(&row.context, &row.question);
                let Some(gold) = p.candidates.iter().find(|c| c.span == row.answer) else {
                    continue;
                };
                usable += 1;
                let Some(best) = p.candidates.iter().max_by(|a, b| {
                    dot(&model.weights, &a.features).total_cmp(&dot(&model.weights, &b.features))
                }) else {
                    continue;
                };
                let loss =
                    1.0 + dot(&model.weights, &best.features) - dot(&model.weights, &gold.features);
                if loss <= 0.0 || best.span == gold.span {
                    continue;
                }
                let mut delta = std::collections::BTreeMap::<usize, f32>::new();
                for (i, x) in &gold.features {
                    *delta.entry(*i).or_default() += x;
                }
                for (i, x) in &best.features {
                    *delta.entry(*i).or_default() -= x;
                }
                let norm = delta.values().map(|x| x * x).sum::<f32>().max(0.001);
                let rate = (loss / norm).min(0.1);
                for (i, x) in delta {
                    if let (Some(w), Some(sum), Some(last)) = (
                        model.weights.get_mut(i),
                        sums.get_mut(i),
                        touched.get_mut(i),
                    ) {
                        *sum += f64::from(*w) * (step - *last) as f64;
                        *last = step;
                        *w += rate * x;
                    }
                }
            }
            model.representable_rows = usable;
            eprintln!("epoch {}: {usable}/{} representable", epoch + 1, rows.len());
        }
        for ((w, sum), last) in model.weights.iter_mut().zip(sums).zip(touched) {
            *w = ((sum + f64::from(*w) * (step - last) as f64) / (step.max(1) as f64)) as f32;
        }
        model
    }
    pub fn predict(&self, context: &str, question: &str) -> Prediction {
        if self.validate().is_err() {
            return refusal("invalid_model");
        }
        predict(context, question, Some(self))
    }
}
pub fn baseline(context: &str, question: &str) -> Prediction {
    predict(context, question, None)
}
fn refusal(reason: &str) -> Prediction {
    Prediction {
        status: Status::Refused,
        answer: String::new(),
        proposal: None,
        evidence: None,
        score: 0.0,
        margin: 0.0,
        reason: reason.into(),
    }
}
fn predict(context: &str, question: &str, model: Option<&Model>) -> Prediction {
    let p = prepare(context, question);
    let mut ranked: Vec<(&Candidate, f32)> = p
        .candidates
        .iter()
        .map(|c| {
            (
                c,
                model.map_or(c.baseline, |m| dot(&m.weights, &c.features)),
            )
        })
        .collect();
    if ranked.iter().any(|(_, score)| !score.is_finite()) {
        return refusal("non_finite_model_score");
    }
    ranked.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then(a.0.span.start.cmp(&b.0.span.start))
            .then(a.0.span.end.cmp(&b.0.span.end))
    });
    let Some((best, score)) = ranked.first() else {
        return refusal("empty_or_input_limit");
    };
    let answer = best.span.text(context).unwrap_or("");
    let second = ranked
        .iter()
        .skip(1)
        .find(|(c, _)| c.span.text(context) != Some(answer))
        .map_or(*score, |(_, s)| *s);
    let margin = score - second;
    if !margin.is_finite() {
        return refusal("non_finite_model_score");
    }
    let accepted = p.supported && margin >= model.map_or(0.0, |m| m.threshold);
    Prediction {
        status: if accepted {
            Status::Predicted
        } else {
            Status::Refused
        },
        answer: if accepted {
            answer.into()
        } else {
            String::new()
        },
        proposal: Some(best.span.clone()),
        evidence: Some(best.evidence.clone()),
        score: *score,
        margin,
        reason: if !p.supported {
            "no_lexical_support"
        } else if !accepted {
            "below_train_calibrated_threshold"
        } else {
            "model_prediction_not_verified"
        }
        .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finite_weights_cannot_emit_non_finite_scores() {
        let model = Model {
            version: 1,
            weights: vec![f32::MAX; DIM],
            threshold: 0.0,
            fit_rows: 0,
            representable_rows: 0,
        };
        let result = model.predict("Ana visited Lima.", "Who visited Lima?");
        assert_eq!(result.status, Status::Refused);
        assert_eq!(result.reason, "non_finite_model_score");
        assert!(result.answer.is_empty());
        assert!(result.score.is_finite());
    }
    #[test]
    fn unicode_offsets_and_trailing_text() {
        let context = "Élodie visited Zürich. Unrelated trailing text.";
        let p = baseline(context, "Who visited Zürich?");
        let span = p.proposal.as_ref();
        assert!(span.and_then(|s| s.text(context)).is_some());
        assert!(p.evidence.as_ref().and_then(|s| s.text(context)).is_some());
        assert_eq!(Span { start: 0, end: 7 }.text(context), Some("Élodie"));
        assert_eq!(Span { start: 0, end: 1 }.text(context), None);
    }
    #[test]
    fn no_support_and_bounds_refuse() {
        assert_eq!(
            baseline("Lena owns a bicycle.", "When did the volcano erupt?").status,
            Status::Refused
        );
        assert_eq!(baseline("", "Who arrived?").status, Status::Refused);
        assert_eq!(
            baseline(&"x".repeat(65537), "Which x?").status,
            Status::Refused
        );
    }
    #[test]
    fn metadata_is_not_runtime_input() {
        assert!(
            serde_json::from_str::<Input>(r#"{"context":"a","question":"b","id":"c"}"#).is_err()
        );
    }
    #[test]
    fn repeated_occurrences_are_deterministic_and_slices() {
        let c = "Ana visited Lima. Ana visited Lima.";
        let a = baseline(c, "Who visited Lima?");
        let b = baseline(c, "Who visited Lima?");
        assert_eq!(a.proposal, b.proposal);
        assert_eq!(
            a.proposal.as_ref().and_then(|s| s.text(c)),
            Some(a.answer.as_str())
        );
    }
    #[test]
    fn trained_model_roundtrip_and_refusal_policy() -> Result<(), Box<dyn std::error::Error>> {
        let rows = vec![TrainingRow {
            context: "Élodie visited Zürich.".into(),
            question: "Who visited Zürich?".into(),
            answer: Span { start: 0, end: 7 },
        }];
        let mut model = Model::train(&rows);
        assert_eq!(model.representable_rows, 1);
        let encoded = serde_json::to_string(&model)?;
        let loaded: Model = serde_json::from_str(&encoded)?;
        loaded.validate()?;
        assert_eq!(
            loaded
                .predict("Élodie visited Zürich.", "Who visited Zürich?")
                .status,
            Status::Refused
        );
        model.threshold = 0.0;
        let p = model.predict("Élodie visited Zürich.", "Who visited Zürich?");
        assert_eq!(p.status, Status::Predicted);
        assert_eq!(
            p.proposal
                .as_ref()
                .and_then(|s| s.text("Élodie visited Zürich.")),
            Some(p.answer.as_str())
        );
        model.version = 0;
        assert_eq!(
            model
                .predict("Élodie visited Zürich.", "Who visited Zürich?")
                .reason,
            "invalid_model"
        );
        Ok(())
    }
}
