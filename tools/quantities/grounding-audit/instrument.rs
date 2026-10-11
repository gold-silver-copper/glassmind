// Included inside an exact copy of original proposal.rs: accesses original private choose.
pub fn audit_oracles(model: &Model, question: &str, gold: &Program) -> Result<serde_json::Value> {
    use serde_json::json;
    let nums = numbers(question)?;
    gold.execute(question)?;
    let base = base_features(question, &nums); // question and native numeric spans ONLY
    let mut prefix = Program { steps: vec![] };
    let mut events = Vec::new();
    let mut perturb_checks = 0;
    for s in &gold.steps {
        let depth = prefix.steps.len();
        let op = op_id(&s.op);
        let left = arg_id(&s.left);
        let right = arg_id(&s.right);
        for (head, target, o, l) in [(0, op, None, None), (1, left, Some(op), None), (2, right, Some(op), Some(left))] {
            let actions = legal(head, nums.len(), depth);
            assert!(actions.contains(&target));
            let fs = state_features(&base, &prefix, o, l);
            if head > 0 {
                let perturbed = state_features(&base, &prefix, Some((op + 1) % 4), l);
                assert_ne!(fs, perturbed);
                perturb_checks += 1;
            }
            let (pred, logp) = model.choose(head, &fs, &actions)?;
            assert!(actions.contains(&pred));
            events.push(json!({"head":head,"depth":depth,"op":op,"target":target,"prediction":pred,"logp":logp,"legal":actions}));
        }
        prefix.steps.push(s.clone());
    }
    // Exactly original fit STOP supervision; horizon-six is not a model decision.
    if gold.steps.len() < 6 {
        let actions = legal(0, nums.len(), prefix.steps.len());
        assert!(actions.contains(&4));
        let (pred, logp) = model.choose(0, &state_features(&base, &prefix, None, None), &actions)?;
        events.push(json!({"head":0,"depth":prefix.steps.len(),"op":4,"target":4,"prediction":pred,"logp":logp,"legal":actions}));
    }
    let mut graph = Program { steps: vec![] };
    let mut oracle_events = Vec::new();
    for s in &gold.steps {
        let depth = graph.steps.len();
        let op = op_id(&s.op); // EXPLICIT ORACLE: full annotation operation plan
        let (left, _) = model.choose(1, &state_features(&base, &graph, Some(op), None), &legal(1, nums.len(), depth))?;
        let (right, _) = model.choose(2, &state_features(&base, &graph, Some(op), Some(left)), &legal(2, nums.len(), depth))?;
        for (head, pred, target) in [(1,left,arg_id(&s.left)),(2,right,arg_id(&s.right))] {
            assert!(legal(head, nums.len(), depth).contains(&pred));
            oracle_events.push(json!({"head":head,"depth":depth,"op":op,"target":target,"prediction":pred}));
        }
        graph.steps.push(Step { op: operation(op)?, left: argument(left), right: argument(right) });
    }
    graph.validate()?; // asserts no illegal forward references; arithmetic may still refuse
    let oracle = match graph.execute(question) {
        Ok((_,value)) => json!({"status":"proposed","program":graph,"value":value.to_string()}),
        Err(reason) => json!({"status":"refusal","program":graph,"reason":reason}),
    };
    Ok(json!({"mode":"explicit_train_annotation_oracles","input_count":nums.len(),"teacher":events,"oracle_events":oracle_events,"oracle":oracle,"forced_stop":gold.steps.len()<6,"forced_horizon":gold.steps.len()==6,"perturb_checks":perturb_checks}))
}
