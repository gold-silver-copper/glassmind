use super::*;

fn evidence() -> PathBuf {
    std::env::var_os("DEDUP_ORDER_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| scratch("dedup-order-evidence"))
}
fn package(root: &Path, code: &str, tests: &str) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("tests")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname=\"probe\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(root.join("src/lib.rs"), code).unwrap();
    std::fs::write(
        root.join("tests/visible.rs"),
        format!("use probe::*;\n{tests}"),
    )
    .unwrap();
}

#[test]
fn frozen_dedup_order_native_contracts() {
    let cases: Value =
        serde_json::from_str(include_str!("../../../tools/coding-dedup-order/cases.json")).unwrap();
    let evidence = evidence();
    std::fs::create_dir_all(&evidence).unwrap();
    let mut records = Vec::new();
    let supplement: Value = serde_json::from_str(include_str!(
        "../../../tools/coding-dedup-order/supplement.json"
    ))
    .unwrap();
    for c in cases
        .as_array()
        .unwrap()
        .iter()
        .chain(supplement.as_array().unwrap())
    {
        let name = c["name"].as_str().unwrap();
        let source = format!("dedup-contract-{name}");
        let root = evidence.join(name);
        let code = c["code"].as_str().unwrap();
        package(&root, code, c["test"].as_str().unwrap());
        assert!(
            run(RustCheck::Build, &source, &root).unwrap().is_empty(),
            "{name}: compile"
        );
        let initial = run(RustCheck::Test, &source, &root).unwrap();
        assert!(
            initial.iter().any(|f| f.item.kind == RustKind::Test),
            "{name}: executed failing test"
        );
        let proposals = dedup_orders(&source, &root);
        assert_eq!(
            proposals.len(),
            c["count"].as_u64().unwrap() as usize,
            "{name}"
        );
        let mut old = legacy_slips(&source, &root, &[]);
        old.extend(extrema(&source, &root));
        let all = slips(&source, &root, &[]);
        assert!(all.starts_with(&old));
        assert_eq!(&all[old.len()..], proposals);
        let mut outputs = Vec::new();
        for (index, p) in proposals.iter().enumerate() {
            let variant = evidence.join(format!("{name}-variant-{index}"));
            copy_tree(&root, &variant).unwrap();
            apply(
                std::slice::from_ref(p),
                &BTreeMap::from([(source.clone(), variant.clone())]),
            )
            .unwrap();
            let after = std::fs::read_to_string(variant.join("src/lib.rs")).unwrap();
            let receiver = p.old.split('.').next().unwrap();
            let expected_old =
                format!("{receiver}.dedup(); /* preserved gap */ {receiver}.sort();");
            let expected_new =
                format!("{receiver}.sort(); /* preserved gap */ {receiver}.dedup();");
            assert_eq!(p.old, expected_old, "{name}");
            assert_eq!(p.new, expected_new, "{name}");
            let start = code.match_indices(&expected_old).nth(index).unwrap().0;
            let expected = format!(
                "{}{}{}",
                &code[..start],
                expected_new,
                &code[start + expected_old.len()..]
            );
            assert_eq!(after, expected, "{name}: exact pair only");
            assert_eq!(
                std::fs::read(variant.join("tests/visible.rs")).unwrap(),
                std::fs::read(root.join("tests/visible.rs")).unwrap()
            );
            assert_eq!(
                std::fs::read(variant.join("Cargo.toml")).unwrap(),
                std::fs::read(root.join("Cargo.toml")).unwrap()
            );
            assert!(
                run(RustCheck::Test, &source, &variant).unwrap().is_empty(),
                "{name}: alternative"
            );
            if name == "two_passing" {
                std::fs::create_dir_all(variant.join("examples")).unwrap();
                std::fs::write(variant.join("examples/output.rs"),
                    "fn main(){println!(\"{:?} {:?}\",probe::left(vec![2,1,2]),probe::right(vec![2,1,2]));}").unwrap();
                let (ok, out) = cargo(
                    &Places::from_env(),
                    &source,
                    &variant,
                    "run",
                    &["--quiet", "--example", "output"],
                    None,
                )
                .unwrap();
                assert!(ok);
                outputs.push(out);
            }
        }
        let mut withdrawn = false;
        let choice =
            verify_native_choice(&source, &root, &proposals, "dedup-order", &mut withdrawn);
        assert_eq!(choice.is_some(), c["accept"].as_bool().unwrap(), "{name}");
        if name == "two_passing" {
            assert!(withdrawn);
            assert_eq!(outputs, vec!["[1, 2] [1, 2, 2]\n", "[1, 2, 2] [1, 2]\n"]);
        }
        if !proposals.is_empty() {
            let tests = initial.iter().map(|f| f.item.clone()).collect::<Vec<_>>();
            assert_eq!(
                undo_slip(&source, &root, &tests),
                choice,
                "{name}: end-to-end"
            );
        }
        assert_eq!(
            std::fs::read_to_string(root.join("src/lib.rs")).unwrap(),
            code
        );
        records.push(
            serde_json::json!({"name":name,"original_compiled":true,"original_tests_failed":true,
            "proposals":proposals,"choice":choice,"withdrawn":withdrawn,"native_outputs":outputs}),
        );
        std::fs::write(
            evidence.join("contracts.json"),
            serde_json::to_vec_pretty(&records).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn bounded_choice_and_old_withdrawal_do_not_fall_through() {
    let evidence = evidence().join("choice-controls");
    let code = "pub fn values(mut xs: Vec<i32>)->Vec<i32>{xs.dedup(); xs.sort(); xs}";
    let root = evidence.join("budget");
    package(
        &root,
        code,
        "#[test] fn t(){assert_eq!(values(vec![2,1,2]),vec![1,2]);}",
    );
    let p = dedup_orders("budget", &root);
    assert_eq!(p.len(), 1);
    let mut withdrawn = false;
    assert!(
        verify_native_choice(
            "budget",
            &root,
            &vec![p[0].clone(); 9],
            "dedup-order",
            &mut withdrawn
        )
        .is_none()
    );
    assert!(withdrawn);
    let mut invalid = p[0].clone();
    invalid.old = "wrong original".into();
    withdrawn = false;
    assert!(
        verify_native_choice("budget", &root, &[invalid], "dedup-order", &mut withdrawn).is_none()
    );
    assert!(withdrawn);
    let mut invalid = p[0].clone();
    invalid.new = "missing_symbol();".into();
    withdrawn = false;
    assert!(
        verify_native_choice("budget", &root, &[invalid], "dedup-order", &mut withdrawn,).is_none()
    );
    assert!(
        withdrawn,
        "compiler errors cannot count as executed failing tests"
    );
    // Two older extrema alternatives and one new vector alternative all pass
    // the same test. The older ambiguity must stop the newer native search.
    let fold = "pub fn left(xs:&[i32])->Option<i32>{if xs.is_empty(){return None;} let mut a=0; for &x in xs {if x>a {a=x;}} Some(a)}";
    let code = format!("{fold}\n{}\n{code}", fold.replace("fn left(", "fn right("));
    let tests = "#[test] fn t(){assert!(left(&[-2])==Some(-2) || right(&[-2])==Some(-2) || values(vec![2,1,2])==vec![1,2]);}";
    let root = evidence.join("older-ambiguity");
    package(&root, &code, tests);
    let initial = run(RustCheck::Test, "older-ambiguity", &root).unwrap();
    assert!(initial.iter().any(|f| f.item.kind == RustKind::Test));
    let new = dedup_orders("older-ambiguity", &root);
    assert_eq!(new.len(), 1);
    assert!(
        verify_native_choice("older-ambiguity", &root, &new, "dedup-order", &mut false).is_some()
    );
    let old = extrema("older-ambiguity", &root);
    assert_eq!(old.len(), 2);
    withdrawn = false;
    assert!(
        verify_native_choice("older-ambiguity", &root, &old, "extremum", &mut withdrawn).is_none()
    );
    assert!(withdrawn);
    assert!(
        undo_slip(
            "older-ambiguity",
            &root,
            &initial.into_iter().map(|f| f.item).collect::<Vec<_>>()
        )
        .is_none()
    );
}
