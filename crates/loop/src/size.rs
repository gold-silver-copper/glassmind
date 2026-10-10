//! Code size, counted in syntax tokens.
//!
//! Identifiers and literals count 1 whatever their length, punctuation counts
//! 1 per character group, each pair of brackets counts 1, and comments and
//! doc comments count 0, so names and docs are free. Test modules
//! (`#[cfg(test)]`) are not counted.

use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;

/// The crates whose code is glassmind's mind, and so is priced.
pub const MIND_CRATES: &[&str] = &[
    "crates/rules",
    "crates/meaning",
    "crates/grammar",
    "crates/english",
    "crates/world-rust",
    "crates/english-rust",
    "crates/mind",
];

fn count(tokens: TokenStream) -> u64 {
    tokens
        .into_iter()
        .map(|t| match t {
            TokenTree::Group(g) => 1 + count(g.stream()),
            _ => 1,
        })
        .sum()
}

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && a.meta.to_token_stream().to_string().replace(' ', "") == "cfg(test)"
    })
}

/// Strips doc attributes and test items, recursively through modules.
fn strip(items: &mut Vec<syn::Item>) {
    items.retain(|item| {
        let attrs: &[syn::Attribute] = match item {
            syn::Item::Mod(m) => &m.attrs,
            syn::Item::Fn(f) => &f.attrs,
            syn::Item::Impl(i) => &i.attrs,
            syn::Item::Use(u) => &u.attrs,
            _ => &[],
        };
        !is_cfg_test(attrs)
    });
    for item in items.iter_mut() {
        if let syn::Item::Mod(m) = item
            && let Some((_, inner)) = &mut m.content
        {
            strip(inner);
        }
    }
}

struct DocStripper;

impl syn::visit_mut::VisitMut for DocStripper {
    fn visit_attributes_mut(&mut self, attrs: &mut Vec<syn::Attribute>) {
        attrs.retain(|a| !a.path().is_ident("doc"));
    }
}

/// Syntax tokens in one Rust file's text. `None` if it does not parse.
pub fn file_tokens(text: &str) -> Option<u64> {
    let mut file = syn::parse_file(text).ok()?;
    strip(&mut file.items);
    syn::visit_mut::VisitMut::visit_file_mut(&mut DocStripper, &mut file);
    file.attrs.retain(|a| !a.path().is_ident("doc"));
    Some(count(file.to_token_stream()))
}

/// Syntax tokens in every `.rs` file under `dir`.
pub fn dir_tokens(dir: &Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs")
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                total += file_tokens(&text).unwrap_or(0);
            }
        }
    }
    total
}

/// Size of each mind crate in a checkout, and their total.
pub fn mind_size(repo: &Path) -> (Vec<(String, u64)>, u64) {
    let per: Vec<(String, u64)> = MIND_CRATES
        .iter()
        .map(|c| ((*c).to_owned(), dir_tokens(&repo.join(c).join("src"))))
        .collect();
    let total = per.iter().map(|(_, n)| n).sum();
    (per, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_docs_are_free_and_tests_uncounted() {
        let short = file_tokens("fn f(a: u8) -> u8 { a }").unwrap();
        let long = file_tokens(
            "/// Doubles nothing at all.\n// a comment\nfn a_much_longer_name(argument: u8) -> u8 { argument }\n#[cfg(test)] mod tests { fn t() { let x = 1; } }",
        )
        .unwrap();
        assert_eq!(short, long);
    }
}
