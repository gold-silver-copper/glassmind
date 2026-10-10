//! Words said before or after a phrase that leave it the phrase it was:
//! determiners ("a field", "its type", "the `x`"), asking whether there
//! are some things, which asks for them ("any structs with ...", "is there
//! some struct ..."), and words after a request that change nothing asked
//! ("... for me", "which ones?"). The system never says one.

use glassmind_grammar::{Rule, Sym};
use glassmind_meaning::Ontology;

use crate::asking::read_only_rule;
use crate::first;
use crate::sem::{Cat, Sem};

use Cat::*;
use Sym::{Cat as C, Word as W};

/// Words said before a phrase of a category, by rule ID and round.
const BEFORE: &[(&str, u32, Cat, &str)] = &[
    // Names, things and descriptions.
    // "just "the leaf"": only that name.
    ("en.alias.just", 6, Alias, "just"),
    // "the functions that `x` calls".
    ("en.restriction.that", 2, Restriction, "that"),
    // "the functions that `x` calls"
    ("en.description.the", 2, Description, "the"),
    // "all the functions that call `x`", "all functions that call it"
    ("en.description.all", 10, Description, "all"),
    // Before a holder
    // or a role: "a field", "one of their fields", "its type". Before a
    // description: asking whether there are some things asks for them ("any
    // structs with ...", "are there structs that ...", "is there many
    // functions that ...", "is there some struct with ..."). And "contains a
    // `Key`", "an `Arc`", "a field type", ", or contains it".
    ("en.noun_phrase.a", 3, NounPhrase, "a"),
    ("en.noun_phrase.an", 3, NounPhrase, "an"),
    ("en.alternative.comma", 3, Alternative, ","),
    ("en.description.any", 3, Description, "any"),
    ("en.description.are_there", 3, Description, "are there"),
    ("en.role_noun.a", 3, RoleNoun, "a"),
    ("en.role_noun.the", 3, RoleNoun, "the"),
    ("en.role_noun.its", 3, RoleNoun, "its"),
    ("en.holder.a", 9, Holder, "a"),
    ("en.holder.some", 9, Holder, "some"),
    ("en.holder.their", 9, Holder, "their"),
    ("en.holder.one_of_their", 9, Holder, "one of their"),
    ("en.holder.one_of_its", 9, Holder, "one of its"),
    ("en.description.many", 11, Description, "many"),
    ("en.description.a", 11, Description, "a"),
    ("en.part.a", 11, Part, "a"),
    ("en.description.some", 13, Description, "some"),
    // "where is it actually defined"
    ("en.defined.actually", 13, Defined, "actually"),
    // "the `x`", "the Walker"
    ("en.mention.the", 3, Mention, "the"),
    // ", and", ", or", ", like"
    ("en.join.comma_join", 3, Join, ","),
    // "name `x`", "the field name `fg`": the thing, by its name.
    ("en.mention.name_first", 5, Mention, "name"),
    // "its field `x`": the thing itself.
    ("en.mention.its", 5, Mention, "its"),
    // "change name of `rounded` to `to_ellipse`", "change `load_older`
    // name to ...": the thing whose name is meant.
    ("en.mention.name_of", 11, Mention, "name of"),
    // "from where `x` gets called", "from where does `x` get used"
    ("en.whence.from", 9, Where, "from"),
    // "either directly or wrapped"
    ("en.widening.either", 11, Widening, "either"),
    // "like &str": a reference to it.
    ("en.enclosure.reference", 11, Enclosure, "&"),
    // "that last question": what was said, by when.
    ("en.earlier.last", 6, Earlier, "last"),
    // "I meant", "I actually meant".
    ("en.meant.i", 10, Meant, "i"),
    // "you're currently keeping", "you currently remember", "your context"
    ("en.keeping.currently", 7, Keeping, "currently"),
    ("en.keep.currently", 7, Keep, "currently"),
    ("en.memory.your", 7, Memory, "your"),
    // "what u keeping in ur context": "your" typed short.
    ("en.memory.ur", 19, Memory, "ur"),
    // "like Vec<Post>", "a Vec", "another type", "some other type": a type
    // something is wrapped in.
    ("en.enclosure.like", 9, Enclosure, "like"),
    ("en.enclosure.a", 9, Enclosure, "a"),
    ("en.enclosure.another", 9, Enclosure, "another"),
    ("en.enclosure.other", 9, Enclosure, "other"),
    ("en.enclosure.some", 9, Enclosure, "some"),
];

/// Words said after a phrase of a category, by rule ID and round.
const AFTER: &[(&str, u32, Cat, &str)] = &[
    // "can you count how many functions call `x` for me?"
    ("en.question.for_me", 3, Question, "for me"),
    ("en.recount.for_me", 11, Recount, "for me"),
    ("en.recount.mark", 11, Recount, "?"),
    // "which ones?",
    // "whats in `x`, fields i mean"; "which structs have a `jobs` field in
    // them" ("them" is the subject again); "is there struct somewhere with
    // ..." (every thing asked about is somewhere).
    ("en.ask_again.question_mark", 5, AskAgain, "?"),
    ("en.verb_phrase.in_them", 5, VerbPhrase, "in them"),
    ("en.noun.somewhere", 11, Noun, "somewhere"),
    ("en.ask_again.i_mean", 14, AskAgain, "i mean"),
    // "I wasn't sure what "tape" means here": words quoted in a remark.
    ("en.remark.quote", 11, Remark, "\""),
    // "could you rename it to `open_tab_at` maybe?"
    ("en.command.maybe", 11, Command, "maybe"),
    // "change `load_older` name to ...": the thing whose name is meant.
    ("en.mention.name_after", 11, Mention, "name"),
    ("en.ask_again.for_me", 7, AskAgain, "for me"),
    // "Option<u16> etc", "Option<...>": wrapped in others like it.
    ("en.widening.etc", 9, Widening, "etc"),
    ("en.enclosure.generic_elided", 9, Enclosure, "< . . . >"),
    ("en.enclosure.etc", 9, Enclosure, "etc"),
    // "change it like that"
    ("en.carrying.like_that", 6, Carrying, "like that"),
    // "what you're keeping in your context, please"
    ("en.recall.please", 7, Recall, ", please"),
    // Words after a verb that say the calls happen inside the subject:
    // "what does `x` call inside", "what `x` calls into", "which functions
    // does `x` call inside it?", "what fields does `real_wiki` have in it?".
    ("en.restriction.into", 2, Restriction, "into"),
    ("en.restriction.inside", 2, Restriction, "inside"),
    ("en.restriction.inside_it", 13, Restriction, "inside it"),
    ("en.restriction.in_it", 14, Restriction, "in it"),
    // The punctuation after an aside: "..., ok?" (an aside asking whether
    // what was said is fine), and what ends one: "hey,", "hi!", "quick
    // question:", "sorry, quick one -", "Thanks.".
    ("en.aside.question", 6, Aside, "?"),
    ("en.aside.comma", 2, Aside, ","),
    ("en.aside.bang", 2, Aside, "!"),
    ("en.aside.colon", 2, Aside, ":"),
    ("en.aside.stop", 2, Aside, "."),
    ("en.aside.dash", 2, Aside, "-"),
];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let before = BEFORE
        .iter()
        .map(|&(id, round, cat, words)| (id, round, cat, vec![W(words), C(cat)]));
    let after = AFTER
        .iter()
        .map(|&(id, round, cat, words)| (id, round, cat, vec![C(cat), W(words)]));
    before
        .chain(after)
        .map(|(id, round, cat, rhs)| read_only_rule(id, round, cat, rhs, first))
        .collect()
}
