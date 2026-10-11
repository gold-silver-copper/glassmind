// SPDX-License-Identifier: MIT
// Actual approved-model compatibility probe, independent of DEV labels.
use glassmind_reading::syntax::{Input, Model};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("requires model path")?;
    let bytes = std::fs::read(path)?;
    let model = Model::load(bytes.as_slice())?;
    let mut encoded = Vec::new();
    model.save(&mut encoded)?;
    assert_eq!(bytes, encoded);
    for n in [1, 2, 3, 127, 128] {
        let forms = vec!["word".to_owned(); n];
        let greedy = model.infer(&forms)?;
        let mut one = model.infer_normalized_beam(&forms, 1)?;
        assert_eq!(one.search_mode.take().as_deref(), Some("normalized_beam_1_greedy_compat"));
        assert_eq!(greedy, one);
        let four = model.infer_normalized_beam(&forms, 4)?;
        assert_eq!(four, model.infer_normalized_beam(&forms, 4)?);
        assert_eq!(four.upos, greedy.upos);
        assert!(four.structurally_valid);
    }
    for width in [0, 2, 3, 5, usize::MAX] {
        assert!(model.infer_normalized_beam(&["x".to_owned()], width).is_err());
        assert!(model.predict_normalized_beam(Input { forms: None, text: Some("x".into()) }, width).is_err());
    }
    assert!(model.infer_normalized_beam(&vec!["x".to_owned(); 129], 4).is_err());
    println!("approved-model byte roundtrip; beam1 compatibility, beam4 determinism/tags/trees at1/2/3/127/128; invalid widths and129 refused");
    Ok(())
}
