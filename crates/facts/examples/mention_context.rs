//! FIT-only prototype: masked grammatical context, native sparse margin model.
use glassmind_everyday::{Example, IntentModel, TrainOptions};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const FUNCTION_WORDS: &str = "what which who whom whose where when why how is are was were be been being am do does did have has had can could will would shall should may might must a an the of to in on at from by with for into onto upon over under between among through about as than and or but if that this these those it its he him his she her hers they them their theirs we us our ours you your yours i me my mine not no nor s";

#[derive(Deserialize)]
struct Match {
    entity: String,
    case_supported: bool,
}
#[derive(Deserialize)]
struct Span {
    token_start: usize,
    token_end: usize,
    byte_start: usize,
    byte_end: usize,
    text: String,
    matches: Vec<Match>,
    ambiguous: bool,
    eligible_entity: Option<String>,
}
#[derive(Deserialize)]
struct Record {
    normalized: String,
    candidates: Vec<Span>,
    baseline_entity: Option<String>,
    parity: bool,
}
#[derive(Serialize, Deserialize)]
struct Feature {
    entity: String,
    span: usize,
    text: String,
}
#[derive(Serialize, Deserialize)]
struct Features {
    row: usize,
    baseline_entity: Option<String>,
    features: Vec<Feature>,
}
#[derive(Serialize)]
struct Scores {
    row: usize,
    baseline_entity: Option<String>,
    entity_scores: Vec<(String, f32)>,
}

fn context(word: Option<&str>) -> &str {
    match word {
        None => "boundary",
        Some(word) if word.chars().all(|c| c.is_ascii_digit()) => "number",
        Some(word) if FUNCTION_WORDS.split_whitespace().any(|w| w == word) => word,
        _ => "other",
    }
}
fn eligible(span: &Span) -> Result<Option<&str>> {
    let expected = match span.matches.as_slice() {
        [item] if item.case_supported => Some(item.entity.as_str()),
        _ => None,
    };
    if span.matches.is_empty()
        || span.matches.len() > 2
        || span.ambiguous != (span.matches.len() == 2)
        || span.eligible_entity.as_deref() != expected
    {
        return Err("invalid pinned span eligibility".into());
    }
    Ok(expected)
}
fn feature(record: &Record, span: &Span, count: usize) -> Result<String> {
    let words: Vec<_> = record.normalized.split_whitespace().collect();
    if span.token_start >= span.token_end
        || span.token_end > words.len()
        || record.normalized.get(span.byte_start..span.byte_end) != Some(span.text.as_str())
        || words
            .get(span.token_start..span.token_end)
            .is_none_or(|parts| parts.join(" ") != span.text)
    {
        return Err("invalid pinned span boundaries".into());
    }
    let mut values = Vec::new();
    for distance in 1..=3 {
        let left = span
            .token_start
            .checked_sub(distance)
            .and_then(|i| words.get(i).copied());
        let right = words.get(span.token_end + distance - 1).copied();
        values.push(format!("left{distance}{}", context(left)));
        values.push(format!("right{distance}{}", context(right)));
    }
    for i in 0..3 {
        let value = if (span.token_start..span.token_end).contains(&i) {
            "mention"
        } else {
            context(words.get(i).copied())
        };
        values.push(format!("prefix{i}{value}"));
    }
    values.push(format!(
        "words{}",
        (span.token_end - span.token_start).min(5)
    ));
    let bytes = match span.text.len() {
        0..=4 => "small",
        5..=8 => "medium",
        9..=16 => "large",
        _ => "long",
    };
    values.push(format!("bytes{bytes}"));
    values.push(format!("start{}", span.token_start == 0));
    values.push(format!("end{}", span.token_end == words.len()));
    values.push(format!(
        "quarter{}",
        span.token_start * 4 / words.len().max(1)
    ));
    values.push(format!(
        "digit{}",
        span.text.chars().any(|c| c.is_ascii_digit())
    ));
    values.push(format!(
        "longest{}",
        record
            .candidates
            .iter()
            .all(|s| s.text.len() <= span.text.len())
    ));
    values.push(format!("entities{}", count.min(5)));
    Ok(values.join(" "))
}
fn features(record: &Record, row: usize) -> Result<Features> {
    if !record.parity {
        return Err("native parity not established".into());
    }
    let mut ids = BTreeSet::new();
    for span in &record.candidates {
        if let Some(id) = eligible(span)? {
            ids.insert(id);
        }
    }
    let mut result = Vec::new();
    for (index, span) in record.candidates.iter().enumerate() {
        if let Some(entity) = eligible(span)? {
            result.push(Feature {
                entity: entity.into(),
                span: index,
                text: feature(record, span, ids.len())?,
            });
        }
    }
    Ok(Features {
        row,
        baseline_entity: record.baseline_entity.clone(),
        features: result,
    })
}
fn rank(items: impl IntoIterator<Item = (String, f32)>) -> Result<Vec<(String, f32)>> {
    let mut maxima = BTreeMap::<String, f32>::new();
    for (id, score) in items {
        if !score.is_finite() {
            return Err("nonfinite model score".into());
        }
        maxima
            .entry(id)
            .and_modify(|old| *old = old.max(score))
            .or_insert(score);
    }
    let mut ranked: Vec<_> = maxima.into_iter().collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    Ok(ranked)
}
fn stream(path: &Path) -> Result<impl Iterator<Item = io::Result<String>>> {
    Ok(BufReader::new(File::open(path)?).lines())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let mut output = io::BufWriter::new(io::stdout().lock());
    match args.as_slice() {
        [_,command,input] if command=="features" => {
            for (row,line) in stream(Path::new(input))?.enumerate() {
                let record:Record=serde_json::from_str(&line?)?;
                serde_json::to_writer(&mut output,&features(&record,row)?)?; writeln!(output)?;
            }
        }
        [_,command,input,destination] if command=="train" => {
            let rows:Vec<Example>=stream(Path::new(input))?.map(|line|Ok(serde_json::from_str(&line?)?)).collect::<Result<_>>()?;
            let model=IntentModel::train(&rows,TrainOptions { epochs:20,char_weight:0.0,min_df:2 })?;
            model.validate()?;
            let bytes=serde_json::to_vec(&model)?;
            if bytes.len()>8*1024*1024 { return Err("model exceeds8MiB".into()); }
            File::options().create_new(true).write(true).open(destination)?.write_all(&bytes)?;
            writeln!(output,"{}",serde_json::json!({"training_spans":rows.len(),"model_bytes":bytes.len(),"labels":model.labels}))?;
        }
        [_,command,model,input] if command=="score" => {
            let model:IntentModel=serde_json::from_reader(File::open(model)?)?;model.validate()?;
            if model.labels!=["mention","other"] { return Err("unexpected labels".into()); }
            for line in stream(Path::new(input))? {
                let record:Features=serde_json::from_str(&line?)?;
                let scores=record.features.iter().map(|f| {
                    let p=model.predict(&f.text);
                    (f.entity.clone(),if p.label=="mention" {p.margin} else {-p.margin})
                });
                let result=Scores { row:record.row,baseline_entity:record.baseline_entity,entity_scores:rank(scores)? };
                serde_json::to_writer(&mut output,&result)?;writeln!(output)?;
            }
        }
        _ => return Err("usage: mention_context features CANDIDATES | train EXAMPLES MODEL | score MODEL FEATURES".into()),
    }
    output.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record(name: &str, id: &str) -> Result<Record> {
        Ok(serde_json::from_value(
            serde_json::json!({"normalized":format!("who is {name}"),"baseline_entity":id,"parity":true,"candidates":[{"token_start":2,"token_end":3,"byte_start":7,"byte_end":7+name.len(),"text":name,"matches":[{"entity":id,"case_supported":true}],"ambiguous":false,"eligible_entity":id}]}),
        )?)
    }
    #[test]
    fn features_hide_names_ids_and_model_receives_only_grammar_and_shape() -> Result<()> {
        let a = features(&record("atlas", "Q1")?, 0)?;
        let b = features(&record("orion", "Q999")?, 0)?;
        assert_eq!(a.features[0].text, b.features[0].text);
        assert!(!a.features[0].text.contains("atlas"));
        assert!(a.features[0].text.contains("prefix2mention"));
        let mut changed = record("atlas", "Q1")?;
        changed.normalized = "atlas is who".into();
        changed.candidates[0].token_start = 0;
        changed.candidates[0].token_end = 1;
        changed.candidates[0].byte_start = 0;
        changed.candidates[0].byte_end = 5;
        assert_ne!(features(&changed, 0)?.features[0].text, a.features[0].text);
        changed.candidates[0].matches[0].case_supported = false;
        changed.candidates[0].eligible_entity = None;
        assert!(features(&changed, 0)?.features.is_empty());
        changed.candidates[0].matches.push(Match {
            entity: "Q2".into(),
            case_supported: true,
        });
        changed.candidates[0].ambiguous = true;
        assert!(features(&changed, 0)?.features.is_empty());
        let scores = rank([("Q1".into(), 0.2), ("Q1".into(), 0.5), ("Q2".into(), 0.5)])?;
        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0].1 - scores[1].1, 0.0);
        assert!(rank([("Q1".into(), f32::NAN)]).is_err());
        Ok(())
    }
}
