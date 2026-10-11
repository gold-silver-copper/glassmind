// Development-only probe, compiled unchanged against baseline and candidate libraries.
use glassmind_everyday::IntentModel;
use facts::Store;
use std::io::{self, BufRead};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let [_, model, index] = args.as_slice() else { return Err("MODEL INDEX required".into()); };
    let model: IntentModel = serde_json::from_slice(&std::fs::read(model)?)?;
    let store: Store = serde_json::from_slice(&std::fs::read(index)?)?;
    model.validate()?; store.validate()?;
    for question in io::stdin().lock().lines() {
        println!("{}", serde_json::to_string(&store.answer_set(&model, &question?))?);
    }
    Ok(())
}
