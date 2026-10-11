//! A closed-module proof for free-function references. This only proposes spans;
//! the existing apply path remains responsible for verification and rollback.
use crate::index::{Pos, SourceIndex};
use crate::rename_package::{Package, inert};
use crate::rename_types::{Module, imports, position};
use crate::{RustItem, RustKind};
use std::collections::BTreeSet;
use syn::visit::Visit;

pub(crate) fn positions(source: &SourceIndex, item: &RustItem, new: &str) -> Option<BTreeSet<Pos>> {
    if item.kind != RustKind::Function
        || item.owner.is_some()
        || !source.skipped.is_empty()
        || source.idents.contains_key(new)
        || source.idents.contains_key(&format!("r#{new}"))
        || source.idents.contains_key(&format!("r#{}", item.name))
        || item.name.starts_with("r#")
        || new.starts_with("r#")
        || syn::parse_str::<syn::Ident>(new).is_err()
    {
        return None;
    }
    // This existing mode admits only conventional cfg(test) modules and
    // resolves super::* by the actual parent's unique bindings.
    let package = Package::load_with_tests(source)?;
    let mut target = None;
    let mut marked = BTreeSet::new();
    for m in &package.modules {
        for node in &m.items {
            if let syn::Item::Fn(f) = node
                && f.sig.ident == item.name
            {
                if target.is_some()
                    || source.files.get(m.file)? != &item.file
                    || position(m.file, &f.sig.ident).1 != item.line
                {
                    return None;
                }
                let mut key = m.path.clone();
                key.push(item.name.clone());
                target = Some(key);
                marked.insert(position(m.file, &f.sig.ident));
            }
        }
    }
    let target = target?;
    for m in &package.modules {
        for node in &m.items {
            if let syn::Item::Use(u) = node {
                // Package has already resolved these globs. They contain no
                // identifier naming the target and contribute no edit span.
                if quote::quote!(#u).to_string().replace(' ', "") == "usesuper::*;" {
                    continue;
                }
                let mut leaves = Vec::new();
                imports(&u.tree, &mut Vec::new(), &mut leaves)?;
                for (ids, alias) in leaves {
                    let names: Vec<_> = ids.iter().map(ToString::to_string).collect();
                    for (i, id) in ids.iter().enumerate() {
                        if *id == item.name {
                            if i + 1 != ids.len() || package.resolve(&m.path, &names)? != target {
                                return None;
                            }
                            marked.insert(position(m.file, id));
                        }
                    }
                    // Explicit aliases are new bindings, never reference edits.
                    if alias == item.name
                        && ids
                            .last()
                            .is_none_or(|id| position(m.file, id) != position(m.file, &alias))
                    {
                        return None;
                    }
                }
            }
        }
        let bindings = package.bindings(m)?.into_keys().collect();
        let mut visitor = References {
            package: &package,
            module: m,
            target: &target,
            old: &item.name,
            bindings,
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
    // Anything unclassified (fields, methods, type positions, raw tokens,
    // shorthand fields, etc.) refuses, rather than becoming a spelling edit.
    if source.idents.get(&item.name)? != &marked {
        return None;
    }
    Some(marked)
}

struct References<'a> {
    package: &'a Package,
    module: &'a Module,
    target: &'a [String],
    old: &'a str,
    bindings: BTreeSet<String>,
    marked: &'a mut BTreeSet<Pos>,
    valid: bool,
}
impl<'ast> Visit<'ast> for References<'_> {
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}
    fn visit_item_use(&mut self, _: &'ast syn::ItemUse) {}
    fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
        self.valid = false;
    }
    fn visit_block(&mut self, b: &'ast syn::Block) {
        if b.stmts.iter().any(|s| matches!(s, syn::Stmt::Item(_))) {
            self.valid = false;
        }
        syn::visit::visit_block(self, b);
    }
    fn visit_pat_ident(&mut self, p: &'ast syn::PatIdent) {
        // Includes let, parameter, match, for, if-let and closure bindings.
        // Conservative across the whole module: no order-dependent guessing.
        if p.ident == self.old || self.bindings.contains(&p.ident.to_string()) {
            self.valid = false;
        }
        syn::visit::visit_pat_ident(self, p);
    }
    fn visit_type_param(&mut self, p: &'ast syn::TypeParam) {
        if p.ident == self.old || self.bindings.contains(&p.ident.to_string()) {
            self.valid = false;
        }
        syn::visit::visit_type_param(self, p);
    }
    fn visit_const_param(&mut self, p: &'ast syn::ConstParam) {
        if p.ident == self.old || self.bindings.contains(&p.ident.to_string()) {
            self.valid = false;
        }
        syn::visit::visit_const_param(self, p);
    }
    fn visit_field_value(&mut self, f: &'ast syn::FieldValue) {
        // Shorthand spans have both a field and value meaning.
        if f.colon_token.is_some() {
            syn::visit::visit_field_value(self, f);
        }
    }
    fn visit_expr_path(&mut self, p: &'ast syn::ExprPath) {
        let ids: Vec<_> = p
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        if ids.iter().any(|n| n == self.old) {
            if p.qself.is_some()
                || p.path.leading_colon.is_some()
                || ids.last().is_none_or(|n| n != self.old)
                || self.package.resolve(&self.module.path, &ids).as_deref() != Some(self.target)
            {
                self.valid = false;
            } else if let Some(last) = p.path.segments.last() {
                self.marked.insert(position(self.module.file, &last.ident));
            }
        }
        syn::visit::visit_expr_path(self, p);
    }
    fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
        let builtin_derive = a.path().is_ident("derive")
            && a.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .is_ok_and(|ps| {
                ps.iter().all(|p| {
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
                    .any(|n| p.is_ident(n))
                })
            });
        if !inert(a) && !builtin_derive {
            self.valid = false;
        }
    }
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let Some(args) = crate::rename_package::rename_macro_arguments(m, self.old) else {
            self.valid = false;
            return;
        };
        for arg in &args {
            self.visit_expr(arg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn proof(code: &str, old: &str, new: &str) -> Option<Vec<(u32, u32)>> {
        let root = std::env::temp_dir().join(format!(
            "function-proof-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='proof'\nversion='0.0.0'\nedition='2024'\n[workspace]\n",
        )
        .unwrap();
        std::fs::write(root.join("src/lib.rs"), code).unwrap();
        let source = crate::index::index_source("test", &root);
        let item = source
            .records
            .iter()
            .find(|r| r.item.kind == RustKind::Function && r.item.name == old)
            .unwrap();
        let result = positions(&source, &item.item, new)
            .map(|ps| ps.into_iter().map(|(_, l, c)| (l, c)).collect());
        std::fs::remove_dir_all(root).unwrap();
        result
    }
    #[test]
    fn preserves_alias_names_and_literal_bytes() {
        let code = "mod inner {pub fn old(x:u32)->u32{x}}\nuse inner::old as retained;\n#[test] fn check(){assert_eq!(retained(3),3); assert_eq!(inner::old(2),2); let _=\"old\";} // old\n";
        let positions = proof(code, "old", "new_name").unwrap();
        assert_eq!(positions.len(), 3);
        for (line, col) in positions {
            assert_eq!(
                code.lines()
                    .nth(line as usize - 1)
                    .unwrap()
                    .chars()
                    .skip(col as usize)
                    .take(3)
                    .collect::<String>(),
                "old"
            );
        }
    }
    #[test]
    fn unicode_columns_are_characters_and_crlf_survives() {
        let code =
            "pub fn café()->u32{3}\r\n#[test] fn test(){let 文=1; assert_eq!(café(),文+2);}\r\n";
        assert_eq!(
            proof(code, "café", "entrée").unwrap(),
            vec![(1, 7), (2, 38)]
        );
        assert!(proof(code, "café", "r#type").is_none());
    }
    #[test]
    fn shadowing_and_macro_definitions_cannot_certify_a_path() {
        for extra in [
            "fn shadow(old:u32)->u32{old}",
            "fn shadow(){let f=|old:u32| old;}",
            "macro_rules! assert_eq {($($t:tt)*)=>{}}",
            "#[cfg(any())] fn unrelated(){}",
            "fn shadow(){fn old(){} old();}",
            "fn shadow(){println!(\"{old}\", old=4);}",
            "fn shadow(){println!(\"{old:?}\");}",
        ] {
            let code =
                format!("pub fn old()->u32{{3}} #[test] fn test(){{assert_eq!(old(),3);}} {extra}");
            assert!(proof(&code, "old", "new_name").is_none(), "{extra}");
        }
    }
    #[test]
    fn apply_reproves_identity_when_old_token_spans_stay_identical() {
        use glassmind_meaning::interface::World;
        let root = std::env::temp_dir().join(format!("function-stale-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='stale'\nversion='0.0.0'\nedition='2024'\n[workspace]\n",
        )
        .unwrap();
        let code = "pub fn old()->u32{1}\n#[test] fn test(){                assert_eq!(old(),1);}";
        std::fs::write(root.join("src/lib.rs"), code).unwrap();
        let world = crate::RustWorld::open(
            &[("s".to_owned(), root.as_path())],
            &glassmind_rules::Switches::all_on(),
        );
        let item = world.sources[0]
            .records
            .iter()
            .find(|r| r.item.name == "old")
            .unwrap()
            .item
            .clone();
        let proposal = world
            .rename_at(
                item,
                None,
                &glassmind_meaning::Name("new_name".into()),
                false,
            )
            .unwrap();
        let changed = code.replace("                ", "let old=||1;    ");
        assert_eq!(code.len(), changed.len());
        assert_eq!(code.rfind("old()"), changed.rfind("old()"));
        std::fs::write(root.join("src/lib.rs"), &changed).unwrap();
        assert!(world.apply(&proposal, &mut Vec::new()).is_err());
        assert_eq!(
            std::fs::read_to_string(root.join("src/lib.rs")).unwrap(),
            changed
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
