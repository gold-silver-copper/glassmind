//! Conservative type renaming with explicit module/import binding checks.
//!
//! This deliberately accepts a closed subset: one conventional crate root,
//! ordinary modules and explicit imports. Every old-name token must be bound
//! to the selected declaration. Unknown syntax, shadow declarations, macros
//! containing the name, glob imports, and unaccounted source files refuse.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use proc_macro2::Ident;
use syn::visit::Visit;

use crate::index::{Pos, SourceIndex};
use crate::{RustItem, RustKind};

struct Module {
    path: Vec<String>,
    file: usize,
    items: Vec<syn::Item>,
}

fn position(file: usize, ident: &Ident) -> Pos {
    let p = ident.span().start();
    (file, p.line as u32, p.column as u32)
}

fn modules(
    source: &SourceIndex,
    file: &str,
    path: Vec<String>,
    directory: &Path,
    items: Vec<syn::Item>,
    out: &mut Vec<Module>,
) -> Option<()> {
    let file_index = source.files.iter().position(|f| f == file)?;
    for item in &items {
        if let syn::Item::Mod(m) = item {
            if !m.attrs.is_empty() {
                return None;
            }
            let mut child_path = path.clone();
            child_path.push(m.ident.to_string());
            let child_dir = directory.join(m.ident.to_string());
            if let Some((_, inner)) = &m.content {
                modules(source, file, child_path, &child_dir, inner.clone(), out)?;
            } else {
                let direct = directory.join(format!("{}.rs", m.ident));
                let nested = child_dir.join("mod.rs");
                let chosen = match (
                    source.root.join(&direct).is_file(),
                    source.root.join(&nested).is_file(),
                ) {
                    (true, false) => direct,
                    (false, true) => nested,
                    _ => return None,
                };
                let text = std::fs::read_to_string(source.root.join(&chosen)).ok()?;
                let parsed = syn::parse_file(&text).ok()?;
                if !parsed.attrs.is_empty() {
                    return None;
                }
                modules(
                    source,
                    chosen.to_str()?,
                    child_path,
                    &child_dir,
                    parsed.items,
                    out,
                )?;
            }
        }
    }
    out.push(Module {
        path,
        file: file_index,
        items,
    });
    Some(())
}

// Only lexical module paths, never dependency paths or implicit preludes.
fn absolute(module: &[String], path: &[String]) -> Option<Vec<String>> {
    let mut result = module.to_vec();
    let mut parts = path.iter().peekable();
    match parts.peek().map(|s| s.as_str()) {
        Some("crate") => {
            result.clear();
            parts.next();
        }
        Some("self") => {
            parts.next();
        }
        Some("super") => {
            while parts.peek().is_some_and(|s| s.as_str() == "super") {
                result.pop()?;
                parts.next();
            }
        }
        _ => {}
    }
    result.extend(parts.cloned());
    Some(result)
}

fn imports(
    tree: &syn::UseTree,
    prefix: &mut Vec<Ident>,
    out: &mut Vec<(Vec<Ident>, Ident)>,
) -> Option<()> {
    match tree {
        syn::UseTree::Path(p) => {
            prefix.push(p.ident.clone());
            imports(&p.tree, prefix, out)?;
            prefix.pop();
        }
        syn::UseTree::Name(n) => {
            let mut path = prefix.clone();
            path.push(n.ident.clone());
            out.push((path, n.ident.clone()));
        }
        syn::UseTree::Rename(n) => {
            let mut path = prefix.clone();
            path.push(n.ident.clone());
            out.push((path, n.rename.clone()));
        }
        syn::UseTree::Group(g) => {
            for item in &g.items {
                imports(item, prefix, out)?;
            }
        }
        syn::UseTree::Glob(_) => return None,
    }
    Some(())
}

pub(crate) fn positions(source: &SourceIndex, item: &RustItem, new: &str) -> Option<BTreeSet<Pos>> {
    if !source.skipped.is_empty()
        || source.idents.contains_key(new)
        || source.idents.contains_key(&format!("r#{}", item.name))
        || source.idents.contains_key(&format!("r#{new}"))
        || syn::parse_str::<Ident>(new).is_err()
        || new.starts_with("r#")
        || item.name.starts_with("r#")
    {
        return None;
    }
    let roots: Vec<_> = ["src/lib.rs", "src/main.rs"]
        .into_iter()
        .filter(|f| source.files.iter().any(|s| s == f))
        .collect();
    if roots.len() != 1 {
        return None;
    }
    let root = roots.first()?;
    let text = std::fs::read_to_string(source.root.join(root)).ok()?;
    let parsed = syn::parse_file(&text).ok()?;
    if !parsed.attrs.is_empty() {
        return None;
    }
    let mut all = Vec::new();
    modules(
        source,
        root,
        Vec::new(),
        Path::new("src"),
        parsed.items,
        &mut all,
    )?;
    if all.iter().map(|m| m.file).collect::<BTreeSet<_>>().len() != source.files.len() {
        return None;
    }
    let mut target = None;
    let mut declaration = None;
    for m in &all {
        for node in &m.items {
            let found = match node {
                syn::Item::Struct(s) if item.kind == RustKind::Struct => Some(&s.ident),
                syn::Item::Enum(e) if item.kind == RustKind::Enum => Some(&e.ident),
                _ => None,
            };
            if let Some(id) = found.filter(|id| *id == item.name.as_str()) {
                if target.is_some()
                    || source.files.get(m.file)? != &item.file
                    || position(m.file, id).1 != item.line
                {
                    return None;
                }
                let mut path = m.path.clone();
                path.push(item.name.clone());
                target = Some(path);
                declaration = Some(position(m.file, id));
            }
        }
    }
    let target = target?;
    let mut marked = BTreeSet::from([declaration?]);
    for m in &all {
        let mut bindings = BTreeMap::new();
        for node in &m.items {
            if let syn::Item::Use(u) = node {
                if u.leading_colon.is_some() || !u.attrs.is_empty() {
                    return None;
                }
                let mut leaves = Vec::new();
                imports(&u.tree, &mut Vec::new(), &mut leaves)?;
                for (ids, alias) in leaves {
                    if alias == "println" || alias == "eprintln" {
                        return None;
                    }
                    let path: Vec<_> = ids.iter().map(ToString::to_string).collect();
                    let resolved = absolute(&m.path, &path)?;
                    if bindings
                        .insert(alias.to_string(), resolved.clone())
                        .is_some()
                    {
                        return None;
                    }
                    for (i, id) in ids.iter().enumerate() {
                        if *id == item.name.as_str() {
                            if absolute(&m.path, path.get(..=i)?)? != target {
                                return None;
                            }
                            marked.insert(position(m.file, id));
                        }
                    }
                    // `as Old` is a second binding, not a reference to Old.
                    if alias == item.name.as_str()
                        && ids
                            .last()
                            .is_none_or(|last| last.span().start() != alias.span().start())
                    {
                        return None;
                    }
                }
            }
        }
        let mut visitor = References {
            module: m,
            target: &target,
            old: &item.name,
            bindings,
            marked: &mut marked,
            valid: true,
            module_names: all.iter().flat_map(|m| m.path.iter().cloned()).collect(),
        };
        for node in &m.items {
            visitor.visit_item(node);
        }
        if !visitor.valid {
            return None;
        }
    }
    // Tokens in unhandled syntax include shadowing declarations, attributes,
    // raw identifiers, macro input and unrelated same-spelled symbols.
    if &marked != source.idents.get(&item.name)? {
        return None;
    }
    Some(marked)
}

struct References<'a> {
    module: &'a Module,
    target: &'a [String],
    old: &'a str,
    bindings: BTreeMap<String, Vec<String>>,
    marked: &'a mut BTreeSet<Pos>,
    valid: bool,
    module_names: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for References<'_> {
    fn visit_field_value(&mut self, f: &'ast syn::FieldValue) {
        // A shorthand token names both a field and a value. Renaming its
        // constructor binding alone must not silently rename the field.
        if f.colon_token.is_some() {
            syn::visit::visit_field_value(self, f);
        }
    }
    fn visit_type_param(&mut self, p: &'ast syn::TypeParam) {
        if self.module_names.contains(&p.ident.to_string())
            || self.bindings.contains_key(&p.ident.to_string())
        {
            self.valid = false;
        }
        syn::visit::visit_type_param(self, p);
    }
    fn visit_type_path(&mut self, p: &'ast syn::TypePath) {
        if p.qself.is_some() {
            self.valid = false;
        }
        syn::visit::visit_type_path(self, p);
    }
    fn visit_expr_path(&mut self, p: &'ast syn::ExprPath) {
        if p.qself.is_some() {
            self.valid = false;
        }
        syn::visit::visit_expr_path(self, p);
    }
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        // Only standard output macros, which cannot introduce outer bindings.
        // Definitions/attributes are rejected separately; imports of these
        // macro names are forbidden, so a dependency cannot replace them.
        if !(m.path.is_ident("println") || m.path.is_ident("eprintln")) {
            self.valid = false;
        }
    }
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}
    fn visit_item_use(&mut self, _: &'ast syn::ItemUse) {} // handled at module scope
    fn visit_block(&mut self, b: &'ast syn::Block) {
        // Block imports/items have different lexical scopes; do not guess.
        if b.stmts.iter().any(|s| matches!(s, syn::Stmt::Item(_))) {
            self.valid = false;
        }
        syn::visit::visit_block(self, b);
    }
    fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
        self.valid = false;
    }
    fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
        // Derive/proc attributes and cfg can introduce invisible bindings or
        // name-sensitive behavior. Doc text is deliberately left unchanged.
        if !a.path().is_ident("doc") && !a.path().is_ident("test") {
            self.valid = false;
        }
    }
    fn visit_path(&mut self, p: &'ast syn::Path) {
        let names: Vec<_> = p.segments.iter().map(|s| s.ident.to_string()).collect();
        for (i, segment) in p.segments.iter().enumerate() {
            if segment.ident == self.old {
                let prefix = names.get(..=i).unwrap_or_default();
                let resolved = if p.leading_colon.is_some() {
                    None
                } else if let Some(binding) = prefix.first().and_then(|n| self.bindings.get(n)) {
                    let mut full = binding.clone();
                    full.extend(prefix.iter().skip(1).cloned());
                    Some(full)
                } else {
                    absolute(&self.module.path, prefix)
                };
                if resolved.as_deref() == Some(self.target) {
                    self.marked
                        .insert(position(self.module.file, &segment.ident));
                } else {
                    self.valid = false;
                }
            }
        }
        syn::visit::visit_path(self, p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rename(case: &str, code: &str, old: &str, new: &str) -> Option<BTreeSet<Pos>> {
        let root =
            std::env::temp_dir().join(format!("glassmind-type-{case}-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), code).unwrap();
        let index = crate::index::index_source("test", &root);
        let item = index.records.iter().find(|r| r.item.name == old).unwrap();
        let result = positions(&index, &item.item, new);
        std::fs::remove_dir_all(root).unwrap();
        result
    }

    #[test]
    fn resolves_nested_modules_generics_and_unicode_columns() {
        let code = "pub struct État<T>(T); mod inner { use super::État; fn f() { let café = 1; let _: État<u32> = État(café); } }";
        let at = rename("unicode", code, "État", "Statut").unwrap();
        assert_eq!(at.len(), 4);
        for (_, line, column) in at {
            assert_eq!(line, 1);
            assert_eq!(
                code.chars()
                    .skip(column as usize)
                    .take(4)
                    .collect::<String>(),
                "État"
            );
        }
    }

    #[test]
    fn refuses_raw_spelling_and_associated_types() {
        assert!(
            rename(
                "raw",
                "struct String; fn f() { let _ = r#String; }",
                "String",
                "Text"
            )
            .is_none()
        );
        assert!(
            rename(
                "associated",
                "struct Output; fn f<T: std::ops::Add>(x: <T as std::ops::Add>::Output) {}",
                "Output",
                "Result"
            )
            .is_none()
        );
        assert!(
            rename(
                "generic_module",
                "mod T { pub struct Output; } fn f<T: std::ops::Add>(x: T::Output) {}",
                "Output",
                "Result"
            )
            .is_none()
        );
    }

    #[test]
    fn refuses_unexpanded_bindings_and_new_raw_collision() {
        assert!(
            rename(
                "shorthand",
                "struct Old; fn f() { let _ = external::Container { Old }; }",
                "Old",
                "New"
            )
            .is_none()
        );
        assert!(
            rename(
                "macro",
                "struct Old; fn f() { generate!(); let _ = Old; }",
                "Old",
                "New"
            )
            .is_none()
        );
        assert!(rename("derive", "#[derive(Debug)] struct Old;", "Old", "New").is_none());
        assert!(
            rename(
                "collision",
                "struct Old; fn f() { let r#New = 1; }",
                "Old",
                "New"
            )
            .is_none()
        );
    }
}
