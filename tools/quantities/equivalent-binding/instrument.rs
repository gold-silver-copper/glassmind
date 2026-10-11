// Included after byte-identical original proposal.rs. Offline supervision only.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Tree { Input(u8), Constant(String), Node(usize, Box<Tree>, Box<Tree>) }
fn node(op: usize, mut l: Tree, mut r: Tree) -> Tree {
    if (op == 0 || op == 2) && r < l { std::mem::swap(&mut l, &mut r); }
    Tree::Node(op, Box::new(l), Box::new(r))
}
fn atom(a: &Arg, prior: &[Tree], n: usize) -> Result<Tree> {
    match a {
        Arg::Input(i) if usize::from(*i) < n => Ok(Tree::Input(*i)),
        Arg::Constant(i) => Ok(Tree::Constant(CONSTANTS.get(usize::from(*i)).ok_or("constant")?.parse::<Rational>()?.to_string())),
        Arg::Step(i) => prior.get(usize::from(*i)).cloned().ok_or("forward/missing reference".into()),
        _ => Err("missing input".into()),
    }
}
fn trees(prefix: &Program, n: usize) -> Result<Vec<Tree>> {
    if prefix.steps.len() > 6 || n > 16 { return Err("bounds".into()); }
    let mut prior = Vec::new();
    for s in &prefix.steps { prior.push(node(op_id(&s.op), atom(&s.left,&prior,n)?, atom(&s.right,&prior,n)?)); }
    Ok(prior)
}
fn pairs(prefix: &Program, s: &Step, n: usize) -> Result<Vec<(usize,usize)>> {
    if prefix.steps.len() >= 6 { return Err("horizon".into()); }
    let prior = trees(prefix,n)?;
    let target = node(op_id(&s.op),atom(&s.left,&prior,n)?,atom(&s.right,&prior,n)?);
    let actions = legal(1,n,prefix.steps.len());
    let atoms = actions.iter().map(|a| atom(&argument(*a),&prior,n)).collect::<Result<Vec<_>>>()?;
    let mut out = Vec::new();
    for (i,l) in actions.iter().enumerate() { for (j,r) in actions.iter().enumerate() {
        let a = atoms.get(i).ok_or("atom")?; let b = atoms.get(j).ok_or("atom")?;
        if node(op_id(&s.op),a.clone(),b.clone()) == target { out.push((*l,*r)); }
    }}
    if !out.contains(&(arg_id(&s.left),arg_id(&s.right))) { return Err("missing gold pair".into()); }
    Ok(out)
}
// Returns log target mass and derivative of log mass w.r.t. each score.
fn mass_gradient(logps: &[f64], actions: &[usize], targets: &[usize]) -> Result<(f64,Vec<f64>)> {
    if targets.is_empty() || logps.len()!=actions.len() || targets.iter().any(|t|!actions.contains(t)) { return Err("invalid targets".into()); }
    let max = actions.iter().zip(logps).filter(|(a,_)|targets.contains(a)).map(|(_,p)|*p).fold(f64::NEG_INFINITY,f64::max);
    let z = actions.iter().zip(logps).filter(|(a,_)|targets.contains(a)).map(|(_,p)|(*p-max).exp()).sum::<f64>();
    let logmass = max+z.ln();
    let grad = actions.iter().zip(logps).map(|(a,p)| if targets.contains(a) {(*p-logmass).exp()-p.exp()} else {-p.exp()}).collect();
    Ok((logmass,grad))
}
struct Event { head: usize, fs: Vec<usize>, actions: Vec<usize>, targets: Vec<usize>, weight: f64 }
struct TrainStep { left: Event, right: Vec<Event> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitRow { question: String, program: Program }
fn update(model: &mut Model, events: &[Event], rate: f64) -> Result<f64> {
    // Calculate every context at the same weights, then apply their weighted sum.
    let mut pending = Vec::new(); let mut loss=0.0;
    for e in events {
        let ps=model.distribution(e.head,&e.fs,&e.actions)?;
        let (lm,g)=mass_gradient(&ps,&e.actions,&e.targets)?;
        loss-=e.weight*lm;
        pending.push(g);
    }
    // For singleton this is the exact original f32 SGD update. For multiple right
    // contexts accumulate f64 deltas by coordinate and cast once, independently of order.
    if events.len()==1 {
        let e=events.first().ok_or("event")?; let g=pending.first().ok_or("gradient")?;
        for (a,d) in e.actions.iter().zip(g) {
            let delta=rate*(1.0/(e.fs.len() as f64).sqrt())*e.weight*d;
            for f in &e.fs { *model.weights.get_mut(e.head).and_then(|w|w.get_mut(a*FEATURES+f)).ok_or("weight")? += delta as f32; }
        }
    } else {
        let head=events.first().ok_or("event")?.head;
        let first=events.first().ok_or("event")?;
        let mut deltas=vec![0.0_f64;FEATURES];let mut seen=vec![false;FEATURES];let mut touched=Vec::new();
        for (j,a) in first.actions.iter().enumerate() {
            for (e,g) in events.iter().zip(&pending) {
                if e.head!=head || e.actions!=first.actions { return Err("mixed heads/actions".into()); }
                let delta=rate*(1.0/(e.fs.len() as f64).sqrt())*e.weight*g.get(j).ok_or("gradient")?;
                for f in &e.fs {
                    let mark=seen.get_mut(*f).ok_or("feature")?;
                    if !*mark {touched.push(*f);*mark=true;}
                    *deltas.get_mut(*f).ok_or("feature")?+=delta;
                }
            }
            let w=model.weights.get_mut(head).ok_or("head")?;
            for f in touched.drain(..) {
                *w.get_mut(a*FEATURES+f).ok_or("weight")?+=*deltas.get(f).ok_or("delta")? as f32;
                *deltas.get_mut(f).ok_or("delta")?=0.0;*seen.get_mut(f).ok_or("feature")?=false;
            }
        }
    }
    Ok(loss)
}
pub fn fit(model: &mut Model, input: Vec<FitRow>, equiv: bool, dest: &str) -> Result<serde_json::Value> {
    if input.len()!=4590 || std::path::Path::new(dest).exists() { return Err("rows or existing destination".into()); }
    let original_op=model.weights.first().ok_or("OP")?.clone();
    let mut rows=Vec::new(); let mut expanded=0; let mut count=0; let mut contexts=0;
    for r in input {
        r.program.execute(&r.question)?;
        let ns=numbers(&r.question)?; let base=base_features(&r.question,&ns);
        let mut prefix=Program{steps:vec![]}; let mut row=Vec::new();
        for s in r.program.steps {
            let op=op_id(&s.op); let actions=legal(1,ns.len(),prefix.steps.len());
            let valid=if equiv {pairs(&prefix,&s,ns.len())?} else {vec![(arg_id(&s.left),arg_id(&s.right))]};
            let ls:Vec<_>=valid.iter().map(|p|p.0).collect::<BTreeSet<_>>().into_iter().collect();
            expanded+=usize::from(valid.len()>1);count+=1;contexts+=ls.len();
            let left=Event{head:1,fs:state_features(&base,&prefix,Some(op),None),actions:actions.clone(),targets:ls.clone(),weight:1.0};
            let right=ls.iter().map(|l|Event{head:2,fs:state_features(&base,&prefix,Some(op),Some(*l)),actions:actions.clone(),targets:valid.iter().filter(|p|p.0==*l).map(|p|p.1).collect(),weight:1.0/ls.len() as f64}).collect();
            row.push(TrainStep{left,right});prefix.steps.push(s);
        } rows.push(row);
    }
    let mut epochs=Vec::new();
    for epoch in 12..18 {
        let rate=0.08/(1.0+0.2*epoch as f64);let mut loss=0.0;
        for k in 0..rows.len() {
            for step in rows.get((k+epoch*997)%rows.len()).ok_or("row")? {
                loss+=update(model,std::slice::from_ref(&step.left),rate)?;
                loss+=update(model,&step.right,rate)?;
            }
        }
        eprintln!("epoch {epoch} rate {rate} nll {}",loss/(2*count) as f64);
        epochs.push(serde_json::json!({"epoch":epoch,"rate":rate,"nll":loss/(2*count) as f64}));
    }
    if model.weights.first().ok_or("OP")?.iter().zip(&original_op).any(|(a,b)|a.to_bits()!=b.to_bits()) { return Err("OP changed".into()); }
    model.validate()?;
    let bytes=serde_json::to_vec(model).map_err(|e|e.to_string())?;
    if bytes.len()>16*1024*1024 { return Err("model cap".into()); }
    Model::from_bytes(&bytes)?;
    std::fs::write(dest,&bytes).map_err(|e|e.to_string())?;
    Ok(serde_json::json!({"arm":if equiv {"B"} else {"A"},"rows":rows.len(),"steps":count,"expanded_steps":expanded,"right_contexts":contexts,"epochs":epochs,"op_bytes_unchanged":true,"bytes":bytes.len()}))
}
pub fn diagnostics(model: &Model, q: &str, gold: &Program) -> Result<serde_json::Value> {
    gold.execute(q)?;
    let nums=numbers(q)?;let base=base_features(q,&nums);let mut prefix=Program{steps:vec![]};let mut teacher=Vec::new();
    for s in &gold.steps {
        let op=op_id(&s.op);let valid=pairs(&prefix,s,nums.len())?;let actions=legal(1,nums.len(),prefix.steps.len());
        let l=model.choose(1,&state_features(&base,&prefix,Some(op),None),&actions)?.0;
        let r=model.choose(2,&state_features(&base,&prefix,Some(op),Some(arg_id(&s.left))),&actions)?.0;
        let paired_r=model.choose(2,&state_features(&base,&prefix,Some(op),Some(l)),&actions)?.0;
        let o=model.choose(0,&state_features(&base,&prefix,None,None),&legal(0,nums.len(),prefix.steps.len()))?.0;
        teacher.push(serde_json::json!({"depth":prefix.steps.len(),"op":op,"op_pred":o,"left":l,"right_gold_left":r,"right_pred_left":paired_r,"gold_left":arg_id(&s.left),"gold_right":arg_id(&s.right),"valid_pairs":valid,"left_exact":l==arg_id(&s.left),"right_exact":r==arg_id(&s.right),"left_equiv":valid.iter().any(|p|p.0==l),"right_equiv":valid.contains(&(arg_id(&s.left),r)),"pair_equiv":valid.contains(&(l,paired_r))}));
        prefix.steps.push(s.clone());
    }
    let mut graph=Program{steps:vec![]};
    for s in &gold.steps {
        let op=op_id(&s.op);let actions=legal(1,nums.len(),graph.steps.len());
        let l=model.choose(1,&state_features(&base,&graph,Some(op),None),&actions)?.0;
        let r=model.choose(2,&state_features(&base,&graph,Some(op),Some(l)),&actions)?.0;
        graph.steps.push(Step{op:operation(op)?,left:argument(l),right:argument(r)});
    }
    let oracle=match graph.execute(q) { Ok((_,v))=>serde_json::json!({"status":"proposed","program":graph,"value":v.to_string()}),Err(reason)=>serde_json::json!({"status":"refusal","program":graph,"reason":reason}) };
    Ok(serde_json::json!({"mode":"gold_plan_and_length_predicted_args_prefix_forced_termination","teacher":teacher,"oracle":oracle,"forced_stop":gold.steps.len()<6,"forced_horizon":gold.steps.len()==6}))
}
