//! Source-local type proof for pure field comparison direction proposals.
//! Direction is chosen only by complete native public-test verification.
use crate::{checks::RustChange, declared_types::source_range};
use std::collections::BTreeMap;
use syn::{ext::IdentExt, spanned::Spanned, visit::Visit};

const ORD: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "bool",
    "char", "String",
];
fn name(i: &syn::Ident) -> String {
    i.unraw().to_string()
}
fn docs(a: &[syn::Attribute]) -> bool {
    a.iter().all(|a| a.path().is_ident("doc"))
}
fn derives(a: &[syn::Attribute]) -> bool {
    a.iter().all(|a| {
        a.path().is_ident("doc")
            || a.path().is_ident("derive")
                && a.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .is_ok_and(|p| {
                    p.iter().all(|p| {
                        p.get_ident().is_some_and(|i| {
                            matches!(
                                name(i).as_str(),
                                "Debug"
                                    | "Clone"
                                    | "Copy"
                                    | "PartialEq"
                                    | "Eq"
                                    | "PartialOrd"
                                    | "Ord"
                                    | "Hash"
                                    | "Default"
                            )
                        })
                    })
                })
    })
}
fn tests(i: &syn::Item) -> bool {
    let syn::Item::Mod(m) = i else { return false };
    let cfg = |a: &syn::Attribute| {
        a.path().is_ident("cfg")
            && a.parse_args::<syn::Path>()
                .is_ok_and(|p| p.is_ident("test"))
    };
    m.content.is_some()
        && m.attrs.iter().any(cfg)
        && m.attrs.iter().all(|a| a.path().is_ident("doc") || cfg(a))
}
fn simple_type(t: &syn::Type) -> Option<String> {
    let syn::Type::Path(p) = t else { return None };
    if p.qself.is_some() {
        return None;
    }
    p.path.get_ident().map(name)
}
fn vector(t: &syn::Type) -> Option<String> {
    let syn::Type::Path(p) = t else { return None };
    if p.qself.is_some() || p.path.leading_colon.is_some() || p.path.segments.len() != 1 {
        return None;
    }
    let s = p.path.segments.first()?;
    let syn::PathArguments::AngleBracketed(a) = &s.arguments else {
        return None;
    };
    if s.ident != "Vec" || a.colon2_token.is_some() || a.args.len() != 1 {
        return None;
    }
    let syn::GenericArgument::Type(t) = a.args.first()? else {
        return None;
    };
    simple_type(t)
}
fn slice(t: &syn::Type) -> Option<String> {
    let syn::Type::Reference(r) = t else {
        return None;
    };
    if r.mutability.is_some() {
        return None;
    }
    let syn::Type::Slice(s) = &*r.elem else {
        return None;
    };
    simple_type(&s.elem)
}
fn binding(p: &syn::Pat, mutable: bool) -> Option<String> {
    let syn::Pat::Ident(p) = p else { return None };
    (p.attrs.is_empty()
        && p.by_ref.is_none()
        && p.subpat.is_none()
        && p.mutability.is_some() == mutable)
        .then(|| name(&p.ident))
}
fn path(e: &syn::Expr) -> Option<String> {
    let syn::Expr::Path(p) = e else { return None };
    if !p.attrs.is_empty() || p.qself.is_some() {
        return None;
    }
    p.path.get_ident().map(name)
}
fn body(e: &syn::Expr) -> Option<&syn::Expr> {
    if let syn::Expr::Block(b) = e {
        if !b.attrs.is_empty() || b.label.is_some() {
            return None;
        }
        let [syn::Stmt::Expr(e, None)] = b.block.stmts.as_slice() else {
            return None;
        };
        Some(e)
    } else {
        Some(e)
    }
}
fn closure(c: &syn::ExprClosure) -> bool {
    c.attrs.is_empty()
        && c.lifetimes.is_none()
        && c.constness.is_none()
        && c.movability.is_none()
        && c.asyncness.is_none()
        && c.capture.is_none()
        && matches!(c.output, syn::ReturnType::Default)
}
fn field<'a>(
    e: &'a syn::Expr,
    fields: &BTreeMap<String, String>,
) -> Option<(String, String, &'a syn::Expr)> {
    let syn::Expr::Field(f) = e else { return None };
    if !f.attrs.is_empty() {
        return None;
    }
    let syn::Member::Named(i) = &f.member else {
        return None;
    };
    let n = name(i);
    fields.get(&n)?;
    Some((path(&f.base)?, n, e))
}
// Recognize the entire chain first; an opaque secondary disqualifies the chain.
fn comparisons<'a>(
    e: &'a syn::Expr,
    params: &[String],
    fields: &BTreeMap<String, String>,
    out: &mut Vec<(&'a syn::Expr, &'a syn::Expr)>,
) -> Option<()> {
    let syn::Expr::MethodCall(c) = body(e)? else {
        return None;
    };
    if !c.attrs.is_empty() || c.turbofish.is_some() || c.args.len() != 1 {
        return None;
    }
    if c.method == "then_with" {
        let syn::Expr::Closure(next) = c.args.first()? else {
            return None;
        };
        if !closure(next) || !next.inputs.is_empty() {
            return None;
        }
        comparisons(&c.receiver, params, fields, out)?;
        return comparisons(&next.body, params, fields, out);
    }
    if c.method != "cmp" {
        return None;
    }
    let syn::Expr::Reference(r) = c.args.first()? else {
        return None;
    };
    if !r.attrs.is_empty() || r.mutability.is_some() {
        return None;
    }
    let (left, lf, le) = field(&c.receiver, fields)?;
    let (right, rf, re) = field(&r.expr, fields)?;
    if lf != rf || left == right || !params.contains(&left) || !params.contains(&right) {
        return None;
    }
    out.push((le, re));
    Some(())
}

pub(crate) fn proposals(source: &str, file: &str, text: &str) -> Vec<RustChange> {
    if !matches!(file, "src/lib.rs" | "src/main.rs") {
        return vec![];
    }
    let Ok(tree) = syn::parse_file(text) else {
        return vec![];
    };
    if !docs(&tree.attrs) {
        return vec![];
    }
    let mut structs = BTreeMap::new();
    let mut names = std::collections::BTreeSet::new();
    for i in &tree.items {
        if tests(i) {
            continue;
        }
        let ident = match i {
            syn::Item::Struct(s) => {
                if !derives(&s.attrs)
                    || !s.generics.params.is_empty()
                    || s.generics.where_clause.is_some()
                {
                    return vec![];
                }
                let syn::Fields::Named(fs) = &s.fields else {
                    return vec![];
                };
                let mut fields = BTreeMap::new();
                for f in &fs.named {
                    let Some(t) = simple_type(&f.ty) else {
                        return vec![];
                    };
                    let Some(n) = f.ident.as_ref() else {
                        return vec![];
                    };
                    if !docs(&f.attrs)
                        || !ORD.contains(&t.as_str())
                        || fields.insert(name(n), t).is_some()
                    {
                        return vec![];
                    }
                }
                structs.insert(name(&s.ident), fields);
                &s.ident
            }
            syn::Item::Fn(f) => {
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
                    return vec![];
                }
                // Nested items, expansions and opaque attributes can change lookup.
                #[derive(Default)]
                struct Guard(bool);
                impl<'a> Visit<'a> for Guard {
                    fn visit_item(&mut self, _: &'a syn::Item) {
                        self.0 = true;
                    }
                    fn visit_macro(&mut self, _: &'a syn::Macro) {
                        self.0 = true;
                    }
                    fn visit_attribute(&mut self, a: &'a syn::Attribute) {
                        self.0 |= !docs(std::slice::from_ref(a));
                    }
                    fn visit_type_path(&mut self, p: &'a syn::TypePath) {
                        // Qualified/associated paths and arbitrary generic
                        // constructors are outside this closed proof.
                        self.0 |= p.qself.is_some()
                            || p.path.leading_colon.is_some()
                            || p.path.segments.len() != 1
                            || p.path.segments.iter().any(|s| {
                                !matches!(s.arguments, syn::PathArguments::None) && s.ident != "Vec"
                            });
                        syn::visit::visit_type_path(self, p);
                    }
                }
                let mut g = Guard::default();
                g.visit_signature(s);
                g.visit_block(&f.block);
                if g.0 {
                    return vec![];
                }
                &s.ident
            }
            _ => return vec![],
        };
        let n = name(ident);
        if n == "Vec" || ORD.contains(&n.as_str()) || !names.insert(n) {
            return vec![];
        }
    }
    let mut changes = Vec::new();
    for i in &tree.items {
        let syn::Item::Fn(f) = i else { continue };
        #[derive(Default)]
        struct Bindings(BTreeMap<String, usize>);
        impl<'a> Visit<'a> for Bindings {
            fn visit_pat_ident(&mut self, p: &'a syn::PatIdent) {
                *self.0.entry(name(&p.ident)).or_default() += 1;
                syn::visit::visit_pat_ident(self, p);
            }
        }
        let mut bindings = Bindings::default();
        bindings.visit_item_fn(f);
        let unique = |n: &String| bindings.0.get(n) == Some(&1);
        let mut vectors = BTreeMap::new();
        let mut slices = BTreeMap::new();
        for a in &f.sig.inputs {
            if let syn::FnArg::Typed(a) = a {
                if let (Some(n), Some(t)) = (binding(&a.pat, true), vector(&a.ty))
                    && unique(&n)
                {
                    vectors.insert(n, t);
                }
                if let (Some(n), Some(t)) = (binding(&a.pat, false), slice(&a.ty))
                    && unique(&n)
                {
                    slices.insert(n, t);
                }
            }
        }
        for stmt in &f.block.stmts {
            if let syn::Stmt::Local(l) = stmt
                && l.attrs.is_empty()
                && let Some(init) = &l.init
                && init.diverge.is_none()
            {
                let (pat, ty) = if let syn::Pat::Type(p) = &l.pat {
                    (&*p.pat, vector(&p.ty))
                } else {
                    (&l.pat, None)
                };
                if let Some(n) = binding(pat, true)
                    && unique(&n)
                {
                    let inferred = if let syn::Expr::MethodCall(c) = &*init.expr {
                        if c.attrs.is_empty()
                            && c.method == "to_vec"
                            && c.args.is_empty()
                            && c.turbofish.is_none()
                        {
                            path(&c.receiver).and_then(|n| slices.get(&n).cloned())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(t) = ty.or(inferred) {
                        vectors.insert(n, t);
                    }
                }
            }
            let syn::Stmt::Expr(syn::Expr::MethodCall(c), Some(_)) = stmt else {
                continue;
            };
            if !c.attrs.is_empty()
                || c.method != "sort_by"
                || c.turbofish.is_some()
                || c.args.len() != 1
            {
                continue;
            }
            let Some(n) = path(&c.receiver) else { continue };
            let Some(fields) = vectors.get(&n).and_then(|t| structs.get(t)) else {
                continue;
            };
            let Some(syn::Expr::Closure(c)) = c.args.first() else {
                continue;
            };
            if !closure(c) || c.inputs.len() != 2 {
                continue;
            }
            let Some(params) = c
                .inputs
                .iter()
                .map(|p| binding(p, false))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            if !params.iter().all(unique) {
                continue;
            }
            let mut pairs = Vec::new();
            if comparisons(&c.body, &params, fields, &mut pairs).is_none() {
                continue;
            }
            for (left, right) in pairs {
                let Some((a, b)) = source_range(text, left.span()) else {
                    continue;
                };
                let Some((c, d)) = source_range(text, right.span()) else {
                    continue;
                };
                if b > c {
                    continue;
                }
                let (Some(l), Some(gap), Some(r), Some(old)) = (
                    text.get(a..b),
                    text.get(b..c),
                    text.get(c..d),
                    text.get(a..d),
                ) else {
                    continue;
                };
                changes.push(RustChange {
                    source: source.into(),
                    file: file.into(),
                    line: left.span().start().line as u32,
                    column: left.span().start().column as u32,
                    old: old.into(),
                    new: format!("{r}{gap}{l}"),
                });
            }
        }
    }
    changes
}
