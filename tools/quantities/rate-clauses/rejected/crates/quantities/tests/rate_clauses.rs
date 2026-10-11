use glassmind_quantities::{Answer, solve};
#[test]
fn frozen_compositional_clauses() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tools/quantities/rate-clauses/authored.json"
    ))
    .unwrap();
    for row in cases.as_array().unwrap() {
        let q = row["question"].as_str().unwrap();
        match solve(q) {
            Answer::Answer { value, unit, .. } => {
                assert_eq!(Some(value.as_str()), row["value"].as_str(), "{q}");
                assert_eq!(unit.as_deref(), row["unit"].as_str(), "{q}");
            }
            Answer::Refusal { reason } => assert!(row["value"].is_null(), "{q}: {reason}"),
        }
    }
}

#[test]
fn supplemental_clause_forms() {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tools/quantities/rate-clauses/supplemental.json"
    ))
    .unwrap();
    for row in rows.as_array().unwrap() {
        let q = row["question"].as_str().unwrap();
        match solve(q) {
            Answer::Answer { value, unit, .. } => {
                assert_eq!(Some(value.as_str()), row["value"].as_str(), "{q}");
                assert_eq!(unit.as_deref(), row["unit"].as_str(), "{q}");
            }
            Answer::Refusal { reason } => assert!(row["value"].is_null(), "{q}: {reason}"),
        }
    }
}

#[test]
fn tank_conditions_and_unused_clauses() {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tools/quantities/rate-clauses/safety.json"
    ))
    .unwrap();
    for row in rows.as_array().unwrap() {
        let q = row["question"].as_str().unwrap();
        match solve(q) {
            Answer::Answer { value, unit, .. } => {
                assert_eq!(Some(value.as_str()), row["value"].as_str(), "{q}");
                assert_eq!(unit.as_deref(), row["unit"].as_str(), "{q}");
            }
            Answer::Refusal { reason } => assert!(row["value"].is_null(), "{q}: {reason}"),
        }
    }
}

#[test]
fn locally_bound_ellipsis() {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tools/quantities/rate-clauses/ellipsis.json"
    ))
    .unwrap();
    for row in rows.as_array().unwrap() {
        let q = row["question"].as_str().unwrap();
        match solve(q) {
            Answer::Answer { value, unit, .. } => {
                assert_eq!(Some(value.as_str()), row["value"].as_str(), "{q}");
                assert_eq!(unit.as_deref(), row["unit"].as_str(), "{q}");
            }
            Answer::Refusal { reason } => assert!(row["value"].is_null(), "{q}: {reason}"),
        }
    }
}
