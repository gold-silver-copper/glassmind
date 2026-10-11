//! Units are exact affine maps to SI. Typed quantities cannot cross dimensions.
use crate::rational::{Rational, Result};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    Length,
    Mass,
    Duration,
    Volume,
    Temperature,
    Speed,
}

pub trait Kind {
    const DIMENSION: Dimension;
}

/// Dimensions with additive magnitudes (absolute temperature is affine).
pub trait AdditiveKind: Kind {}
pub struct Length;
pub struct Mass;
pub struct Duration;
pub struct Volume;
pub struct Temperature;
pub struct Speed;
impl Kind for Length {
    const DIMENSION: Dimension = Dimension::Length;
}
impl Kind for Mass {
    const DIMENSION: Dimension = Dimension::Mass;
}
impl Kind for Duration {
    const DIMENSION: Dimension = Dimension::Duration;
}
impl Kind for Volume {
    const DIMENSION: Dimension = Dimension::Volume;
}
impl Kind for Temperature {
    const DIMENSION: Dimension = Dimension::Temperature;
}
impl Kind for Speed {
    const DIMENSION: Dimension = Dimension::Speed;
}
impl AdditiveKind for Length {}
impl AdditiveKind for Mass {}
impl AdditiveKind for Duration {}
impl AdditiveKind for Volume {}
impl AdditiveKind for Speed {}

#[derive(Clone, Debug)]
pub struct Unit {
    dimension: Dimension,
    scale: Rational,
    offset: Rational,
}

impl Unit {
    pub fn dimension(&self) -> Dimension {
        self.dimension
    }
    pub fn parse(name: &str) -> Result<Self> {
        let normalized = name.trim().to_lowercase();
        let name = normalized.as_str();
        let (dimension, numerator, denominator) = match name {
            "m" | "meter" | "meters" | "metre" | "metres" => (Dimension::Length, 1, 1),
            "km" | "kilometer" | "kilometers" | "kilometre" | "kilometres" => {
                (Dimension::Length, 1000, 1)
            }
            "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => {
                (Dimension::Length, 1, 100)
            }
            "mm" | "millimeter" | "millimeters" => (Dimension::Length, 1, 1000),
            "inch" | "inches" | "in" => (Dimension::Length, 127, 5000),
            "foot" | "feet" | "ft" => (Dimension::Length, 381, 1250),
            "yard" | "yards" | "yd" => (Dimension::Length, 1143, 1250),
            "mile" | "miles" | "mi" => (Dimension::Length, 201168, 125),
            "kg" | "kilogram" | "kilograms" => (Dimension::Mass, 1, 1),
            "g" | "gram" | "grams" => (Dimension::Mass, 1, 1000),
            "mg" | "milligram" | "milligrams" => (Dimension::Mass, 1, 1000000),
            "lb" | "lbs" | "pound" | "pounds" => (Dimension::Mass, 45359237, 100000000),
            "oz" | "ounce" | "ounces" => (Dimension::Mass, 45359237, 1600000000),
            "s" | "second" | "seconds" => (Dimension::Duration, 1, 1),
            "min" | "minute" | "minutes" => (Dimension::Duration, 60, 1),
            "h" | "hour" | "hours" => (Dimension::Duration, 3600, 1),
            "day" | "days" => (Dimension::Duration, 86400, 1),
            "week" | "weeks" => (Dimension::Duration, 604800, 1),
            "l" | "liter" | "liters" | "litre" | "litres" => (Dimension::Volume, 1, 1000),
            "ml" | "milliliter" | "milliliters" => (Dimension::Volume, 1, 1000000),
            "us gallon" | "us gallons" => (Dimension::Volume, 473176473, 125000000000),
            "m/s" => (Dimension::Speed, 1, 1),
            "km/h" | "kph" => (Dimension::Speed, 5, 18),
            "mph" => (Dimension::Speed, 1397, 3125),
            "c" | "celsius" | "degrees celsius" => (Dimension::Temperature, 1, 1),
            "f" | "fahrenheit" | "degrees fahrenheit" => (Dimension::Temperature, 5, 9),
            "k" | "kelvin" => (Dimension::Temperature, 1, 1),
            _ => return Err(format!("unknown or ambiguous unit: {name}")),
        };
        let offset = match name {
            "c" | "celsius" | "degrees celsius" => Rational::new(27315, 100)?,
            "f" | "fahrenheit" | "degrees fahrenheit" => Rational::new(45967, 180)?,
            _ => Rational::integer(0),
        };
        Ok(Self {
            dimension,
            scale: Rational::new(numerator, denominator)?,
            offset,
        })
    }

    pub fn convert(&self, value: Rational, target: &Self) -> Result<Rational> {
        if self.dimension != target.dimension {
            return Err("incompatible unit dimensions".into());
        }
        value
            .checked_mul(self.scale)?
            .checked_add(self.offset)?
            .checked_sub(target.offset)?
            .checked_div(target.scale)
    }
}

/// A validated unit whose dimension is part of its Rust type.
pub struct TypedUnit<D: Kind> {
    unit: Unit,
    marker: PhantomData<D>,
}
impl<D: Kind> TypedUnit<D> {
    pub fn parse(name: &str) -> Result<Self> {
        let unit = Unit::parse(name)?;
        if unit.dimension != D::DIMENSION {
            return Err("unit does not match static dimension".into());
        }
        Ok(Self {
            unit,
            marker: PhantomData,
        })
    }
}

/// Exact magnitude in a dimension-checked unit, reusable without the English parser.
pub struct Quantity<D: Kind> {
    value: Rational,
    unit: TypedUnit<D>,
}
impl<D: Kind> Quantity<D> {
    pub fn new(value: Rational, unit: TypedUnit<D>) -> Self {
        Self { value, unit }
    }
    pub fn in_unit(&self, target: &TypedUnit<D>) -> Result<Rational> {
        self.unit.unit.convert(self.value, &target.unit)
    }

    /// Compare magnitudes after exact conversion, including affine temperatures.
    pub fn checked_cmp(&self, other: &Self) -> Result<std::cmp::Ordering> {
        self.in_unit(&self.unit)?
            .checked_cmp(other.in_unit(&self.unit)?)
    }
}

impl<D: AdditiveKind> Quantity<D> {
    /// Add like dimensions after exact conversion into the receiver's unit.
    pub fn checked_add(&self, other: &Self) -> Result<Self> {
        let value = self.value.checked_add(other.in_unit(&self.unit)?)?;
        Ok(Self::new(
            value,
            TypedUnit {
                unit: self.unit.unit.clone(),
                marker: PhantomData,
            },
        ))
    }
}

impl Quantity<Length> {
    pub fn divided_by_duration(&self, duration: &Quantity<Duration>) -> Result<Quantity<Speed>> {
        let meters = self.in_unit(&TypedUnit::<Length>::parse("meters")?)?;
        let seconds = duration.in_unit(&TypedUnit::<Duration>::parse("seconds")?)?;
        Ok(Quantity::new(
            meters.checked_div(seconds)?,
            TypedUnit::<Speed>::parse("m/s")?,
        ))
    }
}

impl Quantity<Speed> {
    pub fn multiplied_by_duration(
        &self,
        duration: &Quantity<Duration>,
    ) -> Result<Quantity<Length>> {
        let meters_per_second = self.in_unit(&TypedUnit::<Speed>::parse("m/s")?)?;
        let seconds = duration.in_unit(&TypedUnit::<Duration>::parse("seconds")?)?;
        Ok(Quantity::new(
            meters_per_second.checked_mul(seconds)?,
            TypedUnit::<Length>::parse("meters")?,
        ))
    }
}
