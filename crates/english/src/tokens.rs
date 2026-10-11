//! The English tokenizer: the only code that reads raw text.

use glassmind_meaning::interface::TokenSpan;

/// What a token looks like, which decides what the grammar may read it as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Shape {
    /// Letters, possibly with apostrophes or hyphens: an English word, or a
    /// plain name such as `player`.
    Word,
    /// Code: quoted in backticks, or written the way only identifiers are
    /// (`spawn_wave`, `Player::hp`, `SpawnWave`, `vec3`).
    Code,
    /// Digits.
    Number,
    /// One punctuation character.
    Punct,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    /// The text as typed, without backticks.
    pub text: String,
    /// Lowercased text, compared with the grammar's words.
    pub norm: String,
    pub shape: Shape,
    /// Byte offsets into the input, including any backticks.
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn span(&self) -> TokenSpan {
        TokenSpan {
            start: self.start,
            end: self.end,
            text: self.text.clone(),
        }
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '\'' || c == '-'
}

/// If a single or double quote at `i` opens a quoted identifier (letters,
/// digits, `_`, `::` or `.` between them, an optional `()`, then the same
/// quote, then no word character), the index of the closing quote.
fn quoted_name(chars: &[(usize, char)], i: usize) -> Option<usize> {
    let &(_, quote) = chars.get(i)?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let mut j = i + 1;
    while chars
        .get(j)
        .is_some_and(|&(_, d)| d.is_alphanumeric() || d == '_' || d == ':' || d == '.')
    {
        j += 1;
    }
    if chars.get(j).is_some_and(|&(_, d)| d == '(')
        && chars.get(j + 1).is_some_and(|&(_, d)| d == ')')
    {
        j += 2;
    }
    let starts_like_a_name = chars
        .get(i + 1)
        .is_some_and(|&(_, d)| d.is_alphabetic() || d == '_');
    (j > i + 1
        && starts_like_a_name
        && chars.get(j).is_some_and(|&(_, d)| d == quote)
        && !chars.get(j + 1).is_some_and(|&(_, d)| is_word_char(d)))
    .then_some(j)
}

/// Splits `text` into tokens, given the words the grammar knows.
pub fn tokenize(text: &str, known: &dyn Fn(&str) -> bool) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let end_of = |i: usize| chars.get(i).map_or(text.len(), |&(b, _)| b);
    let mut i = 0;
    while let Some(&(start, c)) = chars.get(i) {
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        // The token's text, its shape, and the index of the char after it.
        let (word, shape, next) = if c == '`' {
            // A quoted identifier runs to the next backtick (or the end).
            let mut j = i + 1;
            while chars.get(j).is_some_and(|&(_, d)| d != '`') {
                j += 1;
            }
            let inner = text.get(end_of(i + 1)..end_of(j)).unwrap_or_default();
            // A call's parentheses (`pair()`) are not part of the name.
            let inner = inner.trim();
            let inner = inner.strip_suffix("()").unwrap_or(inner);
            // Past the closing backtick, if there is one.
            (inner, Shape::Code, (j + 1).min(chars.len()))
        } else if let Some(close) = quoted_name(&chars, i) {
            // One identifier in quotes ('Pinned', "out_dir()") is code, as
            // in backticks: people quote names that way too.
            let inner = text.get(end_of(i + 1)..end_of(close)).unwrap_or_default();
            let inner = inner.strip_suffix("()").unwrap_or(inner);
            (inner, Shape::Code, close + 1)
        } else if is_word_char(c) && c != '\'' && c != '-' {
            // A word (a leading apostrophe opens a quote, and a leading
            // hyphen is a dash, so both are punctuation), joined across `::` and `.` when both sides are word characters.
            let mut j = i;
            loop {
                while chars.get(j).is_some_and(|&(_, d)| is_word_char(d)) {
                    j += 1;
                }
                match (chars.get(j), chars.get(j + 1), chars.get(j + 2)) {
                    (Some(&(_, ':')), Some(&(_, ':')), Some(&(_, d))) if is_word_char(d) => j += 2,
                    (Some(&(_, '.')), Some(&(_, d)), _) if d.is_alphanumeric() || d == '_' => {
                        j += 1
                    }
                    _ => break,
                }
            }
            // Trailing apostrophes and hyphens belong to the punctuation, not the word.
            while j > i + 1
                && chars
                    .get(j - 1)
                    .is_some_and(|&(_, d)| d == '\'' || d == '-')
            {
                j -= 1;
            }
            // A word with "'s" that is no known word ("What's" is one) owns
            // what follows ("change App's `x`", "show me mobility's
            // fields"), and the "'s" is a word of its own; but in lower
            // case, a known word before it says it is short for "is"
            // ("there's").
            let lower = |k| {
                text.get(start..end_of(k))
                    .unwrap_or_default()
                    .to_lowercase()
            };
            if j > i + 2
                && chars.get(j - 2).is_some_and(|&(_, d)| d == '\'')
                && chars.get(j - 1).is_some_and(|&(_, d)| d == 's')
                && !known(&lower(j))
                && (c.is_uppercase() || !known(&lower(j - 2)))
            {
                j -= 2;
            }
            // A letter with a slash after it is one shortened word: "w/".
            if j == i + 1 && chars.get(j).is_some_and(|&(_, d)| d == '/') {
                j += 1;
            }
            // A known word with underscores after it ("what teh_ point") is
            // the word, the underscores a slip of the hand.
            let whole = text.get(start..end_of(j)).unwrap_or_default();
            let stem = whole.trim_end_matches('_');
            let word = if known(&stem.to_lowercase()) {
                stem
            } else {
                whole
            };
            // A call's parentheses after a name ("the fn yes() one") are not
            // part of it, and show it to be code.
            let call = text.get(end_of(j)..).is_some_and(|r| r.starts_with("()"));
            let shape = if call { Shape::Code } else { shape_of(word) };
            (word, shape, j + 2 * usize::from(call))
        } else if c == '\''
            && chars.get(i + 1).is_some_and(|&(_, d)| d == 's')
            && !chars.get(i + 2).is_some_and(|&(_, d)| is_word_char(d))
            && tokens.last().is_some_and(|t: &Token| t.end == start)
        {
            // The "'s" of a name owning what follows: "App's", "`App`'s".
            let word = text.get(start..end_of(i + 2)).unwrap_or_default();
            (word, Shape::Word, i + 2)
        } else {
            // One punctuation character; lowercasing leaves it as it is.
            let p = text.get(start..end_of(i + 1)).unwrap_or_default();
            (p, Shape::Punct, i + 1)
        };
        // An empty pair of backticks is no token.
        if !word.is_empty() {
            tokens.push(Token {
                text: word.to_owned(),
                norm: word.to_lowercase(),
                shape,
                start,
                end: end_of(next),
            });
        }
        i = next;
    }
    tokens
}

fn shape_of(word: &str) -> Shape {
    let digit = |c: char| c.is_ascii_digit();
    if word.chars().all(digit) {
        Shape::Number
    } else if word.contains(['_', ':', '.'])
        || word.contains(digit)
        || word.chars().skip(1).any(char::is_uppercase)
    {
        // A word holds `:` only as part of `::`.
        Shape::Code
    } else {
        Shape::Word
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_words_code_and_punctuation() {
        let t = tokenize(
            "What calls `spawn_wave`? rename Player::hp, ok player.hp's w/ Option/Vec App's What's",
            &|w| w == "what's",
        );
        let shapes: Vec<(String, Shape)> = t.iter().map(|t| (t.text.clone(), t.shape)).collect();
        assert_eq!(
            shapes,
            vec![
                ("What".into(), Shape::Word),
                ("calls".into(), Shape::Word),
                ("spawn_wave".into(), Shape::Code),
                ("?".into(), Shape::Punct),
                ("rename".into(), Shape::Word),
                ("Player::hp".into(), Shape::Code),
                (",".into(), Shape::Punct),
                ("ok".into(), Shape::Word),
                ("player.hp".into(), Shape::Code),
                ("'s".into(), Shape::Word),
                ("w/".into(), Shape::Word),
                ("Option".into(), Shape::Word),
                ("/".into(), Shape::Punct),
                ("Vec".into(), Shape::Word),
                ("App".into(), Shape::Word),
                ("'s".into(), Shape::Word),
                ("What's".into(), Shape::Word),
            ]
        );
    }
}
