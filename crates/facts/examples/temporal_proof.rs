//! Development proof stream for comparing the public library API with the CLI.
use glassmind_facts::temporal::TemporalIndex;
use std::io::{self, BufRead};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let [_, index, temporal] = args.as_slice() else {
        return Err("INDEX TEMPORAL".into());
    };
    let loaded = TemporalIndex::from_bytes(&std::fs::read(index)?, &std::fs::read(temporal)?)?;
    for line in io::stdin().lock().lines() {
        let line = line?;
        let (date, question): (String, String) = serde_json::from_str(&line)?;
        println!(
            "{}",
            serde_json::to_string(&loaded.verify_at_str(&date, &question))?
        );
    }
    Ok(())
}
