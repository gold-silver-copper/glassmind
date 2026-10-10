//! The English words for the Rust world's kinds and relations.
//!
//! A world never adds grammar, and the English crate knows no world; this
//! bridge adds one rule per word form, each reading as a Rust concept.

use glassmind_english::{Cat, Number, Sem, VerbForm, word};
use glassmind_grammar::Rule;
use glassmind_meaning::Position;
use glassmind_rules::{RuleInfo, SEED};
use glassmind_world_rust::{Rust, RustKind, RustRelation};

fn kind(
    id: &'static str,
    round: u32,
    form: &'static str,
    kind: RustKind,
    number: Number,
) -> Rule<Cat, Sem<Rust>> {
    word(
        RuleInfo::new(id, round),
        Cat::Noun,
        &[form],
        Sem::Kind { kind, number },
    )
}

fn verb(
    id: &'static str,
    round: u32,
    form: &'static str,
    relation: RustRelation,
    verb_form: VerbForm,
) -> Rule<Cat, Sem<Rust>> {
    word(
        RuleInfo::new(id, round),
        Cat::Verb,
        &[form],
        Sem::Relation {
            relation,
            form: verb_form,
        },
    )
}

/// Every English word for a Rust concept.
pub fn words() -> Vec<Rule<Cat, Sem<Rust>>> {
    use Number::*;
    use RustKind::*;
    use RustRelation::*;
    use VerbForm::*;
    vec![
        kind(
            "en.rust.noun.function",
            SEED,
            "function",
            Function,
            Singular,
        ),
        kind(
            "en.rust.noun.functions",
            SEED,
            "functions",
            Function,
            Plural,
        ),
        kind("en.rust.noun.fn", 2, "fn", Function, Singular),
        kind("en.rust.noun.struct", 2, "struct", Struct, Singular),
        kind("en.rust.noun.enum", 2, "enum", Enum, Singular),
        kind("en.rust.noun.trait", 2, "trait", Trait, Singular),
        kind("en.rust.noun.field", 2, "field", Field, Singular),
        verb("en.rust.verb.calls", SEED, "calls", Calls, ThirdSingular),
        verb("en.rust.verb.call", SEED, "call", Calls, Plain),
        // Short forms in terse requests: "fns called by `cut`".
        kind("en.rust.noun.fns", 2, "fns", Function, Plural),
        kind("en.rust.noun.funcs", 2, "funcs", Function, Plural),
        // "where's that func defined"
        kind("en.rust.noun.func", 3, "func", Function, Singular),
        // "which fns does `search_results` invoke"
        verb("en.rust.verb.invokes", 2, "invokes", Calls, ThirdSingular),
        verb("en.rust.verb.invoke", 2, "invoke", Calls, Plain),
        // "fns called by `parse_archives`"
        verb("en.rust.verb.called", 2, "called", Calls, Participle),
        // "which functions is `parse_request` calling"
        verb("en.rust.verb.calling", 2, "calling", Calls, Progressive),
        // "rename the `retry` method on `Chat`"
        kind("en.rust.noun.method", 5, "method", Function, Singular),
        // "2 fields belong to `Wheel`: ...", "which functions belong to `App`"
        word(
            RuleInfo::new("en.rust.verb.belongs_to", 5),
            Cat::Verb,
            &["belongs", "to"],
            Sem::Relation {
                relation: BelongsTo,
                form: ThirdSingular,
            },
        ),
        word(
            RuleInfo::new("en.rust.verb.belong_to", 5),
            Cat::Verb,
            &["belong", "to"],
            Sem::Relation {
                relation: BelongsTo,
                form: Plain,
            },
        ),
        // "`save_filter` in `App`", "`regrid` on `Scene`"
        word(
            RuleInfo::new("en.rust.preposition.on", 5),
            Cat::Preposition,
            &["on"],
            Sem::Relation {
                relation: BelongsTo,
                form: Plain,
            },
        ),
        word(
            RuleInfo::new("en.rust.preposition.of", 5),
            Cat::Preposition,
            &["of"],
            Sem::Relation {
                relation: BelongsTo,
                form: Plain,
            },
        ),
        word(
            RuleInfo::new("en.rust.preposition.in", 5),
            Cat::Preposition,
            &["in"],
            Sem::Relation {
                relation: BelongsTo,
                form: Plain,
            },
        ),
        // "callees of `dm_vector`": what a function calls; "callers": what calls it.
        word(
            RuleInfo::new("en.rust.role.callees", 2),
            Cat::RoleNoun,
            &["callees"],
            Sem::Role {
                relation: Calls,
                position: Position::Second,
            },
        ),
        word(
            RuleInfo::new("en.rust.role.callers", 2),
            Cat::RoleNoun,
            &["callers"],
            Sem::Role {
                relation: Calls,
                position: Position::First,
            },
        ),
        // "`save_viewed` caller count": the singular, as a noun modifier.
        word(
            RuleInfo::new("en.rust.role.caller", 3),
            Cat::RoleNoun,
            &["caller"],
            Sem::Role {
                relation: Calls,
                position: Position::First,
            },
        )
        .read_only(),
        // "which structs have a field whose type is or contains `Arc`"
        kind("en.rust.noun.structs", 3, "structs", Struct, Plural),
        kind("en.rust.noun.fields", 3, "fields", Field, Plural),
        verb("en.rust.verb.has", 3, "has", HasField, ThirdSingular),
        verb("en.rust.verb.have", 3, "have", HasField, Plain),
        // "which structs hold a field ...", "what structs got a field ..."
        verb("en.rust.verb.holds", 3, "holds", HasField, ThirdSingular).read_only(),
        verb("en.rust.verb.hold", 3, "hold", HasField, Plain).read_only(),
        verb("en.rust.verb.got", 3, "got", HasField, Plain).read_only(),
        // "find structs that contain a RepoCommit field"
        verb("en.rust.verb.contain", 3, "contain", HasField, Plain).read_only(),
        // "which structs keep a `LineCache` in a field", "store a `Shown` in
        // a field", "carry `KeyModifiers` in a field"
        verb("en.rust.verb.keep", 3, "keep", HasField, Plain).read_only(),
        verb("en.rust.verb.store", 3, "store", HasField, Plain).read_only(),
        verb("en.rust.verb.carry", 3, "carry", HasField, Plain).read_only(),
        // "which structs define a field named `order`", "structs containing
        // field `title`", "any structs holding a `jobs` field"
        verb("en.rust.verb.define", 5, "define", HasField, Plain).read_only(),
        verb(
            "en.rust.verb.containing",
            5,
            "containing",
            HasField,
            Progressive,
        )
        .read_only(),
        verb("en.rust.verb.holding", 5, "holding", HasField, Progressive).read_only(),
        // "structs with a field of type `Notice`", "structs w/ a field ..."
        word(
            RuleInfo::new("en.rust.preposition.with", 3),
            Cat::Preposition,
            &["with"],
            Sem::Relation {
                relation: HasField,
                form: Plain,
            },
        ),
        word(
            RuleInfo::new("en.rust.preposition.w", 3),
            Cat::Preposition,
            &["w", "/"],
            Sem::Relation {
                relation: HasField,
                form: Plain,
            },
        ),
        // A field's type: "whose type is `Arc`", "of type `Tab`", "typed `Art`".
        word(
            RuleInfo::new("en.rust.role.type", 3),
            Cat::RoleNoun,
            &["type"],
            Sem::Role {
                relation: TypeMentions,
                position: Position::Second,
            },
        ),
        // A field holds what its type mentions: "a `Gate` field", "a field
        // that is `Key`", "fields containing `Signature`".
        word(
            RuleInfo::new("en.rust.holder.field", 3),
            Cat::Holder,
            &["field"],
            Sem::Holder {
                kind: Field,
                relation: TypeMentions,
                position: Position::Second,
            },
        ),
        word(
            RuleInfo::new("en.rust.holder.fields", 3),
            Cat::Holder,
            &["fields"],
            Sem::Holder {
                kind: Field,
                relation: TypeMentions,
                position: Position::Second,
            },
        ),
        word(
            RuleInfo::new("en.rust.role.typed", 3),
            Cat::RoleAdjective,
            &["typed"],
            Sem::Role {
                relation: TypeMentions,
                position: Position::Second,
            },
        ),
        // A field or method belongs to its type: "the field `x` of `S`",
        // "`x` on the `S` struct", "`x` in `S`". The thing before the word
        // is the member.
        belonging("en.rust.belonging.of", "of"),
        belonging("en.rust.belonging.on", "on"),
        belonging("en.rust.belonging.in", "in"),
        // "Config field saved_max_mb": a field is part of its struct.
        word(
            RuleInfo::new("en.rust.part.field", 5),
            Cat::Part,
            &["field"],
            Sem::Holder {
                kind: Field,
                relation: HasField,
                position: Position::Second,
            },
        ),
    ]
}

fn belonging(id: &'static str, form: &'static str) -> Rule<Cat, Sem<Rust>> {
    word(
        RuleInfo::new(id, 5),
        Cat::Belonging,
        &[form],
        Sem::Role {
            relation: RustRelation::Member,
            position: Position::Second,
        },
    )
}

/// All English rules for talking about Rust: the language's own and the words for the world.
pub fn rules() -> Vec<Rule<Cat, Sem<Rust>>> {
    let mut rules = glassmind_english::rules();
    rules.extend(words());
    rules
}

#[cfg(test)]
mod tests {
    use super::*;
    use glassmind_english::English;
    use glassmind_meaning::interface::{Language, Understanding};
    use glassmind_meaning::{
        Action, Condition, Description, Name, Position, Question, Thing, Utterance,
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
        ] {
            assert_eq!(meanings(text), plain, "{text:?}");
        }
        assert_eq!(
            meanings("callees of `cut`"),
            meanings("what does `cut` call?")
        );
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
        ] {
            assert_eq!(meanings(text), plain, "{text:?}");
        }
        let by_role = meanings("number of callers of `cut`");
        assert_eq!(by_role.len(), 1);
        for text in [
            "how many callers does `cut` have",
            "cut — how many callers",
            "cut caller count",
        ] {
            assert_eq!(meanings(text), by_role, "{text:?}");
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
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().all(|r| r.meaning == meaning), "{text:?}");
        }
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
            "is there any struct with a field named `done`? which ones have it",
            "which structs have field named done? still finding my way around here",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().any(|r| r.meaning == meaning), "{text:?}");
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
            name: Name("drain_events".into()),
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
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().all(|r| r.meaning == meaning), "{text:?}");
        }
        // Words after the request without a break may be part of it.
        assert!(matches!(
            english.understand("rename tick to drain_events tick is vague"),
            Understanding::Failed { .. }
        ));
    }

    /// The ways of asking to rename a field of a struct can all mean the
    /// same: the field that is `x` and a member of the struct that is `S`
    /// ("the `x` field" may also be a field of type `x`, which the world
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
            name: Name("y".into()),
        });
        for text in [
            "rename the field `x` of the `S` struct to `y`",
            "Could you rename the `x` field on the `S` struct to `y`, please?",
            "the field `x` in the `S` struct is confusing, can you rename it to `y`?",
            "in the `S` struct there is field `x`, can we rename it to `y`?",
            "I'd like to rename the field `x` in the `S` struct to `y`. Can you do that for me?",
        ] {
            let Understanding::Readings(readings) = english.understand(text) else {
                panic!("could not read {text:?}");
            };
            assert!(readings.iter().any(|r| r.meaning == meaning), "{text:?}");
        }
    }
}
