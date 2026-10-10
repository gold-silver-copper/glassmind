//! The meaning layer: what glassmind can think.
//!
//! Every sentence glassmind understands becomes a value of these types, and
//! every reply starts as one. They know no human language and no particular
//! data. A world fills in its own kinds, relations and items through the
//! [`Ontology`] trait; a language maps its sentences to and from these
//! values. If a concept has no type here or in a world's ontology, glassmind
//! cannot think it.
//!
//! Types are named after concepts, not after the words or grammatical
//! categories of any language.

use std::fmt::Debug;
use std::hash::Hash;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub mod interface;

/// A world's vocabulary of concepts: what kinds of things exist in it, how
/// they relate, and handles for the things themselves.
///
/// Implemented by a marker type in each world crate. The bounds on the
/// marker type itself only exist so the meaning types can derive their
/// traits; the marker carries no data.
pub trait Ontology: Copy + Eq + Ord + Hash + Debug + Default + 'static {
    /// Kinds of things, such as "function" in a code world.
    type Kind: Concept;
    /// Relations between two things, such as "calls" in a code world.
    type Relation: Concept;
    /// A thing in the world, resolved: a stable handle that can label itself.
    type Item: Concept + Labelled;
}

/// The bounds every concept a world defines must meet.
pub trait Concept: Clone + Eq + Ord + Hash + Debug + Serialize + DeserializeOwned {}
impl<T: Clone + Eq + Ord + Hash + Debug + Serialize + DeserializeOwned> Concept for T {}

/// A thing that can say what it is called in its world (an identifier, a
/// file name, a row key). This is data, not language: languages quote it.
pub trait Labelled {
    fn label(&self) -> String;
    /// Where the thing is set down in its world (a file and line, a table
    /// and row). Also data, quoted as it is.
    fn place(&self) -> String;
}

/// A name as written by the user: a reference to data by its identifier,
/// to be looked up in a world. Not a meaning.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Name(pub String);

/// Which argument of a two-place relation a thing fills.
///
/// For a relation R between a and b (written R(a, b)), `First` is a and
/// `Second` is b. In a code world, for "calls", `First` is the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Position {
    First,
    Second,
}

/// A thing the user talks about: named, or already resolved to an item.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum Thing<O: Ontology> {
    /// Referred to by name; not yet looked up.
    Named(Name),
    /// Resolved to one item of the world.
    Item(O::Item),
    /// Referred back to without a name: a thing said before, to be found
    /// from what came earlier.
    Previous,
}

/// A condition a thing must meet to fit a description.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum Condition<O: Ontology> {
    /// The described thing stands in `relation` to `other`, filling
    /// argument `position` of the relation.
    Related {
        relation: O::Relation,
        position: Position,
        other: Thing<O>,
    },
}

/// A description of things: an optional kind and conditions they all meet.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Description<O: Ontology> {
    /// The kind of the described things; `None` leaves it open ("what ...").
    pub kind: Option<O::Kind>,
    pub conditions: Vec<Condition<O>>,
}

/// A request for information.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum Question<O: Ontology> {
    /// Identify every thing that fits the description.
    Identify(Description<O>),
    /// Say where the thing is; `kind`, if given, is what the user said it is.
    Locate {
        thing: Thing<O>,
        kind: Option<O::Kind>,
    },
    /// Say what the thing is and what it is for; `kind`, if given, is what
    /// the user said it is.
    Describe {
        thing: Thing<O>,
        kind: Option<O::Kind>,
    },
}

/// Everything a user turn can mean.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum Utterance<O: Ontology> {
    Ask(Question<O>),
}

/// A claim that a set of things is exactly what fits a description.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Extent<O: Ontology> {
    pub description: Description<O>,
    /// Every item that fits, in a stable order; empty if nothing does.
    pub members: Vec<O::Item>,
}

/// Why something said could not be understood or carried out. Positions are
/// token indices into the user's turn.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum Trouble<O: Ontology> {
    /// The token at `at` is not a word the language knows.
    UnknownWord { at: usize, word: String },
    /// The words are known, but no sentence shape continues at `at`
    /// (`at` equal to the token count means the sentence stopped early).
    UnknownShape { at: usize },
    /// Nothing in the world has this name.
    NoSuchName { name: Name },
    /// A thing of this kind cannot take part in what was asked.
    WrongKind { item: O::Item },
    /// A reference has several candidates and nothing decides between them.
    Ambiguous { candidates: Vec<O::Item> },
    /// The meaning was understood, but this world cannot carry it out.
    Unsupported,
    /// The input had more readings than the parser keeps.
    TooAmbiguous,
}

/// Something the system took for granted to choose between readings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum Assumption<O: Ontology> {
    /// A name the user wrote was taken to mean this item.
    Means { name: Name, item: O::Item },
    /// The input was taken to mean this.
    Reading { meaning: Utterance<O> },
}

/// Everything a reply can mean, before a language puts it into words.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum Reply<O: Ontology> {
    /// The things that fit what was asked about.
    Extent(Extent<O>),
    /// The item asked about, which carries where it is.
    Location(O::Item),
    /// What the item asked about is: its kind, and where it is.
    Account { item: O::Item, kind: O::Kind },
    /// The request could not be understood or carried out.
    Trouble(Trouble<O>),
    /// A reply that rests on assumptions, which are stated first.
    Assuming {
        assumptions: Vec<Assumption<O>>,
        reply: Box<Reply<O>>,
    },
}

impl<O: Ontology> Utterance<O> {
    /// Every thing the utterance refers to, in order.
    pub fn things(&self) -> Vec<&Thing<O>> {
        match self {
            Utterance::Ask(q) => q.things(),
        }
    }

    /// The utterance with every thing replaced by `f` of it, in the order
    /// [`Utterance::things`] lists them.
    pub fn map_things(&self, f: &mut dyn FnMut(&Thing<O>) -> Thing<O>) -> Self {
        match self {
            Utterance::Ask(q) => Utterance::Ask(q.map_things(f)),
        }
    }
}

impl<O: Ontology> Question<O> {
    pub fn things(&self) -> Vec<&Thing<O>> {
        match self {
            Question::Identify(d) => d.things(),
            Question::Locate { thing, .. } | Question::Describe { thing, .. } => vec![thing],
        }
    }

    pub fn map_things(&self, f: &mut dyn FnMut(&Thing<O>) -> Thing<O>) -> Self {
        match self {
            Question::Identify(d) => Question::Identify(d.map_things(f)),
            Question::Locate { thing, kind } => Question::Locate {
                thing: f(thing),
                kind: kind.clone(),
            },
            Question::Describe { thing, kind } => Question::Describe {
                thing: f(thing),
                kind: kind.clone(),
            },
        }
    }
}

impl<O: Ontology> Description<O> {
    pub fn things(&self) -> Vec<&Thing<O>> {
        self.conditions.iter().flat_map(Condition::things).collect()
    }

    pub fn map_things(&self, f: &mut dyn FnMut(&Thing<O>) -> Thing<O>) -> Self {
        Description {
            kind: self.kind.clone(),
            conditions: self.conditions.iter().map(|c| c.map_things(f)).collect(),
        }
    }
}

impl<O: Ontology> Condition<O> {
    pub fn things(&self) -> Vec<&Thing<O>> {
        match self {
            Condition::Related { other, .. } => vec![other],
        }
    }

    pub fn map_things(&self, f: &mut dyn FnMut(&Thing<O>) -> Thing<O>) -> Self {
        match self {
            Condition::Related {
                relation,
                position,
                other,
            } => Condition::Related {
                relation: relation.clone(),
                position: *position,
                other: f(other),
            },
        }
    }
}

impl<O: Ontology> Reply<O> {
    /// Every item the reply mentions, in order.
    pub fn items(&self) -> Vec<&O::Item> {
        match self {
            Reply::Extent(e) => e
                .members
                .iter()
                .chain(e.description.things().into_iter().filter_map(|t| match t {
                    Thing::Item(i) => Some(i),
                    Thing::Named(_) | Thing::Previous => None,
                }))
                .collect(),
            Reply::Location(item)
            | Reply::Account { item, .. }
            | Reply::Trouble(Trouble::WrongKind { item }) => vec![item],
            Reply::Trouble(Trouble::Ambiguous { candidates }) => candidates.iter().collect(),
            Reply::Trouble(_) => Vec::new(),
            Reply::Assuming { reply, .. } => reply.items(),
        }
    }
}
