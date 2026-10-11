//! Persistent coding tasks using the existing meaning, dialogue and Rust verifier.
mod choices;
mod plans;
mod revisions;
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use glassmind_meaning::interface::{Language as _, World as _};
use glassmind_meaning::{Action, Name, Reply, Thing, Utterance};
use glassmind_mind::{
    Session, TurnRecord,
    task::{Task, TaskStatus},
};
use glassmind_rules::Switches;
use glassmind_world_rust::Rust;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::system::System;

type Snapshot = BTreeMap<String, Vec<u8>>;
const LIMIT: usize = 32 * 1024 * 1024;

struct SessionLock(fs::File);
impl Drop for SessionLock {
    fn drop(&mut self) {
        // A concurrent fork may briefly inherit an open file description.
        // Unlock explicitly rather than relying only on its final close.
        let _ = self.0.unlock();
    }
}

#[derive(Serialize, Deserialize)]
struct CodingSession {
    version: u32,
    repository: PathBuf,
    revision: String,
    conversation: Session<Rust>,
    task: Option<Task<Rust>>,
    #[serde(default)]
    plan: Option<glassmind_mind::task::TaskPlan<Rust>>,
    /// Only files actually changed by the last successful action are retained.
    undo: BTreeMap<String, Option<Vec<u8>>>,
}

fn destination(path: &Path, repository: &Path) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = parent
        .canonicalize()
        .map_err(|e| format!("session/output parent: {e}"))?;
    let path = parent.join(path.file_name().ok_or("output filename required")?);
    if path.starts_with(repository) || fs::symlink_metadata(&path).is_ok_and(|m| !m.is_file()) {
        return Err("task session and output must be regular files outside the repository".into());
    }
    Ok(path)
}

fn snapshot(root: &Path) -> Result<Snapshot, String> {
    let mut files = Snapshot::new();
    let mut pending = vec![root.to_path_buf()];
    let mut size = 0;
    let mut directories = 0;
    while let Some(dir) = pending.pop() {
        directories += 1;
        if directories > 4096 {
            return Err("task snapshot exceeds 4096 directories".into());
        }
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() {
                if entry.file_name() != ".git" && entry.file_name() != "target" {
                    pending.push(entry.path());
                }
            } else if kind.is_file() {
                let metadata = entry.metadata().map_err(|e| e.to_string())?;
                if metadata.len() > LIMIT as u64
                    || size + metadata.len() as usize > LIMIT
                    || files.len() >= 4096
                {
                    return Err("task snapshot exceeds 4096 files or 32 MiB".into());
                }
                let bytes = fs::read(entry.path()).map_err(|e| e.to_string())?;
                size += bytes.len();
                if size > LIMIT {
                    return Err("task snapshot grew beyond 32 MiB".into());
                }
                let path = entry.path();
                let relative = path
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_str()
                    .ok_or("non-UTF8 source path")?;
                files.insert(relative.to_owned(), bytes);
            } else {
                return Err(
                    "task snapshots require regular files; symlinks are unsupported".into(),
                );
            }
        }
    }
    Ok(files)
}

fn fingerprint(files: &Snapshot) -> String {
    let mut hash = Sha256::new();
    for (path, bytes) in files {
        hash.update((path.len() as u64).to_le_bytes());
        hash.update(path.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    format!("{:x}", hash.finalize())
}

fn save(path: &Path, session: &CodingSession) -> Result<(), String> {
    let bytes = serde_json::to_vec(session).map_err(|e| e.to_string())?;
    if bytes.len() > 4 * LIMIT {
        return Err("coding session exceeds size limit".into());
    }
    let temporary = path.with_extension(format!("task-tmp-{}", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        fs::rename(&temporary, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn plain(reply: &Reply<Rust>) -> &Reply<Rust> {
    match reply {
        Reply::Assuming { reply, .. } => plain(reply),
        _ => reply,
    }
}

/// Diagnostic prose includes fresh test process IDs. Execution authority is
/// the exact edit and check, with no remaining unresolved problems.
fn same_proposal(saved: &Reply<Rust>, fresh: &Reply<Rust>) -> bool {
    match (plain(saved), plain(fresh)) {
        (
            Reply::Remedy {
                check: a,
                changes: x,
                left: left_a,
                ..
            },
            Reply::Remedy {
                check: b,
                changes: y,
                left: left_b,
                ..
            },
        ) => a == b && x == y && left_a.is_empty() && left_b.is_empty(),
        (
            Reply::Patch {
                action: a,
                changes: x,
                check: c,
                ..
            },
            Reply::Patch {
                action: b,
                changes: y,
                check: d,
                ..
            },
        ) => a == b && x == y && c == d,
        (a @ Reply::Proposal { .. }, b @ Reply::Proposal { .. }) => a == b,
        _ => false,
    }
}

fn say(system: &System, reply: &Reply<Rust>) -> String {
    crate::coding::CodingLanguage {
        english: &system.english,
        switches: &system.switches,
    }
    .say(reply)
    .map_or_else(|| format!("{reply:?}"), |s| s.text)
}

fn status(session: &CodingSession) -> String {
    if let Some(plan) = &session.plan {
        let mut message = plans::status(plan);
        if let Some(task) = &session.task {
            let revision = revisions::status(task);
            if !revision.is_empty() {
                message.push_str(&format!("\n{revision}"));
            }
            let choices = choices::status(task);
            if !choices.is_empty() {
                message.push_str(&format!("\n{choices}"));
            }
        }
        return message;
    }
    session.task.as_ref().map_or_else(
        || "No coding task is recorded.".into(),
        |t| {
            let mut message = format!(
                "Task {:?}: {}\n{} recorded stages.",
                t.status,
                t.original_text,
                t.events.len()
            );
            let revision = revisions::status(t);
            if !revision.is_empty() {
                message.push_str(&format!("\n{revision}"));
            }
            let choices = choices::status(t);
            if !choices.is_empty() {
                message.push_str(&format!("\n{choices}"));
            }
            message
        },
    )
}

fn open(root: &Path) -> System {
    System::open(
        &[("project".into(), root.to_path_buf())],
        Switches::all_on(),
    )
}

fn invalidate(session: &mut CodingSession, revision: String) {
    if let Some(task) = &mut session.task {
        task.status = TaskStatus::Stale;
    }
    if let Some(plan) = &mut session.plan {
        plan.status = TaskStatus::Stale;
    }
    session.conversation.turns.clear();
    session.undo.clear();
    session.revision = revision;
}

fn apply(session: &mut CodingSession, path: &Path) -> Result<String, String> {
    let before = snapshot(&session.repository)?;
    if fingerprint(&before) != session.revision {
        invalidate(session, fingerprint(&before));
        save(path, session)?;
        return Ok("Source changed. Task and conversational references invalidated; state a fresh request.".into());
    }
    let Some(proposal) = session.task.as_ref().and_then(Task::proposal).cloned() else {
        return Ok(status(session));
    };
    let system = open(&session.repository);
    let request = session.task.as_ref().ok_or("task missing")?.request.clone();
    let fresh = if session
        .task
        .as_ref()
        .is_some_and(|task| task.revision.is_some())
    {
        revisions::proposal(session)
    } else {
        system
            .world
            .propose(&request, &mut Vec::new())
            .map_err(|e| format!("{e:?}"))
    };
    let refreshed = match fresh {
        Ok(reply) => reply,
        Err(trouble) => {
            if let Some(task) = &mut session.task {
                task.status = TaskStatus::Blocked;
            }
            save(path, session)?;
            return Ok(format!("Cannot revalidate task: {trouble:?}"));
        }
    };
    if !same_proposal(&proposal, &refreshed)
        || fingerprint(&snapshot(&session.repository)?) != session.revision
    {
        invalidate(session, fingerprint(&snapshot(&session.repository)?));
        save(path, session)?;
        return Ok(
            "The saved proposal no longer matches fresh verification; state a fresh request."
                .into(),
        );
    }
    if let Some(task) = &mut session.task {
        task.status = TaskStatus::Applying;
    }
    // An interrupted application cannot be replayed as a fresh authorization.
    save(path, session)?;
    let result = system.world.apply(&refreshed, &mut Vec::new());
    let reply = result.unwrap_or_else(Reply::Trouble);
    let after = snapshot(&session.repository)?;
    if matches!(reply, Reply::Applied { .. }) {
        session.undo = before
            .keys()
            .chain(after.keys())
            .filter(|p| before.get(*p) != after.get(*p))
            .map(|p| (p.clone(), before.get(p).cloned()))
            .collect();
    }
    session.revision = fingerprint(&after);
    let spoken = say(&system, &reply);
    if let Some(task) = &mut session.task {
        task.applied(reply.clone());
        // The verified proposal can resolve an owner-qualified description to
        // one item. Use that item when rebinding the conversational target.
        let mut request = match &refreshed {
            Reply::Proposal { action, .. } => action.clone(),
            _ => task.request.clone(),
        };
        // Follow-up references name the renamed item, not its stale identifier.
        let mut topic_valid = true;
        if matches!(reply, Reply::Applied { .. })
            && let Action::Rename {
                thing: Thing::Item(item),
                name: Some(name),
                ..
            } = &mut request
        {
            let fresh = open(&session.repository);
            let candidates = fresh.world.lookup(&Name(name.0.clone()), &mut Vec::new());
            let mut matching = candidates.into_iter().filter(|i| {
                i.source == item.source
                    && i.file == item.file
                    && i.kind == item.kind
                    && i.owner == item.owner
            });
            if let Some(found) = matching.next().filter(|_| matching.next().is_none()) {
                *item = found;
            } else {
                session.conversation.turns.clear();
                topic_valid = false;
            }
        }
        session.conversation.turns.push(TurnRecord {
            text: task.original_text.clone(),
            meaning: topic_valid.then_some(Utterance::Command(request)),
            reply,
            spoken: spoken.clone(),
        });
    }
    save(path, session)?;
    if session.plan.is_some() {
        Ok(spoken)
    } else {
        Ok(format!("{spoken}\n{}", status(session)))
    }
}

fn undo(session: &mut CodingSession, path: &Path) -> Result<String, String> {
    if let Some(task) = &mut session.task {
        task.status = TaskStatus::Cancelled;
    }
    if let Some(plan) = &mut session.plan {
        plan.status = TaskStatus::Cancelled;
    }
    if session.undo.is_empty() {
        return Ok("No verified change is available to undo.".into());
    }
    let now = snapshot(&session.repository)?;
    if fingerprint(&now) != session.revision {
        invalidate(session, fingerprint(&now));
        save(path, session)?;
        return Ok("Source changed; undo refused to preserve external edits.".into());
    }
    // Validate every saved relative path before touching a file.
    for name in session.undo.keys() {
        if !Path::new(name)
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)))
        {
            return Err("invalid saved undo path".into());
        }
    }
    if let Some(task) = &mut session.task {
        task.status = TaskStatus::Applying;
    }
    save(path, session)?;
    let restored = (|| -> Result<(), String> {
        for (name, bytes) in &session.undo {
            let file = session.repository.join(name);
            match bytes {
                Some(bytes) => fs::write(file, bytes).map_err(|e| e.to_string())?,
                None => fs::remove_file(file).map_err(|e| e.to_string())?,
            }
        }
        Ok(())
    })();
    if let Err(error) = restored {
        let mut recovered = true;
        for name in session.undo.keys() {
            let file = session.repository.join(name);
            recovered &= match now.get(name) {
                Some(bytes) => fs::write(file, bytes).is_ok(),
                None => !file.exists() || fs::remove_file(file).is_ok(),
            };
        }
        return Err(format!(
            "undo failed: {error}; original state restored: {recovered}"
        ));
    }
    if let Some(task) = &mut session.task {
        task.status = TaskStatus::Cancelled;
    }
    session.conversation.turns.clear();
    session.undo.clear();
    session.revision = fingerprint(&snapshot(&session.repository)?);
    save(path, session)?;
    Ok(
        "Restored the source bytes from before the last verified change. Task references cleared."
            .into(),
    )
}

pub fn run(
    root: &Path,
    session_path: &Path,
    reply_file: Option<&Path>,
    explain: bool,
    steps: u8,
    text: &str,
) -> Result<(), String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let path = destination(session_path, &root)?;
    let output = reply_file.map(|p| destination(p, &root)).transpose()?;
    let lock_path = path.with_file_name(format!(
        "{}.lock",
        path.file_name()
            .ok_or("session filename required")?
            .to_string_lossy()
    ));
    if output
        .as_ref()
        .is_some_and(|p| p == &path || p == &lock_path)
    {
        return Err("reply file conflicts with task state".into());
    }
    // OS locks are released if the process dies. Keep the lock inode rather
    // than unlinking it while another process may already have opened it.
    if fs::symlink_metadata(&lock_path).is_ok_and(|m| !m.is_file()) {
        return Err("session lock must be a regular file".into());
    }
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|e| {
            format!(
                "cannot acquire task session lock {}: {e}",
                lock_path.display()
            )
        })?;
    lock.try_lock()
        .map_err(|e| format!("coding session is in use: {e}"))?;
    let _lock = SessionLock(lock);
    let system = open(&root);
    let revision = fingerprint(&snapshot(&root)?);
    let mut session: CodingSession = match fs::read(&path) {
        Ok(bytes) if bytes.len() <= 4 * LIMIT => {
            serde_json::from_slice(&bytes).map_err(|e| format!("invalid coding session: {e}"))?
        }
        Ok(_) => return Err("coding session exceeds size limit".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => CodingSession {
            version: 1,
            repository: root.clone(),
            revision: revision.clone(),
            conversation: Session::new(system.source_refs()),
            task: None,
            plan: None,
            undo: BTreeMap::new(),
        },
        Err(e) => return Err(e.to_string()),
    };
    if session.version != 1 || session.repository != root {
        return Err("session version or repository does not match".into());
    }
    let interrupted_plan = session
        .plan
        .as_ref()
        .is_some_and(|p| p.status == TaskStatus::Applying);
    let interrupted = session
        .task
        .as_ref()
        .is_some_and(|t| t.status == TaskStatus::Applying);
    let control = text
        .trim()
        .trim_end_matches(['.', '!'])
        .to_ascii_lowercase();
    let message = if session.revision != revision || interrupted || interrupted_plan {
        invalidate(&mut session, revision);
        "Source changed or execution was interrupted. Context invalidated; repeat a fresh explicit request.".into()
    } else {
        match control.as_str() {
            "show task" | "task status" => status(&session),
            "continue" | "continue task" | "apply it" | "yes" | "yes please" => {
                if session.plan.is_some() {
                    plans::resume(&mut session, &path, explain, steps)?
                } else {
                    revisions::advance(&mut session, &path, steps)?
                }
            }
            "undo" | "undo that change" | "undo the last change" => undo(&mut session, &path)?,
            "cancel task" | "stop" | "no" => {
                if let Some(plan) = &mut session.plan {
                    plan.status = TaskStatus::Cancelled;
                }
                if let Some(task) = &mut session.task {
                    task.status = TaskStatus::Cancelled;
                }
                session.conversation.turns.clear();
                "Task cancelled; no pending action will run.".into()
            }
            _ if glassmind_english::selection::request(text).is_some() => {
                let selection =
                    glassmind_english::selection::request(text).ok_or("selection missing")?;
                choices::select(&mut session, &path, selection, text, explain, steps)?
            }
            _ => match plans::clauses(text) {
                Ok(Some(parts)) => plans::start(&mut session, &path, text, parts, explain, steps)?,
                Ok(None) => request(&mut session, &path, text, explain, steps, false)?.message,
                Err(error) => {
                    if let Some(plan) = &mut session.plan {
                        plan.status = TaskStatus::Cancelled;
                    }
                    if let Some(task) = &mut session.task {
                        task.status = TaskStatus::Cancelled;
                    }
                    format!("Cannot start plan: {error}. Pending actions cancelled.")
                }
            },
        }
    };
    if session.conversation.turns.len() > 64 {
        session
            .conversation
            .turns
            .drain(..session.conversation.turns.len() - 64);
    }
    save(&path, &session)?;
    if let Some(output) = output {
        fs::write(output, format!("{message}\n")).map_err(|e| e.to_string())?;
    }
    say!("{message}");
    Ok(())
}

struct RequestResult {
    message: String,
    meaning: Option<Utterance<Rust>>,
    reply: Reply<Rust>,
}

fn request(
    session: &mut CodingSession,
    path: &Path,
    text: &str,
    explain: bool,
    steps: u8,
    within_plan: bool,
) -> Result<RequestResult, String> {
    let system = open(&session.repository);
    // New input may correct the previous meaning, but cannot consent
    // to a proposal by a path outside the journalled task executor.
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
    let turn = system.respond(&mut session.conversation, text);
    let mut message = if explain {
        format!("{}\n{}", turn.trace.render(), turn.spoken)
    } else {
        turn.spoken.clone()
    };
    let meaning = session
        .conversation
        .turns
        .last()
        .and_then(|r| r.meaning.clone());
    // A guessed question that could not resolve its subject is not evidence
    // that a replacement instruction was understood as read-only. Preserve a
    // pending task only for answered questions; report uncertainty explicitly.
    let answered_question = matches!(
        meaning,
        Some(Utterance::Ask(_) | Utterance::AskRecall | Utterance::AskReason)
    ) && !matches!(
        plain(&turn.reply),
        Reply::Trouble(_) | Reply::Declined | Reply::Clarify { .. }
    );
    if !answered_question
        && let Some(task) = &mut session.task
        && matches!(
            task.status,
            TaskStatus::Ready | TaskStatus::Blocked | TaskStatus::Revising
        )
    {
        task.status = TaskStatus::Cancelled;
        if !matches!(meaning, Some(Utterance::Command(_))) {
            message.push_str("\nPending action cancelled because the new request could not be resolved as a read-only question. State a fresh action to continue.");
        }
    }
    if !within_plan && !answered_question {
        session.plan = None;
    }
    if let Some(Utterance::Command(action)) = meaning.clone() {
        session.task = Some(Task::proposed(
            action,
            text.to_owned(),
            plain(&turn.reply).clone(),
        ));
        if let Some(task) = &mut session.task {
            revisions::begin(task);
            if task.status == TaskStatus::Revising {
                if !explain {
                    message =
                        "No complete repair yet. Preparing bounded revisions in an isolated copy."
                            .into();
                } else {
                    message.push_str("\nPreparing bounded revisions in an isolated copy.");
                }
            }
        }
        let after = fingerprint(&snapshot(&session.repository)?);
        if after != session.revision {
            invalidate(session, after);
            save(path, session)?;
            return Err("source changed during proposal; task invalidated".into());
        }
        save(path, session)?;
        if steps > 1 {
            message.push_str(&format!(
                "\n{}",
                revisions::advance(session, path, steps - 1)?
            ));
        } else if !within_plan {
            message.push_str(&format!("\n{}", status(session)));
            if session
                .task
                .as_ref()
                .is_some_and(|task| task.status == TaskStatus::Ready)
            {
                message.push_str("\nUse continue to execute the recorded proposal.");
            } else if session
                .task
                .as_ref()
                .is_some_and(|task| task.status == TaskStatus::Revising)
            {
                message.push_str("\nUse continue to try the next supported repair in scratch.");
            }
        }
    }
    Ok(RequestResult {
        message,
        meaning,
        reply: turn.reply,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    const SOURCE: &str = "pub struct Parcel;\npub struct Bundle;\npub fn deliver(value: Parcel) -> Parcel { value }\n";

    struct Fixture {
        folder: PathBuf,
        root: PathBuf,
        session: PathBuf,
    }
    impl Fixture {
        fn new(source: &str) -> Self {
            let folder = std::env::temp_dir().join(format!(
                "coding-task-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let root = folder.join("project");
            fs::create_dir_all(root.join("src")).unwrap();
            fs::write(root.join("Cargo.toml"), "[package]\nname=\"task_fixture\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[workspace]\n").unwrap();
            fs::write(root.join("src/lib.rs"), source).unwrap();
            Self {
                session: folder.join("session.json"),
                folder,
                root,
            }
        }
        fn turn(&self, steps: u8, text: &str) {
            run(&self.root, &self.session, None, false, steps, text).unwrap();
        }
        fn state(&self) -> CodingSession {
            serde_json::from_slice(&fs::read(&self.session).unwrap()).unwrap()
        }
        fn source(&self) -> String {
            fs::read_to_string(self.root.join("src/lib.rs")).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.folder);
        }
    }

    #[test]
    fn resume_followup_and_undo_use_persisted_typed_state() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename struct Parcel to Package");
        assert_eq!(f.source(), SOURCE);
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Ready);
        f.turn(2, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert_eq!(f.source(), SOURCE.replace("Parcel", "Package"));
        f.turn(2, "rename it to Delivery");
        assert_eq!(f.source(), SOURCE.replace("Parcel", "Delivery"));
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        f.turn(2, "undo that change");
        assert_eq!(f.source(), SOURCE.replace("Parcel", "Package"));
        assert!(f.state().conversation.turns.is_empty());
    }

    #[test]
    fn correction_replaces_target_and_preserves_destination() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename struct Parcel to Package");
        f.turn(1, "sorry, I meant Bundle");
        let task = f.state().task.unwrap();
        assert_eq!(task.status, TaskStatus::Ready);
        assert!(
            matches!(task.request, Action::Rename { thing: Thing::Item(ref i), name: Some(ref n), .. } if i.name == "Bundle" && n.0 == "Package")
        );
        f.turn(2, "continue");
        assert_eq!(f.source(), SOURCE.replace("Bundle", "Package"));
    }

    #[test]
    fn external_edit_invalidates_proposal_and_pronoun_context() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename struct Parcel to Package");
        let changed = format!("{SOURCE}// externally edited\n");
        fs::write(f.root.join("src/lib.rs"), &changed).unwrap();
        f.turn(2, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
        assert!(f.state().conversation.turns.is_empty());
        f.turn(2, "rename it to Delivery");
        assert_eq!(f.source(), changed);
    }

    #[test]
    fn ambiguous_or_negated_input_cannot_run_an_older_proposal() {
        let f = Fixture::new(
            "pub struct First { pub value: u32 }\npub struct Second { pub value: u32 }\n",
        );
        let before = snapshot(&f.root).unwrap();
        f.turn(2, "rename field value to count");
        f.turn(2, "continue");
        assert_eq!(snapshot(&f.root).unwrap(), before);
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename struct Parcel to Package");
        f.turn(2, "do not rename Parcel");
        f.turn(2, "continue");
        assert_eq!(f.source(), SOURCE);
    }

    #[test]
    fn failed_native_verification_restores_original_source() {
        let original = "pub struct Parcel;\npub fn broken() -> u32 { false }\n";
        let f = Fixture::new(original);
        f.turn(2, "rename struct Parcel to Package");
        assert_eq!(f.source(), original);
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Blocked);
        assert!(f.state().undo.is_empty());
    }

    #[test]
    fn modified_saved_proposal_is_not_execution_authority() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename struct Parcel to Package");
        let mut state = f.state();
        let task = state.task.as_mut().unwrap();
        if let Reply::Proposal {
            action: Action::Rename { name, .. },
            ..
        } = &mut task.events.last_mut().unwrap().result
        {
            *name = Some(Name("Unexpected".into()));
        } else {
            panic!("expected typed rename proposal");
        }
        save(&f.session, &state).unwrap();
        f.turn(2, "continue");
        assert_eq!(f.source(), SOURCE);
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
    }

    #[test]
    fn interrupted_application_is_not_replayed() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename struct Parcel to Package");
        let mut state = f.state();
        state.task.as_mut().unwrap().status = TaskStatus::Applying;
        save(&f.session, &state).unwrap();
        f.turn(2, "continue");
        assert_eq!(f.source(), SOURCE);
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
    }

    #[test]
    fn a_session_cannot_be_rebound_to_another_repository() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename struct Parcel to Package");
        let other = Fixture::new(SOURCE);
        assert!(run(&other.root, &f.session, None, false, 2, "continue").is_err());
        assert_eq!(other.source(), SOURCE);
        assert!(
            run(
                &f.root,
                &f.root.join("session.json"),
                None,
                false,
                1,
                "show task"
            )
            .is_err()
        );
    }

    #[test]
    fn cli_session_is_opt_in_and_step_budget_is_bounded() {
        use clap::Parser;
        assert!(crate::Cli::try_parse_from(["glassmind", "do", ".", "fix tests"]).is_ok());
        assert!(
            crate::Cli::try_parse_from([
                "glassmind",
                "do",
                ".",
                "--session",
                "state.json",
                "--steps",
                "1",
                "fix tests"
            ])
            .is_ok()
        );
        assert!(
            crate::Cli::try_parse_from([
                "glassmind",
                "do",
                ".",
                "--session",
                "state.json",
                "--steps",
                "9",
                "fix tests"
            ])
            .is_err()
        );
    }

    #[test]
    fn different_phrasings_preserve_the_same_resolved_action() {
        let f = Fixture::new(SOURCE);
        let mut requests = Vec::new();
        for text in [
            "rename struct Parcel to Package",
            "Could you rename the Parcel struct to Package, please?",
            "I'd like to rename Parcel to Package. Could you do that for me?",
        ] {
            f.turn(1, text);
            let task = f.state().task.unwrap();
            assert_eq!(task.status, TaskStatus::Ready, "{text}");
            let Action::Rename {
                thing: Thing::Item(item),
                name,
                ..
            } = task.request
            else {
                panic!("rename expected");
            };
            requests.push((item, name));
        }
        assert!(requests.windows(2).all(|w| w.first() == w.get(1)));
        assert_eq!(f.source(), SOURCE);
    }

    #[test]
    fn repair_records_native_test_evidence_and_undo_restores_failure() {
        use glassmind_world_rust::checks::RustCheck;
        let source = "pub fn combine(a: i32, b: i32) -> i32 { a - b }\n#[cfg(test)] mod tests { use super::*; #[test] fn behavior() { assert_eq!(combine(7,3),10); assert_eq!(combine(-5,2),-3); } }\n";
        let f = Fixture::new(source);
        f.turn(1, "fix the tests");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Ready);
        assert_eq!(f.source(), source);
        f.turn(2, "continue");
        let state = f.state();
        let task = state.task.unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(
            matches!(&task.events.last().unwrap().result, Reply::Applied { check: Some(RustCheck::Test), problems, .. } if problems.is_empty())
        );
        assert_ne!(f.source(), source);
        f.turn(2, "undo");
        assert_eq!(f.source(), source);
    }

    #[test]
    fn undo_preserves_edits_made_after_the_verified_change() {
        let f = Fixture::new(SOURCE);
        f.turn(2, "rename Parcel to Package");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        let updated = format!("{}// my later change\n", f.source());
        fs::write(f.root.join("src/lib.rs"), &updated).unwrap();
        f.turn(2, "undo");
        assert_eq!(f.source(), updated);
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
    }

    #[test]
    fn read_only_question_preserves_a_pending_task() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename Parcel to Package");
        f.turn(2, "where is Parcel defined?");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Ready);
        assert_eq!(f.source(), SOURCE);
        f.turn(2, "continue");
        assert_eq!(f.source(), SOURCE.replace("Parcel", "Package"));
    }

    #[test]
    fn session_lock_prevents_concurrent_execution_without_leaving_a_stale_lock() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename Parcel to Package");
        let held = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(f.folder.join("session.json.lock"))
            .unwrap();
        held.try_lock().unwrap();
        assert!(run(&f.root, &f.session, None, false, 2, "continue").is_err());
        assert_eq!(f.source(), SOURCE);
        held.unlock().unwrap();
        drop(held);
        f.turn(2, "continue");
        assert_eq!(f.source(), SOURCE.replace("Parcel", "Package"));
    }

    #[test]
    fn implementation_without_executed_tests_does_not_fall_through_to_repair() {
        let f = Fixture::new("pub fn total(xs: &[i32]) -> i32 { todo!() }\n");
        let before = snapshot(&f.root).unwrap();
        f.turn(2, "implement total");
        assert_eq!(snapshot(&f.root).unwrap(), before);
        let task = f.state().task.unwrap();
        assert_eq!(task.status, TaskStatus::Blocked);
        assert!(matches!(task.request, Action::Implement { .. }));
        assert!(task.revision.is_none());
        assert!(
            matches!(&task.events.last().unwrap().result, Reply::Trouble(glassmind_meaning::Trouble::ActionFailed { reason }) if reason.contains("nonzero executed failing baseline tests required"))
        );
    }
    #[test]
    fn ordered_plan_rebinds_each_target_and_checks_the_final_result() {
        let f = Fixture::new(SOURCE);
        f.turn(
            8,
            "rename Parcel to Package; then rename it to Delivery; then check the tests",
        );
        let state = f.state();
        let plan = state.plan.unwrap();
        assert_eq!(plan.status, TaskStatus::Completed);
        assert_eq!(plan.current, 3);
        assert_eq!(f.source(), SOURCE.replace("Parcel", "Delivery"));
        assert!(
            matches!(&plan.steps[1].task.as_ref().unwrap().request, Action::Rename { thing: Thing::Item(item), .. } if item.name == "Package")
        );
        assert!(
            matches!(plan.steps[2].result, Some(Reply::Checked { ref problems, .. }) if problems.is_empty())
        );
    }

    #[test]
    fn plan_budget_resumes_each_stage_across_sessions() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename Parcel to Package then rename it to Delivery");
        assert_eq!(f.state().plan.unwrap().current, 0);
        assert_eq!(f.source(), SOURCE);
        f.turn(1, "continue");
        assert_eq!(f.state().plan.unwrap().current, 1);
        assert_eq!(f.source(), SOURCE.replace("Parcel", "Package"));
        f.turn(1, "continue");
        assert_eq!(f.state().plan.unwrap().current, 1);
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Ready);
        f.turn(1, "continue");
        assert_eq!(f.state().plan.unwrap().status, TaskStatus::Completed);
        assert_eq!(f.source(), SOURCE.replace("Parcel", "Delivery"));
        let last = f.source();
        f.turn(8, "continue");
        assert_eq!(f.source(), last);
    }

    #[test]
    fn entire_plan_is_understood_before_any_edit() {
        for tail in [
            "delete every file",
            "if I approve rename Bundle to Other",
            "don't fix the tests",
        ] {
            let f = Fixture::new(SOURCE);
            f.turn(8, &format!("rename Parcel to Package; then {tail}"));
            assert_eq!(f.source(), SOURCE, "{tail}");
            assert!(f.state().plan.is_none());
        }
    }

    #[test]
    fn a_failed_check_stops_before_later_edits() {
        let source = format!("{SOURCE}#[test] fn broken() {{ assert_eq!(2, 3); }}\n");
        let f = Fixture::new(&source);
        f.turn(8, "check the tests then rename Parcel to Package");
        let plan = f.state().plan.unwrap();
        assert_eq!(plan.status, TaskStatus::Blocked);
        assert_eq!(plan.current, 0);
        assert!(plan.steps[1].task.is_none());
        f.turn(8, "continue");
        assert_eq!(f.source(), source);
    }

    #[test]
    fn a_rejected_edit_stops_the_plan_and_restores_source() {
        let source = format!("{SOURCE}pub fn broken() -> bool {{ 7 }}\n");
        let f = Fixture::new(&source);
        f.turn(
            8,
            "rename Parcel to Package; then rename Bundle to Shipment",
        );
        let plan = f.state().plan.unwrap();
        assert_eq!(plan.status, TaskStatus::Blocked);
        assert_eq!(plan.current, 0);
        assert_eq!(f.source(), source);
        assert!(plan.steps[1].task.is_none());
    }

    #[test]
    fn external_change_invalidates_all_remaining_plan_steps() {
        let f = Fixture::new(SOURCE);
        f.turn(2, "rename Parcel to Package then rename it to Delivery");
        assert_eq!(f.state().plan.unwrap().current, 1);
        let external = format!("{}// outside change\n", f.source());
        fs::write(f.root.join("src/lib.rs"), &external).unwrap();
        f.turn(8, "continue");
        assert_eq!(f.state().plan.unwrap().status, TaskStatus::Stale);
        assert_eq!(f.source(), external);
    }

    #[test]
    fn plan_delimiters_do_not_consume_quoted_text_or_identifiers() {
        assert!(
            plans::clauses("rename `then` to `after`")
                .unwrap()
                .is_none()
        );
        assert!(
            plans::clauses("rename `name; then other` to `after`")
                .unwrap()
                .is_none()
        );
        assert!(plans::clauses("rename `unclosed then other").is_err());
        assert!(plans::clauses("a then b then c then d then e").is_err());
        let f = Fixture::new("pub struct then;\n");
        f.turn(2, "rename struct then to Later");
        assert_eq!(f.source(), "pub struct Later;\n");
        assert!(f.state().plan.is_none());
    }

    #[test]
    fn an_unsupported_replacement_cancels_a_whole_pending_plan() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename Parcel to Package then rename Bundle to Shipment");
        f.turn(2, "add public field count: u32 to Parcel");
        assert!(f.state().plan.is_none());
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Cancelled);
        f.turn(8, "continue");
        assert_eq!(f.source(), SOURCE);
    }
    #[test]
    fn repair_then_rename_uses_verified_code_and_undo_keeps_the_repair() {
        let source = "pub fn combine(a: i32, b: i32) -> i32 { a - b }\n#[cfg(test)] mod tests { use super::*; #[test] fn behavior() { assert_eq!(combine(7,3),10); assert_eq!(combine(-5,2),-3); } }\n";
        let f = Fixture::new(source);
        f.turn(
            8,
            "fix the tests; then rename function combine to sum; then check the tests",
        );
        let plan = f.state().plan.unwrap();
        assert_eq!(plan.status, TaskStatus::Completed);
        assert_eq!(plan.current, 3);
        assert!(f.source().contains("fn sum("));
        f.turn(8, "undo");
        assert!(f.source().contains("fn combine("));
        assert!(!f.source().contains("a - b"));
        let repaired = f.source();
        f.turn(8, "continue");
        assert_eq!(f.source(), repaired);
    }

    #[test]
    fn cancelled_and_interrupted_plans_cannot_replay_a_ready_edit() {
        for stop in ["undo", "rename Parcel to Box then", "cancel task"] {
            let f = Fixture::new(SOURCE);
            f.turn(1, "rename Parcel to Package then rename it to Delivery");
            f.turn(1, stop);
            f.turn(8, "continue");
            assert_eq!(f.source(), SOURCE, "{stop}");
            assert_eq!(f.state().plan.unwrap().status, TaskStatus::Cancelled);
        }
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename Parcel to Package then rename it to Delivery");
        let mut state = f.state();
        state.plan.as_mut().unwrap().status = TaskStatus::Applying;
        save(&f.session, &state).unwrap();
        f.turn(8, "continue");
        assert_eq!(f.source(), SOURCE);
        assert_eq!(f.state().plan.unwrap().status, TaskStatus::Stale);
    }

    #[test]
    fn question_during_plan_preserves_the_next_verified_stage() {
        let f = Fixture::new(SOURCE);
        f.turn(1, "rename Parcel to Package then rename it to Delivery");
        f.turn(1, "where is Parcel?");
        f.turn(8, "continue");
        assert_eq!(f.state().plan.unwrap().status, TaskStatus::Completed);
        assert_eq!(f.source(), SOURCE.replace("Parcel", "Delivery"));
    }
    #[test]
    fn named_dependencies_and_courtesy_use_the_shared_language() {
        let f = Fixture::new(SOURCE);
        f.turn(8, "Could you rename the Parcel struct to Package, please?; then rename Package to Shipment; then where is Shipment?");
        let plan = f.state().plan.unwrap();
        assert_eq!(plan.status, TaskStatus::Completed);
        assert_eq!(plan.current, 3);
        assert_eq!(f.source(), SOURCE.replace("Parcel", "Shipment"));
        assert!(matches!(
            plan.steps.last().unwrap().result,
            Some(Reply::Location(_))
        ));
    }
    const CHOICES_SOURCE: &str = "pub struct First { pub value: u32 }\npub struct Second { pub value: u32 }\npub fn read_first(x: First) -> u32 { x.value }\npub fn read_second(x: Second) -> u32 { x.value }\n";

    #[test]
    fn explicit_choice_after_restart_renames_only_the_selected_owner() {
        let f = Fixture::new(CHOICES_SOURCE);
        f.turn(1, "rename field value to count");
        let state = f.state();
        assert_eq!(state.task.as_ref().unwrap().status, TaskStatus::Blocked);
        assert_eq!(state.task.unwrap().choices.unwrap().candidates.len(), 2);
        assert_eq!(f.source(), CHOICES_SOURCE);
        f.turn(1, "use the second one");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Ready);
        assert_eq!(f.source(), CHOICES_SOURCE);
        f.turn(2, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert!(f.source().contains("First { pub value: u32 }"));
        assert!(f.source().contains("Second { pub count: u32 }"));
        assert!(
            f.source()
                .contains("read_first(x: First) -> u32 { x.value }")
        );
        assert!(
            f.source()
                .contains("read_second(x: Second) -> u32 { x.count }")
        );
        f.turn(2, "undo");
        assert_eq!(f.source(), CHOICES_SOURCE);
    }

    #[test]
    fn another_choice_replaces_the_preview_without_losing_the_destination() {
        let f = Fixture::new(CHOICES_SOURCE);
        f.turn(1, "rename field value to count");
        f.turn(1, "choose First::value");
        f.turn(1, "use the other one");
        let task = f.state().task.unwrap();
        assert_eq!(task.choices.unwrap().selected, Some(1));
        assert!(matches!(task.request, Action::Rename { name: Some(ref n), .. } if n.0 == "count"));
        assert_eq!(f.source(), CHOICES_SOURCE);
        f.turn(2, "continue");
        assert!(f.source().contains("Second { pub count: u32 }"));
        assert!(f.source().contains("First { pub value: u32 }"));
    }

    #[test]
    fn a_plan_can_resume_after_an_explicit_target_choice() {
        let f = Fixture::new(CHOICES_SOURCE);
        f.turn(8, "rename field value to count; then check the tests");
        assert_eq!(f.state().plan.unwrap().status, TaskStatus::Blocked);
        f.turn(8, "select Second.value");
        let plan = f.state().plan.unwrap();
        assert_eq!(plan.current, 2);
        assert_eq!(plan.status, TaskStatus::Completed);
        assert!(f.source().contains("Second { pub count: u32 }"));
        assert!(f.source().contains("First { pub value: u32 }"));
    }

    #[test]
    fn ambiguous_and_out_of_range_choices_never_select_a_default() {
        let f = Fixture::new(CHOICES_SOURCE);
        f.turn(1, "rename field value to count");
        for text in [
            "other one",
            "choose 0",
            "option 9",
            "choose value",
            "choose Missing::value",
        ] {
            f.turn(8, text);
            assert_eq!(f.source(), CHOICES_SOURCE, "{text}");
            assert_eq!(f.state().task.unwrap().status, TaskStatus::Blocked);
        }
        f.turn(8, "choose 1");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
    }

    #[test]
    fn selecting_a_stale_or_cancelled_choice_cannot_edit() {
        for cancel in ["cancel task", "undo"] {
            let f = Fixture::new(CHOICES_SOURCE);
            f.turn(1, "rename field value to count; then check the tests");
            f.turn(1, cancel);
            f.turn(8, "choose 1");
            f.turn(8, "continue");
            assert_eq!(f.source(), CHOICES_SOURCE, "{cancel}");
        }
        let f = Fixture::new(CHOICES_SOURCE);
        f.turn(1, "rename field value to count");
        f.turn(1, "undo");
        f.turn(8, "choose 1");
        assert_eq!(f.source(), CHOICES_SOURCE);
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Cancelled);
        let f = Fixture::new(CHOICES_SOURCE);
        f.turn(1, "rename field value to count");
        let external = format!("{CHOICES_SOURCE}// user's change\n");
        fs::write(f.root.join("src/lib.rs"), &external).unwrap();
        f.turn(8, "choose 2");
        assert_eq!(f.source(), external);
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
    }

    #[test]
    fn a_tampered_alternative_is_not_a_verified_target() {
        let f = Fixture::new(CHOICES_SOURCE);
        f.turn(1, "rename field value to count");
        let mut state = f.state();
        state
            .task
            .as_mut()
            .unwrap()
            .choices
            .as_mut()
            .unwrap()
            .candidates
            .first_mut()
            .unwrap()
            .line += 10;
        save(&f.session, &state).unwrap();
        f.turn(8, "choose 1");
        assert_eq!(f.source(), CHOICES_SOURCE);
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
    }

    #[test]
    fn a_question_preserves_choices_but_a_replacement_action_does_not() {
        let f = Fixture::new(CHOICES_SOURCE);
        f.turn(1, "rename field value to count");
        f.turn(1, "where is First?");
        assert!(f.state().task.unwrap().choices.is_some());
        f.turn(8, "use Second::value");
        assert!(f.source().contains("Second { pub count: u32 }"));
        let f = Fixture::new(CHOICES_SOURCE);
        f.turn(1, "rename field value to count");
        f.turn(1, "rename struct First to Alpha");
        f.turn(8, "choose 2");
        f.turn(8, "continue");
        assert_eq!(f.source(), CHOICES_SOURCE);
    }
    #[test]
    fn followup_after_a_selected_field_refers_to_its_new_name() {
        let f = Fixture::new(CHOICES_SOURCE);
        f.turn(1, "rename field value to count");
        f.turn(8, "choose Second::value");
        f.turn(8, "check the tests");
        f.turn(8, "rename it to amount");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert!(f.source().contains("Second { pub amount: u32 }"));
        assert!(
            f.source()
                .contains("read_second(x: Second) -> u32 { x.amount }")
        );
        assert!(f.source().contains("First { pub value: u32 }"));
    }

    #[test]
    fn selected_targets_still_roll_back_when_native_verification_fails() {
        let source = format!("{CHOICES_SOURCE}pub fn broken() -> bool {{ 17 }}\n");
        let f = Fixture::new(&source);
        f.turn(1, "rename field value to count");
        f.turn(8, "choose Second::value");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Blocked);
        assert_eq!(f.source(), source);
    }
    #[test]
    fn unsupported_replacements_cancel_unresolved_choices() {
        for text in [
            "implement total",
            "do not rename value",
            "delete every file",
            "choose 2 and remove tests",
        ] {
            let f = Fixture::new(CHOICES_SOURCE);
            f.turn(1, "rename field value to count");
            f.turn(1, text);
            f.turn(8, "choose 1");
            f.turn(8, "continue");
            assert_eq!(f.source(), CHOICES_SOURCE, "{text}");
            assert_ne!(f.state().task.unwrap().status, TaskStatus::Completed);
        }
    }

    #[test]
    fn other_requires_exactly_one_remaining_alternative() {
        let source = format!("{CHOICES_SOURCE}pub struct Third {{ pub value: u32 }}\n");
        let f = Fixture::new(&source);
        f.turn(1, "rename field value to count");
        f.turn(1, "choose 1");
        f.turn(1, "other one");
        assert_eq!(f.source(), source);
        assert_eq!(f.state().task.unwrap().choices.unwrap().selected, Some(0));
    }
    const REVISION_SOURCE: &str = "pub fn combine(a: i32, b: i32) -> i32 { let result = a - b; result += 0; result }\n#[cfg(test)] mod tests { use super::*; #[test] fn behavior() { assert_eq!(combine(7,3),10); assert_eq!(combine(-5,2),-3); } }\n";

    #[test]
    fn repair_uses_compiler_then_test_evidence_before_touching_source() {
        let f = Fixture::new(REVISION_SOURCE);
        f.turn(1, "fix the tests");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Revising);
        assert_eq!(f.source(), REVISION_SOURCE);
        f.turn(1, "continue");
        let task = f.state().task.unwrap();
        assert_eq!(task.status, TaskStatus::Revising);
        let evidence = task.revision.unwrap();
        assert_eq!(evidence.attempts.len(), 1);
        assert!(
            evidence
                .attempts
                .first()
                .unwrap()
                .after
                .iter()
                .any(|p| p.kind == glassmind_world_rust::RustKind::Test)
        );
        assert_eq!(f.source(), REVISION_SOURCE);
        f.turn(1, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Ready);
        assert_eq!(f.source(), REVISION_SOURCE);
        f.turn(1, "continue");
        let task = f.state().task.unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.revision.unwrap().attempts.len(), 2);
        assert!(f.source().contains("let mut result"));
        assert!(f.source().contains("a + b"));
        f.turn(1, "undo");
        assert_eq!(f.source(), REVISION_SOURCE);
    }

    #[test]
    fn revision_completes_within_the_default_stage_budget() {
        let f = Fixture::new(REVISION_SOURCE);
        f.turn(4, "fix the tests");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert!(f.source().contains("a + b"));
    }

    #[test]
    fn revision_can_unblock_later_plan_operations() {
        let f = Fixture::new(REVISION_SOURCE);
        f.turn(
            8,
            "fix the tests; then rename function combine to sum; then check the tests",
        );
        let plan = f.state().plan.unwrap();
        assert_eq!(plan.status, TaskStatus::Completed);
        assert_eq!(plan.current, 3);
        assert_eq!(
            plan.steps
                .first()
                .unwrap()
                .task
                .as_ref()
                .unwrap()
                .revision
                .as_ref()
                .unwrap()
                .attempts
                .len(),
            2
        );
        assert!(f.source().contains("fn sum("));
        assert!(f.source().contains("a + b"));
    }

    #[test]
    fn an_unsupported_repair_stops_without_repeating_or_editing() {
        let f = Fixture::new("pub fn broken() -> u32 { false }\n");
        let original = f.source();
        f.turn(8, "fix the tests");
        let task = f.state().task.unwrap();
        assert_eq!(task.status, TaskStatus::Blocked);
        assert!(task.revision.as_ref().unwrap().stopped.is_some());
        f.turn(8, "continue");
        assert_eq!(f.state().task.unwrap(), task);
        assert_eq!(f.source(), original);
    }

    #[test]
    fn saved_repair_changes_are_not_execution_authority() {
        let f = Fixture::new(REVISION_SOURCE);
        f.turn(2, "fix the tests");
        let mut state = f.state();
        let revision = state.task.as_mut().unwrap().revision.as_mut().unwrap();
        revision
            .attempts
            .first_mut()
            .unwrap()
            .changes
            .first_mut()
            .unwrap()
            .new = "tampered".into();
        save(&f.session, &state).unwrap();
        f.turn(8, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Blocked);
        assert_eq!(f.source(), REVISION_SOURCE);
    }

    #[test]
    fn revision_does_not_overwrite_external_changes_or_resume_after_cancel() {
        for control in ["cancel task", "undo", "choose 1", "implement other"] {
            let f = Fixture::new(REVISION_SOURCE);
            f.turn(2, "fix the tests");
            f.turn(1, control);
            f.turn(8, "continue");
            assert_eq!(f.source(), REVISION_SOURCE, "{control}");
            let task = f.state().task.unwrap();
            if control == "implement other" {
                assert_eq!(task.status, TaskStatus::Blocked);
                assert!(matches!(task.request, Action::Implement { .. }));
                assert!(task.revision.is_none());
            } else {
                assert_eq!(task.status, TaskStatus::Cancelled);
            }
        }
        let f = Fixture::new(REVISION_SOURCE);
        f.turn(2, "fix the tests");
        let changed = format!("{REVISION_SOURCE}// external change\n");
        fs::write(f.root.join("src/lib.rs"), &changed).unwrap();
        f.turn(8, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
        assert_eq!(f.source(), changed);
    }
    #[test]
    fn derived_trait_then_test_repair_uses_the_new_diagnostics() {
        let source = "pub struct Marker;\npub fn show() -> String { format!(\"{:?}\", Marker) }\npub fn combine(a: i32, b: i32) -> i32 { a - b }\n#[cfg(test)] mod tests { use super::*; #[test] fn behavior() { assert_eq!(combine(7,3),10); assert_eq!(combine(-5,2),-3); } }\n";
        let f = Fixture::new(source);
        f.turn(8, "fix the tests");
        let task = f.state().task.unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.revision.unwrap().attempts.len(), 2);
        assert!(f.source().contains("a + b"));
        assert!(f.source().contains("Debug"));
        f.turn(1, "undo");
        assert_eq!(f.source(), source);
    }

    #[test]
    fn a_composed_repair_is_reproved_before_application() {
        let f = Fixture::new(REVISION_SOURCE);
        f.turn(3, "fix the tests");
        let mut state = f.state();
        assert_eq!(state.task.as_ref().unwrap().status, TaskStatus::Ready);
        let Reply::Remedy { changes, .. } = &mut state
            .task
            .as_mut()
            .unwrap()
            .events
            .last_mut()
            .unwrap()
            .result
        else {
            panic!("composed remedy missing")
        };
        changes
            .first_mut()
            .unwrap()
            .new
            .push_str("// forged extra change\n");
        save(&f.session, &state).unwrap();
        f.turn(8, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
        assert_eq!(f.source(), REVISION_SOURCE);
    }

    #[test]
    fn revision_evidence_ignores_process_prose_but_rechecks_code() {
        let f = Fixture::new(REVISION_SOURCE);
        f.turn(2, "fix the tests");
        let mut state = f.state();
        let attempt = state
            .task
            .as_mut()
            .unwrap()
            .revision
            .as_mut()
            .unwrap()
            .attempts
            .first_mut()
            .unwrap();
        for item in attempt.before.iter_mut().chain(&mut attempt.after) {
            item.detail = Some("diagnostic process ID changed".into());
        }
        save(&f.session, &state).unwrap();
        f.turn(8, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert!(f.source().contains("a + b"));
    }

    #[test]
    fn an_interrupted_revision_is_never_replayed_as_authorization() {
        let f = Fixture::new(REVISION_SOURCE);
        f.turn(2, "fix the tests");
        let mut state = f.state();
        state.task.as_mut().unwrap().status = TaskStatus::Applying;
        save(&f.session, &state).unwrap();
        f.turn(8, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
        assert_eq!(f.source(), REVISION_SOURCE);
    }
    const IMPLEMENT_SOURCE: &str = "pub fn total(xs: &[i32]) -> i32 { todo!() }\n#[cfg(test)] mod tests { use super::*; #[test] fn behavior() { assert_eq!(total(&[4,-9,7]),2); assert_eq!(total(&[-3,-5]),-8); assert_eq!(total(&[]),0); } }\n";

    #[test]
    fn implementation_preview_resumes_with_fresh_native_proof_and_undo() {
        let f = Fixture::new(IMPLEMENT_SOURCE);
        f.turn(1, "please implement total");
        let task = f.state().task.unwrap();
        assert_eq!(task.status, TaskStatus::Ready);
        assert!(
            matches!(task.request, Action::Implement { thing: Thing::Item(ref i) } if i.name == "total")
        );
        assert!(
            matches!(task.proposal(), Some(Reply::Patch { check: glassmind_world_rust::checks::RustCheck::Test, trials, .. }) if *trials > 1)
        );
        assert_eq!(f.source(), IMPLEMENT_SOURCE);
        f.turn(1, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert!(!f.source().contains("todo!()"));
        assert!(f.source().contains("assert_eq!(total(&[4,-9,7]),2)"));
        f.turn(1, "undo");
        assert_eq!(f.source(), IMPLEMENT_SOURCE);
    }

    #[test]
    fn an_implementation_plan_can_rename_its_result_and_check_it() {
        let f = Fixture::new(IMPLEMENT_SOURCE);
        f.turn(
            8,
            "implement total; then rename it to accumulated; then check the tests",
        );
        let plan = f.state().plan.unwrap();
        assert_eq!(plan.status, TaskStatus::Completed);
        assert_eq!(plan.current, 3);
        assert!(matches!(
            plan.steps.first().unwrap().task.as_ref().unwrap().request,
            Action::Implement { .. }
        ));
        assert!(f.source().contains("fn accumulated("));
        assert!(!f.source().contains("todo!()"));
    }

    #[test]
    fn contextual_implementation_uses_the_previous_unique_function() {
        let f = Fixture::new(IMPLEMENT_SOURCE);
        f.turn(1, "where is `total`?");
        f.turn(2, "fill it in");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert!(!f.source().contains("todo!()"));
    }

    #[test]
    fn implementation_corrections_retain_the_action_kind() {
        let source = format!("pub fn unused(xs: &[i32]) -> i32 {{ todo!() }}\n{IMPLEMENT_SOURCE}");
        let f = Fixture::new(&source);
        f.turn(1, "implement unused");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Blocked);
        f.turn(2, "sorry, I meant `total`");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert!(
            f.source()
                .contains("fn unused(xs: &[i32]) -> i32 { todo!() }")
        );
        assert!(
            !f.source()
                .contains("fn total(xs: &[i32]) -> i32 { todo!() }")
        );
    }

    #[test]
    fn unexecuted_and_ambiguous_implementations_remain_blocked() {
        let source = "pub fn total(xs: &[i32]) -> i32 { todo!() }\n#[cfg(test)] mod tests { #[test] fn unrelated() { assert_eq!(1,2); } }\n";
        let f = Fixture::new(source);
        f.turn(2, "implement total");
        let task = f.state().task.unwrap();
        assert_eq!(task.status, TaskStatus::Blocked);
        assert!(
            matches!(&task.events.last().unwrap().result, Reply::Trouble(glassmind_meaning::Trouble::ActionFailed { reason }) if reason.contains("execution was not witnessed"))
        );
        assert_eq!(f.source(), source);
        let source = "pub fn total(xs: &[i32]) -> i32 { todo!() }\n#[cfg(test)] mod tests { use super::*; #[test] fn weak() { assert_eq!(total(&[]),0); } }\n";
        let f = Fixture::new(source);
        f.turn(2, "implement total");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Blocked);
        assert_eq!(f.source(), source);
    }

    #[test]
    fn tampered_implementation_patch_cannot_bypass_native_proof() {
        let f = Fixture::new(IMPLEMENT_SOURCE);
        f.turn(1, "implement total");
        let mut state = f.state();
        let Reply::Patch { changes, .. } = &mut state
            .task
            .as_mut()
            .unwrap()
            .events
            .last_mut()
            .unwrap()
            .result
        else {
            panic!("patch missing")
        };
        changes
            .first_mut()
            .unwrap()
            .new
            .push_str("// unproved change\n");
        save(&f.session, &state).unwrap();
        f.turn(2, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
        assert_eq!(f.source(), IMPLEMENT_SOURCE);
    }

    #[test]
    fn an_implementation_plan_rejects_unsupported_tail_before_synthesis() {
        let f = Fixture::new(IMPLEMENT_SOURCE);
        f.turn(8, "implement total; then delete every file");
        assert!(f.state().task.is_none());
        assert!(f.state().plan.is_none());
        assert_eq!(f.source(), IMPLEMENT_SOURCE);
    }
    #[test]
    fn implementation_does_not_bypass_rename_collision_checks() {
        let f = Fixture::new(IMPLEMENT_SOURCE);
        f.turn(8, "implement total; then rename it to sum");
        let state = f.state();
        assert_eq!(state.plan.as_ref().unwrap().current, 1);
        assert_eq!(state.plan.unwrap().status, TaskStatus::Blocked);
        assert!(f.source().contains("fn total("));
        assert!(f.source().contains(".sum()"));
        assert!(!f.source().contains("todo!()"));
    }
    const FIELD_SOURCE: &str = "pub struct Parcel { pub value: u8 }\npub fn make() -> Parcel { Parcel { value: 9 } }\n#[cfg(test)] mod tests { use super::*; #[test] fn keeps_value() { assert_eq!(make().value, 9); } }\n";
    const FIELD_REQUEST: &str =
        "Add public field weight: u16 to Parcel; use 7 for all existing constructors";

    #[test]
    fn field_specification_survives_restart_apply_and_undo() {
        let f = Fixture::new(FIELD_SOURCE);
        f.turn(1, FIELD_REQUEST);
        let task = f.state().task.unwrap();
        assert_eq!(task.status, TaskStatus::Ready);
        assert!(
            matches!(task.request, Action::AddMember { member, .. } if member.name == "weight" && member.ty == "u16" && member.default == "7")
        );
        assert_eq!(f.source(), FIELD_SOURCE);
        f.turn(1, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert!(f.source().contains("pub weight: u16"));
        assert!(f.source().contains("weight: 7"));
        f.turn(1, "undo");
        assert_eq!(f.source(), FIELD_SOURCE);
    }

    #[test]
    fn field_plan_groups_specification_clauses_before_executing() {
        // Existing type renaming supports inert test attributes, but deliberately
        // rejects cfg modules and opaque assertion macros. Exercise its supported
        // subset here; the separate test below retains the rejection boundary.
        let source = "pub struct Parcel { pub value: u8 }\npub fn make() -> Parcel { Parcel { value: 9 } }\n#[test] fn keeps_value() -> Result<(), String> { if make().value == 9 { Ok(()) } else { Err(String::from(\"value changed\")) } }\n";
        let f = Fixture::new(source);
        f.turn(
            8,
            &format!("{FIELD_REQUEST}; then rename it to Packet; then check the tests"),
        );
        let state = f.state();
        let plan = state.plan.unwrap();
        assert_eq!(plan.status, TaskStatus::Completed);
        assert_eq!(plan.current, 3);
        assert!(f.source().contains("struct Packet"));
        assert!(f.source().contains("weight: 7"));
    }

    #[test]
    fn field_owner_correction_retains_type_and_default() {
        let source = format!("pub struct Other {{ pub value: u8 }}\n{FIELD_SOURCE}");
        let f = Fixture::new(&source);
        f.turn(1, FIELD_REQUEST);
        f.turn(2, "sorry, I meant `Other`");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert!(f.source().contains("pub weight: u16"));
        assert!(f.source().contains("struct Parcel { pub value: u8 }"));
    }

    #[test]
    fn field_followup_uses_the_prior_struct() {
        let f = Fixture::new(FIELD_SOURCE);
        f.turn(1, "where is Parcel?");
        f.turn(
            2,
            "Add pub ready: bool to it. Use true for every existing constructor.",
        );
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Completed);
        assert!(f.source().contains("pub ready: bool"));
        assert!(f.source().contains("ready: true"));
    }

    #[test]
    fn unsupported_field_constraints_never_apply_a_partial_plan() {
        for suffix in [
            "; use 8 for all existing constructors",
            "; then delete every file",
            "; then do not add the field",
        ] {
            let f = Fixture::new(FIELD_SOURCE);
            f.turn(8, &format!("{FIELD_REQUEST}{suffix}"));
            assert_eq!(f.source(), FIELD_SOURCE);
            assert!(f.state().plan.is_none());
        }
        let f = Fixture::new(FIELD_SOURCE);
        f.turn(2, "Add pub weight: u16 to Parcel");
        assert_eq!(f.source(), FIELD_SOURCE);
    }

    #[test]
    fn field_patches_are_reproved_and_external_changes_invalidate_them() {
        let f = Fixture::new(FIELD_SOURCE);
        f.turn(1, FIELD_REQUEST);
        let mut state = f.state();
        let Reply::Patch { changes, .. } = &mut state
            .task
            .as_mut()
            .unwrap()
            .events
            .last_mut()
            .unwrap()
            .result
        else {
            panic!("missing patch")
        };
        changes.first_mut().unwrap().new.push_str("// unproved");
        save(&f.session, &state).unwrap();
        f.turn(2, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
        assert_eq!(f.source(), FIELD_SOURCE);
        let f = Fixture::new(FIELD_SOURCE);
        f.turn(1, FIELD_REQUEST);
        let changed = format!("{FIELD_SOURCE}// external\n");
        fs::write(f.root.join("src/lib.rs"), &changed).unwrap();
        f.turn(2, "continue");
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Stale);
        assert_eq!(f.source(), changed);
    }
    #[test]
    fn invalid_field_type_default_and_failed_tests_leave_source_intact() {
        let f = Fixture::new(FIELD_SOURCE);
        f.turn(
            2,
            "Add pub code: u8 to Parcel. Use 999 for all existing constructors.",
        );
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Blocked);
        assert_eq!(f.source(), FIELD_SOURCE);
        let failed = FIELD_SOURCE.replace("make().value, 9", "make().value, 8");
        let f = Fixture::new(&failed);
        f.turn(2, FIELD_REQUEST);
        assert_eq!(f.state().task.unwrap().status, TaskStatus::Blocked);
        assert_eq!(f.source(), failed);
    }

    #[test]
    fn field_plan_does_not_bypass_type_rename_proof_limits() {
        let f = Fixture::new(FIELD_SOURCE);
        f.turn(8, &format!("{FIELD_REQUEST}; then rename it to Packet"));
        let plan = f.state().plan.unwrap();
        assert_eq!(plan.status, TaskStatus::Blocked);
        assert_eq!(plan.current, 1);
        assert!(f.source().contains("struct Parcel"));
        assert!(f.source().contains("weight: 7"));
    }
}
