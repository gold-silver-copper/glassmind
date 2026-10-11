//! Bounded AST additions of standard traits, driven only by compiler errors.
use std::path::Path;

use syn::spanned::Spanned;

use crate::checks::{Finding, RustChange};

// Unknown attributes can transform items; conditional items and tests are not
// repair targets. Qualified derivation avoids local trait/macro shadowing.
fn plain(attrs: &[syn::Attribute]) -> bool {
    use syn::parse::Parser;
    attrs.iter().all(|a| {
        if a.path().is_ident("doc") {
            return true;
        }
        let syn::Meta::List(list) = &a.meta else {
            return false;
        };
        if !list.path.is_ident("derive") {
            return false;
        }
        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
            .parse2(list.tokens.clone())
            .is_ok_and(|paths| {
                paths.iter().all(|p| {
                    [
                        "Debug",
                        "Clone",
                        "Copy",
                        "PartialEq",
                        "Eq",
                        "PartialOrd",
                        "Ord",
                        "Hash",
                        "Default",
                    ]
                    .iter()
                    .any(|name| p.is_ident(name))
                })
            })
    })
}

fn declarations(
    items: &[syn::Item],
    name: &str,
    allowed: bool,
    matches: &mut Vec<Option<proc_macro2::Span>>,
) {
    for item in items {
        match item {
            syn::Item::Struct(s) if s.ident == name => {
                matches.push((allowed && plain(&s.attrs)).then(|| s.span()));
            }
            syn::Item::Enum(e) if e.ident == name => {
                matches.push((allowed && plain(&e.attrs)).then(|| e.span()));
            }
            syn::Item::Type(t) if t.ident == name => matches.push(None),
            syn::Item::Union(u) if u.ident == name => matches.push(None),
            syn::Item::Mod(m) => {
                if let Some((_, children)) = &m.content {
                    declarations(children, name, allowed && plain(&m.attrs), matches);
                }
            }
            _ => {}
        }
    }
}

/// One unique production declaration, with a fully qualified standard derive.
/// No type synthesis, field changes, or user language is involved.
pub(crate) fn debug(source: &str, root: &Path, findings: &[Finding]) -> Vec<RustChange> {
    let Ok(canonical_root) = root.canonicalize() else {
        return Vec::new();
    };
    let mut changes = Vec::new();
    for finding in findings {
        if finding.item.source != source || finding.item.name != "E0277" {
            continue;
        }
        let Some(name) = finding
            .item
            .detail
            .as_deref()
            .and_then(|s| s.strip_prefix('`'))
            .and_then(|s| s.strip_suffix("` doesn't implement `Debug`"))
        else {
            continue;
        };
        if syn::parse_str::<syn::Ident>(name).is_err() {
            continue;
        }
        let mut matches = Vec::new();
        for file in crate::slips::production_files(root) {
            // Reject symlinks escaping the source; proposals never use compiler
            // file spans (which may refer to dependencies or macro expansions).
            let Ok(path) = root.join(&file).canonicalize() else {
                continue;
            };
            if !path.starts_with(&canonical_root) {
                return Vec::new();
            }
            let Ok(text) = std::fs::read_to_string(path) else {
                continue;
            };
            let Ok(tree) = syn::parse_file(&text) else {
                continue;
            };
            let mut spans = Vec::new();
            declarations(&tree.items, name, plain(&tree.attrs), &mut spans);
            for span in spans {
                let change = span.and_then(|span| {
                    let start = span.start();
                    let line = text.lines().nth(start.line.checked_sub(1)?)?;
                    // syn spans and RustChange both count Unicode characters.
                    if start.column > line.chars().count() {
                        return None;
                    }
                    let end = span.end();
                    let offset = |line: usize, column: usize| -> Option<usize> {
                        let prefix: usize = text
                            .split_inclusive('\n')
                            .take(line.checked_sub(1)?)
                            .map(|l| l.chars().count())
                            .sum();
                        prefix.checked_add(column)
                    };
                    let chars: Vec<char> = text.chars().collect();
                    let old: String = chars
                        .get(offset(start.line, start.column)?..offset(end.line, end.column)?)?
                        .iter()
                        .collect();
                    if old.is_empty() {
                        return None;
                    }
                    Some(RustChange {
                        source: source.into(),
                        file: file.to_string_lossy().into_owned(),
                        line: u32::try_from(start.line).ok()?,
                        column: u32::try_from(start.column).ok()?,
                        new: format!("#[derive(::core::fmt::Debug)] {old}"),
                        old,
                    })
                });
                matches.push(change);
            }
        }
        if matches.len() == 1
            && let Some(Some(change)) = matches.pop()
        {
            changes.push(change);
        }
    }
    changes.sort();
    changes.dedup();
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustItem, RustKind};

    fn finding(name: &str) -> Finding {
        Finding {
            item: RustItem {
                source: "s".into(),
                file: "src/main.rs".into(),
                line: 1,
                name: "E0277".into(),
                kind: RustKind::Problem,
                owner: None,
                detail: Some(format!("`{name}` doesn't implement `Debug`")),
            },
            fixes: Vec::new(),
        }
    }

    #[test]
    fn standard_debug_is_atomic_and_preserves_unicode_and_attributes() {
        let root = crate::checks::scratch("debug-unicode");
        std::fs::create_dir_all(root.join("src")).unwrap();
        let text = "const É: u8 = 1; #[derive(Clone)] pub struct État(u8);\n";
        std::fs::write(root.join("src/lib.rs"), text).unwrap();
        let changes = debug("s", &root, &[finding("État"), finding("État")]);
        assert_eq!(changes.len(), 1);
        assert!(changes.iter().all(|c| !c.old.is_empty()));
        crate::checks::apply(
            &changes,
            &std::collections::BTreeMap::from([("s".into(), root.clone())]),
        )
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("src/lib.rs")).unwrap(),
            "const É: u8 = 1; #[derive(::core::fmt::Debug)] #[derive(Clone)] pub struct État(u8);\n"
        );
        let stale = "const É: u8 = 1; #[derive(Clone)] pub struct Changed(u8);\n";
        std::fs::write(root.join("src/lib.rs"), stale).unwrap();
        assert!(
            crate::checks::apply(
                &changes,
                &std::collections::BTreeMap::from([("s".into(), root.clone())])
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(root.join("src/lib.rs")).unwrap(),
            stale
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_ambiguous_conditional_test_and_non_identifier_types() {
        let root = crate::checks::scratch("debug-refusals");
        std::fs::create_dir_all(root.join("src")).unwrap();
        for text in [
            "mod a { struct Entry; } mod b { struct Entry; }",
            "#[cfg(unix)] struct Entry;",
            "#[cfg(test)] mod tests { struct Entry; }",
            "#[unknown] struct Entry;",
            "#[derive(Custom)] struct Entry;",
            "type Entry = u8;",
        ] {
            std::fs::write(root.join("src/lib.rs"), text).unwrap();
            assert!(debug("s", &root, &[finding("Entry")]).is_empty());
        }
        std::fs::write(root.join("src/lib.rs"), "struct Entry;").unwrap();
        assert!(debug("s", &root, &[finding("Entry<u8>")]).is_empty());
        let mut f = finding("Entry");
        f.item.name = "E0603".into();
        assert!(debug("s", &root, &[f]).is_empty());
        assert!(debug("other", &root, &[finding("Entry")]).is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_source_symlink_outside_root() {
        let root = crate::checks::scratch("debug-link");
        let outside = crate::checks::scratch("debug-outside");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(&outside, "struct Entry;").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("src/lib.rs")).unwrap();
        assert!(debug("s", &root, &[finding("Entry")]).is_empty());
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_file(outside).unwrap();
    }
}
