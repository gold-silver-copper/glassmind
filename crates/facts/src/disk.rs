//! Native, read-only SQLite access to a finite facts store.
//! The database contains evidence and native alias keys, never model answers.
use crate::{Answer, AnswerSet, EntityId, SET_DISPLAY_LIMIT, Store, normalize, tokenize};
use glassmind_everyday::IntentModel;
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::Read,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

pub mod pilot;

pub const MAX_DISK_QUESTION_BYTES: usize = 4096;
pub const MAX_DISK_QUESTION_TOKENS: usize = 128;
pub const MAX_DISK_IMPORT_BYTES: usize = 32 * 1024 * 1024;
const APPLICATION_ID: i64 = 0x474d4641;
const QUERY_TICKS: usize = 10_000; // progress callback once per 1,000 VM operations

const SCHEMA: &[(&str, &str, &str)] = &[
    (
        "table",
        "metadata",
        "CREATE TABLE metadata(key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID",
    ),
    (
        "table",
        "entities",
        "CREATE TABLE entities(id TEXT PRIMARY KEY) WITHOUT ROWID",
    ),
    (
        "table",
        "aliases",
        "CREATE TABLE aliases(entity TEXT NOT NULL REFERENCES entities(id), original TEXT NOT NULL,
        normalized TEXT NOT NULL, uppercase INTEGER NOT NULL CHECK(uppercase IN (0,1)),
        PRIMARY KEY(entity,original)) WITHOUT ROWID",
    ),
    (
        "index",
        "aliases_lookup",
        "CREATE INDEX aliases_lookup ON aliases(normalized,entity)",
    ),
    (
        "table",
        "edges",
        "CREATE TABLE edges(subject TEXT NOT NULL REFERENCES entities(id), property TEXT NOT NULL,
        object TEXT NOT NULL, PRIMARY KEY(subject,property,object)) WITHOUT ROWID",
    ),
    (
        "index",
        "edges_inverse",
        "CREATE INDEX edges_inverse ON edges(property,object,subject)",
    ),
];

#[derive(Debug, Serialize)]
pub struct DiskInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_json_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_source: Option<pilot::SourceManifest>,
    pub entities: usize,
    pub aliases: usize,
    pub edges: usize,
}

pub struct DiskStore {
    db: Connection,
    ticks: Arc<AtomicUsize>,
    info: DiskInfo,
}

fn normalizer_version() -> String {
    let (major, minor, patch) = char::UNICODE_VERSION;
    format!("whole-token-lowercase-v1/unicode-{major}.{minor}.{patch}")
}

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
fn entity_id(id: String) -> Result<EntityId, String> {
    if id.starts_with('Q') && id.len() > 1 && id.chars().skip(1).all(|c| c.is_ascii_digit()) {
        Ok(EntityId(id))
    } else {
        Err("invalid entity identifier in disk store".into())
    }
}

fn configure(db: &Connection) -> Result<(), String> {
    db.busy_timeout(std::time::Duration::ZERO).map_err(error)?;
    db.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA foreign_keys=ON; PRAGMA cache_size=-65536; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;").map_err(error)?;
    db.set_limit(rusqlite::limits::Limit::SQLITE_LIMIT_LENGTH, 1_048_576)
        .map_err(error)?;
    db.set_limit(rusqlite::limits::Limit::SQLITE_LIMIT_SQL_LENGTH, 16_384)
        .map_err(error)?;
    Ok(())
}

impl DiskStore {
    /// Import a bounded JSON store. Existing output files are never overwritten.
    /// Failed imports can leave an empty/invalid owned file, which open rejects.
    pub fn import_file(input: &Path, output: &Path) -> Result<Self, String> {
        let mut bytes = Vec::new();
        File::open(input)
            .map_err(error)?
            .take((MAX_DISK_IMPORT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(error)?;
        Self::import_bytes(&bytes, output)
    }

    pub fn import_bytes(bytes: &[u8], output: &Path) -> Result<Self, String> {
        if bytes.len() > MAX_DISK_IMPORT_BYTES {
            return Err("JSON facts import byte cap exceeded".into());
        }
        let source: Store = serde_json::from_slice(bytes).map_err(error)?;
        source.validate()?;
        let hash = format!("{:x}", Sha256::digest(bytes));
        // Reserve the caller's new path before opening SQLite, without truncation.
        File::options()
            .write(true)
            .create_new(true)
            .open(output)
            .map_err(error)?;
        let mut db = Connection::open_with_flags(
            output,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(error)?;
        configure(&db)?;
        let tx = db.transaction().map_err(error)?;
        tx.pragma_update(None, "application_id", APPLICATION_ID)
            .map_err(error)?;
        tx.pragma_update(None, "user_version", 1).map_err(error)?;
        for (_, _, sql) in SCHEMA {
            tx.execute_batch(sql).map_err(error)?;
        }
        let mut aliases = 0usize;
        let mut edges = 0usize;
        {
            let mut add_entity = tx
                .prepare("INSERT INTO entities VALUES (?)")
                .map_err(error)?;
            let mut add_alias = tx
                .prepare("INSERT OR IGNORE INTO aliases VALUES (?,?,?,?)")
                .map_err(error)?;
            let mut add_edge = tx
                .prepare("INSERT OR IGNORE INTO edges VALUES (?,?,?)")
                .map_err(error)?;
            for (id, row) in &source.entities {
                add_entity.execute([&id.0]).map_err(error)?;
                for alias in &row.aliases {
                    let normalized = normalize(alias);
                    if normalized.len() < 2 {
                        continue;
                    }
                    let uppercase = alias.chars().all(|c| c.is_alphabetic() && c.is_uppercase());
                    aliases += add_alias
                        .execute(params![id.0, alias, normalized, uppercase])
                        .map_err(error)?;
                }
                for (property, objects) in &row.claims {
                    for object in objects {
                        edges += add_edge
                            .execute(params![id.0, property.0, object.0])
                            .map_err(error)?;
                    }
                }
            }
        }
        for (key, value) in [
            ("normalizer", normalizer_version()),
            ("source_json_sha256", hash),
            ("entities", source.entities.len().to_string()),
            ("aliases", aliases.to_string()),
            ("edges", edges.to_string()),
        ] {
            tx.execute("INSERT INTO metadata VALUES (?,?)", params![key, value])
                .map_err(error)?;
        }
        tx.commit().map_err(error)?;
        drop(db);
        Self::open(output)
    }

    /// Hold one read snapshot for this connection's lifetime. Metadata, alias
    /// resolution and edges cannot drift between requests if a writer commits.
    pub fn open(path: &Path) -> Result<Self, String> {
        let db = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(error)?;
        configure(&db)?;
        db.execute_batch("PRAGMA query_only=ON; BEGIN DEFERRED;")
            .map_err(error)?;
        let ticks = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&ticks);
        db.progress_handler(
            1000,
            Some(move || counter.fetch_add(1, Ordering::Relaxed) >= QUERY_TICKS),
        )
        .map_err(error)?;
        let application: i64 = db
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .map_err(error)?;
        let version: i64 = db
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(error)?;
        if application != APPLICATION_ID || ![1, 2].contains(&version) {
            return Err("unsupported disk facts format".into());
        }
        for (kind, name, expected) in SCHEMA
            .iter()
            .filter(|(_, name, _)| version == 1 || *name != "aliases")
            .chain(pilot::SCHEMA.iter().filter(|_| version == 2))
        {
            let found: Option<String> = db
                .query_row(
                    "SELECT sql FROM sqlite_schema WHERE type=? AND name=?",
                    params![kind, name],
                    |r| r.get(0),
                )
                .optional()
                .map_err(error)?;
            if !found.is_some_and(|sql| sql.split_whitespace().eq(expected.split_whitespace())) {
                return Err("incompatible disk facts schema".into());
            }
        }
        let meta = |key: &str| -> Result<String, String> {
            db.query_row("SELECT value FROM metadata WHERE key=?", [key], |r| {
                r.get(0)
            })
            .map_err(error)
        };
        if meta("normalizer")? != normalizer_version() {
            return Err("incompatible disk facts normalizer; rebuild the index".into());
        }
        let (source_json_sha256, public_source) = if version == 1 {
            let hash = meta("source_json_sha256")?;
            if !pilot::hex_hash(&hash, 64) {
                return Err("invalid disk facts source hash".into());
            }
            (Some(hash), None)
        } else {
            let manifest: pilot::SourceManifest =
                serde_json::from_str(&meta("public_source")?).map_err(error)?;
            manifest.validate()?;
            if meta("source_kind")? != "audited-wikidata-parquet-pilot" {
                return Err("unsupported public facts source".into());
            }
            (None, Some(manifest))
        };
        let info = DiskInfo {
            source_json_sha256,
            public_source,
            entities: meta("entities")?.parse().map_err(error)?,
            aliases: meta("aliases")?.parse().map_err(error)?,
            edges: meta("edges")?.parse().map_err(error)?,
        };
        Ok(Self { db, ticks, info })
    }

    pub fn info(&self) -> &DiskInfo {
        &self.info
    }

    fn begin(&self, question: &str) -> Result<(), String> {
        if question.len() > MAX_DISK_QUESTION_BYTES {
            return Err("disk facts question byte cap exceeded".into());
        }
        self.ticks.store(0, Ordering::Relaxed);
        if self.db.is_autocommit() {
            return Err("disk facts read snapshot ended; reopen the index".into());
        }
        Ok(())
    }

    pub fn link(&self, question: &str) -> Result<Option<EntityId>, String> {
        self.begin(question)?;
        let answer = self.link_inner(question)?;
        Ok(answer)
    }

    fn link_inner(&self, question: &str) -> Result<Option<EntityId>, String> {
        let normalized = normalize(question);
        let words: Vec<_> = normalized.split_whitespace().collect();
        if words.len() > MAX_DISK_QUESTION_TOKENS {
            return Err("disk facts question token cap exceeded".into());
        }
        let cased = format!(" {} ", tokenize(question));
        // Borrow substrings of one normalized question instead of storing O(n^3)
        // characters. Every whole-token alias present in the question is one span.
        let mut spans = BTreeSet::new();
        let mut offset = 0;
        for (start, word) in words.iter().enumerate() {
            let mut end = offset;
            for part in words.iter().skip(start) {
                end += part.len();
                if let Some(span) = normalized.get(offset..end).filter(|s| s.len() >= 2) {
                    spans.insert(span);
                }
                end += 1;
            }
            offset += word.len() + 1;
        }
        let mut spans: Vec<_> = spans.into_iter().collect();
        spans.sort_by_key(|s| std::cmp::Reverse(s.len()));
        let mut best_length = 0;
        let mut candidates = BTreeSet::new();
        let mut supported = BTreeSet::new();
        let mut statement = self
            .db
            .prepare_cached(
                "SELECT entity, MAX(uppercase=0 OR instr(?2, ' ' || original || ' ')>0)
            FROM aliases WHERE normalized=?1 GROUP BY entity ORDER BY entity LIMIT 2",
            )
            .map_err(error)?;
        for span in spans {
            if span.len() < best_length {
                break;
            }
            let mut rows = statement.query(params![span, cased]).map_err(error)?;
            while let Some(row) = rows.next().map_err(error)? {
                let id = entity_id(row.get(0).map_err(error)?)?;
                let supports: bool = row.get(1).map_err(error)?;
                best_length = span.len();
                candidates.insert(id.clone());
                if supports {
                    supported.insert(id);
                }
                // Spans are descending length: no later span can remove this tie.
                if candidates.len() > 1 {
                    return Ok(None);
                }
            }
        }
        Ok(candidates
            .into_iter()
            .next()
            .filter(|id| supported.contains(id)))
    }

    fn values(
        &self,
        entity: &EntityId,
        label: &str,
        limit: usize,
        count: bool,
    ) -> Result<(BTreeSet<EntityId>, usize), String> {
        let (property, values_sql, count_sql) = if label.starts_with('P') {
            (
                label.to_owned(),
                "SELECT object FROM edges WHERE subject=?1 AND property=?2 ORDER BY object LIMIT ?3",
                "SELECT count(*) FROM edges WHERE subject=?1 AND property=?2",
            )
        } else if let Some(suffix) = label.strip_prefix('R') {
            (
                format!("P{suffix}"),
                "SELECT subject FROM edges WHERE object=?1 AND property=?2 ORDER BY subject LIMIT ?3",
                "SELECT count(*) FROM edges WHERE object=?1 AND property=?2",
            )
        } else {
            return Ok((BTreeSet::new(), 0));
        };
        let mut statement = self.db.prepare_cached(values_sql).map_err(error)?;
        let mut rows = statement
            .query(params![entity.0, property, limit as i64])
            .map_err(error)?;
        let mut values = BTreeSet::new();
        while let Some(row) = rows.next().map_err(error)? {
            values.insert(entity_id(row.get(0).map_err(error)?)?);
        }
        let total = if count {
            usize::try_from(
                self.db
                    .query_row(count_sql, params![entity.0, property], |r| {
                        r.get::<_, i64>(0)
                    })
                    .map_err(error)?,
            )
            .map_err(error)?
        } else {
            values.len()
        };
        Ok((values, total))
    }

    fn proposal(&self, model: &IntentModel, question: &str) -> Result<Answer, String> {
        let mut answer = Answer {
            entity: None,
            relation: None,
            answers: BTreeSet::new(),
            refusal: None,
        };
        let Some(entity) = self.link_inner(question)? else {
            answer.refusal = Some("no unique entity alias in this snapshot".into());
            return Ok(answer);
        };
        answer.entity = Some(entity);
        let prediction = model.predict(question);
        answer.relation = Some(prediction.label);
        if !prediction.accepted {
            answer.refusal = Some("uncertain relation".into());
        }
        Ok(answer)
    }

    pub fn answer(&self, model: &IntentModel, question: &str) -> Result<Answer, String> {
        self.begin(question)?;
        let mut answer = self.proposal(model, question)?;
        if answer.refusal.is_none() {
            if let (Some(entity), Some(label)) = (&answer.entity, &answer.relation) {
                answer.answers = self.values(entity, label, 2, false)?.0;
            }
            if answer.answers.is_empty() {
                answer.refusal = Some("claim missing from this finite snapshot".into());
            } else if answer.answers.len() > 1 {
                answer.answers.clear();
                answer.refusal =
                    Some("this snapshot has multiple answers; narrow the question".into());
            }
        }
        Ok(answer)
    }

    pub fn answer_set(&self, model: &IntentModel, question: &str) -> Result<AnswerSet, String> {
        self.begin(question)?;
        let answer = self.proposal(model, question)?;
        let mut result = AnswerSet {
            answer,
            total_snapshot_matches: 0,
            returned: 0,
            truncated: false,
        };
        if result.answer.refusal.is_none()
            && let (Some(entity), Some(label)) = (&result.answer.entity, &result.answer.relation)
        {
            if label.len() < 2
                || !label
                    .get(1..)
                    .is_some_and(|s| s.bytes().all(|c| c.is_ascii_digit()))
                || !label.starts_with(['P', 'R'])
            {
                result.answer.refusal = Some("unsupported relation".into());
            } else {
                let (values, total) = self.values(entity, label, SET_DISPLAY_LIMIT, true)?;
                result.total_snapshot_matches = total;
                result.returned = values.len();
                result.truncated = total > result.returned;
                result.answer.answers = values;
                if result.returned == 0 {
                    result.answer.refusal = Some("claim missing from this finite snapshot".into());
                }
            }
        }
        Ok(result)
    }

    /// Diagnostic query plans, without exposing mutable database access.
    pub fn query_plans(&self) -> Result<Vec<String>, String> {
        let mut plans = Vec::new();
        self.ticks.store(0, Ordering::Relaxed);
        for sql in [
            "EXPLAIN QUERY PLAN SELECT entity FROM aliases WHERE normalized='example'",
            "EXPLAIN QUERY PLAN SELECT object FROM edges WHERE subject='Q1' AND property='P1'",
            "EXPLAIN QUERY PLAN SELECT subject FROM edges WHERE property='P1' AND object='Q1'",
        ] {
            let mut stmt = self.db.prepare(sql).map_err(error)?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(3))
                .map_err(error)?;
            for row in rows {
                plans.push(row.map_err(error)?);
            }
        }
        Ok(plans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_work_budget_returns_no_partial_set_and_next_request_recovers()
    -> Result<(), Box<dyn std::error::Error>> {
        let path = std::env::temp_dir().join(format!(
            "glassmind-disk-budget-{}.sqlite",
            std::process::id()
        ));
        let source = serde_json::json!({"entities": {"Q1": {"aliases": ["Atlas"], "claims": {
            "P1": (2..3002).map(|i| format!("Q{i}")).collect::<Vec<_>>()
        }}}});
        let disk = DiskStore::import_bytes(&serde_json::to_vec(&source)?, &path)?;
        {
            disk.begin("Atlas")?;
            // Exhaust the actual installed VM budget before counting a large set.
            disk.ticks.store(QUERY_TICKS, Ordering::Relaxed);
            assert!(disk.values(&EntityId("Q1".into()), "P1", 10, true).is_err());
        }
        assert_eq!(disk.link("Atlas")?, Some(EntityId("Q1".into())));
        drop(disk);
        std::fs::remove_file(path)?;
        Ok(())
    }
}
