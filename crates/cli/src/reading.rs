//! Explicit passage proposals; never invoked by factual conversation routing.
use glassmind_reading::qa::{Input, Model, Prediction, Status};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

fn bounded_file(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| format!("cannot open {}: {e}", path.display()))?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err(format!("{} exceeds the {limit}-byte limit", path.display()));
    }
    Ok(bytes)
}

fn emit(
    out: &mut impl Write,
    prediction: &Prediction,
    context: &str,
    json: bool,
    explain: bool,
) -> Result<(), String> {
    if json {
        serde_json::to_writer(&mut *out, prediction).map_err(|e| e.to_string())?;
        return writeln!(out).map_err(|e| e.to_string());
    }
    if prediction.status == Status::Predicted {
        let source = prediction
            .evidence
            .as_ref()
            .and_then(|span| span.text(context))
            .ok_or("prediction has no valid source passage")?;
        writeln!(out, "Candidate answer (unverified): {}", prediction.answer)
            .and_then(|()| writeln!(out, "Source passage: {source}"))
            .map_err(|e| e.to_string())?;
    } else {
        writeln!(out, "I couldn't select an answer from this passage.")
            .map_err(|e| e.to_string())?;
    }
    if explain {
        writeln!(
            out,
            "Selection: {}; source bytes: {:?}; rank score: {}; margin: {} (not probabilities).",
            prediction.reason, prediction.proposal, prediction.score, prediction.margin
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn run(
    model: &Path,
    context: Option<&Path>,
    input: Option<&Path>,
    question: &str,
    json: bool,
    explain: bool,
) -> Result<(), String> {
    let model: Model = serde_json::from_slice(&bounded_file(model, 4 * 1024 * 1024)?)
        .map_err(|e| format!("invalid reading model: {e}"))?;
    model.validate()?;
    let mut out = BufWriter::new(std::io::stdout().lock());
    if let Some(input) = input {
        let mut reader = BufReader::new(File::open(input).map_err(|e| e.to_string())?);
        loop {
            let mut line = Vec::new();
            // Includes room for escaped UTF-8 context/question JSON, but bounds each row.
            let read = reader
                .by_ref()
                .take(512 * 1024 + 1)
                .read_until(b'\n', &mut line)
                .map_err(|e| e.to_string())?;
            if read == 0 {
                break;
            }
            if read > 512 * 1024 {
                return Err("reading input row exceeds the 512-KiB limit".into());
            }
            let row: Input = serde_json::from_slice(&line)
                .map_err(|e| format!("expected only context and question: {e}"))?;
            emit(
                &mut out,
                &model.predict(&row.context, &row.question),
                &row.context,
                true,
                false,
            )?;
        }
    } else {
        let context = context.ok_or("a context file or JSONL input is required")?;
        let context = String::from_utf8(bounded_file(context, 65536)?)
            .map_err(|e| format!("context must be UTF-8: {e}"))?;
        emit(
            &mut out,
            &model.predict(&context, question),
            &context,
            json,
            explain,
        )?;
    }
    out.flush().map_err(|e| e.to_string())
}
