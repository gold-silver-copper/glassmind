//! Indexing a Rust source with `syn`: its items and who calls what.
//!
//! Follows the definitions in `docs/tasks.md`: which files count, what an
//! item is, and when a function calls a name.

use std::path::{Path, PathBuf};

use proc_macro2::Span;
use syn::visit::Visit;

use crate::{RustItem, RustKind};

/// One indexed item with what the world knows about it.
#[derive(Debug, Clone)]
pub struct Record {
    pub item: RustItem,
    /// The declaration's first line as written, such as `pub fn spawn_wave(level: u32) -> usize`.
    pub signature: String,
    /// The doc comment, if any, one string per line.
    pub doc: Vec<String>,
    /// For a field, the identifiers in its type.
    pub type_idents: Vec<String>,
}

/// A call found in a function body: by the function at index `caller`, of `name`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Call {
    pub caller: usize,
    pub name: String,
    pub line: u32,
}

/// Everything indexed from one source.
#[derive(Debug, Clone, Default)]
pub struct SourceIndex {
    pub name: String,
    pub root: PathBuf,
    pub records: Vec<Record>,
    pub calls: Vec<Call>,
    /// Files that could not be read or parsed, with the reason.
    pub skipped: Vec<(String, String)>,
}

/// Every `.rs` file under `root`, relative and sorted, skipping `target`
/// directories and hidden files and directories.
pub fn rust_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                if name != "target" {
                    stack.push(path);
                }
            } else if kind.is_file()
                && name.ends_with(".rs")
                && let Ok(rel) = path.strip_prefix(root)
            {
                let rel: Vec<String> = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                out.push(rel.join("/"));
            }
        }
    }
    out.sort();
    out
}

/// Indexes the source at `root` under the name `name`.
pub fn index_source(name: &str, root: &Path) -> SourceIndex {
    let mut index = SourceIndex {
        name: name.to_owned(),
        root: root.to_path_buf(),
        ..SourceIndex::default()
    };
    for file in rust_files(root) {
        let text = match std::fs::read_to_string(root.join(&file)) {
            Ok(t) => t,
            Err(e) => {
                index.skipped.push((file, e.to_string()));
                continue;
            }
        };
        let parsed = match syn::parse_file(&text) {
            Ok(p) => p,
            Err(e) => {
                index.skipped.push((file, e.to_string()));
                continue;
            }
        };
        let mut visitor = Indexer {
            source: name,
            file: &file,
            text: &text,
            index: &mut index,
            current_fn: Vec::new(),
            owner: Vec::new(),
        };
        visitor.visit_file(&parsed);
    }
    index.calls.sort();
    index.calls.dedup();
    index
}

struct Indexer<'a> {
    source: &'a str,
    file: &'a str,
    text: &'a str,
    index: &'a mut SourceIndex,
    /// Indices of the functions whose bodies we are inside, innermost last.
    current_fn: Vec<usize>,
    /// Names of the impl self types or traits we are inside, innermost last.
    owner: Vec<String>,
}

fn line_of(span: Span) -> u32 {
    u32::try_from(span.start().line).unwrap_or(u32::MAX)
}

fn doc_lines(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| match &a.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => Some(s.value().trim().to_owned()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// The last identifier of a type path, such as `Player` in `crate::game::Player<T>`.
fn type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        syn::Type::Reference(r) => type_name(&r.elem),
        _ => None,
    }
}

fn idents_in(tokens: proc_macro2::TokenStream, out: &mut Vec<String>) {
    for tree in tokens {
        match tree {
            proc_macro2::TokenTree::Ident(i) => out.push(i.to_string()),
            proc_macro2::TokenTree::Group(g) => idents_in(g.stream(), out),
            _ => {}
        }
    }
}

impl Indexer<'_> {
    /// The line of source text that `span` starts on, trimmed.
    fn first_line(&self, span: Span) -> String {
        let start = span.start();
        let line = self
            .text
            .lines()
            .nth(start.line.saturating_sub(1))
            .unwrap_or_default();
        line.trim().trim_end_matches('{').trim().to_owned()
    }

    fn add(
        &mut self,
        name: &syn::Ident,
        kind: RustKind,
        owner: Option<String>,
        attrs: &[syn::Attribute],
    ) -> usize {
        let item = RustItem {
            source: self.source.to_owned(),
            file: self.file.to_owned(),
            line: line_of(name.span()),
            name: name.to_string(),
            kind,
            owner,
        };
        self.index.records.push(Record {
            item,
            signature: self.first_line(name.span()),
            doc: doc_lines(attrs),
            type_idents: Vec::new(),
        });
        self.index.records.len() - 1
    }

    fn function(
        &mut self,
        sig: &syn::Signature,
        attrs: &[syn::Attribute],
        body: Option<&syn::Block>,
    ) {
        let owner = self.owner.last().cloned();
        let at = self.add(&sig.ident, RustKind::Function, owner, attrs);
        if let Some(body) = body {
            // Items nested in the body get their own owner context.
            let saved = std::mem::take(&mut self.owner);
            self.current_fn.push(at);
            self.visit_block(body);
            self.current_fn.pop();
            self.owner = saved;
        }
    }

    fn call(&mut self, name: &syn::Ident) {
        if let Some(&caller) = self.current_fn.last() {
            self.index.calls.push(Call {
                caller,
                name: name.to_string(),
                line: line_of(name.span()),
            });
        }
    }
}

impl<'ast> Visit<'ast> for Indexer<'_> {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.function(&f.sig, &f.attrs, Some(&f.block));
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.function(&f.sig, &f.attrs, Some(&f.block));
    }

    fn visit_trait_item_fn(&mut self, f: &'ast syn::TraitItemFn) {
        self.function(&f.sig, &f.attrs, f.default.as_ref());
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        let owner = type_name(&i.self_ty).unwrap_or_default();
        self.owner.push(owner);
        // Nested items inside an impl are only its functions (and consts);
        // visiting them directly keeps them out of the enclosing function's calls.
        let saved_fn = std::mem::take(&mut self.current_fn);
        for item in &i.items {
            self.visit_impl_item(item);
        }
        self.current_fn = saved_fn;
        self.owner.pop();
    }

    fn visit_item_trait(&mut self, t: &'ast syn::ItemTrait) {
        self.add(&t.ident, RustKind::Trait, None, &t.attrs);
        self.owner.push(t.ident.to_string());
        let saved_fn = std::mem::take(&mut self.current_fn);
        for item in &t.items {
            self.visit_trait_item(item);
        }
        self.current_fn = saved_fn;
        self.owner.pop();
    }

    fn visit_item_struct(&mut self, s: &'ast syn::ItemStruct) {
        self.add(&s.ident, RustKind::Struct, None, &s.attrs);
        if let syn::Fields::Named(fields) = &s.fields {
            for field in &fields.named {
                if let Some(ident) = &field.ident {
                    let at = self.add(
                        ident,
                        RustKind::Field,
                        Some(s.ident.to_string()),
                        &field.attrs,
                    );
                    let mut idents = Vec::new();
                    idents_in(quote::ToTokens::to_token_stream(&field.ty), &mut idents);
                    if let Some(record) = self.index.records.get_mut(at) {
                        record.type_idents = idents;
                    }
                }
            }
        }
    }

    fn visit_item_enum(&mut self, e: &'ast syn::ItemEnum) {
        self.add(&e.ident, RustKind::Enum, None, &e.attrs);
    }

    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) {
        let saved = std::mem::take(&mut self.owner);
        syn::visit::visit_item_mod(self, m);
        self.owner = saved;
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        if let syn::Expr::Path(p) = &*c.func
            && let Some(last) = p.path.segments.last()
        {
            self.call(&last.ident);
        }
        syn::visit::visit_expr_call(self, c);
    }

    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        self.call(&m.method);
        syn::visit::visit_expr_method_call(self, m);
    }
}
