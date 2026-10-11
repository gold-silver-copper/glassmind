use glassmind_facts::{EndpointResolution, Store, Verification};
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    question: String,
    status: String,
    edge: Option<[String; 3]>,
}

#[test]
fn alias_v2_graph_contracts_and_read_only_store() -> Result<(), Box<dyn std::error::Error>> {
    let store: Store =
        serde_json::from_str(include_str!("../../../tools/facts/verification/graph.json"))?;
    store.validate()?;
    let before = serde_json::to_vec(&store)?;
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "../../../tools/facts/verification/cases-alias-v2.json"
    ))?;
    for case in cases {
        let result = store.verify(&case.question);
        let serialized = serde_json::to_value(&result)?;
        assert_eq!(
            serialized.get("status").and_then(|x| x.as_str()),
            Some(case.status.as_str()),
            "{}",
            case.question
        );
        if let Verification::Supported { witness, scope } = result {
            assert_eq!(
                Some([witness.subject.0, witness.property.0, witness.object.0]),
                case.edge,
                "{}",
                case.question
            );
            assert_eq!(scope, "local_snapshot");
        }
    }
    assert_eq!(before, serde_json::to_vec(&store)?);
    Ok(())
}

#[test]
fn edge_truth_and_alias_ambiguity_cannot_choose_a_parse() -> Result<(), Box<dyn std::error::Error>>
{
    #[derive(Deserialize)]
    struct Contrast {
        graph: Store,
        text: String,
    }
    let cases: Vec<Contrast> = serde_json::from_str(include_str!(
        "../../../tools/facts/verification/alias-ambiguity-contrasts.json"
    ))?;
    for case in cases {
        assert!(
            matches!(case.graph.verify(&case.text), Verification::Refused { .. }),
            "{}",
            case.text
        );
    }
    Ok(())
}

#[test]
fn resolution_and_unbounded_edge_membership() -> Result<(), Box<dyn std::error::Error>> {
    let store: Store =
        serde_json::from_str(include_str!("../../../tools/facts/verification/graph.json"))?;
    let result = store.verify("Does Ada work for Q111?");
    assert!(
        matches!(result, Verification::Supported { witness, .. } if witness.subject_resolution == EndpointResolution::Alias && witness.object_resolution == EndpointResolution::ExplicitId && witness.object.0 == "Q111")
    );
    Ok(())
}

#[test]
fn overlapping_predicate_delimiters_refuse() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct Supplement {
        graph: Store,
        cases: Vec<Case>,
    }
    let supplement: Supplement = serde_json::from_str(include_str!(
        "../../../tools/facts/verification/supplemental.json"
    ))?;
    for case in supplement.cases {
        let result = supplement.graph.verify(&case.question);
        let serialized = serde_json::to_value(&result)?;
        assert_eq!(
            serialized.get("status").and_then(|x| x.as_str()),
            Some(case.status.as_str()),
            "{}",
            case.question
        );
        if let Verification::Supported { witness, .. } = result {
            assert_eq!(
                Some([witness.subject.0, witness.property.0, witness.object.0]),
                case.edge
            );
        }
    }
    Ok(())
}
