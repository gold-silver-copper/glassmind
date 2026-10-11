//! Value-free learned relations; proposals can only copy grounded prior predictions.
use crate::{
    adapter::Schema,
    tracker::{SessionState, SlotValue},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceModel {
    pub version: u32,
    pub minimum_support: usize,
    pub relations: Vec<Relation>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    pub source_service: String,
    pub source_slot: String,
    pub target_service: String,
    pub target_slot: String,
    pub matches: usize,
    pub opportunities: usize,
}
#[derive(Debug, PartialEq, Eq)]
pub struct CopyProposal {
    pub value: String,
    pub sources: Vec<(String, String)>,
}
fn input_text_slot(schemas: &[Schema], service: &str, slot: &str) -> bool {
    schemas.iter().any(|s| {
        s.service_name == service
            && s.slots.iter().any(|q| q.name == slot && !q.is_categorical)
            && s.intents.iter().any(|i| {
                i.required_slots.iter().any(|n| n == slot) || i.optional_slots.contains_key(slot)
            })
    })
}
/// Case insensitive whole-span support; no token concatenation or value synthesis.
fn supported(value: &str, history: &[String]) -> bool {
    let value = value.to_lowercase();
    if value.is_empty() || value.split_whitespace().count() > 12 {
        return false;
    }
    history.iter().any(|text| {
        let text = text.to_lowercase();
        text.match_indices(&value).any(|(i, _)| {
            !text[..i]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric)
                && !text[i + value.len()..]
                    .chars()
                    .next()
                    .is_some_and(char::is_alphanumeric)
        })
    })
}
impl ReferenceModel {
    pub fn validate(&self, schemas: &[Schema]) -> Result<(), String> {
        if self.version != 1 || self.minimum_support < 20 {
            return Err("invalid reference model".into());
        }
        for r in &self.relations {
            if r.source_service == r.target_service
                || r.matches != r.opportunities
                || r.matches < self.minimum_support
                || !input_text_slot(schemas, &r.source_service, &r.source_slot)
                || !input_text_slot(schemas, &r.target_service, &r.target_slot)
            {
                return Err("reference relation lacks evidence or input text schema".into());
            }
        }
        Ok(())
    }
    /// All sources are a snapshot from BEFORE the user turn. Refuse disagreement,
    /// unsupported values, categorical values, and schema-invalid destinations.
    pub fn propose(
        &self,
        target: &str,
        slot: &str,
        prior: &BTreeMap<String, SessionState>,
        history: &[String],
        schemas: &[Schema],
    ) -> Option<CopyProposal> {
        if !input_text_slot(schemas, target, slot) {
            return None;
        }
        let mut proposal: Option<CopyProposal> = None;
        for r in &self.relations {
            if r.target_service != target
                || r.target_slot != slot
                || r.matches != r.opportunities
                || r.matches < self.minimum_support
                || !input_text_slot(schemas, &r.source_service, &r.source_slot)
            {
                continue;
            }
            let Some(SlotValue::Text(value)) = prior
                .get(&r.source_service)
                .and_then(|s| s.slots.get(&r.source_slot))
            else {
                continue;
            };
            if !supported(value, history) {
                continue;
            }
            if let Some(p) = &mut proposal {
                if !p.value.eq_ignore_ascii_case(value) {
                    return None;
                }
                p.sources
                    .push((r.source_service.clone(), r.source_slot.clone()));
            } else {
                proposal = Some(CopyProposal {
                    value: value.clone(),
                    sources: vec![(r.source_service.clone(), r.source_slot.clone())],
                });
            }
        }
        proposal
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn schemas() -> Vec<Schema> {
        serde_json::from_str(r#"[
          {"service_name":"A","slots":[{"name":"place","is_categorical":false,"possible_values":[]}],"intents":[{"name":"find","required_slots":["place"],"optional_slots":{}}]},
          {"service_name":"B","slots":[{"name":"city","is_categorical":false,"possible_values":[]},{"name":"output","is_categorical":false,"possible_values":[]}],"intents":[{"name":"find","required_slots":["city"],"optional_slots":{}}]},
          {"service_name":"C","slots":[{"name":"place","is_categorical":false,"possible_values":[]}],"intents":[{"name":"find","required_slots":["place"],"optional_slots":{}}]}
        ]"#).expect("schema")
    }
    fn relation(source: &str) -> Relation {
        Relation {
            source_service: source.into(),
            source_slot: "place".into(),
            target_service: "B".into(),
            target_slot: "city".into(),
            matches: 20,
            opportunities: 20,
        }
    }
    #[test]
    fn copying_requires_provenance_schema_and_unambiguous_prior() {
        let schemas = schemas();
        let mut model = ReferenceModel {
            version: 1,
            minimum_support: 20,
            relations: vec![relation("A")],
        };
        model.validate(&schemas).expect("valid");
        let mut prior = BTreeMap::<String, SessionState>::new();
        prior
            .entry("A".into())
            .or_default()
            .slots
            .insert("place".into(), SlotValue::Text("York".into()));
        assert!(
            model
                .propose("B", "city", &prior, &["Yorkshire".into()], &schemas)
                .is_none()
        );
        let history = vec!["York or Oxford?".into()];
        let p = model
            .propose("B", "city", &prior, &history, &schemas)
            .expect("grounded");
        assert_eq!(p.value, "York");
        assert_eq!(p.sources, vec![("A".into(), "place".into())]);
        assert!(
            model
                .propose("B", "output", &prior, &history, &schemas)
                .is_none()
        );
        assert!(
            model
                .propose("B", "city", &BTreeMap::new(), &history, &schemas)
                .is_none()
        );
        model.relations.push(relation("C"));
        prior
            .entry("C".into())
            .or_default()
            .slots
            .insert("place".into(), SlotValue::Text("Oxford".into()));
        assert!(
            model
                .propose("B", "city", &prior, &history, &schemas)
                .is_none()
        );
        prior.remove("C");
        prior
            .get_mut("A")
            .expect("source")
            .slots
            .insert("place".into(), SlotValue::Categorical("York".into()));
        assert!(
            model
                .propose("B", "city", &prior, &history, &schemas)
                .is_none()
        );
        model.relations[0].matches = 19;
        assert!(model.validate(&schemas).is_err());
    }
}
