//! Dimensionless affine expressions and fully consumed English equalities.
//! No numeric fragment extraction: every token must belong to a clause or target.
use crate::rational::{Rational, Result};

fn zero() -> Rational {
    Rational::integer(0)
}
fn one() -> Rational {
    Rational::integer(1)
}

#[derive(Clone, Copy)]
struct ScalarExpr {
    coefficients: [Rational; 2],
    constant: Rational,
}
impl ScalarExpr {
    fn constant(constant: Rational) -> Self {
        Self {
            coefficients: [zero(); 2],
            constant,
        }
    }
    fn variable(second: bool) -> Self {
        Self {
            coefficients: if second {
                [zero(), one()]
            } else {
                [one(), zero()]
            },
            constant: zero(),
        }
    }
    fn add(self, rhs: Self) -> Result<Self> {
        let [a, b] = self.coefficients;
        let [c, d] = rhs.coefficients;
        Ok(Self {
            coefficients: [a.checked_add(c)?, b.checked_add(d)?],
            constant: self.constant.checked_add(rhs.constant)?,
        })
    }
    fn scale(self, factor: Rational) -> Result<Self> {
        let [a, b] = self.coefficients;
        Ok(Self {
            coefficients: [a.checked_mul(factor)?, b.checked_mul(factor)?],
            constant: self.constant.checked_mul(factor)?,
        })
    }
    fn sub(self, rhs: Self) -> Result<Self> {
        self.add(rhs.scale(one().checked_neg()?)?)
    }
    fn value(self, values: [Rational; 2]) -> Result<Rational> {
        let [a, b] = self.coefficients;
        let [x, y] = values;
        self.constant
            .checked_add(a.checked_mul(x)?)?
            .checked_add(b.checked_mul(y)?)
    }
}
struct Equality(ScalarExpr);
struct Problem {
    constraints: Vec<Equality>,
    target: ScalarExpr,
    pair: bool,
    // Each chosen branch of an absolute difference must be nonnegative.
    differences: Vec<ScalarExpr>,
}
impl Problem {
    fn execute(self) -> Result<Option<Rational>> {
        // Exact elimination; all remaining equations are checked, including redundant ones.
        let mut rows: Vec<[Rational; 3]> = self
            .constraints
            .iter()
            .map(|Equality(e)| {
                let [a, b] = e.coefficients;
                [a, b, e.constant]
            })
            .collect();
        let variables = if self.pair { 2 } else { 1 };
        let mut rank = 0;
        let mut pivots = Vec::new();
        for column in 0..variables {
            let pivot = (rank..rows.len()).find(|&i| {
                rows.get(i)
                    .and_then(|r| r.get(column))
                    .is_some_and(|v| *v != zero())
            });
            let Some(pivot) = pivot else {
                continue;
            };
            rows.swap(rank, pivot);
            let row = *rows.get(rank).ok_or("missing pivot")?;
            let divisor = *row.get(column).ok_or("missing column")?;
            let [a, b, c] = row;
            let normalized = [
                a.checked_div(divisor)?,
                b.checked_div(divisor)?,
                c.checked_div(divisor)?,
            ];
            *rows.get_mut(rank).ok_or("missing pivot")? = normalized;
            for (i, other) in rows.iter_mut().enumerate() {
                if i == rank {
                    continue;
                }
                let factor = *other.get(column).ok_or("missing column")?;
                for (v, p) in other.iter_mut().zip(normalized) {
                    *v = v.checked_sub(factor.checked_mul(p)?)?;
                }
            }
            pivots.push(column);
            rank += 1;
        }
        if rows
            .iter()
            .any(|&[a, b, c]| a == zero() && b == zero() && c != zero())
        {
            return Ok(None);
        }
        if rank < variables {
            let reduce = |mut expression: ScalarExpr| -> Result<ScalarExpr> {
                for (&column, &[a, b, c]) in pivots.iter().zip(&rows) {
                    let factor = *expression
                        .coefficients
                        .get(column)
                        .ok_or("missing column")?;
                    expression = expression.sub(
                        ScalarExpr {
                            coefficients: [a, b],
                            constant: c,
                        }
                        .scale(factor)?,
                    )?;
                }
                Ok(expression)
            };
            let target = reduce(self.target)?;
            let signs = self
                .differences
                .into_iter()
                .map(reduce)
                .collect::<Result<Vec<_>>>()?;
            return projected_target(target, signs);
        }
        let x = rows
            .first()
            .ok_or("missing solution")?
            .get(2)
            .ok_or("missing constant")?
            .checked_neg()?;
        let y = if self.pair {
            rows.get(1)
                .ok_or("missing solution")?
                .get(2)
                .ok_or("missing constant")?
                .checked_neg()?
        } else {
            zero()
        };
        for Equality(e) in &self.constraints {
            if e.value([x, y])? != zero() {
                return Ok(None);
            }
        }
        for difference in &self.differences {
            if difference.value([x, y])?.is_negative() {
                return Ok(None);
            }
        }
        self.target.value([x, y]).map(Some)
    }
}

// Each row denotes a*x + b*y + c*target + d >= 0. Projection preserves
// existence over real numbers; it never assigns arbitrary values to free inputs.
fn projected_target(target: ScalarExpr, signs: Vec<ScalarExpr>) -> Result<Option<Rational>> {
    let [a, b] = target.coefficients;
    let mut rows = vec![
        [a, b, one().checked_neg()?, target.constant],
        [
            a.checked_neg()?,
            b.checked_neg()?,
            one(),
            target.constant.checked_neg()?,
        ],
    ];
    rows.extend(signs.into_iter().map(|e| {
        let [a, b] = e.coefficients;
        [a, b, zero(), e.constant]
    }));
    for column in 0..2 {
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        let mut remaining = Vec::new();
        for mut row in rows {
            let coefficient = *row.get(column).ok_or("missing projection column")?;
            if coefficient == zero() {
                remaining.push(row);
                continue;
            }
            let divisor = if coefficient.is_negative() {
                coefficient.checked_neg()?
            } else {
                coefficient
            };
            for value in &mut row {
                *value = value.checked_div(divisor)?;
            }
            if coefficient.is_negative() {
                negative.push(row);
            } else {
                positive.push(row);
            }
        }
        for lower in positive {
            for upper in &negative {
                let mut combined = lower;
                for (value, other) in combined.iter_mut().zip(upper) {
                    *value = value.checked_add(*other)?;
                }
                remaining.push(combined);
                if remaining.len() > 64 {
                    return Err("too many projected constraints".into());
                }
            }
        }
        rows = remaining;
    }
    let (mut lower, mut upper): (Option<Rational>, Option<Rational>) = (None, None);
    for [_, _, coefficient, constant] in rows {
        if coefficient == zero() {
            if constant.is_negative() {
                return Ok(None);
            }
            continue;
        }
        let bound = constant.checked_neg()?.checked_div(coefficient)?;
        if coefficient.is_negative() {
            if let Some(u) = upper {
                if bound.checked_cmp(u)?.is_lt() {
                    upper = Some(bound);
                }
            } else {
                upper = Some(bound);
            }
        } else if let Some(l) = lower {
            if bound.checked_cmp(l)?.is_gt() {
                lower = Some(bound);
            }
        } else {
            lower = Some(bound);
        }
    }
    match (lower, upper) {
        (Some(l), Some(u)) => match l.checked_cmp(u)? {
            std::cmp::Ordering::Equal => Ok(Some(l)),
            std::cmp::Ordering::Greater => Ok(None),
            std::cmp::Ordering::Less => {
                Err("scalar constraints have multiple target values".into())
            }
        },
        _ => Err("underdetermined scalar target".into()),
    }
}

struct Parser {
    tokens: Vec<String>,
    at: usize,
    single: bool,
    introduced: bool,
    pair: bool,
    choices: usize,
    differences: Vec<ScalarExpr>,
}
impl Parser {
    fn eat(&mut self, phrase: &str) -> bool {
        let words: Vec<_> = phrase.split_whitespace().collect();
        if self
            .tokens
            .get(self.at..self.at + words.len())
            .is_some_and(|s| s.iter().map(String::as_str).eq(words.iter().copied()))
        {
            self.at += words.len();
            true
        } else {
            false
        }
    }
    fn need(&mut self, phrase: &str) -> Result<()> {
        if self.eat(phrase) {
            Ok(())
        } else {
            Err(format!("expected {phrase}"))
        }
    }
    fn scalar(&mut self) -> Result<Option<Rational>> {
        let start = self.at;
        let negative = self.eat("-");
        let Some(token) = self.tokens.get(self.at) else {
            self.at = start;
            return Ok(None);
        };
        let value = match token.as_str() {
            "one" => one(),
            "two" => Rational::integer(2),
            "three" => Rational::integer(3),
            "four" => Rational::integer(4),
            "five" => Rational::integer(5),
            "six" => Rational::integer(6),
            "seven" => Rational::integer(7),
            "eight" => Rational::integer(8),
            "nine" => Rational::integer(9),
            "ten" => Rational::integer(10),
            _ if token.bytes().next().is_some_and(|c| c.is_ascii_digit()) => token.parse()?,
            _ => {
                self.at = start;
                return Ok(None);
            }
        };
        self.at += 1;
        let value = if self.eat("/") {
            let denominator = self
                .tokens
                .get(self.at)
                .ok_or("missing denominator")?
                .parse()?;
            self.at += 1;
            self.eat("th");
            value.checked_div(denominator)?
        } else {
            let saved = self.at;
            self.eat("-");
            let divisor = match self.tokens.get(self.at).map(String::as_str) {
                Some("half" | "halves") => 2,
                Some("third" | "thirds") => 3,
                Some("fourth" | "fourths" | "quarter" | "quarters") => 4,
                Some("fifth" | "fifths") => 5,
                Some("sixth" | "sixths") => 6,
                Some("seventh" | "sevenths") => 7,
                Some("eighth" | "eighths") => 8,
                Some("ninth" | "ninths") => 9,
                Some("tenth" | "tenths") => 10,
                _ => 0,
            };
            if divisor > 0 {
                self.at += 1;
                value.checked_div(Rational::integer(divisor))?
            } else {
                self.at = saved;
                value
            }
        };
        Ok(Some(if negative {
            value.checked_neg()?
        } else {
            value
        }))
    }
    fn single(&mut self) -> ScalarExpr {
        self.single = true;
        ScalarExpr::variable(false)
    }
    fn atom(&mut self, depth: usize) -> Result<ScalarExpr> {
        if depth > 16 {
            return Err("scalar expression too deep".into());
        }
        if self.eat("twice") || self.eat("thrice") {
            let factor = if self.tokens.get(self.at - 1).is_some_and(|s| s == "twice") {
                2
            } else {
                3
            };
            return self.atom(depth + 1)?.scale(Rational::integer(factor));
        }
        if self.eat("its") {
            let factor = self.scalar()?.ok_or("missing possessive fraction")?;
            return self.single().scale(factor);
        }
        if self.eat("a number") {
            if self.introduced {
                return Err("multiple indefinite number referents".into());
            }
            self.introduced = true;
            return Ok(self.single());
        }
        if self.eat("the number") || self.eat("that number") {
            return Ok(self.single());
        }
        if self.eat("the first number") || self.eat("the first") {
            self.pair = true;
            return Ok(ScalarExpr::variable(false));
        }
        if self.eat("the second number") || self.eat("the second") {
            self.pair = true;
            return Ok(ScalarExpr::variable(true));
        }
        if self.eat("their sum") {
            self.pair = true;
            return ScalarExpr::variable(false).add(ScalarExpr::variable(true));
        }
        let start = self.at;
        self.eat("the");
        let difference = if self.eat("sum of") {
            false
        } else if self.eat("difference between") || self.eat("difference of") {
            true
        } else {
            self.at = start;
            return self.factor(depth);
        };
        if !difference && (self.eat("two numbers") || self.eat("the two numbers")) {
            self.pair = true;
            return ScalarExpr::variable(false).add(ScalarExpr::variable(true));
        }
        let left = self.expr(depth + 1)?;
        self.need("and")?;
        let right = self.expr(depth + 1)?;
        if difference {
            // English "difference between" does not specify an ordering.
            // Enumerate both orientations, then check their signs and require
            // agreement among all feasible solutions. Explicit "minus" stays
            // ordered subtraction.
            let index = self.differences.len();
            if index >= 4 {
                return Err("too many absolute differences".into());
            }
            let mut value = left.sub(right)?;
            if self.choices & (1 << index) != 0 {
                value = value.scale(one().checked_neg()?)?;
            }
            self.differences.push(value);
            Ok(value)
        } else {
            left.add(right)
        }
    }
    fn factor(&mut self, depth: usize) -> Result<ScalarExpr> {
        let number = self.scalar()?.ok_or("unsupported scalar expression")?;
        if self.eat("%") || self.eat("percent") {
            self.need("of")?;
            return self
                .atom(depth + 1)?
                .scale(number.checked_div(Rational::integer(100))?);
        }
        if self.eat("times") {
            self.eat("of");
            return self.atom(depth + 1)?.scale(number);
        }
        if self.eat("of") {
            return self.atom(depth + 1)?.scale(number);
        }
        if self.eat("more than") {
            return self.expr(depth + 1)?.add(ScalarExpr::constant(number));
        }
        if self.eat("less than") {
            return self.expr(depth + 1)?.sub(ScalarExpr::constant(number));
        }
        Ok(ScalarExpr::constant(number))
    }
    fn expr(&mut self, depth: usize) -> Result<ScalarExpr> {
        let mut value = self.atom(depth)?;
        loop {
            if self.eat("plus") {
                value = value.add(self.atom(depth + 1)?)?;
            } else if self.eat("minus") {
                value = value.sub(self.atom(depth + 1)?)?;
            } else {
                return Ok(value);
            }
        }
    }
    fn parse(mut self) -> Result<Problem> {
        let mut constraints = Vec::new();
        let target;
        loop {
            self.eat("if");
            self.eat("then");
            if self.eat("what is") || self.eat("find") {
                target = self.expr(0)?;
                break;
            }
            let left = self.expr(0)?;
            if !(self.eat("is") || self.eat("equals") || self.eat("gives")) {
                return Err("missing equality".into());
            }
            if self.at == self.tokens.len() || self.tokens.get(self.at).is_some_and(|s| s == ":") {
                target = left;
                break;
            }
            self.eat("equal to");
            let right = if self.eat("more than") {
                let base = self.expr(0)?;
                self.need("by")?;
                base.add(self.expr(0)?)?
            } else if self.eat("less than") {
                let base = self.expr(0)?;
                self.need("by")?;
                base.sub(self.expr(0)?)?
            } else {
                self.expr(0)?
            };
            constraints.push(Equality(left.sub(right)?));
            if constraints.len() > 8 {
                return Err("too many scalar constraints".into());
            }
            let punctuation = self.eat(".") || self.eat(",") || self.eat(";");
            if !self.eat("and") && !punctuation {
                return Err("missing clause boundary".into());
            }
        }
        self.eat(":");
        self.eat("?");
        self.eat(".");
        if self.at != self.tokens.len() || constraints.is_empty() {
            return Err("unconsumed scalar clauses or no constraints".into());
        }
        if self.single && (self.pair || !self.introduced) {
            return Err("ambiguous scalar referents".into());
        }
        if !self.single && !self.pair {
            return Err("no scalar unknown".into());
        }
        Ok(Problem {
            constraints,
            target,
            pair: self.pair,
            differences: self.differences,
        })
    }
}

fn tokenize(text: &str) -> Result<Vec<String>> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut at = 0;
    while let Some(&c) = chars.get(at) {
        if c.is_ascii_whitespace() {
            at += 1;
            continue;
        }
        let start = at;
        if c.is_ascii_alphabetic() {
            while chars.get(at).is_some_and(char::is_ascii_alphabetic) {
                at += 1;
            }
        } else if c.is_ascii_digit() {
            while chars.get(at).is_some_and(char::is_ascii_digit) {
                at += 1;
            }
            if chars.get(at) == Some(&'.') && chars.get(at + 1).is_some_and(char::is_ascii_digit) {
                at += 1;
                while chars.get(at).is_some_and(char::is_ascii_digit) {
                    at += 1;
                }
            }
        } else if ".,;?:%/-".contains(c) {
            at += 1;
        } else {
            return Err("unsupported scalar token".into());
        }
        tokens.push(
            chars
                .get(start..at)
                .ok_or("invalid token boundary")?
                .iter()
                .collect(),
        );
    }
    Ok(tokens)
}

pub(crate) fn solve(text: &str) -> Result<Option<Rational>> {
    if !text.contains("number") {
        return Ok(None);
    }
    let tokens = tokenize(text)?;
    let parse = |choices| {
        Parser {
            tokens: tokens.clone(),
            at: 0,
            single: false,
            introduced: false,
            pair: false,
            choices,
            differences: Vec::new(),
        }
        .parse()
    };
    let first = parse(0)?;
    let variants = 1 << first.differences.len();
    let mut answer = first.execute()?;
    for choices in 1..variants {
        if let Some(value) = parse(choices)?.execute()? {
            if answer.is_some_and(|previous| previous != value) {
                return Err("scalar constraints have multiple target values".into());
            }
            answer = Some(value);
        }
    }
    answer
        .map(Some)
        .ok_or("inconsistent scalar constraints".into())
}
