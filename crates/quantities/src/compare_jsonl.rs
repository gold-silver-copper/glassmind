//! Bounded streaming transport used only by --compare-jsonl.
use glassmind_quantities::comparison::{
    ComparisonError, ComparisonResponse, MAX_COMPARISON_BYTES, compare_question,
};
use std::io::{self, BufRead, Write};

pub fn run(mut input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    // Any valid <=4096-byte question fits even when every byte is JSON-escaped.
    const MAX_LINE: usize = 6 * MAX_COMPARISON_BYTES + 4;
    loop {
        let mut line = Vec::new();
        let mut oversized = false;
        let mut seen = false;
        loop {
            let buffer = input.fill_buf()?;
            if buffer.is_empty() {
                break;
            }
            seen = true;
            let newline = buffer.iter().position(|&b| b == b'\n');
            let count = newline.map_or(buffer.len(), |p| p + 1);
            if !oversized {
                if line.len() + count > MAX_LINE {
                    oversized = true;
                    line.clear();
                } else {
                    line.extend(buffer.iter().take(count));
                }
            }
            input.consume(count);
            if newline.is_some() {
                break;
            }
        }
        if !seen {
            return Ok(());
        }
        let response = if oversized {
            ComparisonResponse::Refusal {
                error: ComparisonError::InvalidInput,
            }
        } else {
            match serde_json::from_slice::<String>(&line) {
                Ok(question) => ComparisonResponse::from(compare_question(&question)),
                Err(_) => ComparisonResponse::Refusal {
                    error: ComparisonError::Grammar,
                },
            }
        };
        serde_json::to_writer(&mut output, &response)?;
        writeln!(output)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_and_oversized_lines_resume_without_truncation()
    -> Result<(), Box<dyn std::error::Error>> {
        let valid = "\"Is 1 kg equal to 1000 g?\"\n";
        let input = format!(
            "{}\n{{}}\n\"Is 1 m equal to 1 m?\\u0000\"\n{valid}",
            "x".repeat(100_000)
        );
        let mut output = Vec::new();
        run(io::Cursor::new(input), &mut output)?;
        let lines = String::from_utf8(output)?;
        let values = lines
            .lines()
            .map(serde_json::from_str::<serde_json::Value>)
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(values.len(), 4);
        for value in values.iter().take(3) {
            assert_eq!(value["status"], "refusal");
        }
        assert_eq!(
            values.last().ok_or("missing result")?["status"],
            "comparison"
        );
        Ok(())
    }
}
