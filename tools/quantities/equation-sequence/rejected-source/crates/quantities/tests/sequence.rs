use glassmind_quantities::proposal::{Arg, Model, Op, Program, Step, numbers};
fn divide(a: Arg, b: Arg) -> Program {
    Program {
        steps: vec![Step {
            op: Op::Divide,
            left: a,
            right: b,
        }],
    }
}
#[test]
fn source_spans_and_fraction_slots() {
    let q = "Éva has 1,200.50 and 3/4.";
    let ns = numbers(q).unwrap();
    assert_eq!(
        ns.iter().map(|n| n.value.as_str()).collect::<Vec<_>>(),
        ["2401/2", "3", "4"]
    );
    for n in ns {
        assert_eq!(&q[n.start..n.end], n.literal);
    }
    let (_, v) = divide(Arg::Input(1), Arg::Input(2)).execute(q).unwrap();
    assert_eq!(v.to_string(), "3/4");
}
#[test]
fn unsupported_spelling_and_limits_refuse() {
    for q in [
        "1e3 plus 2",
        "2nd and 3",
        "1,23 and 4",
        "1.2.3",
        "１２ plus 3",
        "two and 2",
        "1--2",
        "−2",
        "- 2",
    ] {
        assert!(numbers(q).is_err(), "{q}");
    }
    assert!(numbers(&"x".repeat(4097)).is_err());
    assert!(numbers(&"1 ".repeat(17)).is_err());
    assert!(numbers("-0.5 and +.25").is_ok());
}
#[test]
fn exact_execution_bounds() {
    assert!(
        divide(Arg::Input(0), Arg::Input(1))
            .execute("10 and 0")
            .is_err()
    );
    assert!(
        divide(Arg::Input(2), Arg::Constant(1))
            .execute("10")
            .is_err()
    );
    assert!(divide(Arg::Step(0), Arg::Constant(1)).validate().is_err());
    assert!(
        divide(Arg::Constant(20), Arg::Constant(1))
            .validate()
            .is_err()
    );
    assert!(Program { steps: vec![] }.validate().is_err());
    let p = Program {
        steps: vec![Step {
            op: Op::Multiply,
            left: Arg::Input(0),
            right: Arg::Input(0),
        }],
    };
    assert!(
        p.execute("170141183460469231731687303715884105727")
            .is_err()
    );
    assert!(serde_json::from_str::<Program>(r#"{"steps":[{"op":"power","left":{"kind":"input","index":0},"right":{"kind":"constant","index":1}}]}"#).is_err());
}
#[test]
fn model_and_feature_state_contracts() {
    use glassmind_quantities::proposal::{base_features, legal, state_features};
    let mut m = Model::new("authored prefit contracts".into());
    assert!(m.validate().is_ok());
    m.weights[0][0] = f32::NAN;
    assert!(m.validate().is_err());
    assert!(Model::from_bytes(&vec![b' '; 32_000_001]).is_err());
    let q = "Éva has 8 bolts and receives 2 bolts.";
    let b = base_features(q, &numbers(q).unwrap());
    let empty = Program { steps: vec![] };
    let graph = divide(Arg::Input(0), Arg::Input(1));
    assert_ne!(
        state_features(&b, &empty, None, None),
        state_features(&b, &graph, None, None)
    );
    assert_ne!(
        state_features(&b, &empty, Some(0), Some(0)),
        state_features(&b, &empty, Some(0), Some(1))
    );
    assert!(!legal(1, 2, 0).contains(&36));
    assert!(legal(1, 2, 1).contains(&36));
    assert!(!legal(0, 2, 0).contains(&4));
    assert!(legal(0, 2, 1).contains(&4));
    let bytes = serde_json::to_vec(&Model::new("test".into())).unwrap();
    let loaded = Model::from_bytes(&bytes).unwrap();
    let out = loaded.propose(q);
    assert_eq!(out["semantics"], "unverified");
    assert_eq!(out["model_sha256"].as_str().unwrap().len(), 64);
}
