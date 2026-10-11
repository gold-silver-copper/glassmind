//! Closed-world variant binding resolution. No token is changed by spelling alone.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use proc_macro2::Ident;
use syn::visit::Visit;

use crate::index::{Pos, SourceIndex};
use crate::rename_types::{Module, absolute, imports, modules, position};
use crate::{RustItem, RustKind};

pub(crate) fn positions(source: &SourceIndex, item: &RustItem, new: &str) -> Option<BTreeSet<Pos>> {
    if item.kind != RustKind::Variant
        || !source.skipped.is_empty()
        || source.idents.contains_key(new)
        || source.idents.contains_key(&format!("r#{new}"))
        || source.idents.contains_key(&format!("r#{}", item.name))
        || new.starts_with("r#")
        || item.name.starts_with("r#")
        || syn::parse_str::<Ident>(new).is_err()
    {
        return None;
    }
    let roots: Vec<_> = ["src/lib.rs", "src/main.rs", "lib.rs", "main.rs"]
        .into_iter()
        .filter(|f| source.files.iter().any(|s| s == f))
        .collect();
    let [root] = roots.as_slice() else {
        return None;
    };
    let parsed = syn::parse_file(&std::fs::read_to_string(source.root.join(root)).ok()?).ok()?;
    if !parsed.attrs.is_empty() {
        return None;
    }
    let mut all = Vec::new();
    modules(
        source,
        root,
        Vec::new(),
        Path::new(root).parent()?,
        parsed.items,
        &mut all,
    )?;
    if all.iter().map(|m| m.file).collect::<BTreeSet<_>>().len() != source.files.len() {
        return None;
    }
    let mut target = None;
    let mut marked = BTreeSet::new();
    for m in &all {
        for node in &m.items {
            if let syn::Item::Enum(e) = node {
                for variant in &e.variants {
                    if variant.ident == item.name.as_str() {
                        if target.is_some()
                            || item.owner.as_deref() != Some(e.ident.to_string().as_str())
                            || source.files.get(m.file)? != &item.file
                            || position(m.file, &variant.ident).1 != item.line
                        {
                            return None;
                        }
                        let mut path = m.path.clone();
                        path.push(e.ident.to_string());
                        path.push(item.name.clone());
                        target = Some(path);
                        marked.insert(position(m.file, &variant.ident));
                    }
                }
            }
        }
    }
    let target = target?;
    let variant_paths: BTreeSet<Vec<String>> = all
        .iter()
        .flat_map(|m| {
            m.items
                .iter()
                .filter_map(|node| {
                    if let syn::Item::Enum(e) = node {
                        Some(e)
                    } else {
                        None
                    }
                })
                .flat_map(|e| {
                    e.variants.iter().map(|v| {
                        let mut path = m.path.clone();
                        path.push(e.ident.to_string());
                        path.push(v.ident.to_string());
                        path
                    })
                })
        })
        .collect();
    // All module names and the owner are protected against generic/local shadows.
    let mut protected: BTreeSet<_> = all.iter().flat_map(|m| m.path.iter().cloned()).collect();
    protected.insert(item.owner.clone()?);
    for m in &all {
        let mut bindings = BTreeMap::new();
        // Explicit local definitions prevent imports from silently replacing roots.
        for node in &m.items {
            let ident = match node {
                syn::Item::Enum(n) => Some(&n.ident),
                syn::Item::Struct(n) => Some(&n.ident),
                syn::Item::Mod(n) => Some(&n.ident),
                syn::Item::Type(n) => Some(&n.ident),
                syn::Item::Trait(n) => Some(&n.ident),
                _ => None,
            };
            if let Some(ident) = ident {
                let mut path = m.path.clone();
                path.push(ident.to_string());
                if bindings.insert(ident.to_string(), path).is_some() {
                    return None;
                }
            }
        }
        // Uses are resolved from explicit lexical paths. Chained reexports and
        // imports through aliases intentionally remain outside this subset.
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
                    let mut names: Vec<_> = ids.iter().map(ToString::to_string).collect();
                    let self_import = names.last().is_some_and(|n| n == "self") && names.len() > 1;
                    if self_import {
                        names.pop();
                    }
                    let resolved = absolute(&m.path, &names)?;
                    let local = if self_import && alias == "self" {
                        names.last()?.clone()
                    } else {
                        alias.to_string()
                    };
                    if bindings.insert(local, resolved).is_some() {
                        return None;
                    }
                    for (i, id) in ids.iter().enumerate() {
                        if *id == item.name.as_str() {
                            if absolute(&m.path, names.get(..=i)?)? != target {
                                return None;
                            }
                            marked.insert(position(m.file, id));
                        }
                    }
                    if alias == item.name.as_str()
                        && ids
                            .last()
                            .is_none_or(|id| id.span().start() != alias.span().start())
                    {
                        return None;
                    }
                }
            }
        }
        let mut protected = protected.clone();
        protected.extend(bindings.keys().cloned());
        let mut visitor = References {
            module: m,
            target: &target,
            old: &item.name,
            bindings,
            protected,
            variant_paths: &variant_paths,
            self_type: None,
            marked: &mut marked,
            valid: true,
        };
        for node in &m.items {
            visitor.visit_item(node);
        }
        if !visitor.valid {
            return None;
        }
    }
    // Unhandled syntax, declarations, attributes or macro tokens cannot escape.
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
    protected: BTreeSet<String>,
    variant_paths: &'a BTreeSet<Vec<String>>,
    self_type: Option<Vec<String>>,
    marked: &'a mut BTreeSet<Pos>,
    valid: bool,
}

impl References<'_> {
    fn resolve(&self, names: &[String]) -> Option<Vec<String>> {
        let first = names.first()?;
        let binding = if first == "Self" {
            self.self_type.as_ref()
        } else {
            self.bindings.get(first)
        };
        if let Some(binding) = binding {
            let mut result = binding.clone();
            result.extend(names.iter().skip(1).cloned());
            Some(result)
        } else if first == "Self" {
            None
        } else {
            absolute(&self.module.path, names)
        }
    }
}

impl<'ast> Visit<'ast> for References<'_> {
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}
    fn visit_item_use(&mut self, _: &'ast syn::ItemUse) {}
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let saved = self.self_type.take();
        if node.trait_.is_none()
            && let syn::Type::Path(p) = node.self_ty.as_ref()
            && p.qself.is_none()
            && p.path.leading_colon.is_none()
        {
            self.self_type = self.resolve(
                &p.path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>(),
            );
        }
        syn::visit::visit_item_impl(self, node);
        self.self_type = saved;
    }
    fn visit_type_param(&mut self, p: &'ast syn::TypeParam) {
        if self.protected.contains(&p.ident.to_string()) {
            self.valid = false;
        }
        syn::visit::visit_type_param(self, p);
    }
    fn visit_const_param(&mut self, p: &'ast syn::ConstParam) {
        if self.protected.contains(&p.ident.to_string()) {
            self.valid = false;
        }
        syn::visit::visit_const_param(self, p);
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
    fn visit_pat_ident(&mut self, p: &'ast syn::PatIdent) {
        // syn cannot distinguish bare imported unit variants from bindings.
        // With an explicit import Rust resolves a bare immutable pattern to the
        // variant; ref/mut/@ bindings are not accepted as variant references.
        if p.ident == self.old
            && p.by_ref.is_none()
            && p.mutability.is_none()
            && p.subpat.is_none()
            && self.bindings.get(self.old).map(Vec::as_slice) == Some(self.target)
        {
            self.marked.insert(position(self.module.file, &p.ident));
        } else if self.protected.contains(&p.ident.to_string()) {
            // Other explicitly imported enum variants are also patterns, not
            // local shadow bindings. They stay unchanged.
            let imported_variant = p.by_ref.is_none()
                && p.mutability.is_none()
                && p.subpat.is_none()
                && self
                    .bindings
                    .get(&p.ident.to_string())
                    .is_some_and(|path| self.variant_paths.contains(path));
            if !imported_variant {
                self.valid = false;
            }
        }
        syn::visit::visit_pat_ident(self, p);
    }
    fn visit_field_value(&mut self, f: &'ast syn::FieldValue) {
        if f.colon_token.is_some() {
            syn::visit::visit_field_value(self, f);
        }
    }
    fn visit_field_pat(&mut self, f: &'ast syn::FieldPat) {
        if f.colon_token.is_some() {
            syn::visit::visit_field_pat(self, f);
        }
    }
    fn visit_block(&mut self, b: &'ast syn::Block) {
        if b.stmts.iter().any(|s| matches!(s, syn::Stmt::Item(_))) {
            self.valid = false;
        }
        syn::visit::visit_block(self, b);
    }
    fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
        self.valid = false;
    }
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if !(m.path.is_ident("println") || m.path.is_ident("eprintln")) {
            self.valid = false;
        }
    }
    fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
        if !a.path().is_ident("doc") && !a.path().is_ident("test") {
            self.valid = false;
        }
    }
    fn visit_path(&mut self, p: &'ast syn::Path) {
        let names: Vec<_> = p.segments.iter().map(|s| s.ident.to_string()).collect();
        for (i, segment) in p.segments.iter().enumerate() {
            if segment.ident == self.old {
                if p.leading_colon.is_none()
                    && i + 1 == names.len()
                    && self.resolve(&names).as_deref() == Some(self.target)
                {
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
    use glassmind_meaning::interface::World;
    use glassmind_meaning::{Action, Name, Reply, Thing};
    use glassmind_rules::Switches;

    fn check(case: &str, code: &str) -> Option<BTreeSet<Pos>> {
        let root =
            std::env::temp_dir().join(format!("glassmind-variant-{case}-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), code).unwrap();
        let index = crate::index::index_source("sample", &root);
        let item = index
            .records
            .iter()
            .find(|r| r.item.kind == RustKind::Variant && r.item.name == "Ready")
            .unwrap();
        let result = positions(&index, &item.item, "Active");
        std::fs::remove_dir_all(root).unwrap();
        result
    }

    #[test]
    fn binds_variants_imports_self_and_patterns() {
        let code = "enum State<T> { Ready(T) } impl<T> State<T> { fn f(x: T) -> Self { Self::Ready(x) } } use State::Ready; fn f(x: State<u32>) -> u32 { match x { Ready(n) => n } }";
        assert_eq!(check("paths", code).unwrap().len(), 4);
        assert_eq!(
            check(
                "unit",
                "enum State { Ready, Empty } use State::{Ready, Empty}; fn f(x: State) { match x { Ready => (), Empty => () } }"
            )
            .unwrap()
            .len(),
            3
        );
    }

    #[test]
    fn rejects_unresolved_or_shadowed_bindings() {
        for (name, code) in [
            (
                "generic",
                "enum State { Ready } fn f<State>() { let _ = State::Ready; }",
            ),
            (
                "block",
                "enum State { Ready } fn f() { struct State; let _ = State::Ready; }",
            ),
            (
                "raw",
                "enum State { Ready } fn f() { let _ = State::r#Ready; }",
            ),
            (
                "macro",
                "enum State { Ready } fn f() { made!(); let _ = State::Ready; }",
            ),
            (
                "alias",
                "enum State { Ready } type Alias = State; fn f() { let _ = Alias::Ready; }",
            ),
            ("unrelated", "enum State { Ready } enum Other { Ready }"),
            (
                "glob",
                "enum State { Ready } use State::*; fn f() { let _ = Ready; }",
            ),
        ] {
            assert!(check(name, code).is_none(), "{name}");
        }
    }

    #[test]
    fn variant_ownership_uses_declaration_identity() {
        let root =
            std::env::temp_dir().join(format!("glassmind-variant-owners-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        for (code, expected) in [
            (
                "mod a { pub enum State { Ready } }\nmod b { pub enum State { Empty } }",
                true,
            ),
            (
                "mod a { pub enum State { Ready } } mod b { pub enum State { Empty } }",
                false,
            ),
        ] {
            std::fs::write(root.join("src/lib.rs"), code).unwrap();
            let world =
                crate::RustWorld::open(&[("sample".into(), root.as_path())], &Switches::all_on());
            let item = world
                .lookup(&Name("Ready".into()), &mut Vec::new())
                .pop()
                .unwrap();
            let found = world.related(
                crate::RustRelation::Member,
                glassmind_meaning::Position::First,
                &BTreeSet::from([item]),
                &mut Vec::new(),
            );
            assert_eq!(found.is_ok(), expected);
            if let Ok(owners) = found {
                assert_eq!(owners.len(), 1);
                assert_eq!(owners.first().unwrap().line, 1);
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn variants_have_a_distinct_kind_and_owner() {
        let root =
            std::env::temp_dir().join(format!("glassmind-variant-world-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/lib.rs"),
            "pub enum State { Ready } fn f() { let _ = State::Ready; }",
        )
        .unwrap();
        let world =
            crate::RustWorld::open(&[("sample".into(), root.as_path())], &Switches::all_on());
        let items = world.lookup(&Name("State::Ready".into()), &mut Vec::new());
        assert_eq!(items.len(), 1);
        let item = items.first().unwrap();
        assert_eq!(item.kind, RustKind::Variant);
        assert_eq!(item.owner.as_deref(), Some("State"));
        let mut fired = Vec::new();
        let proposal = world
            .propose(
                &Action::Rename {
                    thing: Thing::Item(item.clone()),
                    kind: Some(RustKind::Variant),
                    name: Some(Name("Active".into())),
                },
                &mut fired,
            )
            .unwrap();
        assert!(matches!(proposal, Reply::Proposal { spots, .. } if spots.len() == 2));
        assert!(fired.iter().any(|r| r.id == "rust.rename.variant"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
