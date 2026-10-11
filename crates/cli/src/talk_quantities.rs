//! Stateless, literal quantity comparisons with exact conversion evidence.
use glassmind_quantities::comparison::{
    ComparisonOutcome, ComparisonResponse, ComparisonResult, SelectedOperand, compare_question,
};

pub fn request(line: &str) -> Option<Result<&str, &'static str>> {
    let trimmed = line.trim_start();
    const PREFIX: &str = "Compare quantities";
    if trimmed
        .get(..PREFIX.len())
        .is_some_and(|s| s.eq_ignore_ascii_case(PREFIX))
    {
        let rest = &trimmed[PREFIX.len()..];
        if rest.is_empty() || rest.starts_with(':') || rest.starts_with(char::is_whitespace) {
            return Some(
                rest.strip_prefix(':')
                    .filter(|payload| !payload.trim().is_empty())
                    .ok_or("Use Compare quantities: followed by a complete comparison."),
            );
        }
    }
    // Reserve malformed quantitative stems too: trailing assertions must not
    // reach personal memory or invalidate an existing story after parse failure.
    let lower = line
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    if let Some(body) = lower.strip_prefix("is ") {
        let numeric = body
            .as_bytes()
            .first()
            .is_some_and(|c| c.is_ascii_digit() || matches!(c, b'+' | b'-' | b'.'));
        if numeric
            && [
                " greater than ",
                " less than ",
                " equal to ",
                " at least ",
                " at most ",
            ]
            .iter()
            .any(|phrase| body.contains(phrase))
        {
            return Some(Ok(line));
        }
    }
    if let Some(body) = lower.strip_prefix("which is ") {
        let adjective = body.split([',', ' ']).next().unwrap_or_default();
        if matches!(
            adjective,
            "longer"
                | "shorter"
                | "heavier"
                | "lighter"
                | "faster"
                | "slower"
                | "hotter"
                | "colder"
        ) {
            return Some(Ok(line));
        }
    }
    None
}

#[derive(serde::Serialize)]
pub struct QuantityReply {
    kind: &'static str,
    pub text: String,
    #[serde(flatten)]
    response: ComparisonResponse,
}

fn describe(result: &ComparisonResult) -> String {
    let conclusion = match result.outcome {
        ComparisonOutcome::Predicate { truth, .. } => if truth { "Yes." } else { "No." }.to_owned(),
        ComparisonOutcome::Selection { selected, .. } => match selected {
            SelectedOperand::Left => format!(
                "The first quantity ({} {}) is the answer.",
                result.left.value(),
                result.left.unit_name()
            ),
            SelectedOperand::Right => format!(
                "The second quantity ({} {}) is the answer.",
                result.right.value(),
                result.right.unit_name()
            ),
            SelectedOperand::Equal => "Both quantities are equal.".to_owned(),
        },
    };
    format!(
        "{conclusion} In {}: the first quantity is {}, and the second is {} (exact).",
        result.witness.unit, result.witness.left, result.witness.right
    )
}

pub fn respond(request: Result<&str, &str>) -> QuantityReply {
    let result = request
        .map_err(|_| glassmind_quantities::comparison::ComparisonError::Grammar)
        .and_then(compare_question);
    let text = match &result {
        Ok(result) => describe(result),
        Err(error) => format!("I cannot compare those quantities: {error}."),
    };
    QuantityReply {
        kind: "quantity_comparison",
        text,
        response: result.into(),
    }
}
