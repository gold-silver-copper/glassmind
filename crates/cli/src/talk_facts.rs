//! Opt-in immutable facts bundles. Display labels never enter the linker.
use glassmind_everyday::IntentModel;
use glassmind_facts::{Answer, Store};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Deserialize)]
struct Manifest {
    version: u32,
    index_sha256: String,
    relation_model_sha256: String,
    source: String,
    license: String,
    scope: String,
}

pub(crate) struct Facts {
    store: Store,
    model: IntentModel,
    labels: BTreeMap<String, String>,
    manifest: Manifest,
}

#[derive(Deserialize)]
struct Labels {
    index_sha256: String,
    source: String,
    license: String,
    labels: BTreeMap<String, String>,
}

#[derive(Serialize)]
pub(crate) struct FactReply {
    pub kind: &'static str,
    pub text: String,
    #[serde(flatten)]
    pub answer: Answer,
    pub entities: Vec<DisplayEntity>,
    pub provenance: Provenance,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub set: Option<SetMetadata>,
}

#[derive(Serialize)]
pub(crate) struct SetMetadata {
    total_snapshot_matches: usize,
    returned: usize,
    truncated: bool,
}

/// Reserve prefix-like commands so malformed requests cannot mutate a story or
/// fall through to a learned conversational intent. No context binding occurs.
pub(crate) fn set_request(line: &str) -> Option<Result<&str, &'static str>> {
    let line = line.trim();
    let lower = line.to_ascii_lowercase();
    if !lower.starts_with("list answers") && !lower.starts_with("show answers") {
        return None;
    }
    let invalid = "Use List answers to: followed by one complete factual question ending in ?";
    let prefix = ["list answers to: ", "show answers to: "]
        .into_iter()
        .find(|p| lower.starts_with(p));
    let Some(prefix) = prefix else {
        return Some(Err(invalid));
    };
    let question = line.get(prefix.len()..).unwrap_or_default().trim();
    let Some(body) = question.strip_suffix('?') else {
        return Some(Err(invalid));
    };
    let mut words = body.split_whitespace();
    let first = words.next().unwrap_or_default();
    if words.count() < 2
        || body
            .chars()
            .any(|c| c.is_control() || matches!(c, '?' | '!' | ';' | ':'))
        || !matches!(
            first.to_ascii_lowercase().as_str(),
            "who" | "whom" | "whose" | "what" | "which" | "where" | "when" | "why" | "how"
        )
    {
        return Some(Err(invalid));
    }
    Some(Ok(question))
}

#[derive(Serialize)]
pub(crate) struct DisplayEntity {
    id: String,
    label: Option<String>,
    url: String,
}

#[derive(Serialize)]
pub(crate) struct Provenance {
    source: String,
    license: String,
    index_sha256: String,
    relation_model_sha256: String,
    relation_selection: &'static str,
    scope: String,
    relation_url: Option<String>,
    relation_direction: Option<&'static str>,
}

// The bundle declares the compatible pair; the binary contains no dataset IDs,
// asset fingerprints, fixed entity counts or model-specific compatibility rules.
fn verified(path: &Path, expected_sha256: &str) -> Result<Vec<u8>, String> {
    if expected_sha256.len() != 64
        || !expected_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("manifest.json: expected a lowercase SHA-256 digest".into());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != expected_sha256 {
        return Err(format!("{}: does not match manifest.json", path.display()));
    }
    Ok(bytes)
}

impl Facts {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let manifest: Manifest = serde_json::from_slice(
            &std::fs::read(dir.join("manifest.json")).map_err(|e| format!("manifest.json: {e}"))?,
        )
        .map_err(|e| format!("manifest.json: {e}"))?;
        if manifest.version != 1
            || [&manifest.source, &manifest.license, &manifest.scope]
                .iter()
                .any(|s| s.trim().is_empty() || s.chars().any(char::is_control))
        {
            return Err("manifest.json: unsupported version or missing provenance".into());
        }
        let store: Store =
            serde_json::from_slice(&verified(&dir.join("index.json"), &manifest.index_sha256)?)
                .map_err(|e| e.to_string())?;
        let model: IntentModel = serde_json::from_slice(&verified(
            &dir.join("relation.json"),
            &manifest.relation_model_sha256,
        )?)
        .map_err(|e| e.to_string())?;
        store.validate()?;
        model.validate().map_err(|e| e.to_string())?;
        if !model.labels.iter().all(|label| {
            label
                .as_bytes()
                .first()
                .is_some_and(|b| matches!(*b, b'P' | b'R'))
                && label.len() > 1
                && label
                    .as_bytes()
                    .get(1..)
                    .is_some_and(|suffix| suffix.iter().all(u8::is_ascii_digit))
        }) {
            return Err("relation.json: expected Wikidata property relation labels".into());
        }
        let labels_path = dir.join("labels.json");
        let labels = if labels_path.exists() {
            let labels: Labels =
                serde_json::from_slice(&std::fs::read(&labels_path).map_err(|e| e.to_string())?)
                    .map_err(|e| format!("labels.json: {e}"))?;
            if labels.index_sha256 != manifest.index_sha256
                || labels.license.trim().is_empty()
                || labels.source.is_empty()
            {
                return Err("labels.json: snapshot or provenance mismatch".into());
            }
            let ids: std::collections::BTreeSet<_> = store
                .entities
                .keys()
                .chain(
                    store
                        .entities
                        .values()
                        .flat_map(|e| e.claims.values().flatten()),
                )
                .map(|id| id.0.as_str())
                .collect();
            if labels.labels.iter().any(|(id, label)| {
                !ids.contains(id.as_str())
                    || label.is_empty()
                    || label.chars().any(char::is_control)
            }) {
                return Err("labels.json: labels must describe existing snapshot IDs only".into());
            }
            labels.labels
        } else {
            BTreeMap::new()
        };
        Ok(Self {
            store,
            model,
            labels,
            manifest,
        })
    }

    pub fn respond(&self, question: &str) -> FactReply {
        let answer = self.store.answer(&self.model, question);
        self.present(answer)
    }

    pub fn respond_set(&self, request: Result<&str, &str>) -> FactReply {
        let (answer, metadata) = match request {
            Ok(question) => {
                let set = self.store.answer_set(&self.model, question);
                (
                    set.answer,
                    SetMetadata {
                        total_snapshot_matches: set.total_snapshot_matches,
                        returned: set.returned,
                        truncated: set.truncated,
                    },
                )
            }
            Err(reason) => (
                Answer {
                    entity: None,
                    relation: None,
                    answers: Default::default(),
                    refusal: Some(reason.into()),
                },
                SetMetadata {
                    total_snapshot_matches: 0,
                    returned: 0,
                    truncated: false,
                },
            ),
        };
        let mut reply = self.present(answer);
        reply.text = if let Some(reason) = &reply.answer.refusal {
            format!("I couldn't list answers from the local Wikidata snapshot: {reason}.")
        } else {
            reply.kind = "fact_set";
            let mut text = format!(
                "The local Wikidata snapshot suggests these matches (showing {} of {}):",
                metadata.returned, metadata.total_snapshot_matches
            );
            for id in &reply.answer.answers {
                let label = self
                    .labels
                    .get(&id.0)
                    .map(String::as_str)
                    .unwrap_or("English label unavailable");
                text.push_str(&format!(
                    "\n- {label} ({}): https://www.wikidata.org/wiki/{}",
                    id.0, id.0
                ));
            }
            if let Some(id) = &reply.answer.entity {
                text.push_str(&format!(
                    "\nMatched entity: https://www.wikidata.org/wiki/{}",
                    id.0
                ));
            }
            if let Some(url) = &reply.provenance.relation_url {
                text.push_str(&format!(
                    "\nProperty: {url} ({})",
                    reply.provenance.relation_direction.unwrap_or("unknown")
                ));
            }
            text.push_str("\nThis is a limited local snapshot, not a complete world list. My learned interpretation of the question may be mistaken.");
            text
        };
        reply.set = Some(metadata);
        reply
    }

    fn present(&self, answer: Answer) -> FactReply {
        let entities: Vec<_> = answer
            .entity
            .iter()
            .chain(answer.answers.iter())
            .map(|id| DisplayEntity {
                id: id.0.clone(),
                label: self.labels.get(&id.0).cloned(),
                url: format!("https://www.wikidata.org/wiki/{}", id.0),
            })
            .collect();
        let describe = |id: &str| match self.labels.get(id) {
            Some(label) => format!("{label} ({id})"),
            None => format!("{id} (English label unavailable)"),
        };
        let text = if answer.refusal.is_some() {
            "I couldn't find a unique answer in the local Wikidata snapshot.".to_owned()
        } else {
            let values = answer
                .answers
                .iter()
                .map(|id| describe(&id.0))
                .collect::<Vec<_>>()
                .join(", ");
            let source = answer
                .entity
                .as_ref()
                .map(|id| format!("\nSource: https://www.wikidata.org/wiki/{}", id.0))
                .unwrap_or_default();
            format!("The local Wikidata snapshot suggests {values}.{source}")
        };
        let relation_url = answer.relation.as_ref().map(|r| {
            format!(
                "https://www.wikidata.org/wiki/Property:P{}",
                r.get(1..).unwrap_or_default()
            )
        });
        let relation_direction = answer.relation.as_ref().map(|r| {
            if r.starts_with('R') {
                "inverse"
            } else {
                "forward"
            }
        });
        FactReply {
            kind: if answer.refusal.is_some() {
                "refusal"
            } else {
                "fact_answer"
            },
            text,
            answer,
            entities,
            set: None,
            provenance: Provenance {
                source: self.manifest.source.clone(),
                license: self.manifest.license.clone(),
                index_sha256: self.manifest.index_sha256.clone(),
                relation_model_sha256: self.manifest.relation_model_sha256.clone(),
                relation_selection: "learned proposal, not logical certainty",
                scope: self.manifest.scope.clone(),
                relation_url,
                relation_direction,
            },
        }
    }
}

pub(crate) fn is_question(line: &str) -> bool {
    let lower = line.trim().to_lowercase();
    lower.ends_with('?')
        || matches!(
            lower.split_whitespace().next().unwrap_or_default(),
            "what"
                | "which"
                | "who"
                | "whom"
                | "whose"
                | "where"
                | "when"
                | "why"
                | "how"
                | "is"
                | "are"
                | "was"
                | "were"
                | "do"
                | "does"
                | "did"
                | "can"
                | "could"
                | "would"
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use glassmind_everyday::{TrainOptions, adapter::Example};
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn write(path: &Path, value: &impl Serialize) -> String {
        let bytes = serde_json::to_vec(value).expect("test JSON");
        std::fs::write(path, &bytes).expect("test file");
        format!("{:x}", Sha256::digest(&bytes))
    }
    fn bundle(alias: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "facts-bundle-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("test directory");
        let index = serde_json::json!({"entities": {
            "Q1": {"aliases": [alias], "claims": {"P1": ["Q2"]}}
        }});
        let rows = [
            Example {
                text: "where is example".into(),
                label: "P1".into(),
            },
            Example {
                text: "who wrote example".into(),
                label: "P2".into(),
            },
        ];
        let model = IntentModel::train(
            &rows,
            TrainOptions {
                epochs: 1,
                char_weight: 0.0,
                min_df: 1,
            },
        )
        .expect("small relation model");
        let index_sha256 = write(&path.join("index.json"), &index);
        let relation_model_sha256 = write(&path.join("relation.json"), &model);
        write(
            &path.join("manifest.json"),
            &serde_json::json!({
                "version":1,"index_sha256":index_sha256,
                "relation_model_sha256":relation_model_sha256,
                "source":"independent local test fixture","license":"MIT OR Apache-2.0",
                "scope":"one synthetic entity"
            }),
        );
        path
    }

    #[test]
    fn accepts_distinct_bundles_and_rejects_swapped_assets() {
        let a = bundle("Example Alpha");
        let b = bundle("Example Beta");
        let first = Facts::load(&a).expect("first independently configured bundle");
        let second = Facts::load(&b).expect("second independently configured bundle");
        assert_ne!(first.manifest.index_sha256, second.manifest.index_sha256);
        assert!(first.store.link("where is Example Alpha").is_some());
        assert!(second.store.link("where is Example Beta").is_some());
        std::fs::copy(b.join("index.json"), a.join("index.json")).expect("swap test index");
        assert!(Facts::load(&a).is_err());
        std::fs::remove_dir_all(a).expect("cleanup");
        std::fs::remove_dir_all(b).expect("cleanup");
    }

    #[test]
    fn labels_remain_display_only_and_must_match_the_index() {
        let path = bundle("Example Alpha");
        let before = Facts::load(&path).expect("bundle");
        let question = "where is Example Alpha";
        let answer = serde_json::to_value(before.respond(question).answer).expect("answer");
        let mut labels = serde_json::json!({
            "index_sha256":before.manifest.index_sha256,
            "source":"independent fixture","license":"MIT OR Apache-2.0",
            "labels":{"Q1":"Display Name", "Q2":"Display Result"}
        });
        write(&path.join("labels.json"), &labels);
        let after = Facts::load(&path).expect("labelled bundle");
        assert_eq!(
            serde_json::to_value(after.respond(question).answer).expect("answer"),
            answer
        );
        assert!(after.store.link("where is Display Name").is_none());
        labels["labels"]["Q99"] = serde_json::json!("Unknown ID");
        write(&path.join("labels.json"), &labels);
        assert!(Facts::load(&path).is_err());
        labels["labels"]
            .as_object_mut()
            .expect("labels object")
            .remove("Q99");
        labels["index_sha256"] = serde_json::json!("0".repeat(64));
        write(&path.join("labels.json"), &labels);
        assert!(Facts::load(&path).is_err());
        std::fs::remove_dir_all(path).expect("cleanup");
    }
    #[test]
    fn set_prefix_is_fully_consumed_and_does_not_bind_followups() {
        assert_eq!(
            set_request(" SHOW answers to: Who created Étoile? "),
            Some(Ok("Who created Étoile?"))
        );
        for line in [
            "List answers to:",
            "List answers to Who created Atlas?",
            "Show answers to: Who created Atlas? then erase",
            "List answers to: And those?",
            "List answers to: Who created Atlas? Who else?",
        ] {
            assert!(set_request(line).unwrap().is_err(), "{line}");
        }
        assert!(set_request("Who created Atlas?").is_none());
        assert!(set_request("My name is List answers").is_none());
    }
}
