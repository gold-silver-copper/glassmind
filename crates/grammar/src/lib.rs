//! The grammar's building blocks, shared by every language.
//!
//! A language's grammar is a list of [`Rule`]s over its own categories `C`
//! and its own semantic values `V`. This crate knows no language and no
//! world. It provides three things over such a list:
//!
//! - [`Grammar::parse`]: an Earley parser that reads tokens left to right and
//!   builds semantic values as it goes. It keeps every reading, reports how
//!   far it got when it gets stuck, and works on unfinished input.
//! - [`Grammar::expected`]: what may come next after an unfinished input,
//!   which powers Tab completion.
//! - [`Grammar::generate`]: the same rules run backwards, from a value to
//!   words.
//!
//! The building blocks are fixed: a rule rewrites one category into a
//! non-empty sequence of [`Sym`]bols (categories, exact words, and token
//! classes such as names or numbers), and carries one [`Sem`]antics that says
//! how to build its value from its children's values, and, for rules that can
//! be used to speak, how to split a value back into its children's values.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

pub use glassmind_rules::{RuleInfo, SEED, Switches};

/// One symbol on the right-hand side of a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sym<C> {
    /// A phrase of this category. Contributes its value as a child.
    Cat(C),
    /// This exact word (compared with the token's normalized form).
    /// Contributes no child value.
    Word(&'static str),
    /// Any token of this class, as decided by the language's [`Lexicon`].
    /// Contributes the value the lexicon reads from the token.
    Class(&'static str),
}

/// How a rule's value relates to its children's values.
#[derive(Debug, Clone)]
pub enum Sem<V> {
    /// A fixed value, for rules without child values such as single words.
    /// Generation uses the rule when the value to express equals it.
    Const(V),
    /// Build the value from the child values (one per `Cat` and `Class`
    /// symbol, in order). Returning `None` rejects the reading.
    Build(fn(&[V]) -> Option<V>),
    /// Like `Build`, plus the inverse used to generate: split a value into
    /// the child values that would build it, or `None` if this rule cannot
    /// express it.
    Both(fn(&[V]) -> Option<V>, fn(&V) -> Option<Vec<V>>),
}

/// One grammar rule.
#[derive(Debug, Clone)]
pub struct Rule<C, V> {
    pub info: RuleInfo,
    pub lhs: C,
    pub rhs: Vec<Sym<C>>,
    pub sem: Sem<V>,
    /// Whether generation may use this rule. Rules for sloppy or rare input
    /// forms are parse-only, so the system always speaks its canonical form.
    pub speak: bool,
}

impl<C, V> Rule<C, V> {
    /// A rule that both reads and speaks.
    pub fn new(info: RuleInfo, lhs: C, rhs: Vec<Sym<C>>, sem: Sem<V>) -> Self {
        Rule {
            info,
            lhs,
            rhs,
            sem,
            speak: true,
        }
    }

    /// The same rule, used only for reading.
    pub fn read_only(mut self) -> Self {
        self.speak = false;
        self
    }
}

/// What a language tells the parser about its tokens.
pub trait Lexicon<V> {
    type Token;
    /// The token's normalized form, compared with `Sym::Word`.
    fn norm<'t>(&self, token: &'t Self::Token) -> &'t str;
    /// The values a token can have as a member of `class`; empty if it is
    /// not one.
    fn read_class(&self, class: &'static str, token: &Self::Token) -> Vec<V>;
    /// The surface text for a value of `class`, used when speaking.
    fn speak_class(&self, class: &'static str, value: &V) -> Option<String>;
}

/// A finished piece of a parse tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node<V> {
    /// The token at this index matched a word or class.
    Token(usize),
    /// A phrase built by a rule.
    Phrase(Rc<Phrase<V>>),
}

/// A phrase: which rule built it, over which tokens, with which value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Phrase<V> {
    pub rule: RuleInfo,
    pub start: usize,
    pub end: usize,
    pub value: V,
    pub children: Vec<Node<V>>,
}

impl<V> Phrase<V> {
    /// Every rule used in this phrase, outermost first.
    pub fn rules(&self) -> Vec<RuleInfo> {
        let mut out = vec![self.rule];
        for child in &self.children {
            if let Node::Phrase(p) = child {
                out.extend(p.rules());
            }
        }
        out
    }
}

/// The result of parsing a token sequence.
#[derive(Debug, Clone)]
pub struct Parse<V> {
    /// Every distinct complete reading, in a stable order.
    pub readings: Vec<Rc<Phrase<V>>>,
    /// How many tokens the parser could read before no rule could continue.
    /// Equal to the token count when the whole input was read.
    pub reached: usize,
    /// True if the parser gave up because the input had too many readings.
    pub overflow: bool,
}

/// Something that may come next after an unfinished input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expectation<V> {
    pub sym: ExpectedSym,
    /// The rule that wants it.
    pub rule: RuleInfo,
    /// The values of the rule's children already read, so the language can
    /// narrow the suggestions (after "rename", only renamable things).
    pub before: Vec<V>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpectedSym {
    Word(&'static str),
    Class(&'static str),
}

/// The tokens a generated phrase is made of, and the rules used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spoken {
    pub words: Vec<String>,
    pub rules: Vec<RuleInfo>,
}

/// Limits that keep a pathological input from using unbounded memory.
const MAX_ITEMS_PER_COLUMN: usize = 50_000;
const MAX_GENERATION_DEPTH: usize = 40;

/// A list of rules, ready to parse and generate.
pub struct Grammar<C, V> {
    rules: Vec<Rule<C, V>>,
    by_lhs: BTreeMap<C, Vec<usize>>,
    start: C,
}

/// An Earley item: a rule, how much of it is read, where it started, and
/// the children read so far.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Item<V> {
    rule: usize,
    dot: usize,
    start: usize,
    values: Vec<V>,
    nodes: Vec<Node<V>>,
}

impl<C, V> Grammar<C, V>
where
    C: Copy + Ord + Debug,
    V: Clone + Eq + Hash + Debug,
{
    /// Builds a grammar from the rules switched on in `switches`. Rules with
    /// an empty right-hand side are dropped (the parser needs every rule to
    /// read at least one token); [`Grammar::problems`] reports them.
    pub fn new(start: C, rules: Vec<Rule<C, V>>, switches: &Switches) -> Self {
        let rules: Vec<Rule<C, V>> = rules
            .into_iter()
            .filter(|r| switches.on(r.info.id) && !r.rhs.is_empty())
            .collect();
        let mut by_lhs: BTreeMap<C, Vec<usize>> = BTreeMap::new();
        for (i, rule) in rules.iter().enumerate() {
            by_lhs.entry(rule.lhs).or_default().push(i);
        }
        Grammar {
            rules,
            by_lhs,
            start,
        }
    }

    /// Problems in a rule list that would make it misbehave: duplicate IDs
    /// and empty right-hand sides.
    pub fn problems(rules: &[Rule<C, V>]) -> Vec<String> {
        let mut out: Vec<String> = glassmind_rules::duplicate_ids(rules.iter().map(|r| &r.info))
            .into_iter()
            .map(|id| format!("duplicate rule id {id}"))
            .collect();
        for rule in rules {
            if rule.rhs.is_empty() {
                out.push(format!(
                    "rule {} has an empty right-hand side",
                    rule.info.id
                ));
            }
        }
        out
    }

    pub fn rules(&self) -> &[Rule<C, V>] {
        &self.rules
    }

    pub fn start(&self) -> C {
        self.start
    }

    /// Parses `tokens` as the start category.
    pub fn parse<L: Lexicon<V>>(&self, lexicon: &L, tokens: &[L::Token]) -> Parse<V> {
        self.parse_as(self.start, lexicon, tokens)
    }

    /// Parses `tokens` as category `cat`.
    pub fn parse_as<L: Lexicon<V>>(&self, cat: C, lexicon: &L, tokens: &[L::Token]) -> Parse<V> {
        let chart = self.chart(cat, lexicon, tokens);
        let n = tokens.len();
        let mut readings = Vec::new();
        let mut seen = HashSet::new();
        if let Some(last) = chart.columns.get(n) {
            for item in last {
                let Some(rule) = self.rules.get(item.rule) else {
                    continue;
                };
                if item.start == 0
                    && item.dot == rule.rhs.len()
                    && rule.lhs == cat
                    && let Some(phrase) = self.finish(item, n)
                    && seen.insert(phrase.value.clone())
                {
                    readings.push(Rc::new(phrase));
                }
            }
        }
        Parse {
            readings,
            reached: chart.reached,
            overflow: chart.overflow,
        }
    }

    /// What may come right after `tokens`, read as the start of a `cat`.
    /// Empty if `tokens` cannot begin one.
    pub fn expected<L: Lexicon<V>>(&self, lexicon: &L, tokens: &[L::Token]) -> Vec<Expectation<V>> {
        let chart = self.chart(self.start, lexicon, tokens);
        let mut out: Vec<Expectation<V>> = Vec::new();
        let Some(column) = chart.columns.get(tokens.len()) else {
            return out;
        };
        for item in column {
            let Some(rule) = self.rules.get(item.rule) else {
                continue;
            };
            let sym = match rule.rhs.get(item.dot) {
                Some(Sym::Word(w)) => ExpectedSym::Word(w),
                Some(Sym::Class(c)) => ExpectedSym::Class(c),
                _ => continue,
            };
            let expectation = Expectation {
                sym,
                rule: rule.info,
                before: item.values.clone(),
            };
            if !out.contains(&expectation) {
                out.push(expectation);
            }
        }
        out
    }

    /// Builds the value of a completed item, if its rule accepts the children.
    fn finish(&self, item: &Item<V>, end: usize) -> Option<Phrase<V>> {
        let rule = self.rules.get(item.rule)?;
        let value = match &rule.sem {
            Sem::Const(v) => v.clone(),
            Sem::Build(build) | Sem::Both(build, _) => build(&item.values)?,
        };
        Some(Phrase {
            rule: rule.info,
            start: item.start,
            end,
            value,
            children: item.nodes.clone(),
        })
    }

    fn chart<L: Lexicon<V>>(&self, cat: C, lexicon: &L, tokens: &[L::Token]) -> Chart<V> {
        let n = tokens.len();
        let mut columns: Vec<Vec<Item<V>>> = vec![Vec::new(); n + 1];
        let mut keys: Vec<HashSet<Item<V>>> = vec![HashSet::new(); n + 1];
        let mut overflow = false;
        let mut reached = 0;

        let mut initial = Vec::new();
        for &r in self.by_lhs.get(&cat).into_iter().flatten() {
            initial.push(Item {
                rule: r,
                dot: 0,
                start: 0,
                values: Vec::new(),
                nodes: Vec::new(),
            });
        }
        for item in initial {
            push(&mut columns, &mut keys, 0, item);
        }

        for i in 0..=n {
            let mut predicted: BTreeSet<C> = BTreeSet::new();
            let mut k = 0;
            while let Some(item) = columns.get(i).and_then(|c| c.get(k)).cloned() {
                k += 1;
                if k > MAX_ITEMS_PER_COLUMN {
                    overflow = true;
                    break;
                }
                let Some(rule) = self.rules.get(item.rule) else {
                    continue;
                };
                match rule.rhs.get(item.dot) {
                    None => {
                        // Complete: advance every item at `item.start` waiting for this category.
                        let Some(phrase) = self.finish(&item, i) else {
                            continue;
                        };
                        let phrase = Rc::new(phrase);
                        let waiting: Vec<Item<V>> = columns
                            .get(item.start)
                            .into_iter()
                            .flatten()
                            .filter(|w| {
                                self.rules
                                    .get(w.rule)
                                    .and_then(|wr| wr.rhs.get(w.dot))
                                    .is_some_and(|s| *s == Sym::Cat(rule.lhs))
                            })
                            .cloned()
                            .collect();
                        for mut w in waiting {
                            w.dot += 1;
                            w.values.push(phrase.value.clone());
                            w.nodes.push(Node::Phrase(phrase.clone()));
                            push(&mut columns, &mut keys, i, w);
                        }
                    }
                    Some(Sym::Cat(c)) => {
                        if predicted.insert(*c) {
                            for &r in self.by_lhs.get(c).into_iter().flatten() {
                                let new = Item {
                                    rule: r,
                                    dot: 0,
                                    start: i,
                                    values: Vec::new(),
                                    nodes: Vec::new(),
                                };
                                push(&mut columns, &mut keys, i, new);
                            }
                        }
                    }
                    Some(Sym::Word(w)) => {
                        if let Some(token) = tokens.get(i)
                            && lexicon.norm(token) == *w
                        {
                            let mut next = item.clone();
                            next.dot += 1;
                            next.nodes.push(Node::Token(i));
                            push(&mut columns, &mut keys, i + 1, next);
                        }
                    }
                    Some(Sym::Class(class)) => {
                        if let Some(token) = tokens.get(i) {
                            for value in lexicon.read_class(class, token) {
                                let mut next = item.clone();
                                next.dot += 1;
                                next.values.push(value);
                                next.nodes.push(Node::Token(i));
                                push(&mut columns, &mut keys, i + 1, next);
                            }
                        }
                    }
                }
            }
            if columns.get(i).is_some_and(|c| !c.is_empty()) {
                reached = i;
            } else {
                break;
            }
            if overflow {
                break;
            }
        }
        Chart {
            columns,
            reached,
            overflow,
        }
    }

    /// Expresses `value` as a phrase of category `cat`, using the first
    /// speaking rule (in list order) that can. `None` if no rule can.
    pub fn generate<L: Lexicon<V>>(&self, lexicon: &L, cat: C, value: &V) -> Option<Spoken> {
        self.generate_at(lexicon, cat, value, 0)
    }

    fn generate_at<L: Lexicon<V>>(
        &self,
        lexicon: &L,
        cat: C,
        value: &V,
        depth: usize,
    ) -> Option<Spoken> {
        if depth > MAX_GENERATION_DEPTH {
            return None;
        }
        'rules: for &r in self.by_lhs.get(&cat).into_iter().flatten() {
            let Some(rule) = self.rules.get(r) else {
                continue;
            };
            if !rule.speak {
                continue;
            }
            let children = match &rule.sem {
                Sem::Const(v) if v == value => Vec::new(),
                Sem::Const(_) | Sem::Build(_) => continue,
                Sem::Both(_, split) => match split(value) {
                    Some(c) => c,
                    None => continue,
                },
            };
            let mut spoken = Spoken {
                words: Vec::new(),
                rules: vec![rule.info],
            };
            let mut child_values = children.iter();
            for sym in &rule.rhs {
                match sym {
                    Sym::Word(w) => spoken.words.push((*w).to_owned()),
                    Sym::Class(class) => {
                        let Some(v) = child_values.next() else {
                            continue 'rules;
                        };
                        let Some(text) = lexicon.speak_class(class, v) else {
                            continue 'rules;
                        };
                        spoken.words.push(text);
                    }
                    Sym::Cat(c) => {
                        let Some(v) = child_values.next() else {
                            continue 'rules;
                        };
                        let Some(sub) = self.generate_at(lexicon, *c, v, depth + 1) else {
                            continue 'rules;
                        };
                        spoken.words.extend(sub.words);
                        spoken.rules.extend(sub.rules);
                    }
                }
            }
            return Some(spoken);
        }
        None
    }
}

struct Chart<V> {
    columns: Vec<Vec<Item<V>>>,
    reached: usize,
    overflow: bool,
}

fn push<V: Clone + Eq + Hash>(
    columns: &mut [Vec<Item<V>>],
    keys: &mut [HashSet<Item<V>>],
    at: usize,
    item: Item<V>,
) {
    // Items that differ only in their parse tree are the same item: keeping
    // the first tree keeps the chart small, and the value is what matters.
    let key = Item {
        nodes: Vec::new(),
        ..item.clone()
    };
    if let (Some(column), Some(seen)) = (columns.get_mut(at), keys.get_mut(at))
        && seen.insert(key)
    {
        column.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum Cat {
        S,
        Np,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum Val {
        Name(String),
        List(Vec<String>),
    }

    struct Lex;
    impl Lexicon<Val> for Lex {
        type Token = String;
        fn norm<'t>(&self, token: &'t String) -> &'t str {
            token
        }
        fn read_class(&self, class: &'static str, token: &String) -> Vec<Val> {
            if class == "name" && token != "and" {
                vec![Val::Name(token.clone())]
            } else {
                Vec::new()
            }
        }
        fn speak_class(&self, _: &'static str, value: &Val) -> Option<String> {
            match value {
                Val::Name(n) => Some(n.clone()),
                Val::List(_) => None,
            }
        }
    }

    fn grammar() -> Grammar<Cat, Val> {
        let one = Rule::new(
            RuleInfo::new("t.one", SEED),
            Cat::Np,
            vec![Sym::Class("name")],
            Sem::Both(
                |c| match c {
                    [Val::Name(n)] => Some(Val::List(vec![n.clone()])),
                    _ => None,
                },
                |v| match v {
                    Val::List(names) if names.len() == 1 => {
                        names.first().map(|n| vec![Val::Name(n.clone())])
                    }
                    _ => None,
                },
            ),
        );
        let and = Rule::new(
            RuleInfo::new("t.and", SEED),
            Cat::Np,
            vec![Sym::Cat(Cat::Np), Sym::Word("and"), Sym::Class("name")],
            Sem::Both(
                |c| match c {
                    [Val::List(l), Val::Name(n)] => {
                        let mut l = l.clone();
                        l.push(n.clone());
                        Some(Val::List(l))
                    }
                    _ => None,
                },
                |v| match v {
                    Val::List(names) if names.len() > 1 => {
                        let (last, rest) = names.split_last()?;
                        Some(vec![Val::List(rest.to_vec()), Val::Name(last.clone())])
                    }
                    _ => None,
                },
            ),
        );
        let s = Rule::new(
            RuleInfo::new("t.s", SEED),
            Cat::S,
            vec![Sym::Word("see"), Sym::Cat(Cat::Np)],
            Sem::Both(
                |c: &[Val]| c.first().cloned(),
                |v: &Val| Some(vec![v.clone()]),
            ),
        );
        Grammar::new(Cat::S, vec![s, one, and], &Switches::all_on())
    }

    fn toks(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn parses_generates_and_round_trips() {
        let g = grammar();
        let parse = g.parse(&Lex, &toks("see a and b and c"));
        assert_eq!(parse.readings.len(), 1);
        let value = parse.readings[0].value.clone();
        let spoken = g.generate(&Lex, Cat::S, &value).unwrap();
        assert_eq!(spoken.words.join(" "), "see a and b and c");
        let again = g.parse(&Lex, &toks(&spoken.words.join(" ")));
        assert_eq!(again.readings[0].value, value);
    }

    #[test]
    fn reports_where_it_got_stuck_and_what_comes_next() {
        let g = grammar();
        let parse = g.parse(&Lex, &toks("see a and and"));
        assert!(parse.readings.is_empty());
        assert_eq!(parse.reached, 3);
        let next = g.expected(&Lex, &toks("see a"));
        assert!(next.iter().any(|e| e.sym == ExpectedSym::Word("and")));
        let switched = Grammar::new(
            Cat::S,
            grammar().rules().to_vec(),
            &Switches::without(["t.and"]),
        );
        assert!(
            switched
                .parse(&Lex, &toks("see a and b"))
                .readings
                .is_empty()
        );
    }
}
