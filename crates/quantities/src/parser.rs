use crate::inventory::{Change, Entity, Inventory, ObjectKind};
use crate::{
    calendar::Date,
    rational::{Rational, Result},
    units::{
        AdditiveKind, Dimension, Duration, Length, Mass, Quantity, Speed, TypedUnit, Unit, Volume,
    },
};
use regex::{Captures, Regex};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

type Solved = (String, Option<String>, &'static str);
const NUMBER: &str = r"(-?\d+(?:\.\d+|/\d+)?)";

fn capture<'a>(pattern: &str, text: &'a str) -> Result<Option<Captures<'a>>> {
    // Patterns are internal grammar productions, never supplied by callers.
    static CACHE: OnceLock<Mutex<HashMap<String, Regex>>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .map_err(|_| "parser cache unavailable")?;
    if !cache.contains_key(pattern) {
        cache.insert(
            pattern.into(),
            Regex::new(pattern).map_err(|e| e.to_string())?,
        );
    }
    Ok(cache
        .get(pattern)
        .ok_or("missing grammar production")?
        .captures(text))
}
fn field<'a>(c: &Captures<'a>, n: usize) -> Result<&'a str> {
    c.get(n)
        .map(|m| m.as_str())
        .ok_or("incomplete parse".into())
}
fn num(c: &Captures<'_>, n: usize) -> Result<Rational> {
    field(c, n)?.parse()
}
fn nonnegative_num(c: &Captures<'_>, n: usize) -> Result<Rational> {
    let value = num(c, n)?;
    if value.is_negative() {
        return Err("negative count, price, or elapsed duration".into());
    }
    Ok(value)
}
fn numeric(value: Rational, method: &'static str) -> Solved {
    (value.to_string(), None, method)
}

pub fn solve(question: &str) -> Result<Solved> {
    if question.len() > 4096 {
        return Err("question too long".into());
    }
    let text = question.trim().trim_end_matches(['?', '.']).to_lowercase();
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if let Some(c) = capture(&format!(r"^convert {NUMBER} (.+) (?:to|into) (.+)$"), &text)? {
        return conversion(num(&c, 1)?, field(&c, 2)?, field(&c, 3)?);
    }
    if let Some(c) = capture(
        &format!(r"^how many (.+?) (?:are |is )?in {NUMBER} (.+)$"),
        &text,
    )? {
        return conversion(num(&c, 2)?, field(&c, 3)?, field(&c, 1)?);
    }
    if let Some(c) = capture(
        &format!(r"^what is {NUMBER} (.+?) plus {NUMBER} (.+?) in (.+)$"),
        &text,
    )? {
        let source = Unit::parse(field(&c, 2)?)?;
        let args = (
            num(&c, 1)?,
            field(&c, 2)?,
            num(&c, 3)?,
            field(&c, 4)?,
            field(&c, 5)?,
        );
        let value = match source.dimension() {
            Dimension::Length => add_quantities::<Length>(args)?,
            Dimension::Mass => add_quantities::<Mass>(args)?,
            Dimension::Duration => add_quantities::<Duration>(args)?,
            Dimension::Volume => add_quantities::<Volume>(args)?,
            Dimension::Speed => add_quantities::<Speed>(args)?,
            Dimension::Temperature => return Err("absolute temperatures cannot be added".into()),
        };
        return Ok((
            value.to_string(),
            Some(field(&c, 5)?.into()),
            "sum of like dimensions after exact unit conversion",
        ));
    }
    if let Some(c) = capture(
        &format!(
            r"^a [a-z]+ travels {NUMBER} (.+?) in {NUMBER} (.+?)\. what is its speed in (.+)$"
        ),
        &text,
    )? {
        let distance =
            Quantity::<Length>::new(nonnegative_num(&c, 1)?, TypedUnit::parse(field(&c, 2)?)?);
        let duration =
            Quantity::<Duration>::new(nonnegative_num(&c, 3)?, TypedUnit::parse(field(&c, 4)?)?);
        let value = distance
            .divided_by_duration(&duration)?
            .in_unit(&TypedUnit::parse(field(&c, 5)?)?)?;
        return Ok((
            value.to_string(),
            Some(field(&c, 5)?.into()),
            "typed length / duration = speed",
        ));
    }
    if let Some(c) = capture(
        &format!(
            r"^a [a-z]+ travels at {NUMBER} (.+?) for {NUMBER} (.+?)\. how far does it travel in (.+)$"
        ),
        &text,
    )? {
        let speed =
            Quantity::<Speed>::new(nonnegative_num(&c, 1)?, TypedUnit::parse(field(&c, 2)?)?);
        let duration =
            Quantity::<Duration>::new(nonnegative_num(&c, 3)?, TypedUnit::parse(field(&c, 4)?)?);
        let value = speed
            .multiplied_by_duration(&duration)?
            .in_unit(&TypedUnit::parse(field(&c, 5)?)?)?;
        return Ok((
            value.to_string(),
            Some(field(&c, 5)?.into()),
            "typed speed * duration = length",
        ));
    }
    if let Some(c) = capture(
        r"^(?:what date is )?(\d+) (days?|weeks?|months?|years?) (after|before) (\d{4}-\d{2}-\d{2})$",
        &text,
    )? {
        let count: i64 = field(&c, 1)?.parse().map_err(|_| "date count overflow")?;
        let count = if field(&c, 3)? == "before" {
            -count
        } else {
            count
        };
        let date: Date = field(&c, 4)?.parse()?;
        let result = match field(&c, 2)? {
            "day" | "days" => date.add_days(count)?,
            "week" | "weeks" => date.add_days(count.checked_mul(7).ok_or("date overflow")?)?,
            "month" | "months" => date.add_months(count)?,
            _ => date.add_months(count.checked_mul(12).ok_or("date overflow")?)?,
        };
        return Ok((
            result.to_string(),
            None,
            "Gregorian calendar; month/year shifts clamp to month end",
        ));
    }
    if let Some(c) = capture(
        r"^how many days (?:are there )?(from|between) (\d{4}-\d{2}-\d{2}) (to|and) (\d{4}-\d{2}-\d{2})$",
        &text,
    )? {
        let from: Date = field(&c, 2)?.parse()?;
        let to: Date = field(&c, 4)?.parse()?;
        let days = match (field(&c, 1)?, field(&c, 3)?) {
            ("from", "to") => from.days_until(to),
            ("between", "and") => from.days_until(to).abs(),
            _ => return Err("inconsistent date relation".into()),
        };
        return Ok(numeric(
            Rational::integer(days),
            "elapsed Gregorian days; from/to is signed, between/and is absolute; excludes start",
        ));
    }
    if let Some(c) = capture(
        r"^what (?:day of the week|weekday) is (\d{4}-\d{2}-\d{2})$",
        &text,
    )? {
        let date: Date = field(&c, 1)?.parse()?;
        return Ok((date.weekday().into(), None, "Gregorian weekday"));
    }
    if let Ok(Some(value)) = crate::linear::solve(&text) {
        return Ok(numeric(value, "exact scalar linear constraints"));
    }
    if let Some((value, is_ratio)) = crate::percent::solve(&text)? {
        return Ok((
            value.to_string(),
            is_ratio.then(|| "percent".into()),
            "exact percentage relation",
        ));
    }
    if let Some(c) = capture(
        &format!(
            r"^(?:what is |calculate )?(?:the )?(sum|difference|product|quotient) of {NUMBER} and {NUMBER}$"
        ),
        &text,
    )? {
        let (left, right) = (num(&c, 2)?, num(&c, 3)?);
        let value = match field(&c, 1)? {
            "sum" => left.checked_add(right)?,
            "difference" => left.checked_sub(right)?,
            "product" => left.checked_mul(right)?,
            _ => left.checked_div(right)?,
        };
        return Ok(numeric(value, "named binary rational operation"));
    }
    if let Some(value) = crate::ratio_total::solve(&text)? {
        return Ok(numeric(value, "exact checked ratio and total"));
    }
    if let Some(answer) = word_problem(&text)? {
        return Ok(answer);
    }
    let expression = text
        .strip_prefix("what is ")
        .or_else(|| text.strip_prefix("calculate "))
        .or_else(|| text.strip_prefix("evaluate "))
        .unwrap_or(&text);
    let expression = expression
        .replace("multiplied by", "*")
        .replace("divided by", "/")
        .replace("plus", "+")
        .replace("minus", "-")
        .replace("times", "*");
    let tokens = lex(&expression)?;
    let mut parser = Expression {
        tokens: tokens.iter().peekable(),
        depth: 0,
    };
    let value = parser.sum()?;
    if parser.tokens.next().is_some() {
        return Err("unconsumed expression or unsupported English".into());
    }
    Ok(numeric(
        value,
        "exact rational expression with standard precedence",
    ))
}

fn add_quantities<D: AdditiveKind>(
    (a, a_unit, b, b_unit, target): (Rational, &str, Rational, &str, &str),
) -> Result<Rational> {
    let a = Quantity::<D>::new(a, TypedUnit::parse(a_unit)?);
    let b = Quantity::<D>::new(b, TypedUnit::parse(b_unit)?);
    a.checked_add(&b)?.in_unit(&TypedUnit::parse(target)?)
}

fn conversion(value: Rational, from: &str, to: &str) -> Result<Solved> {
    let source = Unit::parse(from)?;
    let target = Unit::parse(to)?;
    Ok((
        source.convert(value, &target)?.to_string(),
        Some(to.into()),
        "dimension-checked exact SI conversion",
    ))
}

fn word_problem(text: &str) -> Result<Option<Solved>> {
    // Entities and object kinds are bound and checked, rather than discarded.
    if let Some(c) = capture(
        &format!(
            r"^([a-z]+) has {NUMBER} ([a-z]+)\. ([a-z]+) has {NUMBER} ([a-z]+)\. how many more ([a-z]+) does ([a-z]+) have than ([a-z]+)$"
        ),
        text,
    )? {
        let left = Inventory::new(
            Entity(field(&c, 1)?.into()),
            ObjectKind(field(&c, 3)?.into()),
            num(&c, 2)?,
        )?;
        let right = Inventory::new(
            Entity(field(&c, 4)?.into()),
            ObjectKind(field(&c, 6)?.into()),
            num(&c, 5)?,
        )?;
        if left.owner == right.owner {
            return Err("comparison requires distinct entities".into());
        }
        if field(&c, 7)? != left.kind.0 {
            return Err("comparison asks for a different object kind".into());
        }
        let result = if field(&c, 8)? == left.owner.0 && field(&c, 9)? == right.owner.0 {
            left.difference(&right)?
        } else if field(&c, 8)? == right.owner.0 && field(&c, 9)? == left.owner.0 {
            right.difference(&left)?
        } else {
            return Err("comparison asks about an unknown entity".into());
        };
        return Ok(Some(numeric(
            result,
            "entity-bound comparison of matching object kinds",
        )));
    }
    if let Some(result) = inventory_problem(text)? {
        return Ok(Some(numeric(
            result,
            "entity-bound sequence of inventory events",
        )));
    }
    if let Some(c) = capture(
        &format!(
            r"^there are {NUMBER} ([a-z]+) with {NUMBER} ([a-z]+) (?:in each|each)\. how many ([a-z]+) are there (?:in total|altogether)$"
        ),
        text,
    )? {
        if field(&c, 4)? != field(&c, 5)? {
            return Err("word problem asks for a different object kind".into());
        }
        return Ok(Some(numeric(
            nonnegative_num(&c, 1)?.checked_mul(nonnegative_num(&c, 3)?)?,
            "equal groups: count times items per group",
        )));
    }
    if let Some(c) = capture(
        &format!(
            r"^{NUMBER} ([a-z]+) are shared equally among {NUMBER} ([a-z]+)\. how many ([a-z]+) does each ([a-z]+) get$"
        ),
        text,
    )? {
        if field(&c, 2)? != field(&c, 5)? {
            return Err("word problem asks for a different object kind".into());
        }
        if !same_kind(field(&c, 6)?, field(&c, 4)?) {
            return Err("sharing asks about a different recipient kind".into());
        }
        return Ok(Some(numeric(
            nonnegative_num(&c, 1)?.checked_div(nonnegative_num(&c, 3)?)?,
            "equal sharing: items divided by recipients",
        )));
    }
    if let Some(c) = capture(
        &format!(r"^each ([a-z]+) costs {NUMBER} dollars\. what is the cost of {NUMBER} ([a-z]+)$"),
        text,
    )? {
        let singular = field(&c, 1)?;
        let plural = field(&c, 4)?;
        if plural != singular && plural != format!("{singular}s") {
            return Err("cost refers to a different object kind".into());
        }
        return Ok(Some((
            nonnegative_num(&c, 2)?
                .checked_mul(nonnegative_num(&c, 3)?)?
                .to_string(),
            Some("dollars".into()),
            "price per item times item count",
        )));
    }
    Ok(None)
}

fn same_kind(singular: &str, plural: &str) -> bool {
    singular == plural
        || plural == format!("{singular}s")
        || matches!(
            (singular, plural),
            ("child", "children") | ("person", "people") | ("mouse", "mice")
        )
}

fn inventory_problem(text: &str) -> Result<Option<Rational>> {
    // Sentence boundary is dot + space, so decimal points remain inside numbers.
    let mut sentences = text.split(". ").collect::<Vec<_>>();
    if sentences.len() < 2 {
        return Ok(None);
    }
    let question = sentences.pop().ok_or("missing question")?;
    let mut sentences = sentences.into_iter();
    let initial = sentences.next().ok_or("missing inventory")?;
    let Some(c) = capture(&format!(r"^([a-z]+) has {NUMBER} ([a-z]+)$"), initial)? else {
        return Ok(None);
    };
    let mut inventory = Inventory::new(
        Entity(field(&c, 1)?.into()),
        ObjectKind(field(&c, 3)?.into()),
        num(&c, 2)?,
    )?;
    for sentence in sentences {
        let c = capture(&format!(r"^([a-z]+) (gets|buys|finds|receives|loses|sells|gives away|eats|uses) {NUMBER}(?: more)? ([a-z]+)$"), sentence)?.ok_or("unsupported inventory event")?;
        let subject = field(&c, 1)?;
        if subject != inventory.owner.0 && !matches!(subject, "he" | "she" | "they") {
            return Err("unknown inventory event subject".into());
        }
        let amount = num(&c, 3)?;
        let change = match field(&c, 2)? {
            "gets" | "buys" | "finds" | "receives" => Change::Receive(amount),
            _ => Change::Remove(amount),
        };
        inventory.apply(
            &inventory.owner.clone(),
            &ObjectKind(field(&c, 4)?.into()),
            change,
        )?;
    }
    let c = capture(
        r"^how many ([a-z]+) (?:does|do) ([a-z]+) have (?:now|left)$",
        question,
    )?
    .ok_or("unsupported inventory question")?;
    let subject = field(&c, 2)?;
    if field(&c, 1)? != inventory.kind.0
        || (subject != inventory.owner.0 && !matches!(subject, "he" | "she" | "they"))
    {
        return Err("inventory question entity or object mismatch".into());
    }
    Ok(Some(inventory.count()))
}

#[derive(Clone, Debug)]
enum Token {
    Number(Rational),
    Plus,
    Minus,
    Multiply,
    Divide,
    Left,
    Right,
}

fn lex(text: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        let token = match ch {
            ch if ch.is_whitespace() => continue,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Multiply,
            '/' => Token::Divide,
            '(' => Token::Left,
            ')' => Token::Right,
            ch if ch.is_ascii_digit() || ch == '.' => {
                let mut number = String::from(ch);
                while let Some(next) = chars.peek() {
                    if !next.is_ascii_digit() && *next != '.' {
                        break;
                    }
                    number.push(*next);
                    chars.next();
                }
                Token::Number(number.parse()?)
            }
            ch if ch.is_ascii_alphabetic() => {
                let mut word = String::from(ch);
                while let Some(next) = chars.peek().copied() {
                    let hyphen = next == '-'
                        && word.ends_with(|c: char| c.is_ascii_alphabetic())
                        && chars
                            .clone()
                            .nth(1)
                            .is_some_and(|c| c.is_ascii_alphabetic());
                    if !next.is_ascii_alphabetic() && !next.is_whitespace() && !hyphen {
                        break;
                    }
                    word.push(next);
                    chars.next();
                }
                let value = crate::number_words::parse_cardinal(&word)?;
                Token::Number(Rational::integer(value))
            }
            _ => return Err("unsupported expression character".into()),
        };
        tokens.push(token);
        if tokens.len() > 256 {
            return Err("expression too complex".into());
        }
    }
    Ok(tokens)
}

struct Expression<'a> {
    tokens: std::iter::Peekable<std::slice::Iter<'a, Token>>,
    depth: u32,
}
impl Expression<'_> {
    fn sum(&mut self) -> Result<Rational> {
        let mut value = self.product()?;
        loop {
            match self.tokens.peek() {
                Some(Token::Plus) => {
                    self.tokens.next();
                    value = value.checked_add(self.product()?)?;
                }
                Some(Token::Minus) => {
                    self.tokens.next();
                    value = value.checked_sub(self.product()?)?;
                }
                _ => return Ok(value),
            }
        }
    }
    fn product(&mut self) -> Result<Rational> {
        let mut value = self.atom()?;
        loop {
            match self.tokens.peek() {
                Some(Token::Multiply) => {
                    self.tokens.next();
                    value = value.checked_mul(self.atom()?)?;
                }
                Some(Token::Divide) => {
                    self.tokens.next();
                    value = value.checked_div(self.atom()?)?;
                }
                _ => return Ok(value),
            }
        }
    }
    fn atom(&mut self) -> Result<Rational> {
        self.depth += 1;
        if self.depth > 64 {
            return Err("expression nesting too deep".into());
        }
        let result = match self.tokens.next() {
            Some(Token::Number(n)) => Ok(*n),
            Some(Token::Minus) => self.atom()?.checked_neg(),
            Some(Token::Plus) => self.atom(),
            Some(Token::Left) => {
                let value = self.sum()?;
                match self.tokens.next() {
                    Some(Token::Right) => Ok(value),
                    _ => Err("unclosed parentheses".into()),
                }
            }
            _ => Err("expected a number".into()),
        };
        self.depth -= 1;
        result
    }
}
