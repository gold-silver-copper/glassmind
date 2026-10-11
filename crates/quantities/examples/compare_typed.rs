//! Public review adapter: exact JSON operands + typed operation, no English parser.
use glassmind_quantities::comparison::{
    ComparisonError, ComparisonOperation, ComparisonQuantity, ComparisonQuery, ComparisonResponse,
};
use serde::Deserialize;
use std::io::{self, BufRead};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Operand {
    value: String,
    unit: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    left: Operand,
    right: Operand,
    operation: ComparisonOperation,
}
fn operand(input: Operand) -> Result<ComparisonQuantity, ComparisonError> {
    let value = input
        .value
        .parse()
        .map_err(|reason| ComparisonError::InvalidNumber { reason })?;
    ComparisonQuantity::new(value, &input.unit)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let input: Query = serde_json::from_str(&line?)?;
        let result = (|| {
            ComparisonQuery {
                left: operand(input.left)?,
                right: operand(input.right)?,
                operation: input.operation,
            }
            .evaluate()
        })();
        println!(
            "{}",
            serde_json::to_string(&ComparisonResponse::from(result))?
        );
    }
    Ok(())
}
