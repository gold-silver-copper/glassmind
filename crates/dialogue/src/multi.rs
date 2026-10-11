//! Utterance-only learned service routing with independent persistent states.
use crate::adapter::{Dialogue, Schema};
use crate::tracker::{DialogueModel, InputTurn, ProposalModel, SessionState, Speaker};
use glassmind_everyday::{Example, TrainOptions};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiModel {
    pub version: u32,
    pub router: ProposalModel,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub independent_router: BTreeMap<String, ProposalModel>,
    pub services: BTreeMap<String, DialogueModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub references: Option<crate::references::ReferenceModel>,
}
#[derive(Debug, Default)]
pub struct MultiSession {
    pub states: BTreeMap<String, SessionState>,
    pub active_services: BTreeSet<String>,
    previous_system: String,
    history: Vec<String>,
}
fn features(system: &str, user: &str, previous: &BTreeSet<String>) -> String {
    let system = system
        .split_whitespace()
        .map(|w| format!("sys{w}"))
        .collect::<Vec<_>>()
        .join(" ");
    let prior = previous
        .iter()
        .map(|s| format!("prior{s}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("{user} {system} {prior}")
}
impl MultiModel {
    pub fn train_router(
        dialogues: &[Dialogue],
        selected: &BTreeSet<String>,
        epochs: usize,
    ) -> Result<ProposalModel, String> {
        let mut rows = Vec::new();
        for dialogue in dialogues {
            let mut system = String::new();
            let mut previous = BTreeSet::new();
            for turn in &dialogue.turns {
                if turn.speaker == "SYSTEM" {
                    system.clone_from(&turn.utterance);
                    continue;
                }
                let active: BTreeSet<_> = turn
                    .frames
                    .iter()
                    .filter(|f| selected.contains(&f.service) && f.state.is_some())
                    .map(|f| f.service.clone())
                    .collect();
                rows.push(Example {
                    text: features(&system, &turn.utterance, &previous),
                    label: if active.is_empty() {
                        "NONE".into()
                    } else {
                        active.iter().cloned().collect::<Vec<_>>().join("|")
                    },
                });
                previous = active;
            }
        }
        ProposalModel::train(
            &rows,
            TrainOptions {
                epochs,
                char_weight: 0.15,
                min_df: 2,
            },
        )
    }
    pub fn train_independent_router(
        dialogues: &[Dialogue],
        selected: &BTreeSet<String>,
        epochs: usize,
    ) -> Result<BTreeMap<String, ProposalModel>, String> {
        let mut rows: BTreeMap<String, Vec<Example>> =
            selected.iter().map(|s| (s.clone(), Vec::new())).collect();
        for dialogue in dialogues {
            let mut system = String::new();
            let mut previous = BTreeSet::new();
            for turn in &dialogue.turns {
                if turn.speaker == "SYSTEM" {
                    system.clone_from(&turn.utterance);
                    continue;
                }
                let active: BTreeSet<_> = turn
                    .frames
                    .iter()
                    .filter(|f| selected.contains(&f.service) && f.state.is_some())
                    .map(|f| f.service.clone())
                    .collect();
                let text = features(&system, &turn.utterance, &previous);
                for (service, rows) in &mut rows {
                    rows.push(Example {
                        text: text.clone(),
                        label: if active.contains(service) {
                            "ACTIVE"
                        } else {
                            "INACTIVE"
                        }
                        .into(),
                    });
                }
                previous = active;
            }
        }
        rows.into_iter()
            .map(|(s, rows)| {
                Ok((
                    s,
                    ProposalModel::train(
                        &rows,
                        TrainOptions {
                            epochs,
                            char_weight: 0.15,
                            min_df: 2,
                        },
                    )?,
                ))
            })
            .collect()
    }
    pub fn validate(&self, schemas: &[Schema]) -> Result<(), String> {
        if self.version != 1
            || schemas.len() != self.services.len()
            || schemas
                .iter()
                .map(|s| &s.service_name)
                .collect::<BTreeSet<_>>()
                .len()
                != schemas.len()
        {
            return Err("multi model/schema mismatch".into());
        }
        self.router.validate()?;
        if let Some(r) = &self.references {
            r.validate(schemas)?;
        }
        if !self.independent_router.is_empty()
            && self.independent_router.keys().ne(self.services.keys())
        {
            return Err("independent router/schema mismatch".into());
        }
        for model in self.independent_router.values() {
            model.validate()?;
        }
        for s in schemas {
            self.services
                .get(&s.service_name)
                .ok_or("missing service model")?
                .validate(s)?;
        }
        Ok(())
    }
    /// No annotation-bearing type crosses this runtime boundary. All service
    /// states are available after every turn, including inactive services.
    pub fn advance(&self, session: &mut MultiSession, turn: &InputTurn, schemas: &[Schema]) {
        if matches!(turn.speaker, Speaker::System) {
            session.previous_system.clone_from(&turn.utterance);
            if self.references.is_some() {
                session.history.push(turn.utterance.clone());
            }
            for schema in schemas {
                if let Some(model) = self.services.get(&schema.service_name) {
                    model.advance(
                        session
                            .states
                            .entry(schema.service_name.clone())
                            .or_default(),
                        turn,
                        schema,
                        true,
                    );
                }
            }
            return;
        }
        let text = features(
            &session.previous_system,
            &turn.utterance,
            &session.active_services,
        );
        session.active_services = if self.independent_router.is_empty() {
            self.router
                .predict(&text)
                .split('|')
                .filter(|s| self.services.contains_key(*s))
                .map(str::to_owned)
                .collect()
        } else {
            self.independent_router
                .iter()
                .filter(|(_, m)| m.predict(&text) == "ACTIVE")
                .map(|(s, _)| s.clone())
                .collect()
        };
        let prior = self.references.as_ref().map(|_| session.states.clone());
        for schema in schemas {
            let state = session
                .states
                .entry(schema.service_name.clone())
                .or_default();
            if session.active_services.contains(&schema.service_name)
                && let Some(model) = self.services.get(&schema.service_name)
            {
                model.advance(state, turn, schema, true);
                if let (Some(resolver), Some(prior)) = (&self.references, &prior) {
                    for slot in &schema.slots {
                        if !state.slots.contains_key(&slot.name)
                            && let Some(p) = resolver.propose(
                                &schema.service_name,
                                &slot.name,
                                prior,
                                &session.history,
                                schemas,
                            )
                        {
                            state.slots.insert(
                                slot.name.clone(),
                                crate::tracker::SlotValue::Text(p.value),
                            );
                        }
                    }
                }
            }
        }
        if self.references.is_some() {
            session.history.push(turn.utterance.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_rejects_annotations_and_sparse_models_require_evidence() {
        assert!(
            serde_json::from_str::<InputTurn>(
                r#"{"speaker":"USER","utterance":"hello","frames":[]}"#
            )
            .is_err()
        );
        let rows = vec![Example {
            text: "anything".into(),
            label: "KEEP".into(),
        }];
        let model = ProposalModel::train(
            &rows,
            TrainOptions {
                epochs: 12,
                char_weight: 0.15,
                min_df: 2,
            },
        )
        .expect("one class");
        model.validate().expect("evidence");
        assert_eq!(model.predict("unseen text"), "KEEP");
        assert!(
            ProposalModel::train(
                &[],
                TrainOptions {
                    epochs: 12,
                    char_weight: 0.15,
                    min_df: 2
                }
            )
            .is_err()
        );
        assert!(MultiSession::default().states.is_empty());
    }
    #[test]
    fn switching_preserves_independent_states_and_reset_is_empty() {
        fn constant(label: &str) -> ProposalModel {
            ProposalModel::Constant {
                constant_label: label.into(),
                evidence_rows: 1,
            }
        }
        let schemas: Vec<Schema> = serde_json::from_str(r#"[
            {"service_name":"A","slots":[],"intents":[{"name":"DoA","required_slots":[],"optional_slots":{}}]},
            {"service_name":"B","slots":[],"intents":[{"name":"DoB","required_slots":[],"optional_slots":{}}]}
        ]"#).expect("schemas");
        let mut model = MultiModel {
            version: 1,
            router: constant("A"),
            independent_router: BTreeMap::new(),
            references: None,
            services: schemas
                .iter()
                .map(|s| {
                    (
                        s.service_name.clone(),
                        DialogueModel {
                            version: 2,
                            service: s.service_name.clone(),
                            intent: constant(&format!("SET:Do{}", s.service_name)),
                            categorical: BTreeMap::new(),
                            spans: constant("O"),
                            training_dialogues: 1,
                            training_frames: 1,
                        },
                    )
                })
                .collect(),
        };
        model.validate(&schemas).expect("valid models");
        let mut session = MultiSession::default();
        let user = InputTurn {
            speaker: Speaker::User,
            utterance: "request".into(),
        };
        model.advance(&mut session, &user, &schemas);
        assert_eq!(
            session
                .states
                .get("A")
                .and_then(|s| s.active_intent.as_deref()),
            Some("DoA")
        );
        assert!(
            session
                .states
                .get("B")
                .is_some_and(|s| s.active_intent.is_none())
        );
        model.router = constant("B");
        model.advance(&mut session, &user, &schemas);
        assert_eq!(
            session
                .states
                .get("A")
                .and_then(|s| s.active_intent.as_deref()),
            Some("DoA")
        );
        assert_eq!(
            session
                .states
                .get("B")
                .and_then(|s| s.active_intent.as_deref()),
            Some("DoB")
        );
        assert_eq!(session.active_services, BTreeSet::from(["B".into()]));
        let fresh = MultiSession::default();
        assert!(fresh.states.is_empty() && fresh.active_services.is_empty());
        model.router = constant("unknown");
        model.advance(&mut session, &user, &schemas);
        assert!(session.active_services.is_empty());
        model.independent_router = BTreeMap::from([
            ("A".into(), constant("ACTIVE")),
            ("B".into(), constant("ACTIVE")),
        ]);
        model.advance(&mut session, &user, &schemas);
        assert_eq!(
            session.active_services,
            BTreeSet::from(["A".into(), "B".into()])
        );
    }
    #[test]
    fn references_use_prior_predictions_and_preserve_held_and_filled_states() {
        fn constant(label: &str) -> ProposalModel {
            ProposalModel::Constant {
                constant_label: label.into(),
                evidence_rows: 20,
            }
        }
        let schemas: Vec<Schema> = serde_json::from_str(r#"[
          {"service_name":"A","slots":[{"name":"place","is_categorical":false,"possible_values":[]}],"intents":[{"name":"find","required_slots":["place"],"optional_slots":{}}]},
          {"service_name":"B","slots":[{"name":"city","is_categorical":false,"possible_values":[]}],"intents":[{"name":"find","required_slots":["city"],"optional_slots":{}}]}
        ]"#).expect("schemas");
        let mut model = MultiModel {
            version: 1,
            router: constant("A|B"),
            independent_router: BTreeMap::new(),
            references: Some(crate::references::ReferenceModel {
                version: 1,
                minimum_support: 20,
                relations: vec![crate::references::Relation {
                    source_service: "A".into(),
                    source_slot: "place".into(),
                    target_service: "B".into(),
                    target_slot: "city".into(),
                    matches: 20,
                    opportunities: 20,
                }],
            }),
            services: schemas
                .iter()
                .map(|s| {
                    (
                        s.service_name.clone(),
                        DialogueModel {
                            version: 2,
                            service: s.service_name.clone(),
                            intent: constant("SET:find"),
                            categorical: BTreeMap::new(),
                            spans: constant(if s.service_name == "A" {
                                "B:place"
                            } else {
                                "O"
                            }),
                            training_dialogues: 20,
                            training_frames: 20,
                        },
                    )
                })
                .collect(),
        };
        model.validate(&schemas).expect("valid");
        let user = |s: &str| InputTurn {
            speaker: Speaker::User,
            utterance: s.into(),
        };
        let mut session = MultiSession::default();
        model.advance(&mut session, &user("York"), &schemas);
        // No schema traversal-order leak: B cannot read A's current-turn update.
        assert!(session.states["B"].slots.is_empty());
        model.router = constant("B");
        model.advance(&mut session, &user("there"), &schemas);
        assert_eq!(session.states["B"].slots["city"].text(), "York");
        model.router = constant("A");
        model.advance(&mut session, &user("Oxford"), &schemas);
        assert_eq!(session.states["B"].slots["city"].text(), "York");
        model.router = constant("B");
        model.advance(&mut session, &user("there"), &schemas);
        assert_eq!(session.states["B"].slots["city"].text(), "York");
        let mut fresh = MultiSession::default();
        model.advance(&mut fresh, &user("there"), &schemas);
        assert!(fresh.states["B"].slots.is_empty());
    }
}
