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

/// Splits `text` into tokens.
pub fn tokenize(text: &str) -> Vec<Token> {
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
            // Past the closing backtick, if there is one.
            (inner.trim(), Shape::Code, (j + 1).min(chars.len()))
        } else if is_word_char(c) && c != '\'' {
            // A word (a leading apostrophe opens a quote, so is punctuation), joined across `::` and `.` when both sides are word characters.
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
            let word = text.get(start..end_of(j)).unwrap_or_default();
            (word, shape_of(word), j)
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
        let t = tokenize("What calls `spawn_wave`? rename Player::hp, ok player.hp's");
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
                ("player.hp's".into(), Shape::Code),
            ]
        );
    }
}
