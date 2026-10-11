//! Showing proposed edits and applying ask renames through the verified world.

use glassmind_answer::{Change, Response};

/// The changes a reply proposes, if it proposes an edit.
pub fn changes(response: &Response) -> Option<Vec<Change>> {
    match response {
        Response::Edit { changes } => Some(changes.iter().cloned().collect()),
        _ => None,
    }
}

/// One line per change: where, and what becomes what.
pub fn describe(changes: &[Change]) -> String {
    changes
        .iter()
        .map(|c| {
            format!(
                "  {}:{}:{}  `{}` -> `{}`",
                c.file,
                c.line,
                c.column + 1,
                c.old,
                c.new
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Apply only the rename proposals historically supported by `ask --apply`.
/// The world owns rule gating, stale checks, cargo verification and rollback.
pub fn apply_rename(
    system: &crate::System,
    session: &mut glassmind_mind::Session<glassmind_world_rust::Rust>,
    turn: &mut glassmind_mind::Turn<glassmind_world_rust::Rust>,
) -> Result<(), String> {
    use glassmind_meaning::Reply;
    use glassmind_meaning::interface::{Language as _, World as _};
    use glassmind_mind::Progress;

    if changes(&crate::answer::response(&turn.reply)).is_none() {
        return Ok(());
    }
    let mut fired = Vec::new();
    let result = system.world.apply(&turn.reply, &mut fired);
    let error = result
        .as_ref()
        .err()
        .map(|e| format!("could not make and verify the changes: {e:?}"));
    let reply = result.unwrap_or_else(Reply::Trouble);
    turn.progress = if error.is_some() {
        Progress::Action
    } else {
        Progress::Done
    };
    // Keep any assumptions visible while replacing only the proposed action.
    replace_proposal(&mut turn.reply, reply);
    let said = system.english.say(&turn.reply);
    turn.spoken = said.as_ref().map_or_else(
        || format!("(I have no words for this reply: {:?})", turn.reply),
        |said| said.text.clone(),
    );
    turn.trace.reply = Some(turn.reply.clone());
    turn.trace.reply_rules = said.map_or_else(Vec::new, |said| said.rules);
    if let Some(reading) = turn
        .trace
        .chosen
        .and_then(|i| turn.trace.readings.get_mut(i))
    {
        reading.world_rules.extend(fired);
    }
    if let Some(record) = session.turns.last_mut() {
        record.reply = turn.reply.clone();
        record.spoken = turn.spoken.clone();
    }
    match error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn replace_proposal(
    proposal: &mut glassmind_meaning::Reply<glassmind_world_rust::Rust>,
    result: glassmind_meaning::Reply<glassmind_world_rust::Rust>,
) {
    match proposal {
        glassmind_meaning::Reply::Assuming { reply, .. } => replace_proposal(reply, result),
        _ => *proposal = result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::System;
    use glassmind_meaning::{Reply, Trouble};
    use glassmind_mind::Session;
    use glassmind_rules::Switches;
    use glassmind_world_rust::checks::{RustChange, RustCheck};
    use std::path::PathBuf;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new(name: &str, files: &[(&str, &str)]) -> Self {
            let root =
                std::env::temp_dir().join(format!("askverify-{name}-{}", std::process::id()));
            std::fs::create_dir_all(&root).unwrap();
            for (file, text) in files {
                let path = root.join(file);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, text).unwrap();
            }
            Self(root)
        }
        fn system(&self) -> System {
            System::open(&[("public".into(), self.0.clone())], Switches::all_on())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn stale_multifile_proposal_preserves_every_current_byte() {
        let first = "pub fn weigh() -> u32 { 7 }\n";
        let second = "pub fn shipment() -> u32 { weigh() }\n";
        let fixture = Fixture::new("stale", &[("a.rs", first), ("b.rs", second)]);
        let system = fixture.system();
        let mut session = Session::new(system.source_refs());
        let mut turn = system.respond(&mut session, "rename function weigh to measure");
        assert_eq!(
            changes(&crate::answer::response(&turn.reply))
                .unwrap()
                .len(),
            2
        );
        let stale = "pub fn shipment() -> u32 { other() }\n";
        std::fs::write(fixture.0.join("b.rs"), stale).unwrap();
        assert!(apply_rename(&system, &mut session, &mut turn).is_err());
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("a.rs")).unwrap(),
            first
        );
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("b.rs")).unwrap(),
            stale
        );
        assert!(matches!(turn.reply, Reply::Trouble(_)));
        assert_eq!(turn.trace.reply, Some(turn.reply.clone()));
        assert_eq!(session.turns.last().unwrap().reply, turn.reply);
    }

    #[test]
    fn nested_assumptions_keep_verified_result_and_clear_pending() {
        let fixture = Fixture::new(
            "assuming",
            &[
                (
                    "Cargo.toml",
                    include_str!("../../../tools/coding-askverify/fixtures/bounded/Cargo.toml"),
                ),
                (
                    "src/lib.rs",
                    include_str!("../../../tools/coding-askverify/fixtures/bounded/src/lib.rs"),
                ),
            ],
        );
        let system = fixture.system();
        let mut session = Session::new(system.source_refs());
        let mut turn = system.respond(&mut session, "rename struct Parcel to Package");
        assert!(changes(&crate::answer::response(&turn.reply)).is_some());
        let reading = session.turns.last().unwrap().meaning.clone().unwrap();
        for _ in 0..2 {
            turn.reply = Reply::Assuming {
                reading: reading.clone(),
                reply: Box::new(turn.reply),
            };
        }
        apply_rename(&system, &mut session, &mut turn).unwrap();
        let Reply::Assuming { reply, .. } = &turn.reply else {
            panic!("lost outer assumption")
        };
        let Reply::Assuming { reply, .. } = reply.as_ref() else {
            panic!("lost inner assumption")
        };
        assert!(
            matches!(reply.as_ref(), Reply::Applied { check: Some(RustCheck::Build), problems, .. } if problems.is_empty())
        );
        assert!(session.pending().is_none());
        assert_eq!(turn.trace.reply, Some(turn.reply.clone()));
        assert_eq!(session.turns.last().unwrap().spoken, turn.spoken);
        assert!(
            turn.rules()
                .iter()
                .any(|rule| rule.id == "rust.apply.changes")
        );
        let yes = system.respond(&mut session, "yes");
        assert!(matches!(yes.reply, Reply::Trouble(Trouble::NothingPending)));
    }

    #[test]
    fn remedies_are_not_newly_auto_applied() {
        let original = "pub fn weigh() -> u32 { 7 }\n";
        let fixture = Fixture::new("remedy", &[("lib.rs", original)]);
        let system = fixture.system();
        let mut session = Session::new(system.source_refs());
        let mut turn = system.respond(&mut session, "rename function weigh to measure");
        let Reply::Proposal {
            action:
                glassmind_meaning::Action::Rename {
                    thing: glassmind_meaning::Thing::Item(item),
                    ..
                },
            ..
        } = &turn.reply
        else {
            panic!("expected proposal")
        };
        let item = item.clone();
        for left in [vec![], vec![item]] {
            turn.reply = Reply::Remedy {
                check: RustCheck::Build,
                changes: vec![RustChange {
                    source: "public".into(),
                    file: "lib.rs".into(),
                    line: 1,
                    column: 7,
                    old: "weigh".into(),
                    new: "measure".into(),
                }],
                fixed: vec![],
                left,
            };
            let before = turn.reply.clone();
            apply_rename(&system, &mut session, &mut turn).unwrap();
            assert_eq!(turn.reply, before);
            assert_eq!(
                std::fs::read_to_string(fixture.0.join("lib.rs")).unwrap(),
                original
            );
        }
    }
}
