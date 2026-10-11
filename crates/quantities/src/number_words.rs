//! Compositional English cardinal numbers through 999,999.
use crate::rational::Result;

fn atom(word: &str) -> Option<i64> {
    match word {
        "zero" => Some(0),
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "thirteen" => Some(13),
        "fourteen" => Some(14),
        "fifteen" => Some(15),
        "sixteen" => Some(16),
        "seventeen" => Some(17),
        "eighteen" => Some(18),
        "nineteen" => Some(19),
        "twenty" => Some(20),
        "thirty" => Some(30),
        "forty" => Some(40),
        "fifty" => Some(50),
        "sixty" => Some(60),
        "seventy" => Some(70),
        "eighty" => Some(80),
        "ninety" => Some(90),
        _ => None,
    }
}

fn small(words: &[&str]) -> Result<i64> {
    match words {
        [word] => atom(word).ok_or("unknown number word".into()),
        [tens, ones] => {
            let tens = atom(tens).ok_or("unknown tens word")?;
            let ones = atom(ones).ok_or("unknown ones word")?;
            if tens >= 20 && tens % 10 == 0 && (1..=9).contains(&ones) {
                Ok(tens + ones)
            } else {
                Err("invalid number word sequence".into())
            }
        }
        _ => Err("unsupported English cardinal".into()),
    }
}

fn hundreds(words: &[&str]) -> Result<i64> {
    match words {
        [first, "hundred", rest @ ..] => {
            let first = atom(first)
                .filter(|n| (1..=9).contains(n))
                .ok_or("invalid hundreds multiplier")?;
            if rest.is_empty() {
                return Ok(first * 100);
            }
            let rest = rest.strip_prefix(&["and"]).unwrap_or(rest);
            let tail = small(rest)?;
            if tail == 0 {
                return Err("redundant zero after hundreds".into());
            }
            Ok(first * 100 + tail)
        }
        _ => small(words),
    }
}

pub fn parse_cardinal(text: &str) -> Result<i64> {
    let words = text
        .split([' ', '-'])
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if let Some(index) = words.iter().position(|w| *w == "thousand") {
        let head = words.get(..index).ok_or("invalid thousands prefix")?;
        let tail = words.get(index + 1..).ok_or("invalid thousands suffix")?;
        let thousands = hundreds(head)?;
        if thousands == 0 {
            return Err("zero thousands".into());
        }
        if tail.is_empty() {
            return Ok(thousands * 1000);
        }
        let tail = tail.strip_prefix(&["and"]).unwrap_or(tail);
        let remainder = hundreds(tail)?;
        if remainder == 0 {
            return Err("redundant zero after thousands".into());
        }
        Ok(thousands * 1000 + remainder)
    } else {
        hundreds(&words)
    }
}
