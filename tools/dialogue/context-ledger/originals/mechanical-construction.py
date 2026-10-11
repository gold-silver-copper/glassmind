"""Recorded mechanical construction from the preserved authorized native source."""
from pathlib import Path
p=Path(__file__).parent/'native/context_ledger_experiment.rs'
s=p.read_text()
a=s.index('fn source_context(');b=s.index('#[derive(Serialize, Deserialize, Clone)]\nstruct Span',a)
s=s[:a]+'''fn mask(text: &str, values: &[String]) -> String {
    let mut out = text.to_lowercase();
    let mut values: Vec<_> = values.iter().map(|v| v.to_lowercase()).filter(|v| !v.is_empty()).collect();
    values.sort_by(|a,b| b.len().cmp(&a.len()).then(a.cmp(b)));
    values.dedup();
    for v in values { out = out.replace(&v, " <entity> "); }
    out
}
fn state_context(prior: Option<&Snapshot>) -> (Vec<String>, Vec<String>) {
    let mut fs = Vec::new();
    let mut values = Vec::new();
    if let Some(b) = prior {
        for a in &b.active_services { fs.push(format!("active={a}")); }
        for f in &b.frames {
            fs.push(format!("intent={}:{}",f.service,f.state.active_intent));
            for (slot, vs) in &f.state.slot_values {
                fs.push(format!("filled={}:{}",f.service,slot));
                values.extend(vs.iter().cloned());
            }
        }
    }
    (fs,values)
}
fn source_context(text: &str, s: usize, e: usize, user: &str, prior: Option<&Snapshot>, values: &[String]) -> Vec<String> {
    let (state,mut masked_values)=state_context(prior);
    masked_values.extend_from_slice(values);
    let left=mask(text.get(..s).unwrap_or(""),&masked_values);
    let right=mask(text.get(e..).unwrap_or(""),&masked_values);
    let before=tokens(&left); let after=tokens(&right);
    let mut xs=vec!["bias".into(),format!("len={}",tokens(text.get(s..e).unwrap_or("")).len().min(16))];
    for (i,t) in before.iter().rev().take(3).enumerate() {xs.push(format!("left{i}={}",t.text));}
    for (i,t) in after.iter().take(3).enumerate() {xs.push(format!("right{i}={}",t.text));}
    for t in before.iter().chain(&after) {xs.push(format!("ctx={}",t.text));}
    // Over-limit preceding USER text supplies no lexical features, but is retained in the trace.
    if user.len()<=4096 && tokens(user).len()<=128 {
        let ws=tokens(&mask(user,&masked_values));
        for t in &ws {xs.push(format!("previous_user={}",t.text));}
        for pair in ws.windows(2) {if let [a,b]=pair {xs.push(format!("previous_bigram={}_{}",a.text,b.text));}}
    }
    xs.extend(state);
    xs
}
''' +s[b:]
s=s.replace('self.version != 1','self.version != 2').replace('self.roles.iter().any(|r| !allowed.contains(r))','self.roles.iter().any(|r| r != "NONE" && !allowed.contains(r))\n            || self.roles.first().map(String::as_str) != Some("NONE")\n            || self.boundary.labels != 3 || !self.boundary.bio || self.role.bio || self.rank.bio || self.rank.labels != 1')
a=s.index('fn extract(');b=s.index('#[derive(Clone, Serialize, Deserialize)]\nstruct Candidate',a)
s=s[:a]+'''#[derive(Clone, Serialize, Deserialize)]
struct RawSpan { start: usize, end: usize, boundary_margin: f32 }
fn extract(boundary: &Linear, text: &str) -> Vec<RawSpan> {
    let ts = tokens(text);
    if text.len() > 4096 || ts.len() > 128 {return vec![];}
    let mut labels=Vec::new(); let mut margins=Vec::new(); let mut prev=0;
    for i in 0..ts.len() {
        let mut f=token_features(&ts,i); f.push(hash(&format!("prev={prev}")));
        let (l,m)=boundary.best(&f,prev);labels.push(l);margins.push(m);prev=l;
    }
    let mut out=Vec::new();let mut i=0;
    while i<ts.len() {
        if labels.get(i)!=Some(&1) {i+=1;continue;}
        let first=i;i+=1;while labels.get(i)==Some(&2) {i+=1;}
        if i-first>16 {continue;}
        if let (Some(a),Some(b))=(ts.get(first),ts.get(i-1)) {
            out.push(RawSpan{start:a.s,end:b.e,boundary_margin:margins.get(first..i).unwrap_or(&[]).iter().copied().fold(f32::INFINITY,f32::min)});
        }
    }
    out
}
#[derive(Clone, Serialize, Deserialize)]
struct SourceRow {
    id: String, cal: bool, turn: usize, raw: RawSpan, source: String,
    previous_user: String, context: Vec<String>, features: Vec<usize>,
    exact_roles: Vec<String>, label: String,
}
fn label_for(exact: &[String], roles: &[String]) -> String {
    if let [r]=exact && roles.contains(r) {r.clone()} else {"NONE".into()}
}
// Raw mask history carries values only; NONE predictions never enter Entity/Ledger.
#[derive(Default)]
struct MaskHistory { values: VecDeque<(usize,String)>, systems: usize }
impl MaskHistory {
    fn push(&mut self, values: Vec<String>) {
        self.systems+=1;
        self.values.extend(values.into_iter().map(|v|(self.systems,v)));
        while self.values.len()>128 || self.values.front().is_some_and(|(i,_)|self.systems.saturating_sub(*i)>=32) {self.values.pop_front();}
    }
    fn values(&self)->Vec<String>{self.values.iter().map(|(_,v)|v.clone()).collect()}
}
fn source_rows(data: &Dataset, boundary: &Linear)->Vec<SourceRow> {
    let mut rows=Vec::new();
    for d in &data.dialogues {
        let mut prior=None;let mut user="";let mut history=MaskHistory::default();
        for (i,t) in d.turns.iter().enumerate() {
            if matches!(t.input.speaker,glassmind_dialogue::tracker::Speaker::User) {
                user=&t.input.utterance;prior=t.baseline.as_ref();continue;
            }
            let spans=extract(boundary,&t.input.utterance);
            let current: Vec<_>=spans.iter().filter_map(|s|t.input.utterance.get(s.start..s.end).map(str::to_owned)).collect();
            let mut values=history.values();values.extend(current.iter().cloned());
            for raw in spans {
                let exact_roles: Vec<_>=t.spans.iter().filter(|s|s.start==raw.start && s.end==raw.end).map(|s|s.role.clone()).collect::<BTreeSet<_>>().into_iter().collect();
                let context=source_context(&t.input.utterance,raw.start,raw.end,user,prior,&values);
                rows.push(SourceRow{id:d.id.clone(),cal:d.cal,turn:i,raw,source:t.input.utterance.clone(),previous_user:user.into(),features:features(context.clone()),context,exact_roles,label:String::new()});
            }
            history.push(current);
        }
    }
    rows
}
fn entity_from(row: &SourceRow, m: &Model)->Option<Entity> {
    let (r,confidence)=m.role.best(&row.features,0);
    let role=m.roles.get(r)?;
    if role=="NONE" {return None;}
    Some(Entity{source_turn:row.turn,system_index:0,start:row.raw.start,end:row.raw.end,source:row.source.clone(),value:row.source.get(row.raw.start..row.raw.end)?.into(),role:role.clone(),confidence,context:row.context.clone()})
}
''' +s[b:]
# rank takes extra values, preserve old rank feature structure, add mask history values
s=s.replace('    ledger: &Ledger,\n) -> Vec<usize>', '    ledger: &Ledger,\n    mask_values: &[String],\n) -> Vec<usize>')
s=s.replace('    values.sort_by_key(|v| std::cmp::Reverse(v.len()));','    values.extend(mask_values.iter().map(|v| v.to_lowercase()));\n    values.sort_by(|a,b| b.len().cmp(&a.len()).then(a.cmp(b)));')
a=s.index('fn train_source(');b=s.index('fn opportunities(',a)
s=s[:a]+'''#[derive(Deserialize)]
struct BoundaryOnly { boundary: Linear }
fn train_source(data: &Dataset, frozen: &str, out: &str) -> Result<(Model,Vec<SourceRow>), Box<dyn std::error::Error>> {
    // Unknown role/rank fields are skipped by serde; never deserialize their weights.
    let frozen: BoundaryOnly=read(frozen)?;frozen.boundary.validate()?;
    if frozen.boundary.labels!=3 || !frozen.boundary.bio {return Err("invalid boundary".into());}
    let mut rows=source_rows(data,&frozen.boundary);
    let mut counts=BTreeMap::<String,usize>::new();
    for r in rows.iter().filter(|r|!r.cal) {
        if let [role]=r.exact_roles.as_slice() {*counts.entry(role.clone()).or_default()+=1;}
    }
    let mut labels:Vec<_>=counts.iter().filter(|(_,n)|**n>=20).collect();
    labels.sort_by(|a,b|b.1.cmp(a.1).then(a.0.cmp(b.0)));labels.truncate(127);
    let mut roles=vec!["NONE".into()];roles.extend(labels.iter().map(|(r,_)|(*r).clone()));
    for r in &mut rows {r.label=label_for(&r.exact_roles,&roles);}
    write(&format!("{out}/role-frequencies.json"),&counts)?;
    eprintln!("source rows={} labels={}",rows.len(),roles.len());
    let mut role=Learner::new(roles.len(),false);
    for epoch in 0..8 {
        let mut errors=0;
        for r in rows.iter().filter(|r|!r.cal) {
            role.step+=1;let y=roles.iter().position(|x|*x==r.label).ok_or("missing label")?;
            let (p,_)=role.m.best(&r.features,0);
            if p!=y {role.update(&r.features,y,1.);role.update(&r.features,p,-1.);errors+=1;}
        }
        eprintln!("role epoch {epoch} errors {errors}");
    }
    Ok((Model{version:2,roles,boundary:frozen.boundary,role:role.finish(),rank:Linear::new(1,false)},rows))
}
''' + s[b:]
s=s.replace('fn opportunities(data: &Dataset, m: &Model) -> Vec<Opportunity>', 'fn opportunities(data: &Dataset, m: &Model, source: &[SourceRow]) -> Vec<Opportunity>')
s=s.replace('    let mut rows = Vec::new();\n    let mut all_users', '    let mut by_turn=BTreeMap::<(&str,usize),Vec<&SourceRow>>::new();\n    for r in source {by_turn.entry((&r.id,r.turn)).or_default().push(r);}\n    let mut rows = Vec::new();\n    let mut all_users')
s=s.replace('        let mut prior = Vec::new();','        let mut masks=MaskHistory::default();')
s=s.replace('                ledger.push(extract(m, &t.input.utterance, i, &prior));','''                let rs=by_turn.get(&(d.id.as_str(),i)).cloned().unwrap_or_default();
                masks.push(rs.iter().filter_map(|r|r.source.get(r.raw.start..r.raw.end).map(str::to_owned)).collect());
                ledger.push(rs.iter().filter_map(|r|entity_from(r,m)).collect());''')
s=s.replace('            prior.clone_from(&b.active_services);','            let (_,mut mask_values)=state_context(Some(b));\n            mask_values.extend(masks.values());')
s=s.replace('                                &ledger,\n','                                &ledger,\n                                &mask_values,\n')
s=s.replace('let [_, data_path, out] = args.as_slice()', 'let [_, data_path, out, frozen] = args.as_slice()').replace('usage: entity-ledger DATA OUTPUT_DIR','usage: context-ledger DATA OUTPUT_DIR FROZEN_BOUNDARY_MODEL')
s=s.replace('    let mut m = train_source(&data)?;', '''    let (mut m, source) = train_source(&data,frozen,out)?;
    let mut raw_sources=BufWriter::new(File::create(format!("{out}/source-spans.jsonl"))?);
    for r in &source {
        let scores:Vec<_>=(0..m.roles.len()).map(|l|m.role.score(&r.features,l)).collect();
        let (prediction,margin)=m.role.best(&r.features,0);
        serde_json::to_writer(&mut raw_sources,&serde_json::json!({"row":r,"scores":scores,"prediction":prediction,"margin":margin,"enters_ledger":prediction!=0}))?;
        writeln!(raw_sources)?;
    }
    raw_sources.flush()?;''')
s=s.replace('let mut rows = opportunities(&data, &m);','let mut rows = opportunities(&data, &m, &source);')
s=s.replace('e, &a));','e, &a, &[]));').replace('e, &b));','e, &b, &[]));')
# strengthen explicit negative exclusion defensive ledger; raw mask history is separate.
s=s.replace('self.start < self.end','self.role != "NONE"\n            && self.start < self.end')
p.write_text(s)
