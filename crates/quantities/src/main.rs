use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    if args.first().is_some_and(|s| s == "--jsonl") {
        for line in io::stdin().lock().lines() {
            let line = line?;
            let question: String = serde_json::from_str(&line)?;
            serde_json::to_writer(&mut output, &glassmind_quantities::solve(&question))?;
            writeln!(output)?;
        }
    } else if args.is_empty() {
        writeln!(
            output,
            "Usage: glassmind-quantities 'question' | --jsonl (JSON strings on stdin)"
        )?;
    } else {
        serde_json::to_writer(&mut output, &glassmind_quantities::solve(&args.join(" ")))?;
        writeln!(output)?;
    }
    Ok(())
}
