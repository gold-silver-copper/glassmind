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
use glassmind_meaning::{Ontology, Reply, Trouble, Utterance};
use glassmind_rules::Switches;

pub mod alias;
pub mod around;
pub mod aside;
pub mod asking;
pub mod change;
pub mod checking;
pub mod contents;
pub mod correcting;
pub mod counting;
pub mod describe;
pub mod discourse;
pub mod forgetting;
pub mod having;
pub mod intents;
pub mod lexicon;
pub mod locate;
pub mod mention;
pub mod naming;
pub mod normalize;
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
pub mod typos;
pub mod whence;
pub mod widening;

pub use lexicon::EnglishLexicon;
pub use sem::{Cat, Number, Sem, VerbForm};
pub use tokens::{Shape, Token, tokenize};

/// English rules that are not grammar rules, for listing and switching.
pub const RULES: &[RuleInfo] = &[typos::TYPOS, normalize::NORMALIZER, intents::INTENTS];

/// Every English rule that is not about a particular world.
pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    [
        seed::rules::<O> as fn() -> Vec<Rule<Cat, Sem<O>>>,
        checking::rules,
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
    /// Whether inputs that do not parse are read through the normalizer.
    normalizing: bool,
    /// Whether inputs nothing else reads go to the request-category tagger.
    tagging: bool,
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
            normalizing: switches.on(normalize::NORMALIZER.id),
            tagging: switches.on(intents::INTENTS.id),
            lexicon: EnglishLexicon {
                known,
                standing,
                spellings: spelling::spellings(switches),
                typos: switches.on(typos::TYPOS.id),
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
                Node::Corrected(..) => 0,
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

    /// Readings of the input with the normalizer's edits, when it does not
    /// parse as typed: the cheapest edited inputs that parse, within one
    /// bit of the cheapest. `edits` on each reading is 100 plus ten times
    /// its cost in bits, so any reading of the input as typed (or with
    /// mistyped words corrected) is preferred.
    fn normalized(&self, tokens: &[Token]) -> Option<Vec<Reading<O>>> {
        let known = |w: &str| self.lexicon.known.contains(w);
        let mut found: Vec<Reading<O>> = Vec::new();
        let mut best: Option<f64> = None;
        for (edited, cost) in normalize::candidates(tokens, &known).into_iter().take(80) {
            if best.is_some_and(|b| cost > b + 1.0) {
                break;
            }
            let parse = self.grammar.parse(&self.lexicon, &edited);
            for p in &parse.readings {
                if let Sem::Utterance(u) = &p.value {
                    // Only a reading close enough to what was typed is trusted.
                    let said = self
                        .speak(Cat::Utterance, &p.value)
                        .map(|s| s.text)
                        .unwrap_or_default();
                    let typed: Vec<String> = tokens.iter().map(normalize::model_word).collect();
                    let canonical: Vec<String> = self
                        .tokenize(said.trim_end_matches(['?', '.']))
                        .iter()
                        .map(normalize::model_word)
                        .collect();
                    if said.is_empty() || !normalize::model().trusts(&typed, &canonical) {
                        continue;
                    }
                    best.get_or_insert(cost);
                    found.push(Reading {
                        meaning: u.clone(),
                        rules: [p.rules(), vec![normalize::NORMALIZER]].concat(),
                        parse: render(p, &edited),
                        idle: idle(p),
                        // 100 and up: read through the normalizer; 200 and
                        // up: with an edit too unlikely to answer without
                        // asking (see `normalize::SURE_EDIT_BITS`).
                        edits: if normalize::model().worst_edit(&typed, &canonical)
                            <= normalize::SURE_EDIT_BITS
                        {
                            100
                        } else {
                            200
                        } + (cost * 10.0).round() as u32
                            + p.edits,
                        discarded: tokens
                            .iter()
                            .filter(|t| {
                                !edited
                                    .iter()
                                    .any(|e| e.start == t.start && e.text == t.text)
                            })
                            .map(|t| t.text.clone())
                            .collect(),
                    });
                }
            }
        }
        (!found.is_empty()).then_some(found)
    }

    /// Readings of the input as the request the tagger finds likeliest for
    /// its words (see [`intents`]), when nothing else reads it. `edits` is
    /// 150 for a question the model is sure of and 250 otherwise (asked
    /// first), plus how unlikely the template is.
    fn tagged(&self, tokens: &[Token]) -> Option<Vec<Reading<O>>> {
        let known = |w: &str| self.lexicon.known.contains(w);
        let m = intents::model();
        let (names, words) = intents::split(tokens, &known, m);
        let mut found: Vec<Reading<O>> = Vec::new();
        // Which of the words that may be names are the names: each choice
        // is read, and the world keeps those whose names name something.
        for (chosen, discarded) in intents::choices(&names, m.most_slots()) {
            for (template, p) in m.rank(&words, chosen.len()).into_iter().take(3) {
                if p < m.offer {
                    break;
                }
                if !m.explains(template, &words) {
                    continue;
                }
                // A word that is a noun phrase by itself ("it", "that")
                // refers to something: a template without one would put a
                // name in its place and change what was asked about.
                let refers =
                    |text: &str| text.split(' ').any(|w| self.lexicon.standing.contains(w));
                if words
                    .iter()
                    .any(|w| self.lexicon.standing.contains(w.as_str()))
                    && !refers(template)
                {
                    continue;
                }
                let text = intents::fill(template, &chosen);
                let filled = self.tokenize(&text);
                let parse = self.grammar.parse(&self.lexicon, &filled);
                let before = found.len();
                for r in &parse.readings {
                    if let Sem::Utterance(u) = &r.value {
                        // Only a question is answered on the model's word
                        // alone: anything else (a change, a fact, consent)
                        // is asked first.
                        let sure = p >= m.sure && matches!(u, Utterance::Ask(_));
                        found.push(Reading {
                            meaning: u.clone(),
                            rules: [r.rules(), vec![intents::INTENTS]].concat(),
                            parse: render(r, &filled),
                            idle: idle(r),
                            edits: if sure { 150 } else { 250 }
                                + ((1.0 - p) * 40.0).round() as u32
                                + 5 * discarded.len() as u32
                                + r.edits,
                            discarded: discarded.clone(),
                        });
                    }
                }
                if found.len() > before {
                    break;
                }
            }
        }
        (!found.is_empty()).then_some(found)
    }

    /// What the tagger learns from one request answered right: the words
    /// typed, and the canonical form with the names typed numbered in its
    /// slots. `None` when the names typed and the names in the canonical
    /// form do not match up one to one.
    pub fn intent_example(
        &self,
        typed: &str,
        canonical: &str,
        m: &glassmind_models::IntentModel,
    ) -> Option<(BTreeSet<String>, String)> {
        let known = |w: &str| self.lexicon.known.contains(w);
        let (names, words) = intents::split(&self.tokenize(typed), &known, m);
        let mut used = vec![false; names.len()];
        let mut template = Vec::new();
        for t in self.tokenize(canonical.trim_end_matches(['?', '.'])) {
            let name = matches!(t.shape, Shape::Code | Shape::Number)
                || (t.shape == Shape::Word && !known(&t.norm));
            if name {
                let i = names
                    .iter()
                    .position(|n| n.text.eq_ignore_ascii_case(&t.text))?;
                if let Some(u) = used.get_mut(i) {
                    *u = true;
                }
                template.push(glassmind_models::intents::slot(i));
            } else if t.shape == Shape::Punct {
                template.push(t.text);
            } else {
                template.push(t.norm);
            }
        }
        used.iter().all(|u| *u).then(|| (words, template.join(" ")))
    }

    /// Whether the grammar knows `word` (lowercase).
    pub fn knows(&self, word: &str) -> bool {
        self.lexicon.known.contains(word)
    }

    /// The tokens of `text`, as this grammar splits it.
    pub fn tokens_of(&self, text: &str) -> Vec<Token> {
        self.tokenize(text)
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
                    rules: [
                        p.rules(),
                        respelled.clone(),
                        if p.edits > 0 {
                            vec![typos::TYPOS]
                        } else {
                            Vec::new()
                        },
                    ]
                    .concat(),
                    parse: render(p, &tokens),
                    idle: idle(p),
                    edits: p.edits,
                    discarded: Vec::new(),
                }),
                _ => None,
            })
            .collect();
        if !readings.is_empty() {
            return Understanding::Readings(readings);
        }
        if self.normalizing
            && let Some(normalized) = self.normalized(&tokens)
        {
            return Understanding::Readings(normalized);
        }
        if self.tagging
            && let Some(tagged) = self.tagged(&tokens)
        {
            return Understanding::Readings(tagged);
        }
        // Why nothing was understood: an unknown word where reading
        // stopped, or words in a shape no rule fits (or too many readings).
        let at = parse.reached;
        let trouble = match tokens.get(at) {
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
            Node::Token(_) | Node::Corrected(..) => 0,
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
            Node::Corrected(i, word) => tokens
                .get(*i)
                .map_or_else(String::new, |t| format!("{:?}->{word:?}", t.text)),
        })
        .collect();
    format!("{}[{}]", phrase.rule.id, inner.join(" "))
}
