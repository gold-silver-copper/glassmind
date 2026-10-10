//! Loading one source: its files, items, relations and identifier tokens,
//! exactly as `docs/tasks.md` defines them.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use glassmind_answer::{ItemKey, ItemKind};
use proc_macro2::{LineColumn, Span, TokenStream, TokenTree};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

/// A position of an identifier token: file index, 1-based line, 0-based
/// character column.
pub type Pos = (usize, u32, u32);

/// One item, with the facts used to phrase requests about it.
#[derive(Debug, Clone)]
pub struct Item {
    pub key: ItemKey,
    /// Column of the name identifier.
    pub column: u32,
    /// For methods, the `impl` self type or the trait; for fields, the struct.
    pub owner: Option<String>,
    /// A one-line signature, e.g. `fn f(x: u32) -> u8`, `struct S`, `hp: u32`.
    pub signature: String,
    /// First non-empty line of the doc comment.
    pub doc: Option<String>,
}

impl Item {
    /// `Owner::name` for methods, `Struct.field` for fields, else the name.
    pub fn display(&self) -> String {
        match (&self.owner, self.key.kind) {
            (Some(owner), ItemKind::Field) => format!("{owner}.{}", self.key.name),
            (Some(owner), _) => format!("{owner}::{}", self.key.name),
            (None, _) => self.key.name.clone(),
        }
    }

    pub fn pos(&self, files: &[String]) -> Option<Pos> {
        let file = files.iter().position(|f| *f == self.key.file)?;
        Some((file, self.key.line, self.column))
    }
}

/// Everything the oracle knows about one source.
#[derive(Debug, Default)]
pub struct Source {
    pub name: String,
    /// Parsed files, relative paths with `/` separators, sorted.
    pub files: Vec<String>,
    /// Files that lexed or failed to lex but did not parse; skipped.
    pub skipped: Vec<String>,
    pub items: Vec<Item>,
    /// Function item index -> names it calls.
    pub calls: BTreeMap<usize, BTreeSet<String>>,
    /// Struct item index -> its field item indices.
    pub struct_fields: BTreeMap<usize, Vec<usize>>,
    /// Field item index -> its struct item index.
    pub field_owner: BTreeMap<usize, usize>,
    /// Field item index -> identifiers in its type tokens (lifetimes excluded).
    pub field_type_idents: BTreeMap<usize, BTreeSet<String>>,
    /// Identifier text -> every position where it is written, in parsed files.
    pub tokens: BTreeMap<String, BTreeSet<Pos>>,
    /// Every identifier written anywhere, including in files that lex but do
    /// not parse (used for freshness, conservatively).
    pub taken: BTreeSet<String>,
    pub roles: Roles,
}

/// Syntactic roles of identifier positions, used for rename exactness.
#[derive(Debug, Default)]
pub struct Roles {
    /// `expr.f`, `S { f: v }`, `S { f: p }`.
    pub field_use: BTreeSet<Pos>,
    /// `S { f }` in expressions or patterns.
    pub shorthand: BTreeSet<Pos>,
    /// Last segment of an expression path, a method call's name, a name in a
    /// `use` tree.
    pub fn_use: BTreeSet<Pos>,
}

impl Source {
    /// Indices of items with this name.
    pub fn named(&self, name: &str) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, it)| it.key.name == name)
            .map(|(i, _)| i)
            .collect()
    }

    /// The one item with this name, if exactly one item of any kind has it.
    pub fn unique(&self, name: &str) -> Option<usize> {
        match self.named(name).as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }

    pub fn item(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }

    pub fn key(&self, index: usize) -> Option<ItemKey> {
        self.item(index).map(|it| it.key.clone())
    }

    /// Item names counted by occurrence, for uniqueness checks.
    pub fn name_counts(&self) -> BTreeMap<&str, usize> {
        let mut counts = BTreeMap::new();
        for item in &self.items {
            *counts.entry(item.key.name.as_str()).or_insert(0) += 1;
        }
        counts
    }

    pub fn file_name(&self, index: usize) -> &str {
        self.files.get(index).map_or("", String::as_str)
    }
}

/// Every `*.rs` file under `root`, skipping `target` directories and names
/// starting with `.`, as sorted relative `/` paths.
fn rust_files(root: &Path) -> std::io::Result<Vec<(String, PathBuf)>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let path = entry.path();
            let kind = entry.file_type()?;
            if kind.is_dir() {
                if name != "target" {
                    stack.push(path);
                }
            } else if kind.is_file() && name.ends_with(".rs") {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push((rel, path));
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Strip a BOM and a shebang line the way `syn::parse_file` does, keeping
/// line numbers.
fn preprocess(text: &str) -> &str {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    if let Some(rest) = text.strip_prefix("#!")
        && !rest.trim_start().starts_with('[')
    {
        return text.find('\n').map_or("", |i| text.get(i..).unwrap_or(""));
    }
    text
}

/// Load and analyse a source.
pub fn load(name: &str, root: &Path) -> std::io::Result<Source> {
    let mut source = Source {
        name: name.to_string(),
        ..Source::default()
    };
    for (rel, path) in rust_files(root)? {
        let raw = fs::read_to_string(&path)?;
        let text = preprocess(&raw);
        let lines: Vec<Vec<char>> = text.split('\n').map(|l| l.chars().collect()).collect();
        let Ok(stream) = TokenStream::from_str(text) else {
            source.skipped.push(rel);
            continue;
        };
        let mut idents = Vec::new();
        collect_idents(stream.clone(), &lines, &mut idents);
        for (ident, _) in &idents {
            source.taken.insert(ident.clone());
        }
        let Ok(file) = syn::parse2::<syn::File>(stream) else {
            source.skipped.push(rel);
            continue;
        };
        let file_index = source.files.len();
        source.files.push(rel.clone());
        for (ident, (line, col)) in idents {
            source
                .tokens
                .entry(ident)
                .or_default()
                .insert((file_index, line, col));
        }
        let mut visitor = Collector {
            source: &mut source,
            file: rel,
            file_index,
            lines: &lines,
            current_fn: None,
            owner: None,
        };
        visitor.visit_file(&file);
    }
    Ok(source)
}

/// Identifier tokens actually written in the text (not those proc-macro2
/// synthesizes for doc comments, such as `doc`), with 1-based line and
/// 0-based char column.
fn collect_idents(stream: TokenStream, lines: &[Vec<char>], out: &mut Vec<(String, (u32, u32))>) {
    for tree in stream {
        match tree {
            TokenTree::Group(group) => collect_idents(group.stream(), lines, out),
            TokenTree::Ident(ident) => {
                let text = ident.to_string();
                let start = ident.span().start();
                if written_at(lines, start, &text) {
                    out.push((text, (line_u32(start.line), col_u32(start.column))));
                }
            }
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
}

fn line_u32(line: usize) -> u32 {
    u32::try_from(line).unwrap_or(u32::MAX)
}

fn col_u32(col: usize) -> u32 {
    u32::try_from(col).unwrap_or(u32::MAX)
}

fn written_at(lines: &[Vec<char>], at: LineColumn, text: &str) -> bool {
    let Some(line) = at.line.checked_sub(1).and_then(|l| lines.get(l)) else {
        return false;
    };
    let mut chars = line.iter().skip(at.column);
    text.chars().all(|c| chars.next() == Some(&c))
}

/// The source text a span covers, on one line with whitespace collapsed.
fn span_text(lines: &[Vec<char>], start: LineColumn, end: LineColumn) -> String {
    one_line(&raw_span_text(lines, start, end))
}

/// The source text a span covers, lines joined with spaces.
fn raw_span_text(lines: &[Vec<char>], start: LineColumn, end: LineColumn) -> String {
    let mut out = String::new();
    for line_no in start.line..=end.line {
        let Some(line) = line_no.checked_sub(1).and_then(|l| lines.get(l)) else {
            break;
        };
        let from = if line_no == start.line {
            start.column
        } else {
            0
        };
        let to = if line_no == end.line {
            end.column
        } else {
            line.len()
        };
        out.extend(line.iter().skip(from).take(to.saturating_sub(from)));
        out.push(' ');
    }
    out
}

/// Collapse whitespace and tidy the spaces a multi-line signature leaves.
fn one_line(text: &str) -> String {
    let mut out = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for (from, to) in [
        ("( ", "("),
        (" )", ")"),
        (",)", ")"),
        ("< ", "<"),
        (" >", ">"),
        (",>", ">"),
    ] {
        out = out.replace(from, to);
    }
    if out.chars().count() > 160 {
        out = out.chars().take(157).collect::<String>() + "...";
    }
    out
}

fn first_doc_line(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let syn::Meta::NameValue(nv) = &attr.meta
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &nv.value
        {
            let value = s.value();
            if let Some(line) = value.lines().map(str::trim).find(|l| !l.is_empty()) {
                return Some(line.to_string());
            }
        }
    }
    None
}

/// Identifiers in a type's tokens (its source text, re-lexed), skipping
/// lifetime names (`'a`).
fn type_idents(text: &str) -> BTreeSet<String> {
    fn walk(stream: TokenStream, out: &mut BTreeSet<String>) {
        let mut after_quote = false;
        for tree in stream {
            match tree {
                TokenTree::Group(group) => {
                    walk(group.stream(), out);
                    after_quote = false;
                }
                TokenTree::Ident(ident) => {
                    if !after_quote {
                        out.insert(ident.to_string());
                    }
                    after_quote = false;
                }
                TokenTree::Punct(p) => after_quote = p.as_char() == '\'',
                TokenTree::Literal(_) => after_quote = false,
            }
        }
    }
    let mut out = BTreeSet::new();
    if let Ok(stream) = TokenStream::from_str(text) {
        walk(stream, &mut out);
    }
    out
}

fn type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        syn::Type::Reference(r) => type_name(&r.elem),
        syn::Type::Paren(p) => type_name(&p.elem),
        syn::Type::Group(g) => type_name(&g.elem),
        _ => None,
    }
}

struct Collector<'a> {
    source: &'a mut Source,
    file: String,
    file_index: usize,
    lines: &'a [Vec<char>],
    /// The function whose own body is being visited.
    current_fn: Option<usize>,
    /// The `impl` self type or trait whose items are being visited.
    owner: Option<String>,
}

impl Collector<'_> {
    fn pos(&self, span: Span) -> Pos {
        let start = span.start();
        (self.file_index, line_u32(start.line), col_u32(start.column))
    }

    fn push_item(
        &mut self,
        ident: &syn::Ident,
        kind: ItemKind,
        owner: Option<String>,
        signature: String,
        doc: Option<String>,
    ) -> usize {
        let start = ident.span().start();
        let index = self.source.items.len();
        self.source.items.push(Item {
            key: ItemKey {
                source: self.source.name.clone(),
                file: self.file.clone(),
                line: line_u32(start.line),
                name: ident.to_string(),
                kind,
            },
            column: col_u32(start.column),
            owner,
            signature,
            doc,
        });
        index
    }

    fn text(&self, span: Span) -> String {
        span_text(self.lines, span.start(), span.end())
    }

    /// Record a function and visit its body as its own.
    fn function(
        &mut self,
        attrs: &[syn::Attribute],
        sig: &syn::Signature,
        owner: Option<String>,
        body: Option<&syn::Block>,
    ) {
        let signature = self.text(sig.span());
        let doc = first_doc_line(attrs);
        let index = self.push_item(&sig.ident, ItemKind::Function, owner, signature, doc);
        self.source.calls.entry(index).or_default();
        let saved = (self.current_fn.take(), self.owner.take());
        self.visit_signature(sig);
        if let Some(body) = body {
            self.current_fn = Some(index);
            self.visit_block(body);
        }
        (self.current_fn, self.owner) = saved;
    }

    fn call(&mut self, name: String) {
        if let Some(f) = self.current_fn {
            self.source.calls.entry(f).or_default().insert(name);
        }
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        // Items nested in a function body do not belong to it.
        let saved = (self.current_fn.take(), self.owner.take());
        visit::visit_item(self, item);
        (self.current_fn, self.owner) = saved;
    }

    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.function(&f.attrs, &f.sig, None, Some(&f.block));
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        let owner = self.owner.clone();
        self.function(&f.attrs, &f.sig, owner, Some(&f.block));
    }

    fn visit_trait_item_fn(&mut self, f: &'ast syn::TraitItemFn) {
        let owner = self.owner.clone();
        self.function(&f.attrs, &f.sig, owner, f.default.as_ref());
    }

    fn visit_foreign_item_fn(&mut self, f: &'ast syn::ForeignItemFn) {
        self.function(&f.attrs, &f.sig, None, None);
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        self.owner = type_name(&i.self_ty);
        visit::visit_item_impl(self, i);
    }

    fn visit_item_trait(&mut self, t: &'ast syn::ItemTrait) {
        let doc = first_doc_line(&t.attrs);
        self.push_item(
            &t.ident,
            ItemKind::Trait,
            None,
            format!("trait {}", t.ident),
            doc,
        );
        self.owner = Some(t.ident.to_string());
        visit::visit_item_trait(self, t);
    }

    fn visit_item_enum(&mut self, e: &'ast syn::ItemEnum) {
        let doc = first_doc_line(&e.attrs);
        let sig = format!("enum {}{}", e.ident, self.generics(&e.generics));
        self.push_item(&e.ident, ItemKind::Enum, None, sig, doc);
        visit::visit_item_enum(self, e);
    }

    fn visit_item_struct(&mut self, s: &'ast syn::ItemStruct) {
        let doc = first_doc_line(&s.attrs);
        let sig = format!("struct {}{}", s.ident, self.generics(&s.generics));
        let index = self.push_item(&s.ident, ItemKind::Struct, None, sig, doc);
        let mut fields = Vec::new();
        if let syn::Fields::Named(named) = &s.fields {
            for field in &named.named {
                let Some(ident) = &field.ident else { continue };
                let ty = field.ty.span();
                let sig = format!("{ident}: {}", self.text(ty));
                let idents = type_idents(&raw_span_text(self.lines, ty.start(), ty.end()));
                let doc = first_doc_line(&field.attrs);
                let f = self.push_item(ident, ItemKind::Field, Some(s.ident.to_string()), sig, doc);
                self.source.field_owner.insert(f, index);
                self.source.field_type_idents.insert(f, idents);
                fields.push(f);
            }
        }
        self.source.struct_fields.insert(index, fields);
        visit::visit_item_struct(self, s);
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*call.func
            && let Some(last) = path.path.segments.last()
        {
            self.call(last.ident.to_string());
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.call(call.method.to_string());
        let pos = self.pos(call.method.span());
        self.source.roles.fn_use.insert(pos);
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if let Some(last) = path.path.segments.last() {
            let pos = self.pos(last.ident.span());
            self.source.roles.fn_use.insert(pos);
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_expr_field(&mut self, field: &'ast syn::ExprField) {
        if let syn::Member::Named(ident) = &field.member {
            let pos = self.pos(ident.span());
            self.source.roles.field_use.insert(pos);
        }
        visit::visit_expr_field(self, field);
    }

    fn visit_field_value(&mut self, fv: &'ast syn::FieldValue) {
        if let syn::Member::Named(ident) = &fv.member {
            let pos = self.pos(ident.span());
            if fv.colon_token.is_some() {
                self.source.roles.field_use.insert(pos);
            } else {
                self.source.roles.shorthand.insert(pos);
            }
        }
        visit::visit_field_value(self, fv);
    }

    fn visit_field_pat(&mut self, fp: &'ast syn::FieldPat) {
        if let syn::Member::Named(ident) = &fp.member {
            let pos = self.pos(ident.span());
            if fp.colon_token.is_some() {
                self.source.roles.field_use.insert(pos);
            } else {
                self.source.roles.shorthand.insert(pos);
            }
        }
        visit::visit_field_pat(self, fp);
    }

    fn visit_use_name(&mut self, u: &'ast syn::UseName) {
        let pos = self.pos(u.ident.span());
        self.source.roles.fn_use.insert(pos);
    }

    fn visit_use_rename(&mut self, u: &'ast syn::UseRename) {
        for ident in [&u.ident, &u.rename] {
            let pos = self.pos(ident.span());
            self.source.roles.fn_use.insert(pos);
        }
    }
}

impl Collector<'_> {
    fn generics(&self, generics: &syn::Generics) -> String {
        match (&generics.lt_token, &generics.gt_token) {
            (Some(lt), Some(gt)) => span_text(self.lines, lt.span.start(), gt.span.end()),
            _ => String::new(),
        }
    }
}
