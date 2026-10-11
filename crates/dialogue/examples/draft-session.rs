//! Native JSONL driver for independent parent integration checks.
use glassmind_dialogue::draft::{DialogueBundle, DraftSession};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{self, BufRead, Write},
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    session: String,
    command: Command,
}
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Command {
    User { utterance: String },
    System { utterance: String },
    Show,
    Reset,
    Save,
    Restore { history: Value },
}
fn execute(
    request: Request,
    bundle: &DialogueBundle,
    sessions: &mut BTreeMap<String, DraftSession>,
) -> Result<Value, String> {
    let s = sessions
        .entry(request.session)
        .or_insert_with(|| bundle.new_session());
    match request.command {
        Command::User { utterance } => s.append_user(utterance)?,
        Command::System { utterance } => s.append_system(utterance)?,
        Command::Show => (),
        Command::Reset => s.reset(),
        Command::Save => {
            return Ok(
                json!({"history":serde_json::from_slice::<Value>(&s.serialize_history()?).map_err(|e| e.to_string())?}),
            );
        }
        Command::Restore { history } => {
            s.restore_json(&serde_json::to_vec(&history).map_err(|e| e.to_string())?)?
        }
    }
    serde_json::to_value(s.snapshot()).map_err(|e| e.to_string())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: draft-session BUNDLE_DIRECTORY < commands.jsonl")?;
    let bundle = DialogueBundle::load(path)?;
    let mut sessions = BTreeMap::new();
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    loop {
        // Bound each example command too. This driver is an exercise tool, not a server.
        let mut line = Vec::new();
        let read = std::io::Read::take(&mut input, 1024 * 1024 + 1).read_until(b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        if read > 1024 * 1024 {
            return Err("example command exceeds 1MiB".into());
        }
        let result = serde_json::from_slice::<Request>(&line)
            .map_err(|e| e.to_string())
            .and_then(|r| execute(r, &bundle, &mut sessions));
        let value = match result {
            Ok(value) => json!({"ok":true,"result":value}),
            Err(error) => json!({"ok":false,"error":error}),
        };
        serde_json::to_writer(&mut output, &value)?;
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
