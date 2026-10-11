//! Explicit provisional request drafts, isolated from memory, stories and facts.
use glassmind_dialogue::draft::{DialogueBundle, DraftSession, DraftSnapshot};
use serde_json::Value;
use std::path::Path;

pub enum Request<'a> {
    User(&'a str),
    System(&'a str),
    Show,
    Reset,
    Malformed,
}

/// Reserve whole command prefixes even when their syntax is malformed.
/// Consume at most one separator space after ':'; preserve all payload bytes.
pub fn request(line: &str) -> Option<Request<'_>> {
    let line = line.trim_start_matches(|c: char| c.is_ascii_whitespace());
    for prefix in [
        "Draft request",
        "Service reply",
        "Show request",
        "Reset request",
    ] {
        if !line
            .get(..prefix.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(prefix))
        {
            continue;
        }
        let rest = line.get(prefix.len()..)?;
        if !rest.is_empty() && !rest.starts_with(|c: char| c == ':' || c.is_ascii_whitespace()) {
            continue;
        }
        return Some(match prefix {
            "Show request" if rest.trim().is_empty() => Request::Show,
            "Reset request" if rest.trim().is_empty() => Request::Reset,
            "Draft request" | "Service reply" => match rest.strip_prefix(':') {
                Some(payload) => {
                    let payload = payload.strip_prefix(' ').unwrap_or(payload);
                    if prefix == "Draft request" {
                        Request::User(payload)
                    } else {
                        Request::System(payload)
                    }
                }
                None => Request::Malformed,
            },
            _ => Request::Malformed,
        });
    }
    None
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", rename = "dialogue_draft")]
pub struct DraftReply {
    pub text: String,
    pub provisional: bool,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<DraftSnapshot>,
}

pub struct Drafts {
    bundle: Option<DialogueBundle>,
    live: Option<DraftSession>,
}

impl Drafts {
    pub fn load(directory: Option<&Path>) -> Result<Self, String> {
        Ok(Self {
            bundle: directory.map(DialogueBundle::load).transpose()?,
            live: None,
        })
    }
    pub fn clear(&mut self) {
        self.live = None;
    }
    pub fn respond(&mut self, history: &mut Option<Value>, request: Request<'_>) -> DraftReply {
        match self.apply(history, request) {
            Ok(reply) => reply,
            Err(error) => DraftReply {
                text: format!("Request draft unchanged: {error}"),
                provisional: true,
                status: "refused",
                snapshot: None,
            },
        }
    }
    fn apply(
        &mut self,
        history: &mut Option<Value>,
        request: Request<'_>,
    ) -> Result<DraftReply, String> {
        if matches!(request, Request::Reset) {
            self.clear();
            *history = None;
            return Ok(DraftReply {
                text: "Request draft cleared.".into(),
                provisional: true,
                status: "reset",
                snapshot: None,
            });
        }
        if matches!(request, Request::Malformed) {
            return Err(
                "use Draft request: TEXT, Service reply: TEXT, Show request, or Reset request."
                    .into(),
            );
        }
        let bundle = self
            .bundle
            .as_ref()
            .ok_or("load a local dialogue bundle with --dialogue-dir.")?;
        if self.live.is_none() {
            self.live = Some(match history {
                Some(history) => {
                    bundle.resume_json(&serde_json::to_vec(history).map_err(|e| e.to_string())?)?
                }
                None => bundle.new_session(),
            });
        }
        let live = self.live.as_mut().ok_or("draft session unavailable")?;
        let (status, changed) = match request {
            Request::User(text) => {
                live.append_user(text)?;
                ("updated", true)
            }
            Request::System(text) => {
                live.append_system(text)?;
                ("context_recorded", true)
            }
            Request::Show => ("shown", false),
            Request::Reset | Request::Malformed => return Err("invalid draft operation".into()),
        };
        if changed {
            *history = Some(
                serde_json::from_slice(&live.serialize_history()?).map_err(|e| e.to_string())?,
            );
        }
        let snapshot = live.snapshot();
        let text = render(&snapshot, status);
        Ok(DraftReply {
            text,
            provisional: true,
            status,
            snapshot: Some(snapshot),
        })
    }
}

fn render(snapshot: &DraftSnapshot, status: &str) -> String {
    let mut text = if status == "context_recorded" {
        "Service reply recorded as context for the provisional request draft.".to_owned()
    } else {
        "Provisional request draft (model interpretation):".to_owned()
    };
    let Some(draft) = &snapshot.draft else {
        text.push_str(" empty. Use Draft request: TEXT to begin.");
        return text;
    };
    if draft.active_services.is_empty() {
        text.push_str(" No supported service identified.");
    }
    for frame in &draft.frames {
        if !draft.active_services.contains(&frame.service) {
            continue;
        }
        text.push_str(&format!(
            "\n{} — {}",
            frame.service, frame.state.active_intent
        ));
        for (slot, values) in &frame.state.slot_values {
            text.push_str(&format!("\n  {slot}: {}", values.join(", ")));
        }
    }
    text
}
