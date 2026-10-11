//! Parent expectations frozen before inspecting the worker's implementation.
use glassmind_facts::Store;
use serde_json::{Value, json};

#[test]
fn independent_matching_contracts_including_nul() {
    let store: Store = serde_json::from_str(include_str!(
        "../../../tools/facts/matching/parent/index.json"
    ))
    .unwrap();
    let cases: Vec<Value> = serde_json::from_str(include_str!(
        "../../../tools/facts/matching/parent/cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 30);
    for case in cases {
        let question = case["question"].as_str().unwrap();
        let actual = serde_json::to_value(store.match_question(question)).unwrap();
        let status = actual["status"].as_str().unwrap();
        match case["expected_kind"].as_str().unwrap() {
            "unknown_or_refused" => {
                assert!(matches!(status, "unknown" | "refused"), "{case}: {actual}")
            }
            "matches_or_explicit_unsupported" if status == "refused" => {
                assert_eq!(actual["reason"], "unsupported_grammar");
            }
            _ => {
                assert_eq!(status, "matched", "{case}: {actual}");
                for key in ["total_snapshot_matches", "returned", "truncated"] {
                    assert_eq!(actual[key], case[key], "{case}: {actual}");
                }
                let matches = actual["matches"].as_array().unwrap();
                assert_eq!(
                    matches
                        .iter()
                        .map(|m| m["subject"].clone())
                        .collect::<Vec<_>>(),
                    *case["subjects"].as_array().unwrap()
                );
                let constraints = case["constraints"].as_array().unwrap();
                for matched in matches {
                    let subject = matched["subject"].as_str().unwrap();
                    let expected: Vec<_> = constraints.iter().map(|c| {
                        let property = c[0].as_str().unwrap();
                        json!({"subject":subject,"property":property,"object":c[1],
                            "source_url":format!("https://www.wikidata.org/wiki/{subject}#{property}")})
                    }).collect();
                    assert_eq!(matched["witnesses"], json!(expected));
                }
            }
        }
    }
}
