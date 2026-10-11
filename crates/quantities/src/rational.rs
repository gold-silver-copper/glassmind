//! Bounded exact rationals: arithmetic overflow is a refusal, never rounding.
use std::{fmt, str::FromStr};

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rational {
    numerator: i128,
    denominator: i128,
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

impl Rational {
    pub fn new(numerator: i128, denominator: i128) -> Result<Self> {
        if denominator == 0 {
            return Err("division by zero".into());
        }
        let n = numerator.checked_abs().ok_or("rational overflow")?;
        let d = denominator.checked_abs().ok_or("rational overflow")?;
        let common = gcd(n, d);
        let numerator = if (numerator < 0) != (denominator < 0) {
            -(n / common)
        } else {
            n / common
        };
        Ok(Self {
            numerator,
            denominator: d / common,
        })
    }

    pub fn integer(value: i64) -> Self {
        Self {
            numerator: i128::from(value),
            denominator: 1,
        }
    }

    pub fn is_negative(self) -> bool {
        self.numerator < 0
    }

    pub fn as_i64(self) -> Result<i64> {
        if self.denominator != 1 {
            return Err("expected a whole number".into());
        }
        i64::try_from(self.numerator).map_err(|_| "integer overflow".into())
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self> {
        let common = gcd(self.denominator, rhs.denominator);
        let left = rhs.denominator / common;
        let right = self.denominator / common;
        let numerator = self
            .numerator
            .checked_mul(left)
            .and_then(|n| {
                rhs.numerator
                    .checked_mul(right)
                    .and_then(|r| n.checked_add(r))
            })
            .ok_or("rational overflow")?;
        let denominator = self
            .denominator
            .checked_mul(left)
            .ok_or("rational overflow")?;
        Self::new(numerator, denominator)
    }

    pub fn checked_neg(self) -> Result<Self> {
        Self::new(
            self.numerator.checked_neg().ok_or("rational overflow")?,
            self.denominator,
        )
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self> {
        self.checked_add(rhs.checked_neg()?)
    }

    pub fn checked_mul(self, rhs: Self) -> Result<Self> {
        let a = gcd(self.numerator.abs(), rhs.denominator);
        let b = gcd(rhs.numerator.abs(), self.denominator);
        Self::new(
            (self.numerator / a)
                .checked_mul(rhs.numerator / b)
                .ok_or("rational overflow")?,
            (self.denominator / b)
                .checked_mul(rhs.denominator / a)
                .ok_or("rational overflow")?,
        )
    }

    pub fn checked_div(self, rhs: Self) -> Result<Self> {
        self.checked_mul(Self::new(rhs.denominator, rhs.numerator)?)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

impl FromStr for Rational {
    type Err = String;
    fn from_str(text: &str) -> Result<Self> {
        if text.len() > 100 {
            return Err("number too long".into());
        }
        if let Some((n, d)) = text.split_once('/') {
            return Self::new(
                n.parse().map_err(|_| "invalid numerator")?,
                d.parse().map_err(|_| "invalid denominator")?,
            );
        }
        if let Some((whole, decimal)) = text.split_once('.') {
            if decimal.is_empty() || !decimal.bytes().all(|b| b.is_ascii_digit()) {
                return Err("invalid decimal".into());
            }
            let power = u32::try_from(decimal.len()).map_err(|_| "decimal too long")?;
            let denominator = 10_i128.checked_pow(power).ok_or("decimal overflow")?;
            let joined = format!("{whole}{decimal}");
            return Self::new(joined.parse().map_err(|_| "invalid decimal")?, denominator);
        }
        Self::new(text.parse().map_err(|_| "invalid number")?, 1)
    }
}
