//! English: tokenizer, words, grammar and generator.
//!
//! The same rules read the user's sentences into meanings and speak the
//! system's replies. This crate knows no world: the words for a world's
//! kinds and relations come from a bridge crate (such as
//! `glassmind-english-rust`) as extra rules built with [`word`].

use std::collections::BTreeSet;
use std::marker::PhantomData;

use glassmind_grammar::{ExpectedSym, Grammar, Node, Phrase, Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::interface::{Language, Next, Reading, Spoken, TokenSpan, Understanding};
use glassmind_meaning::{Ontology, Reply, Trouble};
use glassmind_rules::Switches;

pub mod aside;
pub mod asking;
pub mod change;
pub mod counting;
pub mod describe;
pub mod discourse;
pub mod having;
pub mod lexicon;
pub mod locate;
pub mod mention;
pub mod naming;
pub mod remark;
pub mod renaming;
pub mod request;
pub mod seed;
pub mod sem;
pub mod tokens;

pub use lexicon::EnglishLexicon;
pub use sem::{Cat, Number, Sem, VerbForm};
pub use tokens::{Shape, Token, tokenize};

/// Every English rule that is not about a particular world.
pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    let mut rules = seed::rules();
    rules.extend(remark::rules());
    rules.extend(request::rules());
    rules.extend(aside::rules());
    rules.extend(locate::rules());
    rules.extend(asking::rules());
    rules.extend(counting::rules());
    rules.extend(discourse::rules());
    rules.extend(mention::rules());
    rules.extend(describe::rules());
    rules.extend(having::rules());
    rules.extend(change::rules());
    rules.extend(renaming::rules());
    rules.extend(naming::rules());
    rules
}

/// A rule that reads one or more fixed words as a fixed value: how a bridge
/// crate adds the English words for a world's kinds and relations.
pub fn word<O: Ontology>(
    info: RuleInfo,
    cat: Cat,
    words: &[&'static str],
    value: Sem<O>,
) -> Rule<Cat, Sem<O>> {
    Rule::new(
        info,
        cat,
        words.iter().map(|w| Sym::Word(w)).collect(),
        S::Const(value),
    )
}

/// English, ready to read and speak.
pub struct English<O: Ontology> {
    grammar: Grammar<Cat, Sem<O>>,
    lexicon: EnglishLexicon<O>,
}

impl<O: Ontology> English<O> {
    /// English with the given rules (usually [`rules`] plus a bridge crate's
    /// words), minus those switched off.
    pub fn new(rules: Vec<Rule<Cat, Sem<O>>>, switches: &Switches) -> Self {
        let grammar = Grammar::new(Cat::Utterance, rules, switches);
        let known = grammar
            .rules()
            .iter()
            .flat_map(|r| r.rhs.iter())
            .filter_map(|s| match s {
                Sym::Word(w) if w.chars().all(char::is_lowercase) || w.contains('\'') => Some(*w),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        English {
            grammar,
            lexicon: EnglishLexicon {
                known,
                ontology: PhantomData,
            },
        }
    }

    pub fn grammar(&self) -> &Grammar<Cat, Sem<O>> {
        &self.grammar
    }

    /// Speaks a value of any category, for tests and paraphrases.
    pub fn speak(&self, cat: Cat, value: &Sem<O>) -> Option<Spoken> {
        let spoken = self.grammar.generate(&self.lexicon, cat, value)?;
        Some(Spoken {
            text: detokenize(&spoken.words),
            rules: spoken.rules,
        })
    }

    fn diagnose(&self, tokens: &[Token], reached: usize, overflow: bool) -> Trouble<O> {
        if overflow {
            return Trouble::TooAmbiguous;
        }
        match tokens.get(reached) {
            Some(t) if t.shape == Shape::Word && !self.lexicon.known.contains(t.norm.as_str()) => {
                Trouble::UnknownWord {
                    at: reached,
                    word: t.text.clone(),
                }
            }
            _ => Trouble::UnknownShape { at: reached },
        }
    }
}

impl<O: Ontology> Language<O> for English<O> {
    fn tokens(&self, text: &str) -> Vec<TokenSpan> {
        tokenize(text).iter().map(Token::span).collect()
    }

    fn understand(&self, text: &str) -> Understanding<O> {
        let tokens = tokenize(text);
        let parse = self.grammar.parse(&self.lexicon, &tokens);
        let readings: Vec<Reading<O>> = parse
            .readings
            .iter()
            .filter_map(|p| match &p.value {
                Sem::Utterance(u) => Some(Reading {
                    meaning: u.clone(),
                    rules: p.rules(),
                    parse: render(p, &tokens),
                }),
                _ => None,
            })
            .collect();
        if readings.is_empty() {
            let reached = if parse.reached == tokens.len() && !tokens.is_empty() {
                tokens.len()
            } else {
                parse.reached
            };
            Understanding::Failed {
                trouble: self.diagnose(&tokens, reached, parse.overflow),
            }
        } else {
            Understanding::Readings(readings)
        }
    }

    fn say(&self, reply: &Reply<O>) -> Option<Spoken> {
        self.speak(Cat::Reply, &Sem::Reply(reply.clone()))
    }

    fn next(&self, prefix: &str) -> Vec<Next> {
        let tokens = tokenize(prefix);
        let mut out: Vec<Next> = self
            .grammar
            .expected(&self.lexicon, &tokens)
            .into_iter()
            .filter_map(|e| match e.sym {
                ExpectedSym::Word(w) => Some(Next::Word(w.to_owned())),
                ExpectedSym::Class(lexicon::NAME) => Some(Next::Name),
                ExpectedSym::Class(lexicon::NUMBER) => Some(Next::Number),
                ExpectedSym::Class(_) => None,
            })
            .collect();
        out.sort();
        out.dedup();
        out
    }
}

/// Joins spoken words into sentences: no space before punctuation, and a
/// capital letter at the start of each sentence.
pub fn detokenize(words: &[String]) -> String {
    let mut out = String::new();
    let mut sentence_start = true;
    for w in words {
        let punct = matches!(w.as_str(), "." | "," | "?" | "!" | ":" | ";");
        if !out.is_empty() && !punct {
            out.push(' ');
        }
        if sentence_start && !punct {
            let mut chars = w.chars();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
            }
            sentence_start = false;
        } else {
            out.push_str(w);
        }
        if matches!(w.as_str(), "." | "?" | "!") {
            sentence_start = true;
        }
    }
    out
}

/// Writes a parse tree as nested rule IDs over the words they cover.
fn render<O: Ontology>(phrase: &Phrase<Sem<O>>, tokens: &[Token]) -> String {
    let inner: Vec<String> = phrase
        .children
        .iter()
        .map(|child| match child {
            Node::Token(i) => tokens
                .get(*i)
                .map_or_else(String::new, |t| format!("{:?}", t.text)),
            Node::Phrase(p) => render(p, tokens),
        })
        .collect();
    format!("{}[{}]", phrase.rule.id, inner.join(" "))
}
