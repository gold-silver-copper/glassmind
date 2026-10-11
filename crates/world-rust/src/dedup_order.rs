//! Typed pipeline-order proposals, not a proof of intended semantics.
//! Native public tests are not proof of intended semantics either. Only a
//! complete, unique native-test-passing choice may be offered by checks.rs.
use crate::checks::RustChange;
use std::collections::BTreeMap;
use syn::{ext::IdentExt, spanned::Spanned, visit::Visit};

const ELEMENTS: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "bool",
    "char", "String",
];
fn docs(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().all(|a| a.path().is_ident("doc"))
}
fn inline_tests(item: &syn::Item) -> bool {
    let syn::Item::Mod(m) = item else {
        return false;
    };
    let test = |a: &syn::Attribute| {
        a.path().is_ident("cfg")
            && a.parse_args::<syn::Path>()
                .is_ok_and(|p| p.is_ident("test"))
    };
    m.content.is_some()
        && m.attrs.iter().any(test)
        && m.attrs.iter().all(|a| a.path().is_ident("doc") || test(a))
}
fn vector(t: &syn::Type) -> bool {
    let syn::Type::Path(p) = t else { return false };
    if p.qself.is_some() || p.path.leading_colon.is_some() || p.path.segments.len() != 1 {
        return false;
    }
    let Some(s) = p.path.segments.first() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(args) = &s.arguments else {
        return false;
    };
    if s.ident != "Vec" || args.colon2_token.is_some() {
        return false;
    }
    let [syn::GenericArgument::Type(syn::Type::Path(e))] =
        args.args.iter().collect::<Vec<_>>().as_slice()
    else {
        return false;
    };
    e.qself.is_none()
        && e.path
            .get_ident()
            .is_some_and(|i| ELEMENTS.contains(&i.to_string().as_str()))
}
fn binding(p: &syn::Pat) -> Option<String> {
    let syn::Pat::Ident(p) = p else { return None };
    (p.attrs.is_empty() && p.by_ref.is_none() && p.subpat.is_none() && p.mutability.is_some())
        .then(|| p.ident.unraw().to_string())
}
fn receiver<'a>(stmt: &'a syn::Stmt, method: &str) -> Option<&'a syn::Ident> {
    let syn::Stmt::Expr(syn::Expr::MethodCall(c), Some(_)) = stmt else {
        return None;
    };
    if !c.attrs.is_empty() || c.method != method || !c.args.is_empty() || c.turbofish.is_some() {
        return None;
    }
    let syn::Expr::Path(p) = &*c.receiver else {
        return None;
    };
    (p.attrs.is_empty() && p.qself.is_none()).then_some(())?;
    p.path.get_ident()
}
fn offset(text: &str, pos: proc_macro2::LineColumn) -> Option<usize> {
    let row = pos.line.checked_sub(1)?;
    let line = text.split_inclusive('\n').nth(row)?;
    let column = line
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(line.len()))
        .nth(pos.column)?;
    Some(
        text.split_inclusive('\n')
            .take(row)
            .map(str::len)
            .sum::<usize>()
            + column,
    )
}
fn reordered(source: &str, file: &str, text: &str, pair: &[syn::Stmt]) -> Option<RustChange> {
    let [first, second] = pair else { return None };
    let first = first.span();
    let second = second.span();
    let (a, b, c, d) = (
        offset(text, first.start())?,
        offset(text, first.end())?,
        offset(text, second.start())?,
        offset(text, second.end())?,
    );
    Some(RustChange {
        source: source.into(),
        file: file.into(),
        line: first.start().line as u32,
        column: first.start().column as u32,
        old: text.get(a..d)?.into(),
        new: format!("{}{}{}", text.get(c..d)?, text.get(b..c)?, text.get(a..b)?),
    })
}

pub(crate) fn proposals(source: &str, file: &str, text: &str) -> Vec<RustChange> {
    if !matches!(file, "src/lib.rs" | "src/main.rs") {
        return vec![];
    }
    let Ok(tree) = syn::parse_file(text) else {
        return vec![];
    };
    // Closed enclosing namespace, matching extremum's inline-test restriction.
    // No production expansion, imports, aliases, custom types or nested items
    // can redefine builtin/prelude anchors. Test-module names stay local.
    struct Namespace {
        opaque: bool,
        depth: usize,
    }
    impl<'a> Visit<'a> for Namespace {
        fn visit_attribute(&mut self, a: &'a syn::Attribute) {
            if !docs(std::slice::from_ref(a)) {
                self.opaque = true;
            }
        }
        fn visit_macro(&mut self, _: &'a syn::Macro) {
            self.opaque = true;
        }
        fn visit_item(&mut self, i: &'a syn::Item) {
            if self.depth == 0 && inline_tests(i) {
                return;
            }
            let syn::Item::Fn(f) = i else {
                self.opaque = true;
                return;
            };
            let name = f.sig.ident.to_string();
            if self.depth != 0 || name == "Vec" || ELEMENTS.contains(&name.as_str()) {
                self.opaque = true;
            }
            self.depth += 1;
            syn::visit::visit_item(self, i);
            self.depth -= 1;
        }
    }
    let mut namespace = Namespace {
        opaque: false,
        depth: 0,
    };
    namespace.visit_file(&tree);
    if namespace.opaque {
        return vec![];
    }
    let mut changes = Vec::new();
    for item in &tree.items {
        let syn::Item::Fn(f) = item else { continue };
        let s = &f.sig;
        if !docs(&f.attrs)
            || !s.generics.params.is_empty()
            || s.generics.where_clause.is_some()
            || s.unsafety.is_some()
            || s.asyncness.is_some()
            || s.constness.is_some()
            || s.abi.is_some()
            || s.variadic.is_some()
        {
            continue;
        }
        // Count every binding, including closures/nested blocks: do not infer
        // flow through shadowing. Only unique direct bindings can anchor a pair.
        #[derive(Default)]
        struct Bindings(BTreeMap<String, usize>);
        impl<'a> Visit<'a> for Bindings {
            fn visit_pat_ident(&mut self, p: &'a syn::PatIdent) {
                *self.0.entry(p.ident.unraw().to_string()).or_default() += 1;
                syn::visit::visit_pat_ident(self, p);
            }
        }
        let mut bindings = Bindings::default();
        bindings.visit_item_fn(f);
        let mut known = Vec::new();
        for a in &s.inputs {
            if let syn::FnArg::Typed(a) = a
                && a.attrs.is_empty()
                && vector(&a.ty)
                && let Some(name) = binding(&a.pat)
            {
                known.push(name);
            }
        }
        for (i, stmt) in f.block.stmts.iter().enumerate() {
            if let syn::Stmt::Local(l) = stmt
                && l.attrs.is_empty()
                && l.init.as_ref().is_some_and(|init| init.diverge.is_none())
                && let syn::Pat::Type(p) = &l.pat
                && p.attrs.is_empty()
                && vector(&p.ty)
                && let Some(name) = binding(&p.pat)
            {
                known.push(name);
            }
            let Some(pair) = f.block.stmts.get(i..i + 2) else {
                continue;
            };
            let [first, second] = pair else { continue };
            let Some(name) = receiver(first, "dedup") else {
                continue;
            };
            if receiver(second, "sort") != Some(name) {
                continue;
            }
            let name = name.unraw().to_string();
            if !known.contains(&name) || bindings.0.get(&name) != Some(&1) {
                continue;
            }
            if let Some(change) = reordered(source, file, text, pair) {
                changes.push(change);
            }
        }
    }
    changes
}
