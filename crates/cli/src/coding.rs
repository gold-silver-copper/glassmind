//! A small coding-intent fallback. It proposes typed checks and repairs;
//! the world verifies them and the mind keeps ambiguity safeguards.
use std::collections::BTreeSet;
use std::sync::OnceLock;

use glassmind_english::English;
use glassmind_meaning::interface::{Language, Next, Reading, Spoken, TokenSpan, Understanding};
use glassmind_meaning::{Action, Question, Reply, Utterance};
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
            "fix_build" => Utterance::Command(Action::Repair(RustCheck::Build)),
            "fix_tests" => Utterance::Command(Action::Repair(RustCheck::Test)),
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
        let understood = self.english.understand(text);
        if matches!(understood, Understanding::Failed { .. }) {
            let readings = self.coding(text);
            if !readings.is_empty() {
                return Understanding::Readings(readings);
            }
        }
        understood
    }
    fn guesses(&self, text: &str) -> Vec<Reading<Rust>> {
        let mut guesses = self.english.guesses(text);
        guesses.extend(self.coding(text));
        guesses
    }
    fn say(&self, reply: &Reply<Rust>) -> Option<Spoken> {
        self.english.say(reply)
    }
    fn next(&self, prefix: &str) -> Vec<Next> {
        self.english.next(prefix)
    }
}
