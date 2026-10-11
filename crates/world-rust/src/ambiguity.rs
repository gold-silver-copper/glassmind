//! A deliberately small typed interpreter: only concrete disagreement is evidence.
use std::collections::BTreeSet;
use syn::spanned::Spanned;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ty {
    Bool,
    Int { signed: bool, bits: u32 },
}
impl Ty {
    fn parse(t: &syn::Type) -> Option<Self> {
        let syn::Type::Path(p) = t else { return None };
        let name = p.path.get_ident()?.to_string();
        if name == "bool" {
            return Some(Self::Bool);
        }
        let signed = match name.as_bytes().first()? {
            b'i' => true,
            b'u' => false,
            _ => return None,
        };
        let bits = name.get(1..)?.parse().ok()?;
        matches!(bits, 8 | 16 | 32 | 64).then_some(Self::Int { signed, bits })
    }
    fn bounds(self) -> Option<(i128, i128)> {
        match self {
            Self::Bool => None,
            Self::Int { signed: true, bits } => {
                Some((-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1))
            }
            Self::Int {
                signed: false,
                bits,
            } => Some((0, (1i128 << bits) - 1)),
        }
    }
    fn valid(self, n: i128) -> Option<i128> {
        let (lo, hi) = self.bounds()?;
        (lo..=hi).contains(&n).then_some(n)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Val {
    Bool(bool),
    Int(i128),
}
impl Val {
    fn boolean(self) -> Option<bool> {
        if let Self::Bool(b) = self {
            Some(b)
        } else {
            None
        }
    }
    fn integer(self) -> Option<i128> {
        if let Self::Int(n) = self {
            Some(n)
        } else {
            None
        }
    }
    fn text(self) -> String {
        match self {
            Self::Bool(b) => b.to_string(),
            Self::Int(n) => n.to_string(),
        }
    }
}
#[derive(Debug)]
enum Expr {
    Arg,
    Lit(Val),
    Unary(syn::UnOp, Box<Expr>),
    Binary(syn::BinOp, Box<Expr>, Box<Expr>),
}
fn has_arg(e: &Expr) -> bool {
    match e {
        Expr::Arg => true,
        Expr::Lit(_) => false,
        Expr::Unary(_, e) => has_arg(e),
        Expr::Binary(_, l, r) => has_arg(l) || has_arg(r),
    }
}
fn lower(
    e: &syn::Expr,
    arg: &str,
    ty: Ty,
    literals: &mut BTreeSet<i128>,
    depth: usize,
) -> Option<(Expr, Ty)> {
    if depth > 64 {
        return None;
    }
    use syn::{BinOp as B, Expr as E, UnOp as U};
    match e {
        E::Paren(p) if p.attrs.is_empty() => lower(&p.expr, arg, ty, literals, depth + 1),
        E::Path(p) if p.attrs.is_empty() && p.qself.is_none() && p.path.is_ident(arg) => {
            Some((Expr::Arg, ty))
        }
        E::Lit(l) if l.attrs.is_empty() => match &l.lit {
            syn::Lit::Bool(b) => Some((Expr::Lit(Val::Bool(b.value)), Ty::Bool)),
            syn::Lit::Int(n) => {
                if !n.suffix().is_empty() {
                    let t = syn::parse_str::<syn::Type>(n.suffix()).ok()?;
                    if Ty::parse(&t)? != ty {
                        return None;
                    }
                }
                let n = ty.valid(n.base10_parse().ok()?)?;
                literals.insert(n);
                Some((Expr::Lit(Val::Int(n)), ty))
            }
            _ => None,
        },
        E::Unary(u) if u.attrs.is_empty() => {
            let (e, t) = lower(&u.expr, arg, ty, literals, depth + 1)?;
            match (&u.op, t) {
                (U::Not(_), Ty::Bool) | (U::Neg(_), Ty::Int { signed: true, .. }) => {}
                _ => return None,
            }
            Some((Expr::Unary(u.op, Box::new(e)), t))
        }
        E::Binary(b) if b.attrs.is_empty() => {
            let (l, lt) = lower(&b.left, arg, ty, literals, depth + 1)?;
            let (r, rt) = lower(&b.right, arg, ty, literals, depth + 1)?;
            if lt != rt {
                return None;
            }
            // Numeric comparisons must be anchored to the typed argument.
            // Otherwise Rust could default an independent literal expression to
            // i32 rather than the argument's type. Such expressions are outside
            // this subset (even when their mathematical values happen to agree).
            if matches!(
                b.op,
                B::Eq(_) | B::Ne(_) | B::Lt(_) | B::Le(_) | B::Gt(_) | B::Ge(_)
            ) && lt != Ty::Bool
                && !has_arg(&l)
                && !has_arg(&r)
            {
                return None;
            }
            let out = match b.op {
                B::And(_) | B::Or(_) if lt == Ty::Bool => Ty::Bool,
                B::Eq(_) | B::Ne(_) => Ty::Bool,
                B::Lt(_) | B::Le(_) | B::Gt(_) | B::Ge(_) if lt != Ty::Bool => Ty::Bool,
                B::Add(_) | B::Sub(_) | B::Mul(_) | B::Div(_) | B::Rem(_) if lt != Ty::Bool => lt,
                _ => return None,
            };
            Some((Expr::Binary(b.op, Box::new(l), Box::new(r)), out))
        }
        _ => None,
    }
}
fn eval(e: &Expr, arg: Val, ty: Ty) -> Option<Val> {
    use syn::{BinOp as B, UnOp as U};
    match e {
        Expr::Arg => Some(arg),
        Expr::Lit(v) => Some(*v),
        Expr::Unary(op, e) => match op {
            U::Not(_) => Some(Val::Bool(!eval(e, arg, ty)?.boolean()?)),
            U::Neg(_) => Some(Val::Int(
                ty.valid(eval(e, arg, ty)?.integer()?.checked_neg()?)?,
            )),
            _ => None,
        },
        Expr::Binary(op, l, r) => {
            let l = eval(l, arg, ty)?;
            if matches!(op, B::And(_)) && !l.boolean()? {
                return Some(Val::Bool(false));
            }
            if matches!(op, B::Or(_)) && l.boolean()? {
                return Some(Val::Bool(true));
            }
            let r = eval(r, arg, ty)?;
            let b = match op {
                B::Eq(_) => l == r,
                B::Ne(_) => l != r,
                B::And(_) | B::Or(_) => r.boolean()?,
                B::Lt(_) => l.integer()? < r.integer()?,
                B::Le(_) => l.integer()? <= r.integer()?,
                B::Gt(_) => l.integer()? > r.integer()?,
                B::Ge(_) => l.integer()? >= r.integer()?,
                _ => {
                    let (l, r) = (l.integer()?, r.integer()?);
                    // Rust signed MIN / -1 and MIN % -1 both panic.
                    if matches!(op, B::Div(_) | B::Rem(_)) && r == -1 && l == ty.bounds()?.0 {
                        return None;
                    }
                    let n = match op {
                        B::Add(_) => l.checked_add(r),
                        B::Sub(_) => l.checked_sub(r),
                        B::Mul(_) => l.checked_mul(r),
                        B::Div(_) => l.checked_div(r),
                        B::Rem(_) => l.checked_rem(r),
                        _ => None,
                    }?;
                    return Some(Val::Int(ty.valid(n)?));
                }
            };
            Some(Val::Bool(b))
        }
    }
}
fn docs_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().all(|a| a.path().is_ident("doc"))
}
fn function(f: &syn::ItemFn) -> Option<(Expr, Ty, BTreeSet<i128>, String)> {
    let s = &f.sig;
    if !docs_only(&f.attrs)
        || s.constness.is_some()
        || s.asyncness.is_some()
        || s.unsafety.is_some()
        || s.abi.is_some()
        || s.variadic.is_some()
        || !s.generics.params.is_empty()
        || s.generics.where_clause.is_some()
    {
        return None;
    }
    let args: Vec<_> = s.inputs.iter().collect();
    let [syn::FnArg::Typed(a)] = args.as_slice() else {
        return None;
    };
    if !a.attrs.is_empty() {
        return None;
    }
    let syn::Pat::Ident(p) = &*a.pat else {
        return None;
    };
    if p.by_ref.is_some() || p.mutability.is_some() || p.subpat.is_some() || !p.attrs.is_empty() {
        return None;
    }
    let ty = Ty::parse(&a.ty)?;
    let syn::ReturnType::Type(_, ret) = &s.output else {
        return None;
    };
    let [syn::Stmt::Expr(e, None)] = f.block.stmts.as_slice() else {
        return None;
    };
    let mut literals = BTreeSet::new();
    let (expr, out) = lower(e, &p.ident.to_string(), ty, &mut literals, 0)?;
    (out == Ty::parse(ret)?).then(|| (expr, ty, literals, quote::quote!(#a).to_string()))
}

/// Apply one token edit in memory; cargo later checks a fresh full-tree copy.
pub(crate) fn variant(text: &str, change: &crate::checks::RustChange) -> Option<String> {
    let mut offset = 0;
    for (i, line) in text.split_inclusive('\n').enumerate() {
        if i + 1 == change.line as usize {
            let byte = line.char_indices().nth(change.column as usize)?.0;
            let start = offset + byte;
            let end = start.checked_add(change.old.len())?;
            if text.get(start..end)? != change.old {
                return None;
            }
            return Some(format!(
                "{}{}{}",
                text.get(..start)?,
                change.new,
                text.get(end..)?
            ));
        }
        offset += line.len();
    }
    None
}

/// The two sources are exact single-mutation variants. Neither agreement nor
/// unsupported code proves anything. No names/docs participate in input choice.
pub(crate) fn difference(
    left: &str,
    right: &str,
    first_line: u32,
    second_line: u32,
) -> Option<serde_json::Value> {
    if left.len() > 65_536 || right.len() > 65_536 {
        return None;
    }
    let a = syn::parse_file(left).ok()?;
    let b = syn::parse_file(right).ok()?;
    if !docs_only(&a.attrs) || !docs_only(&b.attrs) {
        return None;
    }
    // Imports/type declarations can shadow primitive names or overload operators.
    if a.items
        .iter()
        .chain(&b.items)
        .any(|i| !matches!(i, syn::Item::Fn(_)))
    {
        return None;
    }
    for (a, b) in a.items.iter().zip(&b.items) {
        let (syn::Item::Fn(a), syn::Item::Fn(b)) = (a, b) else {
            continue;
        };
        let span = a.span();
        if ![first_line, second_line]
            .iter()
            .all(|&l| (span.start().line..=span.end().line).contains(&(l as usize)))
        {
            continue;
        }
        let (ae, ty, mut literals, arg) = function(a)?;
        let (be, bt, bl, _) = function(b)?;
        if ty != bt {
            return None;
        }
        literals.extend(bl);
        let inputs = if ty == Ty::Bool {
            vec![Val::Bool(false), Val::Bool(true)]
        } else {
            let (lo, hi) = ty.bounds()?;
            let mut ns = BTreeSet::from([lo, lo + 1, hi - 1, hi]);
            ns.extend(-128..=255);
            for n in literals {
                ns.extend([n - 1, n, n + 1]);
            }
            ns.into_iter()
                .filter_map(|n| ty.valid(n).map(Val::Int))
                .take(512)
                .collect()
        };
        for input in inputs {
            if let (Some(av), Some(bv)) = (eval(&ae, input, ty), eval(&be, input, ty))
                && av != bv
            {
                return Some(
                    serde_json::json!({"function": a.sig.ident.to_string(), "argument": arg,
                    "input": input.text(), "first_output": av.text(), "second_output": bv.text(),
                    "method": "bounded checked scalar AST evaluation; disagreement only"}),
                );
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn value(code: &str, n: i128) -> Option<Val> {
        let f: syn::ItemFn = syn::parse_str(code).unwrap();
        let (e, ty, _, _) = function(&f)?;
        eval(&e, Val::Int(n), ty)
    }
    #[test]
    fn checked_widths_and_short_circuit_match_native_rust() {
        let code = "fn f(x: u8) -> bool { x < 250 && (x + 5) % 7 == 0 }";
        for x in 0..=255u8 {
            assert_eq!(
                value(code, i128::from(x)),
                Some(Val::Bool(x < 250 && (x + 5) % 7 == 0))
            );
        }
        for x in -128..=127i8 {
            let expected = x
                .checked_mul(2)
                .and_then(|n| n.checked_sub(3))
                .map(|n| Val::Int(i128::from(n)));
            assert_eq!(
                value("fn f(x: i8) -> i8 { x * 2 - 3 }", i128::from(x)),
                expected
            );
        }
        assert_eq!(value("fn f(x: u8) -> u8 { x + 1 }", 255), None);
        assert_eq!(value("fn f(x: i8) -> i8 { x % -1 }", -128), None);
        assert_eq!(value("fn f(x: i8) -> i8 { x / -1 }", -128), None);
        assert_eq!(
            value("fn f(x: u64) -> u64 { x + 1 }", i128::from(u64::MAX)),
            None
        );
        assert_eq!(
            value("fn f(x: i64) -> i64 { -x }", i128::from(i64::MIN)),
            None
        );
        assert_eq!(
            value("fn f(x: u8) -> bool { x == 0 || 10 / x > 1 }", 0),
            Some(Val::Bool(true))
        );
        assert_eq!(value("fn f(x: u8) -> bool { 10 / x > 1 }", 0), None);
    }
    #[test]
    fn evidence_is_concrete_and_unsupported_is_not_proof() {
        let a = "fn f(x: u8) -> bool { x < 5 || x > 10 }";
        let b = "fn f(x: u8) -> bool { x <= 5 || x > 10 }";
        let w = difference(a, b, 1, 1).unwrap();
        assert_eq!(w["input"], "5");
        assert_eq!(w["first_output"], "false");
        assert_eq!(w["second_output"], "true");
        assert!(difference(a, a, 1, 1).is_none());
        for code in [
            "fn f(x: u8) -> bool { unknown(x) }",
            "fn f(x: u8) -> bool { false && unknown(x) }",
            "#[test] fn f(x: u8) -> bool { x < 5 }",
            "#[cfg(test)] fn f(x: u8) -> bool { x < 5 }",
            "fn f(x: u8) -> bool { #[cfg(test)] x < 5 }",
            "fn f(x: u8) -> bool { x < 5u16 }",
            "fn f(x: u8) -> bool { 1 + 2 == 3 || x < 5 }",
            "type u8 = i8; fn f(x: u8) -> bool { x < 5 }",
            "fn f(x: f64) -> bool { x < 5.0 }",
        ] {
            assert!(difference(code, b, 1, 1).is_none(), "{code}");
        }
    }
}
