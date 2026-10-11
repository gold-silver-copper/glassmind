//! Checked, source-preserving field addition over a closed package graph.
use crate::{
    RustWorld,
    checks::{self, RustChange, RustCheck},
    index::SourceIndex,
    rename_package::{Package, inert},
    rename_types::Module,
};
use std::collections::BTreeMap;
use syn::{parse::Parser, visit::Visit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddField {
    pub owner: String,
    pub field: String,
    pub ty: String,
    pub default: String,
}

#[derive(Debug, Clone)]
pub struct FieldAddition {
    pub action: AddField,
    pub changes: Vec<RustChange>,
    pub constructors: usize,
}

pub(crate) fn test_cfg(a: &syn::Attribute) -> bool {
    a.path().is_ident("cfg")
        && a.parse_args::<syn::Path>()
            .is_ok_and(|p| p.is_ident("test"))
}
pub(crate) fn standard_macro(name: &str) -> bool {
    matches!(
        name,
        "format"
            | "println"
            | "eprintln"
            | "print"
            | "eprint"
            | "vec"
            | "assert"
            | "assert_eq"
            | "assert_ne"
            | "panic"
    )
}
fn scalar(a: &AddField) -> Result<(), String> {
    for id in [&a.owner, &a.field] {
        if syn::parse_str::<syn::Ident>(id).is_err() || id.starts_with("r#") {
            return Err("ordinary struct and field identifiers required".into());
        }
    }
    let value = syn::parse_str::<syn::Expr>(&a.default)
        .map_err(|_| "explicit scalar literal default required")?;
    let literal = match &value {
        syn::Expr::Lit(v) => &v.lit,
        syn::Expr::Unary(v) if matches!(v.op, syn::UnOp::Neg(_)) => match &*v.expr {
            syn::Expr::Lit(v) => &v.lit,
            _ => return Err("default must be a literal".into()),
        },
        _ => {
            return Err(
                "default expressions with calls, blocks or side effects are unsupported".into(),
            );
        }
    };
    let valid = match a.ty.as_str() {
        "bool" => matches!(literal, syn::Lit::Bool(_)),
        "char" => matches!(literal, syn::Lit::Char(_)),
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128"
        | "isize" => matches!(literal, syn::Lit::Int(_)),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(
            "supported field types are primitive integers, bool and char with literal defaults"
                .into(),
        )
    }
}

impl RustWorld {
    pub fn propose_add_field(&self, action: &AddField) -> Result<FieldAddition, String> {
        scalar(action)?;
        let [source] = self.sources.as_slice() else {
            return Err("exactly one package required".into());
        };
        plan(source, action)
    }

    /// Recompute the plan, trial it, then apply typed changes with rollback and a fresh check.
    pub fn apply_add_field(&self, proposal: &FieldAddition) -> Result<usize, String> {
        let current = self.propose_add_field(&proposal.action)?;
        if current.changes != proposal.changes {
            return Err("source changed since proposal".into());
        }
        let source = self.sources.first().ok_or("missing source")?;
        let scratch = checks::scratch(&source.name);
        let trial: Result<(), String> = (|| {
            checks::copy_tree(&source.root, &scratch)?;
            checks::apply(
                &current.changes,
                &BTreeMap::from([(source.name.clone(), scratch.clone())]),
            )?;
            verify(&scratch, &source.name, &current)?;
            for check in [RustCheck::Build, RustCheck::Test] {
                if !checks::run_in(&checks::Places::default(), check, &source.name, &scratch)?
                    .is_empty()
                {
                    return Err(
                        "field addition failed compiler or existing tests in scratch copy".into(),
                    );
                }
            }
            Ok(())
        })();
        let _ = std::fs::remove_dir_all(scratch);
        trial?;
        let mut originals = BTreeMap::new();
        for change in &current.changes {
            let path = source.root.join(&change.file);
            originals.insert(
                path.clone(),
                Some(std::fs::read(path).map_err(|e| e.to_string())?),
            );
        }
        let lock = source.root.join("Cargo.lock");
        originals.insert(
            lock.clone(),
            if lock.exists() {
                Some(std::fs::read(&lock).map_err(|e| e.to_string())?)
            } else {
                None
            },
        );
        let applied = checks::apply(
            &current.changes,
            &BTreeMap::from([(source.name.clone(), source.root.clone())]),
        )
        .and_then(|count| {
            verify(&source.root, &source.name, &current)?;
            // Do not reuse cached scratch results as evidence about this root.
            if !checks::run_in(
                &checks::Places::default(),
                RustCheck::Test,
                &source.name,
                &source.root,
            )?
            .is_empty()
            {
                return Err("existing tests failed after applying the field; rolling back".into());
            }
            Ok(count)
        });
        if let Ok(mut found) = self.found.lock() {
            found.clear();
        }
        if applied.is_err() {
            for (path, bytes) in originals {
                match bytes {
                    Some(bytes) => {
                        std::fs::write(path, bytes).map_err(|e| format!("rollback failed: {e}"))?
                    }
                    None if path.exists() => {
                        std::fs::remove_file(path).map_err(|e| format!("rollback failed: {e}"))?
                    }
                    None => {}
                }
            }
        }
        applied
    }
}

fn plan(source: &SourceIndex, action: &AddField) -> Result<FieldAddition, String> {
    if !source.skipped.is_empty() {
        return Err("unparsed source files".into());
    }
    let package = Package::load_add_field(source)
        .ok_or("unsupported or ambiguous package/module/import graph")?;
    let mut selected = Vec::new();
    for module in &package.modules {
        for item in &module.items {
            if let syn::Item::Struct(s) = item
                && s.ident == action.owner
            {
                selected.push((module, s));
            }
        }
    }
    let [(module, declaration)] = selected.as_slice() else {
        return Err("struct name must identify exactly one declaration".into());
    };
    if !declaration.generics.params.is_empty() {
        return Err("generic target structs are unsupported".into());
    }
    let syn::Fields::Named(fields) = &declaration.fields else {
        return Err("named-field struct required".into());
    };
    if fields
        .named
        .iter()
        .any(|f| f.ident.as_ref().is_some_and(|n| n == &action.field))
    {
        return Err("field already exists".into());
    }
    for attr in &declaration.attrs {
        if attr.path().is_ident("derive")
            && attr
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .is_ok_and(|ps| ps.iter().any(|p| p.is_ident("Default")))
        {
            return Err("target Default derive would synthesize an unrequested default".into());
        }
    }
    let mut target = module.path.clone();
    target.push(action.owner.clone());
    let mut changes = vec![insertion(
        source,
        module.file,
        fields.brace_token.span.open().end(),
        format!(" pub {}: {},", action.field, action.ty),
    )?];
    let mut constructors = 0;
    for module in &package.modules {
        let mut visitor = Sites {
            package: &package,
            module,
            source,
            target: &target,
            action,
            verify: false,
            self_type: None,
            changes: &mut changes,
            count: &mut constructors,
            error: None,
        };
        for item in &module.items {
            visitor.visit_item(item);
        }
        if let Some(reason) = visitor.error {
            return Err(reason);
        }
    }
    changes.sort();
    changes.dedup();
    Ok(FieldAddition {
        action: action.clone(),
        changes,
        constructors,
    })
}
fn insertion(
    source: &SourceIndex,
    file: usize,
    at: proc_macro2::LineColumn,
    new: String,
) -> Result<RustChange, String> {
    Ok(RustChange {
        source: source.name.clone(),
        file: source.files.get(file).ok_or("missing source")?.clone(),
        line: at.line as u32,
        column: at.column as u32,
        old: String::new(),
        new,
    })
}
struct Sites<'a> {
    package: &'a Package,
    module: &'a Module,
    source: &'a SourceIndex,
    target: &'a [String],
    action: &'a AddField,
    self_type: Option<Vec<String>>,
    verify: bool,
    changes: &'a mut Vec<RustChange>,
    count: &'a mut usize,
    error: Option<String>,
}
impl Sites<'_> {
    fn refuse(&mut self, reason: &str) {
        self.error = Some(reason.into());
    }
    fn resolve(&self, path: &syn::Path) -> Option<Vec<String>> {
        if path.leading_colon.is_some() {
            return None;
        }
        let names: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        if names == ["Self"] {
            self.self_type.clone()
        } else {
            self.package.resolve(&self.module.path, &names)
        }
    }
}
impl<'ast> Visit<'ast> for Sites<'_> {
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}
    fn visit_item_use(&mut self, _: &'ast syn::ItemUse) {}
    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        if signature.unsafety.is_some() {
            self.refuse("unsafe functions may hide construction");
        }
        syn::visit::visit_signature(self, signature);
    }
    fn visit_item_type(&mut self, _: &'ast syn::ItemType) {
        self.refuse("type aliases may hide constructors");
    }
    fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
        self.refuse("item macros may generate constructors or bindings");
    }
    fn visit_expr_unsafe(&mut self, _: &'ast syn::ExprUnsafe) {
        self.refuse("unsafe construction cannot be accounted for");
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let old = self.self_type.clone();
        self.self_type = match &*item.self_ty {
            syn::Type::Path(p) if p.qself.is_none() => self.resolve(&p.path),
            _ => None,
        };
        syn::visit::visit_item_impl(self, item);
        self.self_type = old;
    }
    fn visit_type_param(&mut self, _: &'ast syn::TypeParam) {
        self.refuse("generic bindings are unsupported");
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        if block.stmts.iter().any(|s| matches!(s, syn::Stmt::Item(_))) {
            self.refuse("block-local items/imports may shadow constructor bindings");
        }
        syn::visit::visit_block(self, block);
    }
    fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
        let debug = a.path().is_ident("derive")
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
        if !inert(a) && !debug {
            self.refuse(
                "conditional or custom attributes are unsupported outside cfg(test) modules",
            );
        }
    }
    fn visit_expr_struct(&mut self, s: &'ast syn::ExprStruct) {
        match self.resolve(&s.path) {
            Some(path) if path == self.target => {
                if s.qself.is_some() || s.rest.is_some() {
                    self.refuse("qualified or struct-update constructors are unsupported");
                    return;
                }
                if self.verify {
                    let values: Vec<_> = s.fields.iter().filter(|f| matches!(&f.member, syn::Member::Named(n) if n == &self.action.field)).collect();
                    let expected = syn::parse_str::<syn::Expr>(&self.action.default).ok();
                    if values.len() != 1
                        || expected.is_none_or(|e| {
                            values.first().is_none_or(|value| {
                                let actual = &value.expr;
                                quote::quote!(#actual).to_string() != quote::quote!(#e).to_string()
                            })
                        })
                    {
                        self.refuse("constructor default postcondition failed");
                    }
                    *self.count += 1;
                    syn::visit::visit_expr_struct(self, s);
                    return;
                }
                match insertion(
                    self.source,
                    self.module.file,
                    s.brace_token.span.open().end(),
                    format!(" {}: {},", self.action.field, self.action.default),
                ) {
                    Ok(change) => {
                        self.changes.push(change);
                        *self.count += 1;
                    }
                    Err(e) => self.error = Some(e),
                }
            }
            None => self.refuse("unresolved struct literal constructor"),
            _ => {}
        }
        syn::visit::visit_expr_struct(self, s);
    }
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let name = m
            .path
            .get_ident()
            .map(ToString::to_string)
            .unwrap_or_default();
        if !standard_macro(&name) {
            self.refuse("unknown macro may construct the target");
            return;
        }
        let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
        match parser.parse2(m.tokens.clone()) {
            Ok(args) => {
                for arg in &args {
                    self.visit_expr(arg);
                }
            }
            Err(_) => self.refuse("macro arguments are outside supported expression-list syntax"),
        }
    }
}
fn verify(root: &std::path::Path, name: &str, proposal: &FieldAddition) -> Result<(), String> {
    let source = crate::index::index_source(name, root);
    let package = Package::load_add_field(&source).ok_or("post-edit package graph invalid")?;
    let mut found = 0;
    for module in &package.modules {
        for item in &module.items {
            if let syn::Item::Struct(s) = item
                && s.ident == proposal.action.owner
            {
                for field in &s.fields {
                    if field
                        .ident
                        .as_ref()
                        .is_some_and(|n| n == &proposal.action.field)
                    {
                        if !matches!(field.vis, syn::Visibility::Public(_)) {
                            return Err("field postcondition failed".into());
                        }
                        let ty = &field.ty;
                        if quote::quote!(#ty).to_string() != proposal.action.ty {
                            return Err("field type postcondition failed".into());
                        }
                        found += 1;
                    }
                }
            }
        }
    }
    if found != 1 {
        return Err("field declaration postcondition failed".into());
    }
    let mut count = 0;
    let mut ignored = Vec::new();
    let target = package
        .modules
        .iter()
        .find_map(|m| {
            m.items
                .iter()
                .any(|i| matches!(i, syn::Item::Struct(s) if s.ident == proposal.action.owner))
                .then(|| {
                    let mut p = m.path.clone();
                    p.push(proposal.action.owner.clone());
                    p
                })
        })
        .ok_or("target missing")?;
    for module in &package.modules {
        let mut visitor = Sites {
            package: &package,
            module,
            source: &source,
            target: &target,
            action: &proposal.action,
            verify: true,
            self_type: None,
            changes: &mut ignored,
            count: &mut count,
            error: None,
        };
        for item in &module.items {
            visitor.visit_item(item);
        }
        if let Some(error) = visitor.error {
            return Err(error);
        }
    }
    if count != proposal.constructors {
        return Err("constructor count postcondition failed".into());
    }
    // Every requested insertion must exist byte-for-byte at its original anchor;
    // checks::apply validates original spans and changes only these insertions.
    Ok(())
}
