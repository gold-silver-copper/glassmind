//! Proleptic Gregorian dates, years 1..=9999. Month shifts clamp to month end.
use crate::rational::Result;
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Date {
    year: i32,
    month: u32,
    day: u32,
}

pub fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    }
}

fn before_year(year: i32) -> i64 {
    let y = i64::from(year) - 1;
    365 * y + y / 4 - y / 100 + y / 400
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self> {
        if !(1..=9999).contains(&year) || day == 0 || day > month_days(year, month) {
            return Err("invalid Gregorian date".into());
        }
        Ok(Self { year, month, day })
    }
    pub fn ordinal(self) -> i64 {
        before_year(self.year)
            + (1..self.month)
                .map(|m| i64::from(month_days(self.year, m)))
                .sum::<i64>()
            + i64::from(self.day)
            - 1
    }
    pub fn add_days(self, days: i64) -> Result<Self> {
        let ordinal = self.ordinal().checked_add(days).ok_or("date overflow")?;
        if !(0..before_year(10000)).contains(&ordinal) {
            return Err("date outside years 1..9999".into());
        }
        let (mut low, mut high) = (1, 10000);
        while low + 1 < high {
            let mid = (low + high) / 2;
            if before_year(mid) <= ordinal {
                low = mid;
            } else {
                high = mid;
            }
        }
        let mut remainder = ordinal - before_year(low);
        for month in 1..=12 {
            let count = i64::from(month_days(low, month));
            if remainder < count {
                return Self::new(
                    low,
                    month,
                    u32::try_from(remainder + 1).map_err(|_| "date overflow")?,
                );
            }
            remainder -= count;
        }
        Err("date out of range".into())
    }
    pub fn add_months(self, months: i64) -> Result<Self> {
        let index = (i64::from(self.year) - 1) * 12 + i64::from(self.month) - 1;
        let index = index.checked_add(months).ok_or("date overflow")?;
        if !(0..9999 * 12).contains(&index) {
            return Err("date outside years 1..9999".into());
        }
        let year = i32::try_from(index / 12 + 1).map_err(|_| "date overflow")?;
        let month = u32::try_from(index % 12 + 1).map_err(|_| "date overflow")?;
        Self::new(year, month, self.day.min(month_days(year, month)))
    }
    pub fn days_until(self, other: Self) -> i64 {
        other.ordinal() - self.ordinal()
    }
    pub fn weekday(self) -> &'static str {
        match self.ordinal() % 7 {
            0 => "Monday",
            1 => "Tuesday",
            2 => "Wednesday",
            3 => "Thursday",
            4 => "Friday",
            5 => "Saturday",
            _ => "Sunday",
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}
impl FromStr for Date {
    type Err = String;
    fn from_str(text: &str) -> Result<Self> {
        let mut fields = text.split('-');
        let year = fields
            .next()
            .ok_or("missing year")?
            .parse()
            .map_err(|_| "invalid year")?;
        let month = fields
            .next()
            .ok_or("missing month")?
            .parse()
            .map_err(|_| "invalid month")?;
        let day = fields
            .next()
            .ok_or("missing day")?
            .parse()
            .map_err(|_| "invalid day")?;
        if fields.next().is_some() {
            return Err("invalid date".into());
        }
        Self::new(year, month, day)
    }
}
