//! Offline TRAIN annotation / lexical-baseline execution helper; not a question solver.
use glassmind_quantities::proposal::{Program, numbers};
use serde::Deserialize;
use std::io::{self, BufRead};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    question: String,
    program: Option<Program>,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let input: Input = serde_json::from_str(&line?)?;
        let out = if let Some(p) = input.program {
            match p.execute(&input.question) {
                Ok((n, v)) => {
                    serde_json::json!({"status":"proposed","semantics":"unverified","numbers":n,"value":v.to_string(),"program":p})
                }
                Err(reason) => serde_json::json!({"status":"refusal","reason":reason}),
            }
        } else {
            match numbers(&input.question) {
                Ok(n) => serde_json::json!({"status":"bound","numbers":n}),
                Err(reason) => serde_json::json!({"status":"refusal","reason":reason}),
            }
        };
        println!("{}", serde_json::to_string(&out)?);
    }
    Ok(())
}
