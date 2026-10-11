//! A small coding-intent fallback. It proposes typed checks and repairs;
//! the world verifies them and the mind keeps ambiguity safeguards.
use std::collections::BTreeSet;
use std::sync::OnceLock;

use glassmind_english::English;
use glassmind_english_rust::{
    rename_requests,
    repair_requests::{self, SpeechAct},
};
use glassmind_meaning::interface::{Language, Next, Reading, Spoken, TokenSpan, Understanding};
use glassmind_meaning::{Action, Question, Reply, Trouble, Utterance};
use glassmind_models::IntentModel;
use glassmind_rules::{HAND, RuleInfo, Switches};
use glassmind_world_rust::Rust;
use glassmind_world_rust::checks::RustCheck;

pub const CODING_INTENT: RuleInfo = RuleInfo::new("coding.intent.fallback", HAND);

pub struct CodingLanguage<'a> {
    pub english: &'a English<Rust>,
    pub switches: &'a Switches,
}

impl CodingLanguage<'_> {
    fn contains_rename(understood: &Understanding<Rust>) -> bool {
        matches!(understood, Understanding::Readings(rs) if rs.iter().any(|r|
            matches!(r.meaning, Utterance::Command(Action::Rename { .. }))))
    }

    fn exact(&self, r: &Reading<Rust>) -> bool {
        r.edits == 0
            && r.discarded.is_empty()
            && !r.rules.iter().any(|info| {
                self.english.grammar().rules().iter().any(|rule| {
                    rule.info == *info
                        && (matches!(
                            rule.lhs,
                            glassmind_english::Cat::Remark
                                | glassmind_english::Cat::Topic
                                | glassmind_english::Cat::Musing
                        ) || rule.rhs.iter().any(|s| {
                            matches!(
                                s,
                                glassmind_grammar::Sym::Class(glassmind_english::lexicon::TEXT)
                            )
                        }))
                })
            })
    }

    fn coding(&self, text: &str) -> Vec<Reading<Rust>> {
        if !self.switches.on(CODING_INTENT.id) {
            return Vec::new();
        }
        static MODEL: OnceLock<IntentModel> = OnceLock::new();
        let model = MODEL.get_or_init(|| IntentModel::load(include_str!("../models/coding.tsv")));
        let words: BTreeSet<String> = text
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|w| !w.is_empty())
            .map(str::to_ascii_lowercase)
            .collect();
        let ranked = model.rank(&words, 0);
        let Some(&(intent, probability)) = ranked.first() else {
            return Vec::new();
        };
        if probability < 0.98 {
            return Vec::new();
        }
        let meaning = match intent {
            "build_status" => Utterance::Ask(Question::Passes(RustCheck::Build)),
            "test_status" => Utterance::Ask(Question::Passes(RustCheck::Test)),
            "fix_build"
                if repair_requests::speech_act(text)
                    == Some(SpeechAct::Repair(RustCheck::Build)) =>
            {
                Utterance::Command(Action::Repair(RustCheck::Build))
            }
            "fix_tests"
                if repair_requests::speech_act(text)
                    == Some(SpeechAct::Repair(RustCheck::Test)) =>
            {
                Utterance::Command(Action::Repair(RustCheck::Test))
            }
            _ => return Vec::new(),
        };
        // Negated requests need a richer reading than a bag of words.
        let lower = text.to_ascii_lowercase();
        if ["don't", "do not", "never", "without"]
            .iter()
            .any(|w| lower.contains(w))
        {
            return Vec::new();
        }
        vec![Reading {
            meaning,
            rules: vec![CODING_INTENT],
            parse: format!("coding intent: {intent} ({probability:.4})"),
            idle: 0,
            edits: 250,
            discarded: Vec::new(),
        }]
    }
}

impl Language<Rust> for CodingLanguage<'_> {
    fn tokens(&self, text: &str) -> Vec<TokenSpan> {
        self.english.tokens(text)
    }
    fn understand(&self, text: &str) -> Understanding<Rust> {
        self.repair_boundary(text, self.legacy_understand(text))
    }
    fn guesses(&self, text: &str) -> Vec<Reading<Rust>> {
        let guesses = self.legacy_guesses(text);
        match self.repair_boundary(text, Understanding::Readings(guesses)) {
            Understanding::Readings(rs) => rs,
            _ => Vec::new(),
        }
    }
    fn say(&self, reply: &Reply<Rust>) -> Option<Spoken> {
        self.english.say(reply)
    }
    fn next(&self, prefix: &str) -> Vec<Next> {
        self.english.next(prefix)
    }
}

impl CodingLanguage<'_> {
    fn legacy_understand(&self, text: &str) -> Understanding<Rust> {
        let understood = self.english.understand(text);
        if let Understanding::Readings(ref rs) = understood
            && (rename_requests::legacy_compatible(text)
                || (rename_requests::legacy_directive(text)
                    && rs.iter().all(|r| {
                        self.exact(r)
                            && !matches!(
                                r.meaning,
                                Utterance::Command(Action::Rename {
                                    thing: glassmind_meaning::Thing::Previous,
                                    ..
                                })
                            )
                    })))
            && rs.iter().any(|r| {
                matches!(
                    r.meaning,
                    Utterance::Command(Action::Rename { name: Some(_), .. })
                )
            })
            && rs.iter().all(|r| {
                r.edits < 200
                    && r.discarded.is_empty()
                    && matches!(r.meaning, Utterance::Command(Action::Rename { .. }))
            })
        {
            return understood;
        }
        if self.switches.on("en.rust.rename_request")
            && let Some(action) = rename_requests::request(text)
        {
            let mut readings = vec![Reading {
                meaning: Utterance::Command(action),
                rules: vec![RuleInfo::new("en.rust.rename_request", HAND)],
                parse: "fully consumed rename clauses".into(),
                idle: 0,
                edits: 0,
                discarded: Vec::new(),
            }];
            // Retain exact old alternatives: the new parser is not authority
            // to resolve an existing grammar ambiguity.
            if let Understanding::Readings(old) = understood {
                readings.extend(old.into_iter().filter(|r| self.exact(r)));
            }
            return Understanding::Readings(readings);
        }
        // The generic grammar can find a command inside a quoted example.
        // Every rename must have passed one of the complete-request proofs
        // above, even when the lexical guard treats its words as quoted data.
        if rename_requests::concerns_rename(text) || Self::contains_rename(&understood) {
            return Understanding::Failed {
                trouble: Trouble::Unsupported,
            };
        }
        if matches!(understood, Understanding::Failed { .. }) {
            let readings = self.coding(text);
            if !readings.is_empty() {
                return Understanding::Readings(readings);
            }
        }
        understood
    }
    fn legacy_guesses(&self, text: &str) -> Vec<Reading<Rust>> {
        if rename_requests::concerns_rename(text)
            || rename_requests::request(text).is_some()
            || Self::contains_rename(&self.english.understand(text))
        {
            return Vec::new();
        }
        let mut guesses = self.english.guesses(text);
        guesses.retain(|r| !matches!(r.meaning, Utterance::Command(Action::Rename { .. })));
        guesses.extend(self.coding(text));
        guesses
    }
    fn repair_boundary(&self, text: &str, understood: Understanding<Rust>) -> Understanding<Rust> {
        let act = repair_requests::speech_act(text);
        let meaning = act.map(|a| match a {
            SpeechAct::Repair(k) => Utterance::Command(Action::Repair(k)),
            SpeechAct::Status(k) => Utterance::Ask(Question::Passes(k)),
            SpeechAct::Problems(k) => Utterance::Ask(Question::Problems(k)),
        });
        let mut rs = match understood {
            Understanding::Readings(rs) => rs,
            failed if meaning.is_none() => return failed,
            _ => Vec::new(),
        };
        rs.retain(|r| match r.meaning {
            Utterance::Command(Action::Repair(RustCheck::Build | RustCheck::Test)) => {
                meaning.as_ref() == Some(&r.meaning)
            }
            _ => true,
        });
        if let Some(meaning) = meaning {
            // A complete status/request clause establishes its kind, independent
            // of model confidence. Retain original exact readings for provenance.
            rs.retain(|r| r.meaning == meaning);
            if rs.is_empty() {
                rs.push(Reading {
                    meaning,
                    rules: vec![RuleInfo::new("en.rust.repair_speech", HAND)],
                    parse: "fully consumed repair speech act".into(),
                    idle: 0,
                    edits: 0,
                    discarded: Vec::new(),
                });
            }
        }
        if rs.is_empty() {
            Understanding::Failed {
                trouble: Trouble::Unsupported,
            }
        } else {
            Understanding::Readings(rs)
        }
    }
}

#[cfg(test)]
mod rename_tests {
    use super::*;
    use glassmind_meaning::{Condition, Name, Thing};
    use glassmind_world_rust::RustKind;

    fn key(action: &Action<Rust>) -> Option<String> {
        let Action::Rename {
            thing,
            kind,
            name: Some(Name(to)),
        } = action
        else {
            return None;
        };
        let kind = match kind {
            None => "_",
            Some(RustKind::Function) => "function",
            Some(RustKind::Field) => "field",
            Some(RustKind::Struct) => "struct",
            Some(RustKind::Enum) => "enum",
            _ => return None,
        };
        match thing {
            Thing::Named(Name(from)) => Some(format!("{kind}:{from}:{to}")),
            Thing::Fitting(d) => {
                let from = d.conditions.iter().find_map(|c| {
                    if let Condition::Called(n) = c {
                        Some(&n.0)
                    } else {
                        None
                    }
                })?;
                let owner = d.conditions.iter().find_map(|c| {
                    if let Condition::Related {
                        other: Thing::Named(n),
                        ..
                    } = c
                    {
                        Some(&n.0)
                    } else {
                        None
                    }
                })?;
                Some(format!("{kind}:{from}:{to}:{owner}"))
            }
            _ => None,
        }
    }

    #[test]
    fn frozen_authored_rename_requests_and_fallback_controls() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        let mut failures = Vec::new();
        for line in include_str!("../../../tools/coding-rename-language/cases.tsv").lines() {
            let fields: Vec<_> = line.splitn(3, '\t').collect();
            let [expected, value, text] = fields.as_slice() else {
                panic!("invalid row")
            };
            let understood = language.understand(text);
            let meanings: Vec<_> = match &understood {
                Understanding::Readings(rs) => rs.iter().map(|r| &r.meaning).collect(),
                _ => Vec::new(),
            };
            let okay = match *expected {
                "rename" => !meanings.is_empty()
                    && meanings.iter().all(
                        |m| matches!(m, Utterance::Command(a) if key(a).as_deref() == Some(*value)),
                    ),
                "repair" => {
                    // This control must preserve the baseline, including a
                    // baseline refusal. Frozen expected coverage is reported
                    // separately by the replay, never relabelled as a gain.
                    let old = english.understand(text);
                    let old = if matches!(old, Understanding::Failed { .. }) {
                        let rs = language.coding(text);
                        if rs.is_empty() {
                            old
                        } else {
                            Understanding::Readings(rs)
                        }
                    } else {
                        old
                    };
                    understood == old
                        && !meanings
                            .iter()
                            .any(|m| matches!(m, Utterance::Command(Action::Rename { .. })))
                }
                "refuse" => {
                    meanings.iter().all(|m| !matches!(m, Utterance::Command(_)))
                        && language
                            .guesses(text)
                            .iter()
                            .all(|r| !matches!(r.meaning, Utterance::Command(_)))
                }
                _ => false,
            };
            if !okay {
                failures.push(format!("{text}: {understood:?}"));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn unsupported_tails_never_escape_into_guesses() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        for text in [
            "rename decode to read_entry, I think delete all tests",
            "rename decode to read_entry, I think bananas forever",
            "rename local variable decode to read_entry",
            "rename decode to read_entry, but don't update callers",
            "rename decode to read_entry and fix the tests",
            "rename decode to read_entry. Make it return zero.",
            "Could you rename `decode to read_entry; tests failed",
            "I'd like decode renamed to read_entry if tests pass",
            "rename decode to read_entry (tests only)",
            "rename decode to read_entry everywhere except tests",
            "rename decode to read_entry; run the tests",
            "Explain \"rename decode to read_entry\"; tests are failing",
            "What does the word `rename` mean in the unit tests?",
            "The function decode reads a record, so I'd like it called read_entry instead. Could you rename it everywhere? Do not keep behavior unchanged.",
        ] {
            assert!(
                matches!(language.understand(text), Understanding::Failed { .. }),
                "{text}"
            );
            assert!(language.guesses(text).is_empty(), "{text}");
        }
    }

    #[test]
    fn legacy_rename_coverage() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        let cases: Vec<String> = serde_json::from_str(include_str!(
            "../../../tools/coding-rename-language/legacy.json"
        ))
        .unwrap();
        let mut lost = Vec::new();
        for text in cases {
            let old = english.understand(&text);
            let new = language.understand(&text);
            let complete = |u: &Understanding<Rust>| -> Vec<Utterance<Rust>> {
                match u {
                    Understanding::Readings(rs) => rs
                        .iter()
                        .filter_map(|r| {
                            matches!(
                                r.meaning,
                                Utterance::Command(Action::Rename { name: Some(_), .. })
                            )
                            .then_some(r.meaning.clone())
                        })
                        .collect(),
                    _ => vec![],
                }
            };
            if complete(&old).iter().any(|m| !complete(&new).contains(m)) {
                lost.push(text);
            }
        }
        assert!(lost.is_empty(), "lost legacy: {lost:?}");
    }

    #[test]
    fn old_exact_rename_ambiguity_and_repairs_are_retained() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        for text in [
            "rename function called to saved",
            "rename function it to saved",
            "rename function `café` to naïve",
            "rename the to field to dest_node",
            "Rename the variant `State::Ready` to `Active`",
            "Rename the enum variant `State::Ready` to `Active`",
            "rename Seat's field `x` to `y`",
            "rename Rack x_y to z_w",
            "fix the build",
            "make failing tests pass",
            "do the tests pass?",
        ] {
            let Understanding::Readings(old) = english.understand(text) else {
                panic!("old reading absent: {text}")
            };
            let Understanding::Readings(new) = language.understand(text) else {
                panic!("new reading absent: {text}")
            };
            for r in old.into_iter().filter(|r| r.edits == 0) {
                assert!(new.contains(&r), "{text}: lost {r:?}");
            }
        }
    }

    #[test]
    fn explicit_scope_and_open_description_objects_preserve_identity() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        let mut texts = Vec::new();
        for noun in ["count", "packet", "document", "token", "row"] {
            texts.push(format!("The function `read_count` parses a {noun}, so I would like it called `decode_count` instead. Could you rename it everywhere? Behaviour should stay exactly the same."));
        }
        for scope in [
            "all its references",
            "all references",
            "every reference",
            "all callers",
        ] {
            texts.push(format!(
                "Please rename function read_count to decode_count and update {scope}."
            ));
        }
        for text in texts {
            let Understanding::Readings(rs) = language.understand(&text) else {
                panic!("missing reading: {text}")
            };
            assert!(!rs.is_empty());
            assert!(rs.iter().all(|r| matches!(&r.meaning,
                Utterance::Command(a) if key(a).as_deref() == Some("function:read_count:decode_count"))), "{text}: {rs:?}");
        }
        for text in [
            "rename to decode_count",
            "rename into decode_count",
            "rename -> decode_count",
            "Please rename function read_count to decode_count and update only its definition.",
            "Please rename function read_count to decode_count and update all its references except tests.",
            "The function read_count parses a packet and deletes tests, so I'd like it called decode_count instead. Could you rename it everywhere?",
        ] {
            assert!(
                matches!(language.understand(text), Understanding::Failed { .. }),
                "{text}"
            );
            assert!(language.guesses(text).is_empty(), "{text}");
        }
        // Explicit pronouns retain the old context-dependent semantics. An
        // omitted target above cannot become an action on that same context.
        let Understanding::Readings(rs) = language.understand("rename it to decode_count") else {
            panic!("explicit pronoun lost")
        };
        assert!(rs.iter().any(|r| matches!(
            r.meaning,
            Utterance::Command(Action::Rename {
                thing: Thing::Previous,
                ..
            })
        )));
    }

    #[test]
    fn quoted_rename_examples_cannot_become_commands_or_guesses() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        for text in [
            "The guide says \"rename read_count to decode_count\". Why do its tests fail?",
            "This is an example: \"rename read_count to decode_count\". Explain it.",
            "The test output says `rename read_count to decode_count`. Please explain the message.",
            "Can you explain the instruction \"rename read_count to decode_count\" in the tests?",
            "Why does the documentation say \"rename read_count to decode_count\"?",
            "I found the words \"rename read_count to decode_count\" in a failing unit test. What do they mean?",
            "\"rename read_count to decode_count\" is an example, not a request. The tests are failing.",
            "The README says \"rename read_count to decode_count\"; do not execute that command.",
        ] {
            if let Understanding::Readings(rs) = language.understand(text) {
                assert!(
                    rs.iter()
                        .all(|r| !matches!(r.meaning, Utterance::Command(_))),
                    "{text}: {rs:?}"
                );
            }
            assert!(
                language
                    .guesses(text)
                    .iter()
                    .all(|r| !matches!(r.meaning, Utterance::Command(_))),
                "{text}"
            );
        }
    }
}

#[cfg(test)]
mod repair_speech_tests {
    use super::*;
    #[test]
    fn frozen_authority_in_understand_and_guesses() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        let mut failures = Vec::new();
        for row in include_str!("../../../tools/coding-repair-speech/cases.tsv").lines() {
            let (expected, text) = row.split_once('\t').unwrap();
            let want = match expected {
                "repair_build" => Some(Utterance::Command(Action::Repair(RustCheck::Build))),
                "repair_tests" => Some(Utterance::Command(Action::Repair(RustCheck::Test))),
                "status_build" => Some(Utterance::Ask(Question::Passes(RustCheck::Build))),
                "status_tests" => Some(Utterance::Ask(Question::Passes(RustCheck::Test))),
                "refuse" => None,
                _ => panic!("invalid authored case"),
            };
            let understood = language.understand(text);
            let rs = match understood {
                Understanding::Readings(rs) => rs,
                _ => vec![],
            };
            let guesses = language.guesses(text);
            let safe = |rs: &[Reading<Rust>]| {
                rs.iter().all(|r| {
                    !matches!(
                        r.meaning,
                        Utterance::Command(Action::Repair(RustCheck::Build | RustCheck::Test))
                    )
                })
            };
            let okay = match want {
                Some(ref m) => {
                    !rs.is_empty()
                        && rs.iter().all(|r| &r.meaning == m)
                        && guesses.iter().all(|r| &r.meaning == m)
                }
                None => safe(&rs) && safe(&guesses),
            };
            if !okay {
                failures.push(format!("{text}: {rs:?}; guesses={guesses:?}"));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
