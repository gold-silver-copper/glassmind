//! A sparse relation proposer and a finite, provenance-backed facts store.
//! Answer IDs always come from the store, never from classifier weights.
use glassmind_everyday::IntentModel;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PropertyId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub aliases: Vec<String>,
    pub claims: BTreeMap<PropertyId, Vec<EntityId>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub entities: BTreeMap<EntityId, Entity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Answer {
    pub entity: Option<EntityId>,
    pub relation: Option<String>,
    pub answers: BTreeSet<EntityId>,
    pub refusal: Option<String>,
}

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

impl Store {
    pub fn validate(&self) -> Result<(), String> {
        fn identifier(s: &str, prefix: char) -> bool {
            s.starts_with(prefix) && s.len() > 1 && s.chars().skip(1).all(|c| c.is_ascii_digit())
        }
        for (id, entity) in &self.entities {
            if !identifier(&id.0, 'Q') {
                return Err("invalid entity identifier".into());
            }
            for (property, objects) in &entity.claims {
                if !identifier(&property.0, 'P') || objects.iter().any(|id| !identifier(&id.0, 'Q'))
                {
                    return Err("invalid claim identifier".into());
                }
            }
        }
        Ok(())
    }

    /// Longest whole-token alias; equal-length matches to distinct entities refuse.
    pub fn link(&self, question: &str) -> Option<EntityId> {
        let question = format!(" {} ", normalize(question));
        let mut best_length = 0;
        let mut candidates = BTreeSet::new();
        for (id, entity) in &self.entities {
            for alias in &entity.aliases {
                let alias = normalize(alias);
                if alias.len() < 2
                    || alias.len() < best_length
                    || !question.contains(&format!(" {alias} "))
                {
                    continue;
                }
                if alias.len() > best_length {
                    candidates.clear();
                    best_length = alias.len();
                }
                candidates.insert(id.clone());
            }
        }
        if candidates.len() == 1 {
            candidates.into_iter().next()
        } else {
            None
        }
    }

    pub fn answer(&self, model: &IntentModel, question: &str) -> Answer {
        let mut answer = Answer {
            entity: None,
            relation: None,
            answers: BTreeSet::new(),
            refusal: None,
        };
        let Some(entity) = self.link(question) else {
            answer.refusal = Some("no unique entity alias in this snapshot".into());
            return answer;
        };
        answer.entity = Some(entity.clone());
        let prediction = model.predict(question);
        answer.relation = Some(prediction.label.clone());
        if !prediction.accepted {
            answer.refusal = Some("uncertain relation".into());
            return answer;
        }
        if prediction.label.starts_with('P') {
            if let Some(values) = self
                .entities
                .get(&entity)
                .and_then(|e| e.claims.get(&PropertyId(prediction.label)))
            {
                answer.answers.extend(values.iter().cloned());
            }
        } else if let Some(suffix) = prediction.label.strip_prefix('R') {
            let property = PropertyId(format!("P{suffix}"));
            for (subject, row) in &self.entities {
                if row
                    .claims
                    .get(&property)
                    .is_some_and(|objects| objects.contains(&entity))
                {
                    answer.answers.insert(subject.clone());
                }
            }
        }
        if answer.answers.is_empty() {
            answer.refusal = Some("claim missing from this finite snapshot".into());
        } else if answer.answers.len() > 1 {
            answer.answers.clear();
            answer.refusal = Some("this snapshot has multiple answers; narrow the question".into());
        }
        answer
    }
}
