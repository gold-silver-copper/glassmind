//! Optional TRAIN-calibrated punctuation restoration for existing reference copies.
//! Every output is a contiguous slice of the most recent exact source occurrence.
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryModel {
    pub version: u32,
    pub rules: Vec<BoundaryRule>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryRule {
    pub shape: String,
    pub punctuation: String,
    pub extension_chars: usize,
    pub fit_conversations: usize,
    pub calibration_conversations: usize,
    pub mismatches: usize,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BoundaryProposal {
    pub service: String,
    pub slot: String,
    pub original: String,
    pub value: String,
    pub source_turn: usize,
    pub start: usize,
    pub end: usize,
    pub rule: usize,
}
fn shape(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value
        .split_whitespace()
        .next_back()
        .unwrap_or("")
        .chars()
        .peekable();
    while let Some(c) = chars.next() {
        let class = if c.is_alphabetic() {
            Some('A')
        } else if c.is_numeric() {
            Some('D')
        } else {
            None
        };
        if let Some(class) = class {
            let mut count = 1usize;
            while chars.peek().is_some_and(|q| {
                if class == 'A' {
                    q.is_alphabetic()
                } else {
                    q.is_numeric()
                }
            }) {
                chars.next();
                count += 1;
            }
            out.push(class);
            out.push(char::from(b'0' + count.min(3) as u8));
        } else {
            out.push(c);
        }
    }
    out
}
fn punctuation(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace() && !matches!(c, '\'' | '-' | '_')
}
impl BoundaryModel {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.rules.len() > 256 {
            return Err("invalid boundary model version or size".into());
        }
        for r in &self.rules {
            if r.shape.is_empty()
                || r.shape.len() > 64
                || r.punctuation.is_empty()
                || r.punctuation.chars().count() > 3
                || !r.punctuation.chars().all(punctuation)
                || r.extension_chars == 0
                || r.extension_chars > r.punctuation.chars().count()
                || r.fit_conversations < 5
                || r.calibration_conversations < 3
                || r.mismatches != 0
            {
                return Err("boundary rule lacks bounded shape or calibration evidence".into());
            }
        }
        Ok(())
    }
    /// Most recent exact occurrence only: never fall back to older punctuation.
    /// No lowercase offset mapping, text synthesis, entity store, or gold input.
    pub fn propose(&self, value: &str, history: &[String]) -> Option<BoundaryProposal> {
        if value.is_empty() || value.split_whitespace().count() > 12 {
            return None;
        }
        let sh = shape(value);
        for (source_turn, source) in history.iter().enumerate().rev() {
            let occurrence = source
                .match_indices(value)
                .filter(|(start, _)| {
                    !source[..*start]
                        .chars()
                        .next_back()
                        .is_some_and(char::is_alphanumeric)
                        && !source[*start + value.len()..]
                            .chars()
                            .next()
                            .is_some_and(char::is_alphanumeric)
                })
                .last();
            let Some((start, _)) = occurrence else {
                continue;
            };
            let end = start + value.len();
            let punct: String = source[end..]
                .chars()
                .take_while(|c| punctuation(*c))
                .take(4)
                .collect();
            let mut result: Option<BoundaryProposal> = None;
            for (rule, r) in self.rules.iter().enumerate() {
                if r.shape != sh || r.punctuation != punct {
                    continue;
                }
                let extension: usize = punct
                    .chars()
                    .take(r.extension_chars)
                    .map(char::len_utf8)
                    .sum();
                let new_end = end + extension;
                if let Some(previous) = &result {
                    if previous.end != new_end {
                        return None;
                    }
                } else {
                    result = Some(BoundaryProposal {
                        service: String::new(),
                        slot: String::new(),
                        original: value.into(),
                        value: source.get(start..new_end)?.into(),
                        source_turn,
                        start,
                        end: new_end,
                        rule,
                    });
                }
            }
            return result;
        }
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn model() -> BoundaryModel {
        BoundaryModel {
            version: 1,
            rules: vec![BoundaryRule {
                shape: "A1.A1".into(),
                punctuation: ".".into(),
                extension_chars: 1,
                fit_conversations: 5,
                calibration_conversations: 3,
                mismatches: 0,
            }],
        }
    }
    #[test]
    fn exact_unicode_provenance_and_latest_occurrence() {
        let m = model();
        m.validate().expect("valid");
        let history = vec!["Try Évora Q.R. today".into()];
        let p = m.propose("Évora Q.R", &history).expect("learned boundary");
        assert_eq!(p.value, "Évora Q.R.");
        assert_eq!(&history[p.source_turn][p.start..p.end], p.value);
        assert!(
            m.propose(
                "Évora Q.R",
                &[history[0].clone(), "Évora Q.R tomorrow".into()]
            )
            .is_none()
        );
        assert!(m.propose("Q.R", &["XQ.R.".into()]).is_none());
        assert!(m.propose("Évora Q.R", &["Évora Q.R.,".into()]).is_none());
        assert!(m.propose("Évora Q.R", &[]).is_none());
        assert_eq!(shape("Station 239 B.C"), "A1.A1");
        assert_eq!(shape("Zürich"), "A3");
    }
    #[test]
    fn evidence_and_conflicting_rules_refuse() {
        let mut m = model();
        m.rules[0].calibration_conversations = 2;
        assert!(m.validate().is_err());
        m.rules[0].calibration_conversations = 3;
        m.rules[0].punctuation = ".)".into();
        m.rules.push(BoundaryRule {
            shape: "A1.A1".into(),
            punctuation: ".)".into(),
            extension_chars: 2,
            fit_conversations: 5,
            calibration_conversations: 3,
            mismatches: 0,
        });
        assert!(m.propose("Q.R", &["Q.R.)".into()]).is_none());
    }
}
