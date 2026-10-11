#[cfg(test)]
mod tests {
    use glassmind_world_rust::checks;

    fn swaps(code: &str) -> Vec<checks::RustChange> {
        let root = checks::scratch("argument-parent-guards");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), code).unwrap();
        let result = checks::slips("s", &root, &[])
            .into_iter()
            .filter(|c| c.old.starts_with("pair("))
            .collect();
        std::fs::remove_dir_all(root).unwrap();
        result
    }

    #[test]
    fn standard_format_beside_inert_inline_tests_preserves_call_spans() {
        for macro_name in ["format", "::std::format"] {
            let code = format!(
                "fn pair(a:u32,b:u32)->u32{{a}} fn value(a:u32,b:u32)->String{{{macro_name}!(\"value: {{}}\",pair(b,a))}} \
                #[cfg(test)] mod tests{{use super::*; #[test] fn native(){{assert_eq!(value(4,1),\"value: 4\");}}}}"
            );
            let proposals = swaps(&code);
            assert_eq!(proposals.len(), 1);
            assert_eq!(proposals[0].old, "pair(b,a)");
            assert_eq!(proposals[0].new, "pair(a,b)");
        }
    }

    #[test]
    fn equal_slots_and_one_independent_operand_type_anchor_integer_literals() {
        let helper = "fn pair(a:u32,b:u32)->u32{a}";
        let proposals = swaps(&format!(
            "{helper} fn value(a:u32,b:u32)->u32{{pair(b,a)+pair(4,a)}}"
        ));
        assert_eq!(proposals.len(), 2);
        assert_eq!(proposals[1].new, "pair(a,4)");
        for call in ["pair(4,2)", "pair(4u16,a)", "pair(other(),a)"] {
            assert!(swaps(&format!("{helper} fn value(a:u32){{{call};}}")).is_empty());
        }
    }

    #[test]
    fn bindings_hidden_inside_parsed_macro_expressions_remain_opaque() {
        let helper =
            "fn pair(a:u32,b:u32)->u32{a} fn value(a:u32,b:u32)->String{format!(\"{}\",pair(b,a))}";
        for other in [
            "#[cfg(test)] mod tests {use external::assert_eq; #[test] fn t(){assert_eq!(1,1);}}",
            "#[cfg(test)] mod tests {use super::*; #[test] fn t(){assert_eq!({#[macro_export] macro_rules! format {($($x:tt)*)=>{0}} 1},1);}}",
            "fn other(){let _=format!(\"{}\",{#[macro_export] macro_rules! format {($($x:tt)*)=>{0}} 1});}",
            "#[cfg(test)] mod tests {use super::*; #[test] fn t(){assert_eq!(opaque!(),1);}}",
            "#[cfg(test)] #[macro_use] mod tests {macro_rules! format {($($x:tt)*)=>{0}}}",
        ] {
            assert!(swaps(&format!("{helper}{other}")).is_empty(), "{other}");
        }
    }
}
