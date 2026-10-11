#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use glassmind_facts::{
    parse_match_question,
    temporal::{CalendarDate, TemporalIndex},
};
use serde_json::Value;
fn run(limit: usize) {
    let cases: Vec<Value> = serde_json::from_str(include_str!(
        "../../../tools/facts/timed-matching/contracts.json"
    ))
    .unwrap();
    for c in cases.iter().take(limit) {
        let loaded = TemporalIndex::from_bytes(
            c["index"].as_str().unwrap().as_bytes(),
            &serde_json::to_vec(&c["asset"]).unwrap(),
        )
        .unwrap();
        let q = c["question"].as_str().unwrap();
        let d = c["date"].as_str().unwrap();
        let result = serde_json::to_value(loaded.match_at_str(d, q)).unwrap();
        assert_eq!(result["status"], c["status"], "{}: {result}", c["name"]);
        if c["status"] == "matched" {
            for (k, v) in c["expected"].as_object().unwrap() {
                assert_eq!(&result[k], v, "{} field {k}", c["name"]);
            }
        }
        if let Ok(date) = CalendarDate::parse(d) {
            assert_eq!(
                result,
                serde_json::to_value(loaded.match_at(&date, q)).unwrap()
            );
            if let Ok(cs) = parse_match_question(q) {
                assert_eq!(
                    result,
                    serde_json::to_value(loaded.match_constraints_at(&date, &cs)).unwrap()
                );
            }
        }
    }
}
#[test]
fn fixed_sample() {
    run(8)
}
#[test]
fn full_frozen_contracts() {
    run(usize::MAX)
}

#[test]
fn typed_refusals_alias_normalization_and_claim_order() {
    use glassmind_facts::{
        MatchConstraint, MatchPredicate, MatchRefusalReason, MatchUnknownReason,
        TemporalMatchRefusalReason as R, TemporalMatchResult as M,
    };
    let cases: Vec<Value> = serde_json::from_str(include_str!(
        "../../../tools/facts/timed-matching/contracts.json"
    ))
    .unwrap();
    let c = &cases[0];
    let load = |asset: &Value| {
        TemporalIndex::from_bytes(
            c["index"].as_str().unwrap().as_bytes(),
            &serde_json::to_vec(asset).unwrap(),
        )
        .unwrap()
    };
    let loaded = load(&c["asset"]);
    let date = CalendarDate::parse("2000-01-01").unwrap();
    assert_eq!(
        loaded.match_at_str("1900-02-29", "bad"),
        M::Refused {
            reason: R::InvalidDate
        }
    );
    assert_eq!(
        loaded.match_at(&date, "Who was born in Missing and worked for Q2?"),
        M::Refused {
            reason: R::UnsupportedProperty
        }
    );
    assert_eq!(
        loaded.match_at(&date, "Who worked for Missing and was a citizen of Q3?"),
        M::Unknown {
            reason: MatchUnknownReason::ObjectNotUniqueOrMissing
        }
    );
    for cs in [
        vec![],
        vec![
            MatchConstraint {
                predicate: MatchPredicate::Employer,
                object: "Q2".into()
            };
            4
        ],
    ] {
        assert_eq!(
            loaded.match_constraints_at(&date, &cs),
            M::Refused {
                reason: R::Question(MatchRefusalReason::InvalidConditionCount)
            }
        );
    }
    let alias = loaded.match_at(&date, "Who worked for Employer and worked for Q2?");
    assert_eq!(
        alias,
        loaded.match_at(&date, "Who worked for Q2 and worked for Q2?")
    );
    let mut a = c["asset"].clone();
    a["records"].as_array_mut().unwrap().reverse();
    assert_eq!(
        loaded.match_at(&date, c["question"].as_str().unwrap()),
        load(&a).match_at(&date, c["question"].as_str().unwrap())
    );
    let mut index: Value = serde_json::from_str(c["index"].as_str().unwrap()).unwrap();
    index["entities"]["Q2"]["aliases"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!("Employer and was a citizen of Q3"));
    let bytes = serde_json::to_vec(&index).unwrap();
    use sha2::{Digest, Sha256};
    a["index_sha256"] = serde_json::json!(format!("{:x}", Sha256::digest(&bytes)));
    let reserved = TemporalIndex::from_bytes(&bytes, &serde_json::to_vec(&a).unwrap()).unwrap();
    assert_eq!(
        reserved.match_at(&date, "Who worked for Employer and was a citizen of Q3?"),
        M::Refused {
            reason: R::Question(MatchRefusalReason::UnsupportedGrammar)
        }
    );
}

#[test]
fn distinct_citizenships_use_distinct_original_claims() {
    let cases: Vec<Value> = serde_json::from_str(include_str!(
        "../../../tools/facts/timed-matching/contracts.json"
    ))
    .unwrap();
    let c = &cases[2];
    let mut index: Value = serde_json::from_str(c["index"].as_str().unwrap()).unwrap();
    let claims = index["entities"]["Q1"]["claims"].as_object_mut().unwrap();
    let objects = claims.remove("P108").unwrap();
    claims.insert("P27".into(), objects);
    let mut asset = c["asset"].clone();
    asset["records"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["property"] == "P108");
    for r in asset["records"].as_array_mut().unwrap() {
        r["property"] = serde_json::json!("P27");
        r["source_url"] = serde_json::json!("https://www.wikidata.org/wiki/Q1#P27");
        r["statement"]["mainsnak"]["property"] = serde_json::json!("P27");
    }
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(&index).unwrap();
    asset["index_sha256"] = serde_json::json!(format!("{:x}", Sha256::digest(&bytes)));
    let loaded = TemporalIndex::from_bytes(&bytes, &serde_json::to_vec(&asset).unwrap()).unwrap();
    let result = serde_json::to_value(loaded.match_at_str(
        "2000-01-01",
        "Who was a citizen of Q2 and is a citizen of Q4?",
    ))
    .unwrap();
    assert_eq!(result["total_recorded_interval_matches"], 1);
    let ws = result["matches"][0]["witnesses"].as_array().unwrap();
    assert_eq!(ws.len(), 2);
    for (w, r) in ws.iter().zip(asset["records"].as_array().unwrap()) {
        assert_eq!(w["claim_id"], r["statement"]["id"]);
        assert_eq!(w["edge"]["object"], r["object"]);
        assert_eq!(w["edge"]["property"], r["property"]);
        assert_eq!(
            w["start"]["raw"],
            r["statement"]["qualifiers"]["P580"][0]["datavalue"]["value"]
        );
        assert_eq!(
            w["end"]["raw"],
            r["statement"]["qualifiers"]["P582"][0]["datavalue"]["value"]
        );
        assert_eq!(w["raw_sha256"], r["raw_sha256"]);
        assert_eq!(w["index_sha256"], asset["index_sha256"]);
    }
}
