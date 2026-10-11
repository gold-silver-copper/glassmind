//! Conservative calendar-day evidence from one recorded bounded statement.
use crate::{EdgeWitness, RefusalReason, Store, UnknownReason, Verification};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_TEMPORAL_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_TEMPORAL_INDEX_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_TEMPORAL_RECORDS: usize = 100_000;
pub const TEMPORAL_POLICY: &str = "latest_start_day < requested_day < earliest_end_day";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct CalendarDate(String);
impl CalendarDate {
    pub fn parse(s: &str) -> Result<Self, String> {
        let b = s.as_bytes();
        if b.len() != 10
            || b.get(4) != Some(&b'-')
            || b.get(7) != Some(&b'-')
            || !b
                .iter()
                .enumerate()
                .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        {
            return Err("expected exact ASCII YYYY-MM-DD".into());
        }
        let y = number(s.get(..4).ok_or("invalid year")?)?;
        let m = number(s.get(5..7).ok_or("invalid month")?)?;
        let d = number(s.get(8..).ok_or("invalid day")?)?;
        if y == 0 || m == 0 || m > 12 || d == 0 || d > days(y, m) {
            return Err("invalid Gregorian calendar date".into());
        }
        Ok(Self(s.into()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
fn number(s: &str) -> Result<u32, String> {
    s.parse().map_err(|_| "invalid numeric field".into())
}
fn days(y: u32, m: u32) -> u32 {
    match m {
        2 => {
            if y.is_multiple_of(400) || (y.is_multiple_of(4) && !y.is_multiple_of(100)) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTime {
    pub time: String,
    pub timezone: i64,
    pub before: i64,
    pub after: i64,
    pub precision: u8,
    pub calendarmodel: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundRange {
    pub raw: RawTime,
    pub earliest: CalendarDate,
    pub latest: CalendarDate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimSkipReason {
    Deprecated,
    ExtraQualifier,
    MissingBound,
    DuplicateBound,
    NonvalueBound,
    InvalidBound,
    UnsupportedYear,
    UnsupportedCalendar,
    UnsupportedPrecision,
    NonzeroTimezone,
    NonzeroUncertainty,
    NoncanonicalTime,
    InvalidCalendarFields,
    InvertedInterval,
    OutsideStrictInterior,
}
fn field<'a>(value: &'a Value, path: &str) -> &'a Value {
    value.pointer(path).unwrap_or(&Value::Null)
}
fn range(snak: &Value, property: &str) -> Result<BoundRange, ClaimSkipReason> {
    use ClaimSkipReason::*;
    if field(snak, "/snaktype") != "value" {
        return Err(NonvalueBound);
    }
    if field(snak, "/property") != property
        || field(snak, "/datatype") != "time"
        || field(snak, "/datavalue/type") != "time"
    {
        return Err(InvalidBound);
    }
    let raw: RawTime = serde_json::from_value(field(snak, "/datavalue/value").clone())
        .map_err(|_| InvalidBound)?;
    if raw.timezone != 0 {
        return Err(NonzeroTimezone);
    }
    if raw.before != 0 || raw.after != 0 {
        return Err(NonzeroUncertainty);
    }
    if raw.calendarmodel != "http://www.wikidata.org/entity/Q1985727" {
        return Err(UnsupportedCalendar);
    }
    if !(9..=11).contains(&raw.precision) {
        return Err(UnsupportedPrecision);
    }
    if raw.time.starts_with('-') {
        return Err(UnsupportedYear);
    }
    let b = raw.time.as_bytes();
    if b.len() != 21
        || b.first() != Some(&b'+')
        || b.get(5) != Some(&b'-')
        || b.get(8) != Some(&b'-')
        || b.get(11..) != Some(b"T00:00:00Z".as_slice())
        || !b
            .iter()
            .enumerate()
            .filter(|(i, _)| matches!(i,1..=4|6..=7|9..=10))
            .all(|(_, c)| c.is_ascii_digit())
    {
        return Err(NoncanonicalTime);
    }
    let y = number(raw.time.get(1..5).ok_or(NoncanonicalTime)?).map_err(|_| NoncanonicalTime)?;
    let m = number(raw.time.get(6..8).ok_or(NoncanonicalTime)?).map_err(|_| NoncanonicalTime)?;
    let d = number(raw.time.get(9..11).ok_or(NoncanonicalTime)?).map_err(|_| NoncanonicalTime)?;
    if y == 0 {
        return Err(UnsupportedYear);
    }
    if m > 12
        || d > 31
        || (m == 0 && d != 0)
        || (m != 0 && d > days(y, m))
        || (raw.precision >= 10 && m == 0)
        || (raw.precision == 11 && d == 0)
    {
        return Err(InvalidCalendarFields);
    }
    let (lm, ld, hm, hd) = match raw.precision {
        9 => (1, 1, 12, 31),
        10 => (m, 1, m, days(y, m)),
        _ => (m, d, m, d),
    };
    Ok(BoundRange {
        earliest: CalendarDate(format!("{y:04}-{lm:02}-{ld:02}")),
        latest: CalendarDate(format!("{y:04}-{hm:02}-{hd:02}")),
        raw,
    })
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Record {
    subject: String,
    property: String,
    object: String,
    source_url: String,
    raw_file: String,
    raw_sha256: String,
    statement: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Asset {
    version: u32,
    index_sha256: String,
    records: Vec<Record>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TemporalWitness {
    pub edge: EdgeWitness,
    pub claim_id: String,
    pub rank: String,
    pub start: BoundRange,
    pub end: BoundRange,
    pub source_url: String,
    pub raw_file: String,
    pub raw_sha256: String,
    pub index_sha256: String,
    pub policy: &'static str,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkippedClaim {
    pub claim_id: String,
    pub reason: ClaimSkipReason,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalUnknownReason {
    Static(UnknownReason),
    NoRecordedBoundedInterval,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalRefusalReason {
    InvalidDate,
    Question(RefusalReason),
    UnsupportedProperty,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TemporalVerification {
    Supported {
        date: CalendarDate,
        witness: Box<TemporalWitness>,
        scope: &'static str,
    },
    Unknown {
        reason: TemporalUnknownReason,
        skipped: Vec<SkippedClaim>,
    },
    Refused {
        reason: TemporalRefusalReason,
    },
}
/// Owns the exact hash-bound store; callers cannot swap a different store in later.
pub struct TemporalIndex {
    store: Store,
    hash: String,
    records: Vec<Record>,
}
fn hex(s: &str, n: usize) -> bool {
    s.len() == n
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn claim_id(s: &str, subject: &str) -> bool {
    let Some((prefix, uuid)) = s.split_once('$') else {
        return false;
    };
    prefix.eq_ignore_ascii_case(subject)
        && uuid.len() == 36
        && uuid.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
impl TemporalIndex {
    pub fn from_bytes(index: &[u8], temporal: &[u8]) -> Result<Self, String> {
        if index.len() > MAX_TEMPORAL_INDEX_BYTES || temporal.len() > MAX_TEMPORAL_BYTES {
            return Err("input byte cap exceeded".into());
        }
        let asset: Asset =
            serde_json::from_value(strict_json(temporal)?).map_err(|e| e.to_string())?;
        let hash = format!("{:x}", Sha256::digest(index));
        if asset.version != 1 || asset.index_sha256 != hash {
            return Err("temporal schema version or index SHA256 mismatch".into());
        }
        if asset.records.len() > MAX_TEMPORAL_RECORDS {
            return Err("record cap exceeded".into());
        }
        let store: Store =
            serde_json::from_value(strict_json(index)?).map_err(|e| e.to_string())?;
        store.validate()?;
        let mut unique = BTreeMap::new();
        for r in asset.records {
            let s = &r.statement;
            let main = field(s, "/mainsnak");
            let id = field(s, "/id").as_str().ok_or("missing claim ID")?;
            if !crate::verification::valid_id(&r.subject)
                || !crate::verification::valid_id(&r.object)
                || !matches!(r.property.as_str(), "P108" | "P27")
                || !claim_id(id, &r.subject)
                || field(s, "/type") != "statement"
                || !matches!(
                    field(s, "/rank").as_str(),
                    Some("normal" | "preferred" | "deprecated")
                )
                || serde_json::to_vec(s).map_err(|e| e.to_string())?.len() > 65536
            {
                return Err("invalid statement identity/schema/size".into());
            }
            if field(main, "/snaktype") != "value"
                || field(main, "/property").as_str() != Some(r.property.as_str())
                || field(main, "/datatype") != "wikibase-item"
                || field(main, "/datavalue/type") != "wikibase-entityid"
                || field(main, "/datavalue/value/id").as_str() != Some(r.object.as_str())
                || field(main, "/datavalue/value/entity-type") != "item"
            {
                return Err("inconsistent statement triple".into());
            }
            if let Some(n) = field(main, "/datavalue/value").get("numeric-id")
                && n.as_u64().map(|v| format!("Q{v}")) != Some(r.object.clone())
            {
                return Err("inconsistent numeric ID".into());
            }
            if !store
                .entities
                .get(&crate::EntityId(r.subject.clone()))
                .and_then(|e| e.claims.get(&crate::PropertyId(r.property.clone())))
                .is_some_and(|v| v.contains(&crate::EntityId(r.object.clone())))
            {
                return Err("statement edge absent from bound index".into());
            }
            if r.source_url != format!("https://www.wikidata.org/wiki/{}#{}", r.subject, r.property)
                || !hex(&r.raw_sha256, 64)
                || !r.raw_file.strip_suffix(".json").is_some_and(|v| hex(v, 16))
            {
                return Err("invalid provenance".into());
            }
            if let Some(old) = unique.get(&id.to_ascii_lowercase()) {
                if old != &r {
                    return Err("conflicting duplicate claim ID".into());
                }
            } else {
                unique.insert(id.to_ascii_lowercase(), r);
            }
        }
        Ok(Self {
            store,
            hash,
            records: unique.into_values().collect(),
        })
    }
    pub fn verify_at_str(&self, date: &str, question: &str) -> TemporalVerification {
        match CalendarDate::parse(date) {
            Ok(d) => self.verify_at(&d, question),
            Err(_) => TemporalVerification::Refused {
                reason: TemporalRefusalReason::InvalidDate,
            },
        }
    }
    pub fn verify_at(&self, date: &CalendarDate, question: &str) -> TemporalVerification {
        use TemporalVerification::*;
        match crate::verification::parse(question) {
            Err(reason) => {
                return Refused {
                    reason: TemporalRefusalReason::Question(reason),
                };
            }
            Ok(claims) if claims.iter().any(|c| !matches!(c.property, "P108" | "P27")) => {
                return Refused {
                    reason: TemporalRefusalReason::UnsupportedProperty,
                };
            }
            _ => {}
        }
        let edge = match self.store.verify(question) {
            Verification::Supported { witness, .. } => witness,
            Verification::Unknown { reason } => {
                return Unknown {
                    reason: TemporalUnknownReason::Static(reason),
                    skipped: vec![],
                };
            }
            Verification::Refused { reason } => {
                return Refused {
                    reason: TemporalRefusalReason::Question(reason),
                };
            }
        };
        let mut skipped = Vec::new();
        for r in self.records.iter().filter(|r| {
            r.subject == edge.subject.0
                && r.property == edge.property.0
                && r.object == edge.object.0
        }) {
            let id = field(&r.statement, "/id").as_str().unwrap_or_default();
            let outcome = interval(&r.statement).and_then(|(start, end)| {
                if start.latest < *date && *date < end.earliest {
                    Ok((start, end))
                } else {
                    Err(ClaimSkipReason::OutsideStrictInterior)
                }
            });
            match outcome {
                Ok((start, end)) => {
                    return Supported {
                        date: date.clone(),
                        witness: Box::new(TemporalWitness {
                            edge,
                            claim_id: id.into(),
                            rank: field(&r.statement, "/rank")
                                .as_str()
                                .unwrap_or_default()
                                .into(),
                            start,
                            end,
                            source_url: r.source_url.clone(),
                            raw_file: r.raw_file.clone(),
                            raw_sha256: r.raw_sha256.clone(),
                            index_sha256: self.hash.clone(),
                            policy: TEMPORAL_POLICY,
                        }),
                        scope: "one_recorded_bounded_interval_calendar_day",
                    };
                }
                Err(reason) => skipped.push(SkippedClaim {
                    claim_id: id.into(),
                    reason,
                }),
            }
        }
        Unknown {
            reason: TemporalUnknownReason::NoRecordedBoundedInterval,
            skipped,
        }
    }
}
fn interval(s: &Value) -> Result<(BoundRange, BoundRange), ClaimSkipReason> {
    use ClaimSkipReason::*;
    if field(s, "/rank") == "deprecated" {
        return Err(Deprecated);
    }
    let q = field(s, "/qualifiers").as_object().ok_or(MissingBound)?;
    if q.keys().any(|k| k != "P580" && k != "P582") {
        return Err(ExtraQualifier);
    }
    let one = |p: &str| -> Result<BoundRange, ClaimSkipReason> {
        let a = q.get(p).and_then(Value::as_array).ok_or(MissingBound)?;
        if a.len() != 1 {
            return Err(if a.is_empty() {
                MissingBound
            } else {
                DuplicateBound
            });
        }
        range(a.first().ok_or(MissingBound)?, p)
    };
    let start = one("P580")?;
    let end = one("P582")?;
    if start.earliest > end.latest {
        return Err(InvertedInterval);
    }
    Ok((start, end))
}
// Reject duplicate JSON object keys instead of letting last-key-wins erase qualifiers.
fn strict_json(bytes: &[u8]) -> Result<Value, String> {
    struct Strict(Value);
    impl<'de> Deserialize<'de> for Strict {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = Strict;
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("JSON without duplicate keys")
                }
                fn visit_map<A: serde::de::MapAccess<'de>>(
                    self,
                    mut a: A,
                ) -> Result<Strict, A::Error> {
                    let mut map = serde_json::Map::new();
                    let mut seen = BTreeSet::new();
                    while let Some((k, v)) = a.next_entry::<String, Strict>()? {
                        if !seen.insert(k.clone()) {
                            return Err(serde::de::Error::custom("duplicate JSON key"));
                        }
                        map.insert(k, v.0);
                    }
                    Ok(Strict(Value::Object(map)))
                }
                fn visit_seq<A: serde::de::SeqAccess<'de>>(
                    self,
                    mut a: A,
                ) -> Result<Strict, A::Error> {
                    let mut v = Vec::new();
                    while let Some(s) = a.next_element::<Strict>()? {
                        v.push(s.0)
                    }
                    Ok(Strict(Value::Array(v)))
                }
                fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Strict, E> {
                    Ok(Strict(Value::String(s.into())))
                }
                fn visit_bool<E: serde::de::Error>(self, b: bool) -> Result<Strict, E> {
                    Ok(Strict(Value::Bool(b)))
                }
                fn visit_i64<E: serde::de::Error>(self, n: i64) -> Result<Strict, E> {
                    Ok(Strict(n.into()))
                }
                fn visit_u64<E: serde::de::Error>(self, n: u64) -> Result<Strict, E> {
                    Ok(Strict(n.into()))
                }
                fn visit_f64<E: serde::de::Error>(self, n: f64) -> Result<Strict, E> {
                    Ok(Strict(
                        serde_json::Number::from_f64(n)
                            .map(Value::Number)
                            .ok_or_else(|| E::custom("invalid number"))?,
                    ))
                }
                fn visit_unit<E: serde::de::Error>(self) -> Result<Strict, E> {
                    Ok(Strict(Value::Null))
                }
            }
            d.deserialize_any(Visitor)
        }
    }
    serde_json::from_slice::<Strict>(bytes)
        .map(|s| s.0)
        .map_err(|e| e.to_string())
}
