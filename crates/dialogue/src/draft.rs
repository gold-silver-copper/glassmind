//! Provisional, resumable dialogue drafts. Persistence contains inputs, never predictions.
//!
//! Load a [`DialogueBundle`] once and share its immutable model across independent
//! [`DraftSession`] values. All output is a model proposal, not a verified fact or action.
use crate::{
    adapter::Schema,
    boundaries::BoundaryProposal,
    multi::{MultiModel, MultiSession},
    offers::OfferProposal,
    tracker::{InputTurn, Speaker},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
    sync::Arc,
};

pub const MAX_TURNS: usize = 128;
pub const MAX_UTTERANCE_BYTES: usize = 4096;
pub const MAX_TOTAL_UTTERANCE_BYTES: usize = 65536;
/// Enough for the maximum valid history even when every character is JSON-escaped.
pub const MAX_HISTORY_JSON_BYTES: usize = 512 * 1024;
const MODEL_BYTES: usize = 64 * 1024 * 1024;
const SCHEMA_BYTES: usize = 1024 * 1024;
const MANIFEST_BYTES: usize = 64 * 1024;

/// Provenance supplied with the bundle; hashes refer to exact, unmodified file bytes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    pub version: u32,
    pub model_sha256: String,
    pub schema_sha256: String,
    pub source: String,
    pub license: String,
    pub scope: String,
}

struct BundleData {
    manifest: BundleManifest,
    identity: String,
    model: MultiModel,
    schemas: Vec<Schema>,
}

/// Cheaply cloneable immutable bundle. No model or schema mutation is exposed.
#[derive(Clone)]
pub struct DialogueBundle(Arc<BundleData>);

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err(format!("{}: expected regular file", path.display()));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err(format!("{}: exceeds {limit} bytes", path.display()));
    }
    Ok(bytes)
}

impl DialogueBundle {
    /// Load fixed manifest.json, model.json, schema.json files without fallback.
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, String> {
        let directory = directory.as_ref();
        let manifest: BundleManifest = serde_json::from_slice(&read_bounded(
            &directory.join("manifest.json"),
            MANIFEST_BYTES,
        )?)
        .map_err(|e| format!("manifest: {e}"))?;
        if manifest.version != 1 {
            return Err("unsupported bundle manifest version".into());
        }
        if [&manifest.source, &manifest.license, &manifest.scope]
            .iter()
            .any(|s| s.trim().is_empty())
        {
            return Err("bundle source, license and scope must be nonempty".into());
        }
        let model_bytes = read_bounded(&directory.join("model.json"), MODEL_BYTES)?;
        let schema_bytes = read_bounded(&directory.join("schema.json"), SCHEMA_BYTES)?;
        if digest(&model_bytes) != manifest.model_sha256
            || digest(&schema_bytes) != manifest.schema_sha256
        {
            return Err("bundle asset SHA256 mismatch".into());
        }
        let model: MultiModel =
            serde_json::from_slice(&model_bytes).map_err(|e| format!("model: {e}"))?;
        let schemas: Vec<Schema> =
            serde_json::from_slice(&schema_bytes).map_err(|e| format!("schema: {e}"))?;
        model.validate(&schemas)?;
        // Domain separation and fixed-length, verified lowercase digests make this
        // independent of manifest JSON formatting and unambiguous across versions.
        let identity = digest(
            format!(
                "glassmind-dialogue-bundle\n{}\n{}\n{}\n",
                manifest.version, manifest.model_sha256, manifest.schema_sha256
            )
            .as_bytes(),
        );
        Ok(Self(Arc::new(BundleData {
            manifest,
            identity,
            model,
            schemas,
        })))
    }
    pub fn manifest(&self) -> &BundleManifest {
        &self.0.manifest
    }
    pub fn identity(&self) -> &str {
        &self.0.identity
    }
    pub fn new_session(&self) -> DraftSession {
        DraftSession {
            bundle: self.clone(),
            session: MultiSession::default(),
            turns: Vec::new(),
            utterance_bytes: 0,
            user_turns: 0,
        }
    }
    /// Validate the entire persisted envelope before replaying any turn.
    pub fn resume_json(&self, json: &[u8]) -> Result<DraftSession, String> {
        if json.len() > MAX_HISTORY_JSON_BYTES {
            return Err("history JSON exceeds byte limit".into());
        }
        let history: History = serde_json::from_slice(json).map_err(|e| format!("history: {e}"))?;
        if history.version != 1 {
            return Err("unsupported history version".into());
        }
        if history.bundle_identity != self.identity() {
            return Err("history bundle identity mismatch".into());
        }
        if history.turns.len() > MAX_TURNS {
            return Err("history exceeds turn limit".into());
        }
        let mut total = 0;
        for turn in &history.turns {
            validate_turn(turn)?;
            total += turn.utterance.len();
            if total > MAX_TOTAL_UTTERANCE_BYTES {
                return Err("history exceeds utterance byte limit".into());
            }
        }
        let mut session = self.new_session();
        for turn in history.turns {
            session.advance_validated(turn);
        }
        Ok(session)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct History {
    version: u32,
    bundle_identity: String,
    turns: Vec<InputTurn>,
}
#[derive(Serialize)]
struct HistoryRef<'a> {
    version: u32,
    bundle_identity: &'a str,
    turns: &'a [InputTurn],
}
fn validate_turn(turn: &InputTurn) -> Result<(), String> {
    if turn.utterance.is_empty()
        || turn.utterance.len() > MAX_UTTERANCE_BYTES
        || turn.utterance.trim().is_empty()
        || turn.utterance.contains('\0')
    {
        return Err("utterance must be 1..4096 UTF8 bytes, nonblank and NUL-free".into());
    }
    Ok(())
}

/// Exact legacy prediction JSON shape, including inactive frames and proposals.
#[derive(Debug, Serialize)]
pub struct Prediction {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub boundary_proposals: Vec<BoundaryProposal>,
    pub turn_index: usize,
    pub active_services: BTreeSet<String>,
    pub frames: Vec<DraftFrame>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub offer_proposals: Vec<OfferProposal>,
}
#[derive(Debug, Serialize)]
pub struct DraftFrame {
    pub service: String,
    pub state: DraftFrameState,
}
#[derive(Debug, Serialize)]
pub struct DraftFrameState {
    pub active_intent: String,
    pub slot_values: BTreeMap<String, Vec<String>>,
}
/// Always provisional. `draft` describes the last input (USER or SYSTEM);
/// `user_prediction()` is absent after SYSTEM and for an empty session.
#[derive(Debug, Serialize)]
pub struct DraftSnapshot {
    pub provisional: bool,
    pub provenance: BundleManifest,
    pub bundle_identity: String,
    pub total_turns: usize,
    pub user_turns: usize,
    pub last_speaker: Option<Speaker>,
    pub draft: Option<Prediction>,
}
impl DraftSnapshot {
    pub fn user_prediction(&self) -> Option<&Prediction> {
        if matches!(self.last_speaker, Some(Speaker::User)) {
            self.draft.as_ref()
        } else {
            None
        }
    }
}

/// Independent mutable history and reconstructed state; never serializes predictions.
pub struct DraftSession {
    bundle: DialogueBundle,
    session: MultiSession,
    turns: Vec<InputTurn>,
    utterance_bytes: usize,
    user_turns: usize,
}
impl DraftSession {
    pub fn append_user(&mut self, utterance: impl Into<String>) -> Result<(), String> {
        self.append(InputTurn {
            speaker: Speaker::User,
            utterance: utterance.into(),
        })
    }
    pub fn append_system(&mut self, utterance: impl Into<String>) -> Result<(), String> {
        self.append(InputTurn {
            speaker: Speaker::System,
            utterance: utterance.into(),
        })
    }
    /// Validate before mutation. Exact UTF8 bytes are retained without trimming.
    pub fn append(&mut self, turn: InputTurn) -> Result<(), String> {
        validate_turn(&turn)?;
        if self.turns.len() >= MAX_TURNS {
            return Err("history exceeds turn limit".into());
        }
        if self.utterance_bytes + turn.utterance.len() > MAX_TOTAL_UTTERANCE_BYTES {
            return Err("history exceeds utterance byte limit".into());
        }
        self.advance_validated(turn);
        Ok(())
    }
    fn advance_validated(&mut self, turn: InputTurn) {
        self.bundle
            .0
            .model
            .advance(&mut self.session, &turn, &self.bundle.0.schemas);
        self.utterance_bytes += turn.utterance.len();
        self.user_turns += usize::from(matches!(turn.speaker, Speaker::User));
        self.turns.push(turn);
    }
    pub fn reset(&mut self) {
        *self = self.bundle.new_session();
    }
    /// Replaces this session only after a complete successful validation and replay.
    pub fn restore_json(&mut self, json: &[u8]) -> Result<(), String> {
        let restored = self.bundle.resume_json(json)?;
        *self = restored;
        Ok(())
    }
    pub fn serialize_history(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(&HistoryRef {
            version: 1,
            bundle_identity: self.bundle.identity(),
            turns: &self.turns,
        })
        .map_err(|e| e.to_string())
    }
    pub fn history(&self) -> &[InputTurn] {
        &self.turns
    }
    /// Read-only: does not invoke the model or advance any counter.
    pub fn snapshot(&self) -> DraftSnapshot {
        let draft = self
            .turns
            .len()
            .checked_sub(1)
            .map(|turn_index| Prediction {
                boundary_proposals: self.session.boundary_proposals.clone(),
                turn_index,
                active_services: self.session.active_services.clone(),
                frames: self
                    .session
                    .states
                    .iter()
                    .map(|(service, state)| DraftFrame {
                        service: service.clone(),
                        state: DraftFrameState {
                            active_intent: state
                                .active_intent
                                .clone()
                                .unwrap_or_else(|| "NONE".into()),
                            slot_values: state
                                .slots
                                .iter()
                                .map(|(s, v)| (s.clone(), vec![v.text().into()]))
                                .collect(),
                        },
                    })
                    .collect(),
                offer_proposals: self.session.offer_proposals.clone(),
            });
        DraftSnapshot {
            provisional: true,
            provenance: self.bundle.manifest().clone(),
            bundle_identity: self.bundle.identity().into(),
            total_turns: self.turns.len(),
            user_turns: self.user_turns,
            last_speaker: self.turns.last().map(|t| match t.speaker {
                Speaker::User => Speaker::User,
                Speaker::System => Speaker::System,
            }),
            draft,
        }
    }
}

#[cfg(test)]
mod tests;
