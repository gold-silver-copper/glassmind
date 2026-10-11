//! Persistent coding tasks using the existing meaning, dialogue and Rust verifier.
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
        (a @ Reply::Proposal { .. }, b @ Reply::Proposal { .. }) => a == b,
        _ => false,
    }
}

fn say(system: &System, reply: &Reply<Rust>) -> String {
    system
        .english
        .say(reply)
        .map_or_else(|| format!("{reply:?}"), |s| s.text)
}

fn status(session: &CodingSession) -> String {
    session.task.as_ref().map_or_else(
        || "No coding task is recorded.".into(),
        |t| {
            format!(
                "Task {:?}: {}\n{} recorded stages.",
                t.status,
                t.original_text,
                t.events.len()
            )
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
    let refreshed = match system.world.propose(&request, &mut Vec::new()) {
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
        let mut request = task.request.clone();
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
    Ok(format!("{spoken}\n{}", status(session)))
}

fn undo(session: &mut CodingSession, path: &Path) -> Result<String, String> {
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
            undo: BTreeMap::new(),
        },
        Err(e) => return Err(e.to_string()),
    };
    if session.version != 1 || session.repository != root {
        return Err("session version or repository does not match".into());
    }
    let interrupted = session
        .task
        .as_ref()
        .is_some_and(|t| t.status == TaskStatus::Applying);
    let control = text
        .trim()
        .trim_end_matches(['.', '!'])
        .to_ascii_lowercase();
    let message = if session.revision != revision || interrupted {
        invalidate(&mut session, revision);
        "Source changed or execution was interrupted. Context invalidated; repeat a fresh explicit request.".into()
    } else {
        match control.as_str() {
            "show task" | "task status" => status(&session),
            "continue" | "continue task" | "apply it" | "yes" | "yes please" => {
                apply(&mut session, &path)?
            }
            "undo" | "undo that change" | "undo the last change" => undo(&mut session, &path)?,
            "cancel task" | "stop" | "no" => {
                if let Some(task) = &mut session.task {
                    task.status = TaskStatus::Cancelled;
                }
                session.conversation.turns.clear();
                "Task cancelled; no pending action will run.".into()
            }
            _ if glassmind_english_rust::implementation_grammar::request(text).is_some()
                || glassmind_english_rust::add_fields::request(text).is_some() =>
            {
                if let Some(task) = &mut session.task
                    && task.status == TaskStatus::Ready
                {
                    task.status = TaskStatus::Cancelled;
                }
                "This action is not yet supported in a persistent task. Use do without --session for function implementation or adding fields.".into()
            }
            _ => {
                // New input may correct the previous meaning, but cannot consent
                // to a proposal by a path outside the journalled task executor.
                for turn in &mut session.conversation.turns {
                    if matches!(
                        plain(&turn.reply),
                        Reply::Proposal { .. } | Reply::Remedy { .. } | Reply::Clarify { .. }
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
                if !matches!(
                    meaning,
                    Some(Utterance::Ask(_) | Utterance::AskRecall | Utterance::AskReason)
                ) && let Some(task) = &mut session.task
                    && task.status == TaskStatus::Ready
                {
                    task.status = TaskStatus::Cancelled;
                }
                if let Some(Utterance::Command(action)) = meaning {
                    session.task = Some(Task::proposed(
                        action,
                        text.to_owned(),
                        plain(&turn.reply).clone(),
                    ));
                    let after = fingerprint(&snapshot(&root)?);
                    if after != session.revision {
                        invalidate(&mut session, after);
                        save(&path, &session)?;
                        return Err("source changed during proposal; task invalidated".into());
                    }
                    save(&path, &session)?;
                    if steps > 1 {
                        message.push_str(&format!("\n{}", apply(&mut session, &path)?));
                    } else {
                        message.push_str(&format!(
                            "\n{}\nUse continue to execute the recorded proposal.",
                            status(&session)
                        ));
                    }
                }
                message
            }
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
                "3",
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
    fn unsupported_task_actions_do_not_fall_through_to_repair() {
        let f = Fixture::new("pub fn total(xs: &[i32]) -> i32 { todo!() }\n");
        let before = snapshot(&f.root).unwrap();
        f.turn(2, "implement total");
        assert_eq!(snapshot(&f.root).unwrap(), before);
        assert!(f.state().task.is_none());
    }
}
