mod rational;
mod proposal;
use std::io::{self,BufRead,Write};
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let args:Vec<_>=std::env::args().collect();
    let model=proposal::Model::load(std::path::Path::new(&args[1]))?;
    let mode=args.get(2).ok_or("explicit mode required")?;
    assert!(mode=="--free" || mode=="--train-oracles");
    let mut out=io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line=line?;
        let value=if mode=="--free" {
            let q:String=serde_json::from_str(&line)?;
            model.propose(&q)
        } else {
            let row:serde_json::Value=serde_json::from_str(&line)?;
            let q=row["question"].as_str().ok_or("question")?;
            let gold:proposal::Program=serde_json::from_value(row["program"].clone())?;
            // Free inference has no program argument and runs before the oracle function.
            let free=model.propose(q);
            let diagnostic=proposal::audit_oracles(&model,q,&gold)?;
            serde_json::json!({"index":row["index"],"split":row["split"],"free":free,"diagnostic":diagnostic})
        };
        serde_json::to_writer(&mut out,&value)?; writeln!(out)?;
    }
    Ok(())
}
