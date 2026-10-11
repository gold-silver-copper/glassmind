#[cfg(test)]
mod contracts {
    use super::*;
    fn step(op:Op,l:Arg,r:Arg)->Step {Step{op,left:l,right:r}}
    #[test]
    fn formal_targets() {
        let empty=Program{steps:vec![]};
        for op in [Op::Add,Op::Multiply] {
            let s=step(op,Arg::Input(0),Arg::Input(1));
            assert_eq!(pairs(&empty,&s,2).unwrap(),vec![(0,1),(1,0)]);
        }
        for op in [Op::Subtract,Op::Divide] {
            let s=step(op,Arg::Input(0),Arg::Input(1));
            assert_eq!(pairs(&empty,&s,2).unwrap(),vec![(0,1)]);
        }
        // Equal numeric values do not enter the target constructor.
        let nums=numbers("5 and 5").unwrap();assert_eq!(nums[0].value,nums[1].value);
        assert_ne!(atom(&Arg::Input(0),&[],2).unwrap(),atom(&Arg::Input(1),&[],2).unwrap());
        let s=step(Op::Add,Arg::Input(0),Arg::Input(0));
        assert_eq!(pairs(&empty,&s,2).unwrap(),vec![(0,0)]);
        let prior=Program{steps:vec![step(Op::Add,Arg::Input(0),Arg::Input(1)),step(Op::Add,Arg::Input(1),Arg::Input(0))]};
        let s=step(Op::Subtract,Arg::Step(0),Arg::Input(0));
        assert_eq!(pairs(&prior,&s,2).unwrap(),vec![(36,0),(37,0)]);
        assert_eq!("0.250".parse::<Rational>().unwrap().to_string(),"1/4");
        assert_eq!("2/8".parse::<Rational>().unwrap().to_string(),"1/4");
        assert_eq!(atom(&Arg::Constant(17),&[],0).unwrap(),Tree::Constant("1/4".into()));
        assert_ne!(atom(&Arg::Constant(1),&[],1).unwrap(),atom(&Arg::Input(0),&[],1).unwrap());
        let a=Tree::Input(0);let b=Tree::Input(1);let c=Tree::Input(2);
        assert_ne!(node(0,node(0,a.clone(),b.clone()),c.clone()),node(0,a.clone(),node(0,b.clone(),c)));
        assert_ne!(node(1,node(0,a.clone(),b.clone()),b),a);
        assert!(atom(&Arg::Input(2),&[],2).is_err());
        assert!(atom(&Arg::Step(0),&[],2).is_err());
        assert!(trees(&Program{steps:vec![step(Op::Add,Arg::Step(0),Arg::Input(0))]},2).is_err());
    }
    fn lse(xs:&[f64])->f64 {let m=xs.iter().copied().fold(f64::NEG_INFINITY,f64::max);m+xs.iter().map(|x|(x-m).exp()).sum::<f64>().ln()}
    #[test]
    fn gradient_finite_difference_and_singleton() {
        for target in [vec![0],vec![0,2],vec![0,1,2]] {
            let s=vec![-0.7,0.4,1.1];let z=lse(&s);let logs:Vec<_>=s.iter().map(|x|x-z).collect();
            let (lm,g)=mass_gradient(&logs,&[0,1,2],&target).unwrap();
            let objective=|v:&[f64]| {let selected:Vec<_>=target.iter().map(|i|v[*i]).collect();lse(&selected)-lse(v)};
            assert!((lm-objective(&s)).abs()<1e-12);
            for i in 0..3 {let mut plus=s.clone();let mut minus=s.clone();plus[i]+=1e-6;minus[i]-=1e-6;
                assert!((g[i]-(objective(&plus)-objective(&minus))/2e-6).abs()<1e-8);}
            if target.len()==1 {for i in 0..3 {assert!((g[i]-(f64::from(i==target[0])-logs[i].exp())).abs()<1e-15);}}
        }
        // Weighted conditional-right objective, two contexts with unequal set sizes.
        let contexts=[(vec![-0.2,0.3,0.7],vec![0,2]),(vec![0.8,-0.1,0.4],vec![1])];
        let f=|shift:f64| contexts.iter().map(|(s,t)|{let mut v=s.clone();v[1]+=shift;let vs:Vec<_>=t.iter().map(|i|v[*i]).collect();(lse(&vs)-lse(&v))/2.0}).sum::<f64>();
        let grad: f64=contexts.iter().map(|(s,t)|{let z=lse(s);let ps:Vec<_>=s.iter().map(|x|x-z).collect();mass_gradient(&ps,&[0,1,2],t).unwrap().1[1]/2.0}).sum();
        assert!((grad-(f(1e-6)-f(-1e-6))/2e-6).abs()<1e-8);
    }
    #[test]
    fn rational_transforms_definedness() {
        // Independent direct evaluation versus native Program execution, distinct
        // rational assignments including negative/zero and division failure.
        for (a,b) in [("2.5","-3"),("-1.25","4"),("0","-2"),("3","0")] {
            let q=format!("{a} and {b}");let x=a.parse::<Rational>().unwrap();let y=b.parse::<Rational>().unwrap();
            for (op,id) in [(Op::Add,0),(Op::Multiply,2)] {
                let p=Program{steps:vec![step(op.clone(),Arg::Input(0),Arg::Input(1))]};
                let alt=Program{steps:vec![step(op,Arg::Input(1),Arg::Input(0))]};
                let expected=if id==0 {x.checked_add(y)} else {x.checked_mul(y)};
                assert_eq!(p.execute(&q).map(|v|v.1),expected);assert_eq!(alt.execute(&q).map(|v|v.1),expected);
            }
            let p=Program{steps:vec![step(Op::Divide,Arg::Input(0),Arg::Input(1)),step(Op::Add,Arg::Step(0),Arg::Input(0))]};
            let alt=Program{steps:vec![step(Op::Divide,Arg::Input(0),Arg::Input(1)),step(Op::Add,Arg::Input(0),Arg::Step(0))]};
            let expected=x.checked_div(y).and_then(|v|v.checked_add(x));
            assert_eq!(p.execute(&q).map(|v|v.1),expected);assert_eq!(alt.execute(&q).map(|v|v.1),expected);
        }
    }
    #[test]
    fn original_ties_masks_and_singleton_update() {
        let mut m=Model::new("test".into());let actions=legal(1,2,0);let fs=vec![0,5,72];
        assert_eq!(m.choose(1,&fs,&actions).unwrap().0,0);
        let ps=m.distribution(1,&fs,&actions).unwrap();let rate=0.08/(1.0+0.2*12.0);
        let mut expected=m.weights[1].clone();
        for (a,p) in actions.iter().zip(ps) {let delta=rate/(fs.len() as f64).sqrt()*(f64::from(*a==1)-p.exp());for f in &fs {expected[a*FEATURES+f]+=delta as f32;}}
        update(&mut m,&[Event{head:1,fs,actions,targets:vec![1],weight:1.0}],rate).unwrap();
        assert_eq!(m.weights[1],expected);
    }
}
