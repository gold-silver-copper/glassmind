//! Known-data regression only. Stdin contains questions, never annotations.
use glassmind_everyday::IntentModel;
use glassmind_facts::{Store, disk::DiskStore};
use serde_json::json;
use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let [_, model, index, database] = args.as_slice() else {
        return Err("usage: disk_parity MODEL JSON_INDEX DATABASE < questions".into());
    };
    let started = std::time::Instant::now();
    let model: IntentModel = serde_json::from_slice(&std::fs::read(model)?)?;
    model.validate()?;
    let store: Store = serde_json::from_slice(&std::fs::read(index)?)?;
    store.validate()?;
    let disk = DiskStore::open(std::path::Path::new(database))?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    let mut count = 0;
    for line in io::stdin().lock().lines() {
        let question = line?;
        let legacy = serde_json::to_value(store.answer(&model, &question))?;
        let candidate = serde_json::to_value(disk.answer(&model, &question)?)?;
        let legacy_set = serde_json::to_value(store.answer_set(&model, &question))?;
        let candidate_set = serde_json::to_value(disk.answer_set(&model, &question)?)?;
        if legacy != candidate || legacy_set != candidate_set {
            eprintln!(
                "{}",
                json!({"row": count, "legacy": legacy, "disk": candidate, "legacy_set": legacy_set, "disk_set": candidate_set})
            );
            return Err("disk-store parity failed".into());
        }
        writeln!(
            output,
            "{}",
            json!({"answer": candidate, "set": candidate_set})
        )?;
        count += 1;
    }
    output.flush()?;
    eprintln!(
        "{}",
        json!({"rows": count, "seconds": started.elapsed().as_secs_f64(), "info": disk.info(), "query_plans": disk.query_plans()?})
    );
    Ok(())
}
