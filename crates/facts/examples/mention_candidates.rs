//! Research-only exact alias candidates; labels never enter this extractor.
use glassmind_facts::disk::{DiskStore, MAX_DISK_QUESTION_BYTES, MAX_DISK_QUESTION_TOKENS};
use rusqlite::{Connection, OpenFlags, params};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, BufRead, Write},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Literal copies of crates/facts/src/lib.rs; embedded Unicode version checked.
fn normalize(text: &str) -> String {
    tokenize(&text.to_lowercase())
}
fn tokenize(text: &str) -> String {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Clone, Debug, Serialize)]
struct Match {
    entity: String,
    case_supported: bool,
}
#[derive(Debug, Serialize)]
struct Span {
    token_start: usize,
    token_end: usize,
    byte_start: usize,
    byte_end: usize,
    text: String,
    matches: Vec<Match>,
    // Two matches means at least two, not an exhaustive candidate ID list.
    ambiguous: bool,
    eligible_entity: Option<String>,
}
#[derive(Serialize)]
struct Record {
    question: String,
    normalized: String,
    candidates: Vec<Span>,
    baseline_entity: Option<String>,
    parity: bool,
}

fn reconstructed(spans: &[Span]) -> Option<String> {
    let longest = spans.iter().map(|s| s.text.len()).max()?;
    let mut entities = BTreeSet::new();
    let mut supported = BTreeSet::new();
    for span in spans.iter().filter(|s| s.text.len() == longest) {
        for item in &span.matches {
            entities.insert(item.entity.clone());
            if item.case_supported {
                supported.insert(item.entity.clone());
            }
        }
    }
    if entities.len() != 1 {
        return None;
    }
    entities
        .into_iter()
        .next()
        .filter(|id| supported.contains(id))
}

struct Extractor {
    db: Connection,
    native: DiskStore,
    ticks: Arc<AtomicUsize>,
}
impl Extractor {
    fn open(path: &Path) -> Result<Self> {
        let native = DiskStore::open(path)?;
        let db = Connection::open_with_flags(
            path.canonicalize()?,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        db.execute_batch("PRAGMA query_only=ON; PRAGMA cache_size=-65536; PRAGMA mmap_size=0; PRAGMA temp_store=FILE; BEGIN;")?;
        let version: String = db.query_row(
            "SELECT value FROM metadata WHERE key='normalizer'",
            [],
            |r| r.get(0),
        )?;
        let (major, minor, patch) = char::UNICODE_VERSION;
        if version != format!("whole-token-lowercase-v1/unicode-{major}.{minor}.{patch}") {
            return Err("normalizer version mismatch".into());
        }
        let ticks = Arc::new(AtomicUsize::new(0));
        let progress = ticks.clone();
        db.progress_handler(
            1000,
            Some(move || progress.fetch_add(1, Ordering::Relaxed) >= 10_000),
        )?;
        Ok(Self { db, native, ticks })
    }

    fn extract(&self, question: &str) -> Result<Record> {
        if question.len() > MAX_DISK_QUESTION_BYTES {
            return Err("question byte cap exceeded".into());
        }
        let normalized = normalize(question);
        let words: Vec<_> = normalized.split_whitespace().collect();
        if words.len() > MAX_DISK_QUESTION_TOKENS {
            return Err("question token cap exceeded".into());
        }
        self.ticks.store(0, Ordering::Relaxed);
        let cased = format!(" {} ", tokenize(question));
        let mut query = self.db.prepare_cached("SELECT entity,MAX(uppercase=0 OR instr(?2,' '||original||' ')>0) FROM aliases WHERE normalized=?1 GROUP BY entity ORDER BY entity LIMIT 2")?;
        let mut cached = BTreeMap::<String, Vec<Match>>::new();
        let mut candidates = Vec::new();
        let mut start_byte = 0;
        for (start, word) in words.iter().enumerate() {
            let mut end_byte = start_byte;
            for (end, part) in words.iter().enumerate().skip(start) {
                end_byte += part.len();
                let text = normalized
                    .get(start_byte..end_byte)
                    .ok_or("invalid normalized span")?;
                if text.len() >= 2 {
                    if !cached.contains_key(text) {
                        let rows = query.query_map(params![text, cased], |r| {
                            Ok(Match {
                                entity: r.get(0)?,
                                case_supported: r.get(1)?,
                            })
                        })?;
                        let matches = rows.collect::<std::result::Result<Vec<_>, _>>()?;
                        cached.insert(text.to_owned(), matches);
                    }
                    let matches = cached.get(text).ok_or("missing span lookup")?;
                    if !matches.is_empty() {
                        if candidates.len() >= 512 {
                            return Err("matched span occurrence cap exceeded".into());
                        }
                        let eligible_entity = match matches.as_slice() {
                            [item] if item.case_supported => Some(item.entity.clone()),
                            _ => None,
                        };
                        candidates.push(Span {
                            token_start: start,
                            token_end: end + 1,
                            byte_start: start_byte,
                            byte_end: end_byte,
                            text: text.to_owned(),
                            ambiguous: matches.len() == 2,
                            matches: matches.clone(),
                            eligible_entity,
                        });
                    }
                }
                end_byte += 1;
            }
            start_byte += word.len() + 1;
        }
        let baseline_entity = self.native.link(question)?.map(|id| id.0);
        if reconstructed(&candidates) != baseline_entity {
            return Err("candidate reconstruction differs from native linker".into());
        }
        Ok(Record {
            question: question.into(),
            normalized,
            candidates,
            baseline_entity,
            parity: true,
        })
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let [_, command] = args.as_slice() else {
        return Err("usage: mention_candidates DATABASE | --normalize".into());
    };
    let extractor = if command == "--normalize" {
        None
    } else {
        Some(Extractor::open(Path::new(command))?)
    };
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let question = line?;
        if let Some(extractor) = &extractor {
            serde_json::to_writer(&mut output, &extractor.extract(&question)?)?;
        } else {
            serde_json::to_writer(&mut output, &normalize(&question))?;
        }
        writeln!(output)?;
    }
    output.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    fn fixture() -> Result<Fixture> {
        let path = std::env::temp_dir().join(format!(
            "mention-contract-{}-{}.sqlite",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let store = serde_json::json!({"entities":{
            "Q1":{"aliases":["Atlas","Café"],"claims":{}},
            "Q2":{"aliases":["celestial object","東京"],"claims":{}},
            "Q3":{"aliases":["Xeno"],"claims":{}},
            "Q4":{"aliases":["Xeno"],"claims":{}},
            "Q5":{"aliases":["NASA"],"claims":{}},
            "Q6":{"aliases":["Jo"],"claims":{}},
            "Q7":{"aliases":["Amber"],"claims":{}}
        }});
        drop(DiskStore::import_bytes(
            &serde_json::to_vec(&store)?,
            &path,
        )?);
        Ok(Fixture(path))
    }
    #[test]
    fn spans_preserve_native_ties_case_unicode_and_alternatives() -> Result<()> {
        let f = fixture()?;
        let e = Extractor::open(&f.0)?;
        let long = e.extract("What celestial object is Atlas?")?;
        assert_eq!(long.baseline_entity.as_deref(), Some("Q2"));
        assert!(
            long.candidates
                .iter()
                .any(|s| s.eligible_entity.as_deref() == Some("Q1"))
        );
        let tied = e.extract("Xeno Jo")?;
        assert_eq!(tied.baseline_entity, None);
        assert!(
            tied.candidates
                .iter()
                .any(|s| s.ambiguous && s.eligible_entity.is_none())
        );
        assert_eq!(e.extract("Atlas Amber")?.baseline_entity, None);
        let lower = e.extract("nasa Jo")?;
        assert_eq!(lower.baseline_entity, None);
        assert!(
            lower
                .candidates
                .iter()
                .any(|s| s.eligible_entity.as_deref() == Some("Q6"))
        );
        assert_eq!(e.extract("NASA Jo")?.baseline_entity.as_deref(), Some("Q5"));
        let repeated = e.extract("Atlas and Atlas")?;
        assert_eq!(
            repeated
                .candidates
                .iter()
                .filter(|s| s.text == "atlas")
                .count(),
            2
        );
        assert_eq!(repeated.baseline_entity.as_deref(), Some("Q1"));
        let unicode = e.extract("café—東京")?;
        assert_eq!(unicode.normalized, "café 東京");
        assert_eq!(unicode.baseline_entity.as_deref(), Some("Q2"));
        assert!(unicode.candidates.iter().any(|s| (
            s.token_start,
            s.token_end,
            s.byte_start,
            s.byte_end
        ) == (1, 2, 6, 12)));
        assert_eq!(e.extract("???")?.baseline_entity, None);
        assert!(e.extract(&"x".repeat(4097)).is_err());
        assert!(e.extract(&vec!["ab"; 129].join(" ")).is_err());
        Ok(())
    }
}
