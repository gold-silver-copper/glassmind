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
        // "which fns does `search_results` invoke"
        verb("en.rust.verb.invokes", 2, "invokes", Calls, ThirdSingular),
        verb("en.rust.verb.invoke", 2, "invoke", Calls, Plain),
        // "fns called by `parse_archives`"
        verb("en.rust.verb.called", 2, "called", Calls, Participle),
        // "which functions is `parse_request` calling"
        verb("en.rust.verb.calling", 2, "calling", Calls, Progressive),
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
    ]
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
    use glassmind_meaning::{Condition, Description, Name, Position, Question, Thing, Utterance};
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
}
