//! Durable, bounded repair attempts; all intermediate edits stay in scratch.
use super::*;
use glassmind_mind::task::{RevisionAttempt, TaskEvent, TaskRevision, TaskStage};
use glassmind_world_rust::{
    checks::{self, Finding, RustCheck},
    revision,
};

const MAX_ATTEMPTS: usize = 4;

struct Trial(PathBuf);
impl Drop for Trial {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Trial {
    fn new(files: &Snapshot) -> Result<Self, String> {
        let trial = Self(checks::scratch("task-revision"));
        fs::create_dir_all(&trial.0).map_err(|e| e.to_string())?;
        for (file, bytes) in files {
            let path = trial.0.join(file);
            let parent = path.parent().ok_or("trial parent missing")?;
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            fs::write(path, bytes).map_err(|e| e.to_string())?;
        }
        Ok(trial)
    }
    fn apply(&self, changes: &[checks::RustChange]) -> Result<(), String> {
        if changes.iter().any(|c| {
            c.source != "project"
                || !Path::new(&c.file)
                    .components()
                    .all(|p| matches!(p, std::path::Component::Normal(_)))
        }) {
            return Err("repair proposal refers outside the project".into());
        }
        checks::apply(
            changes,
            &BTreeMap::from([("project".into(), self.0.clone())]),
        )?;
        Ok(())
    }
}

pub(super) fn begin(task: &mut Task<Rust>) {
    if !matches!(task.request, Action::Repair(_)) || task.status != TaskStatus::Blocked {
        return;
    }
    if let Some(TaskEvent {
        result: Reply::Remedy { left, .. },
        ..
    }) = task.events.last()
        && !left.is_empty()
    {
        task.revision = Some(TaskRevision {
            initial: left.clone(),
            attempts: Vec::new(),
            stopped: None,
        });
        task.status = TaskStatus::Revising;
    }
}

pub(super) fn status(task: &Task<Rust>) -> String {
    task.revision.as_ref().map_or(String::new(), |r| {
        let reason = r
            .stopped
            .as_ref()
            .map_or(String::new(), |s| format!(" {s}"));
        let last = r.attempts.last().map_or(String::new(), |a| {
            format!(
                " Last trial: {} problems before, {} after.",
                a.before.len(),
                a.after.len()
            )
        });
        format!(
            "{} of {MAX_ATTEMPTS} isolated repair attempts.{last}{reason}",
            r.attempts.len()
        )
    })
}

fn problems(found: &[Finding]) -> Vec<glassmind_world_rust::RustItem> {
    found.iter().map(|f| f.item.clone()).collect()
}

/// Recreate a saved trial from live source and freshly generated candidates.
/// Persisted changes, diagnostics and hashes must all match independent replay.
fn replay(
    session: &CodingSession,
    task: &Task<Rust>,
) -> Result<(Trial, RustCheck, Vec<Finding>), String> {
    let Action::Repair(check) = task.request else {
        return Err("revision is not a repair".into());
    };
    let revision = task.revision.as_ref().ok_or("repair evidence missing")?;
    if revision.attempts.len() > MAX_ATTEMPTS {
        return Err("saved repair exceeds attempt limit".into());
    }
    let original = snapshot(&session.repository)?;
    if fingerprint(&original) != session.revision {
        return Err("source changed during repair".into());
    }
    let trial = Trial::new(&original)?;
    let mut found = checks::run(check, "project", &trial.0)?;
    let mut seen = std::collections::BTreeSet::from([fingerprint(&original)]);
    for attempt in &revision.attempts {
        let changes = revision::candidate("project", &trial.0, &found);
        if changes.is_empty()
            || changes != attempt.changes
            || revision::diagnostics(&problems(&found)) != revision::diagnostics(&attempt.before)
        {
            return Err("saved repair no longer matches a fresh proposal".into());
        }
        trial.apply(&changes)?;
        found = checks::run(check, "project", &trial.0)?;
        let after = fingerprint(&snapshot(&trial.0)?);
        if after != attempt.source_after
            || !seen.insert(after)
            || revision::diagnostics(&problems(&found)) != revision::diagnostics(&attempt.after)
        {
            return Err("saved repair no longer matches fresh verification".into());
        }
    }
    Ok((trial, check, found))
}

fn checked_proposal(
    session: &CodingSession,
    task: &Task<Rust>,
    trial: &Trial,
    check: RustCheck,
) -> Result<Reply<Rust>, String> {
    let changes = revision::compose(
        "project",
        &snapshot(&session.repository)?,
        &snapshot(&trial.0)?,
    )?;
    if changes.is_empty() {
        return Err("revision produced no change".into());
    }
    Ok(Reply::Remedy {
        check,
        changes,
        fixed: task
            .revision
            .as_ref()
            .ok_or("repair evidence missing")?
            .initial
            .clone(),
        left: Vec::new(),
    })
}

pub(super) fn proposal(session: &CodingSession) -> Result<Reply<Rust>, String> {
    let task = session.task.as_ref().ok_or("task missing")?;
    let (trial, check, found) = replay(session, task)?;
    if !found.is_empty() {
        return Err("replayed repair still has problems".into());
    }
    checked_proposal(session, task, &trial, check)
}

fn stop(task: &mut Task<Rust>, reason: String) {
    task.status = TaskStatus::Blocked;
    if let Some(revision) = &mut task.revision {
        revision.stopped = Some(reason);
    }
}

fn attempt(session: &mut CodingSession, path: &Path) -> Result<String, String> {
    let mut task = session.task.clone().ok_or("task missing")?;
    let count = task
        .revision
        .as_ref()
        .ok_or("repair evidence missing")?
        .attempts
        .len();
    if count >= MAX_ATTEMPTS {
        stop(
            &mut task,
            "Repair attempt limit reached; source is unchanged.".into(),
        );
        session.task = Some(task);
        save(path, session)?;
        return Ok(status(session.task.as_ref().ok_or("task missing")?));
    }
    if let Some(current) = &mut session.task {
        current.status = TaskStatus::Applying;
    }
    save(path, session)?;
    let outcome = (|| -> Result<(), String> {
        let (trial, check, found) = replay(session, &task)?;
        let before = problems(&found);
        let changes = revision::candidate("project", &trial.0, &found);
        if changes.is_empty() {
            return Err("No supported next repair; source is unchanged.".into());
        }
        trial.apply(&changes)?;
        let after = problems(&checks::run(check, "project", &trial.0)?);
        let source_after = fingerprint(&snapshot(&trial.0)?);
        let state = task.revision.as_mut().ok_or("repair evidence missing")?;
        if state.attempts.is_empty() {
            state.initial = before.clone();
        }
        let repeated = source_after == session.revision
            || state
                .attempts
                .iter()
                .any(|a| a.source_after == source_after)
            || (!after.is_empty()
                && (revision::diagnostics(&before) == revision::diagnostics(&after)
                    || state.attempts.iter().any(|a| {
                        revision::diagnostics(&a.before) == revision::diagnostics(&after)
                    })));
        state.attempts.push(RevisionAttempt {
            changes,
            before,
            after: after.clone(),
            source_after,
        });
        task.events.push(TaskEvent {
            stage: TaskStage::Revised,
            result: Reply::Checked {
                check,
                problems: after.clone(),
            },
        });
        if after.is_empty() {
            let proposal = checked_proposal(session, &task, &trial, check)?;
            task.events.push(TaskEvent {
                stage: TaskStage::Proposed,
                result: proposal,
            });
            task.status = TaskStatus::Ready;
        } else if repeated {
            stop(
                &mut task,
                "Repeated source or diagnostics; source is unchanged.".into(),
            );
        } else if count + 1 == MAX_ATTEMPTS {
            stop(
                &mut task,
                "Repair attempt limit reached; source is unchanged.".into(),
            );
        } else {
            task.status = TaskStatus::Revising;
        }
        Ok(())
    })();
    if let Err(error) = outcome {
        stop(&mut task, error);
    }
    session.task = Some(task);
    let current = fingerprint(&snapshot(&session.repository)?);
    if current != session.revision {
        invalidate(session, current);
    }
    save(path, session)?;
    Ok(super::status(session))
}

pub(super) fn advance(
    session: &mut CodingSession,
    path: &Path,
    budget: u8,
) -> Result<String, String> {
    let mut messages = Vec::new();
    for _ in 0..budget {
        match session.task.as_ref().map(|t| t.status) {
            Some(TaskStatus::Revising) => messages.push(attempt(session, path)?),
            Some(TaskStatus::Ready) => {
                messages.push(apply(session, path)?);
                break;
            }
            _ => break,
        }
    }
    if messages.is_empty() {
        messages.push(super::status(session));
    }
    Ok(messages.join("\n"))
}
