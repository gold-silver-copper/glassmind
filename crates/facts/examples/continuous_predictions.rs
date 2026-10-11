//! Question-only diagnostic stream; annotations never reach inference.
use glassmind_everyday::IntentModel;
use glassmind_facts::Store;
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let [_, model_path, index_path] = args.as_slice() else {
        return Err("usage: continuous_predictions MODEL INDEX < questions.txt".into());
    };
    let model: IntentModel = serde_json::from_slice(&std::fs::read(model_path)?)?;
    let store: Store = serde_json::from_slice(&std::fs::read(index_path)?)?;
    model.validate()?;
    store.validate()?;
    for line in io::stdin().lock().lines() {
        let question = line?;
        println!(
            "{}",
            serde_json::to_string(&store.answer(&model, &question))?
        );
    }
    Ok(())
}
