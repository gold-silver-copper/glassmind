//! Parameter corrections retain the active task and the unexecuted plan tail.
use super::*;
use glassmind_meaning::Trouble;

pub(super) fn answer_missing(session: &CodingSession, text: &str) -> Option<String> {
    let task = session.task.as_ref()?;
    if task.status != TaskStatus::Blocked
        || task.choices.is_some()
        || !matches!(&task.request, Action::AddMember { member, .. } if member.default.is_none())
        || !matches!(task.events.last().map(|e| &e.result), Some(Reply::Trouble(Trouble::MissingParameter { parameter })) if parameter == "field default")
    {
        return None;
    }
    glassmind_english_rust::add_fields::literal_answer(text)
}

pub(super) fn amend(
    session: &mut CodingSession,
    path: &Path,
    text: &str,
    explain: bool,
    budget: u8,
) -> Result<String, String> {
    let Some(old) = session
        .task
        .clone()
        .filter(|t| matches!(t.status, TaskStatus::Ready | TaskStatus::Blocked))
    else {
        return Ok("No pending action can take this correction. State a fresh action.".into());
    };
    if session.plan.as_ref().is_some_and(|p| {
        !matches!(p.status, TaskStatus::Ready | TaskStatus::Blocked)
            || p.steps.get(p.current).and_then(|s| s.task.as_ref()) != Some(&old)
    }) {
        return Ok("No active plan step can take this correction.".into());
    }
    let parsed = glassmind_english_rust::add_fields::amendment_request(text)
        .or_else(|| {
            answer_missing(session, text).map(|value| {
                Ok(glassmind_world_rust::amendments::RustAmendment::FieldDefault(value))
            })
        })
        .ok_or("parameter correction missing")?;
    let system = open(&session.repository);
    let revised = match parsed {
        Ok(change) => system.world.amend(&old.request, &change),
        Err(reason) => Err(Trouble::ActionFailed { reason }),
    };
    let action = match revised {
        Ok(action) => action,
        Err(trouble) => {
            if let Some(task) = &mut session.task {
                task.status = TaskStatus::Cancelled;
            }
            if let Some(plan) = &mut session.plan {
                plan.status = TaskStatus::Cancelled;
            }
            session.conversation.turns.clear();
            save(path, session)?;
            return Ok(format!(
                "{} Pending execution cancelled.",
                say(&system, &Reply::Trouble(trouble))
            ));
        }
    };
    let reply = system
        .world
        .propose(&action, &mut Vec::new())
        .unwrap_or_else(Reply::Trouble);
    if fingerprint(&snapshot(&session.repository)?) != session.revision {
        invalidate(session, fingerprint(&snapshot(&session.repository)?));
        save(path, session)?;
        return Ok("Source changed during correction; state a fresh action.".into());
    }
    let mut message = say(&system, &reply);
    let task = Task::proposed(action.clone(), text.into(), reply.clone());
    for turn in &mut session.conversation.turns {
        if matches!(
            plain(&turn.reply),
            Reply::Proposal { .. }
                | Reply::Patch { .. }
                | Reply::Remedy { .. }
                | Reply::Clarify { .. }
        ) {
            turn.reply = Reply::Declined;
        }
    }
    session.conversation.turns.push(TurnRecord {
        text: text.into(),
        meaning: Some(Utterance::Command(action)),
        reply: reply.clone(),
        spoken: message.clone(),
    });
    if let Some(plan) = &mut session.plan {
        let step = plan
            .steps
            .get_mut(plan.current)
            .ok_or("plan step disappeared")?;
        step.task = Some(task.clone());
        step.result = Some(reply);
        plan.status = task.status;
    }
    session.task = Some(task);
    save(path, session)?;
    if budget > 1 {
        message.push('\n');
        message.push_str(&if session.plan.is_some() {
            plans::resume(session, path, explain, budget - 1)?
        } else {
            revisions::advance(session, path, budget - 1)?
        });
    } else {
        message.push_str(&format!("\n{}", super::status(session)));
    }
    Ok(message)
}
