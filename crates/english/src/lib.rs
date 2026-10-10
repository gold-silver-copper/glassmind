//! English: tokenizer, words, grammar and generator.
//!
//! The same rules read the user's sentences into meanings and speak the
//! system's replies. This crate knows no world: the words for a world's
//! kinds and relations come from a bridge crate (such as
//! `glassmind-english-rust`) as extra rules built with [`word`].

use std::collections::BTreeSet;
use std::marker::PhantomData;

use Cat::*;
use glassmind_grammar::{
    ExpectedSym, Grammar, Lexicon, Node, Phrase, Rule, RuleInfo, Sem as S, Sym,
};
use glassmind_meaning::interface::{Language, Next, Reading, Spoken, TokenSpan, Understanding};
use glassmind_meaning::{Ontology, Reply, Trouble};
use glassmind_rules::Switches;

pub mod alias;
pub mod around;
pub mod aside;
pub mod asking;
pub mod change;
pub mod contents;
pub mod correcting;
pub mod counting;
pub mod describe;
pub mod discourse;
pub mod forgetting;
pub mod having;
pub mod lexicon;
pub mod locate;
pub mod mention;
pub mod naming;
pub mod ones;
pub mod reason;
pub mod recall;
pub mod remark;
pub mod renaming;
pub mod request;
pub mod seed;
pub mod sem;
pub mod spelling;
pub mod tokens;
pub mod whence;
pub mod widening;

pub use lexicon::EnglishLexicon;
pub use sem::{Cat, Number, Sem, VerbForm};
pub use tokens::{Shape, Token, tokenize};

/// Every English rule that is not about a particular world.
pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    [
        seed::rules::<O> as fn() -> Vec<Rule<Cat, Sem<O>>>,
        remark::rules,
        request::rules,
        aside::rules,
        locate::rules,
        asking::rules,
        counting::rules,
        discourse::rules,
        mention::rules,
        describe::rules,
        having::rules,
        change::rules,
        renaming::rules,
        naming::rules,
        reason::rules,
        forgetting::rules,
        alias::rules,
        contents::rules,
        recall::rules,
        widening::rules,
        ones::rules,
        whence::rules,
        correcting::rules,
        around::rules,
    ]
    .into_iter()
    .flat_map(|rules| rules())
    .collect()
}

/// The value of words that carry no meaning of their own.
pub fn blank<O: Ontology>() -> Sem<O> {
    Sem::Quote(String::new())
}

/// The value of a rule's first child: for words around a phrase that keep
/// its meaning ("what you remember, please").
pub fn first<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.first().cloned()
}

/// The value of a rule's last child ("even for").
pub fn last<O: Ontology>(c: &[Sem<O>]) -> Option<Sem<O>> {
    c.last().cloned()
}

/// A rule that reads its children with `read` and speaks a value with
/// `speak`, splitting it into its children's values.
pub(crate) fn spoken<O: Ontology>(
    id: &'static str,
    round: u32,
    lhs: Cat,
    rhs: Vec<Sym<Cat>>,
    read: fn(&[Sem<O>]) -> Option<Sem<O>>,
    speak: fn(&Sem<O>) -> Option<Vec<Sem<O>>>,
) -> Rule<Cat, Sem<O>> {
    Rule::new(RuleInfo::new(id, round), lhs, rhs, S::Both(read, speak))
}

/// The words of a phrase written with spaces between them ("is or
/// contains"), as a rule's right-hand side.
pub fn phrase(words: &'static str) -> Vec<Sym<Cat>> {
    words.split(' ').map(Sym::Word).collect()
}

/// A rule that reads a fixed phrase ("is or contains") as a fixed value: how
/// a bridge crate adds the English words for a world's kinds and relations.
pub fn word<O: Ontology>(
    info: RuleInfo,
    cat: Cat,
    words: &'static str,
    value: Sem<O>,
) -> Rule<Cat, Sem<O>> {
    Rule::new(info, cat, phrase(words), S::Const(value))
}

/// English, ready to read and speak.
pub struct English<O: Ontology> {
    grammar: Grammar<Cat, Sem<O>>,
    lexicon: EnglishLexicon<O>,
}

impl<O: Ontology> English<O> {
    /// English with the given rules (usually [`rules`] plus a bridge crate's
    /// words), minus those switched off. A rule's word may be a phrase
    /// written with spaces ("where is"), which reads as its words in turn.
    pub fn new(mut rules: Vec<Rule<Cat, Sem<O>>>, switches: &Switches) -> Self {
        for rule in &mut rules {
            rule.rhs = rule
                .rhs
                .iter()
                .flat_map(|s| match s {
                    Sym::Word(words) => phrase(words),
                    s => vec![*s],
                })
                .collect();
        }
        let grammar = Grammar::new(Utterance, rules, switches);
        let known = grammar
            .rules()
            .iter()
            .flat_map(|r| r.rhs.iter())
            .filter_map(|s| match s {
                Sym::Word(w) if w.chars().all(char::is_lowercase) || w.contains('\'') => Some(*w),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        // Words that are a noun phrase by themselves ("it", "that").
        let standing = grammar
            .rules()
            .iter()
            .filter_map(|r| match (r.lhs, r.rhs.as_slice()) {
                (NounPhrase, [Sym::Word(word)]) => Some(*word),
                _ => None,
            })
            .collect();
        English {
            grammar,
            lexicon: EnglishLexicon {
                known,
                standing,
                spellings: spelling::spellings(switches),
                ontology: PhantomData,
            },
        }
    }

    /// How many words a phrase passes over as text it need not understand
    /// (the words of a remark).
    fn passed_over(&self, phrase: &Phrase<Sem<O>>) -> usize {
        let rule = self.grammar.rules().iter().find(|r| r.info == phrase.rule);
        phrase
            .children
            .iter()
            .enumerate()
            .map(|(i, child)| match child {
                Node::Phrase(p) => self.passed_over(p),
                Node::Token(_) => {
                    usize::from(rule.and_then(|r| r.rhs.get(i)) == Some(&Sym::Class(lexicon::TEXT)))
                }
            })
            .sum()
    }

    /// The tokens of `text`, as the words this grammar knows (or knows
    /// mistyped) split it.
    fn tokenize(&self, text: &str) -> Vec<Token> {
        tokenize(text, &|w| {
            self.lexicon.known.contains(w) || self.lexicon.spellings.contains_key(w)
        })
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
}

impl<O: Ontology> Language<O> for English<O> {
    fn tokens(&self, text: &str) -> Vec<TokenSpan> {
        self.tokenize(text).iter().map(Token::span).collect()
    }

    fn understand(&self, text: &str) -> Understanding<O> {
        let tokens = self.tokenize(text);
        let mut parse = self.grammar.parse(&self.lexicon, &tokens);
        // The spellings corrected, which every reading uses.
        let respelled: Vec<RuleInfo> = tokens
            .iter()
            .filter_map(|t| self.lexicon.respelled(t))
            .map(|(rule, _)| rule)
            .collect();
        // Readings that understand more of what was said come first.
        parse.readings.sort_by_cached_key(|p| self.passed_over(p));
        let readings: Vec<Reading<O>> = parse
            .readings
            .iter()
            .filter_map(|p| match &p.value {
                Sem::Utterance(u) => Some(Reading {
                    meaning: u.clone(),
                    rules: [p.rules(), respelled.clone()].concat(),
                    parse: render(p, &tokens),
                    idle: idle(p),
                }),
                _ => None,
            })
            .collect();
        if !readings.is_empty() {
            return Understanding::Readings(readings);
        }
        // Why nothing was understood: too many readings, an unknown word
        // where reading stopped, or words in a shape no rule fits.
        let at = parse.reached;
        let trouble = match tokens.get(at) {
            _ if parse.overflow => Trouble::TooAmbiguous,
            Some(t)
                if t.shape == Shape::Word && !self.lexicon.known.contains(self.lexicon.norm(t)) =>
            {
                Trouble::UnknownWord {
                    at,
                    word: t.text.clone(),
                }
            }
            _ => Trouble::UnknownShape { at },
        };
        Understanding::Failed { trouble }
    }

    fn say(&self, reply: &Reply<O>) -> Option<Spoken> {
        self.speak(Reply, &Sem::Reply(reply.clone()))
    }

    fn next(&self, prefix: &str) -> Vec<Next> {
        let tokens = self.tokenize(prefix);
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
            out.extend(
                chars
                    .next()
                    .into_iter()
                    .flat_map(char::to_uppercase)
                    .chain(chars),
            );
        } else {
            out.push_str(w);
        }
        // A sentence starts after its end mark, and punctuation before its
        // first word leaves it still to start.
        sentence_start = matches!(w.as_str(), "." | "?" | "!") || (sentence_start && punct);
    }
    out
}

/// Writes a parse tree as nested rule IDs over the words they cover.
/// How many tokens the phrase takes to mean nothing: those under its
/// outermost phrases whose value is blank.
fn idle<O: Ontology>(phrase: &Phrase<Sem<O>>) -> usize {
    if phrase.value == blank() {
        return phrase.end - phrase.start;
    }
    phrase
        .children
        .iter()
        .map(|child| match child {
            Node::Token(_) => 0,
            Node::Phrase(p) => idle(p),
        })
        .sum()
}

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
