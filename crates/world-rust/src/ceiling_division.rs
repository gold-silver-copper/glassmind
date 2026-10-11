//! Conservative, syntax-resolved primitive rounding proposals.
use crate::checks::RustChange;
use std::collections::{BTreeMap, BTreeSet};
use syn::{spanned::Spanned, visit::Visit};

fn unsigned(name: &str) -> bool {
    matches!(name, "u8" | "u16" | "u32" | "u64" | "u128" | "usize")
}
fn name(ty: &syn::Type) -> Option<String> {
    let syn::Type::Path(p) = ty else { return None };
    if p.qself.is_some() {
        return None;
    }
    Some(p.path.get_ident()?.to_string())
}
fn plain(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().all(|a| a.path().is_ident("doc"))
}

pub(crate) fn proposals(
    source: &str,
    file: &str,
    text: &str,
    old: &[RustChange],
) -> Vec<RustChange> {
    // Only roots: parent modules could shadow primitive/struct names.
    if !matches!(file, "src/lib.rs" | "src/main.rs") {
        return Vec::new();
    }
    let Ok(tree) = syn::parse_file(text) else {
        return Vec::new();
    };
    if !plain(&tree.attrs) {
        return Vec::new();
    }
    let mut fields = BTreeMap::new();
    let mut declared = BTreeSet::new();
    for item in &tree.items {
        match item {
            syn::Item::Fn(_) => {}
            syn::Item::Struct(s)
                if plain(&s.attrs)
                    && s.generics.params.is_empty()
                    && s.generics.where_clause.is_none() =>
            {
                let key = s.ident.to_string();
                if unsigned(&key) || !declared.insert(key.clone()) {
                    return Vec::new();
                }
                for f in &s.fields {
                    if plain(&f.attrs)
                        && let (Some(id), Some(t)) = (&f.ident, name(&f.ty))
                        && unsigned(&t)
                    {
                        fields.insert((key.clone(), id.to_string()), t);
                    }
                }
            }
            syn::Item::Mod(m) if super::slips::test_only(&m.attrs) => {}
            syn::Item::Mod(m) if plain(&m.attrs) && !unsigned(&m.ident.to_string()) => {}
            // Imports, aliases, macros and impls need compiler name resolution.
            _ => return Vec::new(),
        }
    }
    let allowed: BTreeSet<_> = old
        .iter()
        .filter(|c| c.old == "/")
        .map(|c| (c.line, c.column))
        .collect();
    let mut changes = Vec::new();
    for item in &tree.items {
        let syn::Item::Fn(f) = item else { continue };
        if !plain(&f.attrs)
            || !f.sig.generics.params.is_empty()
            || f.sig.generics.where_clause.is_some()
        {
            continue;
        }
        let mut bindings = BTreeMap::new();
        for arg in &f.sig.inputs {
            let syn::FnArg::Typed(a) = arg else { continue };
            let syn::Pat::Ident(p) = &*a.pat else {
                continue;
            };
            if !a.attrs.is_empty()
                || !p.attrs.is_empty()
                || p.by_ref.is_some()
                || p.subpat.is_some()
            {
                continue;
            }
            let ty = match &*a.ty {
                syn::Type::Reference(r) => name(&r.elem).filter(|n| declared.contains(n)),
                t => name(t),
            };
            if let Some(t) = ty {
                bindings.insert(p.ident.to_string(), t);
            }
        }
        // Any local binding of the same name makes it unresolved, even in a
        // disjoint branch. Nested items, closures and macros are opaque.
        struct Locals {
            names: BTreeSet<String>,
            opaque: bool,
        }
        impl<'a> Visit<'a> for Locals {
            fn visit_pat_ident(&mut self, p: &'a syn::PatIdent) {
                self.names.insert(p.ident.to_string());
                syn::visit::visit_pat_ident(self, p);
            }
            fn visit_item(&mut self, _: &'a syn::Item) {
                self.opaque = true;
            }
            fn visit_macro(&mut self, _: &'a syn::Macro) {
                self.opaque = true;
            }
            fn visit_expr_closure(&mut self, _: &'a syn::ExprClosure) {
                self.opaque = true;
            }
        }
        let mut locals = Locals {
            names: BTreeSet::new(),
            opaque: false,
        };
        locals.visit_block(&f.block);
        if locals.opaque {
            continue;
        }
        bindings.retain(|k, _| !locals.names.contains(k));
        let mut v = Quotients {
            source,
            file,
            text,
            bindings: &bindings,
            fields: &fields,
            allowed: &allowed,
            changes: &mut changes,
        };
        v.visit_block(&f.block);
    }
    changes
}
struct Quotients<'a> {
    source: &'a str,
    file: &'a str,
    text: &'a str,
    bindings: &'a BTreeMap<String, String>,
    fields: &'a BTreeMap<(String, String), String>,
    allowed: &'a BTreeSet<(u32, u32)>,
    changes: &'a mut Vec<RustChange>,
}
impl Quotients<'_> {
    fn ty(&self, e: &syn::Expr) -> Option<String> {
        match e {
            syn::Expr::Paren(p) if p.attrs.is_empty() => self.ty(&p.expr),
            syn::Expr::Path(p) if p.attrs.is_empty() && p.qself.is_none() => {
                self.bindings.get(&p.path.get_ident()?.to_string()).cloned()
            }
            syn::Expr::Field(f) if f.attrs.is_empty() => {
                let syn::Member::Named(member) = &f.member else {
                    return None;
                };
                self.fields
                    .get(&(self.ty(&f.base)?, member.to_string()))
                    .cloned()
            }
            syn::Expr::Lit(l) if l.attrs.is_empty() => {
                let syn::Lit::Int(n) = &l.lit else {
                    return None;
                };
                unsigned(n.suffix()).then(|| n.suffix().to_owned())
            }
            _ => None,
        }
    }
    fn literal(e: &syn::Expr) -> bool {
        match e {
            syn::Expr::Paren(p) if p.attrs.is_empty() => Self::literal(&p.expr),
            syn::Expr::Lit(l) if l.attrs.is_empty() => {
                matches!(&l.lit,syn::Lit::Int(i) if i.suffix().is_empty())
            }
            _ => false,
        }
    }
    fn text(&self, s: proc_macro2::Span) -> Option<String> {
        let (a, b) = (s.start(), s.end());
        if a.line != b.line {
            return None;
        }
        Some(
            self.text
                .lines()
                .nth(a.line.checked_sub(1)?)?
                .chars()
                .skip(a.column)
                .take(b.column.checked_sub(a.column)?)
                .collect(),
        )
    }
    fn proposal(&self, b: &syn::ExprBinary) -> Option<RustChange> {
        let pos = b.op.span().start();
        if !matches!(b.op, syn::BinOp::Div(_))
            || !self.allowed.contains(&(pos.line as u32, pos.column as u32))
        {
            return None;
        }
        let ty = match (self.ty(&b.left), self.ty(&b.right)) {
            (Some(l), Some(r)) if l == r => l,
            (Some(l), None) if Self::literal(&b.right) => l,
            (None, Some(r)) if Self::literal(&b.left) => r,
            _ => return None,
        };
        if !unsigned(&ty) {
            return None;
        }
        let old = self.text(b.span())?;
        let left = self.text(b.left.span())?;
        let right = self.text(b.right.span())?;
        let separator = old.strip_prefix(&left)?.strip_suffix(&right)?;
        if separator.trim() != "/" {
            return None;
        }
        let start = b.span().start();
        Some(RustChange {
            source: self.source.into(),
            file: self.file.into(),
            line: start.line as u32,
            column: start.column as u32,
            old,
            new: format!("::core::primitive::{ty}::div_ceil({}, {})", left, right),
        })
    }
}
impl<'a> Visit<'a> for Quotients<'_> {
    fn visit_type(&mut self, _: &'a syn::Type) {}
    fn visit_path_arguments(&mut self, _: &'a syn::PathArguments) {}
    fn visit_expr_binary(&mut self, b: &'a syn::ExprBinary) {
        if let Some(c) = self.proposal(b) {
            self.changes.push(c);
        }
        syn::visit::visit_expr_binary(self, b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn get(text: &str) -> Vec<RustChange> {
        let old = crate::slips::proposals("s", "src/lib.rs", text);
        proposals("s", "src/lib.rs", text, &old)
    }
    #[test]
    fn resolves_only_concrete_unsigned_operands() {
        for ty in ["u8", "u16", "u32", "u64", "u128", "usize"] {
            let code = format!("fn f(café:{ty}, d:{ty})->{ty} {{ café / d }}");
            let changes = get(&code);
            assert_eq!(changes.len(), 1);
            assert_eq!(changes[0].old, "café / d");
            assert_eq!(
                changes[0].new,
                format!("::core::primitive::{ty}::div_ceil(café, d)")
            );
            let patched = crate::slips::ambiguity::variant(&code, &changes[0]).unwrap();
            assert!(patched.contains(&changes[0].new));
        }
        assert_eq!(get("fn f()->u32 { 7u32 / 3 }").len(), 1);
        assert_eq!(
            get("struct S { a:u32, b:u32 } fn f(s:&S)->u32 { s.a / s.b }").len(),
            1
        );
        for code in [
            "fn f(n:i32)->i32 { n / 3 }",
            "fn f()->u32 { 7 / 3 }",
            "fn f(n:u32)->u32 { let n=7; n / 3 }",
            "fn f(n:u32)->u32 { n /* keep */ / 3 }",
            "type u32 = u16; fn f(n:u32)->u32 { n / 3 }",
            "use custom::u32; fn f(n:u32)->u32 { n / 3 }",
            "struct u32; fn f(n:u32)->u32 { n / 3 }",
            "fn f<u32>(n:u32)->u32 { n / 3 }",
            "fn f(n:u32)->u32 { shadow!(); n / 3 }",
            "fn f(n:u32)->u32 { m!(n / 3) }",
            "fn f(n:u32)->u32 { n.div_ceil(3) }",
            "#[test] fn f(n:u32)->u32 { n / 3 }",
            "#[cfg(test)] mod tests { fn f(n:u32)->u32 { n / 3 } }",
            "fn f(n:u32)->u32 { #[cfg(test)] let q=n / 3; n }",
            "fn f(n:u32)->u32 { g(n) / 3 }",
            "fn f(n:u32)->u32 { n / 3u16 }",
            "fn f() { let _: [u8; 7usize / 3]; }",
        ] {
            assert!(get(code).is_empty(), "{code}");
        }
    }
}
