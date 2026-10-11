//! A deliberately closed proof of a primitive reduction and its empty policy.
//! No inferred iterator/Deref/PartialOrd facts, and no interpretation of loops.
use crate::checks::RustChange;
use std::collections::BTreeMap;
use syn::{spanned::Spanned, visit::Visit};

fn plain(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().all(|a| a.path().is_ident("doc"))
}
fn inline_tests(item: &syn::Item) -> bool {
    let syn::Item::Mod(module) = item else {
        return false;
    };
    let test_cfg = |a: &syn::Attribute| {
        a.path().is_ident("cfg")
            && a.parse_args::<syn::Path>()
                .is_ok_and(|p| p.is_ident("test"))
    };
    module.content.is_some()
        && module.attrs.iter().any(test_cfg)
        && module
            .attrs
            .iter()
            .all(|a| a.path().is_ident("doc") || test_cfg(a))
}
fn ident(e: &syn::Expr) -> Option<String> {
    let syn::Expr::Path(p) = e else { return None };
    (p.attrs.is_empty() && p.qself.is_none()).then_some(())?;
    Some(p.path.get_ident()?.to_string())
}
fn type_name(t: &syn::Type) -> Option<String> {
    let syn::Type::Path(p) = t else { return None };
    p.qself.is_none().then_some(())?;
    Some(p.path.get_ident()?.to_string())
}
fn integer(t: &syn::Type) -> Option<String> {
    type_name(t).filter(|t| matches!(t.as_str(), "i8" | "i16" | "i32" | "i64" | "i128" | "isize"))
}
fn container(t: &syn::Type) -> Option<(String, bool)> {
    if let syn::Type::Slice(s) = t {
        return Some((integer(&s.elem)?, true));
    }
    let syn::Type::Path(p) = t else { return None };
    if p.qself.is_some() || p.path.leading_colon.is_some() || p.path.segments.len() != 1 {
        return None;
    }
    let s = p.path.segments.first()?;
    if s.ident != "Vec" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(a) = &s.arguments else {
        return None;
    };
    let [syn::GenericArgument::Type(t)] = a.args.iter().collect::<Vec<_>>().as_slice() else {
        return None;
    };
    Some((integer(t)?, false))
}
fn binding(p: &syn::Pat, mutable: bool) -> Option<String> {
    let syn::Pat::Ident(p) = p else { return None };
    (p.attrs.is_empty()
        && p.by_ref.is_none()
        && p.subpat.is_none()
        && p.mutability.is_some() == mutable)
        .then_some(p.ident.to_string())
}
fn collection(e: &syn::Expr, arg: &str, field: Option<&str>) -> bool {
    match field {
        None => ident(e).as_deref() == Some(arg),
        Some(f) => {
            matches!(e, syn::Expr::Field(x) if x.attrs.is_empty() && ident(&x.base).as_deref()==Some(arg) && matches!(&x.member,syn::Member::Named(n) if n==f))
        }
    }
}
fn expression(s: &syn::Stmt) -> Option<&syn::Expr> {
    let syn::Stmt::Expr(e, _) = s else {
        return None;
    };
    Some(e)
}
fn literal(e: &syn::Expr, ty: &str) -> bool {
    match e {
        syn::Expr::Lit(l) if l.attrs.is_empty() => {
            matches!(&l.lit,syn::Lit::Int(i) if i.suffix().is_empty() || i.suffix()==ty)
        }
        syn::Expr::Unary(u) if u.attrs.is_empty() && matches!(u.op, syn::UnOp::Neg(_)) => {
            matches!(&*u.expr, syn::Expr::Lit(_)) && literal(&u.expr, ty)
        }
        _ => false,
    }
}
fn candidate(
    f: &syn::ItemFn,
    fields: &BTreeMap<String, BTreeMap<String, syn::Type>>,
) -> Option<(proc_macro2::Span, String)> {
    let sig = &f.sig;
    if !plain(&f.attrs)
        || !sig.generics.params.is_empty()
        || sig.generics.where_clause.is_some()
        || sig.unsafety.is_some()
        || sig.asyncness.is_some()
        || sig.constness.is_some()
        || sig.abi.is_some()
        || sig.variadic.is_some()
    {
        return None;
    }
    let [syn::FnArg::Typed(a)] = sig.inputs.iter().collect::<Vec<_>>().as_slice() else {
        return None;
    };
    if !a.attrs.is_empty() {
        return None;
    }
    let arg = binding(&a.pat, false)?;
    let syn::Type::Reference(r) = &*a.ty else {
        return None;
    };
    if r.mutability.is_some() {
        return None;
    }
    let [guard, local, loop_stmt, tail] = f.block.stmts.as_slice() else {
        return None;
    };
    let syn::Expr::If(g) = expression(guard)? else {
        return None;
    };
    if !g.attrs.is_empty() || g.else_branch.is_some() {
        return None;
    }
    let syn::Expr::MethodCall(empty) = &*g.cond else {
        return None;
    };
    if !empty.attrs.is_empty()
        || empty.method != "is_empty"
        || !empty.args.is_empty()
        || empty.turbofish.is_some()
    {
        return None;
    }
    let (field, ty, slice) = if let Some((ty, slice)) = container(&r.elem) {
        (None, ty, slice)
    } else {
        let syn::Expr::Field(e) = &*empty.receiver else {
            return None;
        };
        let syn::Member::Named(n) = &e.member else {
            return None;
        };
        let name = n.to_string();
        let (ty, slice) = container(fields.get(&type_name(&r.elem)?)?.get(&name)?)?;
        if slice {
            return None;
        }
        (Some(name), ty, false)
    };
    if !collection(&empty.receiver, &arg, field.as_deref()) {
        return None;
    }
    let [ret] = g.then_branch.stmts.as_slice() else {
        return None;
    };
    let syn::Expr::Return(ret) = expression(ret)? else {
        return None;
    };
    if !ret.attrs.is_empty() || ident(ret.expr.as_deref()?).as_deref() != Some("None") {
        return None;
    }
    let syn::Stmt::Local(local) = local else {
        return None;
    };
    if !local.attrs.is_empty() {
        return None;
    }
    let pat = if let syn::Pat::Type(p) = &local.pat {
        if !p.attrs.is_empty() || integer(&p.ty)? != ty {
            return None;
        }
        &*p.pat
    } else {
        &local.pat
    };
    let acc = binding(pat, true)?;
    let init = local.init.as_ref()?;
    if init.diverge.is_some() || !literal(&init.expr, &ty) || acc == arg {
        return None;
    }
    let syn::Expr::ForLoop(l) = expression(loop_stmt)? else {
        return None;
    };
    if !l.attrs.is_empty() || l.label.is_some() {
        return None;
    }
    let syn::Pat::Reference(p) = &*l.pat else {
        return None;
    };
    if !p.attrs.is_empty() || p.mutability.is_some() {
        return None;
    }
    let elem = binding(&p.pat, false)?;
    if elem == acc
        || elem == arg
        || [arg.as_str(), acc.as_str(), elem.as_str()]
            .iter()
            .any(|n| ["Some", "None", "core"].contains(n))
    {
        return None;
    }
    let iterable = if slice || field.is_none() {
        &*l.expr
    } else {
        let syn::Expr::Reference(r) = &*l.expr else {
            return None;
        };
        if !r.attrs.is_empty() || r.mutability.is_some() {
            return None;
        }
        &*r.expr
    };
    if !collection(iterable, &arg, field.as_deref()) {
        return None;
    }
    let [condition] = l.body.stmts.as_slice() else {
        return None;
    };
    let syn::Expr::If(c) = expression(condition)? else {
        return None;
    };
    if !c.attrs.is_empty() || c.else_branch.is_some() {
        return None;
    }
    let syn::Expr::Binary(b) = &*c.cond else {
        return None;
    };
    if !b.attrs.is_empty() {
        return None;
    }
    let left = ident(&b.left)?;
    let right = ident(&b.right)?;
    let forward = left == elem && right == acc;
    if !forward && !(left == acc && right == elem) {
        return None;
    }
    let maximum = match b.op {
        syn::BinOp::Gt(_) => forward,
        syn::BinOp::Lt(_) => !forward,
        _ => return None,
    };
    let [update] = c.then_branch.stmts.as_slice() else {
        return None;
    };
    let syn::Expr::Assign(update) = expression(update)? else {
        return None;
    };
    if !update.attrs.is_empty() || ident(&update.left)? != acc || ident(&update.right)? != elem {
        return None;
    }
    let syn::Stmt::Expr(syn::Expr::Call(tail), None) = tail else {
        return None;
    };
    if !tail.attrs.is_empty()
        || ident(&tail.func).as_deref() != Some("Some")
        || tail.args.len() != 1
        || ident(tail.args.first()?)? != acc
    {
        return None;
    }
    // The signature independently anchors the accumulator/result type.
    let syn::ReturnType::Type(_, out) = &sig.output else {
        return None;
    };
    let syn::Type::Path(out) = &**out else {
        return None;
    };
    if out.qself.is_some() || out.path.leading_colon.is_some() || out.path.segments.len() != 1 {
        return None;
    }
    let seg = out.path.segments.first()?;
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let [syn::GenericArgument::Type(out)] = args.args.iter().collect::<Vec<_>>().as_slice() else {
        return None;
    };
    if seg.ident != "Option" || integer(out)? != ty {
        return None;
    }
    Some((
        init.expr.span(),
        format!("<{ty}>::{}", if maximum { "MIN" } else { "MAX" }),
    ))
}

pub(crate) fn proposals(source: &str, file: &str, text: &str) -> Vec<RustChange> {
    if !matches!(file, "src/lib.rs" | "src/main.rs") {
        return vec![];
    }
    let Ok(tree) = syn::parse_file(text) else {
        return vec![];
    };
    // A closed root namespace: no import, macro expansion, attribute expansion,
    // external modules or aliases may silently redefine primitive/prelude facts.
    struct Opaque(bool);
    impl<'a> Visit<'a> for Opaque {
        fn visit_attribute(&mut self, a: &'a syn::Attribute) {
            if !plain(std::slice::from_ref(a)) {
                self.0 = true;
            }
        }
        fn visit_macro(&mut self, _: &'a syn::Macro) {
            self.0 = true;
        }
        fn visit_item(&mut self, i: &'a syn::Item) {
            // An ordinary inline test module cannot change its enclosing
            // namespace. Production candidates contain no macro invocations.
            if inline_tests(i) {
                return;
            }
            if !matches!(i, syn::Item::Fn(_) | syn::Item::Struct(_)) {
                self.0 = true;
            }
            syn::visit::visit_item(self, i);
        }
    }
    let mut opaque = Opaque(false);
    opaque.visit_file(&tree);
    if opaque.0 {
        return vec![];
    }
    let mut fields = BTreeMap::new();
    for item in &tree.items {
        if inline_tests(item) {
            continue;
        }
        let name = match item {
            syn::Item::Fn(f) => f.sig.ident.to_string(),
            syn::Item::Struct(s) => {
                if !s.generics.params.is_empty() || s.generics.where_clause.is_some() {
                    return vec![];
                }
                fields.insert(
                    s.ident.to_string(),
                    s.fields
                        .iter()
                        .filter_map(|f| Some((f.ident.as_ref()?.to_string(), f.ty.clone())))
                        .collect(),
                );
                s.ident.to_string()
            }
            _ => return vec![],
        };
        if [
            "core", "Option", "Some", "None", "Vec", "i8", "i16", "i32", "i64", "i128", "isize",
        ]
        .contains(&name.as_str())
        {
            return vec![];
        }
    }
    tree.items
        .iter()
        .filter_map(|i| {
            let syn::Item::Fn(f) = i else { return None };
            let (span, new) = candidate(f, &fields)?;
            let start = span.start();
            let end = span.end();
            let offset = |p: proc_macro2::LineColumn| -> Option<usize> {
                let line = text.split_inclusive('\n').nth(p.line.checked_sub(1)?)?;
                let col = line
                    .char_indices()
                    .map(|(i, _)| i)
                    .chain(std::iter::once(line.len()))
                    .nth(p.column)?;
                Some(
                    text.split_inclusive('\n')
                        .take(p.line - 1)
                        .map(str::len)
                        .sum::<usize>()
                        + col,
                )
            };
            let old = text.get(offset(start)?..offset(end)?)?.to_owned();
            Some(RustChange {
                source: source.into(),
                file: file.into(),
                line: start.line as u32,
                column: start.column as u32,
                old,
                new,
            })
        })
        .collect()
}

#[cfg(test)]
mod parent_tests {
    use super::proposals;

    #[test]
    fn inline_tests_keep_their_own_namespace_and_unknown_root_expansion_refuses() {
        let production = "fn reduce(values: &[i32]) -> Option<i32> { if values.is_empty() { return None; } let mut best = 0; for &x in values { if x > best { best = x; } } Some(best) }";
        let tests = "#[cfg(test)] mod tests { type i32 = u8; struct Vec; #[test] fn check() { assert_eq!(super::reduce(&[-8,-3]), Some(-3)); } }";
        let changes = proposals("parent", "src/lib.rs", &format!("{production}\n{tests}"));
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old, "0");
        assert_eq!(changes[0].new, "<i32>::MIN");
        for module in [
            "#[cfg(test)] mod tests;",
            "mod tests {}",
            "#[cfg_attr(test, inline)] mod tests {}",
            "#[cfg(test)] #[macro_use] mod tests {}",
        ] {
            assert!(
                proposals("parent", "src/lib.rs", &format!("{production}\n{module}")).is_empty()
            );
        }
    }
}
