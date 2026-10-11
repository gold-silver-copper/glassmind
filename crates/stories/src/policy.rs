//! Small categorical action model learned only from observed events, never QA labels.
//! Its predictions are defeasible associations, not logical entailments.
use crate::{Entity, Event, World};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionKind {
    Destination,
    Acquisition,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Observation {
    state: Entity,
    action: ActionKind,
    target: Entity,
    count: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActionModel {
    observations: Vec<Observation>,
    pub stories_seen: u64,
    pub stories_skipped: u64,
}

pub(crate) fn actions(event: &Event) -> Vec<(Entity, ActionKind, Entity)> {
    match event {
        Event::Locate {
            actors,
            places,
            negative: false,
        } if places.len() == 1 => places.first().map_or_else(Vec::new, |place| {
            actors
                .iter()
                .map(|actor| (actor.clone(), ActionKind::Destination, place.clone()))
                .collect()
        }),
        Event::Take { actor, object } => {
            vec![(actor.clone(), ActionKind::Acquisition, object.clone())]
        }
        _ => Vec::new(),
    }
}

impl ActionModel {
    /// Parse one complete training episode. Unsupported stories contribute no counts.
    pub fn learn(&mut self, story: &[String]) {
        self.stories_seen += 1;
        let mut world = World::default();
        let mut states = BTreeMap::new();
        let mut observations = Vec::new();
        for text in story {
            let Some(event) = world.parse(text) else {
                self.stories_skipped += 1;
                return;
            };
            if let Event::State { actor, state } = &event {
                states.insert(actor.clone(), state.clone());
            }
            for (actor, action, target) in actions(&event) {
                if let Some(state) = states.get(&actor) {
                    observations.push((state.clone(), action, target));
                }
            }
            world.apply(event);
        }
        for (state, action, target) in observations {
            if let Some(row) = self
                .observations
                .iter_mut()
                .find(|r| r.state == state && r.action == action && r.target == target)
            {
                row.count += 1;
            } else {
                self.observations.push(Observation {
                    state,
                    action,
                    target,
                    count: 1,
                });
            }
        }
        self.observations
            .sort_by(|a, b| (&a.state, a.action, &a.target).cmp(&(&b.state, b.action, &b.target)));
    }

    /// At least three matching observations, with no conflicting outcome.
    pub fn predict(&self, state: &Entity, action: ActionKind) -> Option<&Entity> {
        let mut matches = self
            .observations
            .iter()
            .filter(|r| &r.state == state && r.action == action);
        let first = matches.next()?;
        if first.count < 3 || matches.next().is_some() {
            return None;
        }
        Some(&first.target)
    }
}
