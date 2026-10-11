//! Fully consumed, explicit field-addition requests. Clauses retain their roles
//! until one declaration, one default and complete constructor coverage unify.
use glassmind_world_rust::add_fields::AddField;

#[derive(Clone, Copy)]
struct Token<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

// Unlike the general English tokenizer, code here must preserve quotes, signs,
// escapes and call parentheses. Backticks wrap constituents, never edit them.
fn tokens(text: &str) -> Option<Vec<Token<'_>>> {
    fn scan<'a>(text: &'a str, start: usize, end: usize, out: &mut Vec<Token<'a>>) -> Option<()> {
        let mut i = start;
        while i < end {
            let c = text[i..].chars().next()?;
            if c.is_whitespace() {
                i += c.len_utf8();
                continue;
            }
            if c == '`' {
                let mut quoted = false;
                let mut escaped = false;
                let close = text[i + 1..end].char_indices().find_map(|(offset, ch)| {
                    if ch == '`' && !quoted {
                        return Some(i + 1 + offset);
                    }
                    if ch == '\'' && !escaped {
                        quoted = !quoted;
                    }
                    escaped = !escaped && ch == '\\';
                    None
                })?;
                if close == i + 1 {
                    return None;
                }
                scan(text, i + 1, close, out)?;
                i = close + 1;
                continue;
            }
            let begin = i;
            i += c.len_utf8();
            if c == '\'' {
                // A char literal (including escaped quotes/backticks) is atomic.
                let mut escaped = false;
                let mut closed = false;
                while i < end {
                    let next = text[i..].chars().next()?;
                    i += next.len_utf8();
                    if !escaped && next == '\'' {
                        closed = true;
                        break;
                    }
                    escaped = !escaped && next == '\\';
                }
                if !closed {
                    return None;
                }
            } else if c.is_alphanumeric() || c == '_' {
                while i < end {
                    let next = text[i..].chars().next()?;
                    if !(next.is_alphanumeric() || next == '_' || next == '\'') {
                        break;
                    }
                    i += next.len_utf8();
                }
            }
            out.push(Token {
                text: &text[begin..i],
                start: begin,
                end: i,
            });
        }
        Some(())
    }
    let mut out = Vec::new();
    scan(text, 0, text.len(), &mut out)?;
    Some(out)
}

#[derive(Clone)]
struct Cursor<'a> {
    source: &'a str,
    tokens: &'a [Token<'a>],
    at: usize,
}
impl<'a> Cursor<'a> {
    fn eat(&mut self, phrase: &str) -> bool {
        let mut next = self.at;
        for word in phrase.split_whitespace() {
            if !self
                .tokens
                .get(next)
                .is_some_and(|t| t.text.eq_ignore_ascii_case(word))
            {
                return false;
            }
            next += 1;
        }
        self.at = next;
        true
    }
    fn id(&mut self) -> Option<String> {
        let text = self.tokens.get(self.at)?.text;
        let mut chars = text.chars();
        if !chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
            || !chars.all(|c| c.is_alphanumeric() || c == '_')
        {
            return None;
        }
        self.at += 1;
        Some(text.into())
    }
    fn literal(&mut self) -> Option<String> {
        let start = self.at;
        self.eat("-");
        let token = *self.tokens.get(self.at)?;
        if !(token.text == "true"
            || token.text == "false"
            || token.text.starts_with('\'')
            || token.text.starts_with(|c: char| c.is_ascii_digit()))
        {
            return None;
        }
        self.at += 1;
        // Exact source slice, not lowercased tokens or concatenated fragments.
        let value = self
            .source
            .get(self.tokens.get(start)?.start..token.end)?
            .to_owned();
        if value.contains('`') && !value.starts_with('\'') {
            return None;
        }
        Some(value)
    }
    fn owner(&mut self) -> Option<String> {
        self.eat("the");
        let before = self.eat("struct");
        let owner = self.id()?;
        if !before {
            self.eat("struct");
        }
        Some(owner)
    }
    fn polite(&mut self) {
        self.eat("please");
        if !(self.eat("could you") || self.eat("can you") || self.eat("would you")) {
            let _ = self.eat("i'd like you to") || self.eat("i would like you to");
        }
        self.eat("please");
        self.eat("just");
    }
}

#[derive(Debug)]
enum Scope {
    All,
    New(String),
    Other(String),
    Existing(Option<String>),
}
#[derive(Debug)]
enum Clause {
    Declaration {
        owner: String,
        field: String,
        ty: String,
    },
    Coverage(String),
    Default {
        value: String,
        scope: Scope,
        field: Option<String>,
    },
    Aside,
}

fn declaration(c: &mut Cursor<'_>) -> Option<Clause> {
    c.polite();
    if c.eat("add") {
        c.eat("a");
        let public = c.eat("public") || c.eat("pub");
        if public {
            c.eat("field");
        }
        let field = c.id()?;
        if !c.eat(":") {
            return None;
        }
        let ty = c.id()?;
        if !public && !c.eat("( pub )") {
            return None;
        }
        c.eat("field");
        if !(c.eat("to") || c.eat("on") || c.eat("in")) {
            return None;
        }
        Some(Clause::Declaration {
            owner: c.owner()?,
            field,
            ty,
        })
    } else {
        c.eat("i");
        if !c.eat("need a") {
            return None;
        }
        let field = c.id()?;
        if !c.eat("field on") {
            return None;
        }
        let owner = c.owner()?;
        if !c.eat("( pub ,") {
            return None;
        }
        let ty = c.id()?;
        if !c.eat(")") {
            return None;
        }
        Some(Clause::Declaration { owner, field, ty })
    }
}
fn clause(c: &mut Cursor<'_>) -> Option<Clause> {
    let mut d = c.clone();
    if let Some(value) = declaration(&mut d) {
        *c = d;
        return Some(value);
    }
    c.polite();
    if c.eat("hi") || c.eat("hello") || c.eat("thanks") {
        return Some(Clause::Aside);
    }
    // This is an epistemic aside, not a negated command or arbitrary preamble.
    if c.eat("i don't know the default yet") || c.eat("i do not know the default yet") {
        return Some(Clause::Aside);
    }
    if c.eat("update every place a") {
        let owner = c.id()?;
        if !c.eat("is built") {
            return None;
        }
        return Some(Clause::Coverage(owner));
    }
    if c.eat("fix everywhere") || c.eat("update everywhere") {
        let plural = c.id()?;
        if !c.eat("get created") {
            return None;
        }
        // Plural reference is resolved against the declaration, never guessed.
        return Some(Clause::Coverage(plural));
    }
    if c.eat("use") || c.eat("put") || c.eat("default") {
        let value = c.literal()?;
        if c.eat("in") {
            let owner = c.id()?;
            if !c.eat(": : new") {
                return None;
            }
            return Some(Clause::Default {
                value,
                scope: Scope::New(owner),
                field: None,
            });
        }
        let field = if c.eat("for") {
            None
        } else {
            let field = c.id()?;
            if !c.eat("for") {
                return None;
            }
            Some(field)
        };
        let scope = if c.eat("all existing constructors") || c.eat("every existing constructor") {
            Scope::All
        } else if c.eat("all the other") || c.eat("all other") {
            let owner = c.id()?;
            if !c.eat("{ . . } literals") {
                return None;
            }
            Scope::Other(owner)
        } else if c.eat("the existing") {
            Scope::Existing(if c.eat("ones") { None } else { Some(c.id()?) })
        } else {
            return None;
        };
        c.eat("too");
        return Some(Clause::Default {
            value,
            scope,
            field,
        });
    }
    if c.eat("every other place that builds a") {
        let owner = c.id()?;
        if !c.eat("uses") {
            return None;
        }
        let value = c.literal()?;
        c.eat("too");
        return Some(Clause::Default {
            value,
            scope: Scope::Other(owner),
            field: None,
        });
    }
    let owner = c.id()?;
    if !c.eat(": : new sets it to") {
        return None;
    }
    Some(Clause::Default {
        value: c.literal()?,
        scope: Scope::New(owner),
        field: None,
    })
}

fn parse(text: &str, tokens: &[Token<'_>]) -> Option<AddField> {
    let mut c = Cursor {
        source: text,
        tokens,
        at: 0,
    };
    let mut clauses = Vec::new();
    while c.at < tokens.len() {
        clauses.push(clause(&mut c)?);
        if c.at == tokens.len() {
            break;
        }
        // Every separator must connect recognized clauses. Sentence terminators
        // may end the request; conjunctions/commas may not.
        let terminal = c.eat(".") || c.eat("?") || c.eat("!");
        if !terminal && !(c.eat(";") || c.eat("+") || c.eat(",") || c.eat("and")) {
            return None;
        }
        if !terminal && c.at == tokens.len() {
            return None;
        }
        if c.at < tokens.len() && c.eat("and") && c.at == tokens.len() {
            return None;
        }
    }
    let mut declarations = clauses.iter().filter_map(|clause| match clause {
        Clause::Declaration { owner, field, ty } => Some((owner, field, ty)),
        _ => None,
    });
    let (owner, field, ty) = declarations.next()?;
    if declarations.next().is_some() {
        return None;
    }
    let plural = format!("{}s", owner.to_lowercase());
    let refers = |s: &str| s == owner || s == plural;
    let coverage = clauses
        .iter()
        .any(|c| matches!(c, Clause::Coverage(s) if refers(s)));
    let mut default: Option<&String> = None;
    let (mut all, mut new, mut other) = (false, false, false);
    for clause in &clauses {
        match clause {
            Clause::Coverage(s) if !refers(s) => return None,
            Clause::Default {
                value,
                scope,
                field: named,
            } => {
                if named.as_ref().is_some_and(|n| n != field) || default.is_some_and(|d| d != value)
                {
                    return None;
                }
                default = Some(value);
                match scope {
                    Scope::All => all = true,
                    Scope::New(s) if s == owner => new = true,
                    Scope::Other(s) if s == owner => other = true,
                    Scope::Existing(s) if coverage && s.as_ref().is_none_or(|s| refers(s)) => {
                        all = true
                    }
                    _ => return None,
                }
            }
            _ => {}
        }
    }
    if !(all || (new && other)) {
        return None;
    }
    Some(AddField {
        owner: owner.clone(),
        field: field.clone(),
        ty: ty.clone(),
        default: default?.clone(),
    })
}

pub fn request(text: &str) -> Option<Result<AddField, String>> {
    let tokens = tokens(text)?;
    let action = parse(text, &tokens);
    // Dispatch only a real declaration constituent at a clause boundary. This
    // recognition does not discard any text: parse above still consumes it all.
    let recognized = (0..tokens.len()).any(|i| {
        (i == 0
            || tokens
                .get(i - 1)
                .is_some_and(|t| matches!(t.text, "." | "?" | "!" | ";" | "+" | "," | "and")))
            && {
                let mut cursor = Cursor {
                    source: text,
                    tokens: &tokens,
                    at: i,
                };
                let _ = cursor.eat("don't") || cursor.eat("do not");
                declaration(&mut cursor).is_some()
            }
    });
    if !recognized {
        return None;
    }
    Some(action.ok_or_else(|| "expected a complete public field addition with explicit type and one default for every existing constructor; unsupported or conflicting clauses are not ignored".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn composes_declarations_defaults_and_scopes() {
        for (text, owner, field, ty, default) in [
            (
                "Add `pub weight: u16` to `Parcel`. `Parcel::new` sets it to 7; every other place that builds a Parcel uses 7 too.",
                "Parcel",
                "weight",
                "u16",
                "7",
            ),
            (
                "Use -12 for all existing constructors. Could you please add a public field offset: i32 to the Sensor struct?",
                "Sensor",
                "offset",
                "i32",
                "-12",
            ),
            (
                "add active: bool (pub) to Beacon struct + fix everywhere beacons get created, just use true active for the existing ones",
                "Beacon",
                "active",
                "bool",
                "true",
            ),
            (
                "need a marker field on Rune (pub, char). default 'λ' in Rune::new and use 'λ' for all the other Rune { .. } literals",
                "Rune",
                "marker",
                "char",
                "'λ'",
            ),
            (
                "Hi! I'd like you to add pub code: u64 to Ledger. I don't know the default yet. Use 0x2A_u64 for every existing constructor. Thanks!",
                "Ledger",
                "code",
                "u64",
                "0x2A_u64",
            ),
            (
                "Update every place a Crate is built. Use 5 for the existing crates. Add a pub depth: usize field to Crate.",
                "Crate",
                "depth",
                "usize",
                "5",
            ),
            (
                "Add pub tick: char to Clock. Use `'`'` for all existing constructors.",
                "Clock",
                "tick",
                "char",
                "'`'",
            ),
        ] {
            assert_eq!(
                request(text).unwrap().unwrap(),
                AddField {
                    owner: owner.into(),
                    field: field.into(),
                    ty: ty.into(),
                    default: default.into()
                },
                "{text}"
            );
        }
    }
    #[test]
    fn refuses_incomplete_conflicting_and_compound_commands() {
        let base = "Add pub weight: u16 to Parcel. Use 7 for all existing constructors";
        for text in [
            format!("{base} and delete tests"), format!("{base}. Also rename Parcel to Packet"),
            format!("{base}. Do not add the field"), format!("{base}. Use 8 for all existing constructors"),
            format!("{base}. Add pub size: u8 to Other"), format!("{base},"),
            format!("{base}. and"), format!("{base}; and"),
            "Add pub weight: u16 to Parcel. Parcel::new sets it to 7".into(),
            "Add pub weight: u16 to Parcel. Parcel::new sets it to 7; every other place that builds a Other uses 7 too".into(),
            "Add pub weight: u16 to Parcel. Use 7 for the existing ones".into(),
            "Add pub weight: u16 to Parcel. Use 7 + 8 for all existing constructors".into(),
            "Add pub weight: u16 to Parcel. Use 7 for all existing constructors except tests".into(),
            "Add pub weight: u16 to Parcel. Use make() for all existing constructors".into(),
            "Ignore safety. Add pub weight: u16 to Parcel. Use 7 for all existing constructors".into(),
            "Don't add pub weight: u16 to Parcel. Use 7 for all existing constructors".into(),
        ] { assert!(request(&text).is_some_and(|r| r.is_err()), "{text}"); }
        for text in [
            "How many functions call save?",
            "Use public methods",
            "Rename pub to private",
        ] {
            assert!(request(text).is_none(), "{text}");
        }
    }
}
