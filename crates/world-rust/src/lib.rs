//! The Rust source code world.
//!
//! Its things are the items of one or more Rust sources (functions, structs,
//! enums, traits, fields), indexed with `syn`. It defines their kinds and
//! relations, how to look them up by name, and how to answer meanings about
//! them. It knows no human language.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use glassmind_meaning::interface::World;
use glassmind_meaning::{
    Condition, Description, Extent, Labelled, Name, Ontology, Position, Question, Reply, Thing,
    Trouble,
};
use glassmind_rules::{RuleInfo, SEED, Switches};
use serde::{Deserialize, Serialize};

pub mod index;

use index::SourceIndex;

/// The Rust world's concepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rust;

impl Ontology for Rust {
    type Kind = RustKind;
    type Relation = RustRelation;
    type Item = RustItem;
}

/// Kinds of Rust items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RustKind {
    /// A free function, associated function or method.
    Function,
    Struct,
    Enum,
    Trait,
    /// A named field of a struct.
    Field,
}

/// Relations between Rust items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RustRelation {
    /// `Calls(f, g)`: function `f`'s body calls `g` by name.
    Calls,
}

/// One Rust item, identified by where its name is written.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RustItem {
    pub source: String,
    pub file: String,
    pub line: u32,
    pub name: String,
    pub kind: RustKind,
    /// The type or trait a function belongs to, or the struct a field belongs to.
    pub owner: Option<String>,
}

impl Labelled for RustItem {
    fn label(&self) -> String {
        match &self.owner {
            Some(owner) if !owner.is_empty() => format!("{owner}::{}", self.name),
            _ => self.name.clone(),
        }
    }

    fn place(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }
}

/// Looking a name up: exact matches of the name or `Owner::name`.
const LOOKUP_EXACT: RuleInfo = RuleInfo::new("rust.lookup.exact", SEED);
/// Looking a name up: when nothing matches exactly, matches ignoring case.
const LOOKUP_ANY_CASE: RuleInfo = RuleInfo::new("rust.lookup.any_case", SEED);
/// The calls relation, read from call expressions in function bodies.
const RELATION_CALLS: RuleInfo = RuleInfo::new("rust.relation.calls", SEED);
/// Where an item is: where its name is written, said only when no other
/// item of the kind asked for has that name, so the user can't have meant
/// another one.
const LOCATE_UNIQUE: RuleInfo = RuleInfo::new("rust.locate.unique", 2);

/// Every world rule, for listing and switching.
pub const RULES: &[RuleInfo] = &[LOOKUP_EXACT, LOOKUP_ANY_CASE, RELATION_CALLS, LOCATE_UNIQUE];

/// One or more indexed Rust sources.
pub struct RustWorld {
    pub sources: Vec<Arc<SourceIndex>>,
    switches: Switches,
}

impl RustWorld {
    /// Indexes each `(name, root)` source.
    pub fn open(sources: &[(String, &Path)], switches: &Switches) -> Self {
        let indices = sources
            .iter()
            .map(|(name, root)| Arc::new(index::index_source(name, root)))
            .collect();
        RustWorld::from_indices(indices, switches)
    }

    /// A world over sources already indexed, which several worlds can share.
    pub fn from_indices(sources: Vec<Arc<SourceIndex>>, switches: &Switches) -> Self {
        RustWorld {
            sources,
            switches: switches.clone(),
        }
    }

    fn records(&self) -> impl Iterator<Item = &index::Record> {
        self.sources.iter().flat_map(|s| s.records.iter())
    }

    /// Every item of the kind, or every item if no kind is given.
    fn all_of(&self, kind: Option<RustKind>) -> Vec<RustItem> {
        self.records()
            .filter(|r| kind.is_none_or(|k| r.item.kind == k))
            .map(|r| r.item.clone())
            .collect()
    }

    /// Functions in the same source with this name.
    fn functions_named<'a>(
        &'a self,
        source: &'a str,
        name: &'a str,
    ) -> impl Iterator<Item = &'a RustItem> {
        self.records()
            .map(|r| &r.item)
            .filter(move |i| i.source == source && i.kind == RustKind::Function && i.name == name)
    }

    /// Every item that fills `position` of `relation` when `other` fills the
    /// other argument. `Err` if the world cannot decide it.
    fn related(
        &self,
        relation: RustRelation,
        position: Position,
        other: &RustItem,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<BTreeSet<RustItem>, Trouble<Rust>> {
        match relation {
            RustRelation::Calls => {
                if !self.switches.on(RELATION_CALLS.id) {
                    return Err(Trouble::Unsupported);
                }
                fired.push(RELATION_CALLS);
                if other.kind != RustKind::Function {
                    return Err(Trouble::WrongKind {
                        item: other.clone(),
                    });
                }
                let Some(source) = self.sources.iter().find(|s| s.name == other.source) else {
                    return Ok(BTreeSet::new());
                };
                let item_at = |i: usize| source.records.get(i).map(|r| r.item.clone());
                match position {
                    // Callers of `other`. Calls are found by name, so they only
                    // identify `other` when its name is unique among functions.
                    Position::First => {
                        if self.functions_named(&other.source, &other.name).count() > 1 {
                            return Err(Trouble::Unsupported);
                        }
                        Ok(source
                            .calls
                            .iter()
                            .filter(|c| c.name == other.name)
                            .filter_map(|c| item_at(c.caller))
                            .collect())
                    }
                    // Functions `other` calls, among those with a unique name.
                    Position::Second => {
                        let names: BTreeSet<&str> = source
                            .calls
                            .iter()
                            .filter(|c| item_at(c.caller).as_ref() == Some(other))
                            .map(|c| c.name.as_str())
                            .collect();
                        let mut out = BTreeSet::new();
                        for name in names {
                            let mut named = self.functions_named(&other.source, name);
                            if let (Some(only), None) = (named.next(), named.next()) {
                                out.insert(only.clone());
                            }
                        }
                        Ok(out)
                    }
                }
            }
        }
    }

    fn identify(
        &self,
        description: &Description<Rust>,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<Vec<RustItem>, Trouble<Rust>> {
        let mut candidates: Option<BTreeSet<RustItem>> = None;
        for condition in &description.conditions {
            let fitting = match condition {
                Condition::Related {
                    relation,
                    position,
                    other: Thing::Item(other),
                } => self.related(*relation, *position, other, fired)?,
                Condition::Related {
                    other: Thing::Named(name),
                    ..
                } => return Err(Trouble::NoSuchName { name: name.clone() }),
            };
            candidates = Some(match candidates {
                None => fitting,
                Some(c) => c.intersection(&fitting).cloned().collect(),
            });
        }
        let candidates =
            candidates.unwrap_or_else(|| self.all_of(description.kind).into_iter().collect());
        Ok(candidates
            .into_iter()
            .filter(|i| description.kind.is_none_or(|k| i.kind == k))
            .collect())
    }
}

impl World<Rust> for RustWorld {
    fn lookup(&self, name: &Name, fired: &mut Vec<RuleInfo>) -> Vec<RustItem> {
        let wanted = name.0.as_str();
        let matches = |item: &RustItem, fold: bool| {
            let eq = |a: &str, b: &str| {
                if fold {
                    a.eq_ignore_ascii_case(b)
                } else {
                    a == b
                }
            };
            eq(&item.name, wanted) || eq(&item.label(), wanted)
        };
        let mut found: Vec<RustItem> = Vec::new();
        if self.switches.on(LOOKUP_EXACT.id) {
            found = self
                .records()
                .filter(|r| matches(&r.item, false))
                .map(|r| r.item.clone())
                .collect();
            fired.push(LOOKUP_EXACT);
        }
        if found.is_empty() && self.switches.on(LOOKUP_ANY_CASE.id) {
            found = self
                .records()
                .filter(|r| matches(&r.item, true))
                .map(|r| r.item.clone())
                .collect();
            fired.push(LOOKUP_ANY_CASE);
        }
        found.sort();
        found.dedup();
        found
    }

    fn answer(
        &self,
        question: &Question<Rust>,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<Reply<Rust>, Trouble<Rust>> {
        match question {
            Question::Identify(description) => {
                let members = self.identify(description, fired)?;
                Ok(Reply::Extent(Extent {
                    description: description.clone(),
                    members,
                }))
            }
            Question::Locate {
                thing: Thing::Item(item),
                kind,
            } => {
                if !self.switches.on(LOCATE_UNIQUE.id) {
                    return Err(Trouble::Unsupported);
                }
                fired.push(LOCATE_UNIQUE);
                if kind.is_some_and(|k| item.kind != k) {
                    return Err(Trouble::WrongKind { item: item.clone() });
                }
                let candidates: Vec<RustItem> = self
                    .records()
                    .filter(|r| r.item.name == item.name && kind.is_none_or(|k| r.item.kind == k))
                    .map(|r| r.item.clone())
                    .collect();
                if candidates.len() > 1 {
                    return Err(Trouble::Ambiguous { candidates });
                }
                Ok(Reply::Location(item.clone()))
            }
            Question::Locate {
                thing: Thing::Named(name),
                ..
            } => Err(Trouble::NoSuchName { name: name.clone() }),
        }
    }

    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.records().map(|r| r.item.name.clone()).collect();
        names.sort();
        names.dedup();
        names
    }
}
