//! Conversation with typed memory, story reasoning, calculations and intent proposals.
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use crate::talk_facts::{
    FactReply, Facts, MatchingReply, TemporalMatchingReply, TemporalReply, VerificationReply,
};
use crate::talk_stories::{Stories, StoryPlan, StoryReply};
use glassmind_everyday::{Conversation, IntentModel, Reply};
use glassmind_quantities::Answer;

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Session {
    #[serde(flatten)]
    memory: Conversation,
    #[serde(default)]
    stories: Stories,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    request_draft: Option<serde_json::Value>,
}

/// Preserve existing conversation JSON while exposing the calculation's typed result.
#[derive(serde::Serialize)]
#[serde(untagged)]
enum TalkReply {
    Conversation(Reply),
    Calculation(Calculation),
    Story(StoryReply),
    Fact(Box<FactReply>),
    Verification(Box<VerificationReply>),
    Matching(Box<MatchingReply>),
    Temporal(Box<TemporalReply>),
    TemporalMatching(Box<TemporalMatchingReply>),
    Draft(Box<crate::talk_drafts::DraftReply>),
    Quantity(Box<crate::talk_quantities::QuantityReply>),
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
            Self::Verification(reply) => &reply.text,
            Self::Matching(reply) => &reply.text,
            Self::Temporal(reply) => &reply.text,
            Self::TemporalMatching(reply) => &reply.text,
            Self::Draft(reply) => &reply.text,
            Self::Quantity(reply) => &reply.text,
        }
    }
}

// Full utterance grammar, independent of classifier confidence. No entity or
// arbitrary trailing-text slot: appended assertions/actions must still be handled.
fn social_form(line: &str) -> bool {
    let lower = line.trim().trim_end_matches(['.', '!', '?']).to_lowercase();
    let words: Vec<_> = lower.split_whitespace().collect();
    match words.as_slice() {
        ["hi" | "hello" | "hey" | "hiya" | "goodbye" | "bye"]
        | ["hi" | "hello" | "hey", "there"]
        | ["hey", "hey"]
        | ["bye", "bye"]
        | ["see", "you" | "ya", "later" | "soon"]
        | ["talk", "to", "you", "later" | "soon"]
        | ["how", "are", "you"]
        | ["how", "are", "things", "going"]
        | ["how", "is", "it", "going"] => true,
        ["nice" | "good" | "great", "to", rest @ ..] => matches!(
            rest,
            ["see", "you"]
                | ["see", "you", "again"]
                | ["speak", "with", "you"]
                | ["speak", "with", "you", "again"]
                | ["talk", "to", "you"]
                | ["talk", "to", "you", "again"]
        ),
        ["thanks", rest @ ..] | ["thank", "you", rest @ ..] => matches!(
            rest,
            [] | ["kindly"]
                | ["a", "lot"]
                | ["so" | "very", "much"]
                | ["so", "very", "much"]
                | ["for", "helping"]
                | ["for", "helping", "me"]
        ),
        _ => false,
    }
}

fn appended_social_clause(line: &str, supported_story: bool, stories: &Stories) -> bool {
    // A complete salutation followed by an explicit clause boundary cannot be
    // used as a story actor (the broad story parser can accept coordinated names).
    if !matches!(
        line.split_whitespace()
            .next()
            .unwrap_or_default()
            .trim_end_matches([',', ';', '.', '!', '?']),
        "hi" | "hello"
            | "hey"
            | "hiya"
            | "goodbye"
            | "bye"
            | "see"
            | "talk"
            | "how"
            | "nice"
            | "good"
            | "great"
            | "thanks"
            | "thank"
    ) {
        return false;
    }
    line.char_indices().any(|(i, c)| {
        if !c.is_whitespace() && !matches!(c, ',' | ';' | '.' | '!' | '?') {
            return false;
        }
        let (prefix, rest) = line.split_at(i);
        let rest = rest.trim_start();
        social_form(prefix)
            && ((!supported_story && !rest.trim_end_matches(['.', '!', '?']).trim().is_empty())
                || (!rest
                    .trim_start_matches([',', ';', '.', '!', '?'])
                    .trim()
                    .is_empty()
                    && rest.starts_with([',', ';', '.', '!', '?']))
                || (rest.starts_with("and ") && !(supported_story && stories.has_actor(prefix)))
                || rest.starts_with("but ")
                || rest.starts_with("then "))
    })
}

fn respond_with_facts(
    session: &mut Session,
    line: &str,
    model: Option<&IntentModel>,
    facts: Option<&Facts>,
) -> TalkReply {
    if let Some(request) = crate::talk_quantities::request(line) {
        session.memory.turn = session.memory.turn.saturating_add(1);
        session.memory.last_user_utterance = Some(line.to_owned());
        session.memory.last_slot = None;
        return TalkReply::Quantity(Box::new(crate::talk_quantities::respond(request)));
    }
    if let Some(request) = crate::talk_facts::temporal_matching_request(line) {
        session.memory.turn = session.memory.turn.saturating_add(1);
        session.memory.last_user_utterance = Some(line.to_owned());
        session.memory.last_slot = None;
        let reply = facts
            .ok_or("Matching dated facts requires a local facts bundle (--facts-dir).")
            .and_then(|facts| facts.match_at(request));
        return match reply {
            Ok(reply) => TalkReply::TemporalMatching(Box::new(reply)),
            Err(text) => TalkReply::Conversation(Reply::Refusal {
                text: text.into(),
                proposal: None,
            }),
        };
    }
    if let Some(request) = crate::talk_facts::temporal_request(line) {
        session.memory.turn = session.memory.turn.saturating_add(1);
        session.memory.last_user_utterance = Some(line.to_owned());
        session.memory.last_slot = None;
        let reply = facts
            .ok_or("Checking dated facts requires a local facts bundle (--facts-dir).")
            .and_then(|facts| facts.verify_at(request));
        return match reply {
            Ok(reply) => TalkReply::Temporal(Box::new(reply)),
            Err(text) => TalkReply::Conversation(Reply::Refusal {
                text: text.into(),
                proposal: None,
            }),
        };
    }
    if let Some(request) = crate::talk_facts::matching_request(line) {
        session.memory.turn = session.memory.turn.saturating_add(1);
        session.memory.last_user_utterance = Some(line.to_owned());
        session.memory.last_slot = None;
        return match facts {
            Some(facts) => TalkReply::Matching(Box::new(facts.match_subjects(request))),
            None => TalkReply::Conversation(Reply::Refusal {
                text: "Matching factual relationships requires a local facts bundle (--facts-dir)."
                    .into(),
                proposal: None,
            }),
        };
    }
    if let Some(request) = crate::talk_facts::verification_request(line) {
        session.memory.turn = session.memory.turn.saturating_add(1);
        session.memory.last_user_utterance = Some(line.to_owned());
        session.memory.last_slot = None;
        return match facts {
            Some(facts) => TalkReply::Verification(Box::new(facts.verify(request))),
            None => TalkReply::Conversation(Reply::Refusal {
                text: "Checking factual relationships requires a local facts bundle (--facts-dir)."
                    .into(),
                proposal: None,
            }),
        };
    }
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
    let mut story_plan = session.stories.plan(line);
    if !memory_authority
        && appended_social_clause(
            &line.to_lowercase(),
            story_plan.supported(),
            &session.stories,
        )
    {
        story_plan = StoryPlan::Invalidate(
            "Please give one statement or question at a time. Use /reset and restate the facts.",
        );
    }
    if !memory_authority && !story_plan.supported() && social_form(line) {
        if let Some((previous, proposal)) = proposed {
            session.memory = previous;
            return TalkReply::Conversation(proposal);
        }
        return TalkReply::Conversation(reply);
    }
    if !session.stories.has_context()
        && proposed.as_ref().is_some_and(|(_, reply)| {
            matches!(reply, Reply::Recalled { .. } | Reply::Missing { .. })
        })
        && !story_plan.supported()
        && let Some((previous, proposal)) = proposed
    {
        session.memory = previous;
        return TalkReply::Conversation(proposal);
    }
    if (matches!(reply, Reply::Intent { .. } | Reply::Refusal { .. })
        || (matches!(reply, Reply::Missing { .. }) && session.stories.has_context()))
        && let Some(story) = session.stories.apply(story_plan)
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
        // Typed story ownership has already been resolved. An unrecognized
        // social shape alone does not invalidate an ordinary learned proposal.
        return TalkReply::Conversation(proposal);
    }
    TalkReply::Conversation(reply)
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    serde_json::from_reader(std::io::BufReader::new(file)).map_err(|e| e.to_string())
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
    dialogue_dir: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let facts = facts_dir.as_deref().map(Facts::load).transpose()?;
    let mut drafts = crate::talk_drafts::Drafts::load(dialogue_dir.as_deref())?;
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
            drafts.clear();
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
            let reply = if let Some(request) = crate::talk_drafts::request(&line) {
                memory.memory.turn = memory.memory.turn.saturating_add(1);
                memory.memory.last_user_utterance = Some(line.clone());
                memory.memory.last_slot = None;
                TalkReply::Draft(Box::new(drafts.respond(&mut memory.request_draft, request)))
            } else {
                respond_with_facts(&mut memory, &line, model.as_ref(), facts.as_ref())
            };
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
    fn social_turn_preserves_story_but_mixed_clause_invalidates() {
        let mut session = Session::default();
        session.stories.restore().unwrap();
        respond(&mut session, "Elin went to the kitchen.", None);
        let before = serde_json::to_value(&session.stories).unwrap();
        for line in ["hey there!", "see you soon", "thanks very much"] {
            respond(&mut session, line, None);
            assert_eq!(serde_json::to_value(&session.stories).unwrap(), before);
            assert_eq!(session.memory.last_user_utterance.as_deref(), Some(line));
        }
        assert_eq!(session.memory.turn, 4);
        for line in [
            "hey there and Elin moved to the garden",
            "how are you Elin teleported to the attic",
        ] {
            session.stories = Stories::default();
            session.stories.restore().unwrap();
            respond(&mut session, "Elin went to the kitchen.", None);
            respond(&mut session, line, None);
            let reply = respond(&mut session, "Where is Elin?", None);
            assert_eq!(serde_json::to_value(reply).unwrap()["kind"], "refusal");
        }
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
