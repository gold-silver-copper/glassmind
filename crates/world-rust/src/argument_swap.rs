//! Resolved pure scalar call arguments; never use names as repair templates.
use crate::checks::RustChange;
use std::collections::{BTreeMap, BTreeSet};
use syn::{parse::Parser, spanned::Spanned, visit::Visit};

fn primitive(s: &str) -> bool {
    matches!(
        s,
        "u8" | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "f32"
            | "f64"
            | "bool"
            | "char"
    )
}
fn name(t: &syn::Type) -> Option<String> {
    let syn::Type::Path(p) = t else { return None };
    if p.qself.is_some() {
        return None;
    }
    Some(p.path.get_ident()?.to_string())
}
fn plain(a: &[syn::Attribute]) -> bool {
    a.iter().all(|a| a.path().is_ident("doc"))
}
// Only built-in test gating is safe to ignore at the root. An unrelated
// procedural attribute could introduce aliases or other bindings into this file.
fn inert_test_attrs(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().all(|a| {
        a.path().is_ident("doc")
            || (a.path().is_ident("test") && matches!(a.meta, syn::Meta::Path(_)))
            || (a.path().is_ident("cfg")
                && a.parse_args::<syn::Path>()
                    .is_ok_and(|p| p.is_ident("test")))
    })
}
fn function(f: &syn::ItemFn) -> bool {
    plain(&f.attrs)
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.unsafety.is_none()
        && f.sig.asyncness.is_none()
        && f.sig.abi.is_none()
        && f.sig.variadic.is_none()
}
// Parsing expressions preserves original spans. Only the standard, simple
// positional subset is supported; captured/named/dynamic format syntax is opaque.
fn format_args(m: &syn::Macro) -> Option<Vec<syn::Expr>> {
    let mut segments = m.path.segments.iter();
    let qualified = m.path.leading_colon.is_some()
        && matches!(
            (segments.next(), segments.next(), segments.next()),
            (Some(namespace), Some(name), None)
                if namespace.ident == "std"
                    && name.ident == "format"
                    && matches!(namespace.arguments, syn::PathArguments::None)
                    && matches!(name.arguments, syn::PathArguments::None)
        );
    if !m.path.is_ident("format") && !qualified {
        return None;
    }
    let args = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
        .parse2(m.tokens.clone())
        .ok()?;
    let mut args = args.into_iter();
    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        attrs,
        ..
    }) = args.next()?
    else {
        return None;
    };
    if !attrs.is_empty() {
        return None;
    }
    let mut chars = s.value().chars().collect::<Vec<_>>().into_iter();
    let mut slots = 0;
    while let Some(c) = chars.next() {
        match c {
            '{' => match chars.next()? {
                '{' => {}
                '}' => slots += 1,
                _ => return None,
            },
            '}' if chars.next()? != '}' => return None,
            _ => {}
        }
    }
    let args: Vec<_> = args.collect();
    if slots != args.len() || args.iter().any(|e| matches!(e, syn::Expr::Assign(_))) {
        return None;
    }
    Some(args)
}

pub(crate) fn proposals(source: &str, file: &str, text: &str) -> Vec<RustChange> {
    if !matches!(file, "src/lib.rs" | "src/main.rs") {
        return Vec::new();
    }
    let Ok(tree) = syn::parse_file(text) else {
        return Vec::new();
    };
    if !plain(&tree.attrs) {
        return Vec::new();
    }
    let mut functions = BTreeMap::new();
    let mut fields = BTreeMap::new();
    let mut declared = BTreeSet::new();
    // Even cfg(test) modules can export macros into the root during native
    // verification. Never assume their macro namespace is inert.
    struct MacroBindings(bool);
    impl<'a> Visit<'a> for MacroBindings {
        fn visit_item_macro(&mut self, _: &'a syn::ItemMacro) {
            self.0 = true;
        }
        fn visit_macro(&mut self, m: &'a syn::Macro) {
            // An opaque invocation (including include!) may introduce exported
            // macro bindings even when it sits in a different function.
            let args = format_args(m).or_else(|| {
                if !["assert", "assert_eq", "assert_ne"]
                    .iter()
                    .any(|name| m.path.is_ident(name))
                {
                    return None;
                }
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
                    .parse2(m.tokens.clone())
                    .ok()
                    .map(|args| args.into_iter().collect())
            });
            match args {
                Some(args) => {
                    // An expression block inside even a standard macro may
                    // contain a macro_export declaration or an opaque import.
                    for arg in &args {
                        self.visit_expr(arg);
                    }
                }
                None => self.0 = true,
            }
        }
        fn visit_item_use(&mut self, item: &'a syn::ItemUse) {
            if !item.attrs.is_empty()
                || item.leading_colon.is_some()
                || !matches!(&item.tree, syn::UseTree::Path(p)
                    if p.ident == "super" && matches!(&*p.tree, syn::UseTree::Glob(_)))
            {
                self.0 = true;
            }
        }
        fn visit_item_mod(&mut self, m: &'a syn::ItemMod) {
            if m.content.is_none() {
                self.0 = true;
            }
            syn::visit::visit_item_mod(self, m);
        }
        fn visit_attribute(&mut self, a: &'a syn::Attribute) {
            if !inert_test_attrs(std::slice::from_ref(a)) {
                self.0 = true;
            }
        }
    }
    let mut macros = MacroBindings(false);
    macros.visit_file(&tree);
    let mut formatting = !macros.0;
    for item in &tree.items {
        match item {
            syn::Item::Fn(f) => {
                if !inert_test_attrs(&f.attrs) {
                    return Vec::new();
                }
                let key = f.sig.ident.to_string();
                if primitive(&key) || !declared.insert(key.clone()) {
                    return Vec::new();
                }
                if function(f) && f.sig.inputs.len() == 2 {
                    let types: Option<Vec<_>> = f
                        .sig
                        .inputs
                        .iter()
                        .map(|a| {
                            let syn::FnArg::Typed(a) = a else { return None };
                            if !a.attrs.is_empty() {
                                return None;
                            }
                            name(&a.ty).filter(|t| primitive(t))
                        })
                        .collect();
                    if let Some(types) = types
                        && types.first() == types.last()
                        && let Some(t) = types.first()
                    {
                        functions.insert(key, t.clone());
                    }
                }
            }
            syn::Item::Struct(s)
                if plain(&s.attrs)
                    && s.generics.params.is_empty()
                    && s.generics.where_clause.is_none() =>
            {
                let key = s.ident.to_string();
                if primitive(&key) || key == "std" || !declared.insert(key.clone()) {
                    return Vec::new();
                }
                for f in &s.fields {
                    if plain(&f.attrs)
                        && let (Some(id), Some(t)) = (&f.ident, name(&f.ty))
                        && primitive(&t)
                    {
                        fields.insert((key.clone(), id.to_string()), t);
                    }
                }
            }
            syn::Item::Mod(m)
                if crate::slips::test_only(&m.attrs) && inert_test_attrs(&m.attrs) => {}
            syn::Item::Mod(m)
                if plain(&m.attrs) && m.content.is_none() && !primitive(&m.ident.to_string()) =>
            {
                formatting = false;
            }
            _ => return Vec::new(),
        }
    }
    let mut changes = Vec::new();
    for item in &tree.items {
        let syn::Item::Fn(f) = item else { continue };
        if !function(f) {
            continue;
        }
        let mut bindings = BTreeMap::new();
        let mut local = Scope {
            names: BTreeSet::new(),
            opaque: false,
            formatting,
        };
        for a in &f.sig.inputs {
            let syn::FnArg::Typed(a) = a else {
                local.opaque = true;
                continue;
            };
            let syn::Pat::Ident(p) = &*a.pat else {
                local.opaque = true;
                continue;
            };
            if !a.attrs.is_empty()
                || !p.attrs.is_empty()
                || p.by_ref.is_some()
                || p.subpat.is_some()
            {
                local.opaque = true;
            }
            let ty = match &*a.ty {
                syn::Type::Reference(r) => name(&r.elem).filter(|t| !primitive(t)),
                t => name(t),
            };
            // A parameter can shadow the callee as well as operand bindings.
            if functions.contains_key(&p.ident.to_string()) {
                local.opaque = true;
            }
            if let Some(t) = ty {
                bindings.insert(p.ident.to_string(), t);
            }
        }
        local.visit_block(&f.block);
        if local.opaque {
            continue;
        }
        bindings.retain(|n, _| !local.names.contains(n));
        let mut visitor = Calls {
            source,
            file,
            text,
            functions: &functions,
            bindings: &bindings,
            fields: &fields,
            shadowed: &local.names,
            changes: &mut changes,
        };
        visitor.visit_block(&f.block);
    }
    changes
}
struct Scope {
    names: BTreeSet<String>,
    opaque: bool,
    formatting: bool,
}
impl<'a> Visit<'a> for Scope {
    fn visit_pat_ident(&mut self, p: &'a syn::PatIdent) {
        self.names.insert(p.ident.to_string());
        syn::visit::visit_pat_ident(self, p);
    }
    fn visit_attribute(&mut self, _: &'a syn::Attribute) {
        self.opaque = true;
    }
    fn visit_item(&mut self, _: &'a syn::Item) {
        self.opaque = true;
    }
    fn visit_expr_closure(&mut self, _: &'a syn::ExprClosure) {
        self.opaque = true;
    }
    fn visit_expr_unsafe(&mut self, _: &'a syn::ExprUnsafe) {
        self.opaque = true;
    }
    fn visit_expr_async(&mut self, _: &'a syn::ExprAsync) {
        self.opaque = true;
    }
    fn visit_expr_for_loop(&mut self, _: &'a syn::ExprForLoop) {
        self.opaque = true;
    }
    fn visit_expr_loop(&mut self, _: &'a syn::ExprLoop) {
        self.opaque = true;
    }
    fn visit_expr_while(&mut self, _: &'a syn::ExprWhile) {
        self.opaque = true;
    }
    fn visit_expr_match(&mut self, _: &'a syn::ExprMatch) {
        self.opaque = true;
    }
    fn visit_macro(&mut self, m: &'a syn::Macro) {
        if self.formatting
            && let Some(args) = format_args(m)
        {
            for a in &args {
                self.visit_expr(a);
            }
        } else {
            self.opaque = true;
        }
    }
}
struct Calls<'a> {
    source: &'a str,
    file: &'a str,
    text: &'a str,
    functions: &'a BTreeMap<String, String>,
    bindings: &'a BTreeMap<String, String>,
    fields: &'a BTreeMap<(String, String), String>,
    shadowed: &'a BTreeSet<String>,
    changes: &'a mut Vec<RustChange>,
}
impl Calls<'_> {
    fn untyped_integer(e: &syn::Expr) -> bool {
        match e {
            syn::Expr::Lit(l) if l.attrs.is_empty() => {
                matches!(&l.lit, syn::Lit::Int(i) if i.suffix().is_empty())
            }
            syn::Expr::Paren(p) if p.attrs.is_empty() => Self::untyped_integer(&p.expr),
            _ => false,
        }
    }
    fn ty(&self, e: &syn::Expr) -> Option<String> {
        match e {
            syn::Expr::Path(p) if p.qself.is_none() => self
                .bindings
                .get(&p.path.get_ident()?.to_string())
                .filter(|t| primitive(t))
                .cloned(),
            syn::Expr::Field(f) => {
                let syn::Expr::Path(p) = &*f.base else {
                    return None;
                };
                if p.qself.is_some() {
                    return None;
                }
                let syn::Member::Named(field) = &f.member else {
                    return None;
                };
                let t = self.bindings.get(&p.path.get_ident()?.to_string())?;
                self.fields.get(&(t.clone(), field.to_string())).cloned()
            }
            syn::Expr::Lit(l) => match &l.lit {
                syn::Lit::Int(i) if primitive(i.suffix()) => Some(i.suffix().into()),
                syn::Lit::Float(i) if matches!(i.suffix(), "f32" | "f64") => {
                    Some(i.suffix().into())
                }
                syn::Lit::Bool(_) => Some("bool".into()),
                syn::Lit::Char(_) => Some("char".into()),
                _ => None,
            },
            syn::Expr::Paren(p) => self.ty(&p.expr),
            _ => None,
        }
    }
    fn offset(&self, p: proc_macro2::LineColumn) -> Option<usize> {
        let line = self
            .text
            .split_inclusive('\n')
            .nth(p.line.checked_sub(1)?)?;
        let base = self
            .text
            .split_inclusive('\n')
            .take(p.line - 1)
            .map(str::len)
            .sum::<usize>();
        let col = if p.column == line.chars().count() {
            line.len()
        } else {
            line.char_indices().nth(p.column)?.0
        };
        Some(base + col)
    }
    fn proposal(&self, c: &syn::ExprCall) -> Option<RustChange> {
        if c.args.len() != 2 {
            return None;
        }
        let syn::Expr::Path(p) = &*c.func else {
            return None;
        };
        if p.qself.is_some() {
            return None;
        }
        let key = p.path.get_ident()?.to_string();
        if self.shadowed.contains(&key) {
            return None;
        }
        let ty = self.functions.get(&key)?;
        let mut args = c.args.iter();
        let a = args.next()?;
        let b = args.next()?;
        let (a_type, b_type) = (self.ty(a), self.ty(b));
        let integer = matches!(
            ty.as_str(),
            "u8" | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
        );
        let a_known = a_type.as_ref() == Some(ty);
        let b_known = b_type.as_ref() == Some(ty);
        if !(a_known && b_known
            || integer && a_known && Self::untyped_integer(b)
            || integer && b_known && Self::untyped_integer(a))
        {
            return None;
        }
        let start = self.offset(c.span().start())?;
        let end = self.offset(c.span().end())?;
        let a0 = self.offset(a.span().start())?;
        let a1 = self.offset(a.span().end())?;
        let b0 = self.offset(b.span().start())?;
        let b1 = self.offset(b.span().end())?;
        let left = self.text.get(a0..a1)?;
        let right = self.text.get(b0..b1)?;
        if left == right {
            return None;
        }
        let old = self.text.get(start..end)?.to_owned();
        let new = format!(
            "{}{}{}{}{}",
            self.text.get(start..a0)?,
            right,
            self.text.get(a1..b0)?,
            left,
            self.text.get(b1..end)?
        );
        Some(RustChange {
            source: self.source.into(),
            file: self.file.into(),
            line: c.span().start().line as u32,
            column: c.span().start().column as u32,
            old,
            new,
        })
    }
}
impl<'a> Visit<'a> for Calls<'_> {
    fn visit_type(&mut self, _: &'a syn::Type) {}
    fn visit_path_arguments(&mut self, _: &'a syn::PathArguments) {}
    fn visit_expr_call(&mut self, c: &'a syn::ExprCall) {
        if let Some(p) = self.proposal(c) {
            self.changes.push(p);
        }
        syn::visit::visit_expr_call(self, c);
    }
    fn visit_macro(&mut self, m: &'a syn::Macro) {
        if let Some(args) = format_args(m) {
            for a in &args {
                self.visit_expr(a);
            }
        }
    }
}
