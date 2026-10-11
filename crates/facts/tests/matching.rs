use glassmind_facts::{
    EntityId, MatchConstraint, MatchPredicate, MatchResult, Store, parse_match_question,
};
use serde::Deserialize;

fn fixture() -> Result<Store, serde_json::Error> {
    serde_json::from_str(include_str!("../../../tools/facts/matching/graph.json"))
}
#[derive(Deserialize)]
struct Case {
    question: String,
    status: String,
    #[serde(default)]
    ids: Vec<String>,
    #[serde(default)]
    conditions: Vec<[String; 2]>,
}
#[test]
fn frozen_contracts_exact_sets_and_witnesses() {
    let store = fixture().unwrap();
    let before = serde_json::to_vec(&store).unwrap();
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("../../../tools/facts/matching/contracts.json")).unwrap();
    for case in cases {
        let result = store.match_question(&case.question);
        assert_eq!(
            serde_json::to_value(&result).unwrap()["status"],
            case.status,
            "{}",
            case.question
        );
        if let MatchResult::Matched {
            matches,
            total_snapshot_matches,
            returned,
            truncated,
            constraints,
            ..
        } = result
        {
            assert_eq!(total_snapshot_matches, case.ids.len(), "{}", case.question);
            assert_eq!(returned, case.ids.len().min(10));
            assert_eq!(truncated, case.ids.len() > 10);
            assert_eq!(
                matches
                    .iter()
                    .map(|m| m.subject.0.clone())
                    .collect::<Vec<_>>(),
                case.ids.into_iter().take(10).collect::<Vec<_>>()
            );
            let mut expected = case.conditions;
            expected.sort();
            assert_eq!(
                constraints
                    .iter()
                    .map(|c| [c.property.0.clone(), c.object.0.clone()])
                    .collect::<Vec<_>>(),
                expected
            );
            for m in matches {
                assert_eq!(m.witnesses.len(), constraints.len());
                for (w, c) in m.witnesses.iter().zip(&constraints) {
                    assert_eq!(w.subject, m.subject);
                    assert_eq!(w.property, c.property);
                    assert_eq!(w.object, c.object);
                    assert!(store.entities[&w.subject].claims[&w.property].contains(&w.object));
                    assert_eq!(
                        w.source_url,
                        format!(
                            "https://www.wikidata.org/wiki/{}#{}",
                            w.subject.0, w.property.0
                        )
                    );
                }
            }
        }
    }
    assert_eq!(before, serde_json::to_vec(&store).unwrap());
}
#[test]
fn typed_constraints_order_duplicates_and_graph_order_invariant() {
    let mut store = fixture().unwrap();
    let mut constraints = vec![
        MatchConstraint {
            predicate: MatchPredicate::Employer,
            object: "Studio".into(),
        },
        MatchConstraint {
            predicate: MatchPredicate::Birth,
            object: "Harbor".into(),
        },
    ];
    let result = store.match_constraints(&constraints);
    constraints.reverse();
    constraints.push(constraints[0].clone());
    assert_eq!(result, store.match_constraints(&constraints));
    for e in store.entities.values_mut() {
        for objects in e.claims.values_mut() {
            objects.reverse();
            objects.extend(objects.clone());
        }
    }
    assert_eq!(result, store.match_constraints(&constraints));
    let other: Store = serde_json::from_slice(&serde_json::to_vec(&store).unwrap()).unwrap();
    assert_eq!(result, other.match_constraints(&constraints));
    let empty = Store {
        entities: Default::default(),
    };
    let constraints = vec![
        MatchConstraint {
            predicate: MatchPredicate::Birth,
            object: "Q101".into(),
        },
        MatchConstraint {
            predicate: MatchPredicate::Employer,
            object: "Q104".into(),
        },
    ];
    assert!(matches!(
        empty.match_constraints(&constraints),
        MatchResult::Matched {
            total_snapshot_matches: 0,
            ..
        }
    ));
    for length in [0, 1, 4] {
        assert!(matches!(
            store.match_constraints(&vec![constraints[0].clone(); length]),
            MatchResult::Refused { .. }
        ));
    }
}
#[test]
fn graph_cannot_disambiguate_and_truncation_follows_intersection() {
    let mut store = fixture().unwrap();
    let ambiguous = "Who was born in Twin and is a citizen of Republic?";
    let before = store.match_question(ambiguous);
    assert!(matches!(before, MatchResult::Unknown { .. }));
    for entity in store.entities.values_mut() {
        entity.claims.clear();
    }
    assert_eq!(before, store.match_question(ambiguous));
    let mut store = fixture().unwrap();
    // Remove second condition from the first thirteen subjects: a match beyond
    // the first ten candidates must still be found.
    for i in 300..313 {
        store
            .entities
            .get_mut(&EntityId(format!("Q{i}")))
            .unwrap()
            .claims
            .remove(&glassmind_facts::PropertyId("P108".into()));
    }
    let result = store.match_question("Who was born in Q900 and works for Q901?");
    assert!(
        matches!(result,MatchResult::Matched {matches,total_snapshot_matches:1,..} if matches[0].subject.0=="Q313")
    );
}
#[test]
fn typed_invalid_endpoints_and_complete_parse() {
    let store = fixture().unwrap();
    for object in [
        "Q0".to_string(),
        "Q101x".into(),
        "x".repeat(257),
        "Harbor\0".into(),
    ] {
        let c = [
            MatchConstraint {
                predicate: MatchPredicate::Birth,
                object,
            },
            MatchConstraint {
                predicate: MatchPredicate::Employer,
                object: "Q104".into(),
            },
        ];
        assert!(matches!(
            store.match_constraints(&c),
            MatchResult::Refused { .. }
        ));
    }
    assert_eq!(
        parse_match_question("Who was born in Évora and works for Born in Bay?").unwrap()[1].object,
        "Born in Bay"
    );
}

#[test]
fn reserved_separator_alias_is_refused_without_using_claims()
-> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct Contrast {
        graph: Store,
        question: String,
    }
    let mut c: Contrast = serde_json::from_str(include_str!(
        "../../../tools/facts/matching/separator-contract.json"
    ))?;
    assert!(matches!(
        c.graph.match_question(&c.question),
        MatchResult::Refused { .. }
    ));
    for entity in c.graph.entities.values_mut() {
        entity.claims.clear();
    }
    assert!(matches!(
        c.graph.match_question(&c.question),
        MatchResult::Refused { .. }
    ));
    Ok(())
}
