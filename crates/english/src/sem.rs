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
    /// A request to change something, without its final punctuation:
    /// "rename `x` to `y`", "`S.x` -> `y`".
    Command,
    /// Words saying what a thing is to become: "to", "into", "->".
    Into,
    /// Words saying a thing is to be called something else: "renamed to",
    /// "should be".
    Becomes,
    /// A verb giving a thing a name: "rename", "call".
    Naming,
    /// A noun for what a thing is called: "name", "current name".
    NameNoun,
    /// A mark ending a sentence: "?", ".", "!".
    Mark,
    /// Words asking again, or in more detail, for what was just asked: "can
    /// you do that?", "rename all uses", "update the callers too".
    FollowUp,
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
    /// The code being talked about, as a whole: "the code", "the
    /// codebase", "code".
    Codebase,
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
    /// Words asking how many things there are: "how many", and its
    /// mistypings "how mnay", "how mny".
    HowMany,
    /// Words asking for the things just asked about to be counted, of a
    /// kind if one is said: "can you count them for me?", "like how many
    /// exactly", "how many functions".
    Recount,
    /// Words saying something happens to a thing in many places: "in lot
    /// of places".
    Widely,
    /// A word standing for things whose kind is left open, before what
    /// describes them: "ones", "those", "anything".
    Ones,
    /// A dash setting a topic before a question about it: "`x` — how
    /// many callers".
    Dash,
    /// A phrase after a noun that restricts it by what fills one side of a
    /// relation: "whose type is or contains `Arc`", "of type `Tab`".
    Qualifier,
    /// Words saying a role's filler is or includes a thing: "is",
    /// "contains", "is or contains", "wraps".
    Includes,
    /// A second way of putting a qualifier, added after it: "or contains
    /// it", "(or something wrapping it)".
    Alternative,
    /// The thing a qualifier named, said again: "it", "one", or its name.
    Again,
    /// Words saying where in a thing another is: "somehow", "in it", "in
    /// its type".
    Within,
    /// A preposition naming a relation: "with" (having).
    Preposition,
    /// An adjective naming one side of a relation: "typed".
    RoleAdjective,
    /// A noun for a kind of thing that holds others: a "field" holds what
    /// its type mentions.
    Holder,
    /// A verb asking for a thing to get a new name: "rename", "change".
    Rename,
    /// An arrow from an old name to a new one: "->", "→".
    Arrow,
    /// A preposition saying what a thing belongs to, naming the relation
    /// between them: "of", "on", "in" (the field `x` of `S`).
    Belonging,
    /// A noun for a kind of thing that is part of another, naming how: a
    /// "field" of a struct ("`S` field `x`").
    Part,
    /// A thing set up to be talked about, perhaps with a comment on it: "the
    /// `x` field on `S`", "`x` in `S` is confusing".
    Topic,
    /// Words asking the hearer to carry out the request just made: "can
    /// you do that for me?", "could you take care of that?".
    Errand,
    /// A question after a question asking again for the things it asks
    /// about: "which ones?", "can you list them?".
    AskAgain,
    /// The name a thing is to be given: "`y`", "something like `y`".
    NewName,
    /// Words saying what a thing ought to be called: "should be called".
    ShouldBe,
    /// Words standing for a new name not given yet: "something clearer",
    /// "something more clear".
    Vague,
    /// What the user wishes for, said after a request: "I'd like the
    /// drawing functions to be named a bit more consistently".
    Wishing,
    /// The new name offered after a request that left it unsaid: "maybe
    /// `y`", "I think `y` fits better".
    Offer,
    /// A question asking why the system took up an action, without its
    /// final punctuation: "why do you want to change that".
    Why,
    /// Words for the action the system took up: "that", "that change",
    /// "the thing you just proposed".
    Deed,
    /// Words saying the system put an action forward: "you just proposed".
    Proposed,
    /// Words saying the system wants something: "do you want", "it wants".
    Wish,
    /// A verb phrase carrying out the action: "change that", "make that
    /// change".
    Carrying,
    /// A noun for why something is done: "reason", "the reasoning".
    Reason,
    /// A verb asking the system to drop what was said: "forget", "never
    /// mind", "scratch".
    Dismiss,
    /// Words pointing back at what the user said before: "that", "it",
    /// "that last question".
    Said,
    /// A noun for something said before, perhaps placed in time: "one",
    /// "question", "last thing".
    Earlier,
    /// A fact the user states, without its final punctuation: "I'll call
    /// `Page` "the leaf" from now on".
    Fact,
    /// A verb giving a thing a name of the user's own: "call", "calling".
    Calling,
    /// A verb giving a thing a name of the user's own, before "as": "refer
    /// to".
    Referring,
    /// Words saying the speaker means to do something: "I'm going to".
    Intent,
    /// Words saying how long something holds: "from now on".
    Time,
    /// A name the user gives a thing: ""the leaf"", "the runner".
    Alias,
    /// The opening of a question about where a thing is or where it
    /// undergoes something from: "where is", "where's", "where does", or
    /// "where" alone.
    Where,
    /// A verb in the passive, without its subject: "called", "gets
    /// called", "getting used".
    Undergo,
    /// A word before a participle making it passive: "is", "gets".
    Becoming,
    /// A thing undergoing a relation, as a condition on what stands on its
    /// other side: "`x` gets called".
    Undergoes,
    /// A word for some thing of any kind: "anything", "anyone".
    Anything,
    /// Words saying what the speaker meant to say: "I meant", "meant", "I
    /// actually meant".
    Meant,
    /// Any words, as typed.
    Words,
    /// Words the system does not know, as typed.
    Unknown,
    /// A preposition saying a thing is inside another, naming the relation
    /// from the container to what it holds: "in", "inside" ("what's in
    /// `x`").
    Inside,
    /// What the system holds of the conversation, as a clause naming it:
    /// "what you're keeping in your context right now", "what's in your
    /// context", "what do you remember of this convo".
    Recall,
    /// The hearer and a verb of holding in mind: "you're keeping", "do you
    /// remember", "you've got".
    Rememberer,
    /// "you're", "you are", "are you": the hearer, before a progressive
    /// verb.
    YouAre,
    /// "you", "do you": the hearer, before a plain verb.
    YouDo,
    /// A progressive verb of holding in mind: "keeping", "remembering".
    Keeping,
    /// A plain verb of holding in mind: "remember", "keep".
    Keep,
    /// A noun for where the system holds what was said: "context",
    /// "your memory".
    Memory,
    /// Words saying when or of what the system holds something: "right
    /// now", "so far", "of this conversation".
    Span,
    /// A preposition before the conversation: "of", "from", "about".
    Among,
    /// The conversation: "this conversation", "our convo".
    Talk,
    /// A word saying the clause after it means what was said before:
    /// "meaning", "like", "i.e.".
    Gloss,
    /// A remark after a question about what things hold, saying a thing
    /// counts when it is held inside another: "incl. wrapped", "also counts
    /// if it's inside some other type", "directly or nested".
    Widening,
    /// Words saying a thing is inside another: "wrapped in", "nested",
    /// "boxed", "inside".
    Wrapping,
    /// What a thing is inside: "something else", "another type", "a `Vec`
    /// or `Option`", "`Option<u64>` etc".
    Enclosure,
    /// Words opening a widening remark: "including", "incl.", "also", "even".
    Inclusion,
    /// Words saying the cases just named count: "counts too", "both count".
    Counted,
    /// Words for a thing held as it is: "directly", "just `Keypair`".
    Directly,
    /// A clause saying a thing is inside another: "it's wrapped in
    /// something", "the type just contains `Key`".
    Wrapped,
    /// A word as users mistype it, then the word they mean: never a phrase,
    /// but read wherever the word meant is ("whcih", "feild").
    Mistyped,
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
        // Only "calls" agrees with one thing; "call", "called" and
        // "calling" go with many.
        match self {
            VerbForm::ThirdSingular => Number::Singular,
            _ => Number::Plural,
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
    /// A kind of thing that holds what fills one side of a relation for it:
    /// a field holds the types its type mentions. Said of a part, the
    /// relation is to its whole: a field is a struct's.
    Holder {
        kind: O::Kind,
        relation: O::Relation,
        position: Position,
    },
    /// A number as typed or spoken.
    Count(u64),
    /// A text the system quotes, such as an unknown word.
    Quote(String),
}
