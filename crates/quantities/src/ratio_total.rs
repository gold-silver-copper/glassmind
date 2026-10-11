//! Fully consumed ratio/sum relations over abstract numbers, with checked slots.
use crate::rational::{Rational, Result};
use regex::Regex;
use std::sync::OnceLock;

const ATOM: &str = r"[0-9]+(?:\.[0-9]+|/[0-9]+)?";
const SCALAR: &str = r"[0-9]+(?:\.[0-9]+)?";
const COUNT: &str = r"two|three|four|2|3|4";

enum Selector {
    Index(usize),
    Minimum,
    Maximum,
}

struct RatioTotal {
    weights: Vec<Rational>,
    total: Rational,
    selector: Selector,
}

impl RatioTotal {
    fn execute(self) -> Result<Rational> {
        if !(2..=4).contains(&self.weights.len()) {
            return Err("ratio arity outside supported range".into());
        }
        let mut sum = Rational::integer(0);
        for weight in &self.weights {
            positive(*weight)?;
            sum = sum.checked_add(*weight)?;
        }
        positive(self.total)?;
        let selected = match self.selector {
            Selector::Index(i) => *self.weights.get(i).ok_or("ordinal outside ratio")?,
            Selector::Minimum | Selector::Maximum => {
                let mut selected = *self.weights.first().ok_or("empty ratio")?;
                for weight in self.weights.iter().skip(1) {
                    let smaller = weight.checked_sub(selected)?.is_negative();
                    if smaller == matches!(self.selector, Selector::Minimum) {
                        selected = *weight;
                    }
                }
                selected
            }
        };
        self.total.checked_mul(selected)?.checked_div(sum)
    }
}

fn positive(value: Rational) -> Result<()> {
    if value.is_negative() || value == Rational::integer(0) {
        Err("ratio weights and total must be positive".into())
    } else {
        Ok(())
    }
}

fn count(text: &str) -> Result<usize> {
    match text {
        "two" | "2" => Ok(2),
        "three" | "3" => Ok(3),
        "four" | "4" => Ok(4),
        _ => Err("unsupported ratio count".into()),
    }
}

fn grammar() -> Result<&'static Vec<Regex>> {
    static GRAMMAR: OnceLock<Result<Vec<Regex>>> = OnceLock::new();
    GRAMMAR.get_or_init(|| {
        let ratios = format!(r"(?P<weights>{ATOM}(?:\s*(?::|to)\s*{ATOM}){{1,3}})");
        let total = format!(r"(?P<total>{ATOM})");
        let n = format!(r"(?P<count>{COUNT})");
        let pair = r"\s*(?::|/)\s*";
        [
            format!(r"^the ratio of {n} numbers is {ratios} and their sum is {total}\s*\.\s*(?P<query>.+)$"),
            format!(r"^{n} numbers are in the ratio {ratios}\s*(?:\.\s*)?(?:and )?(?:their sum is |add up to give a sum of ){total}\s*\.\s*(?P<query>.+)$"),
            format!(r"^the sum of (?:{n} )?(?:the )?numbers is {total}\s*\.\s*(?:if )?the ratio (?:between (?:the )?first and (?:the )?second be|of the first to the second is) (?P<a>{SCALAR}){pair}(?P<b>{SCALAR})\s*,?\s*and (?:that between (?:the )?second and (?:the )?third be|that of the second to the third is|the ratio of the second to the third is) (?P<c>{SCALAR}){pair}(?P<d>{SCALAR})\s*[,\.]?\s*(?:then )?(?P<query>.+)$"),
        ].iter().map(|s| Regex::new(s).map_err(|e| e.to_string())).collect()
    }).as_ref().map_err(Clone::clone)
}

fn query(text: &str, arity: usize) -> Result<Selector> {
    static QUERY: OnceLock<Result<Regex>> = OnceLock::new();
    let regex = QUERY.get_or_init(|| Regex::new(&format!(
        r"^(?:find the (?P<find>first|second|third|fourth|last|biggest|largest|smallest|greater|lesser) number|the (?P<ordinal>first|second|third|fourth|last|biggest|largest|smallest|greater|lesser) number(?: of the (?P<count>{COUNT}) numbers)? is|the (?P<rank>greater|lesser) of the (?P<rank_count>{COUNT}) numbers is)\s*[:?\.]?$"
    )).map_err(|e| e.to_string())).as_ref().map_err(Clone::clone)?;
    let captures = regex.captures(text).ok_or("unsupported ratio query")?;
    if let Some(n) = captures
        .name("count")
        .or_else(|| captures.name("rank_count"))
        && count(n.as_str())? != arity
    {
        return Err("query count disagrees with ratio".into());
    }
    let selector = captures
        .name("find")
        .or_else(|| captures.name("ordinal"))
        .or_else(|| captures.name("rank"))
        .ok_or("missing ratio selector")?
        .as_str();
    if matches!(selector, "greater" | "lesser") && arity != 2 {
        return Err("comparative selector requires two numbers".into());
    }
    Ok(match selector {
        "first" => Selector::Index(0),
        "second" => Selector::Index(1),
        "third" => Selector::Index(2),
        "fourth" => Selector::Index(3),
        "last" => Selector::Index(arity - 1),
        "biggest" | "largest" | "greater" => Selector::Maximum,
        "smallest" | "lesser" => Selector::Minimum,
        _ => return Err("unsupported ratio selector".into()),
    })
}

pub(crate) fn solve(text: &str) -> Result<Option<Rational>> {
    if !text.contains("ratio") || !text.contains("numbers") {
        return Ok(None);
    }
    for grammar in grammar()? {
        let Some(c) = grammar.captures(text.trim()) else {
            continue;
        };
        let get = |key| {
            c.name(key)
                .map(|m| m.as_str())
                .ok_or("missing ratio slot".to_owned())
        };
        let weights = if let Some(raw) = c.name("weights") {
            let weights: Vec<Rational> = raw
                .as_str()
                .replace("to", ":")
                .split(':')
                .map(|s| s.trim().parse())
                .collect::<Result<_>>()?;
            if count(get("count")?)? != weights.len() {
                return Err("declared count disagrees with ratio".into());
            }
            weights
        } else {
            if let Some(n) = c.name("count")
                && count(n.as_str())? != 3
            {
                return Err("adjacent ratio chain requires three numbers".into());
            }
            let a: Rational = get("a")?.parse()?;
            let b: Rational = get("b")?.parse()?;
            let c: Rational = get("c")?.parse()?;
            let d: Rational = get("d")?.parse()?;
            for value in [a, b, c, d] {
                positive(value)?;
            }
            vec![a.checked_mul(c)?, b.checked_mul(c)?, b.checked_mul(d)?]
        };
        let selector = query(get("query")?, weights.len())?;
        return RatioTotal {
            weights,
            total: get("total")?.parse()?,
            selector,
        }
        .execute()
        .map(Some);
    }
    Ok(None)
}
