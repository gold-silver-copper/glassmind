//! Explicit selection retains the requested action and uses the normal verifier.
use super::*;
use glassmind_meaning::{Condition, Description, Labelled, Selection, Trouble};
use glassmind_world_rust::RustKind;

pub(super) fn status(task: &Task<Rust>) -> String {
    let Some(choices) = &task.choices else {
        return String::new();
    };
    if !matches!(task.status, TaskStatus::Blocked | TaskStatus::Ready) {
        return String::new();
    }
    let mut lines = vec!["Choose a target by number or qualified name:".to_owned()];
    for (index, item) in choices.candidates.iter().enumerate() {
        let selected = if choices.selected == Some(index) {
            " (selected)"
        } else {
            ""
        };
        lines.push(format!(
            "{}. {} at {}{selected}",
            index + 1,
            item.label(),
            item.place()
        ));
    }
    lines.join("\n")
}

pub(super) fn select(
    session: &mut CodingSession,
    path: &Path,
    selection: Selection,
    text: &str,
    explain: bool,
    budget: u8,
) -> Result<String, String> {
    if session
        .plan
        .as_ref()
        .is_some_and(|p| !matches!(p.status, TaskStatus::Ready | TaskStatus::Blocked))
    {
        return Ok("This plan no longer has an active target choice.".into());
    }
    let Some(old) = session.task.clone().filter(|t| {
        matches!(
            t.status,
            TaskStatus::Ready | TaskStatus::Blocked | TaskStatus::Revising
        )
    }) else {
        return Ok("No unresolved target choice is available. State a fresh request.".into());
    };
    let Some(mut choices) = old.choices.clone() else {
        if let Some(task) = &mut session.task {
            task.status = TaskStatus::Cancelled;
        }
        if let Some(plan) = &mut session.plan {
            plan.status = TaskStatus::Cancelled;
        }
        save(path, session)?;
        return Ok(
            "This task has no target alternatives; its pending action was cancelled.".into(),
        );
    };
    if choices.candidates.len() < 2 || choices.candidates.len() > 32 {
        return Err("invalid saved target alternatives".into());
    }
    let selected = match selection {
        Selection::Number(number) => number
            .checked_sub(1)
            .filter(|&n| n < choices.candidates.len()),
        Selection::Other if choices.candidates.len() == 2 => {
            choices.selected.filter(|&n| n < 2).map(|n| 1 - n)
        }
        Selection::Other => None,
        Selection::Named(name) => {
            let matches: Vec<_> = choices
                .candidates
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    item.label() == name.0 || item.label().replace("::", ".") == name.0
                })
                .map(|(i, _)| i)
                .collect();
            matches.first().copied().filter(|_| matches.len() == 1)
        }
    };
    let Some(selected) = selected else {
        return Ok(format!(
            "That does not identify one offered alternative.\n{}",
            status(&old)
        ));
    };
    let item = choices
        .candidates
        .get(selected)
        .cloned()
        .ok_or("choice disappeared")?;
    let system = open(&session.repository);
    // The saved list is evidence, not an authority to fabricate an item.
    let recorded = old.events.iter().any(|e| match &e.result {
        Reply::Trouble(Trouble::Ambiguous { candidates }) => {
            let mut candidates = candidates.clone();
            candidates.sort();
            candidates.dedup();
            candidates == choices.candidates
        }
        _ => false,
    });
    let fresh = choices.candidates.iter().all(|item| {
        system
            .world
            .lookup(&Name(item.label()), &mut Vec::new())
            .contains(item)
    });
    if !recorded || !fresh {
        invalidate(session, session.revision.clone());
        save(path, session)?;
        return Ok(
            "Saved target alternatives no longer match the code. State a fresh request.".into(),
        );
    }
    let mut action = old.request.clone();
    let Action::Rename { thing, .. } = &mut action else {
        return Ok("This action does not support target selection.".into());
    };
    *thing = Thing::Item(item.clone());
    if item.kind == RustKind::Field {
        let owner_name = item.owner.as_ref().ok_or("selected field has no owner")?;
        let owners: Vec<_> = system
            .world
            .lookup(&Name(owner_name.clone()), &mut Vec::new())
            .into_iter()
            .filter(|owner| {
                owner.kind == RustKind::Struct
                    && owner.source == item.source
                    && owner.file == item.file
            })
            .collect();
        let Some(owner) = owners.first().filter(|_| owners.len() == 1) else {
            return Ok(
                "The selected field's owner is still ambiguous. State an explicit request.".into(),
            );
        };
        // Preserve the world's requirement for an explicit owner; do not relax
        // the homonymous-field guard just because the frontend chose an item.
        *thing = Thing::Fitting(Description {
            kind: Some(RustKind::Field),
            conditions: vec![
                Condition::Is(Thing::Item(item)),
                Condition::PartOf(Thing::Item(owner.clone())),
            ],
        });
    }
    if let Some(plan) = &session.plan
        && plan.steps.get(plan.current).and_then(|s| s.task.as_ref()) != Some(&old)
    {
        invalidate(session, session.revision.clone());
        save(path, session)?;
        return Ok("The saved choice no longer belongs to the current plan step.".into());
    }
    let reply = system
        .world
        .propose(&action, &mut Vec::new())
        .unwrap_or_else(Reply::Trouble);
    if fingerprint(&snapshot(&session.repository)?) != session.revision {
        invalidate(session, fingerprint(&snapshot(&session.repository)?));
        save(path, session)?;
        return Ok("Source changed while selecting a target. State a fresh request.".into());
    }
    let mut task = Task::proposed(action.clone(), old.original_text, reply.clone());
    choices.selected = Some(selected);
    task.choices = Some(choices);
    let new_events = std::mem::take(&mut task.events);
    task.events = old.events;
    task.events.extend(new_events);
    let ready = task.status == TaskStatus::Ready;
    let mut message = say(&system, &reply);
    session.conversation.turns.push(TurnRecord {
        text: text.to_owned(),
        meaning: Some(Utterance::Command(action)),
        reply,
        spoken: message.clone(),
    });
    if let Some(plan) = &mut session.plan {
        let step = plan
            .steps
            .get_mut(plan.current)
            .ok_or("current plan step missing")?;
        step.task = Some(task.clone());
        step.result = None;
        plan.status = if ready {
            TaskStatus::Ready
        } else {
            TaskStatus::Blocked
        };
    }
    session.task = Some(task);
    save(path, session)?;
    if ready && budget > 1 {
        let result = if session.plan.is_some() {
            plans::resume(session, path, explain, budget - 1)?
        } else {
            apply(session, path)?
        };
        message.push_str(&format!("\n{result}"));
    } else {
        message.push_str(&format!("\n{}", super::status(session)));
    }
    Ok(message)
}
