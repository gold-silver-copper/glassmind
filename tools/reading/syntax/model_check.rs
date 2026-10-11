// SPDX-License-Identifier: MIT
// Post-freeze API contract probe. No training or model mutation.
use glassmind_reading::syntax::Model;
use std::io::BufReader;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("requires model path")?;
    let original = std::fs::read(&path)?;
    let model = Model::load(BufReader::new(std::fs::File::open(&path)?))?;
    let mut encoded = Vec::new();
    model.save(&mut encoded)?;
    if original != encoded { return Err("learned-model roundtrip differs".into()); }
    let restored = Model::load(encoded.as_slice())?;
    let forms = ["The", "nurse", "called", "the", "teacher", "."].map(String::from);
    if model.infer(&forms)? != restored.infer(&forms)? { return Err("prediction roundtrip differs".into()); }
    for n in [1, 2, 3, 127, 128] {
        let forms = vec!["word".to_string(); n];
        let a = model.infer(&forms)?;
        if a != model.infer(&forms)? || !a.structurally_valid { return Err("nondeterministic/invalid output".into()); }
    }
    if model.infer(&vec!["word".to_string(); 129]).is_ok() { return Err("length bound failed".into()); }
    println!("actual learned model: byte-exact roundtrip, deterministic predictions, valid trees at 1/2/3/127/128, refusal at129");
    Ok(())
}
