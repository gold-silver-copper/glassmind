use super::*;
fn evidence() -> PathBuf {
    std::env::var_os("COMPARATOR_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| scratch("comparator-evidence"))
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
fn frozen_comparator_native_contracts() {
    let cases: Value =
        serde_json::from_str(include_str!("../../../tools/coding-comparator/cases.json")).unwrap();
    let ev = evidence();
    std::fs::create_dir_all(&ev).unwrap();
    let mut records = Vec::new();
    for c in cases.as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let source = format!("cmp-{name}");
        let root = ev.join(name);
        let code = c["code"].as_str().unwrap();
        package(&root, code, c["test"].as_str().unwrap());
        assert!(
            run(RustCheck::Build, &source, &root).unwrap().is_empty(),
            "{name}: compile"
        );
        let initial = run(RustCheck::Test, &source, &root).unwrap();
        if c["original_pass"] == true {
            assert!(initial.is_empty());
        } else {
            assert!(
                initial.iter().any(|f| f.item.kind == RustKind::Test),
                "{name}: executed failure"
            );
        }
        let p = comparators(&source, &root);
        assert_eq!(p.len(), c["count"].as_u64().unwrap() as usize, "{name}");
        let mut old = legacy_slips(&source, &root, &[]);
        old.extend(extrema(&source, &root));
        old.extend(dedup_orders(&source, &root));
        assert_eq!(slips(&source, &root, &[]), [old, p.clone()].concat());
        let mut outputs = Vec::new();
        let mut passed = 0;
        for (i, change) in p.iter().enumerate() {
            let variant = ev.join(format!("{name}-variant-{i}"));
            copy_tree(&root, &variant).unwrap();
            apply(
                std::slice::from_ref(change),
                &BTreeMap::from([(source.clone(), variant.clone())]),
            )
            .unwrap();
            let start = code
                .split_inclusive('\n')
                .take(change.line as usize - 1)
                .map(str::len)
                .sum::<usize>()
                + code
                    .lines()
                    .nth(change.line as usize - 1)
                    .unwrap()
                    .char_indices()
                    .nth(change.column as usize)
                    .unwrap()
                    .0;
            let expected = format!(
                "{}{}{}",
                &code[..start],
                change.new,
                &code[start + change.old.len()..]
            );
            assert_eq!(
                std::fs::read_to_string(variant.join("src/lib.rs")).unwrap(),
                expected,
                "{name}: outside bytes"
            );
            for path in ["tests/visible.rs", "Cargo.toml"] {
                assert_eq!(
                    std::fs::read(root.join(path)).unwrap(),
                    std::fs::read(variant.join(path)).unwrap()
                );
            }
            let result = run(RustCheck::Test, &source, &variant).unwrap();
            assert!(
                result.iter().all(|f| f.item.kind == RustKind::Test),
                "{name}: variant compiled"
            );
            if result.is_empty() {
                passed += 1;
                if c["outputs"] == true {
                    std::fs::create_dir_all(variant.join("examples")).unwrap();
                    std::fs::write(variant.join("examples/output.rs"),"fn main(){let v=vec![probe::Row{x:1,y:String::new()},probe::Row{x:2,y:String::new()}];println!(\"{} {}\",probe::order(&v)[0].x,probe::other(&v)[0].x);}").unwrap();
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
        }
        let mut withdrawn = false;
        let choice = verify_native_choice(&source, &root, &p, "comparator", &mut withdrawn);
        assert_eq!(choice.is_some(), c["accept"].as_bool().unwrap(), "{name}");
        if c["accept"] == true {
            assert_eq!(passed, 1, "{name}");
        }
        if c["outputs"] == true {
            assert_eq!(outputs, vec!["2 1\n", "1 2\n"]);
            assert!(withdrawn);
        }
        {
            assert_eq!(
                undo_slip(
                    &source,
                    &root,
                    &initial.iter().map(|f| f.item.clone()).collect::<Vec<_>>()
                ),
                choice,
                "{name}: end to end"
            );
        }
        assert_eq!(
            std::fs::read_to_string(root.join("src/lib.rs")).unwrap(),
            code
        );
        records.push(serde_json::json!({"name":name,"original_compiled":true,"original_failed":!initial.is_empty(),"proposals":p,"passing":passed,"choice":choice,"outputs":outputs,"withdrawn":withdrawn}));
        std::fs::write(
            ev.join("contracts.json"),
            serde_json::to_vec_pretty(&records).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn comparator_older_dedup_veto_and_incomplete() {
    let ev = evidence().join("controls");
    let lib = "pub fn left(mut xs:Vec<i32>)->Vec<i32>{xs.dedup();xs.sort();xs}\npub fn right(mut xs:Vec<i32>)->Vec<i32>{xs.dedup();xs.sort();xs}";
    let main = "#[derive(Clone)] struct Row{x:i32} fn order(mut xs:Vec<Row>)->Vec<Row>{xs.sort_by(|a,b|a.x.cmp(&b.x));xs} fn main(){}\n#[cfg(test)] mod tests {#[test] fn t(){assert!(probe::left(vec![2,1,2])==vec![1,2] || probe::right(vec![2,1,2])==vec![1,2] || super::order(vec![super::Row{x:1},super::Row{x:2}])[0].x==2);}}";
    package(&ev, lib, "");
    std::fs::write(ev.join("src/main.rs"), main).unwrap();
    let initial = run(RustCheck::Test, "cmp-veto", &ev).unwrap();
    assert!(initial.iter().any(|f| f.item.kind == RustKind::Test));
    let p = comparators("cmp-veto", &ev);
    assert_eq!(p.len(), 1);
    assert!(verify_native_choice("cmp-veto", &ev, &p, "comparator", &mut false).is_some());
    let old = dedup_orders("cmp-veto", &ev);
    assert_eq!(old.len(), 2);
    let mut withdrawn = false;
    assert!(verify_native_choice("cmp-veto", &ev, &old, "dedup-order", &mut withdrawn).is_none());
    assert!(withdrawn);
    assert!(
        undo_slip(
            "cmp-veto",
            &ev,
            &initial.iter().map(|f| f.item.clone()).collect::<Vec<_>>()
        )
        .is_none()
    );
    // Preserve extremum's previous compiler-finding semantics, but record a
    // veto for the newly appended comparator fallback.
    let invalid = RustChange {
        new: "missing".into(),
        ..p[0].clone()
    };
    let mut comparator_veto = false;
    withdrawn = false;
    assert_eq!(
        verify_native_choice_observed(
            "cmp-veto",
            &ev,
            &[invalid, p[0].clone()],
            "extremum",
            &mut withdrawn,
            &mut comparator_veto
        ),
        Some(p[0].clone())
    );
    assert!(!withdrawn);
    assert!(comparator_veto);
    for candidates in [
        vec![p[0].clone(); 9],
        vec![RustChange {
            new: "missing".into(),
            ..p[0].clone()
        }],
        vec![RustChange {
            old: "wrong".into(),
            ..p[0].clone()
        }],
    ] {
        withdrawn = false;
        assert!(
            verify_native_choice("cmp-veto", &ev, &candidates, "comparator", &mut withdrawn)
                .is_none()
        );
        assert!(withdrawn);
    }
}
#[test]
fn comparator_opaque_shapes() {
    let base = "struct Row{x:i32} fn f(mut v:Vec<Row>){v.sort_by(|a,b|a.x.cmp(&b.x));}";
    for s in [base.replace("i32","f32"),base.replace("fn f(","fn f<T>("),base.replace("a.x.cmp(&b.x)","a.x.cmp(&b.x).then_with(|| opaque())"),base.replace("struct Row{x:i32}","struct Row{x:i32} impl std::ops::Deref for Row{type Target=i32;fn deref(&self)->&i32{&self.x}}"),base.replace("struct Row{x:i32}","struct Row{x:i32} type Vec<T>=Other<T>;"),base.replace("a.x.cmp(&b.x)","{let a=b;a.x.cmp(&b.x)}")] {
        assert!(crate::comparator::proposals("s","src/lib.rs",&s).is_empty(),"{s}");
    }
}
