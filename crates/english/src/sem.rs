//! English categories and the values phrases carry while they are built.

use glassmind_meaning::{
    Assumption, Condition, Description, Extent, Name, Ontology, Position, Question, Reply, Thing,
    Utterance,
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
    /// A clause the user says about themselves that asks nothing: "I'm
    /// reworking the follow feature a bit".
    Remark,
    /// The polite opening of a request, which asks nothing itself: "can
    /// you", "can you please".
    Request,
    /// A word that greets before a request and asks nothing: "hey".
    Greeting,
    /// A word or two said around a request that asks nothing: a greeting,
    /// a hesitation, thanks ("hey,", "um", "quick question:", "thanks!").
    Aside,
    /// A participle saying a thing is written down somewhere: "defined".
    Defined,
    /// A noun for the place a thing is written down: "definition", "def".
    Definition,
    /// What the user wants located: a thing, or its definition ("the
    /// definition of `x`").
    Target,
    /// A question set inside a sentence: "where `x` is defined".
    Indirect,
    /// Words asking to be told the answer to an indirect question: "can you
    /// tell me", "I'd like to know".
    Inquiry,
    /// A verb asking for a thing to be found: "find", "point me to".
    Seek,
    /// A modal verb that makes a request polite: "can", "could".
    Modal,
    /// A verb asking to be told: "tell me", "show me", "point me to".
    Tell,
    /// A noun for where a thing is: "location", "loc".
    Location,
    /// A phrase that restricts what is asked about to the objects of a
    /// relation: "does `x` call", "`x` calls", "called by `x`".
    Restriction,
    /// A passive participle with its agent: "called by `x`".
    Passive,
    /// A noun phrase describing the things asked for: "fns called by `x`",
    /// "the functions that `x` calls", "callees of `x`".
    Description,
    /// A noun naming one side of a relation: "callees", "callers".
    RoleNoun,
    /// A word that opens or closes a turn and asks nothing: "hey,", "um,",
    /// "quick question:", "thanks!".
    Interjection,
    /// Words that frame a request without changing it: "can you", "tell
    /// me", "I'd like to know".
    Prompt,
    /// A thing as the user points at it, perhaps with the kind they say it
    /// is: "`x`", "the `X` struct", "fn `x`", "it".
    Mention,
    /// A verb for what a thing does when it runs: "do", "checks", "testing".
    Doing,
    /// What a thing serves: "for", "used for", "about".
    Use,
    /// A noun for what a thing is for: "purpose".
    Purpose,
    /// A word that softens or sharpens a question without changing what it
    /// asks: "even", "actually", "exactly".
    Hedge,
    /// "what is", "what's".
    WhatIs,
    /// A mark setting a thing off before what is asked about it: "`x` -
    /// what does it do", "`x`, what's it for".
    Break,
    /// Words joining two ways of asking one question: "and", ", or", "like".
    Join,
    /// A present participle with its object, a condition on what it
    /// describes: "calling `x`".
    Participial,
    /// A noun for how many things there are: "number", "count", "#".
    Amount,
    /// A dash setting a topic before a question about it: "`x` — how
    /// many callers".
    Dash,
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
    /// "called": the passive participle.
    Participle,
    /// "calling": after "is".
    Progressive,
}

impl VerbForm {
    /// The subject number this form agrees with.
    pub fn agrees_with(self) -> Number {
        match self {
            VerbForm::Plain => Number::Plural,
            VerbForm::ThirdSingular => Number::Singular,
            VerbForm::Participle | VerbForm::Progressive => Number::Plural,
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
    /// One side of a relation, named by a noun: "callees" are the second.
    Role {
        relation: O::Relation,
        position: Position,
    },
    /// A thing as mentioned, and the kind the user said it is.
    Mention {
        thing: Thing<O>,
        kind: Option<O::Kind>,
    },
    /// A number as typed or spoken.
    Count(u64),
    /// A text the system quotes, such as an unknown word.
    Quote(String),
}
