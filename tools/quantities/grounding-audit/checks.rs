// Authored diagnostic checks, no evaluation fixtures.
#[cfg(test)]
mod audit_checks {
    use super::*;
    #[test]
    fn legal_masks_and_first_tie_match_decoder_contract() {
        let model = Model::new("diagnostic zero-weight tie control".into());
        for depth in 0..6 {
            for n in 0..=16 {
                let ops = legal(0, n, depth);
                assert_eq!(ops.contains(&4), depth > 0);
                assert_eq!(model.choose(0, &[1,2], &ops).unwrap().0, 0);
                for h in [1,2] {
                    let args = legal(h,n,depth);
                    assert_eq!(args.len(), n+20+depth);
                    for &a in &args { assert!(a<n || (16..36).contains(&a) || (36..36+depth).contains(&a)); }
                    assert_eq!(model.choose(h,&[1,2],&args).unwrap().0,args[0]);
                }
            }
        }
    }
    #[test]
    fn oracle_context_changes_both_heads_without_changing_base() {
        let q="Alice has 7 apples and Bob has 4 apples.";
        let nums=numbers(q).unwrap();let base=base_features(q,&nums);
        let graph=Program{steps:vec![]};
        for left in [None,Some(0)] {
            assert_ne!(state_features(&base,&graph,Some(0),left),state_features(&base,&graph,Some(1),left));
        }
        assert_eq!(base,base_features(q,&nums));
    }
    #[test]
    fn forward_references_are_rejected() {
        let bad=Program{steps:vec![Step{op:Op::Add,left:Arg::Step(0),right:Arg::Input(0)}]};
        assert!(bad.validate().is_err());
    }
}
