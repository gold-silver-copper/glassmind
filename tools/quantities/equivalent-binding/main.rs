mod rational;
mod proposal;
use std::io::{self,BufRead,Write};
fn main()->Result<(),Box<dyn std::error::Error>> {
    let args:Vec<_>=std::env::args().collect();
    let mut m=proposal::Model::load(std::path::Path::new(args.get(1).ok_or("model")?))?;
    let mode=args.get(2).ok_or("mode")?;
    let mut out=io::BufWriter::new(io::stdout().lock());
    if mode=="--fit-a" || mode=="--fit-b" {
        let rows=io::stdin().lock().lines().map(|l|Ok(serde_json::from_str::<proposal::FitRow>(&l?)?)).collect::<Result<Vec<_>,Box<dyn std::error::Error>>>()?;
        serde_json::to_writer(&mut out,&proposal::fit(&mut m,rows,mode=="--fit-b",args.get(3).ok_or("dest")?)?)?;writeln!(out)?;
    } else {
        for line in io::stdin().lock().lines() {
            let l=line?;
            let v=if mode=="--free" {m.propose(&serde_json::from_str::<String>(&l)?)} else if mode=="--diagnostics" {
                let row:serde_json::Value=serde_json::from_str(&l)?;
                let q=row.get("question").and_then(|v|v.as_str()).ok_or("question")?;
                let gold:proposal::Program=serde_json::from_value(row.get("program").ok_or("program")?.clone())?;
                let free=m.propose(q);
                serde_json::json!({"index":row.get("index"),"split":row.get("split"),"free":free,"diagnostic":proposal::diagnostics(&m,q,&gold)?})
            } else {return Err("unknown mode".into());};
            serde_json::to_writer(&mut out,&v)?;writeln!(out)?;
        }
    } Ok(())
}
