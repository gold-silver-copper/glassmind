//! Conversation with typed memory, exact calculations and optional intent proposals.
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use glassmind_everyday::{Conversation, IntentModel, Reply};
use glassmind_quantities::Answer;

/// Preserve existing conversation JSON while exposing the calculation's typed result.
#[derive(serde::Serialize)]
#[serde(untagged)]
enum TalkReply {
    Conversation(Reply),
    Calculation(Calculation),
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
        }
    }
}

fn respond(memory: &mut Conversation, line: &str, model: Option<&IntentModel>) -> TalkReply {
    // Respond once so every utterance, including a calculation, participates in
    // turn provenance and last-message recall. Explicit memory always wins.
    let reply = memory.respond(line, model);
    if matches!(reply, Reply::Intent { .. } | Reply::Refusal { .. })
        && let Answer::Answer {
            value,
            unit,
            method,
        } = glassmind_quantities::solve(line)
    {
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
    TalkReply::Conversation(reply)
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    serde_json::from_reader(file).map_err(|e| e.to_string())
}

fn save(path: &Path, memory: &Conversation) -> Result<(), String> {
    let pending = path.with_extension(format!("{}.pending", std::process::id()));
    let bytes = serde_json::to_vec(memory).map_err(|e| e.to_string())?;
    std::fs::write(&pending, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(pending, path).map_err(|e| e.to_string())
}

pub fn run(session: Option<PathBuf>, model: Option<PathBuf>, json: bool) -> Result<(), String> {
    let model: Option<IntentModel> = model.as_deref().map(read).transpose()?;
    if let Some(model) = &model {
        model.validate().map_err(|e| e.to_string())?;
    }
    let mut memory = match session.as_deref().filter(|p| p.exists()) {
        Some(path) => read::<Conversation>(path)?,
        None => Conversation::default(),
    };
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
            memory = Conversation::default();
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
            let reply = respond(&mut memory, &line, model.as_ref());
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
