//! Compositional linear graph proposals; execution is not semantic verification.
use crate::rational::{Rational, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, io::Read, path::Path, sync::OnceLock};

pub const CONSTANTS: [&str; 20] = [
    "0", "1", "2", "3", "4", "5", "6", "10", "12", "26", "52", "60", "100", "180", "360", "1000",
    "3600", "0.25", "3.6", "1.6",
];
pub const FEATURES: usize = 8192;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    content = "index",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Arg {
    Input(u8),
    Step(u8),
    Constant(u8),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub op: Op,
    pub left: Arg,
    pub right: Arg,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub steps: Vec<Step>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Number {
    pub start: usize,
    pub end: usize,
    pub literal: String,
    pub value: String,
}

pub fn numbers(question: &str) -> Result<Vec<Number>> {
    if question.len() > 4096 {
        return Err("input exceeds 4096 bytes".into());
    }
    if question
        .chars()
        .any(|c| c.is_numeric() && !c.is_ascii_digit())
    {
        return Err("unsupported Unicode numeric spelling".into());
    }
    let lower = question.to_ascii_lowercase();
    let unsupported = "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty thirty forty fifty sixty seventy eighty ninety hundred thousand million billion dozen half quarter";
    if lower
        .split(|c: char| !c.is_ascii_alphabetic())
        .any(|w| !w.is_empty() && unsupported.split(' ').any(|n| n == w))
    {
        return Err("unsupported number word".into());
    }
    static RE: OnceLock<Result<Regex>> = OnceLock::new();
    let re = RE
        .get_or_init(|| {
            Regex::new(r"[+-]?(?:[0-9]{1,3}(?:,[0-9]{3})+|[0-9]+)(?:\.[0-9]+)?|[+-]?\.[0-9]+")
                .map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)?;
    let mut result = Vec::new();
    for m in re.find_iter(question) {
        let prefix = &question[..m.start()];
        let before = prefix.chars().next_back();
        if prefix.trim_end().ends_with(['+', '-', '−', '–', '—']) {
            return Err("unsupported separated or Unicode numeric sign".into());
        }
        let after = question[m.end()..].chars().next();
        let bad_neighbor = |c: char| {
            c.is_alphanumeric() || c == '_' || c == '.' || c == ',' || c == '+' || c == '-'
        };
        // A final sentence period or comma followed by whitespace is punctuation.
        let punctuation = matches!(after, Some('.' | ','))
            && question[m.end() + 1..]
                .chars()
                .next()
                .is_none_or(char::is_whitespace);
        if before.is_some_and(bad_neighbor) || (after.is_some_and(bad_neighbor) && !punctuation) {
            return Err("unsupported numeric spelling or boundary".into());
        }
        let value: Rational = m.as_str().replace(',', "").parse()?;
        result.push(Number {
            start: m.start(),
            end: m.end(),
            literal: m.as_str().into(),
            value: value.to_string(),
        });
        if result.len() > 16 {
            return Err("more than 16 numeric slots".into());
        }
    }
    // Every digit must belong to a retained literal.
    if question
        .char_indices()
        .any(|(i, c)| c.is_ascii_digit() && !result.iter().any(|n| n.start <= i && i < n.end))
    {
        return Err("unbound digit".into());
    }
    Ok(result)
}
impl Program {
    pub fn validate(&self) -> Result<()> {
        if self.steps.is_empty() || self.steps.len() > 6 {
            return Err("program must contain 1..6 steps".into());
        }
        for (i, s) in self.steps.iter().enumerate() {
            for a in [&s.left, &s.right] {
                let valid = match a {
                    Arg::Input(n) => *n < 16,
                    Arg::Step(n) => usize::from(*n) < i,
                    Arg::Constant(n) => usize::from(*n) < CONSTANTS.len(),
                };
                if !valid {
                    return Err("invalid or forward program reference".into());
                }
            }
        }
        Ok(())
    }
    pub fn execute(&self, question: &str) -> Result<(Vec<Number>, Rational)> {
        self.validate()?;
        let nums = numbers(question)?;
        let inputs = nums
            .iter()
            .map(|n| n.value.parse::<Rational>())
            .collect::<Result<Vec<_>>>()?;
        let mut values: Vec<Rational> = Vec::new();
        for s in &self.steps {
            let get = |a: &Arg| -> Result<Rational> {
                match a {
                    Arg::Input(i) => inputs
                        .get(usize::from(*i))
                        .copied()
                        .ok_or_else(|| "missing numeric slot".into()),
                    Arg::Step(i) => values
                        .get(usize::from(*i))
                        .copied()
                        .ok_or_else(|| "missing step".into()),
                    Arg::Constant(i) => CONSTANTS
                        .get(usize::from(*i))
                        .ok_or("invalid constant")?
                        .parse(),
                }
            };
            let a = get(&s.left)?;
            let b = get(&s.right)?;
            values.push(match s.op {
                Op::Add => a.checked_add(b),
                Op::Subtract => a.checked_sub(b),
                Op::Multiply => a.checked_mul(b),
                Op::Divide => a.checked_div(b),
            }?);
        }
        Ok((nums, *values.last().ok_or("empty program")?))
    }
}

pub const ACTIONS: [usize; 3] = [5, 42, 42];
fn hash(s: &str) -> usize {
    let mut h = 14695981039346656037_u64;
    for b in s.bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(1099511628211);
    }
    h as usize % FEATURES
}
fn words(s: &str) -> Vec<String> {
    s.to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphabetic())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}
pub fn arg_id(a: &Arg) -> usize {
    match a {
        Arg::Input(i) => usize::from(*i),
        Arg::Constant(i) => 16 + usize::from(*i),
        Arg::Step(i) => 36 + usize::from(*i),
    }
}
fn argument(i: usize) -> Arg {
    if i < 16 {
        Arg::Input(i as u8)
    } else if i < 36 {
        Arg::Constant((i - 16) as u8)
    } else {
        Arg::Step((i - 36) as u8)
    }
}
pub fn op_id(o: &Op) -> usize {
    match o {
        Op::Add => 0,
        Op::Subtract => 1,
        Op::Multiply => 2,
        Op::Divide => 3,
    }
}
fn operation(i: usize) -> Result<Op> {
    match i {
        0 => Ok(Op::Add),
        1 => Ok(Op::Subtract),
        2 => Ok(Op::Multiply),
        3 => Ok(Op::Divide),
        _ => Err("unknown operation".into()),
    }
}
pub fn legal(head: usize, n: usize, depth: usize) -> Vec<usize> {
    if head == 0 {
        (0..if depth == 0 { 4 } else { 5 }).collect()
    } else {
        (0..n).chain(16..36).chain(36..36 + depth).collect()
    }
}
pub fn base_features(q: &str, nums: &[Number]) -> Vec<String> {
    let ws = words(q);
    let mut fs = vec!["bias".into(), format!("count:{}", nums.len())];
    for w in &ws {
        fs.push(format!("u:{w}"));
    }
    for pair in ws.windows(2) {
        if let [a, b] = pair {
            fs.push(format!("b:{a} {b}"));
        }
    }
    for (i, num) in nums.iter().enumerate() {
        let before = words(&q[..num.start]);
        let after = words(&q[num.end..]);
        for (d, w) in before.iter().rev().take(3).enumerate() {
            fs.push(format!("slot{i}:before{d}:{w}"));
        }
        for (d, w) in after.iter().take(3).enumerate() {
            fs.push(format!("slot{i}:after{d}:{w}"));
        }
        let value = num.literal.replace(',', "").parse::<f64>().unwrap_or(0.0);
        let rank = nums
            .iter()
            .filter(|n| n.literal.replace(',', "").parse::<f64>().unwrap_or(0.0) < value)
            .count();
        fs.push(format!("slot{i}:rank{rank}"));
        fs.push(format!(
            "slot{i}:negative{}:integer{}",
            value < 0.0,
            !num.literal.contains('.')
        ));
    }
    fs
}
pub fn state_features(
    base: &[String],
    graph: &Program,
    op: Option<usize>,
    left: Option<usize>,
) -> Vec<usize> {
    let depth = graph.steps.len();
    let mut fs = BTreeSet::new();
    for s in base {
        fs.insert(hash(s));
        fs.insert(hash(&format!("d{depth}:{s}")));
    }
    fs.insert(hash(&format!("depth:{depth}")));
    for (j, s) in graph.steps.iter().enumerate() {
        let o = op_id(&s.op);
        let l = arg_id(&s.left);
        let r = arg_id(&s.right);
        fs.insert(hash(&format!("graph{j}:op{o}:left{l}:right{r}")));
        fs.insert(hash(&format!("ago{}:op{o}", depth - j)));
        fs.insert(hash(&format!("used:{l}")));
        fs.insert(hash(&format!("used:{r}")));
        if j + 1 == depth {
            for b in base.iter().filter(|b| b.starts_with("u:")) {
                fs.insert(hash(&format!("prev{o}:{b}")));
            }
        }
    }
    if let Some(o) = op {
        fs.insert(hash(&format!("op:{o}")));
        for b in base {
            fs.insert(hash(&format!("op{o}:{b}")));
        }
    }
    if let Some(l) = left {
        fs.insert(hash(&format!("left:{l}")));
        fs.insert(hash(&format!("op{:?}:left{l}", op)));
    }
    fs.into_iter().collect()
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub schema: String,
    pub provenance: String,
    pub weights: Vec<Vec<f32>>,
    #[serde(skip)]
    pub model_sha256: String,
}
impl Model {
    pub fn new(provenance: String) -> Self {
        Self {
            schema: "equation-sequence-v1".into(),
            provenance,
            weights: ACTIONS.iter().map(|n| vec![0.0; n * FEATURES]).collect(),
            model_sha256: String::new(),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema != "equation-sequence-v1"
            || self.provenance.is_empty()
            || self.provenance.len() > 1024
            || self.weights.len() != 3
        {
            return Err("invalid model shape".into());
        }
        for (w, n) in self.weights.iter().zip(ACTIONS) {
            if w.len() != n * FEATURES || w.iter().any(|x| !x.is_finite()) {
                return Err("invalid weights".into());
            }
        }
        Ok(())
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        use sha2::{Digest, Sha256};
        if bytes.len() > 32_000_000 {
            return Err("model exceeds 32000000 bytes".into());
        }
        let mut m: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        m.validate()?;
        m.model_sha256 = format!("{:x}", Sha256::digest(bytes));
        Ok(m)
    }
    pub fn load(path: &Path) -> Result<Self> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(32_000_001)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        Self::from_bytes(&bytes)
    }
    pub fn distribution(&self, head: usize, fs: &[usize], actions: &[usize]) -> Result<Vec<f64>> {
        let weights = self.weights.get(head).ok_or("invalid head")?;
        let scale = 1.0 / (fs.len() as f64).sqrt();
        let mut scores = Vec::new();
        for a in actions {
            let mut score = 0.0;
            for f in fs {
                score +=
                    f64::from(*weights.get(a * FEATURES + f).ok_or("invalid feature")?) * scale;
            }
            if !score.is_finite() {
                return Err("nonfinite score".into());
            }
            scores.push(score);
        }
        let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let z = scores.iter().map(|s| (s - max).exp()).sum::<f64>().ln();
        for s in &mut scores {
            *s = *s - max - z;
            if !s.is_finite() {
                return Err("nonfinite log score".into());
            }
        }
        Ok(scores)
    }
    fn choose(&self, head: usize, fs: &[usize], actions: &[usize]) -> Result<(usize, f64)> {
        let ps = self.distribution(head, fs, actions)?;
        let mut best = None;
        for (&a, &p) in actions.iter().zip(&ps) {
            if best.is_none_or(|(_, b)| p > b) {
                best = Some((a, p));
            }
        }
        best.ok_or("no legal action".into())
    }
    pub fn propose(&self, question: &str) -> serde_json::Value {
        match self.propose_checked(question) {
            Ok(x) => x,
            Err(reason) => serde_json::json!({"status":"refusal","reason":reason}),
        }
    }
    fn propose_checked(&self, question: &str) -> Result<serde_json::Value> {
        let nums = numbers(question)?;
        let base = base_features(question, &nums);
        let mut graph = Program { steps: vec![] };
        let mut score = 0.0;
        let mut count = 0;
        while graph.steps.len() < 6 {
            let depth = graph.steps.len();
            let fs = state_features(&base, &graph, None, None);
            let (op, p) = self.choose(0, &fs, &legal(0, nums.len(), depth))?;
            score += p;
            count += 1;
            if op == 4 {
                break;
            }
            let (left, p) = self.choose(
                1,
                &state_features(&base, &graph, Some(op), None),
                &legal(1, nums.len(), depth),
            )?;
            score += p;
            count += 1;
            let (right, p) = self.choose(
                2,
                &state_features(&base, &graph, Some(op), Some(left)),
                &legal(2, nums.len(), depth),
            )?;
            score += p;
            count += 1;
            graph.steps.push(Step {
                op: operation(op)?,
                left: argument(left),
                right: argument(right),
            });
        }
        let (numbers, value) = graph.execute(question)?;
        Ok(
            serde_json::json!({"status":"proposed","semantics":"unverified","numbers":numbers,"program":graph,"value":value.to_string(),"score":score/f64::from(count),"model_sha256":self.model_sha256,"model_provenance":self.provenance}),
        )
    }
}
