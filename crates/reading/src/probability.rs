//! Fixed, non-generating byte 6-gram probability scorer.
use std::collections::HashMap;
use std::io::{self, Read, Write};

const ORDER: usize = 6;
const EOS: u16 = 256;
const BOS: u16 = 257;
const ALPHABET: f64 = 257.0;
const MAGIC: &[u8; 8] = b"GMBYTE06";

#[derive(Default)]
pub struct Model {
    counts: HashMap<u64, u64>,
    contexts: HashMap<u64, (u64, u64)>,
    pub documents: u64,
    pub bytes: u64,
}

// A leading 1 encodes context length, then each symbol occupies nine bits.
fn context(history: &[u16], length: usize) -> u64 {
    history
        .iter()
        .skip(history.len().saturating_sub(length))
        .fold(1, |key, &symbol| (key << 9) | u64::from(symbol))
}

fn advance(history: &mut Vec<u16>, symbol: u16) {
    history.push(symbol);
    if history.len() >= ORDER {
        history.remove(0);
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

impl Model {
    pub fn observe(&mut self, text: &[u8]) {
        let mut history = vec![BOS; ORDER - 1];
        for symbol in text.iter().map(|&b| u16::from(b)).chain([EOS]) {
            for length in 0..ORDER {
                let key = (context(&history, length) << 9) | u64::from(symbol);
                *self.counts.entry(key).or_default() += 1;
            }
            advance(&mut history, symbol);
        }
        self.documents += 1;
        self.bytes += text.len() as u64;
    }

    pub fn finish(&mut self) -> io::Result<()> {
        self.contexts.clear();
        for (&key, &count) in &self.counts {
            let entry = self.contexts.entry(key >> 9).or_default();
            entry.0 = entry
                .0
                .checked_add(count)
                .ok_or_else(|| invalid("count overflow"))?;
            entry.1 += 1;
        }
        let total = self.contexts.get(&1).map_or(0, |c| c.0);
        if total
            != self
                .bytes
                .checked_add(self.documents)
                .ok_or_else(|| invalid("size overflow"))?
        {
            return Err(invalid("unigram count does not match corpus sizes"));
        }
        Ok(())
    }

    pub fn entries(&self) -> usize {
        self.counts.len()
    }

    fn probability(&self, history: &[u16], symbol: u16, order: usize) -> f64 {
        let total = self.contexts.get(&1).map_or(0, |c| c.0);
        let count = self
            .counts
            .get(&(512 | u64::from(symbol)))
            .copied()
            .unwrap_or(0);
        let mut p = (count as f64 + 0.5) / (total as f64 + 0.5 * ALPHABET);
        for length in 1..order {
            let key = context(history, length);
            if let Some(&(n, types)) = self.contexts.get(&key) {
                let count = self
                    .counts
                    .get(&((key << 9) | u64::from(symbol)))
                    .copied()
                    .unwrap_or(0);
                p = (count as f64 + types as f64 * p) / (n + types) as f64;
            }
        }
        p
    }

    /// Includes exactly one EOS. Orders are fixed at unigram and six-gram.
    pub fn bits(&self, text: &[u8], higher_order: bool) -> f64 {
        let order = if higher_order { ORDER } else { 1 };
        let mut history = vec![BOS; ORDER - 1];
        let mut bits = 0.0;
        for symbol in text.iter().map(|&b| u16::from(b)).chain([EOS]) {
            bits -= self.probability(&history, symbol, order).log2();
            advance(&mut history, symbol);
        }
        bits
    }

    /// Deterministic little-endian counts, sorted by packed key. No corpus text.
    pub fn save(&self, mut out: impl Write) -> io::Result<()> {
        out.write_all(MAGIC)?;
        for value in [self.documents, self.bytes, self.counts.len() as u64] {
            out.write_all(&value.to_le_bytes())?;
        }
        let mut entries: Vec<_> = self.counts.iter().collect();
        entries.sort_unstable_by_key(|&(key, _)| key);
        for (&key, &count) in entries {
            out.write_all(&key.to_le_bytes())?;
            out.write_all(&count.to_le_bytes())?;
        }
        out.flush()
    }

    pub fn load(mut input: impl Read) -> io::Result<Self> {
        fn word(input: &mut impl Read) -> io::Result<u64> {
            let mut bytes = [0; 8];
            input.read_exact(&mut bytes)?;
            Ok(u64::from_le_bytes(bytes))
        }
        let mut magic = [0; 8];
        input.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(invalid("unsupported probability model format"));
        }
        let mut model = Self {
            documents: word(&mut input)?,
            bytes: word(&mut input)?,
            ..Self::default()
        };
        let entries = word(&mut input)?;
        let mut previous = 0;
        for _ in 0..entries {
            let key = word(&mut input)?;
            let count = word(&mut input)?;
            let mut prefix = key >> 9;
            let mut length = 0;
            while prefix > 1 {
                if prefix & 511 > u64::from(BOS) {
                    return Err(invalid("invalid context symbol"));
                }
                prefix >>= 9;
                length += 1;
            }
            if key <= previous
                || key & 511 > u64::from(EOS)
                || prefix != 1
                || length >= ORDER
                || count == 0
            {
                return Err(invalid("invalid or unsorted ngram entry"));
            }
            model.counts.insert(key, count);
            previous = key;
        }
        let mut trailing = [0];
        if input.read(&mut trailing)? != 0 {
            return Err(invalid("trailing model data"));
        }
        model.finish()?;
        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_seen_unseen_and_unknown_bytes() -> io::Result<()> {
        let mut model = Model::default();
        for text in [b"a cat".as_slice(), b"a dog", b"", b"a cat"] {
            model.observe(text);
        }
        model.finish()?;
        for history in [
            vec![BOS; 5],
            vec![0, 255, 254, 253, 252],
            vec![BOS, BOS, 97, 32, 99],
        ] {
            for order in [1, 6] {
                let probabilities: Vec<_> = (0..=EOS)
                    .map(|s| model.probability(&history, s, order))
                    .collect();
                assert!(probabilities.iter().all(|&p| p > 0.0 && p <= 1.0));
                assert!((probabilities.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            }
        }
        assert!(model.bits(&[0, 255, 254], true).is_finite());
        // 15 corpus bytes + 4 EOS; 'a' appears five times.
        assert!((model.probability(&[], 97, 1) - 5.5 / 147.5).abs() < 1e-12);
        Ok(())
    }

    #[test]
    fn serialization_is_exact_deterministic_and_rejects_corruption() -> io::Result<()> {
        let mut model = Model::default();
        model.observe(b"hello");
        model.observe(b"world");
        model.finish()?;
        let mut bytes = Vec::new();
        model.save(&mut bytes)?;
        let restored = Model::load(bytes.as_slice())?;
        assert_eq!(
            model.bits(b"hello\xff", true),
            restored.bits(b"hello\xff", true)
        );
        assert_eq!(model.bits(b"", false), restored.bits(b"", false));
        let mut again = Vec::new();
        restored.save(&mut again)?;
        assert_eq!(bytes, again);
        bytes.pop();
        assert!(Model::load(bytes.as_slice()).is_err());
        assert!(Model::load(b"bad file".as_slice()).is_err());
        Ok(())
    }
}
