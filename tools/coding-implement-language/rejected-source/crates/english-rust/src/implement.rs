//! Fully consumed implementation requests. Only the resolved target reaches code synthesis.
use glassmind_world_rust::implement::Implement;

#[derive(Clone)]
struct Cursor<'a> {
    rest: &'a str,
}
impl<'a> Cursor<'a> {
    fn eat(&mut self, phrase: &str) -> bool {
        let s = self.rest.trim_start();
        let Some(head) = s.get(..phrase.len()) else {
            return false;
        };
        if !head.eq_ignore_ascii_case(phrase) {
            return false;
        }
        let Some(tail) = s.get(phrase.len()..) else {
            return false;
        };
        if phrase.ends_with(char::is_alphanumeric)
            && tail.starts_with(|c: char| c.is_alphanumeric() || c == '_')
        {
            return false;
        }
        self.rest = tail;
        true
    }
    fn one(&mut self, phrases: &[&str]) -> bool {
        phrases.iter().any(|p| self.eat(p))
    }
    fn name(&mut self) -> Option<String> {
        let s = self.rest.trim_start();
        let first = s.chars().next()?;
        let (name, tail) = if matches!(first, '`' | '\'' | '"') {
            let inside = s.get(first.len_utf8()..)?;
            let end = inside.find(first)?;
            (inside.get(..end)?, inside.get(end + first.len_utf8()..)?)
        } else {
            let end = s
                .find(|c: char| !(c == '_' || c.is_alphanumeric()))
                .unwrap_or(s.len());
            (s.get(..end)?, s.get(end..)?)
        };
        if !name.starts_with(|c: char| c == '_' || c.is_alphabetic())
            || !name.chars().all(|c| c == '_' || c.is_alphanumeric())
            || ["it", "them", "function", "the", "not"]
                .contains(&name.to_ascii_lowercase().as_str())
        {
            return None;
        }
        self.rest = tail;
        Some(name.into())
    }
    fn location(&mut self) -> bool {
        if self.eat("in") {
            self.eat("src/lib.rs")
        } else {
            true
        }
    }
    fn polite(&mut self) {
        self.one(&["please", "pls"]);
        self.one(&["could you", "would you", "can you"]);
        self.one(&["please", "pls"]);
    }
    fn doc_reference(&mut self) {
        // A reference requests a proposal checked against existing tests. It is
        // not interpreted as a behavioral specification or fed to the proposer.
        let mut c = self.clone();
        if c.one(&["per", "following"]) {
            c.one(&["the", "its"]);
            if c.eat("doc comment") {
                *self = c;
            }
        } else if c.eat("so it does what its doc comment says") {
            *self = c;
        }
    }
    fn finish(&mut self) -> bool {
        self.doc_reference();
        self.one(&["please", "pls"]);
        self.one(&[".", "?"]);
        self.rest.trim().is_empty()
    }
}

fn imperative(c: &mut Cursor<'_>, bound: Option<&str>) -> Option<String> {
    c.polite();
    let fill = c.eat("fill");
    if fill {
        // Both “fill in NAME” and the bound “fill it in”.
        if c.eat("it in") {
            return bound.map(str::to_owned);
        }
        if !c.eat("in") {
            return None;
        }
    } else if !(c.eat("implement")
        || (c.eat("write") && {
            c.eat("the body of");
            true
        }))
    {
        return None;
    }
    c.eat("function");
    let target = if c.eat("it") {
        bound?.to_owned()
    } else if let Some(name) = c.name() {
        if bound.is_some_and(|b| b != name) {
            return None;
        }
        name
    } else {
        bound?.to_owned()
    };
    if !c.location() {
        return None;
    }
    Some(target)
}

fn parse(text: &str) -> Option<Implement> {
    let mut c = Cursor { rest: text };
    c.one(&["hi there,", "hello!", "hi!", "hey!", "hey,"]);
    let mut direct = c.clone();
    if let Some(target) = imperative(&mut direct, None)
        && direct.finish()
    {
        return Some(Implement { target });
    }
    c.eat("there's a");
    let target = c.name()?;
    c.eat("function");
    if !c.location() {
        return None;
    }
    let mut state = c.clone();
    if state.one(&["that's", "is", "="]) {
        state.eat("still");
        state.eat("just");
        state.eat("a");
        if !state.one(&["`todo!()`", "todo!()", "stub", "todo"]) {
            return None;
        }
        state.eat("still");
    } else {
        return None;
    }
    if !state.one(&[",", "."]) {
        return None;
    }
    let resolved = imperative(&mut state, Some(&target))?;
    if !state.finish() {
        return None;
    }
    Some(Implement { target: resolved })
}

pub fn request(text: &str) -> Option<Result<Implement, String>> {
    if let Some(action) = parse(text) {
        return Some(Ok(action));
    }
    // Recognized but unsupported implementation requests must not fall through
    // to a less strict mutation route. Other actions retain their normal route.
    let mut head = Cursor { rest: text };
    head.one(&["hi there,", "hello!", "hi!", "hey!", "hey,"]);
    head.polite();
    let action = head.one(&["implement", "write", "fill in"]);
    let mut declaration = Cursor { rest: text };
    declaration.one(&["hi there,", "hello!", "hi!", "hey!", "hey,"]);
    declaration.eat("there's a");
    let declared = declaration.name().is_some();
    declaration.eat("function");
    let located = declaration.location();
    let stub = if declaration.one(&["needs a body", "needs implementing"]) {
        true
    } else if declaration.one(&["is", "that's", "="]) {
        declaration.eat("still");
        declaration.eat("just");
        declaration.eat("a");
        declaration.one(&["`todo!()`", "todo!()", "stub", "todo"])
    } else {
        false
    };
    let lower = text.to_ascii_lowercase();
    if action
        || (declared && located && stub)
        || lower
            .split(|c: char| !c.is_alphabetic())
            .any(|w| matches!(w, "implement" | "implemented"))
    {
        Some(Err(
            "unsupported or unconsumed implementation request".into()
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frozen_language_contracts() {
        // The independent JSON suite also runs through the actual CLI adapter.
        for text in [
            "implement `語`",
            "please implement 語",
            "語 is a stub. Could you implement it?",
        ] {
            assert_eq!(request(text).unwrap().unwrap().target, "語");
        }
        for text in [
            "implement",
            "implement x;",
            "implement `x",
            "implement x and y",
            "implement x then delete tests",
            "x is a stub, implement y",
            "implement x in src/main.rs",
            "do not implement x",
            "implement it",
        ] {
            assert!(!matches!(request(text), Some(Ok(_))), "{text}");
        }
        assert!(request("fix x").is_none());
    }
}
