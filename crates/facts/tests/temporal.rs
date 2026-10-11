#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use glassmind_facts::temporal::{
    CalendarDate, MAX_TEMPORAL_BYTES, MAX_TEMPORAL_INDEX_BYTES, TemporalIndex,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
const INDEX: &[u8] = include_bytes!("../../../tools/facts/temporal/index.json");
fn asset(statements: &Value) -> Value {
    json!({"version":1,"index_sha256":format!("{:x}",Sha256::digest(INDEX)),"records":statements.as_array().unwrap().iter().map(|s|json!({"subject":"Q1","property":s["mainsnak"]["property"],"object":s["mainsnak"]["datavalue"]["value"]["id"],"source_url":format!("https://www.wikidata.org/wiki/Q1#{}",s["mainsnak"]["property"].as_str().unwrap()),"raw_file":"0000000000000000.json","raw_sha256":"0".repeat(64),"statement":s})).collect::<Vec<_>>()})
}
fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../../../tools/facts/temporal/contracts.json")).unwrap()
}
#[test]
fn frozen_calendar_claim_query_contracts() {
    let cases = cases();
    assert_eq!(cases.len(), 56);
    for c in cases {
        let loaded = TemporalIndex::from_bytes(
            INDEX,
            &serde_json::to_vec(&asset(&c["statements"])).unwrap(),
        )
        .unwrap();
        let actual = serde_json::to_value(
            loaded.verify_at_str(c["date"].as_str().unwrap(), c["question"].as_str().unwrap()),
        )
        .unwrap();
        assert_eq!(actual["status"], c["status"], "{}: {actual}", c["name"]);
        if let Ok(d) = CalendarDate::parse(c["date"].as_str().unwrap()) {
            assert_eq!(
                serde_json::to_value(loaded.verify_at(&d, c["question"].as_str().unwrap()))
                    .unwrap(),
                actual
            );
        }
        if actual["status"] == "unknown"
            && c["name"] != "missing alias"
            && c["name"] != "missing edge"
            && c["name"] != "ambiguous alias"
        {
            assert!(!actual["skipped"].as_array().unwrap().is_empty());
        }
    }
}
#[test]
fn loader_binding_schema_duplicates_and_caps() {
    let a = asset(&cases()[0]["statements"]);
    for key in ["version", "index_sha256", "unexpected"] {
        let mut b = a.clone();
        b[key] = json!("bad");
        assert!(TemporalIndex::from_bytes(INDEX, &serde_json::to_vec(&b).unwrap()).is_err());
    }
    for key in [
        "subject",
        "object",
        "property",
        "raw_sha256",
        "raw_file",
        "source_url",
    ] {
        let mut b = a.clone();
        b["records"][0][key] = json!("bad");
        assert!(
            TemporalIndex::from_bytes(INDEX, &serde_json::to_vec(&b).unwrap()).is_err(),
            "{key}"
        );
    }
    let mut b = a.clone();
    b["records"][0]["statement"]["mainsnak"]["datavalue"]["value"]["id"] = json!("Q99");
    assert!(TemporalIndex::from_bytes(INDEX, &serde_json::to_vec(&b).unwrap()).is_err());
    let mut b = a.clone();
    let r = b["records"][0].clone();
    b["records"].as_array_mut().unwrap().push(r);
    let loaded = TemporalIndex::from_bytes(INDEX, &serde_json::to_vec(&b).unwrap()).unwrap();
    let original = TemporalIndex::from_bytes(INDEX, &serde_json::to_vec(&a).unwrap()).unwrap();
    assert_eq!(
        loaded.verify_at_str("1980-01-01", "Did Q1 work for Q2?"),
        original.verify_at_str("1980-01-01", "Did Q1 work for Q2?")
    );
    b["records"][1]["statement"]["rank"] = json!("preferred");
    assert!(TemporalIndex::from_bytes(INDEX, &serde_json::to_vec(&b).unwrap()).is_err());
    assert!(TemporalIndex::from_bytes(INDEX, &vec![b' '; MAX_TEMPORAL_BYTES + 1]).is_err());
    assert!(TemporalIndex::from_bytes(&vec![b' '; MAX_TEMPORAL_INDEX_BYTES + 1], b"{}").is_err());
    let raw = serde_json::to_string(&a).unwrap().replacen(
        "\"version\":1",
        "\"version\":2,\"version\":1",
        1,
    );
    assert!(TemporalIndex::from_bytes(INDEX, raw.as_bytes()).is_err());
    let mut b = a.clone();
    b["records"] = json!(vec![
        json!({"subject":"","property":"","object":"","source_url":"","raw_file":"","raw_sha256":"","statement":null});
        100001
    ]);
    assert!(
        TemporalIndex::from_bytes(INDEX, &serde_json::to_vec(&b).unwrap())
            .err()
            .unwrap()
            .contains("record cap")
    );
}
#[test]
fn deterministic_statement_selection() {
    let mut a = asset(&cases()[0]["statements"]);
    let mut r = a["records"][0].clone();
    r["statement"]["id"] = json!("Q1$00000000-0000-0000-0000-000000000000");
    a["records"].as_array_mut().unwrap().push(r);
    let run = |a: &Value| {
        serde_json::to_value(
            TemporalIndex::from_bytes(INDEX, &serde_json::to_vec(a).unwrap())
                .unwrap()
                .verify_at_str("1980-01-01", "Did Q1 work for Q2?"),
        )
        .unwrap()
    };
    let first = run(&a);
    a["records"].as_array_mut().unwrap().reverse();
    assert_eq!(run(&a), first);
    assert_eq!(
        first["witness"]["claim_id"],
        "Q1$00000000-0000-0000-0000-000000000000"
    );
}

#[test]
fn historical_statement_ids_and_reason_contracts() {
    let mut a = asset(&cases()[0]["statements"]);
    a["records"][0]["statement"]["id"] = json!("q1$11111111-1111-1111-1111-111111111111");
    let loaded = TemporalIndex::from_bytes(INDEX, &serde_json::to_vec(&a).unwrap()).unwrap();
    assert!(matches!(
        loaded.verify_at_str("1980-01-01", "Did Q1 work for Q2?"),
        glassmind_facts::temporal::TemporalVerification::Supported { .. }
    ));
    assert!(matches!(
        loaded.verify_at_str("1980-01-01", "Did q1 work for Q2?"),
        glassmind_facts::temporal::TemporalVerification::Refused { .. }
    ));
    for (name, reason) in [
        ("bad bound precision 8", "unsupported_precision"),
        ("bad bound timezone 1", "nonzero_timezone"),
        ("bad bound before 1", "nonzero_uncertainty"),
        ("bad bound time +0000-00-00T00:00:00Z", "unsupported_year"),
        ("extra qualifier", "extra_qualifier"),
        ("duplicate start", "duplicate_bound"),
        ("deprecated", "deprecated"),
        ("inverted", "inverted_interval"),
    ] {
        let c = cases().into_iter().find(|c| c["name"] == name).unwrap();
        let loaded = TemporalIndex::from_bytes(
            INDEX,
            &serde_json::to_vec(&asset(&c["statements"])).unwrap(),
        )
        .unwrap();
        let actual =
            serde_json::to_value(loaded.verify_at_str("1980-06-01", "Did Q1 work for Q2?"))
                .unwrap();
        assert_eq!(actual["skipped"][0]["reason"], reason, "{name}");
    }
}
