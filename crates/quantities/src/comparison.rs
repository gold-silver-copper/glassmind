//! Exact literal comparisons, deliberately separate from scalar `solve`/`Answer`.
//! All entry points validate dimensions. No physical plausibility policy is imposed.
use crate::{
    rational::Rational,
    units::{Dimension, Unit},
};
use serde::{Deserialize, Serialize, Serializer};
use std::cmp::Ordering;

pub const MAX_COMPARISON_BYTES: usize = 4096;
pub type ComparisonResultOrError<T> = Result<T, ComparisonError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ComparisonError {
    InvalidInput,
    Grammar,
    InvalidNumber { reason: String },
    InvalidUnit { reason: String },
    DimensionMismatch,
    SelectionDimensionMismatch,
    Overflow,
}
impl std::fmt::Display for ComparisonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput => f.write_str("input exceeds 4096 UTF-8 bytes or contains NUL"),
            Self::Grammar => f.write_str("expected a fully consumed literal quantity comparison"),
            Self::InvalidNumber { reason } | Self::InvalidUnit { reason } => f.write_str(reason),
            Self::DimensionMismatch => f.write_str("incompatible unit dimensions"),
            Self::SelectionDimensionMismatch => {
                f.write_str("selection adjective does not match dimension")
            }
            Self::Overflow => f.write_str("exact comparison arithmetic overflow"),
        }
    }
}
impl std::error::Error for ComparisonError {}

fn exact<S: Serializer>(value: &Rational, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}
fn validate_input(text: &str) -> ComparisonResultOrError<()> {
    if text.len() > MAX_COMPARISON_BYTES || text.contains('\0') {
        return Err(ComparisonError::InvalidInput);
    }
    Ok(())
}
fn normalize(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

/// A validated exact magnitude and existing unit. Private fields prevent forged units.
#[derive(Clone, Debug, Serialize)]
pub struct ComparisonQuantity {
    #[serde(serialize_with = "exact")]
    value: Rational,
    #[serde(skip)]
    unit: Unit,
    #[serde(rename = "unit")]
    unit_name: String,
}
impl ComparisonQuantity {
    pub fn new(value: Rational, unit_name: &str) -> ComparisonResultOrError<Self> {
        validate_input(unit_name)?;
        let unit_name = normalize(unit_name);
        let unit =
            Unit::parse(&unit_name).map_err(|reason| ComparisonError::InvalidUnit { reason })?;
        Ok(Self {
            value,
            unit,
            unit_name,
        })
    }
    pub fn value(&self) -> Rational {
        self.value
    }
    pub fn unit(&self) -> &Unit {
        &self.unit
    }
    pub fn unit_name(&self) -> &str {
        &self.unit_name
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Predicate {
    GreaterThan,
    LessThan,
    EqualTo,
    AtLeast,
    AtMost,
}
impl Predicate {
    fn test(self, order: Ordering) -> bool {
        match self {
            Self::GreaterThan => order == Ordering::Greater,
            Self::LessThan => order == Ordering::Less,
            Self::EqualTo => order == Ordering::Equal,
            Self::AtLeast => order != Ordering::Less,
            Self::AtMost => order != Ordering::Greater,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Selection {
    Longer,
    Shorter,
    Heavier,
    Lighter,
    Faster,
    Slower,
    Hotter,
    Colder,
}
impl Selection {
    pub fn dimension(self) -> Dimension {
        match self {
            Self::Longer | Self::Shorter => Dimension::Length,
            Self::Heavier | Self::Lighter => Dimension::Mass,
            Self::Faster | Self::Slower => Dimension::Speed,
            Self::Hotter | Self::Colder => Dimension::Temperature,
        }
    }
    fn prefers_greater(self) -> bool {
        matches!(
            self,
            Self::Longer | Self::Heavier | Self::Faster | Self::Hotter
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "operation", rename_all = "snake_case")]
pub enum ComparisonOperation {
    Predicate(Predicate),
    Selection(Selection),
}

/// Direct construction is supported; `evaluate` checks dimensions and operation.
#[derive(Clone, Debug, Serialize)]
pub struct ComparisonQuery {
    pub left: ComparisonQuantity,
    pub right: ComparisonQuantity,
    pub operation: ComparisonOperation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOrdering {
    Less,
    Equal,
    Greater,
}
impl From<Ordering> for ComparisonOrdering {
    fn from(value: Ordering) -> Self {
        match value {
            Ordering::Less => Self::Less,
            Ordering::Equal => Self::Equal,
            Ordering::Greater => Self::Greater,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectedOperand {
    Left,
    Right,
    Equal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ComparisonOutcome {
    Predicate {
        predicate: Predicate,
        truth: bool,
    },
    Selection {
        selection: Selection,
        selected: SelectedOperand,
    },
}
#[derive(Clone, Debug, Serialize)]
pub struct ConversionWitness {
    pub unit: String,
    #[serde(serialize_with = "exact")]
    pub left: Rational,
    #[serde(serialize_with = "exact")]
    pub right: Rational,
}
#[derive(Clone, Debug, Serialize)]
pub struct ComparisonResult {
    pub left: ComparisonQuantity,
    pub right: ComparisonQuantity,
    pub dimension: Dimension,
    pub ordering: ComparisonOrdering,
    pub outcome: ComparisonOutcome,
    pub witness: ConversionWitness,
}
impl ComparisonQuery {
    pub fn evaluate(&self) -> ComparisonResultOrError<ComparisonResult> {
        let dimension = self.left.unit.dimension();
        if dimension != self.right.unit.dimension() {
            return Err(ComparisonError::DimensionMismatch);
        }
        if let ComparisonOperation::Selection(selection) = self.operation
            && selection.dimension() != dimension
        {
            return Err(ComparisonError::SelectionDimensionMismatch);
        }
        let left = self
            .left
            .unit
            .convert(self.left.value, &self.left.unit)
            .map_err(|_| ComparisonError::Overflow)?;
        let right = self
            .right
            .unit
            .convert(self.right.value, &self.left.unit)
            .map_err(|_| ComparisonError::Overflow)?;
        let order = left
            .checked_cmp(right)
            .map_err(|_| ComparisonError::Overflow)?;
        let outcome = match self.operation {
            ComparisonOperation::Predicate(predicate) => ComparisonOutcome::Predicate {
                predicate,
                truth: predicate.test(order),
            },
            ComparisonOperation::Selection(selection) => {
                let selected = if order == Ordering::Equal {
                    SelectedOperand::Equal
                } else if (order == Ordering::Greater) == selection.prefers_greater() {
                    SelectedOperand::Left
                } else {
                    SelectedOperand::Right
                };
                ComparisonOutcome::Selection {
                    selection,
                    selected,
                }
            }
        };
        Ok(ComparisonResult {
            left: self.left.clone(),
            right: self.right.clone(),
            dimension,
            ordering: order.into(),
            outcome,
            witness: ConversionWitness {
                unit: self.left.unit_name.clone(),
                left,
                right,
            },
        })
    }
}

fn unsigned_integer(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}
fn without_sign(text: &str) -> &str {
    text.strip_prefix(['+', '-']).unwrap_or(text)
}
fn literal(text: &str) -> bool {
    if let Some((n, d)) = text.split_once('/') {
        unsigned_integer(without_sign(n)) && unsigned_integer(without_sign(d))
    } else if let Some((whole, decimal)) = without_sign(text).split_once('.') {
        (whole.is_empty() || unsigned_integer(whole)) && unsigned_integer(decimal)
    } else {
        unsigned_integer(without_sign(text))
    }
}
fn quantity(text: &str) -> ComparisonResultOrError<ComparisonQuantity> {
    let (value, unit) = text.split_once(' ').ok_or(ComparisonError::Grammar)?;
    if value.len() > 100 || !literal(value) {
        return Err(ComparisonError::InvalidNumber {
            reason: "expected one exact signed integer, decimal or fraction".into(),
        });
    }
    let value = value
        .parse::<Rational>()
        .map_err(|reason| ComparisonError::InvalidNumber { reason })?;
    ComparisonQuantity::new(value, unit)
}

/// Parse the complete bounded grammar. Evaluation remains mandatory for validation.
pub fn parse_comparison(question: &str) -> ComparisonResultOrError<ComparisonQuery> {
    validate_input(question)?;
    let normalized = normalize(question);
    let question = normalized
        .strip_suffix('?')
        .unwrap_or(&normalized)
        .trim_end();
    if let Some(body) = question.strip_prefix("is ") {
        for (phrase, predicate) in [
            (" greater than ", Predicate::GreaterThan),
            (" less than ", Predicate::LessThan),
            (" equal to ", Predicate::EqualTo),
            (" at least ", Predicate::AtLeast),
            (" at most ", Predicate::AtMost),
        ] {
            if let Some((left, right)) = body.split_once(phrase) {
                return Ok(ComparisonQuery {
                    left: quantity(left)?,
                    right: quantity(right)?,
                    operation: ComparisonOperation::Predicate(predicate),
                });
            }
        }
    } else if let Some(body) = question.strip_prefix("which is ") {
        let (adjective, operands) = body.split_once(',').ok_or(ComparisonError::Grammar)?;
        let selection = match adjective.trim() {
            "longer" => Selection::Longer,
            "shorter" => Selection::Shorter,
            "heavier" => Selection::Heavier,
            "lighter" => Selection::Lighter,
            "faster" => Selection::Faster,
            "slower" => Selection::Slower,
            "hotter" => Selection::Hotter,
            "colder" => Selection::Colder,
            _ => return Err(ComparisonError::Grammar),
        };
        let (left, right) = operands
            .trim()
            .split_once(" or ")
            .ok_or(ComparisonError::Grammar)?;
        return Ok(ComparisonQuery {
            left: quantity(left)?,
            right: quantity(right)?,
            operation: ComparisonOperation::Selection(selection),
        });
    }
    Err(ComparisonError::Grammar)
}

pub fn compare_question(question: &str) -> ComparisonResultOrError<ComparisonResult> {
    parse_comparison(question)?.evaluate()
}

/// Wire response for the independent native comparison mode.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ComparisonResponse {
    Comparison { result: Box<ComparisonResult> },
    Refusal { error: ComparisonError },
}
impl From<ComparisonResultOrError<ComparisonResult>> for ComparisonResponse {
    fn from(value: ComparisonResultOrError<ComparisonResult>) -> Self {
        match value {
            Ok(result) => Self::Comparison {
                result: Box::new(result),
            },
            Err(error) => Self::Refusal { error },
        }
    }
}
