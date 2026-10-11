//! Standalone experimental proposal interface; never solve or talk routing.
use glassmind_quantities::proposal::Model;
use std::io::{self, BufRead, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("model path required")?;
    let model = Model::load(std::path::Path::new(&path))?;
    let mut input = io::stdin().lock();
    let mut output = io::BufWriter::new(io::stdout().lock());
    loop {
        let mut row = Vec::new();
        let n = std::io::Read::take(&mut input, 32770).read_until(b'\n', &mut row)?;
        if n == 0 {
            break;
        }
        if row.len() > 32768 {
            return Err("oversized row".into());
        }
        let q: String = serde_json::from_slice(&row)?;
        serde_json::to_writer(&mut output, &model.propose(&q))?;
        writeln!(output)?;
    }
    Ok(())
}
