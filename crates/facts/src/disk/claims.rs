//! Multi-shard numeric claims projection plus a pinned, native format-2 alias store.
//! The broader source has separate provenance and limits from the original pilot.
use super::{APPLICATION_ID, DiskStore, configure, error, normalizer_version, pilot};
use pilot::{count, file_hash, hex_hash, readonly_uri, stage};
use rusqlite::{Connection, OpenFlags, params};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path, time::Instant};

const MAX_MANIFEST_BYTES: u64 = 1_048_576;
const MAX_OUTPUT_BYTES: u64 = 192 * 1024 * 1024 * 1024;
const MAX_SECONDS: u64 = 21600;

/// Claims row IDs are global raw ordinals + 1, including excluded source rows.
/// Names retain the original pilot's independent row space and provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceManifest {
    pub schema: u32,
    pub scope: String,
    pub names_sha256: String,
    pub names_bytes: u64,
    pub names_source: pilot::SourceManifest,
    pub projection_sha256: String,
    pub projection_bytes: u64,
    pub canonical_audit_sha256: String,
    pub statements: u64,
    pub qualified_statements: u64,
    pub files: Vec<pilot::SourceFile>,
}

impl SourceManifest {
    pub fn validate(&self) -> Result<(), String> {
        self.names_source.validate()?;
        if self.schema != 1
            || self.scope.trim().is_empty()
            || !hex_hash(&self.names_sha256, 64)
            || self.names_bytes == 0
            || !hex_hash(&self.projection_sha256, 64)
            || self.projection_bytes == 0
            || !hex_hash(&self.canonical_audit_sha256, 64)
            || self.qualified_statements > self.statements
            || self.files.is_empty()
            || self.files.len() > 1024
        {
            return Err("invalid claims snapshot provenance".into());
        }
        let revision = &self
            .names_source
            .files
            .iter()
            .find(|f| f.table == "claims")
            .ok_or("missing original claims revision")?
            .revision;
        let mut offset = 0u64;
        let mut kept = 0u64;
        let mut previous: Option<&str> = None;
        for f in &self.files {
            if f.table != "claims"
                || !f.path.starts_with("all/")
                || f.path
                    .split('/')
                    .any(|s| s.is_empty() || s == "." || s == "..")
                || previous.is_some_and(|p| p >= f.path.as_str())
                || f.revision != *revision
                || !hex_hash(&f.sha256, 64)
                || f.bytes == 0
                || f.source_rows == 0
                || f.kept_rows > f.source_rows
                || f.row_id_offset != offset
            {
                return Err("invalid claims file or source row mapping".into());
            }
            offset = offset
                .checked_add(f.source_rows)
                .filter(|n| *n <= i64::MAX as u64)
                .ok_or("claims row range overflow")?;
            kept = kept
                .checked_add(f.kept_rows)
                .ok_or("claims count overflow")?;
            previous = Some(&f.path);
        }
        if kept != self.statements {
            return Err("claims file counts disagree with manifest".into());
        }
        Ok(())
    }
}

/// Build a new format-3 database without mutating either pinned input. The names
/// source must be a validated format-2 native pilot, not a JSON/QA-derived index.
/// Use an external finite supervisor with 8 GiB/no swap and a disk-backed TMPDIR.
pub fn import_file(
    names: &Path,
    claims: &Path,
    manifest_path: &Path,
    output: &Path,
) -> Result<DiskStore, String> {
    let started = Instant::now();
    let mut bytes = Vec::new();
    File::open(manifest_path)
        .map_err(error)?
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(error)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err("claims manifest byte cap exceeded".into());
    }
    let manifest: SourceManifest = serde_json::from_slice(&bytes).map_err(error)?;
    manifest.validate()?;
    if file_hash(names)? != (manifest.names_bytes, manifest.names_sha256.clone())
        || file_hash(claims)?
            != (
                manifest.projection_bytes,
                manifest.projection_sha256.clone(),
            )
    {
        return Err("claims snapshot input hash/size mismatch".into());
    }
    let base = DiskStore::open(names)?;
    if base.info().public_source.as_ref() != Some(&manifest.names_source) {
        return Err("names source provenance mismatch".into());
    }
    let expected_entities = base.info().entities as u64;
    let expected_aliases = base.info().aliases as u64;
    drop(base);
    stage(
        started,
        "both source hashes and native names format verified",
    );
    File::options()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(error)?;
    let mut db = Connection::open_with_flags(
        output.canonicalize().map_err(error)?,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(error)?;
    configure(&db)?;
    db.execute_batch(
        "PRAGMA foreign_keys=OFF; PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;",
    )
    .map_err(error)?;
    let page: i64 = db
        .query_row("PRAGMA page_size", [], |r| r.get(0))
        .map_err(error)?;
    let pages = i64::try_from(MAX_OUTPUT_BYTES - 1)
        .map_err(error)?
        .checked_div(page)
        .filter(|n| *n > 0)
        .ok_or("invalid page size")?;
    db.pragma_update(None, "max_page_count", pages)
        .map_err(error)?;
    db.execute("ATTACH DATABASE ? AS names_source", [readonly_uri(names)?])
        .map_err(error)?;
    db.execute("ATTACH DATABASE ? AS source", [readonly_uri(claims)?])
        .map_err(error)?;
    db.execute_batch(
        "PRAGMA source.cache_size=-65536; PRAGMA source.mmap_size=0;
        PRAGMA names_source.cache_size=-65536; PRAGMA names_source.mmap_size=0;",
    )
    .map_err(error)?;
    db.progress_handler(
        100_000,
        Some(move || started.elapsed().as_secs() >= MAX_SECONDS),
    )
    .map_err(error)?;
    let tx = db.transaction().map_err(error)?;
    let source_kind: String = tx
        .query_row(
            "SELECT type FROM source.sqlite_schema WHERE name='edges'",
            [],
            |r| r.get(0),
        )
        .map_err(error)?;
    if source_kind != "table" {
        return Err("claims projection must contain a real edges table".into());
    }
    for f in &manifest.files {
        let found: i64 = tx
            .query_row(
                "SELECT count(*) FROM source.edges WHERE row_id>? AND row_id<=?",
                params![
                    i64::try_from(f.row_id_offset).map_err(error)?,
                    i64::try_from(f.row_id_offset + f.source_rows).map_err(error)?
                ],
                |r| r.get(0),
            )
            .map_err(error)?;
        if u64::try_from(found).map_err(error)? != f.kept_rows {
            return Err("claims projection file row count mismatch".into());
        }
    }
    if count(&tx, "SELECT count(*) FROM source.edges")? != manifest.statements
        || count(&tx, "SELECT count(*) FROM source.edges WHERE qualified=1")?
            != manifest.qualified_statements
    {
        return Err("claims projection total counts mismatch".into());
    }
    if count(&tx, "SELECT count(*) FROM source.edges WHERE typeof(row_id)!='integer' OR row_id<=0
        OR typeof(subject)!='integer' OR subject<=0 OR typeof(property)!='integer' OR property<=0
        OR typeof(object)!='integer' OR object<=0 OR typeof(preferred)!='integer' OR preferred NOT IN (0,1)
        OR typeof(qualified)!='integer' OR qualified NOT IN (0,1) OR typeof(statement)!='text'
        OR upper(substr(statement,1,instr(statement,'$')-1))!='Q'||subject
        OR instr(statement,'$')<=1 OR length(statement)<=instr(statement,'$')")? != 0 {
        return Err("invalid numeric claims projection row".into());
    }
    stage(started, "claims projection validated");
    for (_, _, sql) in super::SCHEMA
        .iter()
        .filter(|(kind, name, _)| *kind == "table" && *name != "aliases")
        .chain(pilot::SCHEMA.iter())
    {
        tx.execute_batch(sql).map_err(error)?;
    }
    tx.execute_batch(
        "INSERT INTO entities SELECT id FROM names_source.entities;
        INSERT INTO aliases SELECT * FROM names_source.aliases;",
    )
    .map_err(error)?;
    if count(&tx, "SELECT count(*) FROM entities")? != expected_entities
        || count(&tx, "SELECT count(*) FROM aliases")? != expected_aliases
    {
        return Err("native names copy counts mismatch".into());
    }
    stage(started, "native aliases copied without renormalization");
    tx.execute_batch("INSERT OR IGNORE INTO entities SELECT 'Q'||subject FROM source.edges GROUP BY subject ORDER BY 1;
        INSERT INTO edges SELECT 'Q'||subject,'P'||property,'Q'||object FROM source.edges GROUP BY subject,property,object ORDER BY 1,2,3;
        INSERT INTO statements SELECT row_id,'Q'||subject,'P'||property,'Q'||object,preferred,qualified,statement FROM source.edges ORDER BY row_id;")
        .map_err(error)?;
    let entities = count(&tx, "SELECT count(*) FROM entities")?;
    let edges = count(&tx, "SELECT count(*) FROM edges")?;
    if count(&tx, "SELECT count(*) FROM statements")? != manifest.statements {
        return Err("native statement witness count mismatch".into());
    }
    stage(started, "claims and witnesses loaded");
    for (_, _, sql) in super::SCHEMA.iter().filter(|(kind, _, _)| *kind == "index") {
        tx.execute_batch(sql).map_err(error)?;
    }
    stage(started, "secondary indexes built");
    tx.execute_batch(
        "INSERT INTO metadata SELECT key,value FROM names_source.metadata
        WHERE key IN ('accepted_name_rows','excluded_short_name_rows','claim_semantics');",
    )
    .map_err(error)?;
    for (key, value) in [
        ("normalizer", normalizer_version()),
        ("source_kind", "audited-wikidata-claims-snapshot".into()),
        (
            "public_claims",
            serde_json::to_string(&manifest).map_err(error)?,
        ),
        ("entities", entities.to_string()),
        ("aliases", expected_aliases.to_string()),
        ("edges", edges.to_string()),
        ("statements", manifest.statements.to_string()),
        (
            "qualified_statements",
            manifest.qualified_statements.to_string(),
        ),
    ] {
        tx.execute("INSERT INTO metadata VALUES (?,?)", params![key, value])
            .map_err(error)?;
    }
    let integrity: String = tx
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(error)?;
    if integrity != "ok" {
        return Err(format!("claims snapshot integrity failure: {integrity}"));
    }
    if tx
        .prepare("PRAGMA foreign_key_check")
        .map_err(error)?
        .exists([])
        .map_err(error)?
    {
        return Err("claims snapshot foreign key failure".into());
    }
    stage(started, "integrity and references checked");
    tx.pragma_update(None, "application_id", APPLICATION_ID)
        .map_err(error)?;
    tx.pragma_update(None, "user_version", 3).map_err(error)?;
    tx.commit().map_err(error)?;
    drop(db);
    if started.elapsed().as_secs() >= MAX_SECONDS
        || output.metadata().map_err(error)?.len() >= MAX_OUTPUT_BYTES
    {
        return Err("claims snapshot resource gate exceeded".into());
    }
    stage(started, "committed");
    DiskStore::open(output)
}
