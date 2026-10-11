//! One-quantity directed changes with explicit before/after stage binding.
use crate::rational::{Rational, Result};
use regex::Regex;

const N: &str = r"(-?\d+(?:\.\d+|/\d+)?)";
const P: &str = r"\s*(?:%|percent|per cent)";

#[derive(Clone, Copy)]
enum Direction {
    Increase,
    Decrease,
}
impl Direction {
    fn parse(s: &str) -> Self {
        if s.starts_with("increas") {
            Self::Increase
        } else {
            Self::Decrease
        }
    }
}
struct Change {
    direction: Direction,
    percent: Rational,
}
impl Change {
    fn factor(&self) -> Result<Rational> {
        if self.percent.is_negative() {
            return Err("negative change magnitude".into());
        }
        let fraction = self.percent.checked_div(Rational::integer(100))?;
        let factor = match self.direction {
            Direction::Increase => Rational::integer(1).checked_add(fraction)?,
            Direction::Decrease => Rational::integer(1).checked_sub(fraction)?,
        };
        if factor.is_negative() {
            return Err("decrease exceeds nonnegative quantity".into());
        }
        Ok(factor)
    }
}
struct Stages {
    factor: Rational,
}
impl Stages {
    fn new() -> Self {
        Self {
            factor: Rational::integer(1),
        }
    }
    fn append(&mut self, change: Change) -> Result<()> {
        self.factor = self.factor.checked_mul(change.factor()?)?;
        Ok(())
    }
    fn value(&self, known: Rational, inverse: bool) -> Result<Rational> {
        if known.is_negative() {
            return Err("negative base".into());
        }
        if inverse {
            known.checked_div(self.factor)
        } else {
            known.checked_mul(self.factor)
        }
    }
    fn percent(&self, direction: Direction) -> Result<Rational> {
        let signed = self
            .factor
            .checked_sub(Rational::integer(1))?
            .checked_mul(Rational::integer(100))?;
        let result = match direction {
            Direction::Increase => signed,
            Direction::Decrease => signed.checked_neg()?,
        };
        if result.is_negative() {
            return Err("requested direction conflicts with change".into());
        }
        Ok(result)
    }
}
fn re(pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|e| e.to_string())
}
fn change(direction: &str, percent: &str) -> Result<Change> {
    Ok(Change {
        direction: Direction::parse(direction),
        percent: percent.parse()?,
    })
}

pub(crate) fn solve(text: &str) -> Result<Option<(Rational, bool)>> {
    if !["increas", "decreas", "reduced"]
        .iter()
        .any(|word| text.contains(word))
    {
        return Ok(None);
    }
    // No comma or sentence deletion: separators are consumed by productions.
    let population = r"(?:a (?:town|village|city)|(?:the )?(?:town|village|city) [a-z]+)";
    let patterns = [
        format!(
            r"^the population of {population} is {N}\s*\. it (increases|decreases) annually at the rate of {N}{P}(?: p\s*\. a\s*\.)?\s*\.? what will be its population after (\d+) years?$"
        ),
        format!(
            r"^the present population of {population} is {N}\s*\. population (increase|decrease) rate is {N}{P} p\s*\. a\s*\. find the population of (?:the )?(?:town|village|city) (before|after) (\d+) years?$"
        ),
        format!(
            r"^if the annual (increase|decrease) in the population of {population} is {N}{P} and the present number of people is {N}\s*, what will the population be in (\d+) years?$"
        ),
    ];
    for (index, pattern) in patterns.iter().enumerate() {
        if let Some(c) = re(pattern)?.captures(text) {
            let (known, direction, percent, years, inverse) = match index {
                0 => (&c[1], &c[2], &c[3], &c[4], false),
                1 => (&c[1], &c[2], &c[3], &c[5], &c[4] == "before"),
                _ => (&c[3], &c[1], &c[2], &c[4], false),
            };
            if index == 1 {
                let subject = re(r"^the present population of (?:a |the )?(town|village|city)\b")?;
                let source = subject.captures(text).ok_or("missing population entity")?;
                let query = re(r"find the population of (?:the )?(town|village|city)\b")?;
                let target = query.captures(text).ok_or("missing population reference")?;
                if source[1] != target[1] {
                    return Err("conflicting population reference".into());
                }
            }
            let years: usize = years.parse().map_err(|_| "invalid year count")?;
            if years > 100 {
                return Err("too many annual stages".into());
            }
            let mut stages = Stages::new();
            // Check the rate even for a zero-year interval.
            change(direction, percent)?.factor()?;
            for _ in 0..years {
                stages.append(change(direction, percent)?)?;
            }
            let result = stages.value(known.parse()?, inverse)?;
            result.as_i64()?;
            return Ok(Some((result, false)));
        }
    }
    // The second clause explicitly names the new base, not the original base.
    let pattern = format!(
        r"^(increasing|decreasing) the original (price|cost) of (?:a|an) (?:certain )?[a-z]+ by {N}{P} and then (increasing|decreasing) the new (price|cost) by {N}{P} is equivalent to (increasing|decreasing) the original (price|cost) by what (?:percent|percentage|per cent)$"
    );
    if let Some(c) = re(&pattern)?.captures(text) {
        if c[2] != c[5] || c[2] != c[8] {
            return Err("conflicting quantity reference".into());
        }
        let mut stages = Stages::new();
        stages.append(change(&c[1], &c[3])?)?;
        stages.append(change(&c[4], &c[6])?)?;
        return Ok(Some((stages.percent(Direction::parse(&c[7]))?, true)));
    }
    let pattern = format!(
        r"^(?:if )?(a number|the price of (?:a|an) [a-z]+) is (?:first )?(increased|decreased|reduced) by {N}{P} and then (increased|decreased|reduced) by {N}{P}\s*\.(?: find the net (increase|decrease) (?:percent|per cent)| what is the (?:percent|percentage) (increase|decrease))$"
    );
    if let Some(c) = re(&pattern)?.captures(text) {
        let mut stages = Stages::new();
        stages.append(change(&c[2], &c[3])?)?;
        stages.append(change(&c[4], &c[5])?)?;
        let target = c
            .get(6)
            .or_else(|| c.get(7))
            .ok_or("missing direction")?
            .as_str();
        return Ok(Some((stages.percent(Direction::parse(target))?, true)));
    }
    let pattern =
        format!(r"^{N} is (increased|decreased|reduced) by {N}{P}\s*\. find the final number$");
    if let Some(c) = re(&pattern)?.captures(text) {
        let mut stages = Stages::new();
        stages.append(change(&c[2], &c[3])?)?;
        return Ok(Some((stages.value(c[1].parse()?, false)?, false)));
    }
    let pattern =
        format!(r"^a number (increased|decreased|reduced) by {N}{P} gives {N}\s*\. the number is$");
    if let Some(c) = re(&pattern)?.captures(text) {
        let mut stages = Stages::new();
        stages.append(change(&c[1], &c[2])?)?;
        return Ok(Some((stages.value(c[3].parse()?, true)?, false)));
    }
    Ok(None)
}
