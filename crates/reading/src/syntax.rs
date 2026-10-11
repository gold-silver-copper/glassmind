// SPDX-License-Identifier: MIT
//! Bounded, learned basic dependencies. Predicted syntax is never semantic proof.
// All indexing below is into dimensions validated at construction/load, or parser
// states whose transitions preserve the documented token bounds.
#![allow(clippy::indexing_slicing)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::{Read, Write};

pub const VERSION: &str = "ewt-arc-standard-v1";
pub const MAX_TOKENS: usize = 128;
pub const MAX_BYTES: usize = 16 * 1024;
const TAG_BUCKETS: usize = 1 << 16;
const PARSE_BUCKETS: usize = 1 << 17;
const MAX_MODEL: u64 = 128 * 1024 * 1024;
type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub forms: Option<Vec<String>>,
    pub text: Option<String>,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct TokenSpan {
    pub form: String,
    pub start: usize,
    pub end: usize,
}
#[derive(Debug, Serialize, PartialEq)]
pub struct Prediction {
    pub status: String,
    pub model_version: String,
    pub tokenization: String,
    pub forms: Vec<String>,
    pub spans: Option<Vec<TokenSpan>>,
    pub upos: Vec<String>,
    /// Heads are 1-based token positions; zero denotes the unique root.
    pub heads: Vec<usize>,
    pub deprels: Vec<String>,
    pub structurally_valid: bool,
}
fn combining(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036f | 0x1ab0..=0x1aff | 0x1dc0..=0x1dff | 0x20d0..=0x20ff | 0xfe20..=0xfe2f)
}
fn apostrophe(c: char) -> bool {
    matches!(c, '\'' | '’')
}
fn push_span(text: &str, start: usize, end: usize, out: &mut Vec<TokenSpan>) {
    out.push(TokenSpan {
        form: text[start..end].into(),
        start,
        end,
    });
}
/// Split only productive terminal clitics, retaining source spelling. Working
/// backwards also handles compound clitics without a lexical exception table.
fn push_word(text: &str, start: usize, mut end: usize, out: &mut Vec<TokenSpan>) {
    let mut suffixes = Vec::new();
    while let Some((offset, mark)) = text[start..end]
        .char_indices()
        .rfind(|(_, c)| apostrophe(*c))
    {
        let pos = start + offset;
        let tail = &text[pos + mark.len_utf8()..end];
        let cut = if tail.eq_ignore_ascii_case("t")
            && offset > 1
            && matches!(text.as_bytes()[pos - 1], b'n' | b'N')
        {
            pos - 1
        } else if offset > 0
            && ["s", "m", "d", "ll", "re", "ve"]
                .iter()
                .any(|s| tail.eq_ignore_ascii_case(s))
        {
            pos
        } else {
            break;
        };
        suffixes.push((cut, end));
        end = cut;
    }
    push_span(text, start, end, out);
    for (s, e) in suffixes.into_iter().rev() {
        push_span(text, s, e, out);
    }
}
/// Bounded orthographic segmentation with exact UTF8 source slices. Internal
/// apostrophes join letters; terminal English clitics are separate words.
/// Outer quotes and other punctuation remain individual scalars. This does not
/// expand missing apostrophes, normalize words, or resolve lexical ambiguity.
pub fn tokenize(text: &str) -> Result<Vec<TokenSpan>> {
    if text.len() > MAX_BYTES {
        return Err("text exceeds 16KiB".into());
    }
    let mut out = Vec::new();
    let mut start = None;
    let mut letter_tail = false;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let internal = apostrophe(c)
            && letter_tail
            && chars.peek().is_some_and(|(_, next)| next.is_alphabetic());
        if c.is_alphanumeric() || combining(c) || internal {
            if start.is_none() {
                start = Some(i);
            }
            if !combining(c) {
                letter_tail = c.is_alphabetic();
            }
        } else {
            if let Some(s) = start.take() {
                push_word(text, s, i, &mut out);
            }
            letter_tail = false;
            if !c.is_whitespace() {
                push_span(text, i, i + c.len_utf8(), &mut out);
            }
        }
    }
    if let Some(s) = start {
        push_word(text, s, text.len(), &mut out);
    }
    validate_forms(&out.iter().map(|t| t.form.clone()).collect::<Vec<_>>())?;
    Ok(out)
}
fn validate_forms(forms: &[String]) -> Result<()> {
    if forms.is_empty() || forms.len() > MAX_TOKENS {
        return Err("requires 1..128 tokens".into());
    }
    if forms.iter().map(String::len).sum::<usize>() > MAX_BYTES {
        return Err("forms exceed 16KiB".into());
    }
    if forms.iter().any(|s| {
        s.is_empty() || s.chars().any(char::is_whitespace) || s.chars().any(char::is_control)
    }) {
        return Err("empty, whitespace or control in form".into());
    }
    Ok(())
}
/// Independent structural validation; this does not validate linguistic accuracy.
pub fn valid_tree(heads: &[usize], labels: &[String]) -> bool {
    let n = heads.len();
    if n == 0
        || n > MAX_TOKENS
        || labels.len() != n
        || heads.iter().filter(|&&h| h == 0).count() != 1
    {
        return false;
    }
    for (i, (&h, l)) in heads.iter().zip(labels).enumerate() {
        if h > n || h == i + 1 || (h == 0) != (l == "root") || l.is_empty() {
            return false;
        }
        let mut a = i + 1;
        for _ in 0..n {
            if a == 0 {
                break;
            }
            a = heads[a - 1];
            if a > n {
                return false;
            }
        }
        if a != 0 {
            return false;
        }
    }
    true
}
fn hash(s: &str) -> u64 {
    s.bytes().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}
fn feature(out: &mut Vec<usize>, buckets: usize, slot: u64, vals: &[u64]) {
    let mut h = slot.wrapping_mul(0x9e3779b97f4a7c15);
    for &v in vals {
        h = (h ^ v).wrapping_mul(0x100000001b3);
        h ^= h >> 32;
    }
    out.push(h as usize & (buckets - 1));
}
#[derive(Clone)]
struct Word {
    word: u64,
    shape: u64,
    affixes: Vec<u64>,
}
fn words(forms: &[String]) -> Vec<Word> {
    forms
        .iter()
        .map(|s| {
            let low = s.to_lowercase();
            let cs: Vec<char> = low.chars().collect();
            let mut affixes = Vec::new();
            for n in 1..=4 {
                let n = n.min(cs.len());
                affixes.push(hash(&cs[..n].iter().collect::<String>()));
                affixes.push(hash(&cs[cs.len() - n..].iter().collect::<String>()));
            }
            let shape = u64::from(s.chars().next().is_some_and(char::is_uppercase))
                | (u64::from(s.chars().all(char::is_uppercase)) << 1)
                | (u64::from(s.chars().any(char::is_numeric)) << 2)
                | (u64::from(s.contains('-')) << 3);
            Word {
                word: hash(&low),
                shape,
                affixes,
            }
        })
        .collect()
}
fn tag_features(ws: &[Word], i: usize, tags: &[usize]) -> Vec<usize> {
    let mut f = Vec::with_capacity(24);
    feature(&mut f, TAG_BUCKETS, 0, &[]);
    for (slot, off) in (-2isize..=2).enumerate() {
        let w = i
            .checked_add_signed(off)
            .and_then(|j| ws.get(j))
            .map_or(0, |w| w.word);
        feature(&mut f, TAG_BUCKETS, 1 + slot as u64, &[w]);
    }
    feature(&mut f, TAG_BUCKETS, 6, &[ws[i].shape]);
    for (a, v) in ws[i].affixes.iter().enumerate() {
        feature(&mut f, TAG_BUCKETS, 7 + a as u64, &[*v]);
    }
    let p = tags.last().map_or(0, |t| *t as u64 + 1);
    let pp = tags.len().checked_sub(2).map_or(0, |j| tags[j] as u64 + 1);
    feature(&mut f, TAG_BUCKETS, 15, &[p]);
    feature(&mut f, TAG_BUCKETS, 16, &[pp, p]);
    feature(&mut f, TAG_BUCKETS, 17, &[p, ws[i].word]);
    feature(&mut f, TAG_BUCKETS, 18, &[p, ws[i].affixes[5]]);
    f
}
struct Table {
    classes: usize,
    buckets: usize,
    weights: Vec<f32>,
}
impl Table {
    fn new(classes: usize, buckets: usize) -> Self {
        Self {
            classes,
            buckets,
            weights: vec![0.; classes * buckets],
        }
    }
    fn scores(&self, features: &[usize]) -> Vec<f32> {
        let mut scores = vec![0.; self.classes];
        for &f in features {
            for (s, w) in scores
                .iter_mut()
                .zip(&self.weights[f * self.classes..(f + 1) * self.classes])
            {
                *s += w;
            }
        }
        scores
    }
}
struct Learner {
    table: Table,
    totals: Vec<f64>,
    last: Vec<u32>,
    step: u32,
}
impl Learner {
    fn new(c: usize, b: usize) -> Self {
        Self {
            table: Table::new(c, b),
            totals: vec![0.; c * b],
            last: vec![0; c * b],
            step: 0,
        }
    }
    fn update(&mut self, f: &[usize], gold: usize, pred: usize) {
        self.step += 1;
        if gold == pred {
            return;
        }
        for &feat in f {
            for (c, d) in [(gold, 1.), (pred, -1.)] {
                let i = feat * self.table.classes + c;
                self.totals[i] +=
                    f64::from(self.step - self.last[i]) * f64::from(self.table.weights[i]);
                self.last[i] = self.step;
                self.table.weights[i] += d;
            }
        }
    }
    fn finish(mut self) -> Table {
        for i in 0..self.table.weights.len() {
            self.totals[i] +=
                f64::from(self.step - self.last[i]) * f64::from(self.table.weights[i]);
            self.table.weights[i] = (self.totals[i] / f64::from(self.step.max(1))) as f32;
        }
        self.table
    }
}
fn best(scores: &[f32], legal: impl Fn(usize) -> bool) -> Result<usize> {
    let mut b = None;
    let mut val = f32::NEG_INFINITY;
    for (i, &s) in scores.iter().enumerate() {
        if !s.is_finite() {
            return Err("nonfinite score".into());
        }
        if legal(i) && (b.is_none() || s > val) {
            b = Some(i);
            val = s;
        }
    }
    b.ok_or_else(|| "no legal transition".into())
}
struct State {
    stack: Vec<usize>,
    next: usize,
    heads: Vec<usize>,
    labels: Vec<usize>,
    left: Vec<usize>,
    right: Vec<usize>,
    valency: Vec<usize>,
}
impl State {
    fn new(n: usize) -> Self {
        Self {
            stack: vec![0],
            next: 1,
            heads: vec![0; n + 1],
            labels: vec![0; n + 1],
            left: vec![0; n + 1],
            right: vec![0; n + 1],
            valency: vec![0; n + 1],
        }
    }
    fn legal(&self, a: usize, n: usize, l: usize, root: usize) -> bool {
        if a == 0 {
            return self.next <= n;
        }
        if self.stack.len() < 2 {
            return false;
        }
        let left = a <= l;
        let lab = (a - 1) % l;
        let h = self.stack[self.stack.len() - if left { 1 } else { 2 }];
        let d = self.stack[self.stack.len() - if left { 2 } else { 1 }];
        if d == 0 {
            return false;
        }
        if h == 0 {
            return !left && self.next > n && self.stack.len() == 2 && lab == root;
        }
        lab != root
    }
    fn apply(&mut self, a: usize, l: usize) {
        if a == 0 {
            self.stack.push(self.next);
            self.next += 1;
            return;
        }
        let left = a <= l;
        let z = self.stack.len();
        let h = self.stack[z - if left { 1 } else { 2 }];
        let d = self.stack[z - if left { 2 } else { 1 }];
        self.heads[d] = h;
        self.labels[d] = (a - 1) % l;
        if d < h {
            if self.left[h] == 0 || d < self.left[h] {
                self.left[h] = d;
            }
        } else if d > self.right[h] {
            self.right[h] = d;
        }
        self.valency[h] += 1;
        self.stack.remove(z - if left { 2 } else { 1 });
    }
}
fn parse_features(s: &State, ws: &[Word], tags: &[usize]) -> Vec<usize> {
    let n = ws.len();
    let mut f = Vec::with_capacity(100);
    feature(&mut f, PARSE_BUCKETS, 0, &[]);
    let idx = [
        s.stack.last().copied().unwrap_or(0),
        s.stack.len().checked_sub(2).map_or(0, |i| s.stack[i]),
        s.stack.len().checked_sub(3).map_or(0, |i| s.stack[i]),
        s.next,
        s.next + 1,
        s.next + 2,
    ];
    let word = |i: usize| {
        if i == 0 {
            1
        } else {
            ws.get(i - 1).map_or(0, |w| w.word)
        }
    };
    let tag = |i: usize| {
        if i == 0 {
            1
        } else {
            tags.get(i - 1).map_or(0, |t| *t as u64 + 2)
        }
    };
    for (p, &i) in idx.iter().enumerate() {
        feature(&mut f, PARSE_BUCKETS, 1 + p as u64 * 3, &[word(i)]);
        feature(&mut f, PARSE_BUCKETS, 2 + p as u64 * 3, &[tag(i)]);
        feature(&mut f, PARSE_BUCKETS, 3 + p as u64 * 3, &[word(i), tag(i)]);
    }
    for (p, (a, b)) in [(0, 1), (0, 3), (1, 3), (3, 4)].iter().enumerate() {
        for (k, v) in [
            [tag(idx[*a]), tag(idx[*b])],
            [word(idx[*a]), word(idx[*b])],
            [word(idx[*a]), tag(idx[*b])],
            [tag(idx[*a]), word(idx[*b])],
        ]
        .iter()
        .enumerate()
        {
            feature(&mut f, PARSE_BUCKETS, 20 + (p * 4 + k) as u64, v);
        }
    }
    for (p, (a, b, c)) in [(0, 1, 3), (0, 3, 4), (1, 2, 0), (3, 4, 5)]
        .iter()
        .enumerate()
    {
        feature(
            &mut f,
            PARSE_BUCKETS,
            40 + p as u64,
            &[tag(idx[*a]), tag(idx[*b]), tag(idx[*c])],
        );
    }
    let dist = idx[0].abs_diff(idx[1]).min(5) as u64;
    feature(&mut f, PARSE_BUCKETS, 45, &[tag(idx[0]), tag(idx[1]), dist]);
    feature(&mut f, PARSE_BUCKETS, 46, &[word(idx[0]), dist]);
    feature(&mut f, PARSE_BUCKETS, 47, &[word(idx[1]), dist]);
    for (p, &i) in idx[..2].iter().enumerate() {
        if i > n {
            continue;
        }
        feature(
            &mut f,
            PARSE_BUCKETS,
            50 + p as u64,
            &[tag(i), s.valency[i].min(5) as u64],
        );
        for (side, child) in [s.left[i], s.right[i]].iter().copied().enumerate() {
            let base = 60 + (p * 12 + side * 6) as u64;
            let lab = if child == 0 {
                0
            } else {
                s.labels[child] as u64 + 1
            };
            feature(&mut f, PARSE_BUCKETS, base, &[word(child)]);
            feature(&mut f, PARSE_BUCKETS, base + 1, &[tag(child)]);
            feature(&mut f, PARSE_BUCKETS, base + 2, &[lab]);
            feature(&mut f, PARSE_BUCKETS, base + 3, &[tag(i), tag(child), lab]);
            feature(
                &mut f,
                PARSE_BUCKETS,
                base + 4,
                &[tag(idx[0]), tag(idx[1]), tag(child)],
            );
            feature(&mut f, PARSE_BUCKETS, base + 5, &[word(i), tag(child)]);
        }
    }
    f
}

pub struct Model {
    tags: Vec<String>,
    labels: Vec<String>,
    tagger: Table,
    parser: Table,
    root: usize,
}
impl Model {
    fn tag(&self, ws: &[Word]) -> Result<Vec<usize>> {
        let mut tags = Vec::new();
        for i in 0..ws.len() {
            let f = tag_features(ws, i, &tags);
            tags.push(best(&self.tagger.scores(&f), |_| true)?);
        }
        Ok(tags)
    }
    /// Only token forms are accepted. Positions are assigned here, never supplied.
    pub fn infer(&self, forms: &[String]) -> Result<Prediction> {
        validate_forms(forms)?;
        let n = forms.len();
        let ws = words(forms);
        let tags = self.tag(&ws)?;
        let mut s = State::new(n);
        for _ in 0..2 * n {
            let f = parse_features(&s, &ws, &tags);
            let a = best(&self.parser.scores(&f), |a| {
                s.legal(a, n, self.labels.len(), self.root)
            })?;
            s.apply(a, self.labels.len());
        }
        if s.next != n + 1 || s.stack != [0] {
            return Err("incomplete transitions".into());
        }
        let heads = s.heads[1..].to_vec();
        let deprels = s.labels[1..]
            .iter()
            .map(|&l| self.labels[l].clone())
            .collect::<Vec<_>>();
        if !valid_tree(&heads, &deprels) {
            return Err("invalid predicted structure".into());
        }
        Ok(Prediction {
            status: "predicted".into(),
            model_version: VERSION.into(),
            tokenization: "provided_forms".into(),
            forms: forms.to_vec(),
            spans: None,
            upos: tags.iter().map(|&t| self.tags[t].clone()).collect(),
            heads,
            deprels,
            structurally_valid: true,
        })
    }
    pub fn predict(&self, input: Input) -> Result<Prediction> {
        match (input.forms, input.text) {
            (Some(forms), None) => self.infer(&forms),
            (None, Some(text)) => {
                let spans = tokenize(&text)?;
                let mut p =
                    self.infer(&spans.iter().map(|t| t.form.clone()).collect::<Vec<_>>())?;
                p.tokenization = "raw_unicode_apostrophe_v2".into();
                p.spans = Some(spans);
                Ok(p)
            }
            _ => Err("exactly one of forms or text required".into()),
        }
    }
    pub fn save(&self, mut w: impl Write) -> Result<()> {
        let header = Header {
            version: VERSION.into(),
            tags: self.tags.clone(),
            labels: self.labels.clone(),
        };
        let bytes = serde_json::to_vec(&header).map_err(|e| e.to_string())?;
        let nonzero = self
            .tagger
            .weights
            .iter()
            .chain(&self.parser.weights)
            .filter(|&&v| v != 0.)
            .count();
        if bytes.len() as u64 + 36 + nonzero as u64 * 8 > MAX_MODEL {
            return Err("model exceeds 128MiB".into());
        }
        w.write_all(b"GMSYN001").map_err(|e| e.to_string())?;
        put(&mut w, bytes.len() as u32)?;
        w.write_all(&bytes).map_err(|e| e.to_string())?;
        for t in [&self.tagger, &self.parser] {
            put(&mut w, t.buckets as u32)?;
            put(&mut w, t.classes as u32)?;
            put(
                &mut w,
                t.weights.iter().filter(|&&v| v != 0.).count() as u32,
            )?;
            for (i, &v) in t.weights.iter().enumerate() {
                if v != 0. {
                    put(&mut w, i as u32)?;
                    put(&mut w, v.to_bits())?;
                }
            }
        }
        Ok(())
    }
    /// Bounded binary loader. Call with a buffered reader for file input.
    pub fn load(reader: impl Read) -> Result<Self> {
        let mut r = reader.take(MAX_MODEL + 1);
        let mut magic = [0; 8];
        r.read_exact(&mut magic).map_err(|e| e.to_string())?;
        if &magic != b"GMSYN001" {
            return Err("unknown syntax model magic/version".into());
        }
        let len = get(&mut r)? as usize;
        if len > 8192 {
            return Err("header too large".into());
        }
        let mut bytes = vec![0; len];
        r.read_exact(&mut bytes).map_err(|e| e.to_string())?;
        let h: Header = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if h.version != VERSION {
            return Err("unknown model version".into());
        }
        for inventory in [&h.tags, &h.labels] {
            if inventory.is_empty()
                || inventory.len() > 64
                || inventory.iter().any(|v| {
                    v.is_empty()
                        || v.len() > 64
                        || !v
                            .bytes()
                            .all(|c| c.is_ascii_alphanumeric() || b":_-".contains(&c))
                })
                || inventory.windows(2).any(|p| p[0] >= p[1])
            {
                return Err("invalid sorted inventory".into());
            }
        }
        let root = h
            .labels
            .iter()
            .position(|l| l == "root")
            .ok_or("missing root label")?;
        let mut read_table = |classes: usize, buckets: usize| -> Result<Table> {
            if get(&mut r)? as usize != buckets || get(&mut r)? as usize != classes {
                return Err("table dimensions mismatch".into());
            }
            let count = get(&mut r)? as usize;
            if count > classes * buckets || count as u64 * 8 > MAX_MODEL {
                return Err("oversized sparse table".into());
            }
            let mut t = Table::new(classes, buckets);
            let mut prev = None;
            for _ in 0..count {
                let i = get(&mut r)? as usize;
                let v = f32::from_bits(get(&mut r)?);
                if i >= t.weights.len()
                    || prev.is_some_and(|p| i <= p)
                    || !v.is_finite()
                    || v == 0.
                    || v.abs() > 1_000_000.
                {
                    return Err("invalid sparse weight".into());
                }
                t.weights[i] = v;
                prev = Some(i);
            }
            Ok(t)
        };
        let tagger = read_table(h.tags.len(), TAG_BUCKETS)?;
        let parser = read_table(1 + 2 * h.labels.len(), PARSE_BUCKETS)?;
        let mut extra = [0];
        if r.read(&mut extra).map_err(|e| e.to_string())? != 0 || r.limit() == 0 {
            return Err("trailing or oversized model".into());
        }
        Ok(Self {
            tags: h.tags,
            labels: h.labels,
            tagger,
            parser,
            root,
        })
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    version: String,
    tags: Vec<String>,
    labels: Vec<String>,
}
fn put(w: &mut impl Write, n: u32) -> Result<()> {
    w.write_all(&n.to_le_bytes()).map_err(|e| e.to_string())
}
fn get(r: &mut impl Read) -> Result<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b).map_err(|e| e.to_string())?;
    Ok(u32::from_le_bytes(b))
}

/// Training-only annotations. Never accepted by `infer` or JSON runtime input.
pub struct TrainingSentence {
    pub forms: Vec<String>,
    pub upos: Vec<String>,
    pub heads: Vec<usize>,
    pub labels: Vec<String>,
}
#[derive(Default, Serialize)]
pub struct TrainingReport {
    pub sentences: usize,
    pub words: usize,
    pub input_skips: usize,
    pub invalid_tree_skips: usize,
    pub nonprojective_skips: usize,
    pub parser_sentences: usize,
    pub tag_epochs: usize,
    pub parser_epochs: usize,
    pub tag_mistakes: Vec<usize>,
    pub parser_mistakes: Vec<usize>,
}
fn projective(heads: &[usize]) -> bool {
    for (i, &h) in heads.iter().enumerate() {
        for k in (i + 1).min(h) + 1..(i + 1).max(h) {
            let mut a = k;
            for _ in 0..heads.len() {
                if a == 0 || a == h {
                    break;
                }
                a = heads[a - 1];
            }
            if a != h {
                return false;
            }
        }
    }
    true
}
fn oracle(s: &State, g: &TrainingSentence, labs: &[usize], nlabels: usize) -> Result<usize> {
    if s.stack.len() >= 2 {
        let z = s.stack.len();
        let a = s.stack[z - 2];
        let b = s.stack[z - 1];
        let complete = |d: usize| {
            g.heads
                .iter()
                .enumerate()
                .all(|(i, &h)| h != d || s.heads[i + 1] != 0)
        };
        if a != 0 && g.heads[a - 1] == b && complete(a) {
            return Ok(1 + labs[a - 1]);
        }
        if b != 0 && g.heads[b - 1] == a && complete(b) {
            return Ok(1 + nlabels + labs[b - 1]);
        }
    }
    if s.next <= g.forms.len() {
        Ok(0)
    } else {
        Err("static oracle stuck".into())
    }
}
pub fn train(data: &[TrainingSentence]) -> Result<(Model, TrainingReport)> {
    if data.is_empty() || data.len() > 100_000 {
        return Err("training sentence limit".into());
    }
    let tags: Vec<String> = data
        .iter()
        .flat_map(|s| s.upos.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let labels: Vec<String> = data
        .iter()
        .flat_map(|s| s.labels.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if tags.is_empty() || tags.len() > 64 || labels.is_empty() || labels.len() > 64 {
        return Err("training inventory limit".into());
    }
    let root = labels
        .iter()
        .position(|l| l == "root")
        .ok_or("missing root")?;
    let mut report = TrainingReport {
        sentences: data.len(),
        words: data.iter().map(|s| s.forms.len()).sum(),
        tag_epochs: 2,
        parser_epochs: 3,
        ..Default::default()
    };
    let mut usable = Vec::new();
    let mut eligible = Vec::new();
    for (i, s) in data.iter().enumerate() {
        if s.forms.len() != s.upos.len()
            || s.forms.len() != s.heads.len()
            || s.forms.len() != s.labels.len()
        {
            return Err("training annotation lengths".into());
        }
        if validate_forms(&s.forms).is_err() {
            report.input_skips += 1;
            continue;
        }
        usable.push(i);
        if !valid_tree(&s.heads, &s.labels) {
            report.invalid_tree_skips += 1;
            continue;
        }
        if !projective(&s.heads) {
            report.nonprojective_skips += 1;
            continue;
        }
        eligible.push(i);
    }
    report.parser_sentences = eligible.len();
    let cached: Vec<Vec<Word>> = data.iter().map(|s| words(&s.forms)).collect();
    let mut tagger = Learner::new(tags.len(), TAG_BUCKETS);
    for epoch in 0..2 {
        let mut mistakes = 0;
        for &i in &usable {
            let mut predtags = Vec::new();
            for j in 0..data[i].forms.len() {
                let f = tag_features(&cached[i], j, &predtags);
                let pred = best(&tagger.table.scores(&f), |_| true)?;
                let gold = tags
                    .binary_search(&data[i].upos[j])
                    .map_err(|_| "unknown TRAIN tag")?;
                tagger.update(&f, gold, pred);
                mistakes += usize::from(gold != pred);
                predtags.push(pred);
            }
        }
        report.tag_mistakes.push(mistakes);
        eprintln!("tag epoch {} mistakes {}", epoch + 1, mistakes);
    }
    let mut model = Model {
        tags,
        labels,
        tagger: tagger.finish(),
        parser: Table::new(0, 0),
        root,
    };
    let predicted: Vec<Vec<usize>> = cached
        .iter()
        .map(|ws| model.tag(ws))
        .collect::<Result<_>>()?;
    let mut parser = Learner::new(1 + 2 * model.labels.len(), PARSE_BUCKETS);
    for epoch in 0..3 {
        let mut mistakes = 0;
        for &i in &eligible {
            let g = &data[i];
            let n = g.forms.len();
            let mut s = State::new(n);
            let labs = g
                .labels
                .iter()
                .map(|l| {
                    model
                        .labels
                        .binary_search(l)
                        .map_err(|_| "unknown TRAIN label".into())
                })
                .collect::<Result<Vec<_>>>()?;
            for _ in 0..2 * n {
                let f = parse_features(&s, &cached[i], &predicted[i]);
                let gold = oracle(&s, g, &labs, model.labels.len())?;
                if !s.legal(gold, n, model.labels.len(), root) {
                    return Err("illegal TRAIN oracle".into());
                }
                let pred = best(&parser.table.scores(&f), |a| {
                    s.legal(a, n, model.labels.len(), root)
                })?;
                parser.update(&f, gold, pred);
                mistakes += usize::from(gold != pred);
                s.apply(gold, model.labels.len());
            }
            if s.heads[1..] != g.heads || s.labels[1..] != labs {
                return Err("TRAIN oracle reconstruction failed".into());
            }
        }
        report.parser_mistakes.push(mistakes);
        eprintln!("parser epoch {} mistakes {}", epoch + 1, mistakes);
    }
    model.parser = parser.finish();
    Ok((model, report))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    fn tiny() -> Model {
        Model {
            tags: vec!["NOUN".into()],
            labels: vec!["dep".into(), "root".into()],
            tagger: Table::new(1, TAG_BUCKETS),
            parser: Table::new(5, PARSE_BUCKETS),
            root: 1,
        }
    }
    #[test]
    fn strict_inputs_and_unicode_spans() {
        assert!(serde_json::from_str::<Input>(r#"{"forms":["cat"],"heads":[0]}"#).is_err());
        assert!(serde_json::from_str::<Input>(r#"{"text":"cat","id":"gold"}"#).is_err());
        let text = "Élodie can't—go! e\u{301}";
        let ts = tokenize(text).unwrap();
        assert_eq!(
            ts.iter().map(|t| t.form.as_str()).collect::<Vec<_>>(),
            ["Élodie", "ca", "n't", "—", "go", "!", "e\u{301}"]
        );
        for t in ts {
            assert_eq!(&text[t.start..t.end], t.form);
        }
        assert!(tokenize(&"x".repeat(MAX_BYTES + 1)).is_err());
        assert!(tokenize("").is_err());
        assert!(tiny().infer(&vec!["x".into(); 129]).is_err());
    }
    fn assert_source_coverage(text: &str, spans: &[TokenSpan]) {
        let mut end = 0;
        for span in spans {
            assert!(span.start >= end && span.end > span.start);
            assert!(text[end..span.start].chars().all(char::is_whitespace));
            assert_eq!(&text[span.start..span.end], span.form);
            assert!(!span.form.chars().any(char::is_whitespace));
            end = span.end;
        }
        assert!(text[end..].chars().all(char::is_whitespace));
    }
    #[test]
    fn raw_apostrophe_frozen_contrasts() {
        for line in include_str!("../../../tools/reading/raw-syntax/authored.jsonl").lines() {
            let input: Input = serde_json::from_str(line).unwrap();
            let text = input.text.unwrap();
            let spans = tokenize(&text).unwrap();
            assert_eq!(
                spans.iter().map(|t| t.form.clone()).collect::<Vec<_>>(),
                input.forms.unwrap(),
                "{text}"
            );
            assert_source_coverage(&text, &spans);
        }
    }
    #[test]
    fn raw_apostrophe_bounds_and_controls() {
        for (text, expected) in [
            ("x ".repeat(128), 128),
            ("won't ".repeat(64), 128),
            ("é".repeat(MAX_BYTES / 2), 1),
        ] {
            let spans = tokenize(&text).unwrap();
            assert_eq!(spans.len(), expected);
            assert_source_coverage(&text, &spans);
        }
        for text in [
            "x ".repeat(129),
            "won't ".repeat(65),
            "é".repeat(MAX_BYTES / 2 + 1),
            "a\0b".into(),
            "a\u{7f}b".into(),
            "a\u{80}b".into(),
            " \t\n".into(),
        ] {
            assert!(tokenize(&text).is_err());
        }
        // Boundary adversaries: marks without a base, multibyte letters, quotes,
        // whitespace, numbers and compound punctuation in every short ordering.
        let atoms = ["é", "\u{301}", "'", "’", " ", "7", "n", "t", "—"];
        for a in atoms {
            for b in atoms {
                for c in atoms {
                    let text = format!("{a}{b}{c}");
                    if let Ok(spans) = tokenize(&text) {
                        assert_source_coverage(&text, &spans);
                    } else {
                        assert!(text.chars().all(char::is_whitespace));
                    }
                }
            }
        }
    }
    #[test]
    fn roundtrip_and_determinism() {
        let m = tiny();
        let forms = vec!["one".into(), "two".into(), "three".into()];
        let p = m.infer(&forms).unwrap();
        assert!(valid_tree(&p.heads, &p.deprels));
        let mut bytes = Vec::new();
        m.save(&mut bytes).unwrap();
        let loaded = Model::load(bytes.as_slice()).unwrap();
        assert_eq!(p, loaded.infer(&forms).unwrap());
        let mut again = Vec::new();
        loaded.save(&mut again).unwrap();
        assert_eq!(bytes, again);
        for n in [0, 7, 12, bytes.len() - 1] {
            assert!(Model::load(&bytes[..n]).is_err());
        }
        bytes.push(0);
        assert!(Model::load(bytes.as_slice()).is_err());
    }
    #[test]
    fn corrupt_weights_and_dimensions() {
        let mut m = tiny();
        m.parser.weights[0] = 1.;
        let mut b = Vec::new();
        m.save(&mut b).unwrap();
        let n = b.len();
        b[n - 4..].copy_from_slice(&f32::NAN.to_bits().to_le_bytes());
        assert!(Model::load(b.as_slice()).is_err());
        let mut b = Vec::new();
        m.save(&mut b).unwrap();
        let len = u32::from_le_bytes(b[8..12].try_into().unwrap()) as usize;
        b[12 + len..16 + len].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Model::load(b.as_slice()).is_err());
    }
    #[test]
    fn oracle_and_invalid_structures() {
        let g = TrainingSentence {
            forms: vec!["Birds".into(), "eat".into(), "seeds".into()],
            upos: vec!["NOUN".into(), "VERB".into(), "NOUN".into()],
            heads: vec![2, 0, 2],
            labels: vec!["nsubj".into(), "root".into(), "obj".into()],
        };
        let mut s = State::new(3);
        let labs = [0, 2, 1];
        for _ in 0..6 {
            let a = oracle(&s, &g, &labs, 3).unwrap();
            assert!(s.legal(a, 3, 3, 2));
            s.apply(a, 3);
        }
        assert_eq!(&s.heads[1..], &g.heads);
        assert!(projective(&g.heads));
        assert!(!projective(&[3, 4, 0, 3]));
        assert!(!valid_tree(&[0, 0], &["root".into(), "root".into()]));
        assert!(!valid_tree(
            &[0, 3, 2],
            &["root".into(), "dep".into(), "dep".into()]
        ));
        assert!(!valid_tree(&[1], &["root".into()]));
    }
}
