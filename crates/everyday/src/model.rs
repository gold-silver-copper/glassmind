//! TF-IDF word/character n-grams with an averaged multiclass margin model.
//! Feature-major weights keep sparse scoring cheap. No stored answers,
//! retrieval examples, runtime neural model, or benchmark-specific vocabulary.
use crate::adapter::{Example, Totals};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainOptions {
    pub epochs: usize,
    pub char_weight: f32,
    pub min_df: usize,
}

impl Default for TrainOptions {
    fn default() -> Self {
        Self {
            epochs: 20,
            char_weight: 0.3,
            min_df: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Feature {
    index: usize,
    idf: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CalibrationBin {
    pub count: usize,
    pub correct: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefusalPolicy {
    pub min_score: f32,
    pub min_margin: f32,
    /// Selection objective: right - wrong_cost * wrong; refusals have zero cost.
    pub wrong_cost: f32,
    pub dev_rows: usize,
    pub confidence_bins: Vec<CalibrationBin>,
}

impl Default for RefusalPolicy {
    fn default() -> Self {
        Self {
            min_score: f32::MAX,
            min_margin: f32::MAX,
            wrong_cost: 2.0,
            dev_rows: 0,
            confidence_bins: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentModel {
    pub format_version: u32,
    pub labels: Vec<String>,
    pub training_rows: usize,
    pub options: TrainOptions,
    vocabulary: BTreeMap<String, Feature>,
    weights: Vec<f32>,
    pub policy: RefusalPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub label: String,
    pub accepted: bool,
    pub score: f32,
    pub margin: f32,
    /// Empirical validation precision in this margin bin, not a posterior.
    pub dev_confidence: Option<f32>,
    pub confidence_support: usize,
    pub known_features: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Contribution {
    pub feature: String,
    pub contribution: f32,
}

fn features(text: &str, char_weight: f32) -> BTreeMap<String, f32> {
    let text = text.to_lowercase().replace('’', "'");
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .collect();
    let mut result = BTreeMap::new();
    for word in &words {
        result.insert(format!("w:{word}"), 1.0);
        if char_weight > 0.0 {
            let chars: Vec<char> = format!("^{word}$").chars().collect();
            for n in 3..=5 {
                for window in chars.windows(n) {
                    result.insert(
                        format!("c:{}", window.iter().collect::<String>()),
                        char_weight,
                    );
                }
            }
        }
    }
    for pair in words.windows(2) {
        result.insert(format!("b:{}", pair.join(" ")), 0.8);
    }
    result
}

fn best(scores: &[f32], excluding: Option<usize>) -> Option<(usize, f32)> {
    scores
        .iter()
        .copied()
        .enumerate()
        .filter(|(i, _)| Some(*i) != excluding)
        .max_by(|a, b| a.1.total_cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
}

fn score_vector(weights: &[f32], classes: usize, vector: &[(usize, f32)]) -> Vec<f32> {
    let mut scores = vec![0.0; classes];
    for &(index, value) in vector {
        if let Some(row) = weights.get(index * classes..(index + 1) * classes) {
            for (score, weight) in scores.iter_mut().zip(row) {
                *score += value * weight;
            }
        }
    }
    scores
}

impl IntentModel {
    pub fn train(rows: &[Example], options: TrainOptions) -> Result<Self, String> {
        if rows.is_empty()
            || options.min_df == 0
            || !options.char_weight.is_finite()
            || options.char_weight < 0.0
        {
            return Err("nonempty training data, positive min_df and finite nonnegative char_weight required".into());
        }
        let labels: Vec<String> = rows
            .iter()
            .filter(|r| r.label != "oos")
            .map(|r| r.label.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if labels.len() < 2 {
            return Err("at least two in-scope labels required".into());
        }
        let training: Vec<&Example> = rows.iter().filter(|r| r.label != "oos").collect();
        let mut df = BTreeMap::<String, usize>::new();
        for row in &training {
            for key in features(&row.text, options.char_weight).into_keys() {
                *df.entry(key).or_default() += 1;
            }
        }
        let vocabulary: BTreeMap<String, Feature> = df
            .into_iter()
            .filter(|(_, count)| *count >= options.min_df)
            .enumerate()
            .map(|(index, (key, count))| {
                (
                    key,
                    Feature {
                        index,
                        idf: (1.0 + training.len() as f32 / (1.0 + count as f32)).ln(),
                    },
                )
            })
            .collect();
        if vocabulary.is_empty() {
            return Err("training vocabulary is empty".into());
        }
        let mut model = Self {
            format_version: 1,
            weights: vec![0.0; vocabulary.len() * labels.len()],
            labels,
            training_rows: training.len(),
            options,
            vocabulary,
            policy: RefusalPolicy::default(),
        };
        let vectors: Vec<_> = training
            .iter()
            .map(|r| {
                (
                    model.vector(&r.text),
                    model.labels.binary_search(&r.label).unwrap_or(0),
                )
            })
            .collect();
        // A train-only nearest-centroid baseline also initializes margin learning.
        for (vector, label) in &vectors {
            for &(feature, value) in vector {
                if let Some(weight) = model.weights.get_mut(feature * model.labels.len() + label) {
                    *weight += value;
                }
            }
        }
        let mut norms = vec![0.0_f32; model.labels.len()];
        for row in model.weights.chunks(model.labels.len()) {
            for (norm, weight) in norms.iter_mut().zip(row) {
                *norm += weight * weight;
            }
        }
        for row in model.weights.chunks_mut(model.labels.len()) {
            for (weight, norm) in row.iter_mut().zip(&norms) {
                *weight /= norm.sqrt().max(1e-6);
            }
        }
        let mut average = vec![0.0; model.weights.len()];
        let mut averaged_epochs = 0;
        let mut order: Vec<usize> = (0..vectors.len()).collect();
        let mut rng = 0x1ced_cafe_u64;
        for epoch in 0..model.options.epochs {
            // Fixed reproducible Fisher-Yates shuffle, no random dependency.
            for i in (1..order.len()).rev() {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                order.swap(i, rng as usize % (i + 1));
            }
            let mut updates = 0;
            for &index in &order {
                let Some((vector, label)) = vectors.get(index) else {
                    continue;
                };
                let scores = score_vector(&model.weights, model.labels.len(), vector);
                let Some((rival, rival_score)) = best(&scores, Some(*label)) else {
                    continue;
                };
                let own_score = scores.get(*label).copied().unwrap_or(0.0);
                let loss = 1.0 + rival_score - own_score;
                if loss <= 0.0 {
                    continue;
                }
                let rate = (loss / 2.0).min(0.5);
                for &(feature, value) in vector {
                    if let Some(w) = model.weights.get_mut(feature * model.labels.len() + label) {
                        *w += rate * value;
                    }
                    if let Some(w) = model.weights.get_mut(feature * model.labels.len() + rival) {
                        *w -= rate * value;
                    }
                }
                updates += 1;
            }
            if epoch >= model.options.epochs / 2 {
                for (total, weight) in average.iter_mut().zip(&model.weights) {
                    *total += weight;
                }
                averaged_epochs += 1;
            }
            eprintln!("epoch={} margin_updates={updates}", epoch + 1);
        }
        if averaged_epochs > 0 {
            for (weight, total) in model.weights.iter_mut().zip(average) {
                *weight = total / averaged_epochs as f32;
            }
        }
        Ok(model)
    }

    fn vector(&self, text: &str) -> Vec<(usize, f32)> {
        let mut values: Vec<_> = features(text, self.options.char_weight)
            .into_iter()
            .filter_map(|(key, value)| self.vocabulary.get(&key).map(|f| (f.index, value * f.idf)))
            .collect();
        let norm = values
            .iter()
            .map(|(_, v)| v * v)
            .sum::<f32>()
            .sqrt()
            .max(1e-6);
        for (_, value) in &mut values {
            *value /= norm;
        }
        values
    }

    pub fn feature_count(&self) -> usize {
        self.vocabulary.len()
    }

    /// Check deserialized dimensions before accepting a model from disk.
    pub fn validate(&self) -> Result<(), String> {
        if self.format_version != 1
            || self.labels.len() < 2
            || self.weights.len() != self.vocabulary.len().saturating_mul(self.labels.len())
            || self
                .vocabulary
                .values()
                .any(|f| f.index >= self.vocabulary.len() || !f.idf.is_finite())
            || self.weights.iter().any(|v| !v.is_finite())
        {
            return Err("invalid model version, dimensions, or numeric weights".into());
        }
        Ok(())
    }

    pub fn predict(&self, text: &str) -> Prediction {
        let vector = self.vector(text);
        let scores = score_vector(&self.weights, self.labels.len(), &vector);
        let (label, score) = best(&scores, None).unwrap_or((0, 0.0));
        let runner = best(&scores, Some(label)).map(|(_, v)| v).unwrap_or(score);
        let margin = score - runner;
        let bin = self.policy.confidence_bins.get(confidence_bin(margin));
        Prediction {
            label: self.labels.get(label).cloned().unwrap_or_default(),
            accepted: !vector.is_empty()
                && score >= self.policy.min_score
                && margin >= self.policy.min_margin,
            score,
            margin,
            dev_confidence: bin
                .filter(|b| b.count > 0)
                .map(|b| b.correct as f32 / b.count as f32),
            confidence_support: bin.map_or(0, |b| b.count),
            known_features: vector.len(),
        }
    }

    /// Contributions to the proposed class, sorted by absolute contribution.
    pub fn explain(&self, text: &str, limit: usize) -> Vec<Contribution> {
        let prediction = self.predict(text);
        let Ok(label) = self.labels.binary_search(&prediction.label) else {
            return vec![];
        };
        let values: BTreeMap<usize, f32> = self.vector(text).into_iter().collect();
        let mut result: Vec<_> = self
            .vocabulary
            .iter()
            .filter_map(|(name, feature)| {
                let value = values.get(&feature.index)?;
                let weight = self
                    .weights
                    .get(feature.index * self.labels.len() + label)?;
                Some(Contribution {
                    feature: name.clone(),
                    contribution: value * weight,
                })
            })
            .collect();
        result.sort_by(|a, b| b.contribution.abs().total_cmp(&a.contribution.abs()));
        result.truncate(limit);
        result
    }

    /// Grid-select rejection boundaries on DEV ONLY. All reported evaluation
    /// on these rows is selection performance, not an unbiased test estimate.
    pub fn calibrate(&mut self, dev: &[Example], wrong_cost: f32) -> Result<Totals, String> {
        if dev.is_empty() || !wrong_cost.is_finite() || wrong_cost < 0.0 {
            return Err("nonempty dev and finite nonnegative wrong_cost required".into());
        }
        let predictions: Vec<_> = dev.iter().map(|row| self.predict(&row.text)).collect();
        let mut bins = vec![CalibrationBin::default(); 20];
        for (row, prediction) in dev.iter().zip(&predictions) {
            if let Some(bin) = bins.get_mut(confidence_bin(prediction.margin)) {
                bin.count += 1;
                bin.correct += usize::from(row.label == prediction.label);
            }
        }
        let mut best_utility = f32::NEG_INFINITY;
        let mut chosen = (0.0, 0.0);
        let mut selected = Totals::default();
        for score_step in 0..=40 {
            let min_score = score_step as f32 * 0.05;
            for margin_step in 0..=30 {
                let min_margin = margin_step as f32 * 0.025;
                let mut totals = Totals::default();
                for (row, prediction) in dev.iter().zip(&predictions) {
                    let mut p = prediction.clone();
                    p.accepted =
                        p.known_features > 0 && p.score >= min_score && p.margin >= min_margin;
                    totals.observe(&row.label, &p);
                }
                let utility = totals.right as f32 - wrong_cost * totals.wrong as f32;
                if utility > best_utility {
                    best_utility = utility;
                    chosen = (min_score, min_margin);
                    selected = totals;
                }
            }
        }
        self.policy = RefusalPolicy {
            min_score: chosen.0,
            min_margin: chosen.1,
            wrong_cost,
            dev_rows: dev.len(),
            confidence_bins: bins,
        };
        Ok(selected)
    }
}

fn confidence_bin(margin: f32) -> usize {
    ((margin.max(0.0) * 10.0) as usize).min(19)
}
