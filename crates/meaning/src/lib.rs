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
    /// A place where something is written in the world's data, which an
    /// edit can rewrite (a line and column in a file).
    type Spot: Concept;
    /// A way the world checks itself, such as building or testing code.
    type Check: Concept;
    /// One change to the world's data, such as replacing text in a file.
    type Change: Concept;
    /// A world-specific specification for adding a member to an existing thing.
    type Member: Concept;
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
    /// Any thing that fits the description, left unnamed ("a field whose
    /// type is `Arc`").
    Fitting(Description<O>),
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
    /// The described thing is called this name.
    Called(Name),
    /// The described thing is this very thing ("the field `x` of `S`" is
    /// `x`, and also of `S`).
    Is(Thing<O>),
    /// The described thing goes by this name in its world ("a field named
    /// `done`"). The name is matched, not looked up: many things may share it.
    Named(Name),
    /// The described thing goes by the name of this other thing ("a field
    /// with that name", "a field called that"). Once the other thing is
    /// known only by its name, this is [`Condition::Named`].
    NamedLike(Thing<O>),
    /// The described thing is one of the parts of this other thing, as its
    /// world divides wholes into parts, with no relation said: the whole
    /// set before the part ("`S`'s `x`", "`S` `x`").
    PartOf(Thing<O>),
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
    /// Say how many things fit the description.
    Count(Description<O>),
    /// Say whether the world passes the check ("does it build?").
    Passes(O::Check),
    /// Say what the check finds wrong ("why does the build fail?").
    Problems(O::Check),
}

/// An explicit selection from alternatives already offered to the user.
/// Numbers are one-based, matching the displayed alternatives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selection {
    Number(usize),
    Other,
    Named(Name),
}

/// A request to change the world's data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum Action<O: Ontology> {
    /// Supply an implementation for a declared, unresolved target.
    Implement { thing: Thing<O> },
    /// Add a member while retaining its world-typed specification.
    AddMember { thing: Thing<O>, member: O::Member },
    /// Give the thing a new name wherever it is named; `kind`, if given, is
    /// what the user said it is. `name` is `None` when the user asks for a
    /// rename without saying the new name.
    Rename {
        thing: Thing<O>,
        kind: Option<O::Kind>,
        name: Option<Name>,
    },
    /// Make the world pass the check ("fix the build", "make the tests pass").
    Repair(O::Check),
}

/// Something the user tells the system, for it to keep in mind.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum Fact<O: Ontology> {
    /// The user will call the thing `alias`, a name of their own that the
    /// world does not know; `kind`, if given, is what the user said it is.
    Alias {
        thing: Thing<O>,
        kind: Option<O::Kind>,
        alias: Name,
    },
}

/// Everything a user turn can mean.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum Utterance<O: Ontology> {
    Ask(Question<O>),
    /// Asks for an action to be carried out.
    Command(Action<O>),
    /// Asks why the system took up the action it was last asked for.
    AskReason,
    /// Takes back what was said before: the conversation is to go on as if
    /// it had not been.
    Forget,
    /// States a fact for later turns.
    Tell(Fact<O>),
    /// Asks what the system holds of the conversation so far.
    AskRecall,
    /// Corrects the request just made: it was meant to be about this thing,
    /// in place of the one it was about.
    Correct(Thing<O>),
    /// Agrees to (true) or declines (false) what the system proposed.
    Consent(bool),
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
    /// A supported action failed its native preconditions or verification.
    ActionFailed { reason: String },
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
    /// Asked to apply or decline a proposal when none is waiting.
    NothingPending,
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
    /// How many things fit a description.
    Count {
        description: Description<O>,
        count: u64,
    },
    /// The edit that would carry out an action: every spot it rewrites.
    Proposal {
        action: Action<O>,
        spots: Vec<O::Spot>,
    },
    /// A verified concrete patch for a typed action. Evidence identifies the
    /// native proposal run; the host must revalidate it before application.
    Patch {
        action: Action<O>,
        changes: Vec<O::Change>,
        check: O::Check,
        trials: usize,
        evidence: String,
    },
    /// Why an action was taken up: the user asked for it in turn `turn`
    /// (0-based) of the conversation.
    Reason { turn: usize, action: Action<O> },
    /// What was said before is forgotten.
    Forgotten,
    /// The fact the user stated was taken note of.
    Noted(Fact<O>),
    /// What the system holds of the conversation: the items it has spoken
    /// of, most recent first.
    Recall(Vec<O::Item>),
    /// What a check found: nothing wrong if `problems` is empty.
    Checked {
        check: O::Check,
        problems: Vec<O::Item>,
    },
    /// Changes that would make the check pass, offered for the user to
    /// accept: the problems they fix, and those they leave.
    Remedy {
        check: O::Check,
        changes: Vec<O::Change>,
        fixed: Vec<O::Item>,
        left: Vec<O::Item>,
    },
    /// The proposed changes were made; what the check found afterwards, if
    /// there was a check to run.
    Applied {
        changes: usize,
        check: Option<O::Check>,
        problems: Vec<O::Item>,
    },
    /// The user declined what was proposed; nothing was changed.
    Declined,
    /// Not sure the input was understood: asks whether it meant this
    /// reading; "yes" answers it.
    Clarify { reading: Utterance<O> },
    /// The request could not be understood or carried out.
    Trouble(Trouble<O>),
    /// A reply that rests on the system taking the input, which could be
    /// read several ways, to mean `reading`, which is stated first.
    Assuming {
        reading: Utterance<O>,
        reply: Box<Reply<O>>,
    },
}

impl<O: Ontology> Utterance<O> {
    /// The names the utterance says things are called (see
    /// [`Description::called`]).
    pub fn called(&self) -> Vec<&Name> {
        match self {
            Utterance::Ask(Question::Identify(d) | Question::Count(d)) => d.called(),
            _ => self
                .things_within(&|_| false)
                .into_iter()
                .flat_map(Thing::called)
                .collect(),
        }
    }

    /// The things the utterance is about, without looking inside those it
    /// describes: for "which functions call `tick` on `Health`", the thing
    /// called `tick`, not `Health`.
    pub fn subjects(&self) -> Vec<&Thing<O>> {
        match self {
            Utterance::Ask(Question::Identify(d) | Question::Count(d)) => {
                d.conditions.iter().filter_map(Condition::other).collect()
            }
            Utterance::Ask(Question::Locate { thing, .. } | Question::Describe { thing, .. })
            | Utterance::Command(
                Action::Rename { thing, .. }
                | Action::Implement { thing }
                | Action::AddMember { thing, .. },
            )
            | Utterance::Tell(Fact::Alias { thing, .. })
            | Utterance::Correct(thing) => vec![thing],
            Utterance::Ask(Question::Passes(_) | Question::Problems(_))
            | Utterance::Command(Action::Repair(_))
            | Utterance::Consent(_)
            | Utterance::AskReason
            | Utterance::Forget
            | Utterance::AskRecall => Vec::new(),
        }
    }

    /// Every thing the utterance refers to, in order.
    pub fn things(&self) -> Vec<&Thing<O>> {
        self.things_within(&|_| true)
    }

    /// The things the utterance refers to, looking inside only the
    /// descriptions `within` accepts: one it does not is a thing itself.
    pub fn things_within(&self, within: &dyn Fn(&Description<O>) -> bool) -> Vec<&Thing<O>> {
        match self {
            Utterance::Ask(q) => q.things_within(within),
            Utterance::Command(
                Action::Rename { thing, .. }
                | Action::Implement { thing }
                | Action::AddMember { thing, .. },
            )
            | Utterance::Tell(Fact::Alias { thing, .. })
            | Utterance::Correct(thing) => thing.things_within(within),
            Utterance::Command(Action::Repair(_))
            | Utterance::Consent(_)
            | Utterance::AskReason
            | Utterance::Forget
            | Utterance::AskRecall => Vec::new(),
        }
    }

    /// The utterance with every thing replaced by `f` of it, in the order
    /// [`Utterance::things`] lists them.
    pub fn map_things(&self, f: &mut dyn FnMut(&Thing<O>) -> Thing<O>) -> Self {
        let mut out = self.clone();
        match &mut out {
            Utterance::Ask(q) => *q = q.map_things(f),
            Utterance::Command(
                Action::Rename { thing, .. }
                | Action::Implement { thing }
                | Action::AddMember { thing, .. },
            )
            | Utterance::Tell(Fact::Alias { thing, .. })
            | Utterance::Correct(thing) => *thing = thing.map_things(f),
            Utterance::Command(Action::Repair(_))
            | Utterance::Consent(_)
            | Utterance::AskReason
            | Utterance::Forget
            | Utterance::AskRecall => {}
        }
        out
    }
}

impl<O: Ontology> Question<O> {
    pub fn things(&self) -> Vec<&Thing<O>> {
        self.things_within(&|_| true)
    }

    pub fn things_within(&self, within: &dyn Fn(&Description<O>) -> bool) -> Vec<&Thing<O>> {
        match self {
            Question::Identify(d) | Question::Count(d) => d.things_within(within),
            Question::Locate { thing, .. } | Question::Describe { thing, .. } => {
                thing.things_within(within)
            }
            Question::Passes(_) | Question::Problems(_) => Vec::new(),
        }
    }

    pub fn map_things(&self, f: &mut dyn FnMut(&Thing<O>) -> Thing<O>) -> Self {
        let mut out = self.clone();
        match &mut out {
            Question::Identify(d) | Question::Count(d) => *d = d.map_things(f),
            Question::Locate { thing, .. } | Question::Describe { thing, .. } => {
                *thing = thing.map_things(f);
            }
            Question::Passes(_) | Question::Problems(_) => {}
        }
        out
    }
}

impl<O: Ontology> Description<O> {
    pub fn things(&self) -> Vec<&Thing<O>> {
        self.things_within(&|_| true)
    }

    /// Every name the description (and every description inside it) says
    /// a thing is called. Unlike a name matched ([`Condition::Named`]), a
    /// name nothing is called makes the description fit nothing.
    pub fn called(&self) -> Vec<&Name> {
        self.conditions
            .iter()
            .flat_map(|c| match c {
                Condition::Called(name) => vec![name],
                Condition::Related { other: thing, .. }
                | Condition::Is(thing)
                | Condition::NamedLike(thing)
                | Condition::PartOf(thing) => thing.called(),
                Condition::Named(_) => Vec::new(),
            })
            .collect()
    }

    pub fn things_within(&self, within: &dyn Fn(&Description<O>) -> bool) -> Vec<&Thing<O>> {
        self.conditions
            .iter()
            .flat_map(|c| c.things_within(within))
            .collect()
    }

    /// The resolved things the description refers to.
    pub fn items(&self) -> impl Iterator<Item = &O::Item> {
        self.things().into_iter().filter_map(Thing::item)
    }

    pub fn map_things(&self, f: &mut dyn FnMut(&Thing<O>) -> Thing<O>) -> Self {
        Description {
            kind: self.kind.clone(),
            conditions: self.conditions.iter().map(|c| c.map_things(f)).collect(),
        }
    }
}

impl<O: Ontology> Condition<O> {
    /// The other thing the condition relates the described thing to.
    pub fn other(&self) -> Option<&Thing<O>> {
        match self {
            Condition::Related { other, .. }
            | Condition::Is(other)
            | Condition::NamedLike(other)
            | Condition::PartOf(other) => Some(other),
            Condition::Called(_) | Condition::Named(_) => None,
        }
    }

    pub fn things_within(&self, within: &dyn Fn(&Description<O>) -> bool) -> Vec<&Thing<O>> {
        self.other()
            .map(|other| other.things_within(within))
            .unwrap_or_default()
    }

    pub fn things(&self) -> Vec<&Thing<O>> {
        self.other().map(Thing::things).unwrap_or_default()
    }

    pub fn map_things(&self, f: &mut dyn FnMut(&Thing<O>) -> Thing<O>) -> Self {
        let mut out = self.clone();
        if let Condition::Related { other, .. }
        | Condition::Is(other)
        | Condition::NamedLike(other)
        | Condition::PartOf(other) = &mut out
        {
            *other = other.map_things(f);
        }
        match out {
            Condition::NamedLike(Thing::Named(name)) => Condition::Named(name),
            out => out,
        }
    }
}

impl<O: Ontology> Thing<O> {
    /// The named or resolved things this one is, or refers to inside it.
    pub fn things(&self) -> Vec<&Thing<O>> {
        self.things_within(&|_| true)
    }

    /// The names a described thing is said to be called (see
    /// [`Description::called`]).
    pub fn called(&self) -> Vec<&Name> {
        match self {
            Thing::Fitting(d) => d.called(),
            Thing::Named(_) | Thing::Item(_) | Thing::Previous => Vec::new(),
        }
    }

    pub fn things_within(&self, within: &dyn Fn(&Description<O>) -> bool) -> Vec<&Thing<O>> {
        match self {
            Thing::Fitting(d) if within(d) => d.things_within(within),
            _ => vec![self],
        }
    }

    /// The item this thing is resolved to, if it is.
    pub fn item(&self) -> Option<&O::Item> {
        match self {
            Thing::Item(i) => Some(i),
            _ => None,
        }
    }

    pub fn map_things(&self, f: &mut dyn FnMut(&Thing<O>) -> Thing<O>) -> Self {
        match self {
            Thing::Fitting(d) => Thing::Fitting(d.map_things(f)),
            _ => f(self),
        }
    }
}

impl<O: Ontology> Reply<O> {
    /// Every item the reply mentions, in order.
    pub fn items(&self) -> Vec<&O::Item> {
        match self {
            Reply::Extent(e) => e.members.iter().chain(e.description.items()).collect(),
            Reply::Count { description, .. } => description.items().collect(),
            Reply::Location(item)
            | Reply::Account { item, .. }
            | Reply::Trouble(Trouble::WrongKind { item }) => vec![item],
            Reply::Proposal {
                action: Action::Rename { thing, .. },
                ..
            }
            | Reply::Reason {
                action: Action::Rename { thing, .. },
                ..
            }
            | Reply::Patch {
                action:
                    Action::Implement { thing }
                    | Action::Rename { thing, .. }
                    | Action::AddMember { thing, .. },
                ..
            }
            | Reply::Reason {
                action: Action::Implement { thing } | Action::AddMember { thing, .. },
                ..
            }
            | Reply::Noted(Fact::Alias { thing, .. }) => thing.item().into_iter().collect(),
            Reply::Trouble(Trouble::Ambiguous { candidates }) => candidates.iter().collect(),
            Reply::Assuming { reply, .. } => reply.items(),
            _ => Vec::new(),
        }
    }
}
