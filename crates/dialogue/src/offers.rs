//! Training-calibrated context boundaries over one immediately prior system turn.
//! Model stores surrounding words and counts, never candidate values.
use crate::adapter::Schema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct OfferRule {
    pub service: String,
    pub slot: String,
    pub left: Vec<String>,
    pub right: Vec<String>,
    pub fit_correct: usize,
    pub fit_wrong: usize,
    pub calibration_correct: usize,
    pub calibration_wrong: usize,
    pub distinct_values: usize,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OfferModel {
    pub version: u32,
    pub rules: Vec<OfferRule>,
}
#[derive(Debug, Clone, Serialize)]
pub struct OfferProposal {
    pub service: String,
    pub slot: String,
    pub value: String,
    pub source_turn: usize,
    /// UTF-8 byte offsets into the supplied source utterance.
    pub start: usize,
    pub end: usize,
    pub rule: usize,
}
#[derive(Debug)]
struct Token {
    word: String,
    start: usize,
    end: usize,
}
fn tokens(text: &str) -> Vec<Token> {
    let chars: Vec<_> = text.char_indices().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(&(start, c)) = chars.get(i) {
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let word = c.is_alphanumeric();
        i += 1;
        if word {
            while let Some(&(_, c)) = chars.get(i) {
                if c.is_alphanumeric() {
                    i += 1;
                } else if matches!(c, '\'' | '’' | '-')
                    && chars.get(i + 1).is_some_and(|(_, c)| c.is_alphanumeric())
                {
                    i += 2;
                } else {
                    break;
                }
            }
        }
        let end = chars.get(i).map_or(text.len(), |(p, _)| *p);
        out.push(Token {
            word: text[start..end].to_lowercase(),
            start,
            end,
        });
    }
    out
}
fn safe(ts: &[Token]) -> bool {
    !ts.iter().any(|t| {
        matches!(
            t.word.as_str(),
            "no" | "not" | "never" | "instead" | "another" | "different" | "don't" | "or"
        )
    })
}
fn shape(text: &str) -> bool {
    let ts = tokens(text);
    if !text
        .chars()
        .next()
        .is_some_and(|c| c.is_uppercase() || c.is_numeric())
    {
        return false;
    }
    ts.iter().all(|t| {
        text[t.start..]
            .chars()
            .next()
            .is_some_and(|c| c.is_uppercase() || c.is_numeric())
            || matches!(
                t.word.as_str(),
                "of" | "the"
                    | "in"
                    | "and"
                    | "on"
                    | "at"
                    | "de"
                    | "des"
                    | "le"
                    | "la"
                    | "to"
                    | "for"
                    | "'"
                    | "’"
                    | "-"
                    | "&"
                    | "."
                    | "("
                    | ")"
            )
    })
}
fn input_slot(schemas: &[Schema], service: &str, slot: &str) -> bool {
    schemas.iter().any(|s| {
        s.service_name == service
            && s.slots.iter().any(|q| q.name == slot && !q.is_categorical)
            && s.intents.iter().any(|i| {
                i.required_slots.iter().any(|n| n == slot) || i.optional_slots.contains_key(slot)
            })
    })
}
impl OfferModel {
    pub fn validate(&self, schemas: &[Schema]) -> Result<(), String> {
        if self.version != 1 || self.rules.len() > 4096 {
            return Err("invalid offer model".into());
        }
        for r in &self.rules {
            if !input_slot(schemas, &r.service, &r.slot)
                || r.fit_correct < 5
                || r.fit_wrong != 0
                || r.calibration_correct < 3
                || r.calibration_wrong != 0
                || r.distinct_values < 5
                || r.left.is_empty()
                || r.left.len() > 3
                || r.right.is_empty()
                || r.right.len() > 3
            {
                return Err("offer rule lacks calibrated evidence or input schema".into());
            }
        }
        Ok(())
    }
    pub fn propose(
        &self,
        service: &str,
        slot: &str,
        source: (usize, &str),
        user: &str,
        schemas: &[Schema],
    ) -> Option<OfferProposal> {
        if !input_slot(schemas, service, slot) {
            return None;
        }
        let (source_turn, system) = source;
        let ts = tokens(system);
        if ts.len() > 80 || !safe(&ts) || !safe(&tokens(user)) {
            return None;
        }
        let words: Vec<_> = std::iter::once("<bos>")
            .chain(ts.iter().map(|t| t.word.as_str()))
            .chain(std::iter::once("<eos>"))
            .collect();
        let mut candidates = BTreeMap::new();
        for (rule, r) in self.rules.iter().enumerate() {
            if r.service != service || r.slot != slot {
                continue;
            }
            for a in 0..ts.len() {
                if words
                    .get(a.saturating_add(1).saturating_sub(r.left.len())..a + 1)
                    .unwrap_or_default()
                    .iter()
                    .copied()
                    .ne(r.left.iter().map(String::as_str))
                {
                    continue;
                }
                for b in a + 1..=ts.len().min(a + 16) {
                    if words
                        .get(b + 1..words.len().min(b + 1 + r.right.len()))
                        .unwrap_or_default()
                        .iter()
                        .copied()
                        .ne(r.right.iter().map(String::as_str))
                    {
                        continue;
                    }
                    let (Some(first), Some(last)) = (ts.get(a), ts.get(b - 1)) else {
                        continue;
                    };
                    let start = first.start;
                    let end = last.end;
                    let value = &system[start..end];
                    if value.split_whitespace().count() <= 12 && shape(value) {
                        candidates
                            .entry(value.to_lowercase())
                            .or_insert_with(|| OfferProposal {
                                service: service.into(),
                                slot: slot.into(),
                                value: value.into(),
                                source_turn,
                                start,
                                end,
                                rule,
                            });
                    }
                }
            }
        }
        if candidates.len() == 1 {
            candidates.into_values().next()
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    pub fn fixture() -> (Vec<Schema>, OfferModel) {
        let schemas = serde_json::from_str(r#"[{"service_name":"A","slots":[{"name":"place","is_categorical":false,"possible_values":[]}],"intents":[{"name":"go","required_slots":["place"],"optional_slots":{}}]}]"#).expect("schema");
        let model = OfferModel {
            version: 1,
            rules: vec![OfferRule {
                service: "A".into(),
                slot: "place".into(),
                left: vec!["to".into()],
                right: vec!["?".into()],
                fit_correct: 10,
                fit_wrong: 0,
                calibration_correct: 3,
                calibration_wrong: 0,
                distinct_values: 6,
            }],
        };
        (schemas, model)
    }
    #[test]
    fn spans_are_verbatim_with_provenance_and_ambiguity_refuses() {
        let (schemas, model) = fixture();
        model.validate(&schemas).expect("calibrated");
        let source = "Go to École des Arts?";
        let p = model
            .propose("A", "place", (3, source), "Yes please", &schemas)
            .expect("span");
        assert_eq!(p.value, "École des Arts");
        assert_eq!(&source[p.start..p.end], p.value);
        assert_eq!(p.source_turn, 3);
        for (s, u) in [
            ("Go to Alpha? Go to Beta?", "yes"),
            ("Go to Alpha or Beta?", "yes"),
            ("Do not go to Alpha?", "yes"),
            ("Go to Alpha?", "No, somewhere different"),
            ("Go to Alpha?", "another please"),
        ] {
            assert!(model.propose("A", "place", (3, s), u, &schemas).is_none());
        }
        assert!(
            model
                .propose("A", "unknown", (3, source), "yes", &schemas)
                .is_none()
        );
    }
    #[test]
    fn session_consumes_source_preserves_filled_slots_and_resets() {
        use crate::multi::{MultiModel, MultiSession};
        use crate::tracker::{DialogueModel, InputTurn, ProposalModel, SlotValue, Speaker};
        fn constant(label: &str) -> ProposalModel {
            ProposalModel::Constant {
                constant_label: label.into(),
                evidence_rows: 20,
            }
        }
        let (schemas, offers) = fixture();
        let model = MultiModel {
            version: 1,
            router: constant("A"),
            independent_router: BTreeMap::new(),
            references: None,
            offers: Some(offers),
            services: BTreeMap::from([(
                "A".into(),
                DialogueModel {
                    version: 2,
                    service: "A".into(),
                    intent: constant("SET:go"),
                    categorical: BTreeMap::new(),
                    spans: constant("O"),
                    training_dialogues: 20,
                    training_frames: 20,
                },
            )]),
        };
        let mut session = MultiSession::default();
        let user = InputTurn {
            speaker: Speaker::User,
            utterance: "yes".into(),
        };
        let system = |s: &str| InputTurn {
            speaker: Speaker::System,
            utterance: s.into(),
        };
        model.advance(&mut session, &system("Go to Alpha?"), &schemas);
        model.advance(&mut session, &user, &schemas);
        assert_eq!(session.states["A"].slots["place"].text(), "Alpha");
        assert_eq!(session.offer_proposals[0].source_turn, 0);
        session.states.get_mut("A").expect("state").slots.clear();
        model.advance(&mut session, &user, &schemas);
        assert!(session.states["A"].slots.is_empty()); // consumed, no stale retry
        model.advance(&mut session, &system("Go to Beta?"), &schemas);
        model.advance(&mut session, &system("How many people?"), &schemas);
        model.advance(&mut session, &user, &schemas);
        assert!(session.states["A"].slots.is_empty()); // newer source supersedes
        session
            .states
            .get_mut("A")
            .expect("state")
            .slots
            .insert("place".into(), SlotValue::Text("Existing".into()));
        model.advance(&mut session, &system("Go to Gamma?"), &schemas);
        model.advance(&mut session, &user, &schemas);
        assert_eq!(session.states["A"].slots["place"].text(), "Existing");
        let mut fresh = MultiSession::default();
        model.advance(&mut fresh, &user, &schemas);
        assert!(fresh.states["A"].slots.is_empty());
    }
}
