//! Fully consumed local clauses, followed by checked constant independent rates.
use crate::rational::{Rational, Result};
use regex::Regex;
use std::{collections::{BTreeMap, BTreeSet, HashMap}, sync::{Mutex, OnceLock}};

const N: &str = r"[0-9]+(?:\.[0-9]+|/[0-9]+)?";
const U: &str = r"(?:seconds?|secs?|minutes?|mins?|hours?|hrs?|days?)";
const W: &str = r"(?:a piece of work|a job|a work|a tank|a cistern|the same piece of work|the same job|the same work|the same tank|the same cistern|that piece of work|the job|the work|the tank|the cistern|it)";
const VF: &str = r"(?:do|does|complete|completes|finish|finishes|fill|fills|empty|empties|drain|drains)";
const V: &str = r"(?:do|complete|finish|fill|empty|drain)";
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Agent(String);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Work { Job, Tank }
#[derive(Clone, Debug)]
struct Origin { start: usize, end: usize }
#[derive(Debug)]
struct Fact { agents: Vec<Agent>, direction: i64, seconds: Rational, origin: Origin }
#[derive(Debug)]
struct Query { agents: Vec<Agent>, direction: i64, unit: Option<String>, origin: Origin }
#[derive(Default)]
struct Model { work: Option<Work>, facts: Vec<Fact>, query: Option<Query>, together: Option<Vec<Agent>>, initial: Option<i64>, units: BTreeSet<String>, clauses: Vec<(Origin,String)> }
fn re(p: &str) -> Result<Regex> {
    static CACHE: OnceLock<Mutex<HashMap<String, Regex>>> = OnceLock::new();
    let mut cache=CACHE.get_or_init(||Mutex::new(HashMap::new())).lock().map_err(|_|"rate grammar cache unavailable")?;
    if let Some(r)=cache.get(p) {return Ok(r.clone());}
    let r=Regex::new(p).map_err(|e|e.to_string())?;
    cache.insert(p.into(),r.clone()); Ok(r)
}
fn positive(n: Rational) -> Result<Rational> {
    if n.is_negative() || n == Rational::integer(0) { Err("nonpositive completion time or rate".into()) } else { Ok(n) }
}
fn unit(s: &str) -> Result<(&'static str, i64)> {
    match s { "second"|"seconds"|"sec"|"secs" => Ok(("seconds",1)), "minute"|"minutes"|"min"|"mins"=>Ok(("minutes",60)), "hour"|"hours"|"hr"|"hrs"=>Ok(("hours",3600)), "day"|"days"=>Ok(("days",86400)), _=>Err("unknown time unit".into()) }
}
impl Model {
    fn agents(&self, s: &str) -> Result<Vec<Agent>> {
        let s=s.trim().trim_end_matches(" alone").trim_end_matches(" together").trim();
        if matches!(s,"they"|"both"|"both of them"|"both the pipes"|"all pipes"|"all the pipes") {
            if let Some(a)=&self.together { return Ok(a.clone()); }
            let a:BTreeSet<_>=self.facts.iter().flat_map(|f|f.agents.clone()).collect();
            if (2..=3).contains(&a.len()) { return Ok(a.into_iter().collect()); }
            return Err("ambiguous group reference".into());
        }
        let s=s.strip_prefix("two persons ").or_else(||s.strip_prefix("two pipes ")).or_else(||s.strip_prefix("pipes ")).unwrap_or(s);
        let mut out=Vec::new();
        for part in s.split(" and ").flat_map(|s|s.split(',')) {
            let part=part.trim();
            if part.is_empty() {continue;}
            let name=part.strip_prefix("pipe ").or_else(||part.strip_prefix("worker ")).or_else(||part.strip_prefix("machine ")).unwrap_or(part);
            if !name.bytes().all(|c|c.is_ascii_lowercase()) || name.len()>32 || matches!(name,"he"|"she"|"it"|"they"|"both"|"all"|"another"|"one") {return Err("unsupported agent identity".into());}
            out.push(Agent(name.into()));
        }
        out.sort();
        if out.is_empty() || out.len()>3 || out.windows(2).any(|w|w[0]==w[1]) {return Err("ambiguous agent group".into());}
        Ok(out)
    }
    fn work(&mut self, s:&str, verb:&str) -> Result<i64> {
        let verb=match verb {"does"=>"do","completes"=>"complete","finishes"=>"finish","fills"=>"fill","empties"=>"empty","drains"=>"drain",v=>v};
        let kind=if s.contains("tank")||s.contains("cistern") {Some(Work::Tank)} else if s=="it" {None} else {Some(Work::Job)};
        if s.starts_with("a ") {
            if self.work.is_some() {return Err("multiple independent workloads".into());}
            self.work=kind;
        } else {
            let known=self.work.ok_or("unbound work reference")?;
            if kind.is_some_and(|k|k!=known) {return Err("conflicting work identity".into());}
        }
        match (self.work,verb) {
            (Some(Work::Job),"do"|"complete"|"finish")=>Ok(1),
            (Some(Work::Tank),"fill")=>Ok(1),
            (Some(Work::Tank),"empty"|"drain")=>Ok(-1),
            _=>Err("operation incompatible with work".into())
        }
    }
    fn state(&mut self, s:&str) -> Result<()> {
        if self.work!=Some(Work::Tank) {return Err("unbound tank state".into());}
        let initial=if s=="empty" {1}else{-1};
        if self.initial.replace(initial).is_some_and(|v|v!=initial) {return Err("conflicting tank state".into());}
        Ok(())
    }
    fn execute(&self) -> Result<(Rational,String)> {
        let query=self.query.as_ref().ok_or("missing completion query")?;
        let all:BTreeSet<_>=self.facts.iter().flat_map(|f|f.agents.clone()).collect();
        if !(2..=3).contains(&all.len()) || query.agents.iter().any(|a|!all.contains(a)) {return Err("unbound agents".into());}
        if self.together.as_ref().is_some_and(|a|a.iter().cloned().collect::<BTreeSet<_>>()!=all) {return Err("unbound or unused simultaneous group".into());}
        if self.clauses.iter().any(|(o,s)|o.end-o.start!=s.len()) {return Err("invalid source clause".into());}
        // Retain and validate source provenance for every bound operand and query.
        if self.facts.iter().any(|f|f.origin.end<=f.origin.start) || query.origin.end<=query.origin.start {return Err("invalid clause provenance".into());}
        let out_unit=if let Some(u)=&query.unit {u.clone()} else if self.units.len()==1 {self.units.iter().next().ok_or("missing unit")?.clone()} else {return Err("mixed units require explicit query unit".into());};
        let mut singles=BTreeMap::new();
        let mut group=None;
        for f in &self.facts {
            let rate=Rational::integer(f.direction).checked_div(f.seconds)?;
            if f.agents.len()==1 {
                if singles.insert(f.agents[0].clone(),rate).is_some_and(|r|r!=rate) {return Err("conflicting individual facts".into());}
            } else if let Some((a,r))=&group {
                if a!=&f.agents || r!=&rate {return Err("conflicting group facts".into());}
            } else { group=Some((f.agents.clone(),rate)); }
        }
        let rate=if query.agents.len()>=2 {
            if query.agents.iter().cloned().collect::<BTreeSet<_>>()!=all || singles.len()!=all.len() {return Err("unbound simultaneous rate".into());}
            let mut sum=Rational::integer(0);
            for r in singles.values() {sum=sum.checked_add(*r)?;}
            if let Some((a,r))=&group {if a!=&query.agents||*r!=sum {return Err("conflicting group rate".into());}}
            sum
        } else {
            let unknown=&query.agents[0];
            if singles.contains_key(unknown) || singles.len()+1!=all.len() {return Err("query is not one missing individual".into());}
            let (a,mut r)=group.ok_or("missing joint completion fact")?;
            if a.into_iter().collect::<BTreeSet<_>>()!=all {return Err("group omits operands".into());}
            for value in singles.values() {r=r.checked_sub(*value)?;}
            r
        };
        if self.work==Some(Work::Tank) && query.agents.len()>1 && self.initial!=Some(query.direction) {return Err("tank must explicitly start empty for filling or full for draining".into());}
        let seconds=positive(Rational::integer(query.direction).checked_div(rate)?)?;
        Ok((seconds.checked_div(Rational::integer(unit(&out_unit)?.1))?,out_unit))
    }
}

pub(crate) fn solve(text:&str)->Result<Option<(Rational,String)>> {
    if !text.contains(" job") && !text.contains(" work") && !text.contains(" tank") && !text.contains(" cistern") {return Ok(None);}
    let fact=re(&format!(r"^(.*?) (?:alone )?(?:can |could |is able to )?(?:together )?({VF}) ({W})(?: together)? in ({N})(?: ({U}))?(?:(?: and |, )({N})(?: ({U}))?)?(?: and ({N})(?: ({U}))?)?(?: respectively)?"))?;
    let takes=re(&format!(r"^(.*?) (?:takes|take) ({N}) ({U}) to ({V}) ({W})"))?;
    let elliptical=re(&format!(r"^(.*?) in ({N}) ({U})"))?;
    let slot=re(&format!(r"^(.*?) (?:can |will )?({V}) ({W}) in$"))?;
    let bare_query=re(r"^how long will (.*?) take$")?;
    let inverted=re(&format!(r"^what time (.*?) will take to ({V}) ({W})(?: (alone|together))?$"))?;
    let q1=re(&format!(r"^(?:how long|how much time|in what time) (?:will|would|does|do|can) (.*?) take to ({V}) ({W})(?: (together|alone))?$"))?;
    let q2=re(&format!(r"^(?:in )?how many ({U}) (?:can|will) (.*?) ({V}) ({W})(?: (together|alone))?$"))?;
    let q3=re(&format!(r"^in what time can (.*?) ({V}) ({W})(?: (together|alone))?$"))?;
    let passive=re(&format!(r"^(?:(?:in )?how many ({U}) will ({W}) be (completed|finished|filled|emptied)|how long will it take to ({V}) ({W}))$"))?;
    let condition=re(r"^if (.*?) (?:work together|are opened simultaneously|are opened together|are opened|are open)(?: when the tank is (empty|full))?")?;
    let state=re(r"^the (?:tank|cistern) is (?:initially )?(empty|full)")?;
    let mut model=Model::default();
    let mut rest=text;
    let mut suffix_condition: Option<(&str, usize)> = None;
    if let Some((before, after))=text.rsplit_once(" if ") {
        let condition_text=&text[before.len()+1..];
        if condition.captures(condition_text).is_some_and(|c|c[0].len()==condition_text.len()) {
            rest=before;
            suffix_condition=Some((after,before.len()+4));
        }
    }
    let source=text;
    let mut count=0;
    while !rest.is_empty() {
        count+=1;
        if count>16 {return Err("too many clauses".into());}
        let start=rest.as_ptr() as usize-source.as_ptr() as usize;
        // Conditional factual premises are consumed as facts, never discarded.
        if rest.starts_with("if ") && condition.captures(rest).is_none() {rest=&rest[3..];}
        rest=rest.strip_prefix("then ").unwrap_or(rest);
        rest=rest.strip_prefix("working together ").unwrap_or(rest);
        if rest.starts_with("how ") || rest.starts_with("in how ") || rest.starts_with("in what ") {
            if let Some((condition_text, offset))=suffix_condition.take() {
                let clause=format!("if {condition_text}");
                let c=condition.captures(&clause).ok_or("invalid trailing condition")?;
                let agents=model.agents(&c[1])?;
                if agents.len()<2 || model.together.replace(agents).is_some() {return Err("conflicting simultaneous condition".into());}
                if let Some(initial)=c.get(2) {model.state(initial.as_str())?;}
                model.clauses.push((Origin{start:offset-3,end:source.len()},source[offset-3..].into()));
            }
        }
        let consumed;
        if let Some(c)=state.captures(rest) {
            model.state(&c[1])?;
            consumed=c[0].len();
        } else if let Some(c)=condition.captures(rest) {
            let agents=model.agents(&c[1])?;
            if agents.len()<2 || model.together.replace(agents).is_some() {return Err("ambiguous simultaneous condition".into());}
            if let Some(initial)=c.get(2) {model.state(initial.as_str())?;}
            consumed=c[0].len();
        } else if let Some(c)=passive.captures(rest) {
            let agents=model.together.clone().ok_or("unbound passive query agents")?;
            let (work,verb)=if let Some(w)=c.get(2) {
                (w.as_str(),match &c[3] {"completed"=>"complete","finished"=>"finish","filled"=>"fill",_=>"empty"})
            } else {(c.get(5).ok_or("query work")?.as_str(),c.get(4).ok_or("query verb")?.as_str())};
            let direction=model.work(work,verb)?;
            consumed=c[0].len();
            model.query=Some(Query{agents,direction,unit:c.get(1).map(|u|unit(u.as_str()).map(|v|v.0.into())).transpose()?,origin:Origin{start,end:start+consumed}});
        } else if let Some(c)=bare_query.captures(rest) {
            let agents=model.agents(&c[1])?;
            if model.together.as_ref()!=Some(&agents) || model.work.is_none() {return Err("unbound completion query".into());}
            consumed=c[0].len();
            let direction=if model.work==Some(Work::Tank) {model.initial.ok_or("unbound tank direction")?}else{1};
            model.query=Some(Query{agents,direction,unit:None,origin:Origin{start,end:start+consumed}});
        } else if let Some(c)=slot.captures(rest) {
            let agents=model.agents(&c[1])?;
            if agents.len()!=1 {return Err("unbound individual query".into());}
            let direction=model.work(&c[3],&c[2])?;
            consumed=c[0].len();
            model.query=Some(Query{agents,direction,unit:None,origin:Origin{start,end:start+consumed}});
        } else if let Some(c)=q1.captures(rest).or_else(||q3.captures(rest)).or_else(||inverted.captures(rest)) {
            let agents=model.agents(&c[1])?;
            if agents.len()>1 && c.get(4).is_some_and(|m|m.as_str()=="alone") {return Err("conflicting alone and together".into());}
            if agents.len()>1 && c.get(4).map(|m|m.as_str())!=Some("together") && model.together.as_ref()!=Some(&agents) {return Err("missing simultaneous condition".into());}
            let direction=model.work(&c[3],&c[2])?;
            consumed=c[0].len();
            model.query=Some(Query{agents,direction,unit:None,origin:Origin{start,end:start+consumed}});
        } else if let Some(c)=q2.captures(rest) {
            let agents=model.agents(&c[2])?;
            if agents.len()>1 && c.get(5).is_some_and(|m|m.as_str()=="alone") {return Err("group cannot act alone individually in one query".into());}
            if agents.len()>1 && !c[2].ends_with(" together") && c.get(5).map(|m|m.as_str())!=Some("together") && model.together.as_ref()!=Some(&agents) {return Err("missing simultaneous condition".into());}
            let direction=model.work(&c[4],&c[3])?;
            consumed=c[0].len();
            model.query=Some(Query{agents,direction,unit:Some(unit(&c[1])?.0.into()),origin:Origin{start,end:start+consumed}});
        } else if let Some(c)=takes.captures(rest) {
            let agents=model.agents(&c[1])?;
            let direction=model.work(&c[5],&c[4])?;
            let (u,scale)=unit(&c[3])?;
            model.units.insert(u.into());
            consumed=c[0].len();
            model.facts.push(Fact{agents,direction,seconds:positive(c[2].parse::<Rational>()?)?.checked_mul(Rational::integer(scale))?,origin:Origin{start,end:start+consumed}});
        } else if let Some(c)=fact.captures(rest) {
            let agents=model.agents(&c[1])?;
            let direction=model.work(&c[3],&c[2])?;
            let mut values=vec![c[4].parse::<Rational>()?];
            if let Some(n)=c.get(6) {values.push(n.as_str().parse()?);}
            if let Some(n)=c.get(8) {values.push(n.as_str().parse()?);}
            let u=c.get(9).or_else(||c.get(7)).or_else(||c.get(5)).ok_or("missing time unit")?.as_str();
            if c.get(5).is_some_and(|v|unit(v.as_str()).ok()!=unit(u).ok()) {return Err("mixed units in paired times".into());}
            let (u,scale)=unit(u)?;
            model.units.insert(u.into());
            consumed=c[0].len();
            if values.len()>1 {
                if agents.len()!=values.len() {return Err("unbound paired completion times".into());}
                // Agent textual order is significant for respectively; recover it before sorting.
                let raw=c[1].strip_prefix("two persons ").or_else(||c[1].strip_prefix("two pipes ")).or_else(||c[1].strip_prefix("pipes ")).unwrap_or(&c[1]);
                let ordered=raw.split(" and ").flat_map(|s|s.split(',')).filter(|s|!s.trim().is_empty()).map(|s|model.agents(s).map(|a|a[0].clone())).collect::<Result<Vec<_>>>()?;
                for (a,n) in ordered.into_iter().zip(values) {model.facts.push(Fact{agents:vec![a],direction,seconds:positive(n)?.checked_mul(Rational::integer(scale))?,origin:Origin{start,end:start+consumed}});}
            } else {
                model.facts.push(Fact{agents,direction,seconds:positive(values[0])?.checked_mul(Rational::integer(scale))?,origin:Origin{start,end:start+consumed}});
            }
        } else if let Some(c)=elliptical.captures(rest) {
            let agents=model.agents(&c[1])?;
            let previous=model.facts.last().ok_or("unbound completion ellipsis")?;
            let direction=previous.direction;
            let (u,scale)=unit(&c[3])?;
            model.units.insert(u.into());
            consumed=c[0].len();
            model.facts.push(Fact{agents,direction,seconds:positive(c[2].parse::<Rational>()?)?.checked_mul(Rational::integer(scale))?,origin:Origin{start,end:start+consumed}});
        } else {return Err("unsupported rate clause".into());}
        let clause_end=(rest.as_ptr() as usize-source.as_ptr() as usize)+consumed;
        model.clauses.push((Origin{start,end:clause_end},source[start..clause_end].into()));
        rest=&rest[consumed..];
        if model.query.is_some() && !rest.is_empty() {return Err("extra clause after query".into());}
        if !rest.is_empty() {
            let next=rest.trim_start();
            rest=if let Some(s)=next.strip_prefix('.').or_else(||next.strip_prefix(';')).or_else(||next.strip_prefix(',')) {s.trim_start()} else if let Some(s)=next.strip_prefix("and ").or_else(||next.strip_prefix("while ")) {s} else {return Err("unconsumed rate clause text".into());};
            rest=rest.strip_prefix("and ").or_else(||rest.strip_prefix("while ")).unwrap_or(rest);
        }
    }
    model.execute().map(Some)
}
