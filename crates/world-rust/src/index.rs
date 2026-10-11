//! Indexing a Rust source with `syn`: its items and who calls what.
//!
//! Follows the definitions in `docs/tasks.md`: which files count, what an
//! item is, and when a function calls a name.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::visit::Visit;

use crate::RustItem;
use crate::RustKind::{self, Field, Function};

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

/// Where an identifier is written: the index of its file in
/// [`SourceIndex::files`], its 1-based line and 0-based column in characters.
pub type Pos = (usize, u32, u32);

/// Everything indexed from one source.
#[derive(Debug, Clone, Default)]
pub struct SourceIndex {
    pub name: String,
    pub root: PathBuf,
    pub records: Vec<Record>,
    pub calls: Vec<Call>,
    /// The files indexed, relative to the root.
    pub files: Vec<String>,
    /// Every identifier token written in the files (lexed with
    /// `proc-macro2`, so also inside macro invocations), and where.
    pub idents: BTreeMap<String, BTreeSet<Pos>>,
    /// Where syntax alone says what kind of item an identifier names. A
    /// function: its declaration, the last segment of an expression path, a
    /// method call's name, a name in a `use` tree. A field: its declaration,
    /// `expr.f`, `S { f: v }`, `S { f: p }`. Macro bodies are not parsed, so
    /// no token in one is here.
    pub names: BTreeMap<Pos, RustKind>,
    /// Files that could not be read or parsed, with the reason.
    pub skipped: Vec<(String, String)>,
}

/// Every `.rs` file under `root`, relative and sorted, skipping `target`
/// directories and hidden files and directories.
pub fn rust_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        // Unreadable directories and entries are skipped.
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
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
        let (syntax, tokens) =
            match syn::parse_file(&text).and_then(|p| Ok((p, text.parse::<TokenStream>()?))) {
                Ok(p) => p,
                Err(e) => {
                    index.skipped.push((file, e.to_string()));
                    continue;
                }
            };
        let file_index = index.files.len();
        let lines: Vec<&str> = text.lines().collect();
        // Only tokens written as they are: a doc comment lexes as a `doc`
        // attribute whose span is the comment.
        let written = |ident: &Ident| {
            let start = ident.span().start();
            lines.get(start.line.wrapping_sub(1)).is_some_and(|l| {
                ident
                    .to_string()
                    .chars()
                    .zip(l.chars().skip(start.column))
                    .all(|(a, b)| a == b)
            })
        };
        for ident in idents_in(tokens).into_iter().filter(written) {
            index
                .idents
                .entry(ident.to_string())
                .or_default()
                .insert(pos(file_index, &ident));
        }
        index.files.push(file.clone());
        let mut visitor = Indexer {
            source: name,
            file: &file,
            file_index,
            text: &text,
            index: &mut index,
            current_fn: Vec::new(),
            owner: Vec::new(),
        };
        visitor.visit_file(&syntax);
    }
    index.calls.sort();
    index.calls.dedup();
    index
}

struct Indexer<'a> {
    source: &'a str,
    file: &'a str,
    file_index: usize,
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

/// The lines of a doc comment, trimmed.
fn doc_lines(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|a| match &a.meta {
            syn::Meta::NameValue(syn::MetaNameValue {
                path,
                value:
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }),
                ..
            }) if path.is_ident("doc") => Some(s.value().trim().to_owned()),
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

fn pos(file: usize, ident: &Ident) -> Pos {
    let start = ident.span().start();
    (
        file,
        line_of(ident.span()),
        u32::try_from(start.column).unwrap_or(u32::MAX),
    )
}

/// Every identifier in the tokens, also inside groups.
fn idents_in(tokens: TokenStream) -> Vec<Ident> {
    tokens
        .into_iter()
        .flat_map(|tree| match tree {
            TokenTree::Ident(i) => vec![i],
            TokenTree::Group(g) => idents_in(g.stream()),
            _ => Vec::new(),
        })
        .collect()
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
        name: &Ident,
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
            detail: None,
        };
        self.index.records.push(Record {
            item,
            signature: self.first_line(name.span()),
            doc: doc_lines(attrs),
            type_idents: Vec::new(),
        });
        // A function's or field's declaration names its kind.
        if matches!(kind, Function | Field) {
            self.names(name, kind);
        }
        self.index.records.len() - 1
    }

    fn function(
        &mut self,
        sig: &syn::Signature,
        attrs: &[syn::Attribute],
        body: Option<&syn::Block>,
    ) {
        let at = self.add(&sig.ident, Function, self.owner.last().cloned(), attrs);
        if let Some(body) = body {
            // Items nested in the body get their own owner context.
            let saved = std::mem::take(&mut self.owner);
            self.current_fn.push(at);
            self.visit_block(body);
            self.current_fn.pop();
            self.owner = saved;
        }
    }

    /// Visits the items of an `impl` or trait body as members of `owner`.
    /// They are only its functions (and consts); visiting them directly
    /// keeps them out of the enclosing function's calls.
    fn members(&mut self, owner: String, visit: impl FnOnce(&mut Self)) {
        self.owner.push(owner);
        let saved_fn = std::mem::take(&mut self.current_fn);
        visit(self);
        self.current_fn = saved_fn;
        self.owner.pop();
    }

    fn names(&mut self, ident: &Ident, kind: RustKind) {
        self.index.names.insert(pos(self.file_index, ident), kind);
    }

    fn names_field(&mut self, member: &syn::Member) {
        if let syn::Member::Named(ident) = member {
            self.names(ident, Field);
        }
    }

    fn call(&mut self, name: &Ident) {
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
        self.members(type_name(&i.self_ty).unwrap_or_default(), |v| {
            for item in &i.items {
                v.visit_impl_item(item);
            }
        });
    }

    fn visit_item_trait(&mut self, t: &'ast syn::ItemTrait) {
        self.add(&t.ident, RustKind::Trait, None, &t.attrs);
        self.members(t.ident.to_string(), |v| {
            for item in &t.items {
                v.visit_trait_item(item);
            }
        });
    }

    fn visit_item_struct(&mut self, s: &'ast syn::ItemStruct) {
        self.add(&s.ident, RustKind::Struct, None, &s.attrs);
        if let syn::Fields::Named(fields) = &s.fields {
            for field in &fields.named {
                if let Some(ident) = &field.ident {
                    self.add(ident, Field, Some(s.ident.to_string()), &field.attrs);
                    if let Some(record) = self.index.records.last_mut() {
                        record.type_idents = idents_in(field.ty.to_token_stream())
                            .iter()
                            .map(ToString::to_string)
                            .collect();
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

    fn visit_expr_path(&mut self, p: &'ast syn::ExprPath) {
        if let Some(last) = p.path.segments.last() {
            self.names(&last.ident, Function);
        }
        syn::visit::visit_expr_path(self, p);
    }

    /// `S { f }` names the field and a variable at once, so `f` is not
    /// taken as an expression path.
    fn visit_field_value(&mut self, f: &'ast syn::FieldValue) {
        if f.colon_token.is_some() {
            self.names_field(&f.member);
            syn::visit::visit_field_value(self, f);
        }
    }

    fn visit_field_pat(&mut self, f: &'ast syn::FieldPat) {
        if f.colon_token.is_some() {
            self.names_field(&f.member);
            syn::visit::visit_field_pat(self, f);
        }
    }

    fn visit_expr_field(&mut self, f: &'ast syn::ExprField) {
        self.names_field(&f.member);
        syn::visit::visit_expr_field(self, f);
    }

    fn visit_use_name(&mut self, u: &'ast syn::UseName) {
        self.names(&u.ident, Function);
    }

    fn visit_use_rename(&mut self, u: &'ast syn::UseRename) {
        self.names(&u.ident, Function);
        self.names(&u.rename, Function);
    }

    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        self.call(&m.method);
        self.names(&m.method, Function);
        syn::visit::visit_expr_method_call(self, m);
    }
}
