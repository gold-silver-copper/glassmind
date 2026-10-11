//! The Rust source code world.
//!
//! Its things are the items of one or more Rust sources (functions, structs,
//! enums, traits, fields), indexed with `syn`. It defines their kinds and
//! relations, how to look them up by name, and how to answer meanings about
//! them. It knows no human language.
// A world's errors are rare and small in number; their size does not matter.
#![allow(clippy::result_large_err)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use glassmind_meaning::Trouble::{Ambiguous, NoSuchName, Unsupported, WrongKind};
use glassmind_meaning::interface::World;
use glassmind_meaning::{
    Action, Condition, Description, Extent, Fact, Labelled, Name, Ontology, Position, Question,
    Reply, Thing, Trouble,
};
use glassmind_rules::{HAND, RuleInfo, SEED, Switches};
use serde::{Deserialize, Serialize};

pub mod add_fields;
mod argument_swap;
mod ceiling_division;
pub mod checks;
pub mod declared_types;
pub mod index;
mod index_conversion;
mod rename_fields;
mod rename_functions;
mod rename_package;
mod rename_types;
mod rename_variants;
mod slips;
mod standard_traits;

use RustKind::{Field, Function};
use checks::{RustChange, RustCheck};
use index::SourceIndex;

/// The Rust world's concepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rust;

impl Ontology for Rust {
    type Kind = RustKind;
    type Relation = RustRelation;
    type Item = RustItem;
    type Spot = RustSpot;
    type Check = RustCheck;
    type Change = RustChange;
}

/// Kinds of Rust items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RustKind {
    /// A free function, associated function or method.
    Function,
    Struct,
    Enum,
    /// A unit, tuple or record variant declared by an enum.
    Variant,
    Trait,
    /// A named field of a struct.
    Field,
    /// A problem a tool reported: a compiler error or a lint warning (its
    /// code is the name, its message the detail).
    Problem,
    /// A test that failed.
    Test,
}

/// Relations between Rust items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RustRelation {
    /// `Calls(f, g)`: function `f`'s body calls `g` by name.
    Calls,
    /// `HasField(s, f)`: struct `s` declares field `f`.
    HasField,
    /// `Member(t, m)`: `m` belongs to the struct, enum or trait `t`: one
    /// of its fields, or a function in its `impl` or trait body.
    Member,
    /// `TypeMentions(f, t)`: field `f`'s type has the identifier `t` in it
    /// (`Vec<Velocity>` mentions `Vec` and `Velocity`). `t` need not be an
    /// item of the sources: `Arc` and `u32` are types too.
    TypeMentions,
    /// `BelongsTo(m, o)`: `m` is declared in `o`: a field of struct `o`, or
    /// a function in trait `o` or in an `impl` of type `o`.
    BelongsTo,
}

/// One Rust item, identified by where its name is written.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RustItem {
    pub source: String,
    pub file: String,
    pub line: u32,
    pub name: String,
    pub kind: RustKind,
    /// The type/trait owning a function, struct owning a field, or enum owning a variant.
    pub owner: Option<String>,
    /// What a tool said about a problem or a failing test, as it said it.
    #[serde(default)]
    pub detail: Option<String>,
}

/// Where an identifier is written: a 1-based line and a 0-based column, in
/// characters, of a file in a source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RustSpot {
    pub source: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl Labelled for RustItem {
    fn label(&self) -> String {
        match &self.owner {
            Some(owner) if !owner.is_empty() => format!("{owner}::{}", self.name),
            _ => self.name.clone(),
        }
    }

    fn place(&self) -> String {
        if self.file.is_empty() {
            self.name.clone()
        } else {
            format!("{}:{}", self.file, self.line)
        }
    }
}

/// Looking a name up: exact matches of the name, `Owner::name` or
/// `Owner.name`.
const LOOKUP_EXACT: RuleInfo = RuleInfo::new("rust.lookup.exact", SEED);
/// Looking a name up: when nothing matches exactly, matches ignoring case,
/// unless the name is written exactly as a type somewhere (`Arc` is not
/// the function `arc`).
const LOOKUP_ANY_CASE: RuleInfo = RuleInfo::new("rust.lookup.any_case", SEED);
/// The calls relation, read from call expressions in function bodies.
const RELATION_CALLS: RuleInfo = RuleInfo::new("rust.relation.calls", SEED);
/// Where an item is: where its name is written, said only when no other
/// item of the kind asked for has that name, so the user can't have meant
/// another one.
const LOCATE_UNIQUE: RuleInfo = RuleInfo::new("rust.locate.unique", 2);
/// What an item is: its kind and where it is, said only for a function,
/// struct, enum or trait whose name no other item shares (or no other item
/// of the kind the user said it is).
const DESCRIBE_UNIQUE: RuleInfo = RuleInfo::new("rust.describe.unique", 3);
/// The has-field relation, read from struct declarations.
const RELATION_HAS_FIELD: RuleInfo = RuleInfo::new("rust.relation.has_field", 3);
/// The member relation, read from the struct a field is declared in and
/// the type or trait a function's `impl` or trait body is for, by name.
const RELATION_MEMBER: RuleInfo = RuleInfo::new("rust.relation.member", 5);
/// The type-mentions relation, read from the identifiers in field types.
const RELATION_TYPE_MENTIONS: RuleInfo = RuleInfo::new("rust.relation.type_mentions", 3);
/// Things going by a name: the items whose name is exactly the one given,
/// of the kind described. A field's name is how it is told apart from the
/// other fields of its struct, so "a field named `done`" may be several.
const CONDITION_NAMED: RuleInfo = RuleInfo::new("rust.condition.named", 5);
/// A part of a thing, no relation said ("`S`'s `x`", "`S` `x`"): one of
/// the members of the struct, enum or trait, as [`RELATION_MEMBER`] reads
/// them.
const CONDITION_PART_OF: RuleInfo = RuleInfo::new("rust.condition.part_of", 21);

/// The belongs-to relation, read from where functions and fields are declared.
const RELATION_BELONGS_TO: RuleInfo = RuleInfo::new("rust.relation.belongs_to", 5);
/// Renaming a function or a field: only when no other item of its kind in
/// its source has its name, the new name is a fresh identifier, and every
/// identifier token with the old name names an item of that kind by syntax
/// alone (for a function its declaration, a call or other expression path,
/// a method call, a `use`; for a field its declaration, `expr.f`, `S { f: v
/// }`, `S { f: p }`), so no type information is needed to know the edit is
/// exact.
const RENAME: RuleInfo = RuleInfo::new("rust.rename", 5);
/// Renaming a field, proposed only when it is exact without type
/// information: no other field in its source has its name, every
/// identifier token with its name is written as that field (declared,
/// `x.f`, `S { f: v }`, `S { f: p }`; not `S { f }` or inside a macro),
/// and the new name is written nowhere in the source. Decides before
/// [`RENAME`] for anything that is a field.
const RENAME_FIELD: RuleInfo = RuleInfo::new("rust.rename.field", 5);
/// Rename a struct or enum only when explicit module/import bindings account
/// for every occurrence; refuse unsupported or potentially shadowed syntax.
const RENAME_TYPE: RuleInfo = RuleInfo::new("rust.rename.type", HAND);
/// Rename a variant only with explicit enum and reference bindings.
const RENAME_VARIANT: RuleInfo = RuleInfo::new("rust.rename.variant", HAND);
/// Noting a name the user will call a thing by: only when the thing is the
/// only item of its kind (or of the kind the user said) with its name, so
/// later uses of the alias can't mean another one.
const ALIAS_UNIQUE: RuleInfo = RuleInfo::new("rust.alias.unique", 6);
/// A name said to be of a kind no item with that name is ("the `draw`
/// struct" when `draw` is a function): the thing meant is of another kind,
/// not missing, so asking for its fields is asking the wrong kind of thing.
const NAMED_OTHER_KIND: RuleInfo = RuleInfo::new("rust.condition.named.other_kind", 7);

/// Every world rule, for listing and switching.
/// Checking a source with its own tools (cargo check, test, clippy).
const RUN_CHECK: RuleInfo = RuleInfo::new("rust.check.run", HAND);
/// A remedy: the compiler's suggested fixes, kept only if the build then
/// passes in a scratch copy.
const REMEDY_SUGGESTED: RuleInfo = RuleInfo::new("rust.remedy.compiler_suggestions", HAND);
/// The compiler's suggestions for one place are alternatives ("use
/// `crate::fmt`", "use `core::fmt`", ...): a remedy makes only the first
/// it offers there, as no two replacements of the same text can both be
/// made.
const REMEDY_ONE_PER_PLACE: RuleInfo = RuleInfo::new("rust.remedy.one_per_place", 29);
/// Convert a signed slice index only after an AST range and native width proof.
const REMEDY_INDEX: RuleInfo = RuleInfo::new("rust.remedy.signed_index", HAND);
/// Add standard Debug only to a unique compiler-identified local declaration.
const REMEDY_DEBUG: RuleInfo = RuleInfo::new("rust.remedy.derive_debug", HAND);
/// Compiler fixes can expose further errors; compose bounded rounds only
/// when the final requested check passes.
const REMEDY_ITERATE: RuleInfo = RuleInfo::new("rust.remedy.iterate_suggestions", HAND);
/// A remedy for failing tests: undoing one slip of an operator or a truth
/// value in the source's own code (`checks::slips`), kept only if every
/// test then passes in a scratch copy.
const REMEDY_SLIP: RuleInfo = RuleInfo::new("rust.remedy.undo_slip", 29);
/// Applying changes the user accepted, then checking again.
const APPLY_CHANGES: RuleInfo = RuleInfo::new("rust.apply.changes", HAND);

pub const RULES: &[RuleInfo] = &[
    declared_types::DECLARED_TYPE,
    REMEDY_DEBUG,
    REMEDY_INDEX,
    REMEDY_ITERATE,
    RUN_CHECK,
    REMEDY_SUGGESTED,
    REMEDY_ONE_PER_PLACE,
    REMEDY_SLIP,
    APPLY_CHANGES,
    LOOKUP_EXACT,
    LOOKUP_ANY_CASE,
    RELATION_CALLS,
    LOCATE_UNIQUE,
    DESCRIBE_UNIQUE,
    RELATION_HAS_FIELD,
    RELATION_TYPE_MENTIONS,
    RELATION_BELONGS_TO,
    RENAME,
    RENAME_FIELD,
    RENAME_TYPE,
    RENAME_VARIANT,
    RELATION_MEMBER,
    CONDITION_NAMED,
    ALIAS_UNIQUE,
    NAMED_OTHER_KIND,
    CONDITION_PART_OF,
];

/// What the world finds: a value, or the trouble that stopped it.
type Found<T> = Result<T, Trouble<Rust>>;

/// One or more indexed Rust sources.
pub struct RustWorld {
    pub sources: Vec<Arc<SourceIndex>>,
    switches: Switches,
    /// What each check last found, per source, until a change is applied.
    found: std::sync::Mutex<BTreeMap<(String, RustCheck), Vec<checks::Finding>>>,
}

/// The item a thing stands for once its name is resolved: "no such name" if
/// it is still a bare name; a description or "it" is not one item.
fn resolved(thing: &Thing<Rust>) -> Found<&RustItem> {
    match thing {
        Thing::Item(item) => Ok(item),
        Thing::Named(name) => Err(NoSuchName { name: name.clone() }),
        Thing::Fitting(_) | Thing::Previous => Err(Unsupported),
    }
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
            found: std::sync::Mutex::new(BTreeMap::new()),
        }
    }

    /// What a check finds in every source, run once until a change is applied.
    fn findings(&self, check: RustCheck, fired: &mut Vec<RuleInfo>) -> Found<Vec<checks::Finding>> {
        if !self.switches.on(RUN_CHECK.id) {
            return Err(Unsupported);
        }
        fired.push(RUN_CHECK);
        let mut all = Vec::new();
        for source in &self.sources {
            let key = (source.name.clone(), check);
            let cached = self.found.lock().ok().and_then(|f| f.get(&key).cloned());
            let found = match cached {
                Some(f) => f,
                None => {
                    let f =
                        checks::run(check, &source.name, &source.root).map_err(|_| Unsupported)?;
                    if let Ok(mut cache) = self.found.lock() {
                        cache.insert(key, f.clone());
                    }
                    f
                }
            };
            all.extend(found);
        }
        Ok(all)
    }

    fn records(&self) -> impl Iterator<Item = &index::Record> {
        self.sources.iter().flat_map(|s| s.records.iter())
    }

    /// The items for which `keep` holds, in index order.
    fn items<C: FromIterator<RustItem>>(&self, keep: impl Fn(&RustItem) -> bool) -> C {
        self.records()
            .map(|r| &r.item)
            .filter(|i| keep(i))
            .cloned()
            .collect()
    }

    /// Every item with this name, of the kind if one is given.
    fn sharing(&self, name: &str, kind: Option<RustKind>) -> Vec<RustItem> {
        self.items(|i| i.name == name && kind.is_none_or(|k| i.kind == k))
    }

    /// Notes that `rule` decides what follows, if it is switched on;
    /// `Unsupported` if it is off.
    fn fire(&self, rule: RuleInfo, fired: &mut Vec<RuleInfo>) -> Found<()> {
        if !self.switches.on(rule.id) {
            return Err(Unsupported);
        }
        fired.push(rule);
        Ok(())
    }

    /// Every item that fills `position` of `relation` when one of `others`
    /// fills the other argument. `Err` if the world cannot decide it.
    fn related(
        &self,
        relation: RustRelation,
        position: Position,
        others: &BTreeSet<RustItem>,
        fired: &mut Vec<RuleInfo>,
    ) -> Found<BTreeSet<RustItem>> {
        use Position::*;
        use RustKind::*;
        use RustRelation::*;
        let rule = match relation {
            Calls => RELATION_CALLS,
            HasField => RELATION_HAS_FIELD,
            Member => RELATION_MEMBER,
            TypeMentions => RELATION_TYPE_MENTIONS,
            BelongsTo => RELATION_BELONGS_TO,
        };
        self.fire(rule, fired)?;
        // The kinds `others` may be of.
        let wanted: &[RustKind] = match (relation, position) {
            (Calls, _) => &[Function],
            (HasField, First) => &[Field],
            (HasField, Second) => &[Struct],
            (Member, First) => &[Field, Function, Variant],
            (TypeMentions | BelongsTo | Member, _) => &[Struct, Enum, Trait],
        };
        if let Some(item) = others.iter().find(|o| !wanted.contains(&o.kind)) {
            return Err(WrongKind { item: item.clone() });
        }
        let mut out = BTreeSet::new();
        match (relation, position) {
            (Calls, _) => {
                for other in others {
                    out.extend(self.calls(position, other)?);
                }
            }
            // A struct's fields are indexed right after it.
            (HasField, _) => {
                for source in &self.sources {
                    let mut owner = None;
                    for record in &source.records {
                        let item = &record.item;
                        match (item.kind, owner) {
                            (Struct, _) => owner = Some(item),
                            (Field, Some(owner)) => {
                                let (this, that) = match position {
                                    First => (owner, item),
                                    Second => (item, owner),
                                };
                                if others.contains(that) {
                                    out.insert(this.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            // `BelongsTo(m, o)` is `Member(o, m)`.
            (Member, _) | (BelongsTo, First) => {
                for item in self.records().map(|r| &r.item) {
                    for other in others {
                        let (whole, part) = if (relation, position) == (Member, First) {
                            (item, other)
                        } else {
                            (other, item)
                        };
                        if part.source == whole.source
                            && part.owner.as_ref() == Some(&whole.name)
                            && !wanted.contains(&item.kind)
                        {
                            if part.kind == Variant {
                                let owner = self.variant_owner(part).ok_or(Unsupported)?;
                                if owner != whole {
                                    continue;
                                }
                            }
                            out.insert(item.clone());
                        }
                    }
                }
            }
            (TypeMentions, First) => {
                out = self.typed(&others.iter().map(|o| o.name.as_str()).collect());
            }
            (TypeMentions | BelongsTo, Second) => return Err(Unsupported),
        }
        Ok(out)
    }

    /// Variants are indexed immediately after their declaring enum. Use the
    /// declaration identity, not just its name, for the ownership relation.
    fn variant_owner(&self, part: &RustItem) -> Option<&RustItem> {
        let source = self.sources.iter().find(|s| s.name == part.source)?;
        let at = source.records.iter().position(|r| &r.item == part)?;
        let owner = source
            .records
            .get(..at)?
            .iter()
            .rev()
            .find(|r| r.item.kind != RustKind::Variant)
            .map(|r| &r.item)
            .filter(|i| i.kind == RustKind::Enum)?;
        // RustItem currently identifies a declaration by file and line. Two
        // same-named enums on one line must not acquire each other's variants.
        if source.records.iter().filter(|r| &r.item == owner).count() != 1 {
            return None;
        }
        Some(owner)
    }

    /// Fields whose type mentions one of `names`.
    fn typed(&self, names: &BTreeSet<&str>) -> BTreeSet<RustItem> {
        self.records()
            .filter(|r| r.type_idents.iter().any(|t| names.contains(t.as_str())))
            .map(|r| r.item.clone())
            .collect()
    }

    /// The functions calling `other` (`First`), or that it calls (`Second`).
    fn calls(&self, position: Position, other: &RustItem) -> Found<BTreeSet<RustItem>> {
        let Some(source) = self.sources.iter().find(|s| s.name == other.source) else {
            return Ok(BTreeSet::new());
        };
        let item_at = |i: usize| source.records.get(i).map(|r| r.item.clone());
        match position {
            // Callers of `other`. Calls are found by name, so they only
            // identify `other` when its name is unique among the functions
            // of its source.
            Position::First => {
                let namesakes = source.records.iter().filter(|r| r.item.name == other.name);
                if namesakes.count() > 1 {
                    return Err(Unsupported);
                }
                Ok(source
                    .calls
                    .iter()
                    .filter(|c| c.name == other.name)
                    .filter_map(|c| item_at(c.caller))
                    .collect())
            }
            // Functions `other` calls, among those whose name no
            // other item of any kind has.
            Position::Second => Ok(source
                .calls
                .iter()
                .filter(|c| item_at(c.caller).as_ref() == Some(other))
                .filter_map(|c| {
                    let named: Vec<&RustItem> = source
                        .records
                        .iter()
                        .map(|r| &r.item)
                        .filter(|i| i.name == c.name)
                        .collect();
                    match named.as_slice() {
                        [only] if only.kind == Function => Some((*only).clone()),
                        _ => None,
                    }
                })
                .collect()),
        }
    }

    /// The thing asked about, checked by `rule`: of the kind the user said
    /// it is, if they said one, and the only item of that kind with its
    /// name, or the only one fitting what the user said of it ("`x` on
    /// `App`", "the struct named `App`"), so the user can't have meant
    /// another one.
    fn only(
        &self,
        rule: RuleInfo,
        thing: &Thing<Rust>,
        kind: Option<RustKind>,
        fired: &mut Vec<RuleInfo>,
    ) -> Found<RustItem> {
        self.fire(rule, fired)?;
        if let Thing::Fitting(d) = thing {
            let mut fitting = self.identify(d, fired)?;
            return match (fitting.pop(), fitting.is_empty()) {
                (Some(item), true) if kind.is_some_and(|k| item.kind != k) => {
                    Err(WrongKind { item })
                }
                (Some(item), true) => Ok(item),
                (Some(item), false) => {
                    fitting.push(item);
                    Err(Ambiguous {
                        candidates: self.items(|i| fitting.contains(i)),
                    })
                }
                (None, _) => Err(Unsupported),
            };
        }
        let item = resolved(thing)?;
        if kind.is_some_and(|k| item.kind != k) {
            return Err(WrongKind { item: item.clone() });
        }
        let candidates = self.sharing(&item.name, kind);
        if candidates.len() > 1 {
            return Err(Ambiguous { candidates });
        }
        Ok(item.clone())
    }

    /// The items a resolved thing stands for.
    fn things(&self, thing: &Thing<Rust>, fired: &mut Vec<RuleInfo>) -> Found<BTreeSet<RustItem>> {
        match thing {
            Thing::Fitting(d) => Ok(self.identify(d, fired)?.into_iter().collect()),
            _ => Ok([resolved(thing)?.clone()].into()),
        }
    }

    /// Renaming a field, by [`RENAME_FIELD`]: every spot its name is
    /// written. `None` if the thing is not a field, for [`RENAME`] to
    /// decide.
    fn rename_field(
        &self,
        thing: &Thing<Rust>,
        kind: Option<RustKind>,
        name: &Name,
        fired: &mut Vec<RuleInfo>,
    ) -> Option<Found<Reply<Rust>>> {
        if !self.switches.on(RENAME_FIELD.id) || kind.is_some_and(|k| k != Field) {
            return None;
        }
        // A field said to be of a struct is the one so described; a name
        // alone that several fields have asks which, whatever else it names.
        let item = match thing {
            Thing::Fitting(d) => match self.identify(d, fired) {
                Ok(items) => items.first().filter(|_| items.len() == 1)?.clone(),
                Err(trouble) => return Some(Err(trouble)),
            },
            Thing::Item(item) => {
                let candidates = self.sharing(&item.name, Some(Field));
                if candidates.len() > 1 {
                    fired.push(RENAME_FIELD);
                    return Some(Err(Ambiguous { candidates }));
                }
                item.clone()
            }
            Thing::Named(_) | Thing::Previous => return None,
        };
        if item.kind != Field {
            return None;
        }
        fired.push(RENAME_FIELD);
        // Preserve the old unique-spelling path. Homonymous fields require an
        // explicit owner description and a complete owner-aware position audit.
        if self.sharing(&item.name, Some(Field)).len() > 1 {
            let explicit_owner = if let Thing::Fitting(d) = thing {
                d.conditions
                    .iter()
                    .filter_map(|condition| match condition {
                        Condition::PartOf(owner)
                        | Condition::Related {
                            relation: RustRelation::Member | RustRelation::HasField,
                            position: Position::Second,
                            other: owner,
                        }
                        | Condition::Related {
                            relation: RustRelation::BelongsTo,
                            position: Position::First,
                            other: owner,
                        } => Some(owner),
                        _ => None,
                    })
                    .any(|owner| {
                        self.things(owner, &mut Vec::new()).is_ok_and(|owners| {
                            owners.len() == 1
                                && owners.first().is_some_and(|owner| {
                                    owner.kind == RustKind::Struct
                                        && owner.source == item.source
                                        && item.owner.as_deref() == Some(owner.name.as_str())
                                })
                        })
                    })
            } else {
                false
            };
            if !explicit_owner || self.sources.len() != 1 {
                return Some(Err(Unsupported));
            }
            let result = self
                .sources
                .iter()
                .find(|s| s.name == item.source)
                .and_then(|source| {
                    rename_fields::positions(source, &item, &name.0).map(|at| spots(source, &at))
                })
                .ok_or(Unsupported)
                .map(|spots| Reply::Proposal {
                    action: Action::Rename {
                        thing: Thing::Item(item),
                        kind,
                        name: Some(name.clone()),
                    },
                    spots,
                });
            return Some(result);
        }
        Some(self.rename_at(item, kind, name, false))
    }

    /// Renaming `item` to `name` at every spot its name is written in its
    /// source: only when no other item of its kind has its name, every one
    /// of those spots is written as an item of its kind (by syntax alone),
    /// and the new name is written nowhere in the source (nor `taken`
    /// otherwise), so the edit is exact.
    fn rename_at(
        &self,
        item: RustItem,
        kind: Option<RustKind>,
        name: &Name,
        taken: bool,
    ) -> Found<Reply<Rust>> {
        let source = self
            .sources
            .iter()
            .find(|s| s.name == item.source)
            .ok_or(Unsupported)?;
        let at = source.idents.get(&item.name).ok_or(Unsupported)?;
        if taken
            || source.idents.contains_key(&name.0)
            || self.sharing(&item.name, Some(item.kind)).len() > 1
            || at.iter().any(|p| source.names.get(p) != Some(&item.kind))
        {
            if !taken && item.kind == RustKind::Function && self.sources.len() == 1 {
                let proven =
                    rename_functions::positions(source, &item, &name.0).ok_or(Unsupported)?;
                return Ok(Reply::Proposal {
                    action: Action::Rename {
                        thing: Thing::Item(item),
                        kind,
                        name: Some(name.clone()),
                    },
                    spots: spots(source, &proven),
                });
            }
            return Err(Unsupported);
        }
        Ok(Reply::Proposal {
            action: Action::Rename {
                thing: Thing::Item(item),
                kind,
                name: Some(name.clone()),
            },
            spots: spots(source, at),
        })
    }

    /// The items of the kind (any kind if `None`) whose name is exactly
    /// `name`; "no such name" if there are none.
    fn named(
        &self,
        name: &Name,
        kind: Option<RustKind>,
        fired: &mut Vec<RuleInfo>,
    ) -> Found<BTreeSet<RustItem>> {
        self.fire(CONDITION_NAMED, fired)?;
        let (named, other): (BTreeSet<RustItem>, BTreeSet<RustItem>) = self
            .sharing(&name.0, None)
            .into_iter()
            .partition(|i| kind.is_none_or(|k| i.kind == k));
        match other.into_iter().next() {
            Some(item) if named.is_empty() && self.fire(NAMED_OTHER_KIND, fired).is_ok() => {
                Err(WrongKind { item })
            }
            _ if named.is_empty() => Err(NoSuchName { name: name.clone() }),
            _ => Ok(named),
        }
    }

    fn identify(
        &self,
        description: &Description<Rust>,
        fired: &mut Vec<RuleInfo>,
    ) -> Found<Vec<RustItem>> {
        let mut candidates: Option<BTreeSet<RustItem>> = None;
        for condition in &description.conditions {
            let fitting = match condition {
                // Called a name nothing has: not "none of them", but no such
                // name ("an attribute with that name" read as "a thing
                // called `attribute`").
                Condition::Called(name) => {
                    let called: BTreeSet<RustItem> = self.lookup(name, fired).into_iter().collect();
                    if called.is_empty() {
                        return Err(NoSuchName { name: name.clone() });
                    }
                    called
                }
                Condition::Is(thing) => self.things(thing, fired)?,
                Condition::Named(name) => self.named(name, description.kind, fired)?,
                // Named like an item: by the item's own name.
                Condition::NamedLike(Thing::Item(item)) => {
                    self.named(&Name(item.name.clone()), description.kind, fired)?
                }
                Condition::NamedLike(_) => return Err(Unsupported),
                Condition::PartOf(whole) => {
                    self.fire(CONDITION_PART_OF, fired)?;
                    let wholes = self.things(whole, fired)?;
                    self.related(RustRelation::Member, Position::Second, &wholes, fired)?
                }
                // A type needs no item to be named (`Arc`, `u32`), only to be
                // written as one somewhere.
                Condition::Related {
                    relation: RustRelation::TypeMentions,
                    position: Position::First,
                    other: Thing::Named(name),
                } if self.fire(RELATION_TYPE_MENTIONS, fired).is_ok() => {
                    let fields = self.typed(&[name.0.as_str()].into());
                    if fields.is_empty() {
                        return Err(NoSuchName { name: name.clone() });
                    }
                    fields
                }
                Condition::Related {
                    relation,
                    position,
                    other,
                } => {
                    let others = self.things(other, fired)?;
                    self.related(*relation, *position, &others, fired)?
                }
            };
            candidates = Some(match candidates {
                None => fitting,
                Some(c) => c.intersection(&fitting).cloned().collect(),
            });
        }
        let found: Vec<RustItem> = candidates
            .unwrap_or_else(|| self.items(|_| true))
            .into_iter()
            .filter(|i| description.kind.is_none_or(|k| i.kind == k))
            .collect();
        // A thing said to be a certain one but described wrongly ("`f` on
        // `S`" when `f` is not on `S`) is no thing at all.
        if found.is_empty()
            && description
                .conditions
                .iter()
                .any(|c| matches!(c, Condition::Is(_)))
        {
            return Err(Unsupported);
        }
        Ok(found)
    }
}

impl World<Rust> for RustWorld {
    fn lookup(&self, name: &Name, fired: &mut Vec<RuleInfo>) -> Vec<RustItem> {
        let wanted = name.0.as_str();
        let path = wanted.replace('.', "::");
        let matches = |item: &RustItem, fold: bool| {
            let eq = |a: &str, b: &str| {
                if fold {
                    a.eq_ignore_ascii_case(b)
                } else {
                    a == b
                }
            };
            eq(&item.name, wanted) || eq(&item.label(), &path)
        };
        let mut found = BTreeSet::new();
        if self.fire(LOOKUP_EXACT, fired).is_ok() {
            found = self.items(|i| matches(i, false));
        }
        if found.is_empty()
            && !self
                .records()
                .any(|r| r.type_idents.iter().any(|t| t == wanted))
            && self.fire(LOOKUP_ANY_CASE, fired).is_ok()
        {
            found = self.items(|i| matches(i, true));
        }
        found.into_iter().collect()
    }

    fn answer(&self, question: &Question<Rust>, fired: &mut Vec<RuleInfo>) -> Found<Reply<Rust>> {
        match question {
            Question::Identify(description) => Ok(Reply::Extent(Extent {
                description: description.clone(),
                members: self.identify(description, fired)?,
            })),
            Question::Count(description) => Ok(Reply::Count {
                description: description.clone(),
                count: u64::try_from(self.identify(description, fired)?.len()).unwrap_or(u64::MAX),
            }),
            Question::Locate { thing, kind } => Ok(Reply::Location(self.only(
                LOCATE_UNIQUE,
                thing,
                *kind,
                fired,
            )?)),
            Question::Passes(check) | Question::Problems(check) => self.check(check, fired),
            Question::Describe { thing, kind } => {
                let item = self.only(DESCRIBE_UNIQUE, thing, *kind, fired)?;
                if item.kind == Field {
                    return Err(WrongKind { item });
                }
                Ok(Reply::Account {
                    kind: item.kind,
                    item,
                })
            }
        }
    }

    fn propose(&self, action: &Action<Rust>, fired: &mut Vec<RuleInfo>) -> Found<Reply<Rust>> {
        let (thing, kind, name) = match action {
            Action::Rename { thing, kind, name } => (thing, kind, name),
            Action::Repair(check) => return self.remedy(check, fired),
        };
        // With no new name there is no edit to propose.
        let Some(name) = name else {
            resolved(thing)?;
            return Err(Unsupported);
        };
        if let Some(reply) = self.rename_field(thing, *kind, name, fired) {
            return reply;
        }
        let item = self.only(RENAME, thing, *kind, fired)?;
        if matches!(
            item.kind,
            RustKind::Struct | RustKind::Enum | RustKind::Variant
        ) {
            self.fire(
                if item.kind == RustKind::Variant {
                    RENAME_VARIANT
                } else {
                    RENAME_TYPE
                },
                fired,
            )?;
            // A package graph cannot account for references in another source.
            if self.sources.len() != 1 {
                return Err(Unsupported);
            }
            let source = self
                .sources
                .iter()
                .find(|s| s.name == item.source)
                .ok_or(Unsupported)?;
            let at = if item.kind == RustKind::Variant {
                rename_variants::positions(source, &item, &name.0)
            } else {
                rename_types::positions(source, &item, &name.0)
            }
            .ok_or(Unsupported)?;
            return Ok(Reply::Proposal {
                action: Action::Rename {
                    thing: Thing::Item(item),
                    kind: *kind,
                    name: Some(name.clone()),
                },
                spots: spots(source, &at),
            });
        }
        if !matches!(item.kind, Function | Field) {
            return Err(WrongKind { item });
        }
        // A function's new name must be an identifier no source writes.
        let taken = syn::parse_str::<syn::Ident>(&name.0).is_err()
            || self.sources.iter().any(|s| s.idents.contains_key(&name.0));
        self.rename_at(item, *kind, name, taken)
    }

    fn note(&self, fact: &Fact<Rust>, fired: &mut Vec<RuleInfo>) -> Found<Reply<Rust>> {
        let Fact::Alias { thing, kind, alias } = fact;
        let item = self.only(ALIAS_UNIQUE, thing, *kind, fired)?;
        Ok(Reply::Noted(Fact::Alias {
            thing: Thing::Item(item),
            kind: *kind,
            alias: alias.clone(),
        }))
    }

    fn check(&self, check: &RustCheck, fired: &mut Vec<RuleInfo>) -> Found<Reply<Rust>> {
        let problems = self
            .findings(*check, fired)?
            .into_iter()
            .map(|f| f.item)
            .collect();
        Ok(Reply::Checked {
            check: *check,
            problems,
        })
    }

    fn remedy(&self, check: &RustCheck, fired: &mut Vec<RuleInfo>) -> Found<Reply<Rust>> {
        let found = self.findings(*check, fired)?;
        if !self.switches.on(REMEDY_SUGGESTED.id) {
            return Err(Unsupported);
        }
        fired.push(REMEDY_SUGGESTED);
        let problems: Vec<RustItem> = found.iter().map(|f| f.item.clone()).collect();
        let mut changes: Vec<RustChange> =
            found.iter().flat_map(|f| f.fixes.iter().cloned()).collect();
        if self.switches.on(REMEDY_ONE_PER_PLACE.id) {
            fired.push(REMEDY_ONE_PER_PLACE);
            let mut places = BTreeSet::new();
            changes.retain(|c| {
                places.insert((
                    c.source.clone(),
                    c.file.clone(),
                    c.line,
                    c.column,
                    c.old.clone(),
                ))
            });
        }
        let mut derived_debug = false;
        if changes.is_empty() && self.switches.on(REMEDY_DEBUG.id) {
            for source in &self.sources {
                changes.extend(standard_traits::debug(&source.name, &source.root, &found));
            }
            derived_debug = !changes.is_empty();
            if derived_debug {
                fired.push(REMEDY_DEBUG);
            }
        }
        let mut converted_index = false;
        if changes.is_empty() && self.switches.on(REMEDY_INDEX.id) {
            for source in &self.sources {
                changes.extend(index_conversion::proposals(
                    &source.name,
                    &source.root,
                    &found,
                ));
            }
            converted_index = !changes.is_empty();
            if converted_index {
                fired.push(REMEDY_INDEX);
            }
        }
        changes.sort();
        changes.dedup();
        // Failing tests come with no fixes of the compiler's.
        if problems.iter().all(|p| p.kind == RustKind::Test)
            && self.switches.on(REMEDY_SLIP.id)
            && let Some(undone) = self
                .sources
                .iter()
                .filter(|s| problems.iter().any(|p| p.source == s.name))
                .map(|s| checks::undo_slip(&s.name, &s.root, &problems))
                .collect::<Option<Vec<_>>>()
        {
            fired.push(REMEDY_SLIP);
            changes = undone;
        }
        if problems.is_empty() || changes.is_empty() {
            return Ok(Reply::Remedy {
                check: *check,
                changes: Vec::new(),
                fixed: Vec::new(),
                left: problems,
            });
        }
        // Try the changes in a scratch copy; offer them only if they help.
        // Problems in untouched sources remain unresolved.
        let mut left: Vec<RustItem> = problems
            .iter()
            .filter(|p| !changes.iter().any(|c| c.source == p.source))
            .cloned()
            .collect();
        for source in &self.sources {
            let mine: Vec<RustChange> = changes
                .iter()
                .filter(|c| c.source == source.name)
                .cloned()
                .collect();
            if mine.is_empty() {
                continue;
            }
            let scratch = checks::scratch(&source.name);
            let tried = checks::copy_tree(&source.root, &scratch)
                .and_then(|()| {
                    checks::apply(
                        &mine,
                        &BTreeMap::from([(source.name.clone(), scratch.clone())]),
                    )
                })
                .and_then(|_| checks::run(*check, &source.name, &scratch));
            let tried = match tried {
                Ok(after)
                    if !after.is_empty()
                        && !derived_debug
                        && !converted_index
                        && self.switches.on(REMEDY_ITERATE.id) =>
                {
                    fired.push(REMEDY_ITERATE);
                    match checks::complete_suggestions(
                        *check,
                        &source.name,
                        &source.root,
                        &scratch,
                        &mine,
                        after.clone(),
                    ) {
                        Some(composed) => {
                            changes.retain(|c| c.source != source.name);
                            changes.extend(composed);
                            Ok(Vec::new())
                        }
                        None => Ok(after),
                    }
                }
                other => other,
            };
            let _ = std::fs::remove_dir_all(&scratch);
            match tried {
                Ok(after) => left.extend(after.into_iter().map(|f| f.item)),
                Err(_) => return Err(Unsupported),
            }
        }
        if left.len() >= problems.len() || ((derived_debug || converted_index) && !left.is_empty())
        {
            return Ok(Reply::Remedy {
                check: *check,
                changes: Vec::new(),
                fixed: Vec::new(),
                left: problems,
            });
        }
        let fixed = if left.is_empty() {
            problems
        } else {
            Vec::new()
        };
        Ok(Reply::Remedy {
            check: *check,
            changes,
            fixed,
            left,
        })
    }

    fn apply(&self, proposal: &Reply<Rust>, fired: &mut Vec<RuleInfo>) -> Found<Reply<Rust>> {
        if !self.switches.on(APPLY_CHANGES.id) {
            return Err(Unsupported);
        }
        let (changes, check) = match proposal {
            Reply::Remedy { changes, check, .. } if !changes.is_empty() => {
                (changes.clone(), Some(*check))
            }
            Reply::Proposal {
                action:
                    Action::Rename {
                        thing: Thing::Item(item),
                        name: Some(name),
                        ..
                    },
                spots,
            } => {
                let source = self.sources.iter().find(|s| s.name == item.source);
                let module_proof = item.kind == RustKind::Function
                    && source.is_some_and(|s| {
                        s.idents.get(&item.name).is_some_and(|at| {
                            at.iter()
                                .any(|p| s.names.get(p) != Some(&RustKind::Function))
                        })
                    });
                if module_proof {
                    // A preview's token text may survive a change of binding.
                    // Reprove identity before writing any bytes.
                    let source = source.ok_or(Unsupported)?;
                    let fresh = index::index_source(&source.name, &source.root);
                    let proven =
                        rename_functions::positions(&fresh, item, &name.0).ok_or(Unsupported)?;
                    if crate::spots(&fresh, &proven) != *spots {
                        return Err(Unsupported);
                    }
                }
                (
                    spots
                        .iter()
                        .map(|s| RustChange {
                            source: s.source.clone(),
                            file: s.file.clone(),
                            line: s.line,
                            column: s.column,
                            old: item.name.clone(),
                            new: name.0.clone(),
                        })
                        .collect(),
                    Some(if module_proof {
                        RustCheck::Test
                    } else {
                        RustCheck::Build
                    }),
                )
            }
            Reply::Assuming { reply, .. } => return self.apply(reply, fired),
            _ => return Err(Trouble::NothingPending),
        };
        fired.push(APPLY_CHANGES);
        let roots: BTreeMap<String, PathBuf> = self
            .sources
            .iter()
            .map(|s| (s.name.clone(), s.root.clone()))
            .collect();
        // Keep originals until verification succeeds, including when cargo
        // fails without readable diagnostics. A rejected edit changes nothing.
        let mut originals = BTreeMap::new();
        for change in &changes {
            let path = roots
                .get(&change.source)
                .ok_or(Unsupported)?
                .join(&change.file);
            if let std::collections::btree_map::Entry::Vacant(entry) = originals.entry(path.clone())
            {
                let bytes = std::fs::read(&path).map_err(|_| Unsupported)?;
                entry.insert(bytes);
            }
        }
        let applied = checks::apply(&changes, &roots);
        if let Ok(mut found) = self.found.lock() {
            found.clear();
        }
        let verified = applied.map_err(|_| Unsupported).and_then(|count| {
            let problems = match check {
                Some(c) => self.findings(c, fired)?,
                None => Vec::new(),
            };
            if problems.is_empty() {
                Ok(count)
            } else {
                Err(Unsupported)
            }
        });
        let count = match verified {
            Ok(count) => count,
            Err(trouble) => {
                let mut restored = true;
                for (path, bytes) in originals {
                    restored &= std::fs::write(path, bytes).is_ok();
                }
                if let Ok(mut found) = self.found.lock() {
                    found.clear();
                }
                if !restored {
                    eprintln!("glassmind: could not restore every file after a rejected edit");
                }
                return Err(trouble);
            }
        };
        Ok(Reply::Applied {
            changes: count,
            check,
            problems: Vec::new(),
        })
    }

    fn names(&self) -> Vec<String> {
        let names: BTreeSet<String> = self.records().map(|r| r.item.name.clone()).collect();
        names.into_iter().collect()
    }
}

/// The spots in `source` at these positions.
fn spots(source: &SourceIndex, at: &BTreeSet<index::Pos>) -> Vec<RustSpot> {
    at.iter()
        .map(|(file, line, column)| RustSpot {
            source: source.name.clone(),
            file: source.files.get(*file).cloned().unwrap_or_default(),
            line: *line,
            column: *column,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A rename is proposed only when every token with the old name names
    /// the function by syntax alone: a use inside a macro, whose tokens
    /// are not parsed, or a new name already taken, makes it a guess.
    #[test]
    fn renames_only_when_exact() {
        let root = std::env::temp_dir().join(format!("glassmind-rename-{}", std::process::id()));
        let world = |files: &[(&str, &str)]| {
            let _ = std::fs::remove_dir_all(&root);
            for (file, text) in files {
                std::fs::create_dir_all(&root).ok();
                std::fs::write(root.join(file), text).ok();
            }
            RustWorld::open(&[("s".to_owned(), root.as_path())], &Switches::all_on())
        };
        let rename = |world: &RustWorld, to: &str| {
            let item = world.lookup(&Name("step".into()), &mut Vec::new());
            let [item] = item.as_slice() else {
                return Err(Trouble::Unsupported);
            };
            let action = Action::Rename {
                thing: Thing::Item(item.clone()),
                kind: None,
                name: Some(Name(to.into())),
            };
            world.propose(&action, &mut Vec::new())
        };
        let code = "use a::step as go;\nfn step() {}\nfn run() { step(); x.step(); go(); }\n";
        let spots = match rename(&world(&[("a.rs", code)]), "advance") {
            Ok(Reply::Proposal { spots, .. }) => spots.len(),
            other => panic!("{other:?}"),
        };
        assert_eq!(spots, 4);
        assert_eq!(
            rename(&world(&[("a.rs", code)]), "run"),
            Err(Trouble::Unsupported)
        );
        let in_macro = [("a.rs", code), ("b.rs", "fn f() { dbg!(step()); }\n")];
        assert_eq!(
            rename(&world(&in_macro), "advance"),
            Err(Trouble::Unsupported)
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A field is renamed at every spot its name is written, and only when
    /// every one of them is written as that field.
    #[test]
    fn field_renames_are_exact_or_refused() {
        let root =
            std::env::temp_dir().join(format!("glassmind-rename-field-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("lib.rs"),
            "struct S { kept: u32, wrapped: u32, short: u32 }\n\
             fn f(s: &S) -> u32 {\n\
             \x20   let S { kept: k, .. } = s;\n\
             \x20   println!(\"{}\", s.wrapped);\n\
             \x20   let short = 1;\n\
             \x20   let t = S { kept: 1, wrapped: 2, short };\n\
             \x20   *k + t.kept\n\
             }\n",
        )
        .unwrap();
        let world = RustWorld::open(&[("t".into(), root.as_path())], &Switches::all_on());
        let rename = |old: &str, new: &str| {
            let item = world.lookup(&Name(old.into()), &mut Vec::new()).pop();
            let action = Action::Rename {
                thing: Thing::Item(item.unwrap()),
                kind: None,
                name: Some(Name(new.into())),
            };
            match world.propose(&action, &mut Vec::new()) {
                Ok(Reply::Proposal { spots, .. }) => {
                    Some(spots.iter().map(|s| (s.line, s.column)).collect::<Vec<_>>())
                }
                _ => None,
            }
        };
        assert_eq!(
            rename("kept", "held"),
            Some(vec![(1, 11), (3, 12), (6, 16), (7, 11)])
        );
        // Written inside a macro, as a shorthand, or a name already taken.
        assert_eq!(rename("wrapped", "held"), None);
        assert_eq!(rename("short", "held"), None);
        assert_eq!(rename("kept", "f"), None);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
