//! Offline supervised fit. Raw rows are never model assets.
use glassmind_quantities::proposal::{
    FEATURES, Model, Program, arg_id, base_features, legal, numbers, op_id, state_features,
};
use serde::Deserialize;
use std::io::{self, BufRead};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    question: String,
    program: Program,
}
struct Event {
    head: usize,
    fs: Vec<usize>,
    actions: Vec<usize>,
    target: usize,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let dest = args.get(1).ok_or("model path")?;
    let provenance = args.get(2).ok_or("provenance")?;
    if std::path::Path::new(dest).exists() {
        return Err("refuse overwrite existing fit".into());
    }
    let mut rows = Vec::new();
    let mut counts = [0; 3];
    let mut max_steps = 0;
    for line in io::stdin().lock().lines() {
        let r: Row = serde_json::from_str(&line?)?;
        r.program.execute(&r.question)?;
        let nums = numbers(&r.question)?;
        let base = base_features(&r.question, &nums);
        let mut prefix = Program { steps: vec![] };
        let mut events = Vec::new();
        max_steps = max_steps.max(r.program.steps.len());
        for s in &r.program.steps {
            let depth = prefix.steps.len();
            let op = op_id(&s.op);
            let left = arg_id(&s.left);
            let right = arg_id(&s.right);
            for (head, target, o, l) in [
                (0, op, None, None),
                (1, left, Some(op), None),
                (2, right, Some(op), Some(left)),
            ] {
                let actions = legal(head, nums.len(), depth);
                if !actions.contains(&target) {
                    return Err("illegal training target".into());
                }
                events.push(Event {
                    head,
                    target,
                    actions,
                    fs: state_features(&base, &prefix, o, l),
                });
                *counts.get_mut(head).ok_or("invalid head")? += 1;
            }
            prefix.steps.push(s.clone());
        }
        if prefix.steps.len() < 6 {
            events.push(Event {
                head: 0,
                target: 4,
                actions: legal(0, nums.len(), prefix.steps.len()),
                fs: state_features(&base, &prefix, None, None),
            });
            *counts.first_mut().ok_or("missing count")? += 1;
        }
        rows.push(events);
    }
    if rows.len() != 4590 {
        return Err("expected all 4590 cleaned FIT rows".into());
    }
    let mut model = Model::new(provenance.clone());
    let mut losses = Vec::new();
    for epoch in 0..12 {
        let rate = 0.08 / (1.0 + 0.2 * epoch as f64);
        let mut loss = 0.0;
        let mut n = 0;
        for k in 0..rows.len() {
            for e in rows
                .get((k + epoch * 997) % rows.len())
                .ok_or("missing row")?
            {
                let ps = model.distribution(e.head, &e.fs, &e.actions)?;
                let scale = 1.0 / (e.fs.len() as f64).sqrt();
                for (&a, &p) in e.actions.iter().zip(&ps) {
                    if a == e.target {
                        loss -= p;
                    }
                    let delta = rate * scale * (f64::from(a == e.target) - p.exp());
                    for f in &e.fs {
                        *model
                            .weights
                            .get_mut(e.head)
                            .and_then(|w| w.get_mut(a * FEATURES + f))
                            .ok_or("missing weight")? += delta as f32;
                    }
                }
                n += 1;
            }
        }
        losses.push(loss / n as f64);
        eprintln!("epoch {} mean_nll {}", epoch, loss / n as f64);
    }
    model.validate()?;
    let bytes = serde_json::to_vec(&model)?;
    Model::from_bytes(&bytes)?;
    std::fs::write(dest, bytes)?;
    println!(
        "{}",
        serde_json::json!({"rows":rows.len(),"target_counts":counts,"mean_nll":losses,"max_steps":max_steps,"beam":1,"epochs":12})
    );
    Ok(())
}
