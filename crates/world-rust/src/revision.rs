//! One supported repair candidate for an isolated, bounded revision trial.
use crate::{
    RustItem, RustKind,
    checks::{self, Finding, RustChange},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// This is a proposal, never authority to edit the user's source. The caller
/// must check the resulting trial and finally verify the composed application.
pub fn candidate(source: &str, root: &Path, found: &[Finding]) -> Vec<RustChange> {
    let mut changes: Vec<_> = found.iter().flat_map(|f| f.fixes.iter().cloned()).collect();
    let mut places = BTreeSet::new();
    changes.retain(|c| places.insert((c.file.clone(), c.line, c.column, c.old.clone())));
    if changes.is_empty() {
        changes = crate::standard_traits::debug(source, root, found);
    }
    if changes.is_empty() {
        changes = crate::index_conversion::proposals(source, root, found);
    }
    if changes.is_empty()
        && !found.is_empty()
        && found.iter().all(|f| f.item.kind == RustKind::Test)
    {
        let problems: Vec<RustItem> = found.iter().map(|f| f.item.clone()).collect();
        changes.extend(checks::undo_slip(source, root, &problems));
    }
    changes.sort();
    changes.dedup();
    changes
}

/// Diagnostic identity excludes process-specific prose, while retaining exact
/// locations and kinds so compiler errors and failing tests remain distinct.
pub fn diagnostics(items: &[RustItem]) -> BTreeSet<(String, String, u32, RustKind, String)> {
    items
        .iter()
        .map(|i| {
            (
                i.source.clone(),
                i.file.clone(),
                i.line,
                i.kind,
                i.name.clone(),
            )
        })
        .collect()
}

/// Compose a checked trial onto the untouched original using ordinary typed
/// changes. This controller never creates/deletes files or changes non-UTF8 data.
pub fn compose(
    source: &str,
    before: &BTreeMap<String, Vec<u8>>,
    after: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<RustChange>, String> {
    if before.keys().ne(after.keys()) {
        return Err("revision changed the set of source files".into());
    }
    let mut changes = Vec::new();
    for (file, old) in before {
        let new = after.get(file).ok_or("revision file missing")?;
        if old == new {
            continue;
        }
        let old = String::from_utf8(old.clone()).map_err(|_| "revision requires UTF-8 source")?;
        let new = String::from_utf8(new.clone()).map_err(|_| "revision requires UTF-8 source")?;
        changes.push(RustChange {
            source: source.into(),
            file: file.clone(),
            line: 1,
            column: 0,
            old,
            new,
        });
    }
    Ok(changes)
}
