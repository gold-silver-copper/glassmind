//! A sparse relation proposer and a finite, provenance-backed facts store.
//! Answer IDs always come from the store, never from classifier weights.
use glassmind_everyday::IntentModel;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod matching;
pub use matching::{
    MAX_MATCH_CONDITIONS, MAX_MATCH_PARSE_ALTERNATIVES, MAX_MATCH_QUESTION_BYTES, MatchConstraint,
    MatchPredicate, MatchRefusalReason, MatchResult, MatchUnknownReason, MatchWitness,
    ResolvedMatchConstraint, SubjectMatch, parse_match_question,
};

mod verification;
pub use verification::{
    EdgeWitness, EndpointResolution, MAX_ENDPOINT_BYTES, MAX_VERIFY_QUESTION_BYTES, RefusalReason,
    UnknownReason, Verification,
};

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

/// The first ten IDs in lexical order, with counts for this store only.
#[derive(Debug, Clone, Serialize)]
pub struct AnswerSet {
    #[serde(flatten)]
    pub answer: Answer,
    pub total_snapshot_matches: usize,
    pub returned: usize,
    pub truncated: bool,
}

pub const SET_DISPLAY_LIMIT: usize = 10;

fn normalize(text: &str) -> String {
    tokenize(&text.to_lowercase())
}

fn tokenize(text: &str) -> String {
    text.split(|c: char| !c.is_alphanumeric())
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
    /// Single-token uppercase aliases need their original casing: folding an
    /// acronym can turn an ordinary word in the question into an entity mention.
    pub fn link(&self, question: &str) -> Option<EntityId> {
        let cased_question = format!(" {} ", tokenize(question));
        let question = format!(" {} ", normalize(question));
        let mut best_length = 0;
        let mut candidates = BTreeSet::new();
        let mut supported = BTreeSet::new();
        for (id, entity) in &self.entities {
            for alias in &entity.aliases {
                let case_supported = !alias.chars().all(|c| c.is_alphabetic() && c.is_uppercase())
                    || cased_question.contains(&format!(" {alias} "));
                let alias = normalize(alias);
                if alias.len() < 2
                    || alias.len() < best_length
                    || !question.contains(&format!(" {alias} "))
                {
                    continue;
                }
                if alias.len() > best_length {
                    candidates.clear();
                    supported.clear();
                    best_length = alias.len();
                }
                candidates.insert(id.clone());
                if case_supported {
                    supported.insert(id.clone());
                }
            }
        }
        // A failed casing check must not resolve an existing ambiguity or
        // expose a shorter alias. It only withdraws the original link.
        if candidates.len() == 1 {
            candidates
                .into_iter()
                .next()
                .filter(|id| supported.contains(id))
        } else {
            None
        }
    }

    /// Uses the same alias linker and learned relation proposal as `answer`.
    /// A set is local snapshot evidence, not a complete or logically certain list.
    pub fn answer_set(&self, model: &IntentModel, question: &str) -> AnswerSet {
        let mut result = AnswerSet {
            answer: Answer {
                entity: None,
                relation: None,
                answers: BTreeSet::new(),
                refusal: None,
            },
            total_snapshot_matches: 0,
            returned: 0,
            truncated: false,
        };
        let Some(entity) = self.link(question) else {
            result.answer.refusal = Some("no unique entity alias in this snapshot".into());
            return result;
        };
        result.answer.entity = Some(entity.clone());
        let prediction = model.predict(question);
        result.answer.relation = Some(prediction.label.clone());
        if !prediction.accepted {
            result.answer.refusal = Some("uncertain relation".into());
            return result;
        }
        let label = prediction.label.as_str();
        if label.len() < 2
            || !label
                .as_bytes()
                .get(1..)
                .is_some_and(|suffix| suffix.iter().all(u8::is_ascii_digit))
            || !label
                .as_bytes()
                .first()
                .is_some_and(|b| matches!(*b, b'P' | b'R'))
        {
            result.answer.refusal = Some("unsupported relation".into());
            return result;
        }
        if label.starts_with('P') {
            if let Some(values) = self
                .entities
                .get(&entity)
                .and_then(|e| e.claims.get(&PropertyId(label.into())))
            {
                // Deduplicate borrowed IDs; clone only the bounded display set.
                let unique: BTreeSet<_> = values.iter().collect();
                result.total_snapshot_matches = unique.len();
                result
                    .answer
                    .answers
                    .extend(unique.into_iter().take(SET_DISPLAY_LIMIT).cloned());
            }
        } else {
            let property = PropertyId(format!("P{}", label.get(1..).unwrap_or_default()));
            // BTreeMap iteration is sorted and subjects are already unique.
            for (subject, row) in &self.entities {
                if row
                    .claims
                    .get(&property)
                    .is_some_and(|objects| objects.contains(&entity))
                {
                    result.total_snapshot_matches += 1;
                    if result.answer.answers.len() < SET_DISPLAY_LIMIT {
                        result.answer.answers.insert(subject.clone());
                    }
                }
            }
        }
        result.returned = result.answer.answers.len();
        result.truncated = result.total_snapshot_matches > result.returned;
        if result.returned == 0 {
            result.answer.refusal = Some("claim missing from this finite snapshot".into());
        }
        result
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

#[cfg(test)]
mod tests {
    use super::*;

    fn store(rows: &[(&str, &[&str])]) -> Store {
        Store {
            entities: rows
                .iter()
                .map(|(id, aliases)| {
                    (
                        EntityId((*id).into()),
                        Entity {
                            aliases: aliases.iter().map(|s| (*s).into()).collect(),
                            claims: BTreeMap::new(),
                        },
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn uppercase_alias_does_not_match_an_ordinary_word() {
        let store = store(&[("Q1", &["IN", "Example Institute"])]);
        assert_eq!(store.link("Which language is spoken in this film?"), None);
        assert_eq!(store.link("In which city is the film set?"), None);
        assert_eq!(store.link("Where is (IN)?"), Some(EntityId("Q1".into())));
        assert_eq!(store.link("Where is INNER?"), None);
    }

    #[test]
    fn descriptive_aliases_still_ignore_case() {
        let store = store(&[("Q1", &["IN", "Example Institute"])]);
        assert_eq!(
            store.link("Where is example institute?"),
            Some(EntityId("Q1".into()))
        );
    }

    #[test]
    fn explicit_lowercase_alias_remains_valid() {
        let store = store(&[("Q1", &["ACME", "Acme"])]);
        assert_eq!(store.link("Who founded acme?"), Some(EntityId("Q1".into())));
    }

    #[test]
    fn longest_alias_and_ambiguity_are_preserved() {
        let store = store(&[
            ("Q1", &["IN", "Example Institute"]),
            ("Q2", &["Example Institute"]),
            ("Q3", &["Example Institute of Music"]),
        ]);
        assert_eq!(store.link("Where is example institute?"), None);
        assert_eq!(
            store.link("Where is example institute of music?"),
            Some(EntityId("Q3".into()))
        );
    }

    #[test]
    fn rejected_acronym_does_not_resolve_ambiguity_or_expose_shorter_alias() {
        let store = store(&[("Q1", &["IN", "ACME"]), ("Q2", &["Go"])]);
        assert_eq!(store.link("What language is go in?"), None);
        assert_eq!(store.link("Does acme go abroad?"), None);
    }
    #[test]
    fn bounded_sets_preserve_legacy_refusal_and_relation_guards() {
        let mut store = store(&[("Q1", &["Atlas"])]);
        store
            .entities
            .get_mut(&EntityId("Q1".into()))
            .unwrap()
            .claims
            .insert(
                PropertyId("P1".into()),
                (10..30)
                    .rev()
                    .chain(10..20)
                    .map(|n| EntityId(format!("Q{n}")))
                    .collect(),
            );
        let mut model = IntentModel::train(
            &[
                glassmind_everyday::Example {
                    text: "Who created Atlas?".into(),
                    label: "P1".into(),
                },
                glassmind_everyday::Example {
                    text: "Where located Example?".into(),
                    label: "R2".into(),
                },
            ],
            glassmind_everyday::TrainOptions {
                epochs: 5,
                char_weight: 0.0,
                min_df: 1,
            },
        )
        .unwrap();
        model.policy.min_score = 0.0;
        model.policy.min_margin = 0.0;
        let question = "Who created Atlas?";
        assert_eq!(model.predict(question).label, "P1");
        let legacy = store.answer(&model, question);
        assert!(legacy.answers.is_empty());
        assert_eq!(
            legacy.refusal.as_deref(),
            Some("this snapshot has multiple answers; narrow the question")
        );
        let set = store.answer_set(&model, question);
        assert_eq!(set.total_snapshot_matches, 20);
        assert_eq!(set.returned, 10);
        assert!(set.truncated);
        assert_eq!(
            set.answer.answers,
            (10..20).map(|n| EntityId(format!("Q{n}"))).collect()
        );
        model.labels[0] = "unsupported".into();
        assert_eq!(
            store.answer_set(&model, question).answer.refusal.as_deref(),
            Some("unsupported relation")
        );
        model.policy.min_score = f32::MAX;
        assert_eq!(
            store.answer_set(&model, question).answer.refusal.as_deref(),
            Some("uncertain relation")
        );
    }
}

pub mod temporal;

pub mod timed_matching;
pub use timed_matching::{TemporalMatchRefusalReason, TemporalMatchResult, TemporalSubjectMatch};
