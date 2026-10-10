//! The record of how a turn was handled, for `--explain`.

use std::fmt;

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
#[derive(Debug, Clone, Default)]
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
            ..Trace::default()
        }
    }

    /// Every rule that built the chosen reading, answered it, chose it and
    /// spoke the reply, each once, in first-use order.
    pub fn rules(&self) -> Vec<RuleInfo> {
        let chosen = self.chosen.and_then(|i| self.readings.get(i));
        unique(
            chosen
                .into_iter()
                .flat_map(|r| r.rules.iter().chain(&r.world_rules))
                .chain(&self.preferences)
                .chain(&self.reply_rules),
        )
    }

    /// The trace as text.
    pub fn render(&self) -> String {
        self.to_string()
    }
}

/// Each rule once, in first-use order.
fn unique<'a>(rules: impl IntoIterator<Item = &'a RuleInfo>) -> Vec<RuleInfo> {
    let mut out = Vec::new();
    for rule in rules {
        if !out.contains(rule) {
            out.push(*rule);
        }
    }
    out
}

/// Each rule once, with its round.
fn rounds(rules: &[RuleInfo]) -> String {
    let rules: Vec<String> = unique(rules)
        .iter()
        .map(|r| format!("{} (round {})", r.id, r.round))
        .collect();
    rules.join(", ")
}

impl<O: Ontology> fmt::Display for Trace<O> {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(out, "input:  {}", self.input)?;
        let words: Vec<String> = self.tokens.iter().map(|t| format!("[{t}]")).collect();
        writeln!(out, "words:  {}", words.join(" "))?;
        if let Some(failure) = &self.failure {
            writeln!(out, "parse:  no reading: {failure:?}")?;
        }
        for (i, r) in self.readings.iter().enumerate() {
            let marker = if self.chosen == Some(i) {
                " (chosen)"
            } else {
                ""
            };
            writeln!(out, "reading {}{marker}:", i + 1)?;
            writeln!(out, "  parse:   {}", r.parse)?;
            writeln!(out, "  meaning: {:?}", r.meaning)?;
            writeln!(out, "  rules:   {}", rounds(&r.rules))?;
            for (name, items) in &r.resolutions {
                let found: Vec<String> = items.iter().map(Labelled::label).collect();
                let found = if found.is_empty() {
                    "nothing".to_owned()
                } else {
                    found.join(", ")
                };
                writeln!(out, "  name `{}` -> {found}", name.0)?;
            }
            for (meaning, result) in &r.outcomes {
                writeln!(out, "  resolved: {meaning:?}")?;
                match result {
                    Ok(reply) => writeln!(out, "  world:    {reply:?}")?,
                    Err(trouble) => writeln!(out, "  dropped:  {trouble:?}")?,
                }
            }
            if !r.world_rules.is_empty() {
                writeln!(out, "  world rules: {}", rounds(&r.world_rules))?;
            }
        }
        if !self.choice.is_empty() {
            writeln!(out, "choice: {}", self.choice)?;
        }
        if !self.preferences.is_empty() {
            writeln!(out, "preferences: {}", rounds(&self.preferences))?;
        }
        if let Some(reply) = &self.reply {
            writeln!(out, "reply:  {reply:?}")?;
        }
        writeln!(out, "spoken by: {}", rounds(&self.reply_rules))
    }
}
