//! The record of how a turn was handled, for `--explain`.

use std::fmt::Write as _;

use glassmind_meaning::interface::TokenSpan;
use glassmind_meaning::{Labelled, Name, Ontology, Reply, Trouble, Utterance};
use glassmind_rules::RuleInfo;

/// What the world made of one resolved meaning.
pub type WorldResult<O> = Result<Reply<O>, Trouble<O>>;

/// What happened to one reading.
#[derive(Debug, Clone)]
pub struct ReadingTrace<O: Ontology> {
    pub parse: String,
    pub rules: Vec<RuleInfo>,
    pub meaning: Utterance<O>,
    /// Each name and the items it was looked up as.
    pub resolutions: Vec<(Name, Vec<O::Item>)>,
    pub world_rules: Vec<RuleInfo>,
    /// Each resolved meaning and what the world made of it.
    pub outcomes: Vec<(Utterance<O>, WorldResult<O>)>,
}

/// Everything that happened in one turn.
#[derive(Debug, Clone)]
pub struct Trace<O: Ontology> {
    pub input: String,
    pub tokens: Vec<String>,
    /// Set when no reading was found.
    pub failure: Option<Trouble<O>>,
    pub readings: Vec<ReadingTrace<O>>,
    /// The index of the reading the reply came from.
    pub chosen: Option<usize>,
    pub preferences: Vec<RuleInfo>,
    pub choice: String,
    pub reply: Option<Reply<O>>,
    pub reply_rules: Vec<RuleInfo>,
}

impl<O: Ontology> Trace<O> {
    pub fn new(input: &str, tokens: &[TokenSpan]) -> Self {
        Trace {
            input: input.to_owned(),
            tokens: tokens.iter().map(|t| t.text.clone()).collect(),
            failure: None,
            readings: Vec::new(),
            chosen: None,
            preferences: Vec::new(),
            choice: String::new(),
            reply: None,
            reply_rules: Vec::new(),
        }
    }

    /// Every rule that built the chosen reading, answered it, chose it and
    /// spoke the reply, each once, in first-use order.
    pub fn rules(&self) -> Vec<RuleInfo> {
        let chosen = self.chosen.and_then(|i| self.readings.get(i));
        let all = chosen
            .into_iter()
            .flat_map(|r| r.rules.iter().chain(r.world_rules.iter()))
            .chain(self.preferences.iter())
            .chain(self.reply_rules.iter());
        let mut out: Vec<RuleInfo> = Vec::new();
        for rule in all {
            if !out.contains(rule) {
                out.push(*rule);
            }
        }
        out
    }

    /// The trace as text.
    pub fn render(&self) -> String {
        let mut out = String::new();
        let rounds = |rules: &[RuleInfo]| -> String {
            let mut seen: Vec<&RuleInfo> = Vec::new();
            for r in rules {
                if !seen.contains(&r) {
                    seen.push(r);
                }
            }
            seen.iter()
                .map(|r| format!("{} (round {})", r.id, r.round))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(out, "input:  {}", self.input);
        let words: Vec<String> = self.tokens.iter().map(|t| format!("[{t}]")).collect();
        let _ = writeln!(out, "words:  {}", words.join(" "));
        if let Some(failure) = &self.failure {
            let _ = writeln!(out, "parse:  no reading: {failure:?}");
        }
        for (i, r) in self.readings.iter().enumerate() {
            let marker = if self.chosen == Some(i) {
                " (chosen)"
            } else {
                ""
            };
            let _ = writeln!(out, "reading {}{marker}:", i + 1);
            let _ = writeln!(out, "  parse:   {}", r.parse);
            let _ = writeln!(out, "  meaning: {:?}", r.meaning);
            let _ = writeln!(out, "  rules:   {}", rounds(&r.rules));
            for (name, items) in &r.resolutions {
                let found: Vec<String> = items.iter().map(Labelled::label).collect();
                let _ = writeln!(
                    out,
                    "  name `{}` -> {}",
                    name.0,
                    if found.is_empty() {
                        "nothing".to_owned()
                    } else {
                        found.join(", ")
                    }
                );
            }
            for (meaning, result) in &r.outcomes {
                let _ = writeln!(out, "  resolved: {meaning:?}");
                match result {
                    Ok(reply) => {
                        let _ = writeln!(out, "  world:    {reply:?}");
                    }
                    Err(trouble) => {
                        let _ = writeln!(out, "  dropped:  {trouble:?}");
                    }
                }
            }
            if !r.world_rules.is_empty() {
                let _ = writeln!(out, "  world rules: {}", rounds(&r.world_rules));
            }
        }
        if !self.choice.is_empty() {
            let _ = writeln!(out, "choice: {}", self.choice);
        }
        if !self.preferences.is_empty() {
            let _ = writeln!(out, "preferences: {}", rounds(&self.preferences));
        }
        if let Some(reply) = &self.reply {
            let _ = writeln!(out, "reply:  {reply:?}");
        }
        let _ = writeln!(out, "spoken by: {}", rounds(&self.reply_rules));
        out
    }
}
