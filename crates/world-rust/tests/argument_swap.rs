//! Frozen authored structural contracts. Native main-binary replay is retained
//! under tools/coding-argument-swap; these checks exercise refusal boundaries.
#[cfg(test)]
mod tests {
    use glassmind_world_rust::checks;

    fn proposals(code: &str) -> Vec<checks::RustChange> {
        let root = checks::scratch("argument-structural");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), code).unwrap();
        let changes = checks::slips("s", &root, &[]);
        assert_eq!(
            std::fs::read_to_string(root.join("src/lib.rs")).unwrap(),
            code
        );
        std::fs::remove_dir_all(root).unwrap();
        changes
            .into_iter()
            .filter(|c| c.old.starts_with("pair(") || c.old.starts_with("first("))
            .collect()
    }
    #[test]
    fn frozen_positive_spans_and_negative_boundaries() {
        let contracts: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tools/coding-argument-swap/contracts.json"
        ))
        .unwrap();
        for c in contracts.as_array().unwrap() {
            let code = c["code"].as_str().unwrap();
            let changes = proposals(code);
            if let Some(old) = c["old"].as_str() {
                if old.starts_with("ratio(") {
                    continue;
                }
                assert_eq!(changes.len(), 1, "{}", c["name"]);
                assert_eq!(changes[0].old, old, "{}", c["name"]);
                assert_eq!(changes[0].new, c["new"].as_str().unwrap(), "{}", c["name"]);
            } else if !matches!(c["name"].as_str(), Some("two_passing" | "wrong_rollback")) {
                assert!(changes.is_empty(), "{}: {changes:?}", c["name"]);
            }
        }
    }
    #[test]
    fn opaque_syntax_never_exposes_calls() {
        let helper = "fn pair(a:u8,b:u8)->(u8,u8){(a,b)}\n";
        for body in [
            "#[test] fn x(){pair(2u8,1u8);}",
            "#[cfg(test)] mod t {fn x(){pair(2u8,1u8);}}",
            "fn x(){let _=\"pair(2u8,1u8)\"; /* pair(2u8,1u8) */}",
            "fn x(){let _: [u8; {pair(2u8,1u8);2}];}",
            "fn x(){let _=other!(pair(2u8,1u8));}",
            "fn x(){let _=format!(\"{captured}\",pair(2u8,1u8));}",
            "fn x(){let _=format!(\"{:.*}\",2,pair(2u8,1u8));}",
            "fn x(){let _=format!(\"{}\",pair(2u8,1u8),); let _=unknown!();}",
            "fn x(){fn pair(a:u8,b:u8)->(u8,u8){(a,b)} pair(2u8,1u8);}",
            "fn x(pair:fn(u8,u8)->(u8,u8)){pair(2u8,1u8);}",
            "fn x(a:u8,b:u8){ #[cfg(test)] {pair(b,a);} }",
            "fn x(a:u8,b:u8){ pair::<u8>(b,a); }",
            "fn x(a:u8,b:u8){ a.pair(b,a); }",
            "fn x(a:u8,b:u8){ unknown(b,a); }",
        ] {
            assert!(proposals(&format!("{helper}{body}")).is_empty(), "{body}");
        }
        let test_macro = "#[cfg(test)] #[macro_use] mod definitions { macro_rules! format { ($($t:tt)*) => {String::new()} } } fn first(a:u8,b:u8)->u8{a} fn x(a:u8,b:u8)->String{format!(\"{}\",first(b,a))}";
        assert!(proposals(test_macro).is_empty());
        assert!(proposals("fn x(){include!(\"generated.rs\");} fn first(a:u8,b:u8)->u8{a} fn y(a:u8,b:u8)->String{format!(\"{}\",first(b,a))}").is_empty());
        for attr in ["#[external::inject]", "#[cfg_attr(test, external::inject)]"] {
            assert!(
                proposals(&format!(
                    "{helper}{attr} fn unrelated(){{}} fn x(a:u8,b:u8){{pair(b,a);}} "
                ))
                .is_empty()
            );
        }
        for prefix in [
            "use std::format;",
            "use other::*;",
            "#[macro_use] extern crate other;",
            "type u8=u16;",
            "macro_rules! format {($($t:tt)*)=>{0}}",
        ] {
            assert!(
                proposals(&format!("{prefix}{helper}fn x(a:u8,b:u8){{pair(b,a);}} ")).is_empty(),
                "{prefix}"
            );
        }
        assert!(
            proposals("fn pair<T>(a:T,b:T)->(T,T){(a,b)} fn x(a:u8,b:u8){pair(b,a);}").is_empty()
        );
        assert!(proposals("fn pair(a:u8,b:u16){} fn x(a:u8,b:u16){pair(a,b);}").is_empty());
        assert!(proposals(&format!("{helper}fn x(){{pair(2,1);}}")).is_empty());
    }
    #[test]
    fn all_prior_families_keep_global_order() {
        let root = checks::scratch("argument-order");
        std::fs::create_dir_all(root.join("src")).unwrap();
        for (file, code) in [
            (
                "lib.rs",
                "mod early; mod late; fn pair(a:u32,b:u32)->(u32,u32){(a,b)} fn call(a:u32,b:u32){pair(b,a);} fn ceil(a:u32)->u32{a/3} fn add(a:u32)->u32{a+1}",
            ),
            ("early.rs", "fn scale(a:u32)->u32{a*3}"),
            ("late.rs", "fn sub(a:u32)->u32{a-1}"),
        ] {
            std::fs::write(root.join("src").join(file), code).unwrap();
        }
        let changes = checks::slips("s", &root, &[]);
        let actual: Vec<_> = changes
            .iter()
            .map(|c| (c.file.as_str(), c.old.as_str()))
            .collect();
        assert_eq!(
            actual,
            [
                ("src/late.rs", "-"),
                ("src/lib.rs", "+"),
                ("src/early.rs", "*"),
                ("src/lib.rs", "/"),
                ("src/lib.rs", "a/3"),
                ("src/lib.rs", "pair(b,a)")
            ]
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
