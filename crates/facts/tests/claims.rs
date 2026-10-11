use glassmind_everyday::{Example, IntentModel, TrainOptions};
use glassmind_facts::{
    Store,
    disk::{DiskStore, claims, pilot},
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
    names: PathBuf,
    projection: PathBuf,
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
fn file(path: &str, table: &str, raw: u64, kept: u64, offset: u64) -> pilot::SourceFile {
    pilot::SourceFile {
        table: table.into(),
        path: path.into(),
        revision: "a".repeat(40),
        sha256: "b".repeat(64),
        bytes: 1,
        source_rows: raw,
        kept_rows: kept,
        row_id_offset: offset,
    }
}
impl Fixture {
    fn new() -> Result<Self> {
        let root = std::env::temp_dir().join(format!(
            "glassmind-claims-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root)?;
        let raw = root.join("pilot.sqlite");
        let db = Connection::open(&raw)?;
        db.execute_batch("CREATE TABLE names(row_id INTEGER PRIMARY KEY,entity INTEGER,name TEXT,normalized TEXT,language INTEGER,kind INTEGER);
            CREATE TABLE edges(row_id INTEGER PRIMARY KEY,subject INTEGER,property INTEGER,object INTEGER,preferred INTEGER,qualified INTEGER,statement TEXT);
            INSERT INTO names VALUES(1,1,'Atlas','wrong',0,0),(2,2,'東京','wrong',0,0),(3,1,'IN','wrong',0,0),(4,3,'AͅB','wrong',0,0),(5,3,'!','wrong',0,0);
            INSERT INTO edges VALUES(1,1,1,99,0,0,'Q1$old');")?;
        drop(db);
        let base = pilot::SourceManifest {
            schema: 1,
            license: "CC0-1.0".into(),
            publisher: "permutans".into(),
            snapshot: "20260928".into(),
            scope: "Synthetic fixture, not a public artifact".into(),
            pilot_sha256: hash(&raw)?,
            pilot_bytes: raw.metadata()?.len(),
            canonical_audit_sha256: "c".repeat(64),
            names: 5,
            statements: 1,
            qualified_statements: 0,
            files: vec![
                file("all/00.parquet", "claims", 1, 1, 0),
                file("en/0.parquet", "labels", 5, 5, 0),
            ],
        };
        let m = root.join("base.json");
        std::fs::write(&m, serde_json::to_vec(&base)?)?;
        let names = root.join("names ?#%.sqlite");
        drop(pilot::import_file(&raw, &m, &names)?);
        let projection = root.join("claims ?#%.sqlite");
        let db = Connection::open(&projection)?;
        db.execute_batch("CREATE TABLE edges(row_id INTEGER PRIMARY KEY,subject INTEGER,property INTEGER,object INTEGER,preferred INTEGER,qualified INTEGER,statement TEXT);
            INSERT INTO edges VALUES(1,1,1,2,0,0,'q1$first'),(3,1,1,2,1,1,'Q1$second'),(5,4,1,2,0,0,'Q4$unnamed');")?;
        drop(db);
        let manifest = root.join("manifest.json");
        let full = claims::SourceManifest {
            schema: 1,
            scope: "Two independent synthetic claim files".into(),
            names_sha256: hash(&names)?,
            names_bytes: names.metadata()?.len(),
            names_source: base,
            projection_sha256: hash(&projection)?,
            projection_bytes: projection.metadata()?.len(),
            canonical_audit_sha256: "d".repeat(64),
            statements: 3,
            qualified_statements: 1,
            files: vec![
                file("all/00.parquet", "claims", 2, 1, 0),
                file("all/01.parquet", "claims", 3, 2, 2),
            ],
        };
        std::fs::write(&manifest, serde_json::to_vec(&full)?)?;
        let output = root.join("file:literal?mode=ro#%.sqlite");
        Ok(Self {
            root,
            names,
            projection,
            manifest,
            output,
        })
    }
    fn read(&self) -> Result<claims::SourceManifest> {
        Ok(serde_json::from_slice(&std::fs::read(&self.manifest)?)?)
    }
    fn write(&self, m: &claims::SourceManifest) -> Result {
        std::fs::write(&self.manifest, serde_json::to_vec(m)?)?;
        Ok(())
    }
    fn import(&self) -> std::result::Result<DiskStore, String> {
        claims::import_file(&self.names, &self.projection, &self.manifest, &self.output)
    }
}

#[test]
fn shards_preserve_witnesses_replace_old_claims_and_reuse_native_names() -> Result {
    let f = Fixture::new()?;
    let before = (hash(&f.names)?, hash(&f.projection)?);
    let disk = f.import()?;
    assert_eq!(
        (disk.info().entities, disk.info().aliases, disk.info().edges),
        (4, 4, 2)
    );
    assert!(disk.info().public_source.is_none());
    assert_eq!(
        disk.info().public_claims.as_ref().map(|m| m.files.len()),
        Some(2)
    );
    let json: Store = serde_json::from_value(serde_json::json!({"entities":{
        "Q1":{"aliases":["Atlas","IN"],"claims":{"P1":["Q2"]}},
        "Q2":{"aliases":["東京"],"claims":{}},"Q3":{"aliases":["AͅB","!"],"claims":{}},
        "Q4":{"aliases":[],"claims":{"P1":["Q2"]}}
    }}))?;
    for q in ["Atlas", "IN", "in", "東京", "AͅB", "!", "wrong"] {
        assert_eq!(disk.link(q)?, json.link(q), "{q}");
    }
    for (q, relation) in [("Who created Atlas?", "P1"), ("Who mentions 東京?", "R1")] {
        let mut model = IntentModel::train(
            &[
                Example {
                    text: q.into(),
                    label: relation.into(),
                },
                Example {
                    text: "Other question".into(),
                    label: "P9".into(),
                },
            ],
            TrainOptions {
                epochs: 5,
                char_weight: 0.0,
                min_df: 1,
            },
        )?;
        model.policy.min_score = 0.0;
        model.policy.min_margin = 0.0;
        assert_eq!(model.predict(q).label, relation);
        assert_eq!(
            serde_json::to_value(disk.answer(&model, q)?)?,
            serde_json::to_value(json.answer(&model, q))?
        );
        assert_eq!(
            serde_json::to_value(disk.answer_set(&model, q)?)?,
            serde_json::to_value(json.answer_set(&model, q))?
        );
    }
    assert!(disk.query_plans()?.iter().all(|p| p.contains("SEARCH")));
    drop(disk);
    let db = Connection::open(f.output.canonicalize()?)?;
    let rows = db
        .prepare("SELECT source_row,guid,qualified FROM statements ORDER BY source_row")?
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(
        rows,
        vec![
            (1, "q1$first".into(), 0),
            (3, "Q1$second".into(), 1),
            (5, "Q4$unnamed".into(), 0)
        ]
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM edges WHERE object='Q99'", [], |r| r
            .get::<_, i64>(
            0
        ))?,
        0
    );
    drop(db);
    assert!(f.import().is_err());
    assert_eq!(DiskStore::open(&f.output)?.info().edges, 2);
    assert_eq!(before, (hash(&f.names)?, hash(&f.projection)?));
    Ok(())
}

#[test]
fn provenance_mapping_and_input_hash_fail_closed() -> Result {
    for change in 0..8 {
        let f = Fixture::new()?;
        let mut m = f.read()?;
        match change {
            0 => m.names_sha256 = "0".repeat(64),
            1 => m.projection_sha256 = "0".repeat(64),
            2 => m.files[1].row_id_offset = 1,
            3 => m.files[1].revision = "e".repeat(40),
            4 => m.files[1].path = m.files[0].path.clone(),
            5 => {
                m.files[0].kept_rows = 2;
                m.files[1].kept_rows = 1;
            }
            6 => m.names_source.scope = "Mismatched provenance".into(),
            _ => {
                m.files[1].source_rows = 2;
            }
        }
        f.write(&m)?;
        assert!(f.import().is_err(), "change {change}");
        assert!(DiskStore::open(&f.output).is_err(), "change {change}");
    }
    Ok(())
}

#[test]
fn invalid_projection_rows_never_publish_a_readable_header() -> Result {
    for mutation in [
        "UPDATE edges SET subject=0 WHERE row_id=1",
        "UPDATE edges SET property='P1' WHERE row_id=1",
        "UPDATE edges SET object=-2 WHERE row_id=1",
        "UPDATE edges SET preferred=2 WHERE row_id=1",
        "UPDATE edges SET statement='Q4$wrong' WHERE row_id=1",
        "UPDATE edges SET row_id=2 WHERE row_id=3",
        "UPDATE edges SET qualified=0 WHERE row_id=3",
    ] {
        let f = Fixture::new()?;
        let db = Connection::open(&f.projection)?;
        db.execute_batch(mutation)?;
        drop(db);
        let mut m = f.read()?;
        m.projection_sha256 = hash(&f.projection)?;
        m.projection_bytes = f.projection.metadata()?.len();
        f.write(&m)?;
        assert!(f.import().is_err(), "{mutation}");
        assert!(DiskStore::open(&f.output).is_err());
    }
    Ok(())
}
