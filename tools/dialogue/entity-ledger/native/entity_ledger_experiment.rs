//! SYSTEM span/role learning and value-free binding, isolated offline harness.
use glassmind_dialogue::tracker::InputTurn;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

const BINS: usize = 262144;
fn hash(s: &str) -> usize {
    s.bytes().fold(2166136261u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(16777619)
    }) as usize
}
fn features(xs: Vec<String>) -> Vec<usize> {
    let mut f: Vec<_> = xs.iter().map(|s| hash(s)).collect();
    f.sort_unstable();
    f.dedup();
    f
}
#[derive(Serialize, Deserialize, Clone)]
struct Linear {
    w: Vec<f32>,
    labels: usize,
    bio: bool,
}
impl Linear {
    fn new(labels: usize, bio: bool) -> Self {
        Self {
            w: vec![0.; if bio { 3 * 65536 } else { BINS }],
            labels,
            bio,
        }
    }
    fn index(&self, f: usize, l: usize) -> usize {
        if self.bio {
            l * 65536 + (f & 65535)
        } else {
            (f ^ l.wrapping_mul(2654435761)) & (BINS - 1)
        }
    }
    fn score(&self, fs: &[usize], l: usize) -> f32 {
        fs.iter()
            .map(|&f| self.w.get(self.index(f, l)).copied().unwrap_or(0.))
            .sum()
    }
    fn best(&self, fs: &[usize], prev: usize) -> (usize, f32) {
        let mut best = (0, f32::NEG_INFINITY);
        let mut second = f32::NEG_INFINITY;
        for l in 0..self.labels {
            if self.bio && l == 2 && prev == 0 {
                continue;
            }
            let s = self.score(fs, l);
            if s > best.1 {
                second = best.1;
                best = (l, s);
            } else if s > second {
                second = s;
            }
        }
        (
            best.0,
            if second.is_finite() {
                best.1 - second
            } else {
                0.
            },
        )
    }
    fn validate(&self) -> Result<(), String> {
        if self.labels == 0
            || self.labels > 128
            || self.w.len() != if self.bio { 3 * 65536 } else { BINS }
            || self.w.iter().any(|x| !x.is_finite())
        {
            return Err("invalid finite bounded weights".into());
        }
        Ok(())
    }
}
struct Learner {
    m: Linear,
    sum: Vec<f64>,
    last: Vec<u64>,
    step: u64,
}
impl Learner {
    fn new(labels: usize, bio: bool) -> Self {
        let m = Linear::new(labels, bio);
        Self {
            sum: vec![0.; m.w.len()],
            last: vec![0; m.w.len()],
            m,
            step: 0,
        }
    }
    fn update(&mut self, fs: &[usize], label: usize, amount: f32) {
        for &f in fs {
            let i = self.m.index(f, label);
            if let (Some(w), Some(s), Some(last)) = (
                self.m.w.get_mut(i),
                self.sum.get_mut(i),
                self.last.get_mut(i),
            ) {
                *s += (self.step - *last) as f64 * f64::from(*w);
                *last = self.step;
                *w += amount;
            }
        }
    }
    fn finish(mut self) -> Linear {
        for ((w, s), last) in self.m.w.iter_mut().zip(self.sum).zip(self.last) {
            *w = ((s + (self.step - last) as f64 * f64::from(*w)) / self.step.max(1) as f64) as f32;
        }
        self.m
    }
}
#[derive(Clone, Debug)]
struct Token {
    s: usize,
    e: usize,
    text: String,
}
fn tokens(s: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut begin = None;
    for (i, c) in s.char_indices() {
        if c.is_alphanumeric() {
            if begin.is_none() {
                begin = Some(i);
            }
        } else {
            if let Some(b) = begin.take() {
                out.push(Token {
                    s: b,
                    e: i,
                    text: s.get(b..i).unwrap_or("").to_owned(),
                });
            }
            if !c.is_whitespace() {
                out.push(Token {
                    s: i,
                    e: i + c.len_utf8(),
                    text: c.to_string(),
                });
            }
        }
    }
    if let Some(b) = begin {
        out.push(Token {
            s: b,
            e: s.len(),
            text: s.get(b..).unwrap_or("").to_owned(),
        });
    }
    out
}
fn shape(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_uppercase() {
                'A'
            } else if c.is_lowercase() {
                'a'
            } else if c.is_numeric() {
                '0'
            } else {
                c
            }
        })
        .collect()
}
fn token_features(ts: &[Token], i: usize) -> Vec<usize> {
    let mut xs = vec!["bias".into()];
    for offset in -2isize..=2 {
        if let Some(j) = i.checked_add_signed(offset)
            && let Some(t) = ts.get(j)
        {
            xs.push(format!("w{offset}={}", t.text.to_lowercase()));
            xs.push(format!("s{offset}={}", shape(&t.text)));
        }
    }
    if let Some(t) = ts.get(i) {
        for n in 1..=3 {
            xs.push(format!(
                "pre{n}={}",
                t.text.to_lowercase().chars().take(n).collect::<String>()
            ));
            xs.push(format!(
                "suf{n}={}",
                t.text
                    .to_lowercase()
                    .chars()
                    .rev()
                    .take(n)
                    .collect::<String>()
            ));
        }
    }
    features(xs)
}
fn source_context(ts: &[Token], s: usize, e: usize, prior: &[String]) -> Vec<String> {
    let mut xs = vec![
        "bias".into(),
        format!(
            "len={}",
            ts.iter().filter(|t| t.s >= s && t.e <= e).count().min(16)
        ),
    ];
    let before: Vec<_> = ts.iter().filter(|t| t.e <= s).rev().take(3).collect();
    let after: Vec<_> = ts.iter().filter(|t| t.s >= e).take(3).collect();
    for (i, t) in before.iter().enumerate() {
        xs.push(format!("left{i}={}", t.text.to_lowercase()));
    }
    for (i, t) in after.iter().enumerate() {
        xs.push(format!("right{i}={}", t.text.to_lowercase()));
    }
    for t in ts.iter().filter(|t| t.e <= s || t.s >= e) {
        xs.push(format!("ctx={}", t.text.to_lowercase()));
    }
    for a in prior {
        xs.push(format!("prior={a}"));
    }
    xs
}
#[derive(Serialize, Deserialize, Clone)]
struct Span {
    start: usize,
    end: usize,
    role: String,
}
#[derive(Serialize, Deserialize)]
struct Turn {
    input: InputTurn,
    spans: Vec<Span>,
    baseline: Option<Snapshot>,
    gold: Vec<Frame>,
}
#[derive(Serialize, Deserialize)]
struct Conversation {
    id: String,
    cal: bool,
    turns: Vec<Turn>,
}
#[derive(Serialize, Deserialize, Clone, Default)]
struct State {
    active_intent: String,
    slot_values: BTreeMap<String, Vec<String>>,
}
#[derive(Serialize, Deserialize, Clone)]
struct Frame {
    service: String,
    state: State,
}
#[derive(Serialize, Deserialize, Clone)]
struct Snapshot {
    turn_index: usize,
    active_services: Vec<String>,
    frames: Vec<Frame>,
}
#[derive(Serialize, Deserialize)]
struct Dataset {
    roles: BTreeSet<String>,
    targets: BTreeMap<String, Vec<String>>,
    dialogues: Vec<Conversation>,
}
#[derive(Serialize, Deserialize)]
struct Model {
    version: u32,
    roles: Vec<String>,
    boundary: Linear,
    role: Linear,
    rank: Linear,
}
impl Model {
    fn validate(&self, allowed: &BTreeSet<String>) -> Result<(), String> {
        self.boundary.validate()?;
        self.role.validate()?;
        self.rank.validate()?;
        if self.version != 1
            || self.roles.is_empty()
            || self.roles.len() != self.role.labels
            || self.roles.iter().any(|r| !allowed.contains(r))
            || self.roles.iter().collect::<BTreeSet<_>>().len() != self.roles.len()
        {
            return Err("role/schema mismatch".into());
        }
        Ok(())
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Entity {
    source_turn: usize,
    system_index: usize,
    start: usize,
    end: usize,
    source: String,
    value: String,
    role: String,
    confidence: f32,
    context: Vec<String>,
}
impl Entity {
    fn valid(&self) -> bool {
        self.start < self.end
            && self.source.get(self.start..self.end) == Some(self.value.as_str())
            && self.source.len() <= 4096
            && self.confidence.is_finite()
    }
}
#[derive(Default)]
struct Ledger {
    entities: VecDeque<Entity>,
    systems: usize,
}
impl Ledger {
    fn reset(&mut self) {
        *self = Self::default();
    }
    fn push(&mut self, mut es: Vec<Entity>) {
        self.systems += 1;
        for e in &mut es {
            e.system_index = self.systems;
        }
        self.entities.extend(es.into_iter().filter(Entity::valid));
        while self.entities.len() > 128
            || self
                .entities
                .front()
                .is_some_and(|e| self.systems.saturating_sub(e.system_index) >= 32)
        {
            self.entities.pop_front();
        }
    }
}
fn extract(model: &Model, text: &str, turn: usize, prior: &[String]) -> Vec<Entity> {
    let ts = tokens(text);
    if text.len() > 4096 || ts.len() > 128 {
        return vec![];
    }
    let mut labels = Vec::new();
    let mut margins = Vec::new();
    let mut prev = 0;
    for i in 0..ts.len() {
        let mut f = token_features(&ts, i);
        f.push(hash(&format!("prev={prev}")));
        let (l, m) = model.boundary.best(&f, prev);
        labels.push(l);
        margins.push(m);
        prev = l;
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < ts.len() {
        if labels.get(i) != Some(&1) {
            i += 1;
            continue;
        }
        let first = i;
        i += 1;
        while labels.get(i) == Some(&2) {
            i += 1;
        }
        if i - first > 16 {
            continue;
        }
        let (Some(a), Some(b)) = (ts.get(first), ts.get(i - 1)) else {
            continue;
        };
        let context = source_context(&ts, a.s, b.e, prior);
        let (r, rm) = model.role.best(&features(context.clone()), 0);
        let confidence = margins
            .get(first..i)
            .unwrap_or(&[])
            .iter()
            .copied()
            .fold(rm, f32::min);
        if let (Some(role), Some(value)) = (model.roles.get(r), text.get(a.s..b.e)) {
            out.push(Entity {
                source_turn: turn,
                system_index: 0,
                start: a.s,
                end: b.e,
                source: text.into(),
                value: value.into(),
                role: role.clone(),
                confidence,
                context,
            });
        }
    }
    out
}
#[derive(Clone, Serialize, Deserialize)]
struct Candidate {
    entity: Entity,
    features: Vec<usize>,
    correct: bool,
    score: f32,
}
#[derive(Serialize, Deserialize)]
struct Opportunity {
    id: String,
    turn: usize,
    cal: bool,
    service: String,
    slot: String,
    gold: Vec<String>,
    candidates: Vec<Candidate>,
}
fn recency(n: usize) -> usize {
    [0, 1, 2, 4, 8, 16, 32]
        .into_iter()
        .find(|&x| n <= x)
        .unwrap_or(32)
}
fn rank_features(
    user: &str,
    target: &str,
    intent: &str,
    active: &[String],
    filled: &[String],
    e: &Entity,
    ledger: &Ledger,
) -> Vec<usize> {
    let pair = format!("{target}|{}", e.role);
    let age = ledger.systems.saturating_sub(e.system_index);
    let latest = !ledger
        .entities
        .iter()
        .any(|x| x.role == e.role && x.system_index > e.system_index);
    let mut xs = vec![
        format!("pair={pair}"),
        format!("target={target}"),
        format!("age={pair}:{}", recency(age)),
        format!("latest={pair}:{latest}"),
        format!("count={pair}:{}", ledger.entities.len().min(8)),
        format!("intent={pair}:{intent}"),
    ];
    for a in active {
        xs.push(format!("active={pair}:{a}"));
    }
    for s in filled {
        xs.push(format!("filled={pair}:{s}"));
    }
    for c in &e.context {
        xs.push(format!("source={pair}:{c}"));
    }
    // Explicitly remove all ledger values: no candidate text identity/equality feature.
    let mut masked = user.to_lowercase();
    let mut values: Vec<_> = ledger
        .entities
        .iter()
        .map(|x| x.value.to_lowercase())
        .collect();
    values.sort_by_key(|v| std::cmp::Reverse(v.len()));
    values.dedup();
    for v in values {
        if !v.is_empty() {
            masked = masked.replace(&v, " <entity> ");
        }
    }
    let ws: Vec<_> = tokens(&masked).into_iter().map(|t| t.text).collect();
    for w in &ws {
        xs.push(format!("user={pair}:{w}"));
    }
    for w in ws.windows(2) {
        if let [a, b] = w {
            xs.push(format!("bigram={pair}:{a}_{b}"));
        }
    }
    features(xs)
}
fn read<T: serde::de::DeserializeOwned>(p: &str) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_reader(BufReader::new(File::open(p)?))?)
}
fn write<T: Serialize>(p: &str, v: &T) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer(BufWriter::new(File::create(p)?), v)?;
    Ok(())
}

fn train_source(data: &Dataset) -> Result<Model, String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for d in data.dialogues.iter().filter(|d| !d.cal) {
        for t in &d.turns {
            if matches!(
                t.input.speaker,
                glassmind_dialogue::tracker::Speaker::System
            ) {
                for s in &t.spans {
                    *counts.entry(s.role.clone()).or_default() += 1;
                }
            }
        }
    }
    let mut rows: Vec<_> = counts.into_iter().filter(|(_, n)| *n >= 20).collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    rows.truncate(128);
    let roles: Vec<_> = rows.into_iter().map(|x| x.0).collect();
    if roles.is_empty() {
        return Err("no roles".into());
    }
    let mut sequences = Vec::new();
    let mut role_rows = Vec::new();
    let mut unaligned = 0usize;
    for d in data.dialogues.iter().filter(|d| !d.cal) {
        let mut prior = Vec::new();
        for t in &d.turns {
            if let Some(b) = &t.baseline {
                prior.clone_from(&b.active_services);
                continue;
            }
            let ts = tokens(&t.input.utterance);
            if ts.len() > 128 || t.input.utterance.len() > 4096 {
                continue;
            }
            let mut ys = vec![0; ts.len()];
            for s in &t.spans {
                let Some(r) = roles.iter().position(|r| *r == s.role) else {
                    continue;
                };
                let (Some(a), Some(b)) = (
                    ts.iter().position(|t| t.s == s.start),
                    ts.iter().position(|t| t.e == s.end),
                ) else {
                    unaligned += 1;
                    continue;
                };
                if b < a || b - a + 1 > 16 {
                    unaligned += 1;
                    continue;
                }
                for j in a..=b {
                    if let Some(y) = ys.get_mut(j) {
                        *y = if j == a { 1 } else { 2 };
                    }
                }
                role_rows.push((features(source_context(&ts, s.start, s.end, &prior)), r));
            }
            sequences.push((
                (0..ts.len())
                    .map(|i| token_features(&ts, i))
                    .collect::<Vec<_>>(),
                ys,
            ));
        }
    }
    eprintln!(
        "source roles={} sequences={} role_rows={} unaligned={}",
        roles.len(),
        sequences.len(),
        role_rows.len(),
        unaligned
    );
    let mut boundary = Learner::new(3, true);
    for epoch in 0..6 {
        let mut errors = 0;
        for (seq, ys) in &sequences {
            let mut prev = 0;
            for (f, &y) in seq.iter().zip(ys) {
                boundary.step += 1;
                let mut f = f.clone();
                f.push(hash(&format!("prev={prev}")));
                let (p, _) = boundary.m.best(&f, prev);
                if p != y {
                    boundary.update(&f, y, 1.);
                    boundary.update(&f, p, -1.);
                    errors += 1;
                }
                prev = p;
            }
        }
        eprintln!("boundary epoch {epoch} errors {errors}");
    }
    let mut role = Learner::new(roles.len(), false);
    for epoch in 0..8 {
        let mut errors = 0;
        for (f, y) in &role_rows {
            role.step += 1;
            let (p, _) = role.m.best(f, 0);
            if p != *y {
                role.update(f, *y, 1.);
                role.update(f, p, -1.);
                errors += 1;
            }
        }
        eprintln!("role epoch {epoch} errors {errors}");
    }
    Ok(Model {
        version: 1,
        roles,
        boundary: boundary.finish(),
        role: role.finish(),
        rank: Linear::new(1, false),
    })
}
fn opportunities(data: &Dataset, m: &Model) -> Vec<Opportunity> {
    let mut rows = Vec::new();
    let mut all_users = 0usize;
    let mut systems = 0usize;
    for d in &data.dialogues {
        let mut ledger = Ledger::default();
        let mut prior = Vec::new();
        for (i, t) in d.turns.iter().enumerate() {
            if matches!(
                t.input.speaker,
                glassmind_dialogue::tracker::Speaker::System
            ) {
                ledger.push(extract(m, &t.input.utterance, i, &prior));
                systems += 1;
                continue;
            }
            all_users += 1;
            let Some(b) = &t.baseline else {
                continue;
            };
            prior.clone_from(&b.active_services);
            for f in &b.frames {
                if !b.active_services.contains(&f.service) {
                    continue;
                }
                for slot in data.targets.get(&f.service).into_iter().flatten() {
                    if f.state.slot_values.contains_key(slot) {
                        continue;
                    }
                    let gold = t
                        .gold
                        .iter()
                        .find(|g| g.service == f.service)
                        .and_then(|g| g.state.slot_values.get(slot))
                        .cloned()
                        .unwrap_or_default();
                    let mut candidates = Vec::new();
                    if t.input.utterance.len() <= 4096 && tokens(&t.input.utterance).len() <= 128 {
                        for e in &ledger.entities {
                            let fs = rank_features(
                                &t.input.utterance,
                                &format!("{}/{}", f.service, slot),
                                &f.state.active_intent,
                                &b.active_services,
                                &f.state.slot_values.keys().cloned().collect::<Vec<_>>(),
                                e,
                                &ledger,
                            );
                            candidates.push(Candidate {
                                entity: e.clone(),
                                features: fs,
                                correct: gold
                                    .iter()
                                    .any(|v| v.to_lowercase() == e.value.to_lowercase()),
                                score: 0.,
                            });
                        }
                    }
                    rows.push(Opportunity {
                        id: d.id.clone(),
                        turn: i,
                        cal: d.cal,
                        service: f.service.clone(),
                        slot: slot.clone(),
                        gold,
                        candidates,
                    });
                }
            }
        }
        ledger.reset();
    }
    eprintln!(
        "trace USER={all_users} SYSTEM={systems} missing opportunities={}",
        rows.len()
    );
    rows
}
fn train_rank(rows: &[Opportunity]) -> Linear {
    let mut l = Learner::new(1, false);
    for epoch in 0..8 {
        let mut errors = 0;
        for r in rows.iter().filter(|r| !r.cal) {
            l.step += 1;
            let mut best = None;
            let mut best_s = 0.;
            let mut truth = None;
            let mut truth_s = f32::NEG_INFINITY;
            for (i, c) in r.candidates.iter().enumerate() {
                let s = l.m.score(&c.features, 0);
                if s > best_s {
                    best = Some(i);
                    best_s = s;
                }
                if c.correct && s > truth_s {
                    truth = Some(i);
                    truth_s = s;
                }
            }
            if best != truth {
                if let Some(c) = truth.and_then(|i| r.candidates.get(i)) {
                    l.update(&c.features, 0, 1.);
                }
                if let Some(c) = best.and_then(|i| r.candidates.get(i)) {
                    l.update(&c.features, 0, -1.);
                }
                errors += 1;
            }
        }
        eprintln!("rank epoch {epoch} errors {errors}");
    }
    l.finish()
}
fn select(cs: &[Candidate], source: f32, score: f32, gap: f32) -> Option<usize> {
    let mut best = None;
    let mut bs = f32::NEG_INFINITY;
    for (i, c) in cs.iter().enumerate() {
        if c.entity.confidence >= source && c.score > bs {
            best = Some(i);
            bs = c.score;
        }
    }
    let b = cs.get(best?)?;
    if bs < score || !bs.is_finite() {
        return None;
    }
    // Competing values remain competitors even when their source confidence is low.
    let alternative = cs
        .iter()
        .filter(|c| c.entity.value.to_lowercase() != b.entity.value.to_lowercase())
        .map(|c| c.score)
        .fold(0., f32::max);
    if bs - alternative < gap {
        return None;
    }
    best
}
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let [_, data_path, out] = args.as_slice() else {
        return Err("usage: entity-ledger DATA OUTPUT_DIR".into());
    };
    let data: Dataset = read(data_path)?;
    let mut m = train_source(&data)?;
    m.validate(&data.roles)?;
    write(&format!("{out}/source-model.json"), &m)?;
    let mut rows = opportunities(&data, &m);
    m.rank = train_rank(&rows);
    m.validate(&data.roles)?;
    let model_path = format!("{out}/model.json");
    write(&model_path, &m)?;
    if std::fs::metadata(&model_path)?.len() > 16 * 1024 * 1024 {
        return Err("serialized model exceeds16MiB".into());
    }
    let mut raw = BufWriter::new(File::create(format!("{out}/proposals.jsonl"))?);
    for r in &mut rows {
        for c in &mut r.candidates {
            c.score = m.rank.score(&c.features, 0);
        }
        serde_json::to_writer(&mut raw, &r)?;
        writeln!(raw)?;
    }
    raw.flush()?;
    let mut decisions = BufWriter::new(File::create(format!("{out}/decisions.jsonl"))?);
    for source in [0., 2.] {
        for score in [2., 4., 8., 16.] {
            for gap in [2., 4.] {
                for r in rows.iter().filter(|r| r.cal) {
                    let chosen = select(&r.candidates, source, score, gap);
                    serde_json::to_writer(
                        &mut decisions,
                        &serde_json::json!({"grid":[source,score,gap],"id":r.id,"turn":r.turn,"service":r.service,"slot":r.slot,"chosen":chosen}),
                    )?;
                    writeln!(decisions)?;
                }
            }
        }
    }
    decisions.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entity(v: &str, turn: usize) -> Entity {
        Entity {
            source_turn: turn,
            system_index: 0,
            start: 3,
            end: 3 + v.len(),
            source: format!("At {v}!"),
            value: v.into(),
            role: "service/name".into(),
            confidence: 5.,
            context: vec![],
        }
    }
    #[test]
    fn unicode_provenance_and_eviction_reset() {
        let mut l = Ledger::default();
        let e = entity("Élan 東京", 3);
        assert!(e.valid());
        l.push(vec![e]);
        for _ in 0..31 {
            l.push(vec![]);
        }
        assert_eq!(l.entities.len(), 1);
        l.push(vec![]);
        assert!(l.entities.is_empty());
        l.push((0..140).map(|i| entity("Name", i)).collect());
        assert_eq!(l.entities.len(), 128);
        assert_eq!(l.entities.front().map(|e| e.source_turn), Some(12));
        l.reset();
        assert_eq!(l.systems, 0);
        assert!(l.entities.is_empty());
        let mut bad = entity("Élan", 0);
        bad.end -= 1;
        assert!(!bad.valid());
    }
    #[test]
    fn ambiguity_none_and_exhaustive_rank() {
        for a in -2..=6 {
            for b in -2..=6 {
                let c = vec![
                    Candidate {
                        entity: entity("First", 1),
                        features: vec![],
                        correct: false,
                        score: a as f32,
                    },
                    Candidate {
                        entity: entity("Second", 2),
                        features: vec![],
                        correct: false,
                        score: b as f32,
                    },
                ];
                let got = select(&c, 0., 2., 2.);
                let expected = if a >= 2 && a - b.max(0) >= 2 {
                    Some(0)
                } else if b >= 2 && b - a.max(0) >= 2 {
                    Some(1)
                } else {
                    None
                };
                assert_eq!(got, expected);
            }
        }
    }
    #[test]
    fn strict_input_and_model() {
        assert!(
            serde_json::from_str::<InputTurn>(
                r#"{"speaker":"SYSTEM","utterance":"x","frames":[]}"#
            )
            .is_err()
        );
        let mut m = Linear::new(3, true);
        assert!(m.validate().is_ok());
        if let Some(w) = m.w.first_mut() {
            *w = f32::NAN;
        }
        assert!(m.validate().is_err());
        let ts = tokens("Hi Élan 東京.");
        for t in ts {
            assert_eq!("Hi Élan 東京.".get(t.s..t.e), Some(t.text.as_str()));
        }
    }
    #[test]
    fn value_free_binding() {
        let mut a = Ledger::default();
        a.push(vec![entity("Alpha", 1)]);
        let mut b = Ledger::default();
        b.push(vec![entity("Beta", 1)]);
        let fa = a
            .entities
            .front()
            .map(|e| rank_features("go to Alpha", "t", "i", &[], &[], e, &a));
        let fb = b
            .entities
            .front()
            .map(|e| rank_features("go to Beta", "t", "i", &[], &[], e, &b));
        assert_eq!(fa, fb);
    }
}
