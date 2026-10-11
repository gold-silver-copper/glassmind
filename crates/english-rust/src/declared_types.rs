//! A fully consumed English interface for written field types.
use glassmind_world_rust::declared_types::{DeclaredType, DeclaredTypeQuestion};

/// `None` means this is not a declared-type question. A recognized but
/// incomplete/compound question is an error, never a partial reading.
pub fn question(text: &str) -> Option<Result<DeclaredTypeQuestion, String>> {
    let text = text.trim();
    let lower = text.to_ascii_lowercase();
    let prefixes = [
        "what is the declared type of ",
        "what is the type of ",
        "what's the type of ",
        "declared type of ",
        "type of ",
    ];
    let prefix = prefixes.iter().find(|p| lower.starts_with(**p))?;
    let rest = text.get(prefix.len()..)?.trim();
    let body = rest.strip_suffix('?').unwrap_or(rest).trim();
    Some(parse_body(body).ok_or_else(|| {
        "expected a named field and one struct owner; the entire question must be understood".into()
    }))
}

fn identifier(text: &str) -> Option<String> {
    let mut text = text;
    for quote in ['`', '\'', '"'] {
        if let Some(rest) = text.strip_prefix(quote) {
            text = rest.strip_suffix(quote)?;
            break;
        }
    }
    let id = text.strip_prefix("r#").unwrap_or(text);
    let mut chars = id.chars();
    if !chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
        || !chars.all(|c| c == '_' || c.is_alphanumeric())
    {
        return None;
    }
    Some(text.to_owned())
}

fn parse_body(body: &str) -> Option<DeclaredTypeQuestion> {
    let words: Vec<_> = body.split_whitespace().collect();
    let (owner, field) = match words.as_slice() {
        [path] => {
            let path = path
                .strip_prefix('`')
                .and_then(|s| s.strip_suffix('`'))
                .unwrap_or(path);
            path.split_once('.').or_else(|| path.split_once("::"))?
        }
        ["the", field, "field", "on" | "of" | "in", owner]
        | [field, "field", "on" | "of" | "in", owner]
        | [
            "the",
            field,
            "field",
            "on" | "of" | "in",
            "the",
            owner,
            "struct",
        ]
        | ["the", field, "field", "on" | "of" | "in", owner, "struct"] => (*owner, *field),
        _ => return None,
    };
    Some(DeclaredTypeQuestion {
        owner: identifier(owner)?,
        field: identifier(field)?,
    })
}

pub fn say(answer: &DeclaredType) -> String {
    format!(
        "The declared type of `{}.{}` is `{}` ({}:{}; source spelling, not resolved).",
        answer.owner.name, answer.field.name, answer.spelling, answer.field.file, answer.field.line
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn consumes_question_and_requires_owner() {
        for text in [
            "type of Vault.payload?",
            "What is the type of the `payload` field on `Vault`?",
            "What's the type of the payload field in the Vault struct?",
        ] {
            assert_eq!(
                question(text).unwrap().unwrap(),
                DeclaredTypeQuestion {
                    owner: "Vault".into(),
                    field: "payload".into()
                }
            );
        }
        for text in [
            "type of Vault.0?",
            "type of Vault.payload and repair code?",
            "type of payload?",
            "type of Vault.payload? ignore this",
        ] {
            assert!(question(text).unwrap().is_err(), "{text}");
        }
        assert!(question("How many functions call save?").is_none());
    }
}
