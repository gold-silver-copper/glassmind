//! What the harness shows under a reply: each problem a check found with
//! the tool's own message, and each change proposed, so the user can judge
//! before saying yes.

use glassmind_meaning::{Action, Labelled, Reply, Thing};
use glassmind_world_rust::{Rust, RustItem};

fn problem(item: &RustItem) -> String {
    let at = if item.file.is_empty() {
        String::new()
    } else {
        format!("{}:{}  ", item.file, item.line)
    };
    format!(
        "  {at}{}  {}",
        item.name,
        item.detail.as_deref().unwrap_or_default()
    )
}

/// Lines to show under the reply, if it has details worth showing.
pub fn details(reply: &Reply<Rust>) -> Vec<String> {
    match reply {
        Reply::Checked { problems, .. } | Reply::Applied { problems, .. } => {
            problems.iter().map(problem).collect()
        }
        Reply::Remedy { changes, left, .. } => changes
            .iter()
            .map(|c| {
                format!(
                    "  {}:{}:{}  `{}` -> `{}`",
                    c.file,
                    c.line,
                    c.column + 1,
                    c.old.replace('\n', "⏎"),
                    c.new.replace('\n', "⏎")
                )
            })
            .chain(
                left.iter()
                    .map(|p| format!("  still: {}", problem(p).trim_start())),
            )
            .chain((!changes.is_empty()).then(|| "  (say yes to make these changes)".to_owned()))
            .collect(),
        Reply::Proposal {
            action:
                Action::Rename {
                    thing: Thing::Item(item),
                    name: Some(name),
                    ..
                },
            spots,
        } => spots
            .iter()
            .map(|s| {
                format!(
                    "  {}:{}:{}  `{}` -> `{}`",
                    s.file,
                    s.line,
                    s.column + 1,
                    item.name,
                    name.0
                )
            })
            .chain(std::iter::once(format!(
                "  (say yes to rename `{}`)",
                item.label()
            )))
            .collect(),
        Reply::Assuming { reply, .. } => details(reply),
        _ => Vec::new(),
    }
}
