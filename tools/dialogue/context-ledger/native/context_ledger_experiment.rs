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
fn mask(text: &str, values: &[String]) -> String {
    let mut out = text.to_lowercase();
    let mut values: Vec<_> = values
        .iter()
        .map(|v| v.to_lowercase())
        .filter(|v| !v.is_empty())
        .collect();
    values.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    values.dedup();
    for v in values {
        out = out.replace(&v, " <entity> ");
    }
    out
}
fn state_context(prior: Option<&Snapshot>) -> (Vec<String>, Vec<String>) {
    let mut fs = Vec::new();
    let mut values = Vec::new();
    if let Some(b) = prior {
        for a in &b.active_services {
            fs.push(format!("active={a}"));
        }
        for f in &b.frames {
            fs.push(format!("intent={}:{}", f.service, f.state.active_intent));
            for (slot, vs) in &f.state.slot_values {
                fs.push(format!("filled={}:{}", f.service, slot));
                values.extend(vs.iter().cloned());
            }
        }
    }
    (fs, values)
}
fn source_context(
    text: &str,
    s: usize,
    e: usize,
    user: &str,
    prior: Option<&Snapshot>,
    values: &[String],
) -> Vec<String> {
    let (state, mut masked_values) = state_context(prior);
    masked_values.extend_from_slice(values);
    let left = mask(text.get(..s).unwrap_or(""), &masked_values);
    let right = mask(text.get(e..).unwrap_or(""), &masked_values);
    let before = tokens(&left);
    let after = tokens(&right);
    let mut xs = vec![
        "bias".into(),
        format!("len={}", tokens(text.get(s..e).unwrap_or("")).len().min(16)),
    ];
    for (i, t) in before.iter().rev().take(3).enumerate() {
        xs.push(format!("left{i}={}", t.text));
    }
    for (i, t) in after.iter().take(3).enumerate() {
        xs.push(format!("right{i}={}", t.text));
    }
    for t in before.iter().chain(&after) {
        xs.push(format!("ctx={}", t.text));
    }
    // Over-limit preceding USER text supplies no lexical features, but is retained in the trace.
    if user.len() <= 4096 && tokens(user).len() <= 128 {
        let ws = tokens(&mask(user, &masked_values));
        for t in &ws {
            xs.push(format!("previous_user={}", t.text));
        }
        for pair in ws.windows(2) {
            if let [a, b] = pair {
                xs.push(format!("previous_bigram={}_{}", a.text, b.text));
            }
        }
    }
    xs.extend(state);
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
        if self.version != 2
            || self.roles.is_empty()
            || self.roles.len() != self.role.labels
            || self
                .roles
                .iter()
                .any(|r| r != "NONE" && !allowed.contains(r))
            || self.roles.first().map(String::as_str) != Some("NONE")
            || self.boundary.labels != 3
            || !self.boundary.bio
            || self.role.bio
            || self.rank.bio
            || self.rank.labels != 1
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
        self.role != "NONE"
            && self.start < self.end
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
#[derive(Clone, Serialize, Deserialize)]
struct RawSpan {
    start: usize,
    end: usize,
    boundary_margin: f32,
}
fn extract(boundary: &Linear, text: &str) -> Vec<RawSpan> {
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
        let (l, m) = boundary.best(&f, prev);
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
        if let (Some(a), Some(b)) = (ts.get(first), ts.get(i - 1)) {
            out.push(RawSpan {
                start: a.s,
                end: b.e,
                boundary_margin: margins
                    .get(first..i)
                    .unwrap_or(&[])
                    .iter()
                    .copied()
                    .fold(f32::INFINITY, f32::min),
            });
        }
    }
    out
}
#[derive(Clone, Serialize, Deserialize)]
struct SourceRow {
    id: String,
    cal: bool,
    turn: usize,
    raw: RawSpan,
    source: String,
    previous_user: String,
    context: Vec<String>,
    features: Vec<usize>,
    exact_roles: Vec<String>,
    label: String,
}
fn label_for(exact: &[String], roles: &[String]) -> String {
    if let [r] = exact
        && roles.contains(r)
    {
        r.clone()
    } else {
        "NONE".into()
    }
}
// Raw mask history carries values only; NONE predictions never enter Entity/Ledger.
#[derive(Default)]
struct MaskHistory {
    values: VecDeque<(usize, String)>,
    systems: usize,
}
impl MaskHistory {
    fn push(&mut self, values: Vec<String>) {
        self.systems += 1;
        self.values
            .extend(values.into_iter().map(|v| (self.systems, v)));
        while self.values.len() > 128
            || self
                .values
                .front()
                .is_some_and(|(i, _)| self.systems.saturating_sub(*i) >= 32)
        {
            self.values.pop_front();
        }
    }
    fn values(&self) -> Vec<String> {
        self.values.iter().map(|(_, v)| v.clone()).collect()
    }
}
fn source_rows(data: &Dataset, boundary: &Linear) -> Vec<SourceRow> {
    let mut rows = Vec::new();
    for d in &data.dialogues {
        let mut prior = None;
        let mut user = "";
        let mut history = MaskHistory::default();
        for (i, t) in d.turns.iter().enumerate() {
            if matches!(t.input.speaker, glassmind_dialogue::tracker::Speaker::User) {
                user = &t.input.utterance;
                prior = t.baseline.as_ref();
                continue;
            }
            let spans = extract(boundary, &t.input.utterance);
            let current: Vec<_> = spans
                .iter()
                .filter_map(|s| t.input.utterance.get(s.start..s.end).map(str::to_owned))
                .collect();
            let mut values = history.values();
            values.extend(current.iter().cloned());
            for raw in spans {
                let exact_roles: Vec<_> = t
                    .spans
                    .iter()
                    .filter(|s| s.start == raw.start && s.end == raw.end)
                    .map(|s| s.role.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                let context =
                    source_context(&t.input.utterance, raw.start, raw.end, user, prior, &values);
                rows.push(SourceRow {
                    id: d.id.clone(),
                    cal: d.cal,
                    turn: i,
                    raw,
                    source: t.input.utterance.clone(),
                    previous_user: user.into(),
                    features: features(context.clone()),
                    context,
                    exact_roles,
                    label: String::new(),
                });
            }
            history.push(current);
        }
    }
    rows
}
fn entity_from(row: &SourceRow, m: &Model) -> Option<Entity> {
    let (r, confidence) = m.role.best(&row.features, 0);
    let role = m.roles.get(r)?;
    if role == "NONE" {
        return None;
    }
    Some(Entity {
        source_turn: row.turn,
        system_index: 0,
        start: row.raw.start,
        end: row.raw.end,
        source: row.source.clone(),
        value: row.source.get(row.raw.start..row.raw.end)?.into(),
        role: role.clone(),
        confidence,
        context: row.context.clone(),
    })
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
    history: (&Ledger, &[String]),
) -> Vec<usize> {
    let (ledger, mask_values) = history;
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
    values.extend(mask_values.iter().map(|v| v.to_lowercase()));
    values.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
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

#[derive(Deserialize)]
struct BoundaryOnly {
    boundary: Linear,
}
fn train_source(
    data: &Dataset,
    frozen: &str,
    out: &str,
) -> Result<(Model, Vec<SourceRow>), Box<dyn std::error::Error>> {
    // Unknown role/rank fields are skipped by serde; never deserialize their weights.
    let frozen: BoundaryOnly = read(frozen)?;
    frozen.boundary.validate()?;
    if frozen.boundary.labels != 3 || !frozen.boundary.bio {
        return Err("invalid boundary".into());
    }
    let mut rows = source_rows(data, &frozen.boundary);
    let mut counts = BTreeMap::<String, usize>::new();
    for r in rows.iter().filter(|r| !r.cal) {
        if let [role] = r.exact_roles.as_slice() {
            *counts.entry(role.clone()).or_default() += 1;
        }
    }
    let mut labels: Vec<_> = counts.iter().filter(|(_, n)| **n >= 20).collect();
    labels.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    labels.truncate(127);
    let mut roles = vec!["NONE".into()];
    roles.extend(labels.iter().map(|(r, _)| (*r).clone()));
    for r in &mut rows {
        r.label = label_for(&r.exact_roles, &roles);
    }
    write(&format!("{out}/role-frequencies.json"), &counts)?;
    eprintln!("source rows={} labels={}", rows.len(), roles.len());
    let mut role = Learner::new(roles.len(), false);
    for epoch in 0..8 {
        let mut errors = 0;
        for r in rows.iter().filter(|r| !r.cal) {
            role.step += 1;
            let y = roles
                .iter()
                .position(|x| *x == r.label)
                .ok_or("missing label")?;
            let (p, _) = role.m.best(&r.features, 0);
            if p != y {
                role.update(&r.features, y, 1.);
                role.update(&r.features, p, -1.);
                errors += 1;
            }
        }
        eprintln!("role epoch {epoch} errors {errors}");
    }
    Ok((
        Model {
            version: 2,
            roles,
            boundary: frozen.boundary,
            role: role.finish(),
            rank: Linear::new(1, false),
        },
        rows,
    ))
}
fn opportunities(data: &Dataset, m: &Model, source: &[SourceRow]) -> Vec<Opportunity> {
    let mut by_turn = BTreeMap::<(&str, usize), Vec<&SourceRow>>::new();
    for r in source {
        by_turn.entry((&r.id, r.turn)).or_default().push(r);
    }
    let mut rows = Vec::new();
    let mut all_users = 0usize;
    let mut systems = 0usize;
    for d in &data.dialogues {
        let mut ledger = Ledger::default();
        let mut masks = MaskHistory::default();
        for (i, t) in d.turns.iter().enumerate() {
            if matches!(
                t.input.speaker,
                glassmind_dialogue::tracker::Speaker::System
            ) {
                let rs = by_turn
                    .get(&(d.id.as_str(), i))
                    .cloned()
                    .unwrap_or_default();
                masks.push(
                    rs.iter()
                        .filter_map(|r| r.source.get(r.raw.start..r.raw.end).map(str::to_owned))
                        .collect(),
                );
                ledger.push(rs.iter().filter_map(|r| entity_from(r, m)).collect());
                systems += 1;
                continue;
            }
            all_users += 1;
            let Some(b) = &t.baseline else {
                continue;
            };
            let (_, mut mask_values) = state_context(Some(b));
            mask_values.extend(masks.values());
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
                                (&ledger, &mask_values),
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
    let [_, data_path, out, frozen] = args.as_slice() else {
        return Err("usage: context-ledger DATA OUTPUT_DIR FROZEN_BOUNDARY_MODEL".into());
    };
    let data: Dataset = read(data_path)?;
    let (mut m, source) = train_source(&data, frozen, out)?;
    let mut raw_sources = BufWriter::new(File::create(format!("{out}/source-spans.jsonl"))?);
    for r in &source {
        let scores: Vec<_> = (0..m.roles.len())
            .map(|l| m.role.score(&r.features, l))
            .collect();
        let (prediction, margin) = m.role.best(&r.features, 0);
        serde_json::to_writer(
            &mut raw_sources,
            &serde_json::json!({"row":r,"scores":scores,"prediction":prediction,"margin":margin,"enters_ledger":prediction!=0}),
        )?;
        writeln!(raw_sources)?;
    }
    raw_sources.flush()?;
    m.validate(&data.roles)?;
    write(&format!("{out}/source-model.json"), &m)?;
    let mut rows = opportunities(&data, &m, &source);
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
            .map(|e| rank_features("go to Alpha", "t", "i", &[], &[], e, (&a, &[])));
        let fb = b
            .entities
            .front()
            .map(|e| rank_features("go to Beta", "t", "i", &[], &[], e, (&b, &[])));
        assert_eq!(fa, fb);
    }
    #[test]
    fn contextual_source_masks_entities_and_uses_actual_user() {
        let a = source_context(
            "Visit Alpha near Gamma.",
            6,
            11,
            "Find Alpha near Gamma",
            None,
            &["Alpha".into(), "Gamma".into()],
        );
        let b = source_context(
            "Visit Bravo near Delta.",
            6,
            11,
            "Find Bravo near Delta",
            None,
            &["Bravo".into(), "Delta".into()],
        );
        assert_eq!(a, b);
        assert!(
            a.iter()
                .all(|x| !x.contains("alpha") && !x.contains("gamma"))
        );
        let c = source_context(
            "Visit Alpha near Gamma.",
            6,
            11,
            "Avoid Alpha near Gamma",
            None,
            &["Alpha".into(), "Gamma".into()],
        );
        assert_ne!(features(a), features(c));
    }
    #[test]
    fn none_supervision_and_ledger_exclusion() {
        let roles = vec!["NONE".into(), "service/name".into()];
        assert_eq!(label_for(&[], &roles), "NONE");
        assert_eq!(label_for(&["other/name".into()], &roles), "NONE");
        assert_eq!(
            label_for(&["service/name".into(), "other/name".into()], &roles),
            "NONE"
        );
        assert_eq!(label_for(&["service/name".into()], &roles), "service/name");
        let mut l = Ledger::default();
        let mut e = entity("Alpha", 0);
        e.role = "NONE".into();
        l.push(vec![e]);
        assert!(l.entities.is_empty());
        let mut h = MaskHistory::default();
        h.push(vec!["Alpha".into()]);
        for _ in 0..32 {
            h.push(vec![]);
        }
        assert!(h.values().is_empty());
    }
    #[test]
    fn low_confidence_competitor_blocks_and_same_value_does_not() {
        let mut a = entity("Alpha", 0);
        a.confidence = 4.;
        let mut b = entity("Beta", 1);
        b.confidence = 0.;
        let mut cs = vec![
            Candidate {
                entity: a,
                features: vec![],
                correct: false,
                score: 8.,
            },
            Candidate {
                entity: b,
                features: vec![],
                correct: false,
                score: 7.,
            },
        ];
        assert_eq!(select(&cs, 2., 4., 2.), None);
        if let Some(c) = cs.get_mut(1) {
            c.entity.value = "Alpha".into();
        }
        assert_eq!(select(&cs, 2., 4., 2.), Some(0));
    }
}
