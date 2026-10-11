//! Checked percentage relations. This grammar consumes every token and never
//! infers operations by collecting numbers from otherwise unsupported prose.
use crate::rational::{Rational, Result};

fn number(text: &str) -> Result<Rational> {
    let text = text.trim();
    // Thousands grouping is accepted only in a numeric atom, never by globally
    // deleting commas (which could turn a list into one number).
    let mut parts = text.split('/');
    let top = decimal(parts.next().ok_or("missing numerator")?)?;
    if let Some(bottom) = parts.next() {
        if parts.next().is_some() {
            return Err("too many fraction separators".into());
        }
        top.checked_div(decimal(bottom)?)
    } else {
        Ok(top)
    }
}

fn decimal(text: &str) -> Result<Rational> {
    let text = text.trim();
    if text.contains(',') {
        let unsigned = text.strip_prefix('-').unwrap_or(text);
        let integer = unsigned.split('.').next().ok_or("missing integer")?;
        let groups: Vec<_> = integer.split(',').collect();
        if !groups.first().is_some_and(|g| (1..=3).contains(&g.len()))
            || groups.iter().skip(1).any(|g| g.len() != 3)
            || groups
                .iter()
                .any(|g| !g.bytes().all(|c| c.is_ascii_digit()))
            || unsigned
                .split_once('.')
                .is_some_and(|(_, f)| f.contains(','))
        {
            return Err("invalid thousands grouping".into());
        }
    }
    text.replace(',', "").parse()
}

fn amount(text: &str) -> Result<Rational> {
    let mut result = Rational::integer(1);
    // 'of' composes fractions and percentages, not arbitrary English clauses.
    let mut count = 0;
    for atom in text.split(" of ") {
        count += 1;
        if count > 16 {
            return Err("percentage expression too long".into());
        }
        let atom = atom.trim();
        let percent = atom
            .strip_suffix('%')
            .or_else(|| atom.strip_suffix(" percent"));
        let value = if let Some(n) = percent {
            number(n)?.checked_div(Rational::integer(100))?
        } else {
            number(atom)?
        };
        result = result.checked_mul(value)?;
    }
    Ok(result)
}

pub(crate) fn solve(text: &str) -> Result<Option<(Rational, bool)>> {
    if !text.contains('%') && !text.contains("percent") {
        return Ok(None);
    }
    let text = text.trim();
    let ratio = if let Some(rest) = text
        .strip_prefix("what percent of ")
        .or_else(|| text.strip_prefix("what percentage of "))
    {
        let (base, part) = rest
            .split_once(" is ")
            .ok_or("missing percentage relation")?;
        Some((part, base))
    } else if let Some(rest) = text.strip_prefix("what percent is ") {
        let (part, base) = rest.split_once(" of ").ok_or("missing percentage base")?;
        Some((part, base))
    } else {
        text.split_once(" is what percent of ")
            .or_else(|| text.split_once(" is what % of "))
    };
    if let Some((part, base)) = ratio {
        let value = amount(part)?
            .checked_div(amount(base)?)?
            .checked_mul(Rational::integer(100))?;
        return Ok(Some((value, true)));
    }
    let text = text
        .strip_prefix("what is ")
        .or_else(|| text.strip_prefix("calculate "))
        .or_else(|| text.strip_prefix("evaluate "))
        .unwrap_or(text);
    let text = text
        .strip_suffix(" is")
        .or_else(|| text.strip_suffix(" ="))
        .unwrap_or(text)
        .trim();
    // Requiring 'of' avoids silently interpreting an isolated percent as an
    // absolute number, and keeps the existing arithmetic parser authoritative.
    if !text.contains(" of ") {
        return Ok(None);
    }
    Ok(Some((amount(text)?, false)))
}
