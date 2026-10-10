//! Task enumeration and deterministic selection.

use std::collections::{BTreeMap, BTreeSet};

use glassmind_answer::{Change, ItemKey, ItemKind, Refusal, Response};
use serde::Serialize;
use serde_json::{Value, json};

use crate::source::{Item, Pos, Source};

/// Every task kind of `docs/tasks.md`, in output order.
pub const KINDS: [&str; 13] = [
    "callers",
    "callees",
    "count_callers",
    "locate",
    "describe",
    "fields_of",
    "structs_with_field_type",
    "structs_with_field_named",
    "rename_field",
    "rename_function",
    "no_such_name",
    "fields_of_function",
    "ambiguous_rename_field",
];

/// One line of the tasks file.
#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub id: String,
    pub source: String,
    pub kind: String,
    pub args: Value,
    pub expected: Response,
    pub info: Value,
}

/// How much a candidate is wanted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// In the interesting range.
    Preferred,
    /// Zero callers: a small quota of these is wanted.
    Zero,
    /// Valid, used only to fill up.
    Fallback,
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub tier: Tier,
    pub task: Task,
}

/// FNV-1a, 64 bit.
pub fn fnv1a(parts: &[&str]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for byte in part.bytes().chain([0xff]) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    hash
}

fn keys(source: &Source, indices: impl IntoIterator<Item = usize>) -> BTreeSet<ItemKey> {
    indices.into_iter().filter_map(|i| source.key(i)).collect()
}

/// Display names of items, in the order of their keys.
fn names(source: &Source, indices: impl IntoIterator<Item = usize>) -> Vec<String> {
    let by_key: BTreeMap<ItemKey, String> = indices
        .into_iter()
        .filter_map(|i| source.item(i))
        .map(|it| (it.key.clone(), it.display()))
        .collect();
    by_key.into_values().collect()
}

fn kind_name(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Function => "function",
        ItemKind::Struct => "struct",
        ItemKind::Enum => "enum",
        ItemKind::Trait => "trait",
        ItemKind::Field => "field",
    }
}

fn target_info(item: &Item) -> Value {
    let mut info = json!({
        "kind": kind_name(item.key.kind),
        "file": item.key.file,
        "line": item.key.line,
        "signature": item.signature,
    });
    if let Some(map) = info.as_object_mut() {
        if let Some(doc) = &item.doc {
            map.insert("doc".into(), json!(doc));
        }
        if let Some(owner) = &item.owner {
            map.insert("owner".into(), json!(owner));
        }
    }
    info
}

/// Everything derived once per source for enumeration.
struct Index<'a> {
    source: &'a Source,
    /// Names that exactly one item has, with that item.
    unique: BTreeMap<&'a str, usize>,
    /// Name -> functions that call it.
    callers: BTreeMap<&'a str, BTreeSet<usize>>,
}

impl<'a> Index<'a> {
    fn new(source: &'a Source) -> Self {
        let mut unique = BTreeMap::new();
        for (name, count) in source.name_counts() {
            if count == 1
                && let Some(i) = source.unique(name)
            {
                unique.insert(name, i);
            }
        }
        let mut callers: BTreeMap<&str, BTreeSet<usize>> = BTreeMap::new();
        for (f, called) in &source.calls {
            for name in called {
                callers.entry(name.as_str()).or_default().insert(*f);
            }
        }
        Index {
            source,
            unique,
            callers,
        }
    }

    fn unique_of(&self, kinds: &[ItemKind]) -> Vec<(&'a str, usize, &'a Item)> {
        self.unique
            .iter()
            .filter_map(|(name, i)| {
                let item = self.source.item(*i)?;
                kinds.contains(&item.key.kind).then_some((*name, *i, item))
            })
            .collect()
    }

    fn task(&self, kind: &str, name: &str, args: Value, expected: Response, info: Value) -> Task {
        Task {
            id: format!("{}/{kind}/{name}", self.source.name),
            source: self.source.name.clone(),
            kind: kind.to_string(),
            args,
            expected,
            info,
        }
    }
}

/// Every valid candidate task of one kind, in a stable order.
pub fn candidates(source: &Source, kind: &str, seed: u64) -> Vec<Candidate> {
    let index = Index::new(source);
    match kind {
        "callers" | "count_callers" => callers(&index, kind),
        "callees" => callees(&index),
        "locate" => locate(&index),
        "describe" => describe(&index),
        "fields_of" => fields_of(&index),
        "structs_with_field_type" => structs_with_field_type(&index),
        "structs_with_field_named" => structs_with_field_named(&index),
        "rename_field" => rename_field(&index),
        "rename_function" => rename_function(&index),
        "no_such_name" => no_such_name(&index, seed),
        "fields_of_function" => fields_of_function(&index),
        "ambiguous_rename_field" => ambiguous_rename_field(&index),
        _ => Vec::new(),
    }
}

fn tier(n: usize, range: std::ops::RangeInclusive<usize>) -> Tier {
    if range.contains(&n) {
        Tier::Preferred
    } else {
        Tier::Fallback
    }
}

fn callers(ix: &Index, kind: &str) -> Vec<Candidate> {
    let mut out = Vec::new();
    for (name, _, item) in ix.unique_of(&[ItemKind::Function]) {
        let found = ix.callers.get(name).cloned().unwrap_or_default();
        let n = found.len();
        let expected = if kind == "count_callers" {
            Response::Count {
                count: u64::try_from(n).unwrap_or(u64::MAX),
            }
        } else {
            Response::Items {
                items: keys(ix.source, found.iter().copied()),
            }
        };
        let info = json!({
            "target": target_info(item),
            "answer": names(ix.source, found.iter().copied()),
        });
        out.push(Candidate {
            tier: if n == 0 { Tier::Zero } else { tier(n, 1..=15) },
            task: ix.task(kind, name, json!({ "name": name }), expected, info),
        });
    }
    out
}

fn callees(ix: &Index) -> Vec<Candidate> {
    let mut out = Vec::new();
    for (name, i, item) in ix.unique_of(&[ItemKind::Function]) {
        let called = ix.source.calls.get(&i).cloned().unwrap_or_default();
        let found: BTreeSet<usize> = called
            .iter()
            .filter_map(|n| ix.unique.get(n.as_str()).copied())
            .filter(|g| {
                ix.source
                    .item(*g)
                    .is_some_and(|it| it.key.kind == ItemKind::Function)
            })
            .collect();
        let info = json!({
            "target": target_info(item),
            "answer": names(ix.source, found.iter().copied()),
        });
        out.push(Candidate {
            tier: tier(found.len(), 1..=12),
            task: ix.task(
                "callees",
                name,
                json!({ "name": name }),
                Response::Items {
                    items: keys(ix.source, found),
                },
                info,
            ),
        });
    }
    out
}

const ALL_KINDS: [ItemKind; 5] = [
    ItemKind::Function,
    ItemKind::Struct,
    ItemKind::Enum,
    ItemKind::Trait,
    ItemKind::Field,
];

fn locate(ix: &Index) -> Vec<Candidate> {
    ix.unique_of(&ALL_KINDS)
        .into_iter()
        .map(|(name, i, item)| Candidate {
            tier: Tier::Preferred,
            task: ix.task(
                "locate",
                name,
                json!({ "name": name }),
                Response::Items {
                    items: keys(ix.source, [i]),
                },
                json!({ "target": target_info(item) }),
            ),
        })
        .collect()
}

fn describe(ix: &Index) -> Vec<Candidate> {
    ix.unique_of(&[
        ItemKind::Function,
        ItemKind::Struct,
        ItemKind::Enum,
        ItemKind::Trait,
    ])
    .into_iter()
    .map(|(name, _, item)| Candidate {
        tier: Tier::Preferred,
        task: ix.task(
            "describe",
            name,
            json!({ "name": name }),
            Response::Describe {
                item: item.key.clone(),
            },
            json!({ "target": target_info(item) }),
        ),
    })
    .collect()
}

fn fields_of(ix: &Index) -> Vec<Candidate> {
    let mut out = Vec::new();
    for (name, i, item) in ix.unique_of(&[ItemKind::Struct]) {
        let fields = ix.source.struct_fields.get(&i).cloned().unwrap_or_default();
        let info = json!({
            "target": target_info(item),
            "answer": fields
                .iter()
                .filter_map(|f| ix.source.item(*f))
                .map(|f| f.signature.clone())
                .collect::<Vec<_>>(),
        });
        out.push(Candidate {
            tier: tier(fields.len(), 2..=12),
            task: ix.task(
                "fields_of",
                name,
                json!({ "name": name }),
                Response::Items {
                    items: keys(ix.source, fields),
                },
                info,
            ),
        });
    }
    out
}

/// Rust keywords (strict and reserved); never task arguments.
const KEYWORDS: [&str; 51] = [
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

const PRIMITIVES: [&str; 17] = [
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
    "i128", "isize", "f32", "f64",
];

pub fn is_keyword(word: &str) -> bool {
    KEYWORDS.contains(&word)
}

fn structs_with_field_type(ix: &Index) -> Vec<Candidate> {
    // type -> struct -> matching fields
    let mut by_type: BTreeMap<&str, BTreeMap<usize, Vec<usize>>> = BTreeMap::new();
    for (field, idents) in &ix.source.field_type_idents {
        let Some(owner) = ix.source.field_owner.get(field) else {
            continue;
        };
        for ident in idents {
            if !is_keyword(ident) && !ident.starts_with("r#") {
                by_type
                    .entry(ident.as_str())
                    .or_default()
                    .entry(*owner)
                    .or_default()
                    .push(*field);
            }
        }
    }
    by_type
        .into_iter()
        .map(|(ty, structs)| {
            let matching: Vec<String> = structs
                .values()
                .flatten()
                .filter_map(|f| ix.source.item(*f))
                .map(|f| format!("{}.{}", f.owner.as_deref().unwrap_or(""), f.signature))
                .collect();
            let info = json!({
                "answer": names(ix.source, structs.keys().copied()),
                "matching_fields": matching,
            });
            // Lower-case names other than primitives are mostly module
            // paths (`filters::Filter`): valid, but only to fill up.
            let type_like = ty.starts_with(|c: char| c.is_uppercase()) || PRIMITIVES.contains(&ty);
            Candidate {
                tier: if type_like {
                    tier(structs.len(), 1..=10)
                } else {
                    Tier::Fallback
                },
                task: ix.task(
                    "structs_with_field_type",
                    ty,
                    json!({ "type": ty }),
                    Response::Items {
                        items: keys(ix.source, structs.keys().copied()),
                    },
                    info,
                ),
            }
        })
        .collect()
}

/// Field name -> field item indices.
pub fn fields_by_name(source: &Source) -> BTreeMap<&str, Vec<usize>> {
    let mut out: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for f in source.field_owner.keys() {
        if let Some(item) = source.item(*f) {
            out.entry(item.key.name.as_str()).or_default().push(*f);
        }
    }
    out
}

fn structs_with_field_named(ix: &Index) -> Vec<Candidate> {
    fields_by_name(ix.source)
        .into_iter()
        .map(|(field, fields)| {
            let structs: BTreeSet<usize> = fields
                .iter()
                .filter_map(|f| ix.source.field_owner.get(f).copied())
                .collect();
            let info = json!({
                "answer": names(ix.source, structs.iter().copied()),
                "matching_fields": fields
                    .iter()
                    .filter_map(|f| ix.source.item(*f))
                    .map(|f| format!("{}.{}", f.owner.as_deref().unwrap_or(""), f.signature))
                    .collect::<Vec<_>>(),
            });
            Candidate {
                tier: tier(structs.len(), 1..=10),
                task: ix.task(
                    "structs_with_field_named",
                    field,
                    json!({ "field": field }),
                    Response::Items {
                        items: keys(ix.source, structs),
                    },
                    info,
                ),
            }
        })
        .collect()
}

/// The edit renaming every token in `positions`, with `new` left empty for
/// the data pipeline to fill in.
fn edit(source: &Source, old: &str, positions: &BTreeSet<Pos>) -> Response {
    Response::Edit {
        changes: positions
            .iter()
            .map(|(file, line, column)| Change {
                source: source.name.clone(),
                file: source.file_name(*file).to_string(),
                line: *line,
                column: *column,
                old: old.to_string(),
                new: String::new(),
            })
            .collect(),
    }
}

fn files_touched(source: &Source, positions: &BTreeSet<Pos>) -> Vec<String> {
    positions
        .iter()
        .map(|(f, _, _)| source.file_name(*f).to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The positions to rename if renaming field `f` is exact, else `None`.
pub fn field_rename_positions(source: &Source, f: usize) -> Option<BTreeSet<Pos>> {
    let item = source.item(f)?;
    let name = item.key.name.as_str();
    if fields_by_name(source).get(name).map(Vec::len) != Some(1) {
        return None;
    }
    let decl = item.pos(&source.files)?;
    let positions = source.tokens.get(name)?;
    let exact = positions
        .iter()
        .all(|p| *p == decl || source.roles.field_use.contains(p));
    exact.then(|| positions.clone())
}

/// The positions to rename if renaming function `name` is exact, else `None`.
/// A token that is also a shorthand field (`S { name }`) is rejected: it is an
/// expression path, but replacing it would rename the field too.
pub fn function_rename_positions(source: &Source, name: &str) -> Option<BTreeSet<Pos>> {
    let f = source.unique(name)?;
    let item = source.item(f)?;
    if item.key.kind != ItemKind::Function {
        return None;
    }
    let decl = item.pos(&source.files)?;
    let positions = source.tokens.get(name)?;
    let exact = positions.iter().all(|p| {
        *p == decl || (source.roles.fn_use.contains(p) && !source.roles.shorthand.contains(p))
    });
    exact.then(|| positions.clone())
}

fn rename_field(ix: &Index) -> Vec<Candidate> {
    let mut out = Vec::new();
    for (name, fields) in fields_by_name(ix.source) {
        let [f] = fields.as_slice() else { continue };
        let Some(positions) = field_rename_positions(ix.source, *f) else {
            continue;
        };
        let Some(item) = ix.source.item(*f) else {
            continue;
        };
        let owner = item.owner.clone().unwrap_or_default();
        let info = json!({
            "target": target_info(item),
            "changes": positions.len(),
            "files": files_touched(ix.source, &positions),
        });
        out.push(Candidate {
            // Renames that touch more than the declaration are preferred.
            tier: if positions.len() >= 2 {
                Tier::Preferred
            } else {
                Tier::Fallback
            },
            task: ix.task(
                "rename_field",
                &format!("{owner}.{name}"),
                json!({ "struct": owner, "old": name, "new": null }),
                edit(ix.source, name, &positions),
                info,
            ),
        });
    }
    out
}

fn rename_function(ix: &Index) -> Vec<Candidate> {
    let mut out = Vec::new();
    for (name, _, item) in ix.unique_of(&[ItemKind::Function]) {
        let Some(positions) = function_rename_positions(ix.source, name) else {
            continue;
        };
        let info = json!({
            "target": target_info(item),
            "changes": positions.len(),
            "files": files_touched(ix.source, &positions),
        });
        out.push(Candidate {
            // Renames that touch more than the declaration are preferred.
            tier: if positions.len() >= 2 {
                Tier::Preferred
            } else {
                Tier::Fallback
            },
            task: ix.task(
                "rename_function",
                name,
                json!({ "old": name, "new": null }),
                edit(ix.source, name, &positions),
                info,
            ),
        });
    }
    out
}

/// Split `snake_case` into parts, or `CamelCase` into words.
fn parts(name: &str) -> Vec<String> {
    if name.contains('_') || name.chars().all(|c| !c.is_uppercase()) {
        return name
            .split('_')
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect();
    }
    let mut out: Vec<String> = Vec::new();
    for c in name.chars() {
        match out.last_mut() {
            Some(word) if !c.is_uppercase() => word.push(c),
            _ => out.push(c.to_string()),
        }
    }
    out
}

/// Plausible names that are not identifiers anywhere in the source, built
/// from parts of real item names: all but the last part of one name plus a
/// part of another (`spawn_wave` + `boss_hp` -> `spawn_boss`).
fn no_such_name(ix: &Index, seed: u64) -> Vec<Candidate> {
    let seed = seed.to_string();
    // Names to build from (functions, or types) and names to borrow a part
    // from (also fields); long names such as test sentences are left out.
    let mut pools: [(BTreeSet<&str>, BTreeSet<&str>); 2] = Default::default();
    for item in &ix.source.items {
        let name = item.key.name.as_str();
        if name.starts_with("r#") || name.starts_with('_') || parts(name).len() > 3 {
            continue;
        }
        let (pool, base) = match item.key.kind {
            ItemKind::Function => (0, true),
            ItemKind::Field => (0, false),
            ItemKind::Struct | ItemKind::Enum | ItemKind::Trait => (1, true),
        };
        if let Some((bases, partners)) = pools.get_mut(pool) {
            if base {
                bases.insert(name);
            }
            partners.insert(name);
        }
    }
    let [snake, camel] = pools;
    let mut found: BTreeMap<String, (String, String, &str)> = BTreeMap::new();
    for ((bases, partners), sep, questions) in [
        (snake, "_", ["callers", "locate", "describe"]),
        (camel, "", ["fields_of", "locate", "describe"]),
    ] {
        let names: Vec<&str> = partners.iter().copied().collect();
        if names.is_empty() {
            continue;
        }
        let len = u64::try_from(names.len()).unwrap_or(u64::MAX);
        for a in &bases {
            let a_parts = parts(a);
            let Some((_, prefix)) = a_parts.split_last() else {
                continue;
            };
            if prefix.is_empty() || prefix.concat().chars().count() < 3 {
                continue;
            }
            for k in 0..3 {
                let pick = fnv1a(&[&seed, "partner", a, &k.to_string()]) % len;
                let Some(b) = usize::try_from(pick).ok().and_then(|i| names.get(i)) else {
                    continue;
                };
                let b_parts = parts(b);
                let (Some(first), Some(last)) = (b_parts.first(), b_parts.last()) else {
                    continue;
                };
                let take = if k % 2 == 0 { first } else { last };
                let wordlike = take.chars().count() >= 3
                    && take.chars().all(|c| c.is_ascii_alphabetic())
                    && prefix
                        .iter()
                        .all(|p| p.chars().all(|c| c.is_ascii_alphanumeric()));
                if !wordlike || prefix.contains(take) || a_parts.last() == Some(take) {
                    continue;
                }
                let mut words = prefix.to_vec();
                words.push(take.clone());
                let candidate = words.join(sep);
                if ix.source.taken.contains(&candidate)
                    || is_keyword(&candidate)
                    || candidate.chars().next().is_some_and(|c| c.is_ascii_digit())
                {
                    continue;
                }
                let qpick = fnv1a(&[&seed, "question", &candidate]) % 3;
                let question = questions
                    .get(usize::try_from(qpick).unwrap_or(0))
                    .copied()
                    .unwrap_or("locate");
                found
                    .entry(candidate)
                    .or_insert_with(|| ((*a).to_string(), b.to_string(), question));
            }
        }
    }
    found
        .into_iter()
        .map(|(name, (a, b, question))| Candidate {
            tier: Tier::Preferred,
            task: ix.task(
                "no_such_name",
                &name,
                json!({ "name": name, "question": question }),
                Response::Refuse {
                    refusal: Refusal::NoSuchName { name: name.clone() },
                },
                json!({ "built_from": [a, b] }),
            ),
        })
        .collect()
}

fn fields_of_function(ix: &Index) -> Vec<Candidate> {
    ix.unique_of(&[ItemKind::Function])
        .into_iter()
        .map(|(name, _, item)| Candidate {
            tier: Tier::Preferred,
            task: ix.task(
                "fields_of_function",
                name,
                json!({ "name": name }),
                Response::Refuse {
                    refusal: Refusal::WrongKind,
                },
                json!({ "target": target_info(item) }),
            ),
        })
        .collect()
}

fn ambiguous_rename_field(ix: &Index) -> Vec<Candidate> {
    let mut out = Vec::new();
    for (name, fields) in fields_by_name(ix.source) {
        let structs: BTreeSet<usize> = fields
            .iter()
            .filter_map(|f| ix.source.field_owner.get(f).copied())
            .collect();
        if structs.len() < 2 {
            continue;
        }
        let info = json!({
            "answer": fields
                .iter()
                .filter_map(|f| ix.source.item(*f))
                .map(|f| format!("{}.{}", f.owner.as_deref().unwrap_or(""), f.signature))
                .collect::<Vec<_>>(),
        });
        out.push(Candidate {
            tier: tier(structs.len(), 2..=6),
            task: ix.task(
                "ambiguous_rename_field",
                name,
                json!({ "old": name, "new": null }),
                Response::Ask {
                    candidates: keys(ix.source, fields.iter().copied()),
                },
                info,
            ),
        });
    }
    out
}

/// Pick up to `n` candidates: about a tenth from the zero tier, the rest
/// preferred, then fill up from whatever is left; each tier in hash order.
pub fn select(mut candidates: Vec<Candidate>, n: usize, seed: u64) -> Vec<Task> {
    let seed = seed.to_string();
    candidates.sort_by_cached_key(|c| {
        let suffix = c
            .task
            .id
            .splitn(3, '/')
            .nth(2)
            .unwrap_or_default()
            .to_string();
        (fnv1a(&[&seed, &c.task.kind, &suffix]), c.task.id.clone())
    });
    let mut chosen = Vec::new();
    let mut rest = Vec::new();
    let zero_quota = n / 10;
    let mut zero_taken = 0;
    for c in candidates.iter().filter(|c| c.tier == Tier::Zero) {
        if zero_taken < zero_quota {
            chosen.push(c.task.clone());
            zero_taken += 1;
        } else {
            rest.push(c);
        }
    }
    for c in candidates.iter().filter(|c| c.tier == Tier::Preferred) {
        if chosen.len() < n {
            chosen.push(c.task.clone());
        } else {
            rest.push(c);
        }
    }
    rest.extend(candidates.iter().filter(|c| c.tier == Tier::Fallback));
    rest.sort_by_cached_key(|c| {
        (
            fnv1a(&[&seed, &c.task.kind, "fill", &c.task.id]),
            c.task.id.clone(),
        )
    });
    for c in rest {
        if chosen.len() >= n {
            break;
        }
        chosen.push(c.task.clone());
    }
    chosen.truncate(n);
    chosen
}
