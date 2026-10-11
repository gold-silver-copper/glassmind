use glassmind_facts::{
    Store,
    disk::{
        DiskStore,
        pilot::{SourceFile, SourceManifest, import_file},
    },
};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture {
    root: PathBuf,
    source: PathBuf,
    manifest: PathBuf,
    output: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn hash(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}
impl Fixture {
    fn new() -> Result<Self> {
        let root = std::env::temp_dir().join(format!(
            "glassmind-pilot-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root)?;
        // URI metacharacters must not change the attached source path or mode.
        let source = root.join("pilot ?#%.sqlite");
        let manifest = root.join("manifest.json");
        let output = root.join("native.sqlite");
        let db = Connection::open(&source)?;
        db.execute_batch("CREATE TABLE names(row_id INTEGER PRIMARY KEY, entity INTEGER NOT NULL, name TEXT NOT NULL,
            normalized TEXT NOT NULL, language INTEGER NOT NULL, kind INTEGER NOT NULL);
            CREATE TABLE edges(row_id INTEGER PRIMARY KEY,subject INTEGER NOT NULL, property INTEGER NOT NULL,
            object INTEGER NOT NULL,preferred INTEGER NOT NULL,qualified INTEGER NOT NULL,statement TEXT NOT NULL);
            INSERT INTO names VALUES (1,1,'Atlas','deliberately wrong',0,0),(2,2,'東京','wrong',0,0),
            (3,4,'!','wrong',0,0),(4,5,'AͅB','wrong',0,0),(5,1,'Atlas','wrong',0,1),
            (6,1,'IN','wrong',0,1),(7,1,'x','wrong',0,1),(8,1,'Atlas','wrong',1,0),
            (9,2,'東京','wrong',1,0),(10,1,'Atlas','wrong',1,1);
            INSERT INTO edges VALUES (1,1,1,2,0,0,'q1$first'),(2,1,1,2,1,1,'Q1$second'),(3,3,2,1,0,0,'Q3$third');")?;
        drop(db);
        let files = [
            ("claims", "all/test.parquet", 3, 0),
            ("labels", "en/test.parquet", 4, 0),
            ("aliases", "en/test.parquet", 3, 4),
            ("labels", "mul/test.parquet", 2, 7),
            ("aliases", "mul/test.parquet", 1, 9),
        ]
        .into_iter()
        .map(|(table, path, rows, offset)| SourceFile {
            table: table.into(),
            path: path.into(),
            revision: "a".repeat(40),
            sha256: "b".repeat(64),
            bytes: 1,
            source_rows: rows,
            kept_rows: rows,
            row_id_offset: offset,
        })
        .collect();
        let provenance = SourceManifest {
            schema: 1,
            license: "CC0-1.0".into(),
            publisher: "permutans".into(),
            snapshot: "20260928".into(),
            scope: "Synthetic test fixture; not a public source claim".into(),
            pilot_sha256: hash(&source)?,
            pilot_bytes: source.metadata()?.len(),
            canonical_audit_sha256: "c".repeat(64),
            names: 10,
            statements: 3,
            qualified_statements: 1,
            files,
        };
        std::fs::write(&manifest, serde_json::to_vec(&provenance)?)?;
        Ok(Self {
            root,
            source,
            manifest,
            output,
        })
    }
    fn repin(&self) -> Result {
        let mut m: SourceManifest = serde_json::from_slice(&std::fs::read(&self.manifest)?)?;
        m.pilot_sha256 = hash(&self.source)?;
        m.pilot_bytes = self.source.metadata()?.len();
        std::fs::write(&self.manifest, serde_json::to_vec(&m)?)?;
        Ok(())
    }
}

#[test]
fn native_projection_preserves_witnesses_provenance_and_json_link_semantics() -> Result {
    let fixture = Fixture::new()?;
    let before = hash(&fixture.source)?;
    let disk = import_file(&fixture.source, &fixture.manifest, &fixture.output)?;
    assert_eq!(
        (disk.info().entities, disk.info().aliases, disk.info().edges),
        (5, 4, 2)
    );
    assert!(disk.info().source_json_sha256.is_none());
    assert_eq!(
        disk.info().public_source.as_ref().map(|s| s.names),
        Some(10)
    );
    let json: Store = serde_json::from_value(serde_json::json!({"entities": {
        "Q1":{"aliases":["Atlas","Atlas","IN","x"],"claims":{"P1":["Q2","Q2"]}},
        "Q2":{"aliases":["東京"],"claims":{}},"Q3":{"aliases":[],"claims":{"P2":["Q1"]}},
        "Q4":{"aliases":["!"],"claims":{}},"Q5":{"aliases":["AͅB"],"claims":{}}
    }}))?;
    for question in [
        "Who created Atlas?",
        "東京",
        "AͅB",
        "IN",
        "in",
        "x",
        "!",
        "wrong",
        "deliberately wrong",
    ] {
        assert_eq!(disk.link(question)?, json.link(question), "{question}");
    }
    assert!(
        disk.query_plans()?
            .iter()
            .all(|plan| plan.contains("SEARCH"))
    );
    drop(disk);
    let db =
        Connection::open_with_flags(&fixture.output, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let provenance: (i64,i64,i64) = db.query_row("SELECT source_mask,occurrences,first_source_row FROM aliases WHERE entity='Q1' AND original='Atlas'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    assert_eq!(provenance, (15, 4, 1));
    assert_eq!(
        db.query_row(
            "SELECT value FROM metadata WHERE key='excluded_short_name_rows'",
            [],
            |r| r.get::<_, String>(0)
        )?,
        "2"
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM statements", [], |r| r
            .get::<_, i64>(0))?,
        3
    );
    let witness: (String, i64, i64) = db.query_row(
        "SELECT guid,preferred,qualified FROM statements WHERE source_row=2",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    assert_eq!(witness, ("Q1$second".into(), 1, 1));
    assert_eq!(
        db.query_row("SELECT guid FROM statements WHERE source_row=1", [], |r| {
            r.get::<_, String>(0)
        })?,
        "q1$first"
    );
    assert_eq!(hash(&fixture.source)?, before);
    assert!(import_file(&fixture.source, &fixture.manifest, &fixture.output).is_err());
    assert_eq!(DiskStore::open(&fixture.output)?.info().edges, 2);
    Ok(())
}

#[test]
fn hash_mapping_and_invalid_rows_fail_without_usable_partial_output() -> Result {
    for mutation in [
        "UPDATE names SET language=1 WHERE row_id=1",
        "UPDATE edges SET subject=-1 WHERE row_id=1",
        "UPDATE edges SET statement='Q9$wrong' WHERE row_id=1",
        "UPDATE names SET entity=1.5 WHERE row_id=1",
    ] {
        let fixture = Fixture::new()?;
        Connection::open(&fixture.source)?.execute_batch(mutation)?;
        fixture.repin()?;
        assert!(
            import_file(&fixture.source, &fixture.manifest, &fixture.output).is_err(),
            "{mutation}"
        );
        assert!(DiskStore::open(&fixture.output).is_err());
    }
    let fixture = Fixture::new()?;
    Connection::open(&fixture.source)?
        .execute_batch("UPDATE names SET name='changed' WHERE row_id=1")?;
    assert!(import_file(&fixture.source, &fixture.manifest, &fixture.output).is_err());
    assert!(!fixture.output.exists());
    Ok(())
}

#[test]
fn command_line_import_reopens_and_preserves_version_one_output_shape() -> Result {
    let fixture = Fixture::new()?;
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_glassmind-facts"))
        .args(["import-wikidata-pilot", "--pilot"])
        .arg(&fixture.source)
        .arg("--manifest")
        .arg(&fixture.manifest)
        .arg("--database")
        .arg(&fixture.output)
        .output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output: serde_json::Value = serde_json::from_slice(&result.stdout)?;
    assert!(output.get("source_json_sha256").is_none());
    assert_eq!(output["public_source"]["names"], 10);
    let v1 = DiskStore::import_bytes(br#"{"entities":{}}"#, &fixture.root.join("v1.sqlite"))?;
    let info = serde_json::to_value(v1.info())?;
    assert_eq!(info.as_object().map(|o| o.len()), Some(4));
    assert!(info["source_json_sha256"].is_string());
    Ok(())
}
