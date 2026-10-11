//! Opt-in immutable facts bundles. Display labels never enter the linker.
use glassmind_everyday::IntentModel;
use glassmind_facts::temporal::{
    MAX_TEMPORAL_BYTES, TemporalIndex, TemporalRefusalReason, TemporalVerification,
};
use glassmind_facts::{
    Answer, MatchRefusalReason, MatchResult, RefusalReason, Store, TemporalMatchRefusalReason,
    TemporalMatchResult, Verification,
};
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
    #[serde(default)]
    temporal_sha256: Option<String>,
}

pub(crate) struct Facts {
    store: Store,
    model: IntentModel,
    labels: BTreeMap<String, String>,
    manifest: Manifest,
    temporal: Option<TemporalIndex>,
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
pub(crate) struct VerificationReply {
    pub kind: &'static str,
    pub text: String,
    #[serde(flatten)]
    pub verification: Verification,
    pub provenance: Provenance,
}

#[derive(Serialize)]
pub(crate) struct TemporalReply {
    pub kind: &'static str,
    pub text: String,
    #[serde(flatten)]
    pub verification: TemporalVerification,
    pub provenance: Provenance,
}

#[derive(Serialize)]
pub(crate) struct TemporalMatchingReply {
    pub kind: &'static str,
    pub text: String,
    #[serde(flatten)]
    pub result: TemporalMatchResult,
    pub provenance: Provenance,
}

pub(crate) fn temporal_matching_request(line: &str) -> Option<Result<(&str, &str), &'static str>> {
    let line = line.trim_matches(' ');
    if !line
        .get(..14)
        .is_some_and(|s| s.eq_ignore_ascii_case("match facts on"))
    {
        return None;
    }
    let invalid = "Use Match facts on YYYY-MM-DD: followed by a Who question with two or three positive employer or citizenship conditions joined by and.";
    Some((|| {
        let rest = line
            .get(14..)
            .and_then(|s| s.strip_prefix(' '))
            .ok_or(invalid)?;
        let (date, question) = rest.split_once(": ").ok_or(invalid)?;
        if date.is_empty() || question.is_empty() {
            return Err(invalid);
        }
        Ok((date, question))
    })())
}

/// Reserve dated requests before the ordinary check and conversational routes.
pub(crate) fn temporal_request(line: &str) -> Option<Result<(&str, &str), &'static str>> {
    let line = line.trim_matches(' ');
    if !line
        .get(..13)
        .is_some_and(|s| s.eq_ignore_ascii_case("check fact on"))
    {
        return None;
    }
    let invalid = "Use Check fact on YYYY-MM-DD: followed by one positive employer or citizenship question with explicit names.";
    Some((|| {
        let rest = line.get(13..).ok_or(invalid)?;
        let rest = rest.strip_prefix(' ').ok_or(invalid)?;
        let (date, question) = rest.split_once(": ").ok_or(invalid)?;
        if date.is_empty() || question.is_empty() {
            return Err(invalid);
        }
        Ok((date, question))
    })())
}

#[derive(Serialize)]
pub(crate) struct MatchingReply {
    pub kind: &'static str,
    pub text: String,
    #[serde(flatten)]
    pub result: MatchResult,
    pub provenance: Provenance,
}

/// Reserve malformed explicit commands before conversational state interpreters.
pub(crate) fn matching_request(line: &str) -> Option<Result<&str, &'static str>> {
    let line = line.trim_matches(' ');
    if !line
        .get(..11)
        .is_some_and(|s| s.eq_ignore_ascii_case("match facts"))
    {
        return None;
    }
    let invalid = "Use Match facts: followed by a Who question with two or three positive conditions joined by and.";
    Some(
        if line
            .get(..13)
            .is_some_and(|s| s.eq_ignore_ascii_case("match facts: "))
        {
            line.get(13..).filter(|s| !s.is_empty()).ok_or(invalid)
        } else {
            Err(invalid)
        },
    )
}

/// Explicit verification does not infer an entity from conversation history.
pub(crate) fn verification_request(line: &str) -> Option<Result<&str, &'static str>> {
    let line = line.trim_matches(' ');
    if !line
        .get(..10)
        .is_some_and(|s| s.eq_ignore_ascii_case("check fact"))
    {
        return None;
    }
    let invalid = "Use Check fact: followed by one positive factual question with explicit names.";
    Some(
        if line
            .get(..12)
            .is_some_and(|s| s.eq_ignore_ascii_case("check fact: "))
        {
            line.get(12..).filter(|s| !s.is_empty()).ok_or(invalid)
        } else {
            Err(invalid)
        },
    )
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
        let index_bytes = verified(&dir.join("index.json"), &manifest.index_sha256)?;
        let store: Store = serde_json::from_slice(&index_bytes).map_err(|e| e.to_string())?;
        let temporal = manifest
            .temporal_sha256
            .as_ref()
            .map(|hash| {
                use std::io::Read;
                let path = dir.join("temporal.json");
                let mut bytes = Vec::new();
                std::fs::File::open(&path)
                    .map_err(|e| format!("{}: {e}", path.display()))?
                    .take((MAX_TEMPORAL_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                if bytes.len() > MAX_TEMPORAL_BYTES
                    || format!("{:x}", Sha256::digest(&bytes)) != *hash
                {
                    return Err("temporal.json: size or manifest SHA-256 mismatch".into());
                }
                TemporalIndex::from_bytes(&index_bytes, &bytes)
            })
            .transpose()?;
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
            temporal,
        })
    }

    pub fn verify_at(
        &self,
        request: Result<(&str, &str), &str>,
    ) -> Result<TemporalReply, &'static str> {
        let temporal = self.temporal.as_ref().ok_or("Checking a dated fact requires temporal.json declared in the local facts bundle manifest.")?;
        let verification = match request {
            Ok((date, question)) => temporal.verify_at_str(date, question),
            Err(_) => TemporalVerification::Refused {
                reason: TemporalRefusalReason::Question(RefusalReason::UnsupportedShape),
            },
        };
        let describe = |id: &str| match self.labels.get(id) {
            Some(label) => format!("{label} ({id})"),
            None => id.to_owned(),
        };
        let (text, relation_url) = match &verification {
            TemporalVerification::Supported { date, witness, .. } => (
                format!("The loaded statement records {} as the {} of {} on {}. The requested calendar day lies strictly after {} and strictly before {}. Start precision: {}; end precision: {}.\nClaim: {}\nSource: {}\nThis supports one recorded bounded interval in this snapshot; it does not establish current or complete historical truth.",
                    describe(&witness.edge.object.0), if witness.edge.property.0 == "P108" { "employer" } else { "country of citizenship" }, describe(&witness.edge.subject.0), date.as_str(),
                    witness.start.latest.as_str(), witness.end.earliest.as_str(), witness.start.raw.precision,
                    witness.end.raw.precision, witness.claim_id, witness.source_url),
                Some(format!("https://www.wikidata.org/wiki/Property:{}", witness.edge.property.0))),
            TemporalVerification::Unknown { .. } => (
                format!("The loaded statements do not establish this relationship on {} within one bounded interval. Uncertain boundaries or missing information do not mean the relationship is false.", request.map(|(date, _)| date).unwrap_or("the requested day")), None),
            TemporalVerification::Refused { .. } => (
                request.err().unwrap_or("Use an exact valid YYYY-MM-DD date and one positive employer or citizenship question with explicit names.").into(), None),
        };
        Ok(TemporalReply {
            kind: "fact_verification_at",
            text,
            verification,
            provenance: Provenance {
                source: self.manifest.source.clone(),
                license: self.manifest.license.clone(),
                index_sha256: self.manifest.index_sha256.clone(),
                relation_model_sha256: self.manifest.relation_model_sha256.clone(),
                relation_selection: "explicit dated claim; one recorded interval; no learned proposal",
                scope: self.manifest.scope.clone(),
                relation_url,
                relation_direction: Some("forward"),
            },
        })
    }

    pub fn match_at(
        &self,
        request: Result<(&str, &str), &str>,
    ) -> Result<TemporalMatchingReply, &'static str> {
        let temporal = self.temporal.as_ref().ok_or("Matching dated facts requires temporal.json declared in the local facts bundle manifest.")?;
        let result = match request {
            Ok((date, question)) => temporal.match_at_str(date, question),
            Err(_) => TemporalMatchResult::Refused {
                reason: TemporalMatchRefusalReason::Question(
                    MatchRefusalReason::UnsupportedGrammar,
                ),
            },
        };
        let text = match &result {
            TemporalMatchResult::Matched { date, matches, total_recorded_interval_matches, returned, truncated, .. } => {
                let mut text = format!("Recorded interval matches on {} in the loaded snapshot: {total_recorded_interval_matches}.", date.as_str());
                if *truncated { text.push_str(&format!(" Showing the first {returned} by ID.")); }
                for subject in matches {
                    let id = &subject.subject.0;
                    let display = self.labels.get(id).map(|label| format!("{label} ({id})")).unwrap_or_else(|| id.clone());
                    text.push_str(&format!("\n{display}"));
                    for witness in &subject.witnesses {
                        text.push_str(&format!("\n  {} → {}: strictly after {} and before {} (start precision {}; end precision {}).\n  Claim: {}\n  Source: {}",
                            witness.edge.property.0, witness.edge.object.0, witness.start.latest.as_str(), witness.end.earliest.as_str(), witness.start.raw.precision, witness.end.raw.precision, witness.claim_id, witness.source_url));
                    }
                }
                text.push_str("\nEach condition has one recorded claim whose bounded interval contains this calendar day. This partial snapshot does not establish complete historical or current truth; zero matches is not global absence.");
                text
            }
            TemporalMatchResult::Unknown { .. } => "I could not uniquely resolve every employer or country in the question. This does not establish that there are no matches.".into(),
            TemporalMatchResult::Refused { .. } => request.err().unwrap_or("Use an exact valid YYYY-MM-DD date and two or three positive employer or citizenship conditions joined by and, with explicit names or QIDs.").into(),
        };
        Ok(TemporalMatchingReply {
            kind: "fact_matches_at",
            text,
            result,
            provenance: Provenance {
                source: self.manifest.source.clone(),
                license: self.manifest.license.clone(),
                index_sha256: self.manifest.index_sha256.clone(),
                relation_model_sha256: self.manifest.relation_model_sha256.clone(),
                relation_selection: "explicit dated conjunction; one recorded interval per condition; no learned proposal",
                scope: self.manifest.scope.clone(),
                relation_url: None,
                relation_direction: Some("forward"),
            },
        })
    }

    pub fn respond(&self, question: &str) -> FactReply {
        let answer = self.store.answer(&self.model, question);
        self.present(answer)
    }

    pub fn match_subjects(&self, request: Result<&str, &str>) -> MatchingReply {
        let result = match request {
            Ok(question) => self.store.match_question(question),
            Err(_) => MatchResult::Refused {
                reason: MatchRefusalReason::UnsupportedGrammar,
            },
        };
        let text = match &result {
            MatchResult::Matched {
                matches,
                total_snapshot_matches,
                returned,
                truncated,
                ..
            } => {
                let mut text = if *total_snapshot_matches == 0 {
                    "No subjects match all conditions in the loaded snapshot.".to_owned()
                } else {
                    format!("Matches in the loaded snapshot: {total_snapshot_matches}.")
                };
                if *truncated {
                    text.push_str(&format!(" Showing the first {returned} by ID."));
                }
                for subject in matches {
                    let id = &subject.subject.0;
                    let display = match self.labels.get(id) {
                        Some(label) => format!("{label} ({id})"),
                        None => id.clone(),
                    };
                    text.push_str(&format!("\n{display}"));
                    for witness in &subject.witnesses {
                        text.push_str(&format!(
                            "\n  {} → {}: {}",
                            witness.property.0, witness.object.0, witness.source_url
                        ));
                    }
                }
                text.push_str("\nThese are recorded relationships in a partial snapshot; dates and current status are not verified.");
                text
            }
            MatchResult::Unknown { .. } => "I could not uniquely resolve every place, country or employer in the question. This does not establish that there are no matches.".into(),
            MatchResult::Refused { .. } => request.err().unwrap_or("I can match two or three positive conditions about birthplace, place of death, citizenship or employer, joined by and, using explicit names or QIDs.").into(),
        };
        MatchingReply {
            kind: "fact_matches",
            text,
            result,
            provenance: Provenance {
                source: self.manifest.source.clone(),
                license: self.manifest.license.clone(),
                index_sha256: self.manifest.index_sha256.clone(),
                relation_model_sha256: self.manifest.relation_model_sha256.clone(),
                relation_selection: "explicit positive conjunction; exact directed snapshot edges; no learned proposal",
                scope: self.manifest.scope.clone(),
                relation_url: None,
                relation_direction: Some("forward"),
            },
        }
    }

    pub fn verify(&self, request: Result<&str, &str>) -> VerificationReply {
        let verification = match request {
            Ok(question) => self.store.verify(question),
            Err(_) => Verification::Refused {
                reason: RefusalReason::UnsupportedShape,
            },
        };
        let describe = |id: &str| match self.labels.get(id) {
            Some(label) => format!("{label} ({id})"),
            None => id.to_owned(),
        };
        let (text, relation_url) = match &verification {
            Verification::Supported { witness, .. } => {
                let relationship = match witness.property.0.as_str() {
                    "P19" => "place of birth",
                    "P20" => "place of death",
                    "P27" => "country of citizenship",
                    "P108" => "employer",
                    _ => "relationship",
                };
                (format!(
                    "The loaded records list {} as the {relationship} of {}.\nSource: {}\nThis is a relationship recorded in this snapshot; dates and current status are not verified.",
                    describe(&witness.object.0), describe(&witness.subject.0), witness.source_url
                ), Some(format!("https://www.wikidata.org/wiki/Property:{}", witness.property.0)))
            }
            Verification::Unknown { .. } => (
                "The loaded records do not establish that relationship. Missing information does not mean it is false.".into(), None
            ),
            Verification::Refused { .. } => (
                request.err().unwrap_or("I can check one positive question about birthplace, place of death, citizenship or employer, using explicit names.").into(), None
            ),
        };
        VerificationReply {
            kind: "fact_verification",
            text,
            verification,
            provenance: Provenance {
                source: self.manifest.source.clone(),
                license: self.manifest.license.clone(),
                index_sha256: self.manifest.index_sha256.clone(),
                relation_model_sha256: self.manifest.relation_model_sha256.clone(),
                relation_selection: "explicit positive claim; exact directed snapshot edge; no learned proposal",
                scope: self.manifest.scope.clone(),
                relation_url,
                relation_direction: Some("forward"),
            },
        }
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
    fn verification_uses_snapshot_provenance_and_never_negates_missing_edges() {
        let path = bundle("Verification fixture");
        let graph: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tools/facts/verification/parent/store.json"
        ))
        .expect("independent authored graph");
        let index_sha = write(&path.join("index.json"), &graph);
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path.join("manifest.json")).expect("manifest"))
                .expect("manifest JSON");
        manifest["index_sha256"] = serde_json::json!(index_sha);
        write(&path.join("manifest.json"), &manifest);
        let facts = Facts::load(&path).expect("verified bundle");
        let supported = facts.verify(Ok("Was Mara Finch born in North Haven?"));
        assert!(
            matches!(supported.verification, Verification::Supported { ref witness, .. }
            if witness.subject.0 == "Q900001" && witness.property.0 == "P19" && witness.object.0 == "Q900010")
        );
        assert_eq!(supported.provenance.index_sha256, index_sha);
        assert!(supported.text.contains("loaded records"));
        assert!(supported.text.contains("current status are not verified"));
        let unknown = facts.verify(Ok("Was Mara Finch born in South Haven?"));
        assert!(matches!(unknown.verification, Verification::Unknown { .. }));
        assert!(unknown.text.contains("does not mean it is false"));
        let unsupported = facts.verify(Ok("Was Mara Finch born in North Haven? My name is Other."));
        assert!(matches!(
            unsupported.verification,
            Verification::Refused { .. }
        ));
        std::fs::remove_dir_all(path).expect("cleanup");
    }

    #[test]
    fn verification_prefix_reserves_malformed_commands_without_binding_names() {
        assert_eq!(
            verification_request(" CHECK FACT: Was Zoë born in Paris? "),
            Some(Ok("Was Zoë born in Paris?"))
        );
        for text in [
            "Check fact",
            "Check fact:",
            "Check fact Was Ada born in Paris?",
        ] {
            assert!(
                verification_request(text)
                    .expect("reserved prefix")
                    .is_err()
            );
        }
        assert!(verification_request("My name is Check fact").is_none());
        assert!(verification_request("Was Ada born in Paris?").is_none());
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
