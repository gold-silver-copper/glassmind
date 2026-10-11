//! Closed package graph. A symbol identity includes its Cargo target root.
use crate::index::SourceIndex;
use crate::rename_types::{Module, imports, modules_with_tests};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

type Name = Vec<String>;

pub(crate) fn inert(a: &syn::Attribute) -> bool {
    if a.path().is_ident("doc") {
        let syn::Meta::NameValue(value) = &a.meta else {
            return false;
        };
        let syn::Expr::Lit(expr) = &value.value else {
            return false;
        };
        let syn::Lit::Str(text) = &expr.lit else {
            return false;
        };
        let text = text.value();
        // Rustdoc can execute fenced/indented examples and resolve intra-doc
        // links. Only literal prose, not those name-sensitive forms, is inert.
        return !text.contains("```")
            && !text.contains("~~~")
            && !text.contains('[')
            && !text.contains(']')
            && !text
                .lines()
                .any(|line| line.starts_with("    ") || line.starts_with('\t'));
    }
    if a.path().is_ident("test") {
        return matches!(a.meta, syn::Meta::Path(_));
    }
    if !a.path().is_ident("derive") {
        return false;
    }
    a.parse_args_with(syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
        .is_ok_and(|paths| {
            !paths.is_empty()
                && paths.iter().all(|p| {
                    [
                        "Clone",
                        "Copy",
                        "PartialEq",
                        "Eq",
                        "PartialOrd",
                        "Ord",
                        "Hash",
                        "Default",
                    ]
                    .iter()
                    .any(|s| p.is_ident(s))
                })
        })
}

// Keep standard output calls opaque to edits, but do not hide nested macros.
pub(crate) fn output_macro(m: &syn::Macro) -> bool {
    fn plain(tokens: proc_macro2::TokenStream, depth: usize) -> bool {
        depth <= 64
            && tokens.into_iter().all(|t| match t {
                proc_macro2::TokenTree::Punct(p) => p.as_char() != '!',
                proc_macro2::TokenTree::Group(g) => plain(g.stream(), depth + 1),
                _ => true,
            })
    }
    (m.path.is_ident("println") || m.path.is_ident("eprintln")) && plain(m.tokens.clone(), 0)
}

pub(crate) struct Package {
    pub(crate) modules: Vec<Module>,
    symbols: BTreeSet<Name>,
    uses: BTreeMap<Name, (Name, Name)>,
    library: Option<String>,
}

impl Package {
    pub(crate) fn load(source: &SourceIndex) -> Option<Self> {
        Self::load_mode(source, false)
    }

    pub(crate) fn load_add_field(source: &SourceIndex) -> Option<Self> {
        Self::load_mode(source, true)
    }

    fn load_mode(source: &SourceIndex, tests: bool) -> Option<Self> {
        let manifest = source.root.join("Cargo.toml");
        let mut library = None;
        // Discover ambiguity by paths, without loading unrelated package sources.
        let mut pending = vec![source.root.clone()];
        let mut entries = 0usize;
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(&dir).ok()? {
                let entry = entry.ok()?;
                entries += 1;
                if entries > 4096 {
                    return None;
                }
                let name = entry.file_name();
                if name == "target" || name == ".git" {
                    continue;
                }
                let kind = entry.file_type().ok()?;
                if kind.is_symlink() || name == ".cargo" {
                    return None;
                }
                if kind.is_dir() {
                    pending.push(entry.path());
                } else if (name == "Cargo.toml" && entry.path() != manifest)
                    || (entry.path().extension().is_some_and(|e| e == "rs")
                        && !source
                            .files
                            .iter()
                            .any(|f| source.root.join(f) == entry.path()))
                {
                    return None;
                }
            }
        }
        if manifest.exists() {
            let value: toml::Value =
                toml::from_str(&std::fs::read_to_string(manifest).ok()?).ok()?;
            let package = value.get("package")?.as_table()?;
            if value.get("workspace").is_none()
                && source
                    .root
                    .ancestors()
                    .skip(1)
                    .any(|p| p.join("Cargo.toml").exists())
            {
                return None;
            }
            let name = package.get("name")?.as_str()?;
            if !matches!(
                package.get("edition").and_then(toml::Value::as_str),
                Some("2018" | "2021" | "2024")
            ) || package.contains_key("workspace")
                || package.contains_key("build")
                || value
                    .get("workspace")
                    .is_some_and(|w| w.as_table().is_none_or(|t| !t.is_empty()))
                || value.get("patch").is_some()
                || value.get("replace").is_some()
                // Explicit additional targets can compile an already indexed
                // file as another root. Target-specific dependencies can shadow
                // the library's extern name without a top-level dependency.
                || ["example", "test", "bench", "target"].iter().any(|k| value.get(*k).is_some())
            {
                return None;
            }
            for key in ["autolib", "autobins"] {
                if package.get(key).is_some_and(|v| v.as_bool() != Some(true)) {
                    return None;
                }
            }
            if let Some(lib) = value.get("lib") {
                let lib = lib.as_table()?;
                if lib.keys().any(|k| !["name", "path"].contains(&k.as_str()))
                    || lib
                        .get("path")
                        .is_some_and(|v| v.as_str() != Some("src/lib.rs"))
                {
                    return None;
                }
            }
            if let Some(bins) = value.get("bin") {
                let bins = bins.as_array()?;
                if bins.len() != 1 {
                    return None;
                }
                let bin = bins.first()?.as_table()?;
                if bin.keys().any(|k| !["name", "path"].contains(&k.as_str()))
                    || bin
                        .get("path")
                        .is_some_and(|v| v.as_str() != Some("src/main.rs"))
                {
                    return None;
                }
            }
            let libname = value
                .get("lib")
                .and_then(|v| v.get("name"))
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| name.replace('-', "_"));
            syn::parse_str::<proc_macro2::Ident>(&libname).ok()?;
            for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if value
                    .get(section)
                    .and_then(toml::Value::as_table)
                    .is_some_and(|deps| deps.keys().any(|k| k.replace('-', "_") == libname))
                {
                    return None;
                }
            }
            library = Some(libname);
        }
        let roots: Vec<_> = ["src/lib.rs", "src/main.rs", "lib.rs", "main.rs"]
            .into_iter()
            .filter(|r| source.files.iter().any(|f| f == r))
            .collect();
        if roots.is_empty()
            || (roots.len() > 1 && (library.is_none() || roots != ["src/lib.rs", "src/main.rs"]))
        {
            return None;
        }
        let mut all = Vec::new();
        for root in roots {
            let parsed =
                syn::parse_file(&std::fs::read_to_string(source.root.join(root)).ok()?).ok()?;
            if !parsed
                .attrs
                .iter()
                .all(|a| a.path().is_ident("doc") && inert(a))
            {
                return None;
            }
            let root_name = if root.ends_with("lib.rs") {
                "@lib"
            } else {
                "@bin"
            };
            modules_with_tests(
                source,
                root,
                vec![root_name.into()],
                Path::new(root).parent()?,
                parsed.items,
                &mut all,
                tests,
            )?;
        }
        let files: BTreeSet<_> = all.iter().map(|m| m.file).collect();
        if files.len() != source.files.len() {
            return None;
        }
        // A file compiled into two targets has two meanings but only one edit span.
        for file in files {
            let roots: BTreeSet<_> = all
                .iter()
                .filter(|m| m.file == file)
                .filter_map(|m| m.path.first())
                .collect();
            if roots.len() != 1 {
                return None;
            }
        }
        let mut result = Self {
            modules: all,
            symbols: BTreeSet::new(),
            uses: BTreeMap::new(),
            library,
        };
        let mut globs = Vec::new();
        for m in &result.modules {
            result.symbols.insert(m.path.clone());
            let mut locals = BTreeSet::new();
            for node in &m.items {
                let id = match node {
                    syn::Item::Struct(n) => Some(&n.ident),
                    syn::Item::Enum(n) => Some(&n.ident),
                    syn::Item::Mod(n) => Some(&n.ident),
                    syn::Item::Type(n) => Some(&n.ident),
                    syn::Item::Trait(n) => Some(&n.ident),
                    syn::Item::Fn(n) => Some(&n.sig.ident),
                    syn::Item::Const(n) => Some(&n.ident),
                    syn::Item::Static(n) => Some(&n.ident),
                    syn::Item::ExternCrate(_) => return None,
                    _ => None,
                };
                if let Some(id) = id {
                    if tests && crate::add_fields::standard_macro(&id.to_string()) {
                        return None;
                    }
                    if !locals.insert(id.to_string()) {
                        return None;
                    }
                    let mut key = m.path.clone();
                    key.push(id.to_string());
                    result.symbols.insert(key.clone());
                    if let syn::Item::Enum(e) = node {
                        for v in &e.variants {
                            let mut vkey = key.clone();
                            vkey.push(v.ident.to_string());
                            result.symbols.insert(vkey);
                        }
                    }
                }
                if let syn::Item::Use(u) = node {
                    if u.leading_colon.is_some()
                        || !u.attrs.iter().all(|a| a.path().is_ident("doc") && inert(a))
                    {
                        return None;
                    }
                    if tests && quote::quote!(#u).to_string().replace(' ', "") == "usesuper::*;" {
                        globs.push(m.path.clone());
                        continue;
                    }
                    let mut leaves = Vec::new();
                    imports(&u.tree, &mut Vec::new(), &mut leaves)?;
                    for (ids, alias) in leaves {
                        let mut names: Name = ids.iter().map(ToString::to_string).collect();
                        let self_import =
                            names.last().is_some_and(|n| n == "self") && names.len() > 1;
                        if self_import {
                            names.pop();
                        }
                        let local = if self_import && alias == "self" {
                            names.last()?.clone()
                        } else {
                            alias.to_string()
                        };
                        if (tests && crate::add_fields::standard_macro(&local))
                            || [
                                "println",
                                "eprintln",
                                "Debug",
                                "Clone",
                                "Copy",
                                "PartialEq",
                                "Eq",
                                "PartialOrd",
                                "Ord",
                                "Hash",
                                "Default",
                            ]
                            .contains(&local.as_str())
                            || !locals.insert(local.clone())
                        {
                            return None;
                        }
                        let mut key = m.path.clone();
                        key.push(local);
                        result.uses.insert(key, (m.path.clone(), names));
                    }
                }
            }
        }
        for module in globs {
            let (_, parent) = module.split_last()?;
            let keys: Vec<_> = result
                .symbols
                .iter()
                .chain(result.uses.keys())
                .filter(|k| k.len() == parent.len() + 1 && k.starts_with(parent))
                .cloned()
                .collect();
            for key in keys {
                let mut local = module.clone();
                let name = key.last()?.clone();
                local.push(name.clone());
                if result.symbols.contains(&local) || result.uses.contains_key(&local) {
                    return None;
                }
                result.uses.insert(local, (parent.to_vec(), vec![name]));
            }
        }
        // Resolve every import, even unused imports. Bound cycles deterministically.
        for (module, names) in result.uses.values() {
            result.resolve(module, names)?;
        }
        Some(result)
    }

    pub(crate) fn resolve(&self, module: &[String], names: &[String]) -> Option<Name> {
        self.resolve_inner(module, names, &mut BTreeSet::new(), 0)
    }
    fn resolve_inner(
        &self,
        module: &[String],
        names: &[String],
        seen: &mut BTreeSet<Name>,
        depth: usize,
    ) -> Option<Name> {
        if depth > 64 {
            return None;
        }
        let first = names.first()?;
        let (mut at, skip) = if first == "crate" {
            (vec![module.first()?.clone()], 1)
        } else if first == "self" {
            (module.to_vec(), 1)
        } else if first == "super" {
            let mut p = module.to_vec();
            p.pop()?;
            if p.is_empty() {
                return None;
            }
            (p, 1)
        } else if self.library.as_ref() == Some(first)
            && module.first().is_some_and(|n| n == "@bin")
        {
            // A local declaration/import of the crate name is ambiguous.
            let mut local = module.to_vec();
            local.push(first.clone());
            if self.symbols.contains(&local) || self.uses.contains_key(&local) {
                return None;
            }
            (vec!["@lib".into()], 1)
        } else {
            (module.to_vec(), 0)
        };
        for name in names.iter().skip(skip) {
            if name == "super" {
                if at.len() <= 1 {
                    return None;
                }
                at.pop();
                continue;
            }
            at.push(name.clone());
            if let Some((scope, target)) = self.uses.get(&at) {
                if !seen.insert(at.clone()) {
                    return None;
                }
                let key = at.clone();
                at = self.resolve_inner(scope, target, seen, depth + 1)?;
                seen.remove(&key);
            } else if !self.symbols.contains(&at) {
                return None;
            }
        }
        Some(at)
    }
    pub(crate) fn bindings(&self, m: &Module) -> Option<BTreeMap<String, Name>> {
        let mut out = BTreeMap::new();
        for key in self.symbols.iter().chain(self.uses.keys()) {
            if key.len() == m.path.len() + 1 && key.starts_with(&m.path) {
                let name = key.last()?.clone();
                out.insert(name.clone(), self.resolve(&m.path, &[name])?);
            }
        }
        if let Some(lib) = &self.library
            && m.path.first().is_some_and(|s| s == "@bin")
        {
            out.entry(lib.clone()).or_insert(vec!["@lib".into()]);
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn documentation_is_literal_prose_only() {
        let prose: syn::Attribute = syn::parse_quote!(#[doc = "Parcel is a message."]);
        let link: syn::Attribute = syn::parse_quote!(#[doc = "See [Parcel]."]);
        let code: syn::Attribute = syn::parse_quote!(#[doc = "```rust"]);
        let macro_doc: syn::Attribute = syn::parse_quote!(#[doc = include_str!("guide")]);
        assert!(inert(&prose));
        assert!(!inert(&link));
        assert!(!inert(&code));
        assert!(!inert(&macro_doc));
    }
    #[test]
    fn package_roots_reexports_and_exact_spans() {
        let root = std::env::temp_dir().join(format!("package-graph-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]\nname='parcel-box'\nversion='0.0.0'\nedition='2021'\n[lib]\nname='parcel_api'\n[workspace]\n").unwrap();
        std::fs::write(
            root.join("src/lib.rs"),
            "mod model { pub struct Parcel(pub u32); } pub use model::Parcel;\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/main.rs"),
            "use parcel_api::Parcel; fn main() { let p = Parcel(3); }\n",
        )
        .unwrap();
        let source = crate::index::index_source("package", &root);
        let package = Package::load(&source).unwrap();
        assert_eq!(
            package.resolve(&["@bin".into()], &["Parcel".into()]),
            Some(vec!["@lib".into(), "model".into(), "Parcel".into()])
        );
        let item = source
            .records
            .iter()
            .find(|r| r.item.name == "Parcel")
            .unwrap();
        assert_eq!(
            crate::rename_types::positions(&source, &item.item, "Packet")
                .unwrap()
                .len(),
            4
        );
        std::fs::write(
            root.join("src/lib.rs"),
            "pub struct Parcel(pub u32); pub struct Other; use self::Other as Local;",
        )
        .unwrap();
        std::fs::write(
            root.join("src/main.rs"),
            "use parcel_api::Parcel as Local; fn main() { let p = Local(3); }",
        )
        .unwrap();
        let source = crate::index::index_source("package", &root);
        let graph = Package::load(&source).unwrap();
        assert_eq!(
            graph.resolve(&["@lib".into()], &["Local".into()]),
            Some(vec!["@lib".into(), "Other".into()])
        );
        assert_eq!(
            graph.resolve(&["@bin".into()], &["Local".into()]),
            Some(vec!["@lib".into(), "Parcel".into()])
        );
        // Without an explicit lib name Cargo maps package hyphens to underscores.
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='parcel-box'\nversion='0.0.0'\nedition='2021'\n[workspace]\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/main.rs"),
            "use parcel_box::Parcel; fn main() { let p = Parcel(3); }",
        )
        .unwrap();
        let source = crate::index::index_source("package", &root);
        assert!(Package::load(&source).is_some());
        let manifest = root.join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest).unwrap();
        std::fs::write(
            &manifest,
            format!("{text}\n[[example]]\nname='again'\npath='src/lib.rs'\n"),
        )
        .unwrap();
        assert!(Package::load(&source).is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
