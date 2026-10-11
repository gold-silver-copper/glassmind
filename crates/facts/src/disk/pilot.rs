//! Import a hash-pinned, independently audited numeric Wikidata projection.
//! Raw Parquet remains the authority for complete qualifiers and references.
use super::{APPLICATION_ID, DiskStore, configure, error, normalize, normalizer_version};
use rusqlite::{Connection, OpenFlags, functions::FunctionFlags, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path, time::Instant};

const MAX_MANIFEST_BYTES: u64 = 1_048_576;
const MAX_OUTPUT_BYTES: u64 = 24 * 1024 * 1024 * 1024;
const MAX_SECONDS: u64 = 3600;

pub(super) const SCHEMA: &[(&str, &str, &str)] = &[
    (
        "table",
        "aliases",
        "CREATE TABLE aliases(entity TEXT NOT NULL REFERENCES entities(id),
        original TEXT NOT NULL, normalized TEXT NOT NULL,
        uppercase INTEGER NOT NULL CHECK(uppercase IN (0,1)),
        source_mask INTEGER NOT NULL CHECK(source_mask BETWEEN 1 AND 15),
        occurrences INTEGER NOT NULL CHECK(occurrences>0),
        first_source_row INTEGER NOT NULL CHECK(first_source_row>0),
        PRIMARY KEY(entity,original)) WITHOUT ROWID",
    ),
    (
        "table",
        "statements",
        "CREATE TABLE statements(source_row INTEGER PRIMARY KEY CHECK(source_row>0),
        subject TEXT NOT NULL, property TEXT NOT NULL, object TEXT NOT NULL,
        preferred INTEGER NOT NULL CHECK(preferred IN (0,1)),
        qualified INTEGER NOT NULL CHECK(qualified IN (0,1)), guid TEXT NOT NULL,
        FOREIGN KEY(subject,property,object) REFERENCES edges(subject,property,object))",
    ),
];

/// Source-row positions are zero-based file offsets; stored row IDs add one.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub table: String,
    pub path: String,
    pub revision: String,
    pub sha256: String,
    pub bytes: u64,
    pub source_rows: u64,
    pub kept_rows: u64,
    pub row_id_offset: u64,
}

/// Portable provenance for the exact numeric pilot and its original public files.
/// The canonical-ID audit is a prerequisite, not an assertion made by import.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceManifest {
    pub schema: u32,
    pub license: String,
    pub publisher: String,
    pub snapshot: String,
    pub scope: String,
    pub pilot_sha256: String,
    pub pilot_bytes: u64,
    pub canonical_audit_sha256: String,
    pub names: u64,
    pub statements: u64,
    pub qualified_statements: u64,
    pub files: Vec<SourceFile>,
}

pub(super) fn hex_hash(hash: &str, length: usize) -> bool {
    hash.len() == length
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

impl SourceManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != 1
            || self.license != "CC0-1.0"
            || self.publisher != "permutans"
            || self.snapshot.len() != 8
            || !self.snapshot.bytes().all(|b| b.is_ascii_digit())
            || self.scope.trim().is_empty()
            || !hex_hash(&self.pilot_sha256, 64)
            || !hex_hash(&self.canonical_audit_sha256, 64)
            || self.pilot_bytes == 0
            || self.files.is_empty()
            || self.files.len() > 64
            || self.qualified_statements > self.statements
        {
            return Err("invalid Wikidata pilot provenance".into());
        }
        let mut offset = 0u64;
        let mut names = 0u64;
        let mut statements = 0u64;
        let mut claims_files = 0;
        let mut paths = std::collections::BTreeSet::new();
        for file in &self.files {
            if !hex_hash(&file.sha256, 64)
                || !hex_hash(&file.revision, 40)
                || file.bytes == 0
                || file.source_rows == 0
                || file.kept_rows > file.source_rows
                || !paths.insert((&file.table, &file.path))
                || file
                    .path
                    .split('/')
                    .any(|s| s.is_empty() || s == "." || s == "..")
            {
                return Err("invalid Wikidata source file provenance".into());
            }
            if file.table == "claims" {
                if file.row_id_offset != 0 || !file.path.starts_with("all/") {
                    return Err("invalid claim source row mapping".into());
                }
                claims_files += 1;
                statements = file.kept_rows;
            } else {
                if !["labels", "aliases"].contains(&file.table.as_str())
                    || !(file.path.starts_with("en/") || file.path.starts_with("mul/"))
                    || file.row_id_offset != offset
                {
                    return Err("invalid name source row mapping".into());
                }
                offset = offset
                    .checked_add(file.source_rows)
                    .ok_or("source row overflow")?;
                names = names
                    .checked_add(file.kept_rows)
                    .ok_or("name count overflow")?;
            }
            if file
                .row_id_offset
                .checked_add(file.source_rows)
                .is_none_or(|n| n > i64::MAX as u64)
            {
                return Err("source row range exceeds SQLite integer range".into());
            }
        }
        if claims_files != 1 || names != self.names || statements != self.statements {
            return Err("source file counts do not match the pilot manifest".into());
        }
        Ok(())
    }
}

fn file_hash(path: &Path) -> Result<(u64, String), String> {
    let mut file = File::open(path).map_err(error)?;
    let mut sha = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = vec![0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(error)?;
        if read == 0 {
            break;
        }
        bytes += read as u64;
        sha.update(buffer.get(..read).ok_or("invalid source read length")?);
    }
    Ok((bytes, format!("{:x}", sha.finalize())))
}

fn readonly_uri(path: &Path) -> Result<String, String> {
    let path = path.canonicalize().map_err(error)?;
    let path = path.to_str().ok_or("pilot path is not UTF-8")?;
    let mut uri = String::from("file:");
    for b in path.bytes() {
        if b.is_ascii_alphanumeric() || b"/-._~".contains(&b) {
            uri.push(char::from(b));
        } else {
            uri.push_str(&format!("%{b:02X}"));
        }
    }
    uri.push_str("?mode=ro");
    Ok(uri)
}

fn count(db: &Connection, sql: &str) -> Result<u64, String> {
    u64::try_from(
        db.query_row(sql, [], |r| r.get::<_, i64>(0))
            .map_err(error)?,
    )
    .map_err(error)
}

fn stage(started: Instant, name: &str) {
    eprintln!(
        "{}",
        serde_json::json!({"stage":name,"seconds":started.elapsed().as_secs_f64()})
    );
}

/// Creates a separate format-2 output, never mutating or replacing the source.
/// Run in an external memory/no-swap scope and a private bounded disk TMPDIR.
/// Any interrupted transaction leaves an invalid output; preserve it for review.
pub fn import_file(pilot: &Path, manifest_path: &Path, output: &Path) -> Result<DiskStore, String> {
    let started = Instant::now();
    let mut bytes = Vec::new();
    File::open(manifest_path)
        .map_err(error)?
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(error)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err("pilot manifest byte cap exceeded".into());
    }
    let manifest: SourceManifest = serde_json::from_slice(&bytes).map_err(error)?;
    manifest.validate()?;
    if file_hash(pilot)? != (manifest.pilot_bytes, manifest.pilot_sha256.clone()) {
        return Err("pilot database does not match the pinned source hash/size".into());
    }
    stage(started, "source hash verified");
    File::options()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(error)?;
    let mut db = Connection::open_with_flags(
        output,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(error)?;
    configure(&db)?;
    // Foreign keys are checked in full before the single transaction commits.
    db.execute_batch(
        "PRAGMA foreign_keys=OFF; PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;",
    )
    .map_err(error)?;
    let page_size: i64 = db
        .query_row("PRAGMA page_size", [], |r| r.get(0))
        .map_err(error)?;
    let pages = i64::try_from(MAX_OUTPUT_BYTES - 1)
        .map_err(error)?
        .checked_div(page_size)
        .filter(|n| *n > 0)
        .ok_or("invalid SQLite page size")?;
    db.pragma_update(None, "max_page_count", pages)
        .map_err(error)?;
    db.execute("ATTACH DATABASE ? AS source", [readonly_uri(pilot)?])
        .map_err(error)?;
    db.execute_batch("PRAGMA source.cache_size=-65536; PRAGMA source.mmap_size=0;")
        .map_err(error)?;
    db.progress_handler(
        100_000,
        Some(move || started.elapsed().as_secs() >= MAX_SECONDS),
    )
    .map_err(error)?;
    let flags = FunctionFlags::SQLITE_UTF8
        | FunctionFlags::SQLITE_DETERMINISTIC
        | FunctionFlags::SQLITE_INNOCUOUS;
    db.create_scalar_function("native_key", 1, flags, |ctx| {
        let value: String = ctx.get(0)?;
        let key = normalize(&value);
        Ok((key.len() >= 2).then_some(key))
    })
    .map_err(error)?;
    db.create_scalar_function("native_uppercase", 1, flags, |ctx| {
        let value: String = ctx.get(0)?;
        Ok(value.chars().all(|c| c.is_alphabetic() && c.is_uppercase()))
    })
    .map_err(error)?;
    let tx = db.transaction().map_err(error)?;
    // Hash pinning is the trust boundary; still reject views and malformed values.
    for table in ["names", "edges"] {
        let kind: String = tx
            .query_row(
                "SELECT type FROM source.sqlite_schema WHERE name=?",
                [table],
                |r| r.get(0),
            )
            .map_err(error)?;
        if kind != "table" {
            return Err("pilot source must contain real tables".into());
        }
    }
    for file in &manifest.files {
        let (sql, language, kind) = if file.table == "claims" {
            (
                "SELECT count(*) FROM source.edges WHERE row_id>?1 AND row_id<=?2",
                0,
                0,
            )
        } else {
            (
                "SELECT count(*) FROM source.names WHERE row_id>?1 AND row_id<=?2 AND language=?3 AND kind=?4",
                i64::from(file.path.starts_with("mul/")),
                i64::from(file.table == "aliases"),
            )
        };
        let mut stmt = tx.prepare(sql).map_err(error)?;
        let start = i64::try_from(file.row_id_offset).map_err(error)?;
        let end = i64::try_from(file.row_id_offset + file.source_rows).map_err(error)?;
        let found: i64 = if file.table == "claims" {
            stmt.query_row(params![start, end], |r| r.get(0))
                .map_err(error)?
        } else {
            stmt.query_row(params![start, end, language, kind], |r| r.get(0))
                .map_err(error)?
        };
        if u64::try_from(found).map_err(error)? != file.kept_rows {
            return Err("pilot source row mapping/count mismatch".into());
        }
    }
    if count(&tx, "SELECT count(*) FROM source.names")? != manifest.names
        || count(&tx, "SELECT count(*) FROM source.edges")? != manifest.statements
        || count(&tx, "SELECT count(*) FROM source.edges WHERE qualified=1")?
            != manifest.qualified_statements
    {
        return Err("pilot source counts do not match manifest".into());
    }
    for sql in [
        "SELECT count(*) FROM source.names WHERE typeof(row_id)!='integer' OR row_id<=0 OR typeof(entity)!='integer' OR entity<0 OR typeof(name)!='text' OR typeof(language)!='integer' OR language NOT IN (0,1) OR typeof(kind)!='integer' OR kind NOT IN (0,1)",
        "SELECT count(*) FROM source.edges WHERE typeof(row_id)!='integer' OR row_id<=0 OR typeof(subject)!='integer' OR subject<0 OR typeof(property)!='integer' OR property<0 OR typeof(object)!='integer' OR object<0 OR typeof(preferred)!='integer' OR preferred NOT IN (0,1) OR typeof(qualified)!='integer' OR qualified NOT IN (0,1) OR typeof(statement)!='text' OR upper(substr(statement,1,instr(statement,'$')-1))!='Q'||subject OR instr(statement,'$')<=1 OR length(statement)<=instr(statement,'$')",
    ] {
        if count(&tx, sql)? != 0 {
            return Err("invalid numeric pilot row".into());
        }
    }
    stage(started, "source rows validated");
    for (_, _, sql) in super::SCHEMA
        .iter()
        .filter(|(kind, name, _)| *kind == "table" && *name != "aliases")
        .chain(SCHEMA.iter())
    {
        tx.execute_batch(sql).map_err(error)?;
    }
    tx.execute_batch("INSERT INTO entities SELECT 'Q'||entity FROM source.names GROUP BY entity ORDER BY 1;
        INSERT OR IGNORE INTO entities SELECT 'Q'||subject FROM source.edges GROUP BY subject ORDER BY 1;")
        .map_err(error)?;
    let entities = count(&tx, "SELECT count(*) FROM entities")?;
    stage(started, "entities loaded");
    // Group by exact original text before normalizing. A mask retains every
    // language/kind combination, count retains duplicate occurrences, and the
    // first row plus the file offsets provides a deterministic raw-row witness.
    tx.execute_batch("WITH grouped AS MATERIALIZED (
        SELECT 'Q'||entity AS entity, name AS original, native_key(name) AS normalized,
        native_uppercase(name) AS uppercase,
        MAX(language=0 AND kind=0) + 2*MAX(language=0 AND kind=1)
          + 4*MAX(language=1 AND kind=0) + 8*MAX(language=1 AND kind=1) AS source_mask,
        count(*) AS occurrences, MIN(row_id) AS first_source_row
        FROM source.names GROUP BY entity,name)
        INSERT INTO aliases SELECT * FROM grouped WHERE normalized IS NOT NULL ORDER BY entity,original;")
        .map_err(error)?;
    let aliases = count(&tx, "SELECT count(*) FROM aliases")?;
    let accepted_names = count(&tx, "SELECT coalesce(sum(occurrences),0) FROM aliases")?;
    let excluded_names = manifest
        .names
        .checked_sub(accepted_names)
        .ok_or("alias occurrence count overflow")?;
    stage(started, "native aliases loaded");
    tx.execute_batch("INSERT INTO edges SELECT 'Q'||subject,'P'||property,'Q'||object
        FROM source.edges GROUP BY subject,property,object ORDER BY 1,2,3;
        INSERT INTO statements SELECT row_id,'Q'||subject,'P'||property,'Q'||object,preferred,qualified,statement
        FROM source.edges ORDER BY row_id;")
        .map_err(error)?;
    let edges = count(&tx, "SELECT count(*) FROM edges")?;
    if count(&tx, "SELECT count(*) FROM statements")? != manifest.statements
        || count(&tx, "SELECT count(*) FROM statements WHERE qualified=1")?
            != manifest.qualified_statements
    {
        return Err("statement witness count mismatch".into());
    }
    stage(started, "edges and statement witnesses loaded");
    for (_, _, sql) in super::SCHEMA.iter().filter(|(kind, _, _)| *kind == "index") {
        tx.execute_batch(sql).map_err(error)?;
    }
    stage(started, "secondary indexes built");
    for (key, value) in [
        ("normalizer", normalizer_version()),
        ("source_kind", "audited-wikidata-parquet-pilot".into()),
        ("public_source", serde_json::to_string(&manifest).map_err(error)?),
        ("entities", entities.to_string()), ("aliases", aliases.to_string()), ("edges", edges.to_string()),
        ("accepted_name_rows", accepted_names.to_string()), ("excluded_short_name_rows", excluded_names.to_string()),
        ("statements", manifest.statements.to_string()),
        ("qualified_statements", manifest.qualified_statements.to_string()),
        ("claim_semantics", "Recorded normal/preferred entity-valued statements, including qualified and historical claims; not a current timeless fact assertion. Raw Parquet retains full qualifiers/references.".into()),
    ] {
        tx.execute("INSERT INTO metadata VALUES (?,?)", params![key, value]).map_err(error)?;
    }
    let integrity: String = tx
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(error)?;
    if integrity != "ok" {
        return Err(format!("native pilot integrity check failed: {integrity}"));
    }
    if tx
        .prepare("PRAGMA foreign_key_check")
        .map_err(error)?
        .exists([])
        .map_err(error)?
    {
        return Err("native pilot foreign key check failed".into());
    }
    stage(started, "integrity and references checked");
    // Only a fully validated build receives a recognizable format header.
    tx.pragma_update(None, "application_id", APPLICATION_ID)
        .map_err(error)?;
    tx.pragma_update(None, "user_version", 2).map_err(error)?;
    tx.commit().map_err(error)?;
    drop(db);
    if started.elapsed().as_secs() >= MAX_SECONDS
        || output.metadata().map_err(error)?.len() >= MAX_OUTPUT_BYTES
    {
        return Err("native pilot resource gate exceeded".into());
    }
    stage(started, "committed");
    DiskStore::open(output)
}
