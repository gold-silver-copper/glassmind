// SPDX-License-Identifier: MIT OR Apache-2.0
//! Fixed TRAIN-vocabulary word trigram ranker; no generation or byte likelihood.
use std::collections::HashMap;
use std::io::{self, Read, Write};

const EOS: u32 = 0;
const UNK: u32 = 1;
const BOS: u32 = 2;
const NONE: u32 = u32::MAX;
const MAGIC: &[u8; 8] = b"GMWRD003";
const MAX_BYTES: u64 = 150 * 1024 * 1024;

type Key = [u32; 3];

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Segmentation precedes lowercasing; punctuation is one Unicode scalar per token.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() {
            word.extend(c.to_lowercase());
        } else {
            if !word.is_empty() {
                tokens.push(std::mem::take(&mut word));
            }
            if !c.is_whitespace() {
                tokens.push(c.to_string());
            }
        }
    }
    if !word.is_empty() {
        tokens.push(word);
    }
    tokens
}

pub struct Model {
    vocabulary: Vec<String>,
    ids: HashMap<String, u32>,
    counts: HashMap<Key, u64>,
    contexts: HashMap<[u32; 2], (u64, u64)>,
    pub documents: u64,
    pub tokens: u64,
}

impl Model {
    /// Frequencies must come only from TRAIN, using `tokenize`.
    pub fn from_frequencies(frequencies: HashMap<String, u64>) -> io::Result<Self> {
        let mut vocabulary: Vec<_> = frequencies
            .into_iter()
            .filter_map(|(token, n)| (n >= 2).then_some(token))
            .collect();
        vocabulary.sort_unstable();
        Self::with_vocabulary(vocabulary)
    }

    fn with_vocabulary(vocabulary: Vec<String>) -> io::Result<Self> {
        if vocabulary.len() > (u32::MAX - 3) as usize {
            return Err(invalid("vocabulary exceeds ID space"));
        }
        let ids = vocabulary
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i as u32 + 3))
            .collect();
        Ok(Self {
            vocabulary,
            ids,
            counts: HashMap::new(),
            contexts: HashMap::new(),
            documents: 0,
            tokens: 0,
        })
    }

    fn symbols(&self, text: &str) -> Vec<u32> {
        tokenize(text)
            .iter()
            .map(|t| self.ids.get(t).copied().unwrap_or(UNK))
            .collect()
    }

    pub fn observe(&mut self, text: &str) -> io::Result<()> {
        let symbols = self.symbols(text);
        self.tokens = self
            .tokens
            .checked_add(symbols.len() as u64)
            .ok_or_else(|| invalid("token count overflow"))?;
        self.documents = self
            .documents
            .checked_add(1)
            .ok_or_else(|| invalid("document count overflow"))?;
        let mut history = [BOS, BOS];
        for symbol in symbols.into_iter().chain([EOS]) {
            let [a, b] = history;
            for key in [[NONE, NONE, symbol], [NONE, b, symbol], [a, b, symbol]] {
                let count = self.counts.entry(key).or_default();
                *count = count
                    .checked_add(1)
                    .ok_or_else(|| invalid("count overflow"))?;
            }
            history = [b, symbol];
        }
        Ok(())
    }

    pub fn finish(&mut self) -> io::Result<()> {
        self.contexts.clear();
        let mut totals = [0u64; 3];
        for (&[a, b, _], &count) in &self.counts {
            let (n, types) = self.contexts.entry([a, b]).or_default();
            *n = n
                .checked_add(count)
                .ok_or_else(|| invalid("context overflow"))?;
            *types += 1;
            let order = if b == NONE {
                0
            } else if a == NONE {
                1
            } else {
                2
            };
            let total = totals
                .get_mut(order)
                .ok_or_else(|| invalid("invalid order"))?;
            *total = total
                .checked_add(count)
                .ok_or_else(|| invalid("total overflow"))?;
        }
        let events = self
            .tokens
            .checked_add(self.documents)
            .ok_or_else(|| invalid("size overflow"))?;
        if totals != [events; 3]
            || self.counts.get(&[NONE, NONE, EOS]).copied().unwrap_or(0) != self.documents
        {
            return Err(invalid("ngram totals or EOS disagree with corpus sizes"));
        }
        Ok(())
    }

    pub fn vocabulary_size(&self) -> usize {
        self.vocabulary.len()
    }
    pub fn entries(&self) -> usize {
        self.counts.len()
    }

    fn probability(&self, [a, b]: [u32; 2], symbol: u32) -> f64 {
        let unigram = self.counts.get(&[NONE, NONE, symbol]).copied().unwrap_or(0);
        let mut p = (unigram as f64 + 0.5)
            / ((self.tokens + self.documents) as f64 + 0.5 * (self.vocabulary.len() + 2) as f64);
        for [x, y] in [[NONE, b], [a, b]] {
            if let Some(&(n, types)) = self.contexts.get(&[x, y]) {
                let count = self.counts.get(&[x, y, symbol]).copied().unwrap_or(0);
                p = (count as f64 + types as f64 * p) / (n as f64 + types as f64);
            }
        }
        p
    }

    /// Total token-sequence negative log2 probability including one EOS.
    /// Case, whitespace and unknown spellings are lost: this is NOT bits/byte.
    pub fn bits(&self, text: &str) -> f64 {
        let mut history = [BOS, BOS];
        let mut bits = 0.0;
        for symbol in self.symbols(text).into_iter().chain([EOS]) {
            bits -= self.probability(history, symbol).log2();
            let [_, b] = history;
            history = [b, symbol];
        }
        bits
    }

    /// Sorted UTF-8 vocabulary and sorted (three u32 IDs, u64 count) entries.
    pub fn save(&self, mut out: impl Write) -> io::Result<()> {
        let size = 40
            + self
                .vocabulary
                .iter()
                .map(|s| 4 + s.len() as u64)
                .sum::<u64>()
            + 20 * self.counts.len() as u64;
        if size > MAX_BYTES {
            return Err(invalid("model exceeds 150 MiB"));
        }
        out.write_all(MAGIC)?;
        for n in [
            self.documents,
            self.tokens,
            self.vocabulary.len() as u64,
            self.counts.len() as u64,
        ] {
            out.write_all(&n.to_le_bytes())?;
        }
        for token in &self.vocabulary {
            let length = u32::try_from(token.len()).map_err(|_| invalid("token too long"))?;
            out.write_all(&length.to_le_bytes())?;
            out.write_all(token.as_bytes())?;
        }
        let mut entries: Vec<_> = self.counts.iter().collect();
        entries.sort_unstable_by_key(|&(key, _)| *key);
        for (key, count) in entries {
            for id in key {
                out.write_all(&id.to_le_bytes())?;
            }
            out.write_all(&count.to_le_bytes())?;
        }
        out.flush()
    }

    pub fn load(input: impl Read) -> io::Result<Self> {
        let mut bytes = Vec::new();
        input.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(invalid("model exceeds 150 MiB"));
        }
        let mut input = bytes.as_slice();
        fn array<const N: usize>(input: &mut &[u8]) -> io::Result<[u8; N]> {
            let mut out = [0; N];
            input.read_exact(&mut out)?;
            Ok(out)
        }
        fn u64_in(input: &mut &[u8]) -> io::Result<u64> {
            Ok(u64::from_le_bytes(array(input)?))
        }
        fn u32_in(input: &mut &[u8]) -> io::Result<u32> {
            Ok(u32::from_le_bytes(array(input)?))
        }
        if &array::<8>(&mut input)? != MAGIC {
            return Err(invalid("unsupported word model"));
        }
        let documents = u64_in(&mut input)?;
        let tokens = u64_in(&mut input)?;
        let vocab_len = u64_in(&mut input)?;
        let entries = u64_in(&mut input)?;
        if vocab_len > MAX_BYTES / 5 || entries > MAX_BYTES / 20 {
            return Err(invalid("impossible model lengths"));
        }
        let mut vocabulary = Vec::<String>::new();
        for _ in 0..vocab_len {
            let length = u32_in(&mut input)? as usize;
            if length == 0 || length > input.len() {
                return Err(invalid("invalid token length"));
            }
            let (token, rest) = input.split_at(length);
            let token =
                String::from_utf8(token.to_vec()).map_err(|_| invalid("invalid UTF-8 token"))?;
            if vocabulary.last().is_some_and(|p| p >= &token) {
                return Err(invalid("unsorted vocabulary"));
            }
            vocabulary.push(token);
            input = rest;
        }
        let mut model = Self::with_vocabulary(vocabulary)?;
        model.documents = documents;
        model.tokens = tokens;
        let mut previous = None;
        let valid_context = |x| x == BOS || x == UNK || (x >= 3 && u64::from(x) < vocab_len + 3);
        for _ in 0..entries {
            let key = [
                u32_in(&mut input)?,
                u32_in(&mut input)?,
                u32_in(&mut input)?,
            ];
            let [a, b, c] = key;
            let count = u64_in(&mut input)?;
            if previous.is_some_and(|p| p >= key)
                || count == 0
                || c == BOS
                || u64::from(c) >= vocab_len + 3
                || !(a == NONE || valid_context(a))
                || !(b == NONE || valid_context(b))
                || (b == NONE && a != NONE)
            {
                return Err(invalid("invalid or unsorted ngram"));
            }
            model.counts.insert(key, count);
            previous = Some(key);
        }
        if !input.is_empty() {
            return Err(invalid("trailing model data"));
        }
        model.finish()?;
        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn trained() -> io::Result<Model> {
        let docs = [
            "A cat can't jump!",
            "A dog can't jump!",
            "Élan 42.",
            "Élan 42.",
            "",
        ];
        let mut frequencies = HashMap::new();
        for doc in docs {
            for t in tokenize(doc) {
                *frequencies.entry(t).or_default() += 1;
            }
        }
        let mut m = Model::from_frequencies(frequencies)?;
        for doc in docs {
            m.observe(doc)?;
        }
        m.finish()?;
        Ok(m)
    }
    #[test]
    fn normalized_seen_and_unseen_contexts() -> io::Result<()> {
        let m = trained()?;
        for h in [[BOS, BOS], [UNK, UNK], [3, 4], [4, 3]] {
            let probabilities: Vec<_> = (0..m.vocabulary_size() as u32 + 3)
                .filter(|&x| x != BOS)
                .map(|x| m.probability(h, x))
                .collect();
            assert!(probabilities.iter().all(|&p| p > 0.0 && p <= 1.0));
            assert!((probabilities.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        }
        let empty = Model::from_frequencies(HashMap::new())?;
        assert_eq!(empty.bits(""), 1.0);
        Ok(())
    }
    #[test]
    fn unicode_punctuation_empty_and_oov() -> io::Result<()> {
        assert_eq!(
            tokenize("ÉLAN\tcan't—42! İ e\u{301}"),
            [
                "élan", "can", "'", "t", "—", "42", "!", "i\u{307}", "e", "\u{301}"
            ]
        );
        let m = trained()?;
        assert_eq!(m.bits("A CAT"), m.bits("a cat"));
        assert_eq!(m.bits("newword"), m.bits("别词"));
        assert_eq!(m.bits(""), m.bits("\t \n"));
        for s in ["", "\0", "¿未知?!", "ÉLAN 42."] {
            assert!(m.bits(s).is_finite());
        }
        assert!(!m.ids.contains_key("cat"));
        assert!(m.ids.contains_key("élan"));
        Ok(())
    }
    #[test]
    fn deterministic_roundtrip_and_malformed() -> io::Result<()> {
        let m = trained()?;
        let mut bytes = Vec::new();
        m.save(&mut bytes)?;
        let loaded = Model::load(bytes.as_slice())?;
        let mut again = Vec::new();
        loaded.save(&mut again)?;
        assert_eq!(bytes, again);
        let mut independent = Vec::new();
        trained()?.save(&mut independent)?;
        assert_eq!(bytes, independent);
        for s in ["A cat", "", "未知 😺!"] {
            assert_eq!(m.bits(s), loaded.bits(s));
        }
        for end in [0, 8, 39, bytes.len() - 1] {
            assert!(Model::load(bytes.get(..end).ok_or_else(|| invalid("test slice"))?).is_err());
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(Model::load(extra.as_slice()).is_err());
        let mut magic = bytes.clone();
        if let Some(b) = magic.first_mut() {
            *b = 0;
        }
        assert!(Model::load(magic.as_slice()).is_err());
        let mut zero_count = bytes;
        let length = zero_count.len();
        if let Some(last) = zero_count.get_mut(length - 8..) {
            last.fill(0);
        }
        assert!(Model::load(zero_count.as_slice()).is_err());
        Ok(())
    }
}
