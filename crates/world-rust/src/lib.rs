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
    Action, Condition, Description, Extent, Labelled, Name, Ontology, Position, Question, Reply,
    Thing, Trouble,
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
    type Spot = RustSpot;
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
    /// The type or trait a function belongs to, or the struct a field belongs to.
    pub owner: Option<String>,
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
        format!("{}:{}", self.file, self.line)
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
/// Renaming a thing when the new name is not given ("rename the `x` field"):
/// a rename is only exact without type information, so a name that several
/// items of its kind share (a field declared by two structs) can't be
/// renamed until the user says which one is meant.
const RENAME_UNIQUE: RuleInfo = RuleInfo::new("rust.rename.unique", 5);

/// Every world rule, for listing and switching.
pub const RULES: &[RuleInfo] = &[
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
    RELATION_MEMBER,
    CONDITION_NAMED,
    RENAME_UNIQUE,
];

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

    /// Every item that fills `position` of `relation` when one of `others`
    /// fills the other argument. `Err` if the world cannot decide it.
    fn related(
        &self,
        relation: RustRelation,
        position: Position,
        others: &BTreeSet<RustItem>,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<BTreeSet<RustItem>, Trouble<Rust>> {
        let rule = match relation {
            RustRelation::Calls => RELATION_CALLS,
            RustRelation::HasField => RELATION_HAS_FIELD,
            RustRelation::Member => RELATION_MEMBER,
            RustRelation::TypeMentions => RELATION_TYPE_MENTIONS,
            RustRelation::BelongsTo => RELATION_BELONGS_TO,
        };
        if !self.switches.on(rule.id) {
            return Err(Trouble::Unsupported);
        }
        fired.push(rule);
        // The kinds `others` may be of.
        let wanted: &[RustKind] = match (relation, position) {
            (RustRelation::Calls, _) => &[RustKind::Function],
            (RustRelation::HasField, Position::First) => &[RustKind::Field],
            (RustRelation::HasField, Position::Second) => &[RustKind::Struct],
            (RustRelation::Member, Position::First) => &[RustKind::Field, RustKind::Function],
            (RustRelation::TypeMentions | RustRelation::BelongsTo | RustRelation::Member, _) => {
                &[RustKind::Struct, RustKind::Enum, RustKind::Trait]
            }
        };
        if let Some(item) = others.iter().find(|o| !wanted.contains(&o.kind)) {
            return Err(Trouble::WrongKind { item: item.clone() });
        }
        let mut out = BTreeSet::new();
        match (relation, position) {
            (RustRelation::Calls, _) => {
                for other in others {
                    out.extend(self.calls(position, other)?);
                }
            }
            // A struct's fields are indexed right after it.
            (RustRelation::HasField, _) => {
                for source in &self.sources {
                    let mut owner = None;
                    for record in &source.records {
                        let item = &record.item;
                        match (item.kind, owner) {
                            (RustKind::Struct, _) => owner = Some(item),
                            (RustKind::Field, Some(owner)) => {
                                let (this, that) = match position {
                                    Position::First => (owner, item),
                                    Position::Second => (item, owner),
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
            (RustRelation::Member, _) => {
                for item in self.records().map(|r| &r.item) {
                    for other in others {
                        let (whole, part) = match position {
                            Position::First => (item, other),
                            Position::Second => (other, item),
                        };
                        if part.source == whole.source
                            && part.owner.as_ref() == Some(&whole.name)
                            && !wanted.contains(&item.kind)
                        {
                            out.insert(item.clone());
                        }
                    }
                }
            }
            (RustRelation::TypeMentions, Position::First) => {
                let names: BTreeSet<&str> = others.iter().map(|o| o.name.as_str()).collect();
                out = self.typed(&names);
            }
            (RustRelation::BelongsTo, Position::First) => {
                out = self
                    .records()
                    .map(|r| &r.item)
                    .filter(|i| {
                        others
                            .iter()
                            .any(|o| o.source == i.source && i.owner.as_ref() == Some(&o.name))
                    })
                    .cloned()
                    .collect();
            }
            (RustRelation::TypeMentions | RustRelation::BelongsTo, Position::Second) => {
                return Err(Trouble::Unsupported);
            }
        }
        Ok(out)
    }

    /// Fields whose type mentions one of `names`.
    fn typed(&self, names: &BTreeSet<&str>) -> BTreeSet<RustItem> {
        self.records()
            .filter(|r| r.type_idents.iter().any(|t| names.contains(t.as_str())))
            .map(|r| r.item.clone())
            .collect()
    }

    /// The functions calling `other` (`First`), or that it calls (`Second`).
    fn calls(
        &self,
        position: Position,
        other: &RustItem,
    ) -> Result<BTreeSet<RustItem>, Trouble<Rust>> {
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
            // Functions `other` calls, among those whose name no
            // other item of any kind has.
            Position::Second => {
                let names: BTreeSet<&str> = source
                    .calls
                    .iter()
                    .filter(|c| item_at(c.caller).as_ref() == Some(other))
                    .map(|c| c.name.as_str())
                    .collect();
                let mut out = BTreeSet::new();
                for name in names {
                    let mut named = source.records.iter().filter(|r| r.item.name == name);
                    if let (Some(only), None) = (named.next(), named.next())
                        && only.item.kind == RustKind::Function
                    {
                        out.insert(only.item.clone());
                    }
                }
                Ok(out)
            }
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
    ) -> Result<RustItem, Trouble<Rust>> {
        if !self.switches.on(rule.id) {
            return Err(Trouble::Unsupported);
        }
        fired.push(rule);
        let item = match thing {
            Thing::Item(item) => item,
            Thing::Fitting(d) => {
                let mut fitting = self.identify(d, fired)?;
                return match (fitting.pop(), fitting.is_empty()) {
                    (Some(item), true) if kind.is_some_and(|k| item.kind != k) => {
                        Err(Trouble::WrongKind { item })
                    }
                    (Some(item), true) => Ok(item),
                    (Some(item), false) => {
                        fitting.push(item);
                        Err(Trouble::Ambiguous {
                            candidates: self
                                .records()
                                .filter(|r| fitting.contains(&r.item))
                                .map(|r| r.item.clone())
                                .collect(),
                        })
                    }
                    (None, _) => Err(Trouble::Unsupported),
                };
            }
            Thing::Named(name) => return Err(Trouble::NoSuchName { name: name.clone() }),
            Thing::Previous => return Err(Trouble::Unsupported),
        };
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
        Ok(item.clone())
    }

    /// The items a resolved thing stands for.
    fn things(
        &self,
        thing: &Thing<Rust>,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<BTreeSet<RustItem>, Trouble<Rust>> {
        match thing {
            Thing::Item(item) => Ok([item.clone()].into()),
            Thing::Fitting(d) => Ok(self.identify(d, fired)?.into_iter().collect()),
            Thing::Named(name) => Err(Trouble::NoSuchName { name: name.clone() }),
            Thing::Previous => Err(Trouble::Unsupported),
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
    ) -> Option<Result<Reply<Rust>, Trouble<Rust>>> {
        if !self.switches.on(RENAME_FIELD.id) || kind.is_some_and(|k| k != RustKind::Field) {
            return None;
        }
        let fields_named = |name: &str| -> Vec<RustItem> {
            self.records()
                .filter(|r| r.item.kind == RustKind::Field && r.item.name == name)
                .map(|r| r.item.clone())
                .collect()
        };
        // A field said to be of a struct is the one so described; a name
        // alone that several fields have asks which, whatever else it names.
        let item = match thing {
            Thing::Fitting(d) => match self.identify(d, fired) {
                Ok(items) => items.first().filter(|_| items.len() == 1)?.clone(),
                Err(trouble) => return Some(Err(trouble)),
            },
            Thing::Item(item) if fields_named(&item.name).len() > 1 => {
                fired.push(RENAME_FIELD);
                return Some(Err(Trouble::Ambiguous {
                    candidates: fields_named(&item.name),
                }));
            }
            Thing::Item(item) => item.clone(),
            Thing::Named(_) | Thing::Previous => return None,
        };
        if item.kind != RustKind::Field {
            return None;
        }
        fired.push(RENAME_FIELD);
        let source = self.sources.iter().find(|s| s.name == item.source)?;
        let at = source.idents.get(&item.name)?;
        if fields_named(&item.name).len() > 1
            || source.idents.contains_key(&name.0)
            || at
                .iter()
                .any(|p| source.names.get(p) != Some(&RustKind::Field))
        {
            return Some(Err(Trouble::Unsupported));
        }
        Some(Ok(Reply::Proposal {
            action: Action::Rename {
                thing: Thing::Item(item),
                kind,
                name: Some(name.clone()),
            },
            spots: spots(source, at),
        }))
    }

    /// A rename to a name not given, by [`RENAME_UNIQUE`]: which item is
    /// meant, if items of its kind share its name; otherwise there is no
    /// edit to propose.
    fn rename_unnamed(
        &self,
        thing: &Thing<Rust>,
        kind: Option<RustKind>,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<Reply<Rust>, Trouble<Rust>> {
        let item = match thing {
            Thing::Item(item) => item,
            Thing::Named(name) => return Err(Trouble::NoSuchName { name: name.clone() }),
            Thing::Previous | Thing::Fitting(_) => return Err(Trouble::Unsupported),
        };
        if !self.switches.on(RENAME_UNIQUE.id) {
            return Err(Trouble::Unsupported);
        }
        fired.push(RENAME_UNIQUE);
        // Every item of the kind meant that has the name, whichever of them
        // this reading took the name to mean. With no kind said, a name
        // several fields have asks which field, as [`RENAME_FIELD`] does.
        let sharing = |kind: RustKind| -> Vec<RustItem> {
            self.records()
                .filter(|r| r.item.name == item.name && r.item.kind == kind)
                .map(|r| r.item.clone())
                .collect()
        };
        let kind = kind.unwrap_or(if sharing(RustKind::Field).len() > 1 {
            RustKind::Field
        } else {
            item.kind
        });
        let candidates = sharing(kind);
        if candidates.len() > 1 {
            return Err(Trouble::Ambiguous { candidates });
        }
        if item.kind != kind {
            return Err(Trouble::WrongKind { item: item.clone() });
        }
        // With no new name there is no edit to propose.
        Err(Trouble::Unsupported)
    }

    /// The items of the kind (any kind if `None`) whose name is exactly
    /// `name`; "no such name" if there are none.
    fn named(
        &self,
        name: &Name,
        kind: Option<RustKind>,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<BTreeSet<RustItem>, Trouble<Rust>> {
        if !self.switches.on(CONDITION_NAMED.id) {
            return Err(Trouble::Unsupported);
        }
        fired.push(CONDITION_NAMED);
        let named: BTreeSet<RustItem> = self
            .records()
            .map(|r| &r.item)
            .filter(|i| i.name == name.0 && kind.is_none_or(|k| i.kind == k))
            .cloned()
            .collect();
        if named.is_empty() {
            return Err(Trouble::NoSuchName { name: name.clone() });
        }
        Ok(named)
    }

    fn identify(
        &self,
        description: &Description<Rust>,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<Vec<RustItem>, Trouble<Rust>> {
        let mut candidates: Option<BTreeSet<RustItem>> = None;
        for condition in &description.conditions {
            let fitting = match condition {
                Condition::Called(name) => self.lookup(name, fired).into_iter().collect(),
                Condition::Is(thing) => self.things(thing, fired)?,
                Condition::Named(name) => self.named(name, description.kind, fired)?,
                // A type needs no item to be named (`Arc`, `u32`), only to be
                // written as one somewhere.
                Condition::Related {
                    relation: RustRelation::TypeMentions,
                    position: Position::First,
                    other: Thing::Named(name),
                } if self.switches.on(RELATION_TYPE_MENTIONS.id) => {
                    fired.push(RELATION_TYPE_MENTIONS);
                    let fields = self.typed(&[name.0.as_str()].into());
                    if fields.is_empty() {
                        return Err(Trouble::NoSuchName { name: name.clone() });
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
        let candidates =
            candidates.unwrap_or_else(|| self.all_of(description.kind).into_iter().collect());
        let found: Vec<RustItem> = candidates
            .into_iter()
            .filter(|i| description.kind.is_none_or(|k| i.kind == k))
            .collect();
        // A thing said to be a certain one but described wrongly ("`f` on
        // `S`" when `f` is not on `S`) is no thing at all.
        let definite = || {
            description
                .conditions
                .iter()
                .any(|c| matches!(c, Condition::Is(_)))
        };
        if found.is_empty() && definite() {
            return Err(Trouble::Unsupported);
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
        let mut found: Vec<RustItem> = Vec::new();
        if self.switches.on(LOOKUP_EXACT.id) {
            found = self
                .records()
                .filter(|r| matches(&r.item, false))
                .map(|r| r.item.clone())
                .collect();
            fired.push(LOOKUP_EXACT);
        }
        if found.is_empty()
            && self.switches.on(LOOKUP_ANY_CASE.id)
            && !self
                .records()
                .any(|r| r.type_idents.iter().any(|t| t == wanted))
        {
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
            Question::Count(description) => Ok(Reply::Count {
                description: description.clone(),
                count: u64::try_from(self.identify(description, fired)?.len()).unwrap_or(u64::MAX),
            }),
            Question::Locate { thing, kind } => {
                let item = self.only(LOCATE_UNIQUE, thing, *kind, fired)?;
                Ok(Reply::Location(item))
            }
            Question::Describe { thing, kind } => {
                let item = self.only(DESCRIBE_UNIQUE, thing, *kind, fired)?;
                if item.kind == RustKind::Field {
                    return Err(Trouble::WrongKind { item: item.clone() });
                }
                Ok(Reply::Account {
                    kind: item.kind,
                    item,
                })
            }
        }
    }

    fn propose(
        &self,
        action: &Action<Rust>,
        fired: &mut Vec<RuleInfo>,
    ) -> Result<Reply<Rust>, Trouble<Rust>> {
        let Action::Rename { thing, kind, name } = action;
        let Some(name) = name else {
            return self.rename_unnamed(thing, *kind, fired);
        };
        if let Some(reply) = self.rename_field(thing, *kind, name, fired) {
            return reply;
        }
        let item = self.only(RENAME, thing, *kind, fired)?;
        if !matches!(item.kind, RustKind::Function | RustKind::Field) {
            return Err(Trouble::WrongKind { item: item.clone() });
        }
        let source = self
            .sources
            .iter()
            .find(|s| s.name == item.source)
            .ok_or(Trouble::Unsupported)?;
        let at = source.idents.get(&item.name).ok_or(Trouble::Unsupported)?;
        if syn::parse_str::<syn::Ident>(&name.0).is_err()
            || self.sources.iter().any(|s| s.idents.contains_key(&name.0))
            || at.iter().any(|p| source.names.get(p) != Some(&item.kind))
            || self
                .records()
                .filter(|r| r.item.name == item.name && r.item.kind == item.kind)
                .count()
                > 1
        {
            return Err(Trouble::Unsupported);
        }
        Ok(Reply::Proposal {
            action: Action::Rename {
                thing: Thing::Item(item.clone()),
                kind: *kind,
                name: Some(name.clone()),
            },
            spots: spots(source, at),
        })
    }

    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.records().map(|r| r.item.name.clone()).collect();
        names.sort();
        names.dedup();
        names
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
