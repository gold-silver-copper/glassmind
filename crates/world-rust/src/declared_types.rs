//! Exact written types, independent of the identifier-only TypeMentions relation.
use glassmind_meaning::{Name, Trouble};
use glassmind_rules::{HAND, RuleInfo};
use serde::{Deserialize, Serialize};
use syn::{spanned::Spanned, visit::Visit};

use crate::{Rust, RustItem, RustKind, RustWorld};

pub const DECLARED_TYPE: RuleInfo = RuleInfo::new("rust.metadata.declared_type", HAND);

/// A named struct owner and named field; neither is inferred or case-folded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclaredTypeQuestion {
    pub owner: String,
    pub field: String,
}

/// Source spelling, not a resolved, expanded or inferred Rust type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclaredType {
    pub owner: RustItem,
    pub field: RustItem,
    pub spelling: String,
    /// Half-open UTF-8 byte offsets in field.file, including internal whitespace/comments.
    pub start: usize,
    pub end: usize,
}

impl RustWorld {
    pub fn declared_type(
        &self,
        question: &DeclaredTypeQuestion,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<DeclaredType, Trouble<Rust>> {
        self.fire(DECLARED_TYPE, fired)?;
        if self.sources.is_empty()
            || self
                .sources
                .iter()
                .any(|s| s.files.is_empty() || !s.skipped.is_empty() || s.declared_types_uncertain)
        {
            return Err(Trouble::Unsupported);
        }
        // Alias/enum/union declarations also make an unqualified owner name
        // ambiguous, even though only named struct fields are supported.
        if self
            .sources
            .iter()
            .flat_map(|s| &s.declared_owner_names)
            .filter(|name| *name == &question.owner)
            .count()
            != 1
        {
            return Err(Trouble::Unsupported);
        }
        let owners: Vec<_> = self
            .records()
            .filter(|r| r.item.kind == RustKind::Struct && r.item.name == question.owner)
            .collect();
        let owner = match owners.as_slice() {
            [owner] => &owner.item,
            [] => {
                return Err(Trouble::NoSuchName {
                    name: Name(question.owner.clone()),
                });
            }
            _ => {
                return Err(Trouble::Ambiguous {
                    candidates: owners.iter().map(|r| r.item.clone()).collect(),
                });
            }
        };
        let fields: Vec<_> = self
            .records()
            .filter_map(|r| r.declared_type.as_ref())
            .filter(|d| &d.owner == owner && d.field.name == question.field)
            .collect();
        match fields.as_slice() {
            [field] => Ok((*field).clone()),
            [] => Err(Trouble::NoSuchName {
                name: Name(question.field.clone()),
            }),
            _ => Err(Trouble::Ambiguous {
                candidates: fields.iter().map(|d| d.field.clone()).collect(),
            }),
        }
    }
}

/// Convert syn's character columns to UTF-8 offsets without changing source text.
pub(crate) fn source_range(text: &str, span: proc_macro2::Span) -> Option<(usize, usize)> {
    fn offset(text: &str, at: proc_macro2::LineColumn) -> Option<usize> {
        let mut base = 0;
        for (i, line) in text.split_inclusive('\n').enumerate() {
            if i + 1 == at.line {
                return if at.column == line.chars().count() {
                    Some(base + line.len())
                } else {
                    line.char_indices().nth(at.column).map(|(b, _)| base + b)
                };
            }
            base += line.len();
        }
        None
    }
    Some((offset(text, span.start())?, offset(text, span.end())?))
}

/// All explicit type-owner declarations, including unsupported owner kinds.
pub(crate) fn owner_names(file: &syn::File) -> Vec<String> {
    struct Owners(Vec<String>);
    impl<'ast> Visit<'ast> for Owners {
        fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
            self.0.push(item.ident.to_string());
        }
        fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
            self.0.push(item.ident.to_string());
        }
        fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
            self.0.push(item.ident.to_string());
        }
        fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
            self.0.push(item.ident.to_string());
        }
        fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
            self.0.push(item.ident.to_string());
        }
    }
    let mut owners = Owners(Vec::new());
    owners.visit_file(file);
    owners.0
}

/// Syntax presence is not cfg or macro expansion. Refuse metadata if either
/// could change declaration availability; ordinary index behavior is untouched.
pub(crate) fn uncertain(file: &syn::File) -> bool {
    struct Guard(bool);
    impl<'ast> Visit<'ast> for Guard {
        fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
            let path = attr.path();
            if path.is_ident("derive") {
                let safe = attr
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .is_ok_and(|paths| {
                        paths.iter().all(|p| {
                            p.get_ident().is_some_and(|i| {
                                matches!(
                                    i.to_string().as_str(),
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
                    });
                self.0 |= !safe;
            } else if ![
                "doc",
                "allow",
                "warn",
                "deny",
                "forbid",
                "repr",
                "inline",
                "must_use",
                "deprecated",
                "non_exhaustive",
            ]
            .iter()
            .any(|p| path.is_ident(p))
            {
                self.0 = true;
            }
        }
        fn visit_macro(&mut self, _: &'ast syn::Macro) {
            self.0 = true;
        }
        fn visit_type(&mut self, ty: &'ast syn::Type) {
            if matches!(ty, syn::Type::Verbatim(_) | syn::Type::Infer(_)) {
                self.0 = true;
            }
            syn::visit::visit_type(self, ty);
        }
    }
    let mut guard = Guard(false);
    guard.visit_file(file);
    guard.0
}

pub(crate) fn spelling(text: &str, ty: &syn::Type) -> Option<(String, usize, usize)> {
    let (start, end) = source_range(text, ty.span())?;
    Some((text.get(start..end)?.to_owned(), start, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glassmind_rules::Switches;

    #[test]
    fn declared_syntax_and_provenance_are_reusable_without_english() {
        let root = std::env::temp_dir().join(format!("declared-types-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = "type Alias = u8; struct État<'a> { café: &'a /* keep */ (Alias, [u8; 4]) }";
        std::fs::write(root.join("types.rs"), source).unwrap();
        let world = RustWorld::open(&[("fixture".into(), root.as_path())], &Switches::all_on());
        let q = DeclaredTypeQuestion {
            owner: "État".into(),
            field: "café".into(),
        };
        let mut fired = Vec::new();
        let answer = world.declared_type(&q, &mut fired).unwrap();
        assert_eq!(answer.spelling, "&'a /* keep */ (Alias, [u8; 4])");
        assert_eq!(&source[answer.start..answer.end], answer.spelling);
        assert_eq!(answer.field.file, "types.rs");
        assert_eq!(answer.owner.name, "État");
        assert_eq!(fired, [DECLARED_TYPE]);
        let disabled = RustWorld::open(
            &[("fixture".into(), root.as_path())],
            &Switches::without([DECLARED_TYPE.id]),
        );
        assert!(disabled.declared_type(&q, &mut Vec::new()).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn declaration_uncertainty_is_not_type_resolution() {
        for source in [
            "#[cfg(any())] struct X { a: u8 }",
            "make!();",
            "struct X { a: ty!() }",
            "#[custom] struct X;",
        ] {
            assert!(uncertain(&syn::parse_file(source).unwrap()));
        }
        assert!(!uncertain(
            &syn::parse_file("type Alias = Vec<u8>; struct X { a: Alias }").unwrap()
        ));
    }
}
