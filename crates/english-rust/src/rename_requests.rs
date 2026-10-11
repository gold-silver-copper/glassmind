//! Bounded, fully consumed rename clauses. This helper never proposes a repair.
use glassmind_meaning::{Action, Condition, Description, Name, Position, Thing};
use glassmind_world_rust::{Rust, RustKind, RustRelation};

#[derive(Clone, Debug)]
struct Token<'a> {
    text: &'a str,
    quoted: bool,
}

fn tokens(text: &str) -> Option<Vec<Token<'_>>> {
    let mut out = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        rest = rest.trim_start();
        let Some(ch) = rest.chars().next() else { break };
        if ch == '`' || ch == '"' {
            let body = rest.get(ch.len_utf8()..)?;
            let end = body.find(ch)?;
            out.push(Token {
                text: body.get(..end)?,
                quoted: true,
            });
            rest = body.get(end + ch.len_utf8()..)?;
        } else {
            let len = if rest.starts_with("->") {
                2
            } else if ch.is_alphanumeric() || ch == '_' {
                rest.char_indices()
                    .find(|(_, c)| !(c.is_alphanumeric() || matches!(c, '_' | '#' | '\'')))
                    .map_or(rest.len(), |(i, _)| i)
            } else {
                ch.len_utf8()
            };
            out.push(Token {
                text: rest.get(..len)?,
                quoted: false,
            });
            rest = rest.get(len..)?;
        }
    }
    Some(out)
}

#[derive(Clone)]
struct Cursor<'a> {
    tokens: &'a [Token<'a>],
    at: usize,
}
impl Cursor<'_> {
    fn eat(&mut self, phrase: &str) -> bool {
        let mut at = self.at;
        for word in phrase.split_whitespace() {
            if !self
                .tokens
                .get(at)
                .is_some_and(|t| !t.quoted && t.text.eq_ignore_ascii_case(word))
            {
                return false;
            }
            at += 1;
        }
        self.at = at;
        true
    }
    fn one(&mut self, phrases: &[&str]) -> bool {
        phrases.iter().any(|p| self.eat(p))
    }
    fn id(&mut self) -> Option<Name> {
        let t = self.tokens.get(self.at)?;
        let body = t.text.strip_prefix("r#").unwrap_or(t.text);
        let mut chars = body.chars();
        if !chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
            || !chars.all(|c| c.is_alphanumeric() || c == '_')
        {
            return None;
        }
        self.at += 1;
        Some(Name(t.text.to_owned()))
    }
    fn courtesy(&mut self) {
        self.one(&["please", "pls"]);
        self.one(&["could you", "can you", "would you", "can we", "i'd like to"]);
        self.one(&["please", "pls"]);
    }
    fn target(&mut self) -> Option<(Thing<Rust>, Option<RustKind>)> {
        self.eat("the");
        let kind = if self.one(&["function", "method", "fn"]) {
            Some(RustKind::Function)
        } else if self.eat("field") {
            Some(RustKind::Field)
        } else if self.eat("struct") {
            Some(RustKind::Struct)
        } else if self.eat("enum") {
            Some(RustKind::Enum)
        } else {
            None
        };
        if kind.is_none()
            && self.tokens.get(self.at).is_some_and(|t| {
                !t.quoted
                    && ["it", "that", "this"]
                        .iter()
                        .any(|w| t.text.eq_ignore_ascii_case(w))
            })
        {
            return None;
        }
        let name = self.id()?;
        let thing = if self.one(&["on", "of", "in"]) {
            if !matches!(kind, Some(RustKind::Function | RustKind::Field)) {
                return None;
            }
            let owner = self.id()?;
            Thing::Fitting(Description {
                kind,
                conditions: vec![
                    Condition::Called(name),
                    Condition::Related {
                        relation: RustRelation::BelongsTo,
                        position: Position::First,
                        other: Thing::Named(owner),
                    },
                ],
            })
        } else {
            Thing::Named(name)
        };
        Some((thing, kind))
    }
    fn noun_phrase(&mut self, context: bool) -> bool {
        self.one(&["a", "an", "the", "this"]);
        self.one(&["little", "small", "whole", "complete", "single"]);
        self.one(&["log", "config", "data", "text", "event"]);
        if context {
            self.one(&["parser", "reader", "decoder", "writer", "module", "project"])
        } else if self.tokens.get(self.at).is_some_and(|t| {
            !t.quoted && !t.text.is_empty() && t.text.chars().all(char::is_alphabetic)
        }) {
            // The descriptive object is one lexical noun, not an operation
            // or a free-text tail. The caller still requires the comma and
            // the complete, explicit naming request immediately afterward.
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn member(&mut self) -> bool {
        self.eat("the");
        if self.one(&[
            "definition",
            "callers",
            "call sites",
            "unit tests",
            "tests",
            "main",
        ]) {
            return true;
        }
        // A named code component is an example within already explicit global
        // coverage, never a request to narrow the rename to that component.
        self.id().is_some() && self.eat("code")
    }
    fn members(&mut self) -> bool {
        if !self.member() {
            return false;
        }
        while self.one(&[",", "+", "and"]) {
            self.eat("and");
            if !self.member() {
                return false;
            }
        }
        true
    }
    fn coverage(&mut self) -> bool {
        if !self.eat("everywhere") {
            return false;
        }
        self.eat("it's used");
        self.one(&["please", "pls"]);
        if self.eat("(") {
            if !self.members() || !self.eat(")") {
                return false;
            }
        } else if self.eat(",") {
            if !(self.eat("including") || self.eat("all")) || !self.members() {
                return false;
            }
            self.eat("too");
        }
        true
    }
    fn preserve(&mut self) -> bool {
        self.one(&[
            "preserve behavior",
            "preserve behaviour",
            "keep tests",
            "keep the tests",
            "behavior should stay exactly the same",
            "behaviour should stay exactly the same",
            "keep behavior unchanged",
            "keep behaviour unchanged",
        ])
    }
    fn finish(&mut self) -> bool {
        self.one(&["please", "pls"]);
        if self
            .tokens
            .get(self.at)
            .is_some_and(|t| t.text.eq_ignore_ascii_case("everywhere"))
            && !self.coverage()
        {
            return false;
        }
        self.one(&[
            "and update all its references",
            "and update all references",
            "and update every reference",
            "and update all callers",
        ]);
        self.one(&["please", "pls"]);
        while self.one(&[";", ".", "?"]) {
            if self.at == self.tokens.len() {
                return true;
            }
            if !self.preserve() {
                return false;
            }
        }
        self.at == self.tokens.len()
    }
}

fn parse(tokens: &[Token<'_>]) -> Option<Action<Rust>> {
    let mut c = Cursor { tokens, at: 0 };
    if c.one(&["hey", "hi", "hello"]) && !c.one(&["!", ","]) {
        return None;
    }
    c.courtesy();
    let (thing, kind, name) = if c.eat("rename") {
        let (thing, kind) = c.target()?;
        if !c.one(&["to", "->"]) {
            return None;
        }
        (thing, kind, c.id()?)
    } else {
        if c.eat("in") && (!c.noun_phrase(true) || !c.eat(",")) {
            return None;
        }
        let (thing, kind) = c.target()?;
        if kind != Some(RustKind::Function) {
            return None;
        }
        c.eat("really");
        if !c.one(&["parses", "reads", "processes", "decodes", "writes"])
            || !c.noun_phrase(false)
            || !c.eat(",")
        {
            return None;
        }
        c.eat("so");
        if !c.one(&["i'd like it called", "i would like it called"]) {
            return None;
        }
        let name = c.id()?;
        if !c.eat("instead .") {
            return None;
        }
        c.courtesy();
        if !c.eat("rename it") {
            return None;
        }
        (thing, kind, name)
    };
    if !c.finish() {
        return None;
    }
    Some(Action::Rename {
        thing,
        kind,
        name: Some(name),
    })
}

/// Whether rename language is incompatible with an unrelated repair fallback.
/// Quoted examples are fenced only inside an explicit diagnostic request;
/// names containing rename, and mentions in a repair, are not rename commands.
/// This function never supplies identifiers for an action.
pub fn concerns_rename(text: &str) -> bool {
    let parsed = tokens(text).or_else(|| tokens(text.split(['`', '"']).next().unwrap_or_default()));
    let Some(tokens) = parsed else {
        return false;
    };
    let mut opening = Cursor {
        tokens: &tokens,
        at: 0,
    };
    opening.courtesy();
    let diagnostic = opening.one(&[
        "explain",
        "what does",
        "what would",
        "check whether",
        "tell me about",
        "suppose",
    ]);
    if diagnostic
        && tokens.iter().any(|t| {
            t.quoted && {
                let mut words = t.text.split_whitespace();
                words.next().is_some_and(|w| {
                    ["rename", "renamed", "renaming"]
                        .iter()
                        .any(|v| w.eq_ignore_ascii_case(v))
                })
            }
        })
    {
        return true;
    }
    tokens.iter().enumerate().any(|(i, t)| {
        !t.quoted
            && ["rename", "renaming", "renamed"]
                .iter()
                .any(|w| t.text.eq_ignore_ascii_case(w))
            && !i
                .checked_sub(1)
                .and_then(|p| tokens.get(p))
                .is_some_and(|p| {
                    [
                        "named",
                        "function",
                        "method",
                        "field",
                        "test",
                        "word",
                        "mentions",
                        "mentioning",
                    ]
                    .iter()
                    .any(|w| p.text.eq_ignore_ascii_case(w))
                })
    })
}

/// A typed action is returned only after every token has a bounded role.
pub fn request(text: &str) -> Option<Action<Rust>> {
    parse(&tokens(text)?)
}

// The legacy grammar has productive free-text remarks. Keep its readings only
// when a bounded description accounts for those remarks as well. This validator
// supplies no action or names: ambiguities remain with the original grammar.
fn legacy_name(c: &mut Cursor<'_>) -> bool {
    c.eat("the");
    c.one(&["function", "method", "field", "struct", "enum"]);
    if !(c.eat("it") || c.id().is_some()) {
        return false;
    }
    c.one(&["function", "method", "field", "struct", "enum"]);
    if c.one(&["on", "of", "in"]) {
        c.eat("the");
        c.one(&["struct", "enum"]);
        if c.id().is_none() {
            return false;
        }
        c.one(&["struct", "enum"]);
    }
    true
}
fn judgment(c: &mut Cursor<'_>) -> bool {
    c.one(&["a bit", "kind of", "kinda"]);
    c.one(&["vague", "cryptic", "unclear"])
}
fn legacy_tail(c: &mut Cursor<'_>) -> bool {
    c.one(&["please", "pls"]);
    c.one(&["everywhere", "evrywhere"]);
    c.one(&["please", "pls"]);
    if c.at == c.tokens.len() {
        return true;
    }
    if !c.one(&[",", "?", "."]) {
        return false;
    }
    if c.at == c.tokens.len() {
        return true;
    }
    if c.one(&["please", "pls", "thanks", "thx"]) {
        c.one(&["?", "!", "."]);
        if c.one(&["thanks", "thx"]) {
            c.one(&["!", "."]);
        }
    } else if c.one(&[
        "could you do that for me",
        "can you do that",
        "could you rename it",
        "rename it",
    ]) {
        c.one(&["?", "."]);
    } else if c.one(&["cleaner", "clearer", "shorter"]) {
        c.eat(".");
    } else {
        let saved = c.at;
        c.eat("the");
        if c.eat("current name") {
            c.eat("is");
            if !judgment(c) {
                return false;
            }
        } else {
            c.at = saved;
            if c.eat("i think")
                && c.one(&["that's", "that is"])
                && c.one(&["better", "clearer"])
                && c.eat("for the")
                && c.one(&["] and [", "[ and ]", "( and )", ") and ("])
                && c.one(&["thing", "part"])
            {
                // A bounded explanatory comparison already understood by the
                // old grammar; brackets are meaningful here, never stripped.
            } else if !legacy_name(c) || !c.eat("is") || !judgment(c) {
                return false;
            }
        }
        c.eat(".");
    }
    c.at == c.tokens.len()
}
fn legacy_destination(c: &mut Cursor<'_>) -> bool {
    if c.one(&["something clearer", "something better", "smth better"]) {
        c.one(&["please", "pls"]);
        if !c.one(&[",", "?"]) {
            return false;
        }
        c.one(&["maybe", "like"]);
    }
    c.id().is_some()
}
fn legacy_command(c: &mut Cursor<'_>, topic: bool) -> bool {
    c.courtesy();
    if !c.eat("rename") {
        return false;
    }
    if topic && c.eat("it") {
        // Source already bound by the fully consumed topic description.
    } else if !legacy_name(c) {
        return false;
    }
    if c.one(&["to", "into", "->"]) {
        if !legacy_destination(c) {
            return false;
        }
    } else if c.eat(",") && c.one(&["call it", "want it called"]) {
        if c.id().is_none() {
            return false;
        }
        c.one(&["instead", "isntead"]);
    } else {
        return false;
    }
    legacy_tail(c)
}

/// Proof that an old rename reading's surrounding text is within bounded
/// descriptive/courtesy forms. Never accepts unknown sentence tails.
pub fn legacy_compatible(text: &str) -> bool {
    let Some(ts) = tokens(text) else {
        return false;
    };
    let mut c = Cursor { tokens: &ts, at: 0 };
    if c.one(&["hey", "hi", "hello"]) && !c.one(&[",", "!"]) {
        return false;
    }
    if legacy_command(&mut c.clone(), false) {
        return true;
    }
    let mut passive = c.clone();
    if passive.one(&["i'd like", "i would like"])
        && legacy_name(&mut passive)
        && passive.eat("renamed to")
        && passive.id().is_some()
        && legacy_tail(&mut passive)
    {
        return true;
    }
    // Topic with a naming judgment, or a name-to-name arrow followed by an
    // explicit rename instruction. These preserve the old unresolved readings.
    c.eat("i think");
    if legacy_name(&mut c) {
        if c.eat("is") && judgment(&mut c) && c.one(&[".", ","]) {
            return legacy_command(&mut c, true);
        }
        if c.one(&["should be", "shoud be"]) {
            c.eat("called");
            if c.id().is_none() || !c.eat(",") {
                return false;
            }
            c.courtesy();
            if !c.eat("rename") {
                return false;
            }
            c.eat("it");
            c.one(&["please", "pls"]);
            c.one(&["?", "."]);
            return c.at == ts.len();
        }
        if c.eat("->") && c.id().is_some() {
            c.one(&["please", "pls"]);
            c.eat(",");
            if !c.eat("rename") {
                return false;
            }
            c.eat("it");
            c.one(&["everywhere", "evrywhere"]);
            c.one(&["?", "."]);
            return c.at == ts.len();
        }
    }
    let mut c = Cursor { tokens: &ts, at: 0 };
    if c.eat("so there's") && c.id().is_some() && c.eat("too .") {
        return legacy_command(&mut c, true);
    }
    // Observational context: a present-tense, intransitive description is
    // grammatically distinct from a second requested operation.
    let mut c = Cursor { tokens: &ts, at: 0 };
    if c.eat("i'm looking at") && c.id().is_some() && c.eat(". . . seems it") {
        let verb = c.tokens.get(c.at);
        if !verb.is_some_and(|t| {
            !t.quoted && t.text.ends_with('s') && t.text.chars().all(char::is_alphabetic)
        }) {
            return false;
        }
        c.at += 1;
        if !c.eat(", nothing") {
            return false;
        }
        let verb = c.tokens.get(c.at);
        if !verb.is_some_and(|t| {
            !t.quoted && t.text.ends_with('s') && t.text.chars().all(char::is_alphabetic)
        }) {
            return false;
        }
        c.at += 1;
        if !c.eat("really ?") {
            return false;
        }
        return legacy_command(&mut c, true);
    }
    false
}

/// Recognize a direct legacy imperative. This is only a routing guard: callers
/// must still demand an exact, fully consumed old grammar reading with no TEXT
/// constituents. Unknown target kinds must not be widened by noun synonyms.
pub fn legacy_directive(text: &str) -> bool {
    let Some(ts) = tokens(text) else {
        return false;
    };
    let mut c = Cursor { tokens: &ts, at: 0 };
    c.courtesy();
    if !c.eat("rename") {
        return false;
    }
    c.eat("the");
    !c.one(&[
        "local variable",
        "variable",
        "parameter",
        "module",
        "constant",
        "trait",
        "type alias",
    ])
}
