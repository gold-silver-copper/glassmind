//! Syntax-bounded mutation proposals. Acceptance still requires every test.
use proc_macro2::{Span, TokenTree};
use syn::{spanned::Spanned, visit::Visit};

use crate::checks::RustChange;

#[path = "ambiguity.rs"]
pub(crate) mod ambiguity;

fn mentions_test(tokens: proc_macro2::TokenStream) -> bool {
    tokens.into_iter().any(|t| match t {
        TokenTree::Ident(i) => i == "test",
        TokenTree::Group(g) => mentions_test(g.stream()),
        _ => false,
    })
}

fn test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().segments.last().is_some_and(|s| s.ident == "test")
            || matches!(&a.meta, syn::Meta::List(l)
                if (l.path.is_ident("cfg") || l.path.is_ident("cfg_attr"))
                    && mentions_test(l.tokens.clone()))
    })
}

/// Follow declared production modules from crate roots. Scanning every .rs
/// file independently would accidentally mutate out-of-line test helpers.
pub(crate) fn production_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    use std::path::{Path, PathBuf};
    fn modules(items: &[syn::Item], base: &Path, pending: &mut Vec<PathBuf>) {
        for item in items {
            let syn::Item::Mod(module) = item else {
                continue;
            };
            if test_only(&module.attrs) {
                continue;
            }
            let name = module.ident.to_string();
            if let Some((_, items)) = &module.content {
                modules(items, &base.join(name), pending);
            } else if module
                .attrs
                .iter()
                .any(|a| a.path().is_ident("path") || a.path().is_ident("cfg_attr"))
            {
                // Nonstandard module paths require compiler resolution.
                continue;
            } else {
                pending.push(base.join(format!("{name}.rs")));
                pending.push(base.join(name).join("mod.rs"));
            }
        }
    }
    let mut pending = vec![PathBuf::from("src/lib.rs"), PathBuf::from("src/main.rs")];
    let mut found = std::collections::BTreeSet::new();
    while let Some(file) = pending.pop() {
        if found.contains(&file) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(&file)) else {
            continue;
        };
        let Ok(tree) = syn::parse_file(&text) else {
            continue;
        };
        if test_only(&tree.attrs) {
            continue;
        }
        let Some(parent) = file.parent() else {
            continue;
        };
        let base = if matches!(
            file.file_name().and_then(|n| n.to_str()),
            Some("lib.rs" | "main.rs" | "mod.rs")
        ) {
            parent.to_path_buf()
        } else {
            parent.join(file.file_stem().unwrap_or_default())
        };
        modules(&tree.items, &base, &mut pending);
        found.insert(file);
    }
    found.into_iter().collect()
}

pub(crate) fn proposals(source: &str, file: &str, text: &str) -> Vec<RustChange> {
    let Ok(tree) = syn::parse_file(text) else {
        return Vec::new();
    };
    if test_only(&tree.attrs) {
        return Vec::new();
    }
    let mut visitor = Slips {
        source,
        file,
        text,
        changes: Vec::new(),
    };
    visitor.visit_file(&tree);
    // Preserve the old operator priority within each line. Changing search
    // order could select a different underconstrained, test-passing patch.
    const ORDER: &[&str] = &[
        "+", "-", "<", "<=", ">", ">=", "==", "!=", "&&", "||", "true", "false", "..", "..=",
    ];
    visitor.changes.sort_by_key(|c| {
        (
            c.line,
            ORDER
                .iter()
                .position(|op| *op == c.old)
                .unwrap_or(ORDER.len()),
            c.column,
        )
    });
    visitor.changes
}

struct Slips<'a> {
    source: &'a str,
    file: &'a str,
    text: &'a str,
    changes: Vec<RustChange>,
}

impl Slips<'_> {
    fn replace(&mut self, span: Span, new: &str) {
        let start = span.start();
        let end = span.end();
        // Only exact, single-line token spans. Convert the parser's character
        // columns using chars, matching RustChange's coordinate convention.
        if start.line != end.line {
            return;
        }
        let Some(line) = self.text.lines().nth(start.line.saturating_sub(1)) else {
            return;
        };
        let old: String = line
            .chars()
            .skip(start.column)
            .take(end.column.saturating_sub(start.column))
            .collect();
        if old.is_empty() {
            return;
        }
        self.changes.push(RustChange {
            source: self.source.into(),
            file: self.file.into(),
            line: start.line as u32,
            column: start.column as u32,
            old,
            new: new.into(),
        });
    }
}

impl<'ast> Visit<'ast> for Slips<'_> {
    fn visit_local(&mut self, node: &'ast syn::Local) {
        if !test_only(&node.attrs) {
            syn::visit::visit_local(self, node);
        }
    }
    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        // Test-only expressions can occur inside an otherwise production
        // function. Guard the enclosing expression before visiting its children.
        let attrs = match node {
            syn::Expr::Array(e) => &e.attrs,
            syn::Expr::Assign(e) => &e.attrs,
            syn::Expr::Async(e) => &e.attrs,
            syn::Expr::Await(e) => &e.attrs,
            syn::Expr::Binary(e) => &e.attrs,
            syn::Expr::Block(e) => &e.attrs,
            syn::Expr::Break(e) => &e.attrs,
            syn::Expr::Call(e) => &e.attrs,
            syn::Expr::Cast(e) => &e.attrs,
            syn::Expr::Closure(e) => &e.attrs,
            syn::Expr::Const(e) => &e.attrs,
            syn::Expr::Continue(e) => &e.attrs,
            syn::Expr::Field(e) => &e.attrs,
            syn::Expr::ForLoop(e) => &e.attrs,
            syn::Expr::Group(e) => &e.attrs,
            syn::Expr::If(e) => &e.attrs,
            syn::Expr::Index(e) => &e.attrs,
            syn::Expr::Infer(e) => &e.attrs,
            syn::Expr::Let(e) => &e.attrs,
            syn::Expr::Lit(e) => &e.attrs,
            syn::Expr::Loop(e) => &e.attrs,
            syn::Expr::Macro(e) => &e.attrs,
            syn::Expr::Match(e) => &e.attrs,
            syn::Expr::MethodCall(e) => &e.attrs,
            syn::Expr::Paren(e) => &e.attrs,
            syn::Expr::Path(e) => &e.attrs,
            syn::Expr::Range(e) => &e.attrs,
            syn::Expr::RawAddr(e) => &e.attrs,
            syn::Expr::Reference(e) => &e.attrs,
            syn::Expr::Repeat(e) => &e.attrs,
            syn::Expr::Return(e) => &e.attrs,
            syn::Expr::Struct(e) => &e.attrs,
            syn::Expr::Try(e) => &e.attrs,
            syn::Expr::TryBlock(e) => &e.attrs,
            syn::Expr::Tuple(e) => &e.attrs,
            syn::Expr::Unary(e) => &e.attrs,
            syn::Expr::Unsafe(e) => &e.attrs,
            syn::Expr::While(e) => &e.attrs,
            syn::Expr::Yield(e) => &e.attrs,
            _ => return,
        };
        if !test_only(attrs) {
            syn::visit::visit_expr(self, node);
        }
    }
    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        if !test_only(&node.attrs) {
            syn::visit::visit_arm(self, node);
        }
    }
    fn visit_field_value(&mut self, node: &'ast syn::FieldValue) {
        if !test_only(&node.attrs) {
            syn::visit::visit_field_value(self, node);
        }
    }
    fn visit_field(&mut self, node: &'ast syn::Field) {
        if !test_only(&node.attrs) {
            syn::visit::visit_field(self, node);
        }
    }
    fn visit_item(&mut self, node: &'ast syn::Item) {
        let attrs = match node {
            syn::Item::Const(i) => &i.attrs,
            syn::Item::Enum(i) => &i.attrs,
            syn::Item::ExternCrate(i) => &i.attrs,
            syn::Item::Fn(i) => &i.attrs,
            syn::Item::ForeignMod(i) => &i.attrs,
            syn::Item::Impl(i) => &i.attrs,
            syn::Item::Macro(i) => &i.attrs,
            syn::Item::Mod(i) => &i.attrs,
            syn::Item::Static(i) => &i.attrs,
            syn::Item::Struct(i) => &i.attrs,
            syn::Item::Trait(i) => &i.attrs,
            syn::Item::TraitAlias(i) => &i.attrs,
            syn::Item::Type(i) => &i.attrs,
            syn::Item::Union(i) => &i.attrs,
            syn::Item::Use(i) => &i.attrs,
            _ => return,
        };
        if !test_only(attrs) {
            syn::visit::visit_item(self, node);
        }
    }
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if !test_only(&node.attrs) {
            syn::visit::visit_item_fn(self, node);
        }
    }
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if !test_only(&node.attrs) {
            syn::visit::visit_item_mod(self, node);
        }
    }
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if !test_only(&node.attrs) {
            syn::visit::visit_item_impl(self, node);
        }
    }
    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        if !test_only(&node.attrs) {
            syn::visit::visit_item_const(self, node);
        }
    }
    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        if !test_only(&node.attrs) {
            syn::visit::visit_item_static(self, node);
        }
    }
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if !test_only(&node.attrs) {
            syn::visit::visit_impl_item_fn(self, node);
        }
    }
    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        if !test_only(&node.attrs) {
            syn::visit::visit_item_trait(self, node);
        }
    }
    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        if !test_only(&node.attrs) {
            syn::visit::visit_impl_item_const(self, node);
        }
    }
    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if !test_only(&node.attrs) {
            syn::visit::visit_trait_item_fn(self, node);
        }
    }
    fn visit_trait_item_const(&mut self, node: &'ast syn::TraitItemConst) {
        if !test_only(&node.attrs) {
            syn::visit::visit_trait_item_const(self, node);
        }
    }
    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        use syn::BinOp::*;
        let new = match node.op {
            Add(_) => Some("-"),
            Sub(_) => Some("+"),
            Lt(_) => Some("<="),
            Le(_) => Some("<"),
            Gt(_) => Some(">="),
            Ge(_) => Some(">"),
            Eq(_) => Some("!="),
            Ne(_) => Some("=="),
            And(_) => Some("||"),
            Or(_) => Some("&&"),
            _ => None,
        };
        if let Some(new) = new {
            self.replace(node.op.span(), new);
        }
        syn::visit::visit_expr_binary(self, node);
    }
    fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
        if let syn::Lit::Bool(value) = &node.lit {
            self.replace(value.span(), if value.value { "false" } else { "true" });
        }
    }
    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        // Loop bounds only: changing slice bounds can merely hide a wrong
        // length calculation and violate the slice's end-of-input behavior.
        if let syn::Expr::Range(range) = &*node.expr
            && range.end.is_some()
        {
            let new = match range.limits {
                syn::RangeLimits::HalfOpen(_) => "..=",
                syn::RangeLimits::Closed(_) => "..",
            };
            self.replace(range.limits.span(), new);
        }
        syn::visit::visit_expr_for_loop(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_non_code_tests_and_keeps_later_production_code() {
        let code = r#"
const WORD: &str = "true + false";
fn is_true() -> bool { false }
#[test] fn standalone() { assert!(false); let flag = true; }
#[cfg(any(test, feature = "extra"))] mod checks { fn helper() -> bool { true } }
fn later(a: usize) -> bool { /* true */ a<3||is_true() }
fn output() { println!("{}", true); }
#[cfg(test)] enum Expected { Three = 1 + 2 }
fn conditional(a: usize) -> bool {
    #[cfg(test)] let expected = 1 + 2;
    #[cfg(test)] { let expected = true; }
    match a { #[cfg(test)] 1 => true, _ => is_true() }
}
fn build() { let _ = State { #[cfg(test)] expected: true }; }
"#;
        let found = proposals("s", "src/lib.rs", code);
        let replacements: Vec<_> = found
            .iter()
            .map(|c| (c.old.as_str(), c.new.as_str()))
            .collect();
        assert_eq!(replacements, [("false", "true"), ("<", "<="), ("||", "&&")]);
    }

    #[test]
    fn loop_bounds_only_and_unicode_coordinates_apply() {
        let code =
            "fn f(v: &[u8]) { let café = 1; for i in 0..v.len()-café { let _ = &v[0..i]; } }\n";
        let found = proposals("s", "src/lib.rs", code);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].old, "-");
        assert_eq!(found[1].new, "..=");
        for c in found {
            let actual: String = code
                .chars()
                .skip(c.column as usize)
                .take(c.old.chars().count())
                .collect();
            assert_eq!(actual, c.old);
        }
    }
}
