use glassmind_everyday::{Example, IntentModel, TrainOptions};
use glassmind_facts::{
    Entity, EntityId, PropertyId, Store,
    disk::{DiskStore, MAX_DISK_QUESTION_BYTES},
};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "glassmind-disk-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)?;
        Ok(Self(path))
    }
    fn database(&self) -> PathBuf {
        self.0.join("facts.sqlite")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn store(rows: &[(&str, &[&str])]) -> Store {
    Store {
        entities: rows
            .iter()
            .map(|(id, aliases)| {
                (
                    EntityId((*id).into()),
                    Entity {
                        aliases: aliases.iter().map(|a| (*a).to_owned()).collect(),
                        claims: BTreeMap::new(),
                    },
                )
            })
            .collect(),
    }
}
fn add(store: &mut Store, subject: &str, property: &str, objects: Vec<EntityId>) {
    store
        .entities
        .entry(EntityId(subject.into()))
        .or_insert(Entity {
            aliases: vec![],
            claims: BTreeMap::new(),
        })
        .claims
        .insert(PropertyId(property.into()), objects);
}
fn model() -> Result<IntentModel> {
    let mut model = IntentModel::train(
        &[
            Example {
                text: "Who created Atlas?".into(),
                label: "P1".into(),
            },
            Example {
                text: "Where located other?".into(),
                label: "R2".into(),
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
    assert_eq!(model.predict("Who created Atlas?").label, "P1");
    Ok(model)
}

#[test]
fn unicode_alias_ties_and_acronym_case_match_the_existing_store() -> Result {
    let scratch = Scratch::new()?;
    let source = store(&[
        ("Q1", &["IN", "Example Institute", "ACME"]),
        ("Q2", &["Example Institute", "Go"]),
        (
            "Q3",
            &[
                "Example Institute of Music",
                "D’Ávila",
                "a\u{345}b",
                "ΟΣ",
                "\"quoted\" name",
            ],
        ),
        ("Q4", &["Acme", "BB"]),
        ("Q5", &["Aa"]),
        ("Q6", &["İstanbul", "東京", "strange\0name"]),
    ]);
    let disk = DiskStore::import_bytes(&serde_json::to_vec(&source)?, &scratch.database())?;
    let cases = [
        ("Which language is spoken in this film?", None),
        ("Where is (IN)?", Some("Q1")),
        ("Where is INNER?", None),
        ("Where is example institute?", None),
        ("Where is Example Institute of Music?", Some("Q3")),
        ("Who founded acme?", None),
        ("Does acme go abroad?", None),
        ("What language is go in?", None),
        ("aa bb", None),
        ("D’Ávila", Some("Q3")),
        ("a\u{345}b", Some("Q3")),
        ("\"quoted\" name", Some("Q3")),
        ("İSTANBUL", Some("Q6")),
        ("東京", Some("Q6")),
        ("strange name", Some("Q6")),
        ("ΟΣ", Some("Q3")),
        ("ος", None),
        ("nothing matches", None),
        ("", None),
    ];
    for (question, expected) in cases {
        let expected = expected.map(|id| EntityId(id.into()));
        assert_eq!(source.link(question), expected, "legacy {question:?}");
        assert_eq!(disk.link(question)?, expected, "disk {question:?}");
    }
    assert!(disk.query_plans()?.iter().all(|p| p.contains("SEARCH")));
    let before = std::fs::read(scratch.database())?;
    drop(disk);
    let reopened = DiskStore::open(&scratch.database())?;
    assert_eq!(reopened.link("東京")?, Some(EntityId("Q6".into())));
    assert_eq!(before, std::fs::read(scratch.database())?);
    Ok(())
}

#[test]
fn forward_inverse_cardinality_order_and_refusals_match() -> Result {
    let scratch = Scratch::new()?;
    let mut source = store(&[("Q100", &["Atlas"])]);
    let objects: Vec<_> = (1..=25)
        .rev()
        .chain(1..=10)
        .map(|i| EntityId(format!("Q{i}")))
        .collect();
    add(&mut source, "Q100", "P1", objects);
    add(
        &mut source,
        "Q100",
        "P3",
        vec![EntityId("Q0001".into()), EntityId("Q0001".into())],
    );
    add(
        &mut source,
        "Q100",
        "P4",
        vec![EntityId("Q922337203685477580800".into())],
    );
    for i in 1..=25 {
        add(
            &mut source,
            &format!("Q{i}"),
            "P1",
            vec![EntityId("Q100".into()); 2],
        );
    }
    let disk = DiskStore::import_bytes(&serde_json::to_vec(&source)?, &scratch.database())?;
    assert_eq!(disk.info().edges, 52);
    let original = model()?;
    let question = "Who created Atlas?";
    for relation in [
        "P1",
        "R1",
        "P2",
        "R9",
        "P3",
        "P4",
        "unsupported",
        "Pbad",
        "R",
        "",
        "😀",
    ] {
        let mut model = original.clone();
        for label in &mut model.labels {
            if label == "P1" {
                *label = relation.into();
            }
        }
        for uncertain in [false, true] {
            model.policy.min_score = if uncertain { f32::MAX } else { 0.0 };
            assert_eq!(
                serde_json::to_value(disk.answer(&model, question)?)?,
                serde_json::to_value(source.answer(&model, question))?,
                "answer {relation} uncertain={uncertain}"
            );
            let set = disk.answer_set(&model, question)?;
            assert_eq!(
                serde_json::to_value(&set)?,
                serde_json::to_value(source.answer_set(&model, question))?,
                "set {relation} uncertain={uncertain}"
            );
            if !uncertain && matches!(relation, "P1" | "R1") {
                assert_eq!(
                    (set.total_snapshot_matches, set.returned, set.truncated),
                    (25, 10, true)
                );
                assert_eq!(
                    set.answer.answers.iter().next().map(|id| id.0.as_str()),
                    Some("Q1")
                );
                assert_eq!(
                    set.answer.answers.iter().nth(1).map(|id| id.0.as_str()),
                    Some("Q10")
                );
            }
        }
    }
    for question in ["No entity here", "", "Who created Atlas?"] {
        assert_eq!(
            serde_json::to_value(disk.answer(&original, question)?)?,
            serde_json::to_value(source.answer(&original, question))?
        );
    }
    Ok(())
}

#[test]
fn invalid_schema_input_limits_and_failed_query_recovery() -> Result {
    let scratch = Scratch::new()?;
    let source = store(&[("Q1", &["Atlas"])]);
    let bytes = serde_json::to_vec(&source)?;
    let disk = DiskStore::import_bytes(&bytes, &scratch.database())?;
    assert!(DiskStore::import_bytes(&bytes, &scratch.database()).is_err());
    assert!(disk.link(&"x".repeat(MAX_DISK_QUESTION_BYTES + 1)).is_err());
    assert!(disk.link(&"word ".repeat(129)).is_err());
    assert_eq!(disk.link("Atlas")?, Some(EntityId("Q1".into())));
    assert!(DiskStore::open(&scratch.0.join("missing.sqlite")).is_err());
    assert!(!scratch.0.join("missing.sqlite").exists());
    assert!(DiskStore::import_bytes(b"bad json", &scratch.0.join("bad.sqlite")).is_err());
    assert!(!scratch.0.join("bad.sqlite").exists());
    let invalid = store(&[("wrong", &["Atlas"])]);
    assert!(
        DiskStore::import_bytes(
            &serde_json::to_vec(&invalid)?,
            &scratch.0.join("invalid.sqlite")
        )
        .is_err()
    );
    drop(disk);
    let db = rusqlite::Connection::open(scratch.database())?;
    db.execute(
        "UPDATE metadata SET value='old-normalizer' WHERE key='normalizer'",
        [],
    )?;
    assert!(DiskStore::open(&scratch.database()).is_err());
    db.execute(
        "UPDATE metadata SET value=? WHERE key='normalizer'",
        [format!(
            "whole-token-lowercase-v1/unicode-{}.{}.{}",
            char::UNICODE_VERSION.0,
            char::UNICODE_VERSION.1,
            char::UNICODE_VERSION.2
        )],
    )?;
    db.execute_batch("DROP INDEX edges_inverse; CREATE INDEX edges_inverse ON edges(subject)")?;
    assert!(DiskStore::open(&scratch.database()).is_err());
    Ok(())
}

#[test]
fn connection_keeps_one_snapshot_across_external_wal_commits() -> Result {
    let scratch = Scratch::new()?;
    let source = store(&[("Q1", &["Atlas"])]);
    drop(DiskStore::import_bytes(
        &serde_json::to_vec(&source)?,
        &scratch.database(),
    )?);
    let writer = rusqlite::Connection::open(scratch.database())?;
    writer.execute_batch("PRAGMA journal_mode=WAL")?;
    let disk = DiskStore::open(&scratch.database())?;
    assert_eq!(disk.link("Atlas")?, Some(EntityId("Q1".into())));
    writer.execute(
        "UPDATE aliases SET original='Changed', normalized='changed'",
        [],
    )?;
    assert_eq!(disk.link("Atlas")?, Some(EntityId("Q1".into())));
    assert_eq!(disk.link("Changed")?, None);
    let reopened = DiskStore::open(&scratch.database())?;
    assert_eq!(reopened.link("Changed")?, Some(EntityId("Q1".into())));
    assert_eq!(reopened.link("Atlas")?, None);
    Ok(())
}

#[test]
fn commands_preserve_stream_rows_and_fail_closed_on_oversized_input() -> Result {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let scratch = Scratch::new()?;
    let mut source = store(&[("Q1", &["Atlas"])]);
    add(&mut source, "Q1", "P1", vec![EntityId("Q2".into())]);
    let index = scratch.0.join("index.json");
    let model_path = scratch.0.join("model.json");
    std::fs::write(&index, serde_json::to_vec(&source)?)?;
    std::fs::write(&model_path, serde_json::to_vec(&model()?)?)?;
    let binary = env!("CARGO_BIN_EXE_glassmind-facts");
    let result = Command::new(binary)
        .arg("index-disk")
        .arg("--index")
        .arg(&index)
        .arg("--database")
        .arg(scratch.database())
        .output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result = Command::new(binary)
        .arg("ask-disk")
        .arg("--model")
        .arg(&model_path)
        .arg("--database")
        .arg(scratch.database())
        .arg("--set")
        .arg("Who created Atlas?")
        .output()?;
    assert!(result.status.success());
    let set: serde_json::Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(
        set.get("total_snapshot_matches"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(set.get("answers"), Some(&serde_json::json!(["Q2"])));
    for (input, expected_rows, success) in [
        (
            b"Who created Atlas?\r\n\nWho created Atlas?".to_vec(),
            3,
            true,
        ),
        (vec![b'x'; MAX_DISK_QUESTION_BYTES + 1], 0, false),
        (vec![0xff], 0, false),
    ] {
        let mut child = Command::new(binary)
            .arg("predict-disk")
            .arg("--model")
            .arg(&model_path)
            .arg("--database")
            .arg(scratch.database())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("missing child stdin")?
            .write_all(&input)?;
        let result = child.wait_with_output()?;
        assert_eq!(result.status.success(), success);
        let text = String::from_utf8(result.stdout)?;
        assert_eq!(text.lines().count(), expected_rows);
        if success {
            let replies: Vec<serde_json::Value> = text
                .lines()
                .map(serde_json::from_str)
                .collect::<std::result::Result<_, _>>()?;
            assert_eq!(
                replies.first().and_then(|r| r.get("answers")),
                Some(&serde_json::json!(["Q2"]))
            );
            assert_eq!(
                replies.get(1).and_then(|r| r.get("answers")),
                Some(&serde_json::json!([]))
            );
            assert_eq!(
                replies.last().and_then(|r| r.get("answers")),
                Some(&serde_json::json!(["Q2"]))
            );
        }
    }
    Ok(())
}
