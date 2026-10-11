//! Explicit positive claims checked against directed edges in a finite snapshot.
//! No learned proposals, closed-world negation, temporal inference or mutation.
use crate::{EntityId, PropertyId, Store, normalize, tokenize};
use serde::Serialize;

pub const MAX_VERIFY_QUESTION_BYTES: usize = 4096;
pub const MAX_ENDPOINT_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointResolution {
    Alias,
    ExplicitId,
}

/// An exact indexed edge, not a claim of global truth or completeness.
/// The compact index does not retain Wikidata statement UUIDs or qualifiers.
/// Callers must attach their verified index/bundle provenance to this witness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EdgeWitness {
    pub subject: EntityId,
    pub property: PropertyId,
    pub object: EntityId,
    pub subject_resolution: EndpointResolution,
    pub object_resolution: EndpointResolution,
    pub source_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownReason {
    SubjectNotUniqueOrMissing,
    ObjectNotUniqueOrMissing,
    EdgeMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalReason {
    QuestionTooLong,
    UnsupportedShape,
    AmbiguousDelimiter,
    InvalidEndpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Verification {
    Supported {
        witness: EdgeWitness,
        scope: &'static str,
    },
    Unknown {
        reason: UnknownReason,
    },
    Refused {
        reason: RefusalReason,
    },
}

struct Claim<'a> {
    subject: &'a str,
    property: &'static str,
    object: &'a str,
}

pub(super) enum EndpointLink {
    Missing,
    Ambiguous,
    Known(EntityId, EndpointResolution),
}

pub(super) fn valid_id(id: &str) -> bool {
    let Some(digits) = id.strip_prefix('Q') else {
        return false;
    };
    !digits.is_empty()
        && digits.len() <= 20
        && !digits.starts_with('0')
        && digits.bytes().all(|c| c.is_ascii_digit())
}

pub(super) fn endpoint(text: &str) -> Result<(), RefusalReason> {
    if text.is_empty()
        || text.len() > MAX_ENDPOINT_BYTES
        || text.trim() != text
        || !text
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '\'' | '’' | '-' | '.'))
    {
        return Err(RefusalReason::InvalidEndpoint);
    }
    let lower = text.to_lowercase();
    if lower.contains("n't") || lower.contains("n’t") {
        return Err(RefusalReason::InvalidEndpoint);
    }
    if lower.split(|c: char| !c.is_alphanumeric()).any(|word| {
        matches!(
            word,
            "not"
                | "no"
                | "never"
                | "neither"
                | "nor"
                | "and"
                | "or"
                | "but"
                | "if"
                | "unless"
                | "then"
                | "would"
                | "could"
                | "might"
                | "may"
        )
    }) {
        return Err(RefusalReason::InvalidEndpoint);
    }
    if lower
        .strip_prefix('q')
        .is_some_and(|rest| rest.is_empty() || rest.chars().all(|c| c.is_ascii_digit() || c == '-'))
        && !valid_id(text)
    {
        return Err(RefusalReason::InvalidEndpoint);
    }
    Ok(())
}

fn parse(question: &str) -> Result<Vec<Claim<'_>>, RefusalReason> {
    // Bound before trimming, case conversion, tokenization or store traversal.
    if question.len() > MAX_VERIFY_QUESTION_BYTES {
        return Err(RefusalReason::QuestionTooLong);
    }
    if question.chars().any(char::is_control) {
        return Err(RefusalReason::UnsupportedShape);
    }
    let question = question.trim_matches(' ');
    let lower = question.to_ascii_lowercase();
    let prefix = "please verify: ";
    let offset = if lower.starts_with(prefix) {
        prefix.len()
    } else {
        0
    };
    let body = lower
        .get(offset..)
        .and_then(|s| s.strip_suffix('?'))
        .ok_or(RefusalReason::UnsupportedShape)?;
    if body.contains('?') {
        return Err(RefusalReason::UnsupportedShape);
    }
    let mut found = Vec::new();
    let mut invalid_endpoint = false;
    for (start, delimiter, property) in [
        ("was ", " born in ", "P19"),
        ("did ", " die in ", "P20"),
        ("is ", " a citizen of ", "P27"),
        ("was ", " a citizen of ", "P27"),
        ("does ", " work for ", "P108"),
        ("did ", " work for ", "P108"),
    ] {
        let Some(rest) = body.strip_prefix(start) else {
            continue;
        };
        // Enumerate overlapping delimiters at UTF-8 boundaries. Names can
        // contain predicate words; only endpoint bindings can disambiguate.
        for (at, _) in rest.char_indices().filter(|(at, _)| {
            rest.get(*at..)
                .is_some_and(|tail| tail.starts_with(delimiter))
        }) {
            let begin = offset + start.len();
            let subject = question
                .get(begin..begin + at)
                .ok_or(RefusalReason::UnsupportedShape)?;
            let object = question
                .get(begin + at + delimiter.len()..offset + body.len())
                .ok_or(RefusalReason::UnsupportedShape)?;
            if endpoint(subject).is_err() || endpoint(object).is_err() {
                invalid_endpoint = true;
                continue;
            }
            found.push(Claim {
                subject,
                property,
                object,
            });
        }
    }
    if found.is_empty() {
        Err(if invalid_endpoint {
            RefusalReason::InvalidEndpoint
        } else {
            RefusalReason::UnsupportedShape
        })
    } else {
        Ok(found)
    }
}

impl Store {
    // Equal FULL normalized aliases only. Casing may withdraw a unique link,
    // never disambiguate it. Bound aliases before allocating normalized text.
    pub(super) fn verification_endpoint(&self, text: &str) -> EndpointLink {
        if valid_id(text) {
            return EndpointLink::Known(EntityId(text.into()), EndpointResolution::ExplicitId);
        }
        let normal = normalize(text);
        let cased = tokenize(text);
        let mut candidate: Option<&EntityId> = None;
        let mut case_supported = false;
        for (id, entity) in &self.entities {
            if !valid_id(&id.0) {
                continue;
            }
            for alias in &entity.aliases {
                if alias.len() > MAX_ENDPOINT_BYTES {
                    continue;
                }
                let normalized_alias = normalize(alias);
                if normalized_alias.len() < 2 || normalized_alias != normal {
                    continue;
                }
                if candidate.is_some_and(|other| other != id) {
                    return EndpointLink::Ambiguous;
                }
                candidate = Some(id);
                case_supported |= !alias.chars().all(|c| c.is_alphabetic() && c.is_uppercase())
                    || cased == *alias;
            }
        }
        match candidate.filter(|_| case_supported) {
            Some(id) => EndpointLink::Known(id.clone(), EndpointResolution::Alias),
            None => EndpointLink::Missing,
        }
    }

    /// Verify a whole positive question against actual directed snapshot edges.
    /// See `tools/facts/verification/ALIAS-PLAN.md` for the bounded grammar.
    /// Absence is Unknown, never False. Tense does not establish temporal validity.
    /// Object IDs can be edge-only; object aliases must be in this store itself.
    pub fn verify(&self, question: &str) -> Verification {
        let claims = match parse(question) {
            Ok(claims) => claims,
            Err(reason) => return Verification::Refused { reason },
        };
        let mut selected = None;
        let mut any_subject = false;
        for claim in claims {
            let subject = self.verification_endpoint(claim.subject);
            let object = self.verification_endpoint(claim.object);
            any_subject |= !matches!(subject, EndpointLink::Missing);
            if matches!(subject, EndpointLink::Missing) || matches!(object, EndpointLink::Missing) {
                continue;
            }
            // Ambiguous aliases remain viable alternatives. Edge membership
            // must never choose between interpretations of the question.
            if selected.is_some() {
                return Verification::Refused {
                    reason: RefusalReason::AmbiguousDelimiter,
                };
            }
            selected = Some((subject, object, claim.property));
        }
        let Some((subject, object, property)) = selected else {
            return Verification::Unknown {
                reason: if any_subject {
                    UnknownReason::ObjectNotUniqueOrMissing
                } else {
                    UnknownReason::SubjectNotUniqueOrMissing
                },
            };
        };
        let EndpointLink::Known(subject, subject_resolution) = subject else {
            return Verification::Unknown {
                reason: UnknownReason::SubjectNotUniqueOrMissing,
            };
        };
        let Some(entity) = self.entities.get(&subject) else {
            return Verification::Unknown {
                reason: UnknownReason::SubjectNotUniqueOrMissing,
            };
        };
        let EndpointLink::Known(object, object_resolution) = object else {
            return Verification::Unknown {
                reason: UnknownReason::ObjectNotUniqueOrMissing,
            };
        };
        let property = PropertyId(property.into());
        if !entity
            .claims
            .get(&property)
            .is_some_and(|values| values.contains(&object))
        {
            return Verification::Unknown {
                reason: UnknownReason::EdgeMissing,
            };
        }
        let source_url = format!("https://www.wikidata.org/wiki/{}#{}", subject.0, property.0);
        Verification::Supported {
            witness: EdgeWitness {
                subject,
                property,
                object,
                subject_resolution,
                object_resolution,
                source_url,
            },
            scope: "local_snapshot",
        }
    }
}
