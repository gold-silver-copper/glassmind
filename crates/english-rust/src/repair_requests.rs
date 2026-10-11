//! Bounded speech acts for repair authority. Only a complete positive request
//! licenses a repair; quotation contents are data, never reparsed as clauses.
use glassmind_world_rust::checks::RustCheck;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeechAct {
    Repair(RustCheck),
    Status(RustCheck),
    Problems(RustCheck),
}

#[derive(Clone)]
struct Token {
    text: String,
    quoted: bool,
}
fn tokens(text: &str) -> Option<Vec<Token>> {
    if text.len() > 4096 {
        return None;
    }
    let mut rest = text.trim();
    let mut out = Vec::new();
    while let Some(c) = rest.chars().next() {
        let (word, len, quoted) = if matches!(c, '`' | '"' | '“') {
            let close = if c == '“' { '”' } else { c };
            let body = &rest[c.len_utf8()..];
            let end = body.find(close)?;
            (&body[..end], c.len_utf8() + end + close.len_utf8(), true)
        } else {
            let len = if c.is_alphanumeric() || c == '_' {
                rest.char_indices()
                    .find(|(_, c)| !(c.is_alphanumeric() || matches!(c, '_' | '\'' | '’' | '-')))
                    .map_or(rest.len(), |(i, _)| i)
            } else {
                c.len_utf8()
            };
            (&rest[..len], len, false)
        };
        out.push(Token {
            text: word.to_lowercase().replace('’', "'"),
            quoted,
        });
        if out.len() > 128 {
            return None;
        }
        rest = rest[len..].trim_start();
    }
    Some(out)
}
#[derive(Clone)]
struct Cursor<'a> {
    ts: &'a [Token],
    at: usize,
}
impl Cursor<'_> {
    fn eat(&mut self, phrase: &str) -> bool {
        let mut at = self.at;
        for w in phrase.split_whitespace() {
            if !self.ts.get(at).is_some_and(|t| !t.quoted && t.text == w) {
                return false;
            }
            at += 1;
        }
        self.at = at;
        true
    }
    fn one(&mut self, choices: &[&str]) -> bool {
        choices.iter().any(|s| self.eat(s))
    }
    fn id(&mut self) -> bool {
        let Some(t) = self.ts.get(self.at) else {
            return false;
        };
        // Source identifiers may be quoted, but quoted commands are not names.
        if !t.quoted {
            return false;
        }
        let mut cs = t.text.chars();
        if !cs.next().is_some_and(|c| c.is_alphabetic() || c == '_')
            || !cs.all(|c| c.is_alphanumeric() || matches!(c, '_' | '-'))
        {
            return false;
        }
        self.at += 1;
        true
    }
    fn named_id(&mut self) -> bool {
        let Some(t) = self.ts.get(self.at) else {
            return false;
        };
        let mut cs = t.text.chars();
        if !cs.next().is_some_and(|c| c.is_alphabetic() || c == '_')
            || !cs.all(|c| c.is_alphanumeric() || matches!(c, '_' | '-'))
        {
            return false;
        }
        self.at += 1;
        true
    }
    fn source_name(&mut self) -> bool {
        let Some(t) = self.ts.get(self.at) else {
            return false;
        };
        if !t.quoted
            && [
                "if", "unless", "when", "while", "not", "never", "don't", "do", "could", "would",
                "should", "can", "will", "please", "fix", "repair", "explain", "delete", "rename",
                "and", "or", "then",
            ]
            .contains(&t.text.as_str())
        {
            return false;
        }
        self.named_id()
    }
    fn path(&mut self) -> bool {
        if !self.source_name() {
            return false;
        }
        loop {
            let saved = self.clone();
            if self.one(&[": :", "/", "."]) && self.source_name() {
                continue;
            }
            *self = saved;
            return true;
        }
    }
    fn diagnostic(&mut self) -> bool {
        if !self.eat("cannot find")
            || !self.one(&["value", "function", "trait", "type", "module"])
            || !self.id()
        {
            return false;
        }
        self.eat("in this scope");
        true
    }
    fn evidence(&mut self) -> bool {
        if self.one(&["i'm seeing", "i am seeing"]) {
            let Some(t) = self.ts.get(self.at) else {
                return false;
            };
            if !t.text.starts_with('e')
                || !t.text[1..].chars().all(|c| c.is_ascii_digit())
                || t.text.len() != 5
            {
                return false;
            }
            self.at += 1;
            if !self.eat(",") {
                return false;
            }
        }
        if self.ts.get(self.at).is_some_and(|t| t.quoted) {
            self.at += 1;
            if self.eat("(") {
                let Some(t) = self.ts.get(self.at) else {
                    return false;
                };
                if t.quoted
                    || t.text.len() != 5
                    || !t.text.starts_with('e')
                    || !t.text[1..].chars().all(|c| c.is_ascii_digit())
                {
                    return false;
                }
                self.at += 1;
                if !self.eat(")") {
                    return false;
                }
            }
            true
        } else {
            self.diagnostic()
        }
    }
    fn example(&mut self) -> bool {
        if !self.one(&["e . g .", "for example", "like"]) {
            return false;
        }
        self.eat("one in");
        self.path()
    }
    fn courtesy(&mut self) {
        self.one(&["please", "pls"]);
        self.one(&[
            "could you",
            "can you",
            "would you",
            "will you",
            "can u",
            "i would like you to",
            "i'd like you to",
            "i want you to",
        ]);
        self.one(&["please", "pls"]);
    }
    fn end(&mut self) -> bool {
        self.eat(",");
        self.one(&["please", "pls"]);
        self.one(&[".", "?", "!"]);
        if self.one(&["thanks", "thank you"]) {
            self.one(&[".", "!"]);
        }
        self.at == self.ts.len()
    }
    fn target(&mut self) -> Option<RustCheck> {
        let determined = self.one(&["all the", "all", "the", "this", "these"]);
        self.one(&["failing", "broken"]);
        // Bare source names before a check noun require a determiner. An
        // introductory verb ("suppose build ...") is not a source modifier.
        let ordinary_source_noun = self.ts.get(self.at).is_some_and(|t| {
            !t.quoted && ["code", "project", "crate", "rust"].contains(&t.text.as_str())
        });
        if !self.id() && determined && !ordinary_source_noun {
            let saved = self.clone();
            if self.source_name() {
                let next = self.ts.get(self.at);
                if !next.is_some_and(|t| {
                    !t.quoted && ["build", "tests", "test"].contains(&t.text.as_str())
                }) {
                    *self = saved;
                }
            }
        }
        self.eat("rust");
        let kind = if self.one(&["unit tests", "test suite", "tests", "test"]) {
            RustCheck::Test
        } else if self.one(&[
            "compiler errors",
            "compilation errors",
            "compile errors",
            "compile erors",
            "compiler error",
            "compilation error",
            "compile error",
            "build",
            "compilation",
            "project",
            "crate",
            "code",
        ]) {
            RustCheck::Build
        } else {
            return None;
        };
        self.one(&["errors", "issues", "failures"]);
        if self.eat("for function") && !self.named_id() {
            return None;
        }
        Some(kind)
    }
    fn location(&mut self) -> bool {
        if self.eat("in") {
            self.source_name()
        } else {
            true
        }
    }
    fn predicate(&mut self, kind: RustCheck) -> bool {
        match kind {
            RustCheck::Build => self.one(&[
                "to compile",
                "compile",
                "compiles",
                "compiling",
                "builds",
                "build",
                "pass",
                "passes",
                "passing",
                "working",
                "green",
            ]),
            RustCheck::Test => {
                self.one(&["to pass", "pass", "passes", "passing", "working", "green"])
            }
            _ => false,
        }
    }
    fn subject(&mut self) -> Option<RustCheck> {
        let saved = self.clone();
        if let Some(k) = self.target() {
            return Some(k);
        }
        *self = saved;
        self.source_name().then_some(RustCheck::Build)
    }
    fn status(&mut self) -> Option<RustCheck> {
        let auxiliary = self.one(&["does", "do", "can", "is", "are"]);
        if !auxiliary {
            return None;
        }
        let k = self.subject()?;
        (self.predicate(k) && self.end()).then_some(k)
    }
    fn problems(&mut self) -> Option<RustCheck> {
        if !self.one(&["are there", "is there"]) {
            return None;
        }
        self.one(&["any", "a", "an"]);
        let k = if self.one(&[
            "compiler errors",
            "compiler error",
            "compilation errors",
            "compilation error",
            "compile errors",
            "compile error",
            "build errors",
            "build error",
        ]) {
            RustCheck::Build
        } else if self.one(&[
            "test failures",
            "test failure",
            "failing tests",
            "failing test",
        ]) {
            RustCheck::Test
        } else {
            return None;
        };
        self.end().then_some(k)
    }
    fn check_request(&mut self) -> Option<RustCheck> {
        self.courtesy();
        if !self.one(&["check whether", "check if", "check that"]) {
            return None;
        }
        let k = self.subject()?;
        (self.predicate(k) && self.end()).then_some(k)
    }
    fn repair(&mut self, context: Option<RustCheck>) -> Option<RustCheck> {
        self.courtesy();
        let causative = if self.one(&["fix", "repair", "resolve", "correct"]) {
            self.one(&["things so", "it so", "so"])
        } else if self.one(&["make", "get"]) {
            true
        } else {
            return None;
        };
        let bare = self.clone();
        if let Some(k) = context
            && !causative
            && self.end()
        {
            return Some(k);
        }
        *self = bare;
        let k = if self.one(&["it", "them", "they"]) {
            if let Some(k) = context {
                k
            } else if causative && self.one(&["compile", "compiles", "compiling"]) {
                return self.end().then_some(RustCheck::Build);
            } else {
                return None;
            }
        } else {
            let saved = self.clone();
            if let Some(k) = self.target() {
                k
            } else if causative {
                *self = saved;
                if !self.source_name() {
                    return None;
                }
                RustCheck::Build
            } else {
                return None;
            }
        };
        if causative && !self.predicate(k) {
            return None;
        }
        self.eat("again");
        if !self.location() {
            return None;
        }
        if self.one(&["so it", "so they"]) && !self.predicate(k) {
            return None;
        }
        self.eat("again");
        let tail = self.clone();
        if self.eat(",") {
            let after_comma = self.clone();
            if !(self.diagnostic() || {
                *self = after_comma;
                self.example()
            }) {
                *self = tail;
            }
        }
        // A preservation constraint adds no second operation. Consume the
        // entire constraint so conditional or destructive continuations refuse.
        if self.one(&["and preserve", "while preserving", "without changing"]) {
            self.eat("the");
            self.one(&["existing", "current"]);
            if !self.one(&["behavior", "behaviour"]) {
                return None;
            }
        }
        self.end().then_some(k)
    }
    fn context(&mut self) -> Option<RustCheck> {
        let existential = self.eat("there are");
        self.eat("some");
        let k = self.subject()?;
        if !self.location() {
            return None;
        }
        if !existential
            && !self.one(&[
                "is broken",
                "is failing",
                "are failing",
                "fails",
                "broke",
                "does not compile",
                "doesn't compile",
                "doesn't build",
            ])
        {
            return None;
        }
        if self.eat("with") && !self.evidence() {
            return None;
        }
        if !self.one(&[".", ",", ";"]) {
            return None;
        }
        let saved = self.clone();
        if self.evidence() || {
            *self = saved.clone();
            self.example()
        } {
            if !self.one(&[".", ",", ";"]) {
                return None;
            }
        } else {
            *self = saved;
        }
        Some(k)
    }
}

/// Parse the entire supported utterance. None carries no repair permission;
/// callers must still retain unrelated, independently supported actions.
pub fn speech_act(text: &str) -> Option<SpeechAct> {
    let ts = tokens(text)?;
    let c = Cursor { ts: &ts, at: 0 };
    if let Some(k) = c.clone().problems() {
        return Some(SpeechAct::Problems(k));
    }
    if let Some(k) = c.clone().status().or_else(|| c.clone().check_request()) {
        return Some(SpeechAct::Status(k));
    }
    if let Some(k) = c.clone().repair(None) {
        return Some(SpeechAct::Repair(k));
    }
    let mut c = c;
    let context = c.context()?;
    c.repair(Some(context)).map(SpeechAct::Repair)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_diagnostic_and_preservation_families() {
        for noun in [
            "compiler error",
            "compilation error",
            "compile error",
            "Rust crate",
            "Rust project",
            "Rust code",
        ] {
            for prefix in ["Fix the", "Could you repair this", "Please correct the"] {
                let text = format!("{prefix} {noun}.");
                assert_eq!(
                    speech_act(&text),
                    Some(SpeechAct::Repair(RustCheck::Build)),
                    "{text}"
                );
                for tail in [
                    " if I approve",
                    " and delete tests",
                    " after approval",
                    " or explain it",
                ] {
                    let text = format!("{prefix} {noun}{tail}.");
                    assert_eq!(speech_act(&text), None, "{text}");
                }
            }
        }
        for tail in [
            "and preserve behavior",
            "while preserving the existing behaviour",
            "without changing current behavior",
        ] {
            let text = format!("Please fix the build {tail}.");
            assert_eq!(
                speech_act(&text),
                Some(SpeechAct::Repair(RustCheck::Build)),
                "{text}"
            );
            let text = format!("Fix the build {tail} if I authorize it.");
            assert_eq!(speech_act(&text), None, "{text}");
        }
        for prefix in [
            "The build is failing;",
            "The crate does not compile;",
            "The project is broken;",
        ] {
            let text = format!("{prefix} please fix it.");
            assert_eq!(
                speech_act(&text),
                Some(SpeechAct::Repair(RustCheck::Build)),
                "{text}"
            );
            for tail in [
                "do not fix it.",
                "explain how to fix it.",
                "fix it after I approve.",
                "fix it and delete tests.",
            ] {
                let text = format!("{prefix} {tail}");
                assert_eq!(speech_act(&text), None, "{text}");
            }
        }
        assert_eq!(
            speech_act("Does this code build?"),
            Some(SpeechAct::Status(RustCheck::Build))
        );
        assert_eq!(
            speech_act("Are there any compiler errors?"),
            Some(SpeechAct::Problems(RustCheck::Build))
        );
        assert_eq!(
            speech_act("Is there a test failure?"),
            Some(SpeechAct::Problems(RustCheck::Test))
        );
        assert_eq!(speech_act("Are there compiler errors? Fix them."), None);
    }

    #[test]
    fn existing_public_diagnostic_requests_and_outer_scope() {
        for row in include_str!("../../../tools/coding-repair-speech/public-legacy.tsv").lines() {
            let (kind, text) = row.split_once('\t').unwrap();
            let k = if kind == "repair_build" {
                RustCheck::Build
            } else {
                RustCheck::Test
            };
            assert_eq!(speech_act(text), Some(SpeechAct::Repair(k)), "{text}");
        }
        for text in [
            "Suppose build is broken. Please fix it.",
            "Imagine tests are failing. Fix them.",
            "Assume build is broken, can you fix it?",
        ] {
            assert_eq!(speech_act(text), None, "{text}");
        }
        for context in [
            "The build is broken.",
            "The crate build fails with \"missing symbol\" (E0405).",
            "The tests are failing.",
        ] {
            for request in [
                "Could you explain how to fix it?",
                "Please fix it only if I confirm.",
                "Don't fix it.",
                "Please fix it and delete the tests.",
                "\"Please fix it\".",
            ] {
                let text = format!("{context} {request}");
                assert_eq!(speech_act(&text), None, "{text}");
            }
        }
    }

    #[test]
    fn frozen_complete_speech_acts() {
        for row in include_str!("../../../tools/coding-repair-speech/cases.tsv").lines() {
            let (expected, text) = row.split_once('\t').unwrap();
            let want = match expected {
                "repair_build" => Some(SpeechAct::Repair(RustCheck::Build)),
                "repair_tests" => Some(SpeechAct::Repair(RustCheck::Test)),
                "status_build" => Some(SpeechAct::Status(RustCheck::Build)),
                "status_tests" => Some(SpeechAct::Status(RustCheck::Test)),
                "refuse" => None,
                _ => panic!("invalid authored case"),
            };
            assert_eq!(speech_act(text), want, "{text}");
        }
    }
}
