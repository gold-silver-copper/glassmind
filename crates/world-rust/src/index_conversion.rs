//! Signed slice indices: a deliberately small control-flow and range proof.
use crate::checks::{self, Finding, RustChange, RustCheck};
use std::{collections::BTreeMap, path::Path};
use syn::{spanned::Spanned, visit::Visit};

fn name(e: &syn::Expr, wanted: &str) -> bool {
    matches!(e, syn::Expr::Path(p) if p.attrs.is_empty() && p.qself.is_none() && p.path.is_ident(wanted))
}
fn integer(e: &syn::Expr, n: u8) -> bool {
    matches!(e, syn::Expr::Lit(l) if matches!(&l.lit, syn::Lit::Int(i) if i.suffix().is_empty() && i.base10_parse::<u8>().ok() == Some(n)))
}
fn primitive(t: &syn::Type, names: &[&str]) -> bool {
    matches!(t, syn::Type::Path(p) if p.qself.is_none() && names.iter().any(|n| p.path.is_ident(n)))
}
fn element(t: &syn::Type) -> bool {
    primitive(
        t,
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "bool", "char",
        ],
    ) || matches!(t, syn::Type::Reference(r) if r.mutability.is_none() && primitive(&r.elem, &["str"]))
}
fn plain(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().all(|a| a.path().is_ident("doc"))
}
fn cast(e: &syn::Expr, rank: &str) -> bool {
    matches!(e, syn::Expr::Cast(c) if name(&c.expr, rank) && primitive(&c.ty, &["usize"]))
}
fn length(e: &syn::Expr, slice: &str) -> bool {
    matches!(e, syn::Expr::MethodCall(m) if name(&m.receiver, slice) && m.method == "len" && m.args.is_empty() && m.turbofish.is_none())
}
// Facts hold only along the fall-through edge of an exiting guard.
#[derive(Default)]
struct Bounds {
    lower: Option<u8>,
    upper: Option<u8>,
}
fn condition(e: &syn::Expr, rank: &str, slice: &str, b: &mut Bounds, depth: u8) -> Option<()> {
    if depth == 0 {
        return None;
    }
    let syn::Expr::Binary(c) = e else { return None };
    if matches!(c.op, syn::BinOp::Or(_)) {
        condition(&c.left, rank, slice, b, depth - 1)?;
        return condition(&c.right, rank, slice, b, depth - 1);
    }
    let (left, right, op) = match c.op {
        syn::BinOp::Lt(_) => (&*c.left, &*c.right, 0),
        syn::BinOp::Le(_) => (&*c.left, &*c.right, 1),
        syn::BinOp::Gt(_) => (&*c.right, &*c.left, 0),
        syn::BinOp::Ge(_) => (&*c.right, &*c.left, 1),
        _ => return None,
    };
    if name(left, rank) {
        let lower = if integer(right, 0) {
            op
        } else if integer(right, 1) && op == 0 {
            1
        } else {
            return None;
        };
        b.lower = Some(b.lower.map_or(lower, |n| n.max(lower)));
    } else if length(left, slice) && cast(right, rank) && b.lower.is_some() {
        // !(len < rank) => rank <= len; !(len <= rank) => rank < len.
        b.upper = Some(op);
    } else {
        return None;
    }
    Some(())
}
fn exits(block: &syn::Block) -> bool {
    matches!(block.stmts.as_slice(), [syn::Stmt::Expr(syn::Expr::Return(r), _)] if r.expr.as_deref().is_some_and(|e| name(e, "None")))
}
fn offset(e: &syn::Expr, rank: &str) -> Option<u8> {
    if name(e, rank) {
        return Some(0);
    }
    let syn::Expr::Binary(b) = e else { return None };
    if !name(&b.left, rank) {
        return None;
    }
    match b.op {
        syn::BinOp::Sub(_) if integer(&b.right, 1) => Some(1),
        syn::BinOp::Add(_) if integer(&b.right, 0) => Some(0),
        _ => None,
    }
}
struct Attributes(bool);
impl<'ast> Visit<'ast> for Attributes {
    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {
        self.0 = true;
    }
}
fn prove(f: &syn::ItemFn) -> Option<(&syn::Expr, u8)> {
    if f.block.stmts.len() > 32
        || !plain(&f.attrs)
        || !f.sig.generics.params.is_empty()
        || f.sig.generics.where_clause.is_some()
        || f.sig.asyncness.is_some()
        || f.sig.unsafety.is_some()
        || f.sig.abi.is_some()
        || f.sig.variadic.is_some()
        || f.sig.inputs.len() != 2
    {
        return None;
    }
    let mut attributes = Attributes(false);
    attributes.visit_block(&f.block);
    if attributes.0 {
        return None;
    }
    let mut rank = None;
    let mut slice = None;
    let mut width = 0;
    for arg in &f.sig.inputs {
        let syn::FnArg::Typed(t) = arg else {
            return None;
        };
        let syn::Pat::Ident(p) = &*t.pat else {
            return None;
        };
        if p.mutability.is_some() || p.by_ref.is_some() || p.subpat.is_some() || !t.attrs.is_empty()
        {
            return None;
        }
        let id = p.ident.to_string();
        if let Some(w) = [("i8", 8), ("i16", 16), ("i32", 32), ("i64", 64)]
            .iter()
            .find_map(|(n, w)| primitive(&t.ty, &[*n]).then_some(*w))
        {
            if rank.replace(id).is_some() {
                return None;
            }
            width = w;
        } else if matches!(&*t.ty, syn::Type::Reference(r) if r.mutability.is_none() && matches!(&*r.elem, syn::Type::Slice(s) if element(&s.elem)))
        {
            if slice.replace(id).is_some() {
                return None;
            }
        } else {
            return None;
        }
    }
    let (rank, slice) = (rank?, slice?);
    if rank == slice {
        return None;
    }
    let mut bounds = Bounds::default();
    let mut binding: Option<(String, u8)> = None;
    let (tail, statements) = f.block.stmts.split_last()?;
    for stmt in statements {
        match stmt {
            syn::Stmt::Expr(syn::Expr::If(i), _)
                if binding.is_none() && i.else_branch.is_none() && exits(&i.then_branch) =>
            {
                condition(&i.cond, &rank, &slice, &mut bounds, 16)?;
            }
            syn::Stmt::Local(l) if binding.is_none() => {
                let syn::Pat::Ident(p) = &l.pat else {
                    return None;
                };
                if p.mutability.is_some()
                    || p.by_ref.is_some()
                    || p.subpat.is_some()
                    || p.ident == rank
                    || p.ident == slice
                {
                    return None;
                }
                let init = l.init.as_ref()?;
                if init.diverge.is_some() {
                    return None;
                }
                let sub = offset(&init.expr, &rank)?;
                if bounds.lower? < sub {
                    return None;
                }
                binding = Some((p.ident.to_string(), sub));
            }
            _ => return None,
        }
    }
    let syn::Stmt::Expr(syn::Expr::Call(call), None) = tail else {
        return None;
    };
    if !name(&call.func, "Some") || call.args.len() != 1 {
        return None;
    }
    let mut value = call.args.first()?;
    if let syn::Expr::MethodCall(m) = value {
        if m.method != "to_string" || !m.args.is_empty() || m.turbofish.is_some() {
            return None;
        }
        value = &m.receiver;
    }
    let syn::Expr::Index(index) = value else {
        return None;
    };
    if !name(&index.expr, &slice) {
        return None;
    }
    let sub = if let Some((id, sub)) = binding {
        if !name(&index.index, &id) {
            return None;
        }
        sub
    } else {
        offset(&index.index, &rank)?
    };
    // A nonnegative signed value, subtraction defined, and strict slice bound.
    if bounds.lower? < sub || bounds.upper? + sub < 1 {
        return None;
    }
    Some((&index.index, width))
}
fn change(source: &str, text: &str, expr: &syn::Expr) -> Option<RustChange> {
    let start = expr.span().start();
    let end = expr.span().end();
    if start.line != end.line {
        return None;
    }
    let old: String = text
        .lines()
        .nth(start.line.checked_sub(1)?)?
        .chars()
        .skip(start.column)
        .take(end.column.checked_sub(start.column)?)
        .collect();
    if old.is_empty() {
        return None;
    }
    Some(RustChange {
        source: source.into(),
        file: "src/lib.rs".into(),
        line: u32::try_from(start.line).ok()?,
        column: u32::try_from(start.column).ok()?,
        new: format!("({old}) as usize"),
        old,
    })
}
fn layout(root: &Path) -> Option<()> {
    for parent in root.ancestors() {
        if [".cargo/config", ".cargo/config.toml"]
            .iter()
            .any(|p| parent.join(p).exists())
        {
            return None;
        }
    }
    if root.join("build.rs").exists() {
        return None;
    }
    let manifest: toml::Value =
        toml::from_str(&std::fs::read_to_string(root.join("Cargo.toml")).ok()?).ok()?;
    let table = manifest.as_table()?;
    if table
        .keys()
        .any(|k| !["package", "lib", "workspace"].contains(&k.as_str()))
    {
        return None;
    }
    let package = table.get("package")?.as_table()?;
    if package
        .keys()
        .any(|k| !["name", "version", "edition", "license", "description"].contains(&k.as_str()))
    {
        return None;
    }
    if let Some(lib) = table.get("lib") {
        let lib = lib.as_table()?;
        if lib.len() != 1 || lib.get("path")?.as_str()? != "src/lib.rs" {
            return None;
        }
    }
    Some(())
}
/// Compiler identity, AST proof, and a target-width compile witness are all required.
pub(crate) fn proposals(source: &str, root: &Path, findings: &[Finding]) -> Vec<RustChange> {
    fn candidate(source: &str, root: &Path, findings: &[Finding]) -> Option<RustChange> {
        layout(root)?;
        let canonical = root.canonicalize().ok()?;
        let path = root.join("src/lib.rs").canonicalize().ok()?;
        if !path.starts_with(&canonical) {
            return None;
        }
        let text = std::fs::read_to_string(path).ok()?;
        if text.len() > 65_536 {
            return None;
        }
        let tree = syn::parse_file(&text).ok()?;
        if !plain(&tree.attrs) {
            return None;
        }
        let mut candidates = Vec::new();
        for item in &tree.items {
            match item {
                syn::Item::Fn(f)
                    if plain(&f.attrs) && f.sig.ident != "Some" && f.sig.ident != "None" =>
                {
                    if let Some((expr, width)) = prove(f) {
                        let edit = change(source, &text, expr)?;
                        if findings.iter().any(|d| {
                            d.item.source == source
                                && d.item.file == edit.file
                                && d.item.line == edit.line
                                && d.item.name == "E0277"
                                && d.item.detail.as_deref().is_some_and(|m| {
                                    m.starts_with("the type `[")
                                        && m.ends_with(&format!(
                                            "` cannot be indexed by `i{width}`"
                                        ))
                                })
                        }) {
                            candidates.push((edit, width));
                        }
                    }
                }
                syn::Item::Mod(m)
                    if m.attrs.len() == 1
                        && m.attrs
                            .first()
                            .is_some_and(|a| quote::quote!(#a).to_string() == "# [cfg (test)]") => {
                }
                _ => return None,
            }
        }
        if candidates.len() != 1 {
            return None;
        }
        let (edit, width) = candidates.pop()?;
        // This equality is checked by the actual compilation target, never by
        // this process's usize. No runtime branch, panic, or check is introduced.
        let mut witness = edit.clone();
        witness.new = format!(
            "{{ let _: [(); 1] = [(); (::core::primitive::usize::BITS >= {width}) as usize]; {} }}",
            edit.new
        );
        let scratch = checks::scratch("index-width-proof");
        let result = checks::copy_tree(root, &scratch)
            .and_then(|()| {
                checks::apply(
                    &[witness],
                    &BTreeMap::from([(source.into(), scratch.clone())]),
                )
            })
            .and_then(|_| {
                let places = checks::Places {
                    target: checks::Places::from_env().target,
                    cache: None,
                };
                checks::run_in(&places, RustCheck::Build, source, &scratch)
            });
        let _ = std::fs::remove_dir_all(&scratch);
        result.ok()?.is_empty().then_some(edit)
    }
    candidate(source, root, findings).into_iter().collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    const GOOD: &str = "fn select(xs: &[i32], n: i32) -> Option<i32> { if n < 1 || n as usize > xs.len() { return None; } let at = n - 1; Some(xs[at]) }";
    fn supported(text: &str) -> bool {
        syn::parse_str::<syn::ItemFn>(text)
            .ok()
            .is_some_and(|f| prove(&f).is_some())
    }
    #[test]
    fn subtraction_requires_dominating_defined_bounds() {
        assert!(supported(GOOD));
        for (old, new) in [
            ("n < 1 || ", ""),
            (" || n as usize > xs.len()", ""),
            (" || ", " && "),
            ("return None;", "None;"),
            ("n - 1", "n + 1"),
            ("n < 1", "n < -1"),
            ("let at", "let n = 0; let at"),
            ("let at", "#[cfg(unix)] let at"),
            ("n: i32", "mut n: i32"),
            ("let at", "n = 0; let at"),
            ("let at", "call(); let at"),
        ] {
            assert!(!supported(&GOOD.replace(old, new)), "{old} => {new}");
        }
    }
    #[test]
    fn zero_based_requires_strict_upper_bound() {
        let zero = GOOD.replace("n < 1", "n < 0").replace("n - 1", "n + 0");
        assert!(!supported(&zero));
        assert!(supported(&zero.replace(" > xs", " >= xs")));
        let reversed = GOOD
            .replace("n < 1", "1 > n")
            .replace("n as usize > xs.len()", "xs.len() < n as usize");
        assert!(supported(&reversed));
    }
    #[test]
    fn diagnostic_scope_and_escaping_paths_refuse_before_native_probe() {
        let root = checks::scratch("index-scope");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='proof'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(root.join("src/lib.rs"), GOOD).unwrap();
        assert!(layout(&root).is_some());
        assert!(proposals("s", &root, &[]).is_empty());
        let outside = checks::scratch("index-outside");
        std::fs::write(&outside, GOOD).unwrap();
        std::fs::remove_file(root.join("src/lib.rs")).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("src/lib.rs")).unwrap();
        let finding = Finding {
            item: crate::RustItem {
                source: "s".into(),
                file: "src/lib.rs".into(),
                line: 1,
                name: "E0277".into(),
                kind: crate::RustKind::Problem,
                owner: None,
                detail: Some("the type `[i32]` cannot be indexed by `i32`".into()),
            },
            fixes: Vec::new(),
        };
        assert!(proposals("s", &root, &[finding]).is_empty());
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_file(outside).unwrap();
    }
}
