use super::*;

#[test]
fn frozen_extremum_native_contracts() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../tools/coding-extremum/contracts-native.json"
    ))
    .unwrap();
    let evidence = std::env::var_os("EXTREMUM_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| scratch("extremum-contract-evidence"));
    std::fs::create_dir_all(&evidence).unwrap();
    let mut records = Vec::new();
    for c in cases.as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let source = format!("extremum-{name}");
        let root = evidence.join(name);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("tests")).unwrap();
        let code = c["code"].as_str().unwrap();
        let tests = format!("use probe::*;\n{}", c["test"].as_str().unwrap());
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname=\"probe\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\n",
        )
        .unwrap();
        std::fs::write(root.join("src/lib.rs"), code).unwrap();
        std::fs::write(root.join("tests/visible.rs"), &tests).unwrap();
        assert!(
            run(RustCheck::Build, &source, &root).unwrap().is_empty(),
            "{name}: must compile"
        );
        let initial = run(RustCheck::Test, &source, &root).unwrap();
        assert!(
            initial.iter().any(|f| f.item.kind == RustKind::Test),
            "{name}: executed failing test required"
        );
        let proposals = extrema(&source, &root);
        assert_eq!(
            proposals.len(),
            c["count"].as_u64().unwrap() as usize,
            "{name}"
        );
        let mut alternatives = Vec::new();
        for (index, p) in proposals.iter().enumerate() {
            assert_eq!(p.old, "0", "{name}");
            if let Some(new) = c["new"].as_str() {
                assert_eq!(p.new, new, "{name}");
            }
            let variant = root.with_file_name(format!("{name}-variant-{index}"));
            copy_tree(&root, &variant).unwrap();
            apply(
                std::slice::from_ref(p),
                &BTreeMap::from([(source.clone(), variant.clone())]),
            )
            .unwrap();
            let after = std::fs::read_to_string(variant.join("src/lib.rs")).unwrap();
            let expected = if name.starts_with("two_passing") {
                let offset = code.match_indices("= 0;").nth(index).unwrap().0 + 2;
                format!("{}{}{}", &code[..offset], p.new, &code[offset + 1..])
            } else {
                code.replacen("= 0;", &format!("= {};", p.new), 1)
            };
            assert_eq!(after, expected, "one exact initializer replacement: {name}");
            assert!(
                run(RustCheck::Test, &source, &variant).unwrap().is_empty(),
                "{name}"
            );
            if name.starts_with("two_passing") {
                std::fs::create_dir_all(variant.join("examples")).unwrap();
                std::fs::write(variant.join("examples/output.rs"),"fn main(){println!(\"{:?} {:?}\",probe::left(&[-8,-2]),probe::right(&[-8,-2]));}").unwrap();
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
                alternatives.push(out);
            }
        }
        let choice = verify_extrema(&source, &root, &proposals);
        assert_eq!(choice.is_some(), c["new"].is_string(), "{name}");
        if name.starts_with("two_passing") {
            assert_eq!(
                alternatives,
                vec!["Some(-2) Some(0)\n", "Some(0) Some(-2)\n"]
            );
        }
        // End-to-end priority and native verification for every positive case.
        if c["new"].is_string() || name == "two_passing_full" {
            let old = initial.iter().map(|f| f.item.clone()).collect::<Vec<_>>();
            assert_eq!(undo_slip(&source, &root, &old), choice, "{name}");
        }
        assert_eq!(
            std::fs::read_to_string(root.join("src/lib.rs")).unwrap(),
            code
        );
        assert_eq!(
            std::fs::read_to_string(root.join("tests/visible.rs")).unwrap(),
            tests
        );
        records.push(serde_json::json!({"name":name,"initial":initial,"proposals":proposals,"choice":choice,"native_outputs":alternatives}));
        std::fs::write(
            evidence.join("report.json"),
            serde_json::to_string_pretty(&records).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn suffix_budget_and_additional_shapes() {
    let base = "pub fn f(xs: &Vec<i16>)->Option<i16>{if xs.is_empty(){return None;} let mut a:i16=-7; for &e in xs {if a < e {a=e;}} Some(a)}";
    let root = scratch("extremum-supplement");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("tests")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname=\"probe\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\n",
    )
    .unwrap();
    let tests = "use probe::*; #[test] fn edges(){for v in [vec![],vec![i16::MIN],vec![i16::MAX],vec![0],vec![-9,-8],vec![-5,2]] {assert_eq!(f(&v),v.iter().copied().max());}}";
    for (code, tests, replacement) in [
        (base.to_owned(), tests.to_owned(), "<i16>::MIN"),
        (
            base.replace("a < e", "a > e"),
            tests.replace(".max()", ".min()"),
            "<i16>::MAX",
        ),
    ] {
        std::fs::write(root.join("src/lib.rs"), &code).unwrap();
        std::fs::write(root.join("tests/visible.rs"), tests).unwrap();
        let before = run(RustCheck::Test, "supplement", &root).unwrap();
        assert!(before.iter().any(|f| f.item.kind == RustKind::Test));
        let legacy = legacy_slips("supplement", &root, &[]);
        let all = slips("supplement", &root, &[]);
        assert!(all.starts_with(&legacy));
        let new = extrema("supplement", &root);
        assert_eq!(new.len(), 1);
        assert_eq!(all.len(), legacy.len() + 1);
        assert_eq!(new[0].old, "-7");
        assert_eq!(new[0].new, replacement);
        assert_eq!(
            verify_extrema("supplement", &root, &new),
            Some(new[0].clone())
        );
        assert!(verify_extrema("supplement", &root, &vec![new[0].clone(); 9]).is_none());
    }
    for code in [
        base.replace("xs.is_empty()", "!xs.is_empty()"),
        base.replace("return None;", "return Some(0);"),
        base.replace("a=e;", "a=e; break;"),
        base.replace("Some(a)", "Some(a+1)"),
        base.replace("for &e", "#[allow(unused)] for &e"),
        base.replace("let mut a", "let xs = xs; let mut a"),
        base.replace("a=e;", "a=e; a=0;"),
        base.replace("if a < e", "if {println!(\"effect\"); a < e}"),
    ] {
        assert!(
            crate::extremum::proposals("s", "src/lib.rs", &code).is_empty(),
            "{code}"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
