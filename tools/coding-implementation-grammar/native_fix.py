from pathlib import Path
p=Path('crates/world-rust/src/implementation.rs');s=p.read_text()
s=s.replace('    let mut passed = 0;','    if stderr.contains("could not compile") || stderr.contains("error:") {\n        return Err("native compiler error; candidate family not exhausted".into());\n    }\n    let mut passed = 0;')
s=s.replace('        let scratch = checks::scratch("implement");','''        let scratch = checks::scratch("implement");
        // Validate the nearest existing ancestor before any scratch writes.
        let live = source.root.canonicalize().map_err(|e| e.to_string())?;
        let mut ancestor = scratch.as_path();
        while !ancestor.exists() {
            ancestor = ancestor.parent().ok_or("scratch ancestor required")?;
        }
        let resolved = ancestor.canonicalize().map_err(|e| e.to_string())?;
        if !scratch.is_absolute()
            || scratch.components().any(|c| matches!(c, std::path::Component::ParentDir))
            || resolved.starts_with("/tmp") || resolved.starts_with(&live)
        {
            return Err("scratch must be persistent disk outside the source package".into());
        }''')
s=s.replace('            let mut passing = Vec::new();','''            // Prove that the existing test execution reaches this exact target.
            // This probe is never a candidate and consumes no test body features.
            let marker = format!("glassmind_target_executed_{}", std::process::id());
            let mut probe = plan.text.clone();
            probe.replace_range(plan.start..plan.end, &format!("::std::panic!({marker:?})"));
            fs::write(scratch.join("src/lib.rs"), &probe).map_err(|e| e.to_string())?;
            let reached = run_tests(&scratch, &source.root,
                started + Duration::from_secs(75), &evidence, "target-execution")?;
            let log = fs::read_to_string(evidence.join("target-execution.log")).map_err(|e| e.to_string())?;
            if reached.success || reached.failed == 0 || !log.contains(&marker) {
                return Err("target execution was not witnessed in original tests".into());
            }
            let mut probe_expected = original.clone();
            probe_expected.insert(PathBuf::from("src/lib.rs"), probe.into_bytes());
            unchanged_inputs(&scratch, &probe_expected)?;
            let mut passing = Vec::new();
            let mut passing_indices = Vec::new();''')
s=s.replace('                    passing.push(replacement);','                    passing.push(replacement);\n                    passing_indices.push(i);')
s=s.replace('            let [replacement] = passing.as_slice() else {','''            fs::write(evidence.join("choice.json"), format!(
                "{{\\"candidate_count\\":{},\\"passing_indices\\":{:?},\\"target_execution\\":true}}\\n",
                plan.candidates.len(), passing_indices)).map_err(|e| e.to_string())?;
            let [replacement] = passing.as_slice() else {''')
p.write_text(s)
