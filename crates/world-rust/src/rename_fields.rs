//! Bounded owner-aware field positions. Every old-name token must be classified,
//! including tokens belonging to other structs, before any edit is proposed.
use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Ident;
use syn::visit::Visit;

use crate::index::{Pos, SourceIndex};
use crate::rename_package::{Package, inert, output_macro};
use crate::rename_types::{Module, position};
use crate::{RustItem, RustKind};

type Owner = Vec<String>;

pub(crate) fn positions(source: &SourceIndex, item: &RustItem, new: &str) -> Option<BTreeSet<Pos>> {
    if item.kind != RustKind::Field
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
    let package = Package::load(source)?;
    let mut owners = BTreeSet::new();
    let mut declarations = Vec::new();
    let mut target = None;
    let mut owner_count = 0;
    for m in &package.modules {
        for node in &m.items {
            if let syn::Item::Struct(s) = node {
                let mut owner = m.path.clone();
                owner.push(s.ident.to_string());
                owners.insert(owner.clone());
                if item.owner.as_deref() == Some(s.ident.to_string().as_str()) {
                    owner_count += 1;
                }
                for f in &s.fields {
                    if let Some(id) = &f.ident
                        && id == item.name.as_str()
                    {
                        let at = position(m.file, id);
                        if item.owner.as_deref() == Some(s.ident.to_string().as_str())
                            && source.files.get(m.file)? == &item.file
                            && at.1 == item.line
                            && target.replace(owner.clone()).is_some()
                        {
                            return None;
                        }
                        declarations.push((owner.clone(), at));
                    }
                }
            }
        }
    }
    if owner_count != 1 {
        return None;
    }
    let target = target?;
    let mut classified = BTreeSet::new();
    let mut marked = BTreeSet::new();
    for (owner, at) in declarations {
        classified.insert(at);
        if owner == target {
            marked.insert(at);
        }
    }
    for module in &package.modules {
        let mut visitor = References {
            package: &package,
            module,
            owners: &owners,
            target: &target,
            old: &item.name,
            parameters: BTreeMap::new(),
            classified: &mut classified,
            marked: &mut marked,
            valid: true,
        };
        for node in &module.items {
            visitor.visit_item(node);
        }
        if !visitor.valid {
            return None;
        }
    }
    (source.idents.get(&item.name)? == &classified).then_some(marked)
}

struct References<'a> {
    package: &'a Package,
    module: &'a Module,
    owners: &'a BTreeSet<Owner>,
    target: &'a Owner,
    old: &'a str,
    parameters: BTreeMap<String, Option<Owner>>,
    classified: &'a mut BTreeSet<Pos>,
    marked: &'a mut BTreeSet<Pos>,
    valid: bool,
}

impl References<'_> {
    fn owner(&self, path: &syn::Path) -> Option<Owner> {
        if path.leading_colon.is_some() || path.segments.iter().any(|s| !s.arguments.is_empty()) {
            return None;
        }
        let names: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        let owner = self.package.resolve(&self.module.path, &names)?;
        self.owners.contains(&owner).then_some(owner)
    }
    fn typed(&self, ty: &syn::Type) -> Option<Owner> {
        match ty {
            syn::Type::Path(p) if p.qself.is_none() => self.owner(&p.path),
            syn::Type::Reference(r) => self.typed(&r.elem),
            _ => None,
        }
    }
    fn receiver(&self, e: &syn::Expr) -> Option<Owner> {
        match e {
            syn::Expr::Struct(s) if s.qself.is_none() => self.owner(&s.path),
            syn::Expr::Path(p) if p.qself.is_none() => self
                .parameters
                .get(&p.path.get_ident()?.to_string())?
                .clone(),
            syn::Expr::Paren(p) => self.receiver(&p.expr),
            syn::Expr::Group(g) => self.receiver(&g.expr),
            syn::Expr::Reference(r) => self.receiver(&r.expr),
            _ => None,
        }
    }
    fn classify(&mut self, member: &syn::Member, owner: Option<Owner>) {
        if let syn::Member::Named(id) = member
            && id == self.old
        {
            if let Some(owner) = owner {
                let at = position(self.module.file, id);
                self.classified.insert(at);
                if &owner == self.target {
                    self.marked.insert(at);
                }
            } else {
                self.valid = false;
            }
        }
    }
}

impl<'ast> Visit<'ast> for References<'_> {
    // Modules are visited once, in the package graph's lexical context.
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}
    fn visit_item_use(&mut self, _: &'ast syn::ItemUse) {
        // Imports/aliases are outside this first closed subset.
        self.valid = false;
    }
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        let saved = std::mem::take(&mut self.parameters);
        for a in &f.attrs {
            self.visit_attribute(a);
        }
        self.visit_signature(&f.sig);
        for input in &f.sig.inputs {
            let syn::FnArg::Typed(p) = input else {
                self.valid = false;
                continue;
            };
            let syn::Pat::Ident(id) = p.pat.as_ref() else {
                self.valid = false;
                continue;
            };
            if id.by_ref.is_some() || id.subpat.is_some() {
                self.valid = false;
            }
            self.parameters
                .insert(id.ident.to_string(), self.typed(&p.ty));
        }
        self.visit_block(&f.block);
        self.parameters = saved;
    }
    fn visit_expr_field(&mut self, f: &'ast syn::ExprField) {
        self.classify(&f.member, self.receiver(&f.base));
        syn::visit::visit_expr_field(self, f);
    }
    fn visit_expr_struct(&mut self, s: &'ast syn::ExprStruct) {
        let owner = s.qself.is_none().then(|| self.owner(&s.path)).flatten();
        for f in &s.fields {
            // Shorthand couples the field token to a local binding: refuse.
            if f.colon_token.is_some() {
                self.classify(&f.member, owner.clone());
            }
        }
        syn::visit::visit_expr_struct(self, s);
    }
    fn visit_pat_ident(&mut self, p: &'ast syn::PatIdent) {
        // No inference across shadowing, even in a nested lexical scope.
        if self.parameters.contains_key(&p.ident.to_string()) {
            self.valid = false;
        }
        syn::visit::visit_pat_ident(self, p);
    }
    fn visit_generics(&mut self, g: &'ast syn::Generics) {
        if !g.params.is_empty() || g.where_clause.is_some() {
            self.valid = false;
        }
        syn::visit::visit_generics(self, g);
    }
    fn visit_block(&mut self, b: &'ast syn::Block) {
        if b.stmts.iter().any(|s| matches!(s, syn::Stmt::Item(_))) {
            self.valid = false;
        }
        syn::visit::visit_block(self, b);
    }
    fn visit_item_impl(&mut self, _: &'ast syn::ItemImpl) {
        self.valid = false;
    }
    fn visit_item_type(&mut self, _: &'ast syn::ItemType) {
        self.valid = false;
    }
    fn visit_item_trait(&mut self, _: &'ast syn::ItemTrait) {
        self.valid = false;
    }
    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {
        self.valid = false;
    }
    fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
        self.valid = false;
    }
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if !output_macro(m) {
            self.valid = false;
        }
    }
    fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
        if !inert(a) {
            self.valid = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(label: &str, code: &str) -> Option<BTreeSet<Pos>> {
        let root =
            std::env::temp_dir().join(format!("field-owners-{label}-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), code).unwrap();
        let source = crate::index::index_source("sample", &root);
        let item = source
            .records
            .iter()
            .find(|r| r.item.kind == RustKind::Field && r.item.owner.as_deref() == Some("Parcel"))
            .unwrap();
        let result = positions(&source, &item.item, "total");
        std::fs::remove_dir_all(root).unwrap();
        result
    }

    #[test]
    fn separates_declarations_literals_and_typed_references() {
        let code = "struct Parcel { count: u32 } struct Ledger { count: u32 } fn f(x: &Parcel, y: &Ledger) -> u32 { x.count + y.count + Parcel { count: 3 }.count + Ledger { count: 7 }.count }";
        assert_eq!(check("bound", code).unwrap().len(), 4);
    }

    #[test]
    fn refuses_unclassified_tokens_and_receiver_uncertainty() {
        let prefix = "struct Parcel { count: u32 } struct Ledger { count: u32 } ";
        for (label, suffix) in [
            (
                "shadow",
                "fn f(x: Parcel) { let x = Ledger { count: 2 }; let _ = x.count; }",
            ),
            (
                "call",
                "fn make() -> Parcel { Parcel { count: 2 } } fn f() { let _ = make().count; }",
            ),
            (
                "shorthand",
                "fn f(count: u32) { let _ = Parcel { count }; }",
            ),
            (
                "pattern",
                "fn f(x: Parcel) { let Parcel { count: n } = x; }",
            ),
            ("alias", "type Alias = Parcel;"),
            ("generic", "fn f<Parcel>() {}"),
            (
                "macro",
                "fn f() { println!(\"{}\", Parcel { count: 2 }.count); }",
            ),
            ("impl", "impl Parcel { fn f(&self) -> u32 { self.count } }"),
            ("raw", "fn f(x: Parcel) { let _ = x.r#count; }"),
            ("unknown", "fn f(x: External) { let _ = x.count; }"),
            ("owner", "mod nested { struct Parcel { count: u32 } }"),
        ] {
            assert!(
                check(label, &format!("{prefix}{suffix}")).is_none(),
                "{label}"
            );
        }
    }
}
