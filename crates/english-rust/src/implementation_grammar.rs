//! Consumed action grammar. Quoted names and qualified references are single roles.
use glassmind_meaning::{Name, Thing};
use glassmind_world_rust::{Rust, implementation::Implement};

#[derive(Clone, Debug)]
enum Target {
    Named(String),
    Previous,
}
impl Target {
    fn thing(self) -> Thing<Rust> {
        match self {
            Self::Named(n) => Thing::Named(Name(n)),
            Self::Previous => Thing::Previous,
        }
    }
}

#[derive(Clone, Debug)]
enum Token<'a> {
    Word(&'a str),
    Quoted(&'a str),
    Reference(&'a str),
    Mark(char),
}
fn lex(text: &str) -> Option<Vec<Token<'_>>> {
    let mut rest = text;
    let mut out = Vec::new();
    while !rest.trim_start().is_empty() {
        rest = rest.trim_start();
        let ch = rest.chars().next()?;
        if matches!(ch, '`' | '\'' | '"') {
            let inside = rest.get(ch.len_utf8()..)?;
            let end = inside.find(ch)?;
            out.push(Token::Quoted(inside.get(..end)?));
            rest = inside.get(end + ch.len_utf8()..)?;
        } else if ch.is_alphanumeric() || ch == '_' {
            let end = rest
                .find(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | ':' | '/' | '.' | '\'')))
                .unwrap_or(rest.len());
            let raw = rest.get(..end)?;
            let word = raw.trim_end_matches('.');
            if word.contains([':', '/', '.']) {
                out.push(Token::Reference(word));
            } else {
                out.push(Token::Word(word));
            }
            for _ in word.len()..raw.len() {
                out.push(Token::Mark('.'));
            }
            rest = rest.get(end..)?;
        } else {
            out.push(Token::Mark(ch));
            rest = rest.get(ch.len_utf8()..)?;
        }
    }
    Some(out)
}
#[derive(Clone)]
struct Cursor<'a> {
    tokens: &'a [Token<'a>],
    at: usize,
}
impl<'a> Cursor<'a> {
    fn eat(&mut self, phrase: &str) -> bool {
        let Some(want) = lex(phrase) else {
            return false;
        };
        let mut trial = self.clone();
        for token in want {
            let Some(actual) = trial.tokens.get(trial.at) else {
                return false;
            };
            let same = match (&token, actual) {
                (Token::Word(a), Token::Word(b)) => a.eq_ignore_ascii_case(b),
                (Token::Reference(a), Token::Reference(b)) => a == b,
                (Token::Quoted(a), Token::Quoted(b)) => a == b,
                (Token::Mark(a), Token::Mark(b)) => a == b,
                _ => false,
            };
            if !same {
                return false;
            }
            trial.at += 1;
        }
        *self = trial;
        true
    }
    fn one(&mut self, choices: &[&str]) -> bool {
        choices.iter().any(|s| self.eat(s))
    }
    fn target(&mut self) -> Option<String> {
        let text = match self.tokens.get(self.at)? {
            Token::Word(s) | Token::Quoted(s) | Token::Reference(s) => *s,
            _ => return None,
        };
        if text.is_empty() {
            return None;
        }
        self.at += 1;
        Some(text.to_owned())
    }
    fn location(&mut self) -> bool {
        !self.eat("in") || self.eat("src/lib.rs")
    }
    fn polite(&mut self) {
        self.one(&["please", "pls"]);
        self.one(&["could you", "would you", "can you"]);
        self.one(&["please", "pls"]);
    }
    fn greeting(&mut self) {
        self.one(&["hi there,", "hello!", "hi!", "hey!", "hey,"]);
    }
    fn finish(&mut self) -> bool {
        if self.one(&["per", "following"]) {
            self.one(&["the", "its"]);
            if !self.eat("doc comment") {
                return false;
            }
        } else {
            self.eat("so it does what its doc comment says");
        }
        self.one(&["please", "pls"]);
        self.one(&[".", "?"]);
        self.at == self.tokens.len()
    }
}
fn valid_target(name: &str) -> bool {
    name.starts_with(|c: char| c == '_' || c.is_alphabetic())
        && name.chars().all(|c| c == '_' || c.is_alphanumeric())
}
fn imperative(c: &mut Cursor<'_>, bound: Option<&str>) -> Option<Target> {
    c.polite();
    if c.eat("fill it in") {
        return Some(bound.map_or(Target::Previous, |n| Target::Named(n.to_owned())));
    }
    if !(c.eat("implement")
        || c.eat("fill in")
        || (c.eat("write") && {
            c.eat("the body of");
            true
        }))
    {
        return None;
    }
    let target = if c.eat("it") {
        bound.map_or(Target::Previous, |n| Target::Named(n.to_owned()))
    } else if let Some(target) = c.target() {
        if bound.is_some_and(|b| b != target) {
            return None;
        }
        Target::Named(target)
    } else {
        Target::Named(bound?.to_owned())
    };
    if matches!(&target, Target::Named(n) if !valid_target(n)) || !c.location() {
        return None;
    }
    Some(target)
}
enum Declaration {
    Accepted(String),
    Unsupported,
}

fn declaration(c: &mut Cursor<'_>) -> Option<Declaration> {
    c.eat("there's a");
    let target = c.target()?;
    c.eat("function");
    if !c.location() {
        return None;
    }
    if c.one(&["needs a body", "needs implementing"]) {
        return Some(Declaration::Unsupported);
    }
    if !c.one(&["that's", "is", "="]) {
        return None;
    }
    c.eat("still");
    c.eat("just");
    c.eat("a");
    if !c.one(&["`todo!()`", "todo!()", "stub", "todo"]) {
        return None;
    }
    c.eat("still");
    Some(Declaration::Accepted(target))
}
fn action_head(c: &mut Cursor<'_>) -> bool {
    c.greeting();
    c.polite();
    c.one(&["do not", "don't", "never", "not"]);
    c.polite();
    c.one(&["implement", "write", "fill"])
}
/// None delegates to the established query/rename/field/repair routes.
fn parse_target(text: &str) -> Option<Result<Target, String>> {
    let tokens = match lex(text) {
        Some(tokens) => tokens,
        None => {
            // An unterminated target quote does not erase a preceding action.
            let end = text
                .char_indices()
                .find_map(|(i, ch)| {
                    let quoted = matches!(ch, '`' | '"')
                        || (ch == '\''
                            && (i == 0
                                || text
                                    .get(..i)
                                    .is_some_and(|s| s.ends_with(char::is_whitespace))));
                    quoted.then_some(i)
                })
                .unwrap_or(text.len());
            let prefix = text.get(..end).unwrap_or("");
            let tokens = lex(prefix)?;
            return action_head(&mut Cursor {
                tokens: &tokens,
                at: 0,
            })
            .then(|| Err("malformed implementation target".into()));
        }
    };
    let mut c = Cursor {
        tokens: &tokens,
        at: 0,
    };
    c.greeting();
    let mut direct = c.clone();
    if let Some(target) = imperative(&mut direct, None)
        && direct.finish()
    {
        return Some(Ok(target));
    }
    let mut stub = c.clone();
    let declared = declaration(&mut stub);
    if let Some(Declaration::Accepted(target)) = &declared {
        let mut body = stub.clone();
        if body.one(&[",", "."])
            && let Some(target) = imperative(&mut body, Some(target))
            && body.finish()
        {
            return Some(Ok(target));
        }
    }
    let mut passive = c.clone();
    let passive = passive.target().is_some() && {
        passive.eat("function");
        passive.one(&["should", "must", "shall"]) && {
            passive.eat("not");
            passive.eat("be") && passive.eat("implemented")
        }
    };
    // A compound is parsed from a complete action prefix. Do not scan query
    // arguments, quoted strings, or identifier lists for apparent verbs.
    let mut other = c.clone();
    other.polite();
    let other_clause = if other.eat("rename") {
        other.one(&["function", "method", "field", "struct", "enum"]);
        other.target().is_some() && other.eat("to") && other.target().is_some()
    } else if other.one(&["repair", "fix"]) {
        other.target().is_some()
    } else {
        false
    };
    let compound = other_clause
        && other.one(&["and then", "and", "then", ".", ";", ","])
        && action_head(&mut other);
    (action_head(&mut c) || declared.is_some() || passive || compound)
        .then(|| Err("unsupported or unconsumed implementation request".into()))
}

/// The standalone command requires an explicit bound name, as before.
pub fn request(text: &str) -> Option<Result<Implement, String>> {
    parse_target(text).map(|r| {
        r.and_then(|t| match t {
            Target::Named(target) => Ok(Implement { target }),
            Target::Previous => Err("implementation needs a target from task context".into()),
        })
    })
}

/// Task sessions can resolve an explicit pronoun through the shared mind.
pub fn task_request(text: &str) -> Option<Result<Thing<Rust>, String>> {
    parse_target(text).map(|r| r.map(Target::thing))
}

#[cfg(test)]
mod tests {
    use super::*;
    include!("../../../tools/coding-implementation-grammar/contracts.rs");
    #[test]
    fn function_is_a_bare_target_identifier() {
        for text in [
            "implement function",
            "please implement function.",
            "implement function in src/lib.rs",
            "implement `function`",
            "function is a stub, implement it",
        ] {
            assert_eq!(request(text).unwrap().unwrap().target, "function", "{text}");
        }
        assert!(matches!(
            request("implement function fold_values"),
            Some(Err(_))
        ));
    }
    #[test]
    fn recognized_requirements_do_not_bind_an_accepted_declaration() {
        for description in ["needs a body", "needs implementing"] {
            for action in ["", ", implement it", ". please implement fold_values"] {
                let text = format!("fold_values {description}{action}");
                assert!(matches!(request(&text), Some(Err(_))), "{text}");
            }
        }
        assert_eq!(
            request("fold_values is a stub, implement it")
                .unwrap()
                .unwrap()
                .target,
            "fold_values"
        );
    }
    #[test]
    fn lexical_roles_are_not_action_reservations() {
        for text in [
            "which functions call alpha and write?",
            "which functions call alpha, implement?",
            "rename field implement to write",
            "where is function `hello. implement x`?",
        ] {
            assert!(request(text).is_none(), "{text}");
        }
        for text in ["please don't implement `x", "REPAIR x AND IMPLEMENT y"] {
            assert!(matches!(request(text), Some(Err(_))), "{text}");
        }
        for target in ["it", "function", "not", "the"] {
            assert_eq!(
                request(&format!("implement `{target}`"))
                    .unwrap()
                    .unwrap()
                    .target,
                target
            );
        }
    }
    #[test]
    fn frozen_contracts() {
        for &(text, target) in ACCEPT {
            assert_eq!(request(text).unwrap().unwrap().target, target, "{text}");
        }
        for &text in REFUSE {
            assert!(matches!(request(text), Some(Err(_))), "{text}");
        }
        for &text in NONE {
            assert!(request(text).is_none(), "{text}");
        }
    }
}
