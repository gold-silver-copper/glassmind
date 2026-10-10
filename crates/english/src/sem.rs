//! English categories and the values phrases carry while they are built.

use glassmind_meaning::{
    Assumption, Condition, Description, Extent, Name, Ontology, Question, Reply, Thing, Utterance,
};

/// English phrase categories. These are English's own categories, used only
/// inside this crate; the meaning layer never sees them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cat {
    /// A whole user turn. The start category.
    Utterance,
    /// A question, without its final punctuation.
    Question,
    /// A noun phrase that refers to one thing.
    NounPhrase,
    /// A noun naming a kind of thing, singular or plural.
    Noun,
    /// A verb naming a two-place relation.
    Verb,
    /// A verb phrase: a verb and its object, a condition on the subject.
    VerbPhrase,
    /// A declarative sentence the system says.
    Statement,
    /// A list of items: "`a`", "`a` and `b`", "`a`, `b` and `c`".
    ItemList,
    /// A whole reply the system says.
    Reply,
    /// A statement of something the system assumed.
    Assumption,
}

/// English grammatical number, used for agreement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Number {
    Singular,
    Plural,
}

/// The English verb forms the grammar distinguishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerbForm {
    /// "call": plural subjects.
    Plain,
    /// "calls": singular third-person subjects.
    ThirdSingular,
}

impl VerbForm {
    /// The subject number this form agrees with.
    pub fn agrees_with(self) -> Number {
        match self {
            VerbForm::Plain => Number::Plural,
            VerbForm::ThirdSingular => Number::Singular,
        }
    }
}

/// The value of an English phrase while it is being read or spoken: a piece
/// of meaning plus the English features it needs (number, verb form).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sem<O: Ontology> {
    Utterance(Utterance<O>),
    Question(Question<O>),
    Reply(Reply<O>),
    Assumption(Assumption<O>),
    Extent(Extent<O>),
    Description(Description<O>),
    Thing(Thing<O>),
    Kind {
        kind: O::Kind,
        number: Number,
    },
    Relation {
        relation: O::Relation,
        form: VerbForm,
    },
    /// A condition on a subject, and the number the subject must have.
    Condition {
        condition: Condition<O>,
        number: Number,
    },
    /// A name as typed.
    Name(Name),
    /// A resolved item, as the system speaks of it.
    Item(O::Item),
    /// Several resolved items.
    Items(Vec<O::Item>),
    /// A number as typed or spoken.
    Count(u64),
    /// A text the system quotes, such as an unknown word.
    Quote(String),
}
