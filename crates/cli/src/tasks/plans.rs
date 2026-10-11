//! Ordered clauses share the existing parser and verified task executor.
use super::*;
use glassmind_meaning::{
    Question,
    interface::{Language, Understanding},
};
use glassmind_mind::task::{PlanStep, TaskPlan};

const MAX_STEPS: usize = 4;

pub(super) fn clauses(text: &str) -> Result<Option<Vec<String>>, String> {
    if text.len() > 8192 {
        return Err("task request exceeds 8192 bytes".into());
    }
    let mut quote = None;
    let mut parts = Vec::new();
    let mut start = 0;
    let mut after_semicolon = false;
    for (i, c) in text.char_indices() {
        if let Some(end) = quote {
            if c == end {
                quote = None;
            }
            continue;
        }
        let previous = text.get(..i).and_then(|s| s.chars().next_back());
        let next = text.get(i + c.len_utf8()..).and_then(|s| s.chars().next());
        let apostrophe = c == '\''
            && previous.is_some_and(char::is_alphanumeric)
            && next.is_some_and(char::is_alphanumeric);
        if matches!(c, '`' | '"' | '\'') && !apostrophe {
            quote = Some(c);
            continue;
        }
        if i < start {
            continue;
        }
        let then = text
            .get(i..i + 4)
            .is_some_and(|s| s.eq_ignore_ascii_case("then"))
            && previous.is_none_or(|c| c.is_whitespace() || c == ';')
            && text
                .get(i + 4..)
                .and_then(|s| s.chars().next())
                .is_none_or(char::is_whitespace);
        if c == ';' || then {
            let part = text
                .get(start..i)
                .ok_or("invalid clause boundary")?
                .trim()
                .trim_end_matches(',')
                .trim();
            if part.is_empty() {
                if !(then && after_semicolon) {
                    return Err("empty step in task plan".into());
                }
            } else {
                parts.push(part.to_owned());
            }
            start = i + if then { 4 } else { c.len_utf8() };
            after_semicolon = c == ';';
        }
    }
    if quote.is_some() {
        return Err("unclosed quote in task request".into());
    }
    if parts.is_empty() {
        return Ok(None);
    }
    let tail = text.get(start..).ok_or("invalid final clause")?.trim();
    if tail.is_empty() {
        return Err("empty final step in task plan".into());
    }
    parts.push(tail.to_owned());
    if parts.len() > MAX_STEPS {
        return Err("at most four operations per task plan".into());
    }
    Ok(Some(parts))
}

fn interpretations(system: &System, text: &str) -> Result<Vec<Utterance<Rust>>, String> {
    if glassmind_english_rust::add_fields::request(text).is_some() {
        return Err("add-field actions are not yet plan operations".into());
    }
    let language = crate::coding::CodingLanguage {
        english: &system.english,
        switches: &system.switches,
    };
    let Understanding::Readings(readings) = language.understand(text) else {
        return Err(format!("cannot understand complete plan step: {text}"));
    };
    let mut meanings: Vec<_> = readings
        .into_iter()
        .filter(|r| language.exact(r))
        .map(|r| r.meaning)
        .collect();
    if meanings.is_empty()
        || meanings
            .iter()
            .any(|m| !matches!(m, Utterance::Command(_) | Utterance::Ask(_)))
    {
        return Err(format!(
            "plan steps must be complete actions or questions: {text}"
        ));
    }
    meanings.sort();
    meanings.dedup();
    Ok(meanings)
}

pub(super) fn status(plan: &TaskPlan<Rust>) -> String {
    let next = plan
        .steps
        .get(plan.current)
        .map_or(String::new(), |s| format!("\nNext: {}", s.text));
    format!(
        "Plan {:?}: {}/{} operations completed.{next}",
        plan.status,
        plan.current,
        plan.steps.len()
    )
}

pub(super) fn start(
    session: &mut CodingSession,
    path: &Path,
    original: &str,
    parts: Vec<String>,
    explain: bool,
    budget: u8,
) -> Result<String, String> {
    let system = open(&session.repository);
    let mut introduced = std::collections::BTreeSet::new();
    let steps = parts
        .into_iter()
        .map(|text| {
            let meanings = interpretations(&system, &text)?;
            // Unknown words can form a grammatical location question. At least
            // one reading must refer to existing code or a name introduced by
            // an earlier step; actual resolution remains with the mind.
            if !meanings.iter().any(|m| {
                m.things().iter().all(|thing| match thing {
                    Thing::Named(name) => {
                        introduced.contains(&name.0)
                            || !system.world.lookup(name, &mut Vec::new()).is_empty()
                    }
                    _ => true,
                })
            }) {
                return Err(format!("unresolved names in plan step: {text}"));
            }
            for meaning in &meanings {
                if let Utterance::Command(Action::Rename {
                    name: Some(name), ..
                }) = meaning
                {
                    introduced.insert(name.0.clone());
                }
            }
            Ok(PlanStep {
                text,
                interpretations: meanings,
                task: None,
                result: None,
            })
        })
        .collect::<Result<Vec<_>, String>>();
    let steps = match steps {
        Ok(steps) => steps,
        Err(error) => {
            // `then` can also be an ordinary identifier in a complete request.
            if glassmind_english_rust::rename_requests::request(original).is_some()
                && interpretations(&system, original).is_ok()
            {
                return Ok(request(session, path, original, explain, budget, false)?.message);
            }
            if let Some(task) = &mut session.task {
                task.status = TaskStatus::Cancelled;
            }
            if let Some(plan) = &mut session.plan {
                plan.status = TaskStatus::Cancelled;
            }
            save(path, session)?;
            return Ok(format!(
                "Cannot start plan: {error}. No plan operations executed."
            ));
        }
    };
    session.plan = Some(TaskPlan {
        steps,
        current: 0,
        status: TaskStatus::Ready,
    });
    session.task = None;
    save(path, session)?;
    resume(session, path, explain, budget)
}

pub(super) fn resume(
    session: &mut CodingSession,
    path: &Path,
    explain: bool,
    budget: u8,
) -> Result<String, String> {
    let mut messages = Vec::new();
    for _ in 0..budget {
        let plan = session.plan.as_ref().ok_or("plan missing")?;
        if plan.status != TaskStatus::Ready {
            break;
        }
        let index = plan.current;
        let step = plan
            .steps
            .get(index)
            .cloned()
            .ok_or("plan position out of bounds")?;
        if plan.steps.len() > MAX_STEPS {
            return Err("saved plan exceeds operation limit".into());
        }
        if fingerprint(&snapshot(&session.repository)?) != session.revision {
            invalidate(session, fingerprint(&snapshot(&session.repository)?));
            break;
        }
        let fresh = interpretations(&open(&session.repository), &step.text)?;
        if fresh != step.interpretations {
            invalidate(session, session.revision.clone());
            messages.push("Saved plan interpretation changed; request a fresh plan.".into());
            break;
        }
        if let Some(plan) = &mut session.plan {
            plan.status = TaskStatus::Applying;
        }
        save(path, session)?;
        let (message, result, task, completed, blocked) = if let Some(task) = step.task {
            if !matches!(task.status, TaskStatus::Ready | TaskStatus::Revising)
                || session.task.as_ref() != Some(&task)
            {
                invalidate(session, session.revision.clone());
                break;
            }
            let message = revisions::advance(session, path, 1)?;
            let task = session
                .task
                .clone()
                .ok_or("current plan task disappeared")?;
            let completed = task.status == TaskStatus::Completed;
            let result = task.events.last().map(|e| e.result.clone());
            let blocked = !matches!(
                task.status,
                TaskStatus::Ready | TaskStatus::Revising | TaskStatus::Completed
            );
            (message, result, Some(task), completed, blocked)
        } else {
            session.task = None;
            let observed = request(session, path, &step.text, explain, 1, true)?;
            if matches!(observed.meaning, Some(Utterance::Command(_))) {
                let task = session
                    .task
                    .clone()
                    .ok_or("action did not produce a task")?;
                let completed = task.status == TaskStatus::Completed;
                let blocked = !matches!(
                    task.status,
                    TaskStatus::Ready | TaskStatus::Revising | TaskStatus::Completed
                );
                let result = completed.then_some(observed.reply);
                (observed.message, result, Some(task), completed, blocked)
            } else {
                let complete = match (&observed.meaning, plain(&observed.reply)) {
                    (
                        Some(Utterance::Ask(Question::Passes(_))),
                        Reply::Checked { problems, .. },
                    ) => problems.is_empty(),
                    (_, Reply::Trouble(_) | Reply::Clarify { .. }) => false,
                    (Some(Utterance::Ask(_)), _) => true,
                    _ => false,
                };
                (
                    observed.message,
                    Some(observed.reply),
                    None,
                    complete,
                    !complete,
                )
            }
        };
        messages.push(message);
        if let Some(plan) = &mut session.plan {
            if plan.status == TaskStatus::Stale {
                break;
            }
            let record = plan.steps.get_mut(index).ok_or("plan position changed")?;
            record.result = result;
            record.task = task;
            if completed {
                plan.current += 1;
            }
            plan.status = if blocked {
                TaskStatus::Blocked
            } else if plan.current == plan.steps.len() {
                TaskStatus::Completed
            } else {
                TaskStatus::Ready
            };
        }
        save(path, session)?;
    }
    save(path, session)?;
    messages.push(super::status(session));
    Ok(messages.join("\n"))
}
