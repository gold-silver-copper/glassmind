//! Conversation with typed memory, story reasoning, calculations and intent proposals.
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use crate::talk_facts::{FactReply, Facts};
use crate::talk_stories::{Stories, StoryReply};
use glassmind_everyday::{Conversation, IntentModel, Reply};
use glassmind_quantities::Answer;

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Session {
    #[serde(flatten)]
    memory: Conversation,
    #[serde(default)]
    stories: Stories,
}

/// Preserve existing conversation JSON while exposing the calculation's typed result.
#[derive(serde::Serialize)]
#[serde(untagged)]
enum TalkReply {
    Conversation(Reply),
    Calculation(Calculation),
    Story(StoryReply),
    Fact(Box<FactReply>),
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", rename = "calculated")]
struct Calculation {
    text: String,
    value: String,
    unit: Option<String>,
    method: String,
}

impl TalkReply {
    fn text(&self) -> &str {
        match self {
            Self::Conversation(reply) => reply.text(),
            Self::Calculation(reply) => &reply.text,
            Self::Story(reply) => &reply.text,
            Self::Fact(reply) => &reply.text,
        }
    }
}

fn respond_with_facts(
    session: &mut Session,
    line: &str,
    model: Option<&IntentModel>,
    facts: Option<&Facts>,
) -> TalkReply {
    if let Some(request) = crate::talk_facts::set_request(line) {
        // Explicit external facts never enter the memory/story interpreters.
        session.memory.turn = session.memory.turn.saturating_add(1);
        session.memory.last_user_utterance = Some(line.to_owned());
        session.memory.last_slot = None;
        return match facts {
            Some(facts) => TalkReply::Fact(Box::new(facts.respond_set(request))),
            None => TalkReply::Conversation(Reply::Refusal {
                text: "Listing factual answers requires a local facts bundle (--facts-dir).".into(),
                proposal: None,
            }),
        };
    }
    let memory = &mut session.memory;
    // Persist one turn per utterance. Explicit memory always wins; learned
    // intent proposals run only after typed reasoning, from the prior state.
    let previous = model.map(|_| memory.clone());
    let calculation_focus = memory.last_slot.clone();
    let (reply, memory_authority) = memory.respond_with_authority(line, None);
    if memory_authority && matches!(reply, Reply::Refusal { .. }) {
        return TalkReply::Conversation(reply);
    }
    if matches!(reply, Reply::Intent { .. } | Reply::Refusal { .. })
        && let Answer::Answer {
            value,
            unit,
            method,
        } = glassmind_quantities::solve(line)
    {
        // Preserve the established typed-calculation continuation contract.
        // Generic intent/refusal turns still invalidate personal-memory focus.
        memory.last_slot = calculation_focus;
        let text = match &unit {
            Some(unit) => format!("{value} {unit}"),
            None => value.clone(),
        };
        return TalkReply::Calculation(Calculation {
            text,
            value,
            unit,
            method,
        });
    }
    let fact = if matches!(reply, Reply::Intent { .. } | Reply::Refusal { .. }) {
        facts.map(|facts| facts.respond(line))
    } else {
        None
    };
    if !session.stories.has_context()
        && fact
            .as_ref()
            .is_some_and(|r| !r.answer.answers.is_empty() || crate::talk_facts::is_question(line))
        && let Some(fact) = fact
    {
        session.memory.last_slot = None;
        return TalkReply::Fact(Box::new(fact));
    }
    let proposed = if matches!(reply, Reply::Refusal { .. }) {
        previous.map(|mut previous| {
            let proposal = previous.respond(line, model);
            (previous, proposal)
        })
    } else {
        None
    };
    // Without a story referent, a learned memory-read request can resolve
    // against typed personal state. Do not feed it to a story write parser.
    if !session.stories.has_context()
        && proposed.as_ref().is_some_and(|(_, reply)| {
            matches!(reply, Reply::Recalled { .. } | Reply::Missing { .. })
        })
        && {
            // Check typed story operations first without letting an unsupported
            // personal question invalidate otherwise absent story context.
            let mut probe = Stories::default();
            probe.restore().is_ok()
                && probe
                    .respond(line)
                    .is_none_or(|reply| reply.kind == "refusal")
        }
        && let Some((previous, proposal)) = proposed
    {
        session.memory = previous;
        return TalkReply::Conversation(proposal);
    }
    if (matches!(reply, Reply::Intent { .. } | Reply::Refusal { .. })
        || (matches!(reply, Reply::Missing { .. }) && session.stories.has_context()))
        && let Some(story) = session.stories.respond(line)
    {
        // A story turn changes the conversational focus. A later bare
        // correction must not overwrite an earlier name or preference.
        session.memory.last_slot = None;
        return TalkReply::Story(story);
    }
    if let Some(fact) = fact
        && crate::talk_facts::is_question(line)
    {
        session.memory.last_slot = None;
        return TalkReply::Fact(Box::new(fact));
    }
    if let Some((previous, proposal)) = proposed {
        session.memory = previous;
        return TalkReply::Conversation(proposal);
    }
    TalkReply::Conversation(reply)
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    serde_json::from_reader(file).map_err(|e| e.to_string())
}

fn save(path: &Path, memory: &Session) -> Result<(), String> {
    let pending = path.with_extension(format!("{}.pending", std::process::id()));
    let bytes = serde_json::to_vec(memory).map_err(|e| e.to_string())?;
    std::fs::write(&pending, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(pending, path).map_err(|e| e.to_string())
}

pub fn run(
    session: Option<PathBuf>,
    model: Option<PathBuf>,
    facts_dir: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let facts = facts_dir.as_deref().map(Facts::load).transpose()?;
    let model: Option<IntentModel> = model.as_deref().map(read).transpose()?;
    if let Some(model) = &model {
        model.validate().map_err(|e| e.to_string())?;
    }
    let mut memory = match session.as_deref().filter(|p| p.exists()) {
        Some(path) => read::<Session>(path)?,
        None => Session::default(),
    };
    memory.stories.restore()?;
    let stdin = std::io::stdin();
    let interactive = stdin.is_terminal();
    let mut output = std::io::stdout().lock();
    if interactive {
        let _ = writeln!(
            std::io::stderr(),
            "Talk to glassmind. /quit to leave; /reset to clear memory."
        );
    }
    let mut lines = stdin.lock().lines();
    loop {
        if interactive {
            write!(output, "> ")
                .and_then(|()| output.flush())
                .map_err(|e| e.to_string())?;
        }
        let Some(line) = lines.next() else { break };
        let line = line.map_err(|e| e.to_string())?;
        if line.trim() == "/quit" {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        if line.len() > 8192 {
            return Err("keep each message below 8192 bytes".into());
        }
        if line.trim() == "/reset" {
            memory = Session::default();
            memory.stories.restore()?;
            writeln!(
                output,
                "{}",
                if json {
                    "{\"kind\":\"reset\"}"
                } else {
                    "Memory cleared."
                }
            )
            .map_err(|e| e.to_string())?;
        } else {
            let reply = respond_with_facts(&mut memory, &line, model.as_ref(), facts.as_ref());
            if json {
                serde_json::to_writer(&mut output, &reply).map_err(|e| e.to_string())?;
                writeln!(output).map_err(|e| e.to_string())?;
            } else {
                writeln!(output, "{}", reply.text()).map_err(|e| e.to_string())?;
            }
        }
        if let Some(path) = &session {
            save(path, &memory)?;
        }
        output.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn respond(session: &mut Session, line: &str, model: Option<&IntentModel>) -> TalkReply {
        respond_with_facts(session, line, model, None)
    }

    #[test]
    fn unresolved_correction_invalidates_story_across_restart() {
        let mut session = Session::default();
        session.stories.restore().unwrap();
        let reply = respond(&mut session, "Actually, it's Alice", None);
        assert_eq!(serde_json::to_value(reply).unwrap()["kind"], "missing");
        respond(&mut session, "Mary is in the kitchen.", None);
        respond(&mut session, "Actually, it is the garden", None);
        let mut restored: Session =
            serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
        restored.stories.restore().unwrap();
        let reply = respond(&mut restored, "Where is Mary?", None);
        let value = serde_json::to_value(reply).unwrap();
        assert_eq!(value["kind"], "refusal");
        assert!(value["answer"].is_null());
    }

    #[test]
    fn story_focus_prevents_implicit_personal_memory_edits() {
        let mut session = Session::default();
        session.stories.restore().unwrap();
        respond(&mut session, "My name is Mira", None);
        respond(&mut session, "Mary is in the kitchen.", None);
        let mut restored: Session =
            serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
        restored.stories.restore().unwrap();
        respond(&mut restored, "Actually, it is the garden", None);
        assert_eq!(restored.memory.name.as_ref().unwrap().value, "Mira");
        assert_eq!(restored.memory.turn, 3);
        assert_eq!(
            serde_json::to_value(respond(&mut restored, "Where is Mary?", None)).unwrap()["kind"],
            "refusal"
        );
        // An explicit personal-memory question establishes that focus again.
        respond(&mut restored, "What is my name?", None);
        respond(&mut restored, "Actually, it is Alice", None);
        assert_eq!(restored.memory.name.as_ref().unwrap().value, "Alice");
    }
}
