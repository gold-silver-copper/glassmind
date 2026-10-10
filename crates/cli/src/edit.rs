//! Showing and applying a proposed edit. Files are only written with
//! `--apply`, and nothing is ever committed.

use std::collections::BTreeMap;
use std::path::PathBuf;

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

/// Applies every change, or none: each spot must still hold the old name.
/// `roots` maps a source name to its directory.
pub fn apply(changes: &[Change], roots: &BTreeMap<String, PathBuf>) -> Result<usize, String> {
    let mut by_file: BTreeMap<PathBuf, Vec<&Change>> = BTreeMap::new();
    for c in changes {
        let root = roots
            .get(&c.source)
            .ok_or_else(|| format!("unknown source {}", c.source))?;
        by_file.entry(root.join(&c.file)).or_default().push(c);
    }
    // Check and rewrite every file in memory before writing any.
    let mut rewritten = Vec::new();
    for (path, mut file_changes) in by_file {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let mut lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
        // Later spots first, so earlier columns stay valid.
        file_changes.sort_by_key(|c| std::cmp::Reverse((c.line, c.column)));
        for c in file_changes {
            let index = usize::try_from(c.line)
                .unwrap_or(usize::MAX)
                .wrapping_sub(1);
            let line = lines
                .get_mut(index)
                .ok_or_else(|| format!("{}:{} no longer exists", path.display(), c.line))?;
            let chars: Vec<char> = line.chars().collect();
            let start = usize::try_from(c.column).unwrap_or(usize::MAX);
            let old: Vec<char> = c.old.chars().collect();
            let end = start.saturating_add(old.len());
            if chars.get(start..end) != Some(old.as_slice()) {
                return Err(format!(
                    "{}:{}:{} no longer holds `{}`; nothing was changed",
                    path.display(),
                    c.line,
                    c.column + 1,
                    c.old
                ));
            }
            let mut new_line: String = chars.get(..start).unwrap_or_default().iter().collect();
            new_line.push_str(&c.new);
            new_line.extend(chars.get(end..).unwrap_or_default().iter());
            *line = new_line;
        }
        rewritten.push((path, lines.join("\n")));
    }
    for (path, text) in &rewritten {
        std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    Ok(changes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(line: u32, column: u32) -> Change {
        Change {
            source: "s".into(),
            file: "lib.rs".into(),
            line,
            column,
            old: "hp".into(),
            new: "health".into(),
        }
    }

    #[test]
    fn applies_every_change_or_none() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/test-edit");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "struct P { hp: u32 }\nfn f(p: P) -> u32 { p.hp + p.hp }\n",
        )
        .unwrap();
        let roots = BTreeMap::from([("s".to_owned(), dir.clone())]);
        // A spot that no longer holds `hp` stops the whole edit.
        let stale = [change(1, 11), change(2, 0)];
        assert!(apply(&stale, &roots).is_err());
        assert!(
            std::fs::read_to_string(&file)
                .unwrap()
                .contains("{ hp: u32 }")
        );
        let good = [change(1, 11), change(2, 22), change(2, 29)];
        assert_eq!(apply(&good, &roots), Ok(3));
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "struct P { health: u32 }\nfn f(p: P) -> u32 { p.health + p.health }\n"
        );
    }
}
