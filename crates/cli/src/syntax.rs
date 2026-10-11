//! Explicit predicted syntax; never used to authorize facts or code actions.
use glassmind_reading::syntax::{Input, Model, Prediction, VERSION};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

fn emit(
    out: &mut impl Write,
    result: Result<Prediction, String>,
    json: bool,
) -> Result<(), String> {
    if json {
        let value = match result {
            Ok(prediction) => serde_json::to_value(prediction).map_err(|e| e.to_string())?,
            Err(reason) => serde_json::json!({
                "status": "refused", "model_version": VERSION, "reason": reason
            }),
        };
        serde_json::to_writer(&mut *out, &value).map_err(|e| e.to_string())?;
        return writeln!(out).map_err(|e| e.to_string());
    }
    match result {
        Ok(prediction) => {
            writeln!(out, "Predicted dependencies (unverified):").map_err(|e| e.to_string())?;
            for (i, ((form, head), relation)) in prediction
                .forms
                .iter()
                .zip(&prediction.heads)
                .zip(&prediction.deprels)
                .enumerate()
            {
                let parent = head
                    .checked_sub(1)
                    .and_then(|n| prediction.forms.get(n))
                    .map_or("ROOT", String::as_str);
                writeln!(
                    out,
                    "{}: {form:?} --{relation}--> {head}: {parent:?}",
                    i + 1
                )
                .map_err(|e| e.to_string())?;
            }
        }
        Err(reason) => {
            writeln!(out, "I couldn't parse this input: {reason}").map_err(|e| e.to_string())?
        }
    }
    Ok(())
}

pub fn run(model: &Path, input: Option<&Path>, text: &str, json: bool) -> Result<(), String> {
    let model = Model::load(BufReader::new(
        File::open(model).map_err(|e| e.to_string())?,
    ))?;
    let mut out = BufWriter::new(std::io::stdout().lock());
    if let Some(input) = input {
        let mut reader = BufReader::new(File::open(input).map_err(|e| e.to_string())?);
        loop {
            let mut line = String::new();
            let n = reader
                .by_ref()
                .take(128 * 1024 + 1)
                .read_line(&mut line)
                .map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            if n > 128 * 1024 {
                return Err("JSONL row exceeds128KiB".into());
            }
            let prediction = serde_json::from_str::<Input>(&line)
                .map_err(|e| e.to_string())
                .and_then(|input| model.predict(input));
            emit(&mut out, prediction, true)?;
        }
    } else {
        emit(
            &mut out,
            model.predict(Input {
                forms: None,
                text: Some(text.into()),
            }),
            json,
        )?;
    }
    out.flush().map_err(|e| e.to_string())
}
