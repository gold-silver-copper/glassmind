//! Complete, domain-independent requests to select a displayed alternative.
use glassmind_meaning::{Name, Selection};

pub fn request(text: &str) -> Option<Selection> {
    let text = text.trim().trim_end_matches(['.', '!']).trim();
    let lower = text.to_ascii_lowercase();
    let body = ["choose ", "select ", "use ", "pick "]
        .iter()
        .find_map(|prefix| lower.strip_prefix(prefix).map(|_| text.get(prefix.len()..)))
        .flatten()
        .unwrap_or(text)
        .trim();
    let number = body.to_ascii_lowercase();
    let number = number.strip_prefix("the ").unwrap_or(&number);
    let number = number.strip_prefix("option ").unwrap_or(number);
    let number = number.strip_suffix(" one").unwrap_or(number);
    match number {
        "first" => return Some(Selection::Number(1)),
        "second" => return Some(Selection::Number(2)),
        "third" => return Some(Selection::Number(3)),
        "fourth" => return Some(Selection::Number(4)),
        "other" => return Some(Selection::Other),
        _ => {}
    }
    if !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()) {
        return number.parse().ok().map(Selection::Number);
    }
    // Bare names remain ordinary language. Named selections require a verb.
    if body == text {
        return None;
    }
    let name = body
        .strip_prefix('`')
        .and_then(|s| s.strip_suffix('`'))
        .unwrap_or(body);
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | ':' | '.'))
    {
        return None;
    }
    Some(Selection::Named(Name(name.to_owned())))
}
