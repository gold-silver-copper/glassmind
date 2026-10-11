#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use super::*;
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = PathBuf::from(std::env::var("TMPDIR").unwrap()).join(format!(
            "draft-contract-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        let constant = |label| json!({"constant_label":label,"evidence_rows":1});
        let model = json!({"version":1,"router":constant("svc"),"services":{"svc":{"version":2,"service":"svc","intent":constant("SET:Find"),"categorical":{},"spans":constant("NONE"),"training_dialogues":1,"training_frames":1}}});
        fs::write(p.join("model.json"), serde_json::to_vec(&model).unwrap()).unwrap();
        fs::write(p.join("schema.json"), br#"[{"service_name":"svc","slots":[],"intents":[{"name":"Find","required_slots":[],"optional_slots":{}}]}]"#).unwrap();
        let f = Self(p);
        f.remanifest();
        f
    }
    fn remanifest(&self) {
        let m = BundleManifest {
            version: 1,
            model_sha256: digest(&fs::read(self.0.join("model.json")).unwrap()),
            schema_sha256: digest(&fs::read(self.0.join("schema.json")).unwrap()),
            source: "authored constant model fixture".into(),
            license: "MIT OR Apache-2.0".into(),
            scope: "authored contracts only".into(),
        };
        fs::write(
            self.0.join("manifest.json"),
            serde_json::to_vec(&m).unwrap(),
        )
        .unwrap();
    }
    fn bundle(&self) -> DialogueBundle {
        DialogueBundle::load(&self.0).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn snapshot(s: &DraftSession) -> Value {
    serde_json::to_value(s.snapshot()).unwrap()
}
fn history(s: &DraftSession) -> Value {
    serde_json::from_slice(&s.serialize_history().unwrap()).unwrap()
}
fn rejects(s: &mut DraftSession, h: Value) {
    let before = snapshot(s);
    let bytes = s.serialize_history().unwrap();
    assert!(s.restore_json(&serde_json::to_vec(&h).unwrap()).is_err());
    assert_eq!(snapshot(s), before);
    assert_eq!(s.serialize_history().unwrap(), bytes);
}
#[test]
fn sequence_restart_every_boundary_reset_interleave_and_show() {
    let f = Fixture::new();
    let b = f.bundle();
    let mut a = b.new_session();
    let mut other = b.new_session();
    assert!(a.snapshot().draft.is_none());
    assert!(a.snapshot().user_prediction().is_none());
    assert_eq!(
        snapshot(&a),
        snapshot(&b.resume_json(&a.serialize_history().unwrap()).unwrap())
    );
    let turns = [
        (false, "Find something"),
        (true, "I can offer this one"),
        (false, "Yes please"),
        (true, "Anything else?"),
        (false, "No thanks"),
    ];
    for (system, text) in turns {
        let before = a.serialize_history().unwrap();
        assert_eq!(snapshot(&a), snapshot(&a));
        assert_eq!(a.serialize_history().unwrap(), before);
        if system {
            a.append_system(text).unwrap();
        } else {
            a.append_user(text).unwrap();
        }
        assert_eq!(a.snapshot().user_prediction().is_some(), !system);
        assert_eq!(
            snapshot(&a),
            snapshot(&b.resume_json(&a.serialize_history().unwrap()).unwrap())
        );
        assert_eq!(other.snapshot().total_turns, 0);
    }
    other.append_user("Independent").unwrap();
    let saved = snapshot(&other);
    a.reset();
    assert_eq!(a.snapshot().total_turns, 0);
    assert_eq!(snapshot(&other), saved);
    a.append_user("Independent").unwrap();
    assert_eq!(snapshot(&a), snapshot(&other));
    assert_eq!(a.snapshot().user_turns, 1);
    assert_eq!(a.snapshot().draft.unwrap().turn_index, 0);
}
#[test]
fn malformed_history_and_atomic_failures() {
    let f = Fixture::new();
    let b = f.bundle();
    let mut s = b.new_session();
    s.append_user("  café 😀  ").unwrap();
    assert_eq!(s.history()[0].utterance, "  café 😀  ");
    let base = history(&s);
    for (key, val) in [
        ("version", json!(2)),
        ("version", json!(null)),
        ("bundle_identity", json!("bad")),
        ("frames", json!([])),
        ("system_actions", json!([])),
    ] {
        let mut h = base.clone();
        h[key] = val;
        rejects(&mut s, h);
    }
    for (key, val) in [
        ("frames", json!([])),
        ("annotations", json!({})),
        ("speaker", json!("TOOL")),
        ("utterance", json!("")),
        ("utterance", json!(" \t\n")),
        ("utterance", json!("a\0b")),
        ("utterance", json!("x".repeat(4097))),
    ] {
        let mut h = base.clone();
        h["turns"][0][key] = val;
        rejects(&mut s, h);
    }
    let before = snapshot(&s);
    for bad in [
        b"{".as_slice(),
        b"[]",
        b"null",
        b"{\"version\":1,\"version\":1}",
        &[0xff],
    ] {
        assert!(s.restore_json(bad).is_err());
        assert_eq!(snapshot(&s), before);
    }
    assert!(
        s.restore_json(&vec![b' '; MAX_HISTORY_JSON_BYTES + 1])
            .is_err()
    );
    for bad in ["".into(), " \n".into(), "a\0b".into(), "x".repeat(4097)] {
        assert!(s.append_user(bad).is_err());
        assert_eq!(snapshot(&s), before);
    }
    let persisted = s.serialize_history().unwrap();
    let h: Value = serde_json::from_slice(&persisted).unwrap();
    assert_eq!(h.as_object().unwrap().len(), 3);
    assert_eq!(h["turns"][0].as_object().unwrap().len(), 2);
}
#[test]
fn exact_utf8_turn_and_total_byte_boundaries() {
    let f = Fixture::new();
    let b = f.bundle();
    let mut s = b.new_session();
    let utf8 = "😀".repeat(1024);
    assert_eq!(utf8.len(), 4096);
    s.append_system(&utf8).unwrap();
    assert_eq!(s.history()[0].utterance, utf8);
    assert!(s.append_system(format!("{utf8}x")).is_err());
    for _ in 1..16 {
        s.append_system(&utf8).unwrap();
    }
    assert_eq!(s.utterance_bytes, 65536);
    let h = history(&s);
    assert_eq!(
        snapshot(&s),
        snapshot(&b.resume_json(&s.serialize_history().unwrap()).unwrap())
    );
    let before = snapshot(&s);
    assert!(s.append_system("x").is_err());
    assert_eq!(snapshot(&s), before);
    let mut oversized = h;
    oversized["turns"]
        .as_array_mut()
        .unwrap()
        .push(json!({"speaker":"SYSTEM","utterance":"x"}));
    rejects(&mut s, oversized);
    s.reset();
    for _ in 0..128 {
        s.append_system("x").unwrap();
    }
    assert_eq!(s.history().len(), 128);
    assert!(b.resume_json(&s.serialize_history().unwrap()).is_ok());
    let before = snapshot(&s);
    assert!(s.append_user("x").is_err());
    assert_eq!(snapshot(&s), before);
    let mut h = history(&s);
    h["turns"]
        .as_array_mut()
        .unwrap()
        .push(json!({"speaker":"USER","utterance":"x"}));
    rejects(&mut s, h);
}
#[test]
fn loader_missing_corrupt_version_unknown_fields_and_identity() {
    let f = Fixture::new();
    let b = f.bundle();
    let mut s = b.new_session();
    s.append_user("Find").unwrap();
    let h = s.serialize_history().unwrap();
    let original = fs::read(f.0.join("manifest.json")).unwrap();
    for (key, value) in [
        ("version", json!(2)),
        ("source", json!(" ")),
        ("license", json!("")),
        ("scope", json!("")),
        ("unknown", json!(1)),
        ("model_sha256", json!("0".repeat(64))),
        ("schema_sha256", json!("0".repeat(64))),
    ] {
        let mut m: Value = serde_json::from_slice(&original).unwrap();
        m[key] = value;
        fs::write(f.0.join("manifest.json"), serde_json::to_vec(&m).unwrap()).unwrap();
        assert!(DialogueBundle::load(&f.0).is_err());
    }
    fs::write(f.0.join("manifest.json"), &original).unwrap();
    for file in ["manifest.json", "model.json", "schema.json"] {
        let path = f.0.join(file);
        let bytes = fs::read(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert!(DialogueBundle::load(&f.0).is_err());
        fs::write(&path, b"broken").unwrap();
        assert!(DialogueBundle::load(&f.0).is_err());
        fs::write(&path, bytes).unwrap();
    }
    for file in ["model.json", "schema.json"] {
        let path = f.0.join(file);
        let bytes = fs::read(&path).unwrap();
        let mut changed = bytes.clone();
        changed.push(b' ');
        fs::write(&path, changed).unwrap();
        assert!(DialogueBundle::load(&f.0).is_err());
        f.remanifest();
        let changed = f.bundle();
        assert_ne!(changed.identity(), b.identity());
        assert!(changed.resume_json(&h).is_err());
        fs::write(path, bytes).unwrap();
        f.remanifest();
    }
    // Existing loaded bundle is immutable even if its files subsequently change.
    assert!(b.resume_json(&h).is_ok());
    fs::write(f.0.join("schema.json"), b"[]").unwrap();
    f.remanifest();
    assert!(DialogueBundle::load(&f.0).is_err());
    fs::write(f.0.join("model.json"), b"{}").unwrap();
    f.remanifest();
    assert!(DialogueBundle::load(&f.0).is_err());
}
#[test]
fn loader_enforces_actual_file_size_caps() {
    for (name, cap) in [
        ("manifest.json", MANIFEST_BYTES),
        ("schema.json", SCHEMA_BYTES),
        ("model.json", MODEL_BYTES),
    ] {
        let f = Fixture::new();
        File::create(f.0.join(name))
            .unwrap()
            .set_len(cap as u64 + 1)
            .unwrap();
        let err = DialogueBundle::load(&f.0).err().unwrap();
        assert!(err.contains("exceeds"), "{err}");
    }
}

#[test]
fn authored_system_offer_survives_restart_without_cross_session_leakage() {
    let f = Fixture::new();
    let model_path = f.0.join("model.json");
    let mut model: Value = serde_json::from_slice(&fs::read(&model_path).unwrap()).unwrap();
    // Authored fixture counts satisfy the existing model's validation contract;
    // they are not fitted evidence or an accuracy experiment.
    model["offers"] = json!({"version":1,"rules":[{"service":"svc","slot":"place","left":["to"],"right":["?"],"fit_correct":10,"fit_wrong":0,"calibration_correct":3,"calibration_wrong":0,"distinct_values":6}]});
    fs::write(model_path, serde_json::to_vec(&model).unwrap()).unwrap();
    fs::write(f.0.join("schema.json"), br#"[{"service_name":"svc","slots":[{"name":"place","is_categorical":false,"possible_values":[]}],"intents":[{"name":"Find","required_slots":["place"],"optional_slots":{}}]}]"#).unwrap();
    f.remanifest();
    let bundle = f.bundle();
    let mut original = bundle.new_session();
    original.append_user("Find somewhere").unwrap();
    original.append_system("Go to École des Arts?").unwrap();
    let mut restarted = bundle
        .resume_json(&original.serialize_history().unwrap())
        .unwrap();
    let mut other = bundle.new_session();
    other.append_system("Go to Beta?").unwrap();
    for s in [&mut original, &mut restarted, &mut other] {
        s.append_user("Yes please").unwrap();
    }
    assert_eq!(snapshot(&original), snapshot(&restarted));
    let prediction = restarted.snapshot().draft.unwrap();
    assert_eq!(prediction.offer_proposals.len(), 1);
    assert_eq!(prediction.offer_proposals[0].value, "École des Arts");
    assert_eq!(prediction.offer_proposals[0].source_turn, 1);
    assert_eq!(prediction.turn_index, 2);
    assert_eq!(
        snapshot(&other)["draft"]["frames"][0]["state"]["slot_values"]["place"],
        json!(["Beta"])
    );
    restarted.reset();
    restarted.append_user("Yes please").unwrap();
    assert!(
        restarted
            .snapshot()
            .draft
            .unwrap()
            .offer_proposals
            .is_empty()
    );
    let mut bad = history(&original);
    bad["turns"]
        .as_array_mut()
        .unwrap()
        .push(json!({"speaker":"SYSTEM","utterance":"\0"}));
    rejects(&mut original, bad);
}
