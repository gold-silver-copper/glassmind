//! The English words for the Rust world's kinds and relations.
//!
//! A world never adds grammar, and the English crate knows no world; this
//! bridge adds one rule per word form, each reading as a Rust concept.

use glassmind_english::{Cat, Number, Sem, VerbForm, blank, word};
use glassmind_grammar::Rule;
use glassmind_meaning::Position;
use glassmind_rules::{RuleInfo, SEED};
use glassmind_world_rust::{Rust, RustKind, RustRelation};

type Word = Rule<Cat, Sem<Rust>>;

/// One word rule per `(id, round, words, a, b)` row, reading the words as
/// a `cat` whose meaning `sem` makes from the row's `a` and `b`.
fn table<A: Copy, B: Copy>(
    cat: Cat,
    sem: fn(A, B) -> Sem<Rust>,
    rows: &[(&'static str, u32, &'static str, A, B)],
) -> Vec<Word> {
    rows.iter()
        .map(|&(id, round, words, a, b)| word(RuleInfo::new(id, round), cat, words, sem(a, b)))
        .collect()
}

/// The same rules, accepted but never said.
fn read_only(rules: Vec<Word>) -> Vec<Word> {
    rules.into_iter().map(Word::read_only).collect()
}

/// A kind of item, in the singular or plural: "function", "structs".
fn kind(kind: RustKind, number: Number) -> Sem<Rust> {
    Sem::Kind { kind, number }
}

/// A relation, as a verb in the given form or a preposition: "calls", "on".
fn relation(relation: RustRelation, form: VerbForm) -> Sem<Rust> {
    Sem::Relation { relation, form }
}

/// One argument of a relation, as a noun or adjective: "callers", "type".
fn role(relation: RustRelation, position: Position) -> Sem<Rust> {
    Sem::Role { relation, position }
}

/// A field, standing in a place of the relation: "a `Gate` field".
fn field(relation: RustRelation, position: Position) -> Sem<Rust> {
    Sem::Holder {
        kind: RustKind::Field,
        relation,
        position,
    }
}

/// Every English word for a Rust concept.
pub fn words() -> Vec<Word> {
    use Cat::*;
    use Number::*;
    use Position::*;
    use RustKind::*;
    use RustRelation::*;
    use VerbForm::*;
    [
        table(
            Noun,
            kind,
            &[
                (
                    "en.rust.noun.function",
                    SEED,
                    "function",
                    Function,
                    Singular,
                ),
                (
                    "en.rust.noun.functions",
                    SEED,
                    "functions",
                    Function,
                    Plural,
                ),
                ("en.rust.noun.fn", 2, "fn", Function, Singular),
                ("en.rust.noun.struct", 2, "struct", Struct, Singular),
                ("en.rust.noun.enum", 2, "enum", Enum, Singular),
                ("en.rust.noun.field", 2, "field", Field, Singular),
                // Short forms in terse requests: "fns called by `cut`",
                // "where's that func defined".
                ("en.rust.noun.fns", 2, "fns", Function, Plural),
                ("en.rust.noun.funcs", 2, "funcs", Function, Plural),
                ("en.rust.noun.func", 3, "func", Function, Singular),
                // "rename the `retry` method on `Chat`"
                ("en.rust.noun.method", 5, "method", Function, Singular),
                // "which structs have a field whose type is or contains `Arc`"
                ("en.rust.noun.structs", 3, "structs", Struct, Plural),
                ("en.rust.noun.fields", 3, "fields", Field, Plural),
            ],
        ),
        // A test is a function: "where is the test `a_blast_pushes_it`
        // defined?"
        read_only(table(
            Noun,
            kind,
            &[
                ("en.rust.noun.test", 13, "test", Function, Singular),
                // "what does `x` hold, field-wise": as to its fields.
                ("en.rust.noun.field_wise", 14, "field-wise", Field, Plural),
                // Mistyped, as hurried users do: "how many fucntions call
                // `x`", "how many functoins call `list_mut`", "where's that
                // fucntion defined".
                (
                    "en.rust.noun.mistyped.fucntions",
                    18,
                    "fucntions",
                    Function,
                    Plural,
                ),
                (
                    "en.rust.noun.mistyped.funcitons",
                    18,
                    "funcitons",
                    Function,
                    Plural,
                ),
                (
                    "en.rust.noun.mistyped.functoins",
                    18,
                    "functoins",
                    Function,
                    Plural,
                ),
                (
                    "en.rust.noun.mistyped.functons",
                    18,
                    "functons",
                    Function,
                    Plural,
                ),
                (
                    "en.rust.noun.mistyped.funtcions",
                    18,
                    "funtcions",
                    Function,
                    Plural,
                ),
                (
                    "en.rust.noun.mistyped.fucntion",
                    18,
                    "fucntion",
                    Function,
                    Singular,
                ),
            ],
        )),
        table(
            Verb,
            relation,
            &[
                ("en.rust.verb.calls", SEED, "calls", Calls, ThirdSingular),
                ("en.rust.verb.call", SEED, "call", Calls, Plain),
                // "which fns does `search_results` invoke"
                ("en.rust.verb.invokes", 2, "invokes", Calls, ThirdSingular),
                ("en.rust.verb.invoke", 2, "invoke", Calls, Plain),
                // "fns called by `parse_archives`"
                ("en.rust.verb.called", 2, "called", Calls, Participle),
                // "which functions is `parse_request` calling"
                ("en.rust.verb.calling", 2, "calling", Calls, Progressive),
                // "which functions belong to `App`"
                ("en.rust.verb.belong_to", 5, "belong to", BelongsTo, Plain),
                // "which structs have a field ..."
                ("en.rust.verb.has", 3, "has", HasField, ThirdSingular),
                ("en.rust.verb.have", 3, "have", HasField, Plain),
            ],
        ),
        read_only(table(
            Verb,
            relation,
            &[
                // What uses a function calls it: "who uses `on_floor`",
                // "which functions use it", "where `x` gets used".
                ("en.rust.verb.uses", 9, "uses", Calls, ThirdSingular),
                ("en.rust.verb.use", 9, "use", Calls, Plain),
                ("en.rust.verb.used", 9, "used", Calls, Participle),
                // A struct uses what it keeps in a field: "which structs use
                // `OnceLock` in a field type", "what structs use `UserCount`
                // as field type".
                ("en.rust.verb.use_field", 19, "use", HasField, Plain),
                (
                    "en.rust.verb.uses_field",
                    19,
                    "uses",
                    HasField,
                    ThirdSingular,
                ),
                // Mistyped: "how many fns calll `x`", "which fns `x`
                // callls", "which fns get caled by `x`", "how many fns
                // callling `x`".
                ("en.rust.verb.mistyped.calll", 18, "calll", Calls, Plain),
                (
                    "en.rust.verb.mistyped.callls",
                    18,
                    "callls",
                    Calls,
                    ThirdSingular,
                ),
                (
                    "en.rust.verb.mistyped.caled",
                    18,
                    "caled",
                    Calls,
                    Participle,
                ),
                (
                    "en.rust.verb.mistyped.callling",
                    18,
                    "callling",
                    Calls,
                    Progressive,
                ),
                // "which structs hold a field ...", "what structs got a field ..."
                ("en.rust.verb.holds", 3, "holds", HasField, ThirdSingular),
                ("en.rust.verb.hold", 3, "hold", HasField, Plain),
                ("en.rust.verb.got", 3, "got", HasField, Plain),
                // Mistyped: "what fields `Span` hav", "what fields `x` hve".
                ("en.rust.verb.mistyped.hav", 18, "hav", HasField, Plain),
                ("en.rust.verb.mistyped.hve", 18, "hve", HasField, Plain),
                // "find structs that contain a RepoCommit field"
                ("en.rust.verb.contain", 3, "contain", HasField, Plain),
                // "which structs keep a `LineCache` in a field", "store a
                // `Shown` in a field", "carry `KeyModifiers` in a field"
                ("en.rust.verb.keep", 3, "keep", HasField, Plain),
                ("en.rust.verb.store", 3, "store", HasField, Plain),
                ("en.rust.verb.carry", 3, "carry", HasField, Plain),
                // "which structs define a field named `order`", "structs
                // containing field `title`", "any structs holding a `jobs` field"
                ("en.rust.verb.define", 5, "define", HasField, Plain),
                (
                    "en.rust.verb.containing",
                    5,
                    "containing",
                    HasField,
                    Progressive,
                ),
                ("en.rust.verb.holding", 5, "holding", HasField, Progressive),
                // "what structs are storing a `Regex` in a field", "structs
                // storing an Fn in a field"
                ("en.rust.verb.storing", 9, "storing", HasField, Progressive),
                // "who stores a RefCell in a struct field", "structs that
                // embed ThemeConfig in a field"
                ("en.rust.verb.stores", 11, "stores", HasField, ThirdSingular),
                ("en.rust.verb.embed", 11, "embed", HasField, Plain),
                // "structs that declare a `sliding` field"
                ("en.rust.verb.declare", 13, "declare", HasField, Plain),
                // "which fields `S` contains", "which fields make up `S`",
                // "what fields `PostText` is made up of"
                (
                    "en.rust.verb.contains",
                    7,
                    "contains",
                    HasField,
                    ThirdSingular,
                ),
                ("en.rust.verb.make_up", 7, "make up", BelongsTo, Plain),
                (
                    "en.rust.verb.is_made_up_of",
                    7,
                    "is made up of",
                    HasField,
                    ThirdSingular,
                ),
            ],
        )),
        table(
            Preposition,
            relation,
            &[
                // "`save_filter` in `App`", "`regrid` on `Scene`"
                ("en.rust.preposition.on", 5, "on", BelongsTo, Plain),
                ("en.rust.preposition.of", 5, "of", BelongsTo, Plain),
                ("en.rust.preposition.in", 5, "in", BelongsTo, Plain),
                // "structs with a field of type `Notice`", "structs w/ a field ..."
                ("en.rust.preposition.with", 3, "with", HasField, Plain),
            ],
        ),
        read_only(table(
            Preposition,
            relation,
            &[
                // "fields inside `S`"
                ("en.rust.preposition.inside", 7, "inside", BelongsTo, Plain),
                // A struct has the fields it owns: "structs whose fields use
                // `Tab` type", "structs whose field types include `Sort`".
                ("en.rust.preposition.whose", 11, "whose", HasField, Plain),
            ],
        )),
        // A field holds what its type mentions: "a `Gate` field", "a field
        // that is `Key`", "fields containing `Signature`".
        table(
            Holder,
            field,
            &[
                ("en.rust.holder.field", 3, "field", TypeMentions, Second),
                ("en.rust.holder.fields", 3, "fields", TypeMentions, Second),
            ],
        ),
        table(
            RoleNoun,
            role,
            &[
                // "callees of `dm_vector`": what a function calls; "callers":
                // what calls it.
                ("en.rust.role.callees", 2, "callees", Calls, Second),
                ("en.rust.role.callers", 2, "callers", Calls, First),
                // A field's type: "whose type is `Arc`", "of type `Tab`".
                ("en.rust.role.type", 3, "type", TypeMentions, Second),
            ],
        ),
        // "typed `Art`"
        table(
            RoleAdjective,
            role,
            &[("en.rust.role.typed", 3, "typed", TypeMentions, Second)],
        ),
        read_only(table(
            RoleNoun,
            role,
            &[
                // "`save_viewed` caller count": the singular, as a noun modifier.
                ("en.rust.role.caller", 3, "caller", Calls, First),
                // "list structs whose field types include `Sort`"
                ("en.rust.role.types", 9, "types", TypeMentions, Second),
                // "call sites of `penetrator`": where it is called, which
                // are in its callers.
                ("en.rust.role.call_sites", 9, "call sites", Calls, First),
                // "calls made by `x`", "list calls made inside `pin_seen`":
                // what a function calls.
                ("en.rust.role.calls_made", 13, "calls made", Calls, Second),
                // A struct's fields are what it has: "`Conv` fields",
                // "`QueryEntry` members".
                ("en.rust.role.fields", 7, "fields", HasField, Second),
                ("en.rust.role.members", 7, "members", HasField, Second),
            ],
        )),
        // A field or method belongs to its type: "`x` in `S`". The thing
        // before the word is the member.
        table(
            Belonging,
            role,
            &[("en.rust.belonging.in", 5, "in", Member, Second)],
        ),
        // A type owns its fields and methods: "App's `on_mouse`". The
        // thing after the word is the member.
        read_only(table(
            Possessive,
            role,
            &[("en.rust.possessive.s", 21, "'s", Member, Second)],
        )),
        // What is inside a struct is what it has: "what's inside `NewDm`".
        read_only(table(
            Inside,
            relation,
            &[
                ("en.rust.inside.in", 7, "in", HasField, Plain),
                ("en.rust.inside.inside", 7, "inside", HasField, Plain),
            ],
        )),
        // A field is part of its struct: "Config field saved_max_mb".
        table(
            Part,
            field,
            &[("en.rust.part.field", 5, "field", HasField, Second)],
        ),
        // Kinds as users mistype them, then the word meant: "what
        // structs got a `dead` feild", "what strcuts have field named lists",
        // "callers of `gun_order`, whcih fucntions".
        [
            ("en.rust.mistyped.feild", "feild field"),
            ("en.rust.mistyped.fiedl", "fiedl field"),
            ("en.rust.mistyped.fild", "fild field"),
            ("en.rust.mistyped.strucs", "strucs structs"),
            ("en.rust.mistyped.strucst", "strucst structs"),
            ("en.rust.mistyped.strutcs", "strutcs structs"),
            ("en.rust.mistyped.strcuts", "strcuts structs"),
            ("en.rust.mistyped.fucntion", "fucntion function"),
            ("en.rust.mistyped.fucntions", "fucntions functions"),
        ]
        .map(|(id, words)| word(RuleInfo::new(id, 18), Cat::Mistyped, words, blank()).read_only())
        .to_vec(),
    ]
    .concat()
}

/// All English rules for talking about Rust: the language's own and the words for the world.
pub fn rules() -> Vec<Word> {
    glassmind_english::rules()
        .into_iter()
        .chain(words())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use glassmind_english::English;
    use glassmind_meaning::interface::{Language, Understanding};
    use glassmind_meaning::{
        Action, Condition, Description, Fact, Name, Position, Question, Reply, Thing, Utterance,
    };
    use glassmind_rules::Switches;

    /// Every question the system can say about every kind and relation it
    /// has words for parses back to the meaning it was said from.
    #[test]
    fn spoken_questions_parse_back_to_their_meaning() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let mut kinds = vec![None];
        let mut relations = Vec::new();
        for rule in words() {
            match rule.sem {
                glassmind_grammar::Sem::Const(Sem::Kind { kind, .. }) => kinds.push(Some(kind)),
                glassmind_grammar::Sem::Const(Sem::Relation { relation, .. }) => {
                    relations.push(relation)
                }
                _ => {}
            }
        }
        let mut checked = 0;
        for kind in &kinds {
            for relation in &relations {
                for position in [Position::First, Position::Second] {
                    let meaning = Utterance::Ask(Question::Identify(Description {
                        kind: *kind,
                        conditions: vec![Condition::Related {
                            relation: *relation,
                            position,
                            other: Thing::Named(Name("spawn_wave".into())),
                        }],
                    }));
                    let Some(spoken) =
                        english.speak(Cat::Utterance, &Sem::Utterance(meaning.clone()))
                    else {
                        continue;
                    };
                    let Understanding::Readings(readings) = english.understand(&spoken.text) else {
                        panic!("could not read back {:?}", spoken.text);
                    };
                    assert!(
                        readings.iter().any(|r| r.meaning == meaning),
                        "{:?} read back as something else",
                        spoken.text
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 0);
    }

    /// A remark about the speaker, before or after a request, a polite
    /// opening such as "can you tell me", or a greeting before it, leaves the
    /// request's meaning as it was.
    #[test]
    fn remarks_and_politeness_do_not_change_the_request() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meanings = |text: &str| match english.understand(text) {
            Understanding::Readings(readings) => {
                readings.into_iter().map(|r| r.meaning).collect::<Vec<_>>()
            }
            Understanding::Failed { .. } => Vec::new(),
        };
        let request = "which functions call `spawn_wave`?";
        let plain = meanings(request);
        assert!(!plain.is_empty());
        for text in [
            format!("I'm tidying up the `Wave` code. {request}"),
            format!("i'm looking at waves 2 and 3, {request}"),
            format!("{request} I'm curious."),
            "Can you tell me which functions call `spawn_wave`, please?".into(),
            "can you please show me which functions call `spawn_wave`".into(),
            format!("Hey, {request}"),
            format!("hey, I'm tidying up the `Wave` code, {request}"),
        ] {
            assert_eq!(meanings(&text), plain, "{text:?}");
        }
    }

    /// Hurried ways of asking what calls a function mean what the plain
    /// question means, of the kind said or of any kind.
    #[test]
    fn hurried_ways_of_asking_for_callers_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meanings = |text: &str| match english.understand(text) {
            Understanding::Readings(readings) => {
                let mut m: Vec<_> = readings.into_iter().map(|r| r.meaning).collect();
                m.dedup();
                m
            }
            Understanding::Failed { .. } => Vec::new(),
        };
        for (plain, texts) in [
            (
                "which functions call `cut`?",
                &[
                    "cut called by what fns",
                    "anythign call cut, which fucntions",
                    "um, which functions call `cut`, if any?",
                ][..],
            ),
            (
                "what calls `cut`?",
                &[
                    "cut called from where",
                    "who cals cut",
                    "who uses cut, callers only",
                ][..],
            ),
        ] {
            let plain = meanings(plain);
            assert_eq!(plain.len(), 1);
            for text in texts {
                assert_eq!(meanings(text), plain, "{text:?}");
            }
        }
    }

    /// However the user asks where a thing is, and whatever asides they
    /// put around it, it means what the system says when it asks that.
    #[test]
    fn ways_of_asking_where_mean_the_same() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meaning = Utterance::Ask(Question::Locate {
            thing: Thing::Named(Name("spawn_wave".into())),
            kind: None,
        });
        let spoken = english
            .speak(Cat::Utterance, &Sem::Utterance(meaning.clone()))
            .map(|s| s.text);
        assert_eq!(spoken.as_deref(), Some("Where is `spawn_wave` defined?"));
        for text in [
            "Where is `spawn_wave` defined?",
            "Hey, quick question: where's spawn_wave defined? Thanks!",
            "Can you tell me where `spawn_wave` is defined, please?",
            "I'd like to know where `spawn_wave` is defined.",
            "um, where does `spawn_wave` live in the codebase? can't find it",
            "find the definition of `spawn_wave`",
            "spawn_wave defined where",
            "where spawn_wave defined at",
            "spawn_wave where is it",
            "wehre is spawn_wave defiend",
            "spawn_wave defnied whre",
            "Where's spawn_wave defined, in which file?",
            "def of spawn_wave",
            "I'm trying to figure out where `spawn_wave` is defined, can you point me?",
            "where is `spawn_wave` defined? keep seeing it around",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().all(|r| r.meaning == meaning), "{text:?}");
        }
    }

    /// Asking where the thing said before is, going on from the last turn,
    /// leaves what it refers back to for the conversation to find.
    #[test]
    fn ways_of_asking_where_it_is_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meaning = Utterance::Ask(Question::Locate {
            thing: Thing::Previous,
            kind: None,
        });
        for text in [
            "And where is it defined?",
            "Got it. Can you tell me where it's defined?",
            "Great, and where is that defined actually?",
            "where's that defined",
            "where's it live",
            "and where it is defined?",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().all(|r| r.meaning == meaning), "{text:?}");
        }
    }

    /// The many ways of asking what a function calls all mean the same.
    #[test]
    fn ways_of_asking_for_callees_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meanings = |text: &str| match english.understand(text) {
            Understanding::Readings(readings) => {
                let mut m: Vec<_> = readings.into_iter().map(|r| r.meaning).collect();
                m.dedup();
                m
            }
            Understanding::Failed { .. } => Vec::new(),
        };
        let plain = meanings("which functions does `cut` call?");
        assert_eq!(plain.len(), 1);
        for text in [
            "Hey, can you tell me which functions `cut` calls, please?",
            "I'd like to know which functions `cut` calls. Thanks!",
            "um, which functions are called by `cut`?",
            "fns called from inside cut",
            "Could you list the functions that `cut` calls?",
            "cut calls which fns",
            "what functions does `cut` end up calling?",
            "which functions `cut` ends up calling?",
            "what functions are getting called from `cut`?",
        ] {
            assert_eq!(meanings(text), plain, "{text:?}");
        }
        for text in [
            "callees of `cut`",
            "what's `cut` calling",
            "cut -> callees",
            "calls made by cut",
        ] {
            assert_eq!(
                meanings(text),
                meanings("what does `cut` call?"),
                "{text:?}"
            );
        }
    }

    /// However the user asks what a thing is for, whether they name it
    /// before the question or ask twice, it means what the system says
    /// when it asks that.
    #[test]
    fn ways_of_asking_what_a_thing_does_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meaning = Utterance::Ask(Question::Describe {
            thing: Thing::Named(Name("seed_at".into())),
            kind: None,
        });
        let spoken = english
            .speak(Cat::Utterance, &Sem::Utterance(meaning.clone()))
            .map(|s| s.text);
        assert_eq!(spoken.as_deref(), Some("What does `seed_at` do?"));
        for text in [
            "what's seed_at for",
            "Quick question: what does `seed_at` do, and what is it for?",
            "what is `seed_at` even for? what does it do",
            "I'd like to know what `seed_at` does. Could you explain it, please?",
            "seed_at - what's it do",
            "purpose of `seed_at`",
            "explain seed_at quickly",
            "what seed_at for",
            "seed_at, what it for",
            "describe seed_at",
            "what seed_at does, can you explain?",
            "what seed_at do / whats it for",
            "can you tell me what seed_at is",
            "what seed_at is, what it for",
            "what seed_at doo exaclty",
            "seed_at whats it fro",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().all(|r| r.meaning == meaning), "{text:?}");
        }
        // A second question about something else is not the same question.
        assert!(matches!(
            english.understand("what does `seed_at` do, and which functions call it?"),
            Understanding::Failed { .. }
        ));
    }

    /// The ways of asking how many functions call a function all mean the same.
    #[test]
    fn ways_of_counting_callers_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meanings = |text: &str| match english.understand(text) {
            Understanding::Readings(readings) => {
                let mut m: Vec<_> = readings.into_iter().map(|r| r.meaning).collect();
                m.dedup();
                m
            }
            Understanding::Failed { .. } => Vec::new(),
        };
        let plain = meanings("how many functions call `cut`?");
        assert_eq!(plain.len(), 1);
        for text in [
            "Hey, can you count how many functions call `cut` for me?",
            "how many fns are calling cut",
            "count of funcs that call `cut`",
            "number of fns calling `cut`",
            "cut called by how many functions",
            "how many places call `cut`, fn count only",
            "how many fns call cut? or exact number if you can",
            "how many functions call `cut`, is it used in lot of places?",
            "how mnay fucntions calll cut",
            "cut called by how mny functoins",
        ] {
            assert_eq!(meanings(text), plain, "{text:?}");
        }
        // How widely it is used asks again only of the things that use it.
        assert!(
            meanings("how many functions does `cut` call, is it used in lot of places?").is_empty()
        );
        let by_role = meanings("number of callers of `cut`");
        assert_eq!(by_role.len(), 1);
        for text in [
            "how many callers does `cut` have",
            "cut — how many callers",
            "cut caller count",
            "how many callers `cut` has",
            "cut, how many callers it got",
            "cut how many callers it hav",
        ] {
            assert_eq!(meanings(text), by_role, "{text:?}");
        }
    }

    /// Ways of saying the request just made was meant to be about another
    /// thing all correct it to that thing.
    #[test]
    fn ways_of_correcting_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let corrects = Utterance::Correct(Thing::Named(Name("cut".into())));
        for text in [
            "Oops, sorry, I actually meant `cut`.",
            "sorry, meant cut",
            "oh sorry, I meant `cut` actually",
            "Sorry, I meant `cut`, not that one.",
            "no, I meant the function `cut`, not `trim`",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().any(|r| r.meaning == corrects), "{text:?}");
        }
    }

    /// Asking where a function is called from, whether anything calls it,
    /// or who uses it, all ask for its callers: of any kind, or functions
    /// when a later question says so.
    #[test]
    fn ways_of_asking_for_callers_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let asks = |kind| {
            Utterance::Ask(Question::Identify(Description {
                kind,
                conditions: vec![Condition::Related {
                    relation: RustRelation::Calls,
                    position: Position::First,
                    other: Thing::Named(Name("cut".into())),
                }],
            }))
        };
        for (text, kind) in [
            ("where is `cut` called from?", None),
            ("where's cut getting called from", None),
            ("from where does `cut` get used", None),
            ("does anything call `cut()`?", None),
            ("who uses cut", None),
            (
                "is there anything that ends up calling `cut`? which functions call it",
                Some(RustKind::Function),
            ),
            (
                "I'm trying to figure out where `cut` gets used, which functions?",
                Some(RustKind::Function),
            ),
            (
                "trying to understand `cut`, which functions are calling it?",
                Some(RustKind::Function),
            ),
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().any(|r| r.meaning == asks(kind)), "{text:?}");
        }
    }

    /// The ways of asking which structs have a field of a type mean what
    /// the system says when it asks that.
    #[test]
    fn ways_of_asking_for_structs_by_field_type_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let field = Thing::Fitting(Description {
            kind: Some(RustKind::Field),
            conditions: vec![Condition::Related {
                relation: RustRelation::TypeMentions,
                position: Position::First,
                other: Thing::Named(Name("Arc".into())),
            }],
        });
        let meaning = Utterance::Ask(Question::Identify(Description {
            kind: Some(RustKind::Struct),
            conditions: vec![Condition::Related {
                relation: RustRelation::HasField,
                position: Position::First,
                other: field,
            }],
        }));
        let spoken = english
            .speak(Cat::Utterance, &Sem::Utterance(meaning.clone()))
            .map(|s| s.text);
        assert_eq!(
            spoken.as_deref(),
            Some("Which structs have a field whose type is or contains `Arc`?")
        );
        for text in [
            "I'd like to know which structs have a field whose type is `Arc` or contains it.",
            "structs w/ a field of type Arc (or something wrapping it)",
            "what structs have a field with a type that contains `Arc`?",
            "any structs with fields typed Arc or containing Arc",
            "are there structs that have a field that is an Arc? which ones",
            "which structs keep an `Arc` in one of their fields? also counts if it's wrapped in something",
            "structs w/ fields typed Arc (incl. nested, Option<Arc> etc)",
            "which structs have a field with `Arc` in its type, directly or wrapped",
            "which structs have a field typed as `Arc`, or with a type that includes it?",
            "who has a field typed `Arc`? structs only",
            "structs w/ fields of `Arc` (direct or nested)",
            "is there a struct with a field of type `Arc`, or a field that contains it?",
            "where is `Arc` used as a field type? which structs have a field that contains it?",
            "`Arc` used as field type where? structs only",
            "any structs w/ field typed Arc, list them pls",
            "whcih structs have field w/ Arc type",
            "structs that hav field type Arc, which",
            "field type Arc or contains it, what structs have tht",
            "what structs got a feild that uses Arc",
            "which structs have a field of type Arc (or Option of it etc)",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().all(|r| r.meaning == meaning), "{text:?}");
        }
        // A type named by an English word, capitalized, is still named.
        let Understanding::Readings(readings) =
            english.understand("which structs have a field typed Handle (or wraping one)")
        else {
            panic!("could not read the capitalized word");
        };
        assert!(readings.iter().any(|r| r.meaning
            == meaning.map_things(&mut |t| match t {
                Thing::Named(_) => Thing::Named(Name("Handle".into())),
                other => other.clone(),
            })));
        // "an `Arc` field" may also be a field named `Arc`; the world
        // settles which.
        let Understanding::Readings(readings) =
            english.understand("which structs hold an `Arc` field")
        else {
            panic!("could not read the compound");
        };
        assert!(readings.iter().any(|r| r.meaning == meaning));
    }

    /// The ways of asking which structs have a field of a name mean what
    /// the system says when it asks that.
    #[test]
    fn ways_of_asking_for_structs_by_field_name_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meaning = Utterance::Ask(Question::Identify(Description {
            kind: Some(RustKind::Struct),
            conditions: vec![Condition::Related {
                relation: RustRelation::HasField,
                position: Position::First,
                other: Thing::Fitting(Description {
                    kind: Some(RustKind::Field),
                    conditions: vec![Condition::Named(Name("done".into()))],
                }),
            }],
        }));
        let spoken = english
            .speak(Cat::Utterance, &Sem::Utterance(meaning.clone()))
            .map(|s| s.text);
        assert_eq!(
            spoken.as_deref(),
            Some("Which structs have a field named `done`?")
        );
        for text in [
            "I'd like to know which structs in the codebase have a field called `done`, please.",
            "so which structs have the field `done` in them?",
            "which struct(s) define a done field",
            "list structs containing field `done`",
            "what structs got done as a field",
            "which structs have field named done? still finding my way around here",
            "I saw `done` somewhere, which structs have a field with that name?",
            "i keep seeing `done` around, which structs have field named that?",
            "`done` field - what structs",
            "I'm looking for structs that have `done` field, which ones are there?",
            "where's done declared as a struct field, which structs",
            "what structs have `done` as field",
            "which struct has done field in it",
            "is there any struct that has a field `done`? what structs is it in",
            "whcih strucs hav teh feild named done",
            "structs that have a `done` field, which",
            "field done, which structs have it",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().any(|r| r.meaning == meaning), "{text:?}");
        }
        // A name the system also knows as English, where only a name fits.
        let named = |text: &str| match english.understand(text) {
            Understanding::Readings(readings) => readings.into_iter().map(|r| r.meaning).collect(),
            Understanding::Failed { .. } => Vec::new(),
        };
        let calls = named("which structs have a field named `calls`?");
        for text in [
            "which structs have field named calls?",
            "which structs have field calls",
        ] {
            assert!(named(text).iter().any(|m| calls.contains(m)), "{text:?}");
        }
    }

    /// However the user asks for a rename, with a topic set before it, a
    /// follow-up or remarks after it, it means what the system says when it
    /// proposes it.
    #[test]
    fn ways_of_asking_for_a_rename_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meaning = Utterance::Command(Action::Rename {
            thing: Thing::Named(Name("tick".into())),
            kind: None,
            name: Some(Name("drain_events".into())),
        });
        let spoken = english
            .speak(Cat::Utterance, &Sem::Utterance(meaning.clone()))
            .map(|s| s.text);
        assert_eq!(spoken.as_deref(), Some("Rename `tick` to `drain_events`"));
        for text in [
            "Could you rename `tick` to `drain_events`, please?",
            "tick -> drain_events pls",
            "`tick` is a bit cryptic. Can you rename it to `drain_events`?",
            "I'd like to rename `tick` to `drain_events`. Could you do that for me?",
            "rename tick to drain_events? the current name is vague",
            "tick should be drain_events, rename it",
            "rename tick, call it drain_events",
            "can we rename `tick` to something clearer? maybe `drain_events`",
            "please rename `tick` to `drain_events`, tick is kind of vague",
            "`tick` to `drain_events`, thx",
            "rename tick to smth better pls, maybe drain_events",
            "rename tick to something better, `drain_events`",
            "rename tick, want it called drain_events isntead",
            "tick shoud be called drain_events, rename pls",
            "rename tick to drain_events pls, current name kinda vague",
            "tick -> drain_events pls rename evrywhere",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            // A reading may also take words before "rename it" as a remark,
            // leaving the new name unsaid; it can only ask back.
            let unnamed = |u: &Utterance<Rust>| {
                matches!(u, Utterance::Command(Action::Rename { name: None, .. }))
            };
            assert!(readings.iter().any(|r| r.meaning == meaning), "{text:?}");
            assert!(
                readings
                    .iter()
                    .all(|r| r.meaning == meaning || unnamed(&r.meaning)),
                "{text:?}"
            );
        }
        // Words after the request without a break may be part of it.
        assert!(matches!(
            english.understand("rename tick to drain_events tick is vague"),
            Understanding::Failed { .. }
        ));
    }

    /// Asking to rename a field after setting its struct as the topic can
    /// mean the field that is `x` and a member of the struct that is `S`
    /// ("field `x`" may also be a field of type `x`, which the world
    /// decides).
    #[test]
    fn ways_of_asking_to_rename_a_field_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let is = |kind, name: &str| {
            Thing::Fitting(Description {
                kind: Some(kind),
                conditions: vec![Condition::Is(Thing::Named(Name(name.into())))],
            })
        };
        let field = Description {
            kind: None,
            conditions: vec![
                Condition::Is(Thing::Named(Name("x".into()))),
                Condition::Related {
                    relation: RustRelation::Member,
                    position: Position::Second,
                    other: is(RustKind::Struct, "S"),
                },
            ],
        };
        let meaning = Utterance::Command(Action::Rename {
            thing: Thing::Fitting(field),
            kind: Some(RustKind::Field),
            name: Some(Name("y".into())),
        });
        let text = "in the `S` struct there is field `x`, can we rename it to `y`?";
        let Understanding::Readings(readings) = english.understand(text) else {
            panic!("could not read {text:?}");
        };
        assert!(readings.iter().any(|r| r.meaning == meaning));
        // "Seat's field `x`": the same member, its owner named before "'s".
        let owned = Utterance::Command(Action::Rename {
            thing: Thing::Fitting(Description {
                kind: None,
                conditions: vec![
                    Condition::Is(Thing::Named(Name("x".into()))),
                    Condition::Related {
                        relation: RustRelation::Member,
                        position: Position::Second,
                        other: Thing::Named(Name("Seat".into())),
                    },
                ],
            }),
            kind: Some(RustKind::Field),
            name: Some(Name("y".into())),
        });
        let text = "rename Seat's field `x` to `y`";
        let Understanding::Readings(readings) = english.understand(text) else {
            panic!("could not read {text:?}");
        };
        assert!(readings.iter().any(|r| r.meaning == owned));
    }

    /// A thing mentioned by its owner can be what a verb's subject is, and
    /// the question word can stand where the object goes: these all ask
    /// which functions the method `x` of `Rack` calls.
    #[test]
    fn an_owned_thing_can_be_asked_what_it_calls() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meaning = Utterance::Ask(Question::Identify(Description {
            kind: Some(RustKind::Function),
            conditions: vec![Condition::Related {
                relation: RustRelation::Calls,
                position: Position::Second,
                other: Thing::Fitting(Description {
                    kind: None,
                    conditions: vec![
                        Condition::Is(Thing::Named(Name("x_y".into()))),
                        Condition::Related {
                            relation: RustRelation::Member,
                            position: Position::Second,
                            other: Thing::Named(Name("Rack".into())),
                        },
                    ],
                }),
            }],
        }));
        for text in [
            "which functions does Rack's `x_y` call internally?",
            "Rack's x_y calls what fucntions",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().any(|r| r.meaning == meaning), "{text:?}");
        }
    }

    /// A part named after its whole, with no word for how it belongs
    /// ("`S` `x`", "S's x"), is the thing that is `x` and a part of `S`.
    #[test]
    fn a_part_after_its_whole_is_part_of_it() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meaning = Utterance::Command(Action::Rename {
            thing: Thing::Fitting(Description {
                kind: None,
                conditions: vec![
                    Condition::Is(Thing::Named(Name("x_y".into()))),
                    Condition::PartOf(Thing::Named(Name("Rack".into()))),
                ],
            }),
            kind: None,
            name: Some(Name("z_w".into())),
        });
        for text in [
            "rename Rack x_y to z_w",
            "Rack `x_y` -> `z_w` plz",
            "rename Rack x_y to something better, like z_w",
            "rename Rack x_y to z_w? and update the places using it too please",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().any(|r| r.meaning == meaning), "{text:?}");
        }
        // A capitalized owner's "'s" is split off by the tokenizer and read
        // as the owner's member.
        let owned = Utterance::Command(Action::Rename {
            thing: Thing::Fitting(Description {
                kind: None,
                conditions: vec![
                    Condition::Is(Thing::Named(Name("x_y".into()))),
                    Condition::Related {
                        relation: RustRelation::Member,
                        position: Position::Second,
                        other: Thing::Named(Name("Rack".into())),
                    },
                ],
            }),
            kind: None,
            name: Some(Name("z_w".into())),
        });
        let text = "change Rack's x_y to z_w everywhere";
        let Understanding::Readings(readings) = english.understand(text) else {
            panic!("could not read {text:?}");
        };
        assert!(readings.iter().any(|r| r.meaning == owned), "{text:?}");
    }

    /// The ways of asking for a field to be renamed mean what the system
    /// says when it asks that, whether the field is named in the request or
    /// before it.
    #[test]
    fn ways_of_asking_for_a_field_rename_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meaning = Utterance::Command(Action::Rename {
            thing: Thing::Named(Name("scroll".into())),
            kind: Some(RustKind::Field),
            name: Some(Name("offset".into())),
        });
        let spoken = english
            .speak(Cat::Utterance, &Sem::Utterance(meaning.clone()))
            .map(|s| s.text);
        assert_eq!(
            spoken.as_deref(),
            Some("Rename the `scroll` field to `offset`")
        );
        for text in [
            "Hey, could you rename the field `scroll` to `offset`, please? Thanks!",
            "I'd like the field `scroll` renamed to `offset`. Can you do that?",
            "scroll field -> offset, rename it",
            "the field `scroll` is a bit vague. Could you rename it to `offset`?",
            "I think `scroll` field should be called offset, could you rename it?",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().any(|r| r.meaning == meaning), "{text:?}");
        }
    }

    /// The ways of asking why the system wants a change all ask only that,
    /// and the system answers with the request that asked for it.
    #[test]
    fn ways_of_asking_why_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        for text in [
            "Hmm, can you explain why you want to make that change?",
            "why does it want to change that, the change you just proposed?",
            "why do you want to change that? not sure I get the reason",
            "Why do you want to change that? Can you walk me through your reasoning?",
            "whats the reasoning behind that change",
            "why change it tho",
            "why u wnat chagne that one",
            "Why does that change need to happen?",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(
                readings.iter().all(|r| r.meaning == Utterance::AskReason),
                "{text:?}"
            );
        }
        let reason = Reply::Reason {
            turn: 0,
            action: Action::Rename {
                thing: Thing::Named(Name("tick".into())),
                kind: None,
                name: Some(Name("drain_events".into())),
            },
        };
        assert_eq!(
            english.say(&reason).map(|s| s.text).as_deref(),
            Some("Because you asked me to rename `tick` to `drain_events`.")
        );
    }

    /// The ways of taking back the last request all mean forgetting it, and
    /// the system says it has.
    #[test]
    fn ways_of_taking_back_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        for text in [
            "nvm, forget that",
            "Actually, never mind that last question. Please forget it.",
            "ok forget that one, not needed anymore",
            "Please forget that last one, thanks.",
            "scratch that",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(
                readings.iter().all(|r| r.meaning == Utterance::Forget),
                "{text:?}"
            );
        }
        assert!(english.say(&Reply::Forgotten).is_some());
    }

    /// Asking what is in a struct, for its members, or for its fields after
    /// setting it off or asking again, can all mean asking what it has.
    #[test]
    fn ways_of_asking_what_a_struct_holds_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meanings = |text: &str| match english.understand(text) {
            Understanding::Readings(readings) => readings.into_iter().map(|r| r.meaning).collect(),
            Understanding::Failed { .. } => Vec::new(),
        };
        let mut has = meanings("what does `Conv` have?");
        assert_eq!(has.len(), 1);
        // Asking again for its fields asks for the same, of that kind.
        has.extend(meanings("what fields does `Conv` have?"));
        for text in [
            "um, what's inside `Conv`?",
            "what is in Conv",
            "`Conv` - which fields?",
            "what are the members of `Conv`?",
            "Conv fields",
            "whats inside Conv, fields i mean",
            "what does `Conv` hold, field-wise",
            "what fields `Conv` has in it",
            "I'm trying to figure out what's inside Conv, like which fields the struct has?",
            "show me the struct fields of Conv",
            "whcih fileds does `Conv` hav",
            "Conv whta fields",
            "`Conv` fields, waht r they",
        ] {
            assert!(meanings(text).iter().any(|m| has.contains(m)), "{text:?}");
        }
    }

    /// A field set off by its name and referred back to is any field going
    /// by that name, as when it is said in place.
    #[test]
    fn a_field_set_off_by_name_is_asked_about_by_name() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meanings = |text: &str| match english.understand(text) {
            Understanding::Readings(readings) => readings.into_iter().map(|r| r.meaning).collect(),
            Understanding::Failed { .. } => Vec::new(),
        };
        let by_name = meanings("which structs have field credentials");
        assert!(!by_name.is_empty());
        assert!(
            meanings("field credentials, which structs have it")
                .iter()
                .any(|m| by_name.contains(m))
        );
    }

    /// The ways of asking, in a later turn, for the callers of what was
    /// said before all ask for the things that call it, whatever kind
    /// they say those things are.
    #[test]
    fn ways_of_asking_for_the_callers_of_what_was_said_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        for text in [
            "And which functions call that one?",
            "who calls that",
            "ok and which ones are they, the functions that call it?",
            "is there anything that ends up calling that function?",
            "Got it. Which functions are those that call it?",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            for reading in readings {
                let Utterance::Ask(Question::Identify(d)) = reading.meaning else {
                    panic!("{text:?} read as {:?}", reading.meaning);
                };
                assert_eq!(
                    d.conditions,
                    vec![Condition::Related {
                        relation: RustRelation::Calls,
                        position: Position::First,
                        other: Thing::Previous,
                    }],
                    "{text:?}"
                );
            }
        }
    }

    /// The ways of asking what the system holds of the conversation all
    /// mean the same, and the system can answer with or without anything
    /// to recall.
    #[test]
    fn ways_of_asking_what_is_remembered_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        for text in [
            "Could you show me what you're keeping in your context right now, i.e. what you remember of this conversation so far?",
            "What do you remember of this conversation so far? Please show me what's in your context.",
            "btw what are you holding in context rn",
            "dump what you've got in context so far",
            "can you show what you remember from this convo? like what's in your context",
            "what u keepin in ur context rn, show me",
            "show what ur keepign in context",
            "what u rememeber of this convo so far, show me ur context",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(
                readings.iter().all(|r| r.meaning == Utterance::AskRecall),
                "{text:?}"
            );
        }
        assert!(english.say(&Reply::Recall(Vec::new())).is_some());
    }

    /// The ways of telling the system what the user will call a struct all
    /// mean the same fact, whether the name is quoted or not.
    #[test]
    fn ways_of_stating_an_alias_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meaning = Utterance::Tell(Fact::Alias {
            thing: Thing::Named(Name("Page".into())),
            kind: Some(RustKind::Struct),
            alias: Name("the leaf".into()),
        });
        for text in [
            "Just so you know, I'm going to call the struct `Page` \"the leaf\" from now on.",
            "Ok so, from now on I'll refer to the `Page` struct as 'the leaf'.",
            "gonna call struct Page the leaf from here on",
            "I'll just call `Page` struct \"the leaf\", ok?",
            "`Page` struct = the leaf going forward, easier for me",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().all(|r| r.meaning == meaning), "{text:?}");
        }
    }

    /// Asking which fields are on a struct, by where they are, means the
    /// same however it is put.
    #[test]
    fn ways_of_asking_for_the_fields_on_a_struct_agree() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        let meaning = Utterance::Ask(Question::Identify(Description {
            kind: Some(RustKind::Field),
            conditions: vec![Condition::Related {
                relation: RustRelation::BelongsTo,
                position: Position::First,
                other: Thing::Fitting(Description {
                    kind: Some(RustKind::Struct),
                    conditions: vec![Condition::Named(Name("Page".into()))],
                }),
            }],
        }));
        for text in [
            "what fields are on the `Page` struct?",
            "Could you tell me which fields are defined in the `Page` struct, please?",
            "what are the fields of the `Page` struct? sorry, still new to this code",
            "what fields inside the Page struct",
            "which fields make up the `Page` struct?",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().any(|r| r.meaning == meaning), "{text:?}");
        }
    }

    /// A name the user gave a thing, quoted, after "the" or bare, stands for
    /// that thing where a name can.
    #[test]
    fn names_the_user_gave_stand_for_things() {
        let english: English<Rust> = English::new(rules(), &Switches::all_on());
        for (text, name) in [
            ("what fields does the slot have?", "the slot"),
            ("so what fields are in the mover?", "the mover"),
            ("what fields does \"the leaf\" have", "the leaf"),
            ("what fields does sun glow have", "sun glow"),
            ("what fields does the history sheet have?", "history sheet"),
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            let named = Thing::Named(Name(name.into()));
            assert!(
                readings
                    .iter()
                    .any(|r| r.meaning.things().contains(&&named)),
                "{text:?}"
            );
        }
    }
}
