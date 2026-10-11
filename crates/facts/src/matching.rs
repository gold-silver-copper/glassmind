//! Positive shared-subject conjunctions over a finite, unqualified snapshot.
//! No absence-as-false inference, temporal validity, learned fallback or mutation.
use crate::verification::{EndpointLink, endpoint, valid_id};
use crate::{
    EndpointResolution, EntityId, MAX_ENDPOINT_BYTES, PropertyId, SET_DISPLAY_LIMIT, Store,
};
use serde::{Deserialize, Serialize};

pub const MAX_MATCH_QUESTION_BYTES: usize = 4096;
pub const MAX_MATCH_CONDITIONS: usize = 3;
/// The grammar reserves every ` and ` separator, including inside aliases.
/// Thus only one complete parse is possible; no edge-driven parse selection.
pub const MAX_MATCH_PARSE_ALTERNATIVES: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchPredicate {
    Birth,
    Death,
    Citizenship,
    Employer,
}
impl MatchPredicate {
    pub fn property(self) -> PropertyId {
        PropertyId(
            match self {
                Self::Birth => "P19",
                Self::Death => "P20",
                Self::Citizenship => "P27",
                Self::Employer => "P108",
            }
            .into(),
        )
    }
}

/// A positive directed constraint on the shared subject. `object` is a full
/// alias or canonical QID; explicit IDs may occur only in stored edge objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchConstraint {
    pub predicate: MatchPredicate,
    pub object: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedMatchConstraint {
    pub property: PropertyId,
    pub object: EntityId,
    pub object_resolution: EndpointResolution,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchWitness {
    pub subject: EntityId,
    pub property: PropertyId,
    pub object: EntityId,
    pub source_url: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubjectMatch {
    pub subject: EntityId,
    pub witnesses: Vec<MatchWitness>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchRefusalReason {
    QuestionTooLong,
    UnsupportedGrammar,
    InvalidConditionCount,
    InvalidEndpoint,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchUnknownReason {
    ObjectNotUniqueOrMissing,
}
/// Counts describe this loaded store only. A zero count is not global absence.
/// Callers must attach verified bundle provenance, as with `Store::verify`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MatchResult {
    Matched {
        constraints: Vec<ResolvedMatchConstraint>,
        matches: Vec<SubjectMatch>,
        total_snapshot_matches: usize,
        returned: usize,
        truncated: bool,
        scope: &'static str,
        semantics: &'static str,
    },
    Unknown {
        reason: MatchUnknownReason,
    },
    Refused {
        reason: MatchRefusalReason,
    },
}

fn match_endpoint(text: &str) -> Result<(), MatchRefusalReason> {
    if text.len() > MAX_ENDPOINT_BYTES || endpoint(text).is_err() {
        return Err(MatchRefusalReason::InvalidEndpoint);
    }
    let lower = text.to_lowercase();
    if lower.strip_prefix('q').is_some_and(|rest| {
        rest.is_empty() || rest.starts_with(|c: char| c.is_ascii_digit() || c == '-')
    }) && !valid_id(text)
    {
        return Err(MatchRefusalReason::InvalidEndpoint);
    }
    if lower.split(|c: char| !c.is_alphanumeric()).any(|word| {
        matches!(
            word,
            "currently"
                | "current"
                | "now"
                | "today"
                | "yesterday"
                | "tomorrow"
                | "before"
                | "after"
                | "since"
                | "until"
                | "during"
                | "when"
                | "while"
                | "more"
                | "less"
                | "than"
                | "latest"
                | "still"
                | "only"
                | "always"
                | "ever"
                | "ignore"
                | "instead"
                | "please"
                | "he"
                | "she"
                | "it"
                | "they"
                | "him"
                | "her"
                | "them"
                | "we"
                | "us"
                | "you"
                | "me"
                | "my"
                | "our"
                | "their"
                | "his"
                | "its"
        )
    }) {
        return Err(MatchRefusalReason::InvalidEndpoint);
    }
    Ok(())
}

/// Parse the complete, bounded grammar frozen in tools/facts/matching/PLAN.md.
/// AND is reserved inside aliases, so predicate words cannot cause truncation.
/// Parsing is independent of entity resolution and of the graph's claims.
pub fn parse_match_question(question: &str) -> Result<Vec<MatchConstraint>, MatchRefusalReason> {
    if question.len() > MAX_MATCH_QUESTION_BYTES {
        return Err(MatchRefusalReason::QuestionTooLong);
    }
    if question.chars().any(char::is_control) {
        return Err(MatchRefusalReason::UnsupportedGrammar);
    }
    let question = question.trim_matches(' ');
    let lower = question.to_ascii_lowercase();
    let body = lower
        .strip_prefix("who ")
        .and_then(|s| s.strip_suffix('?'))
        .ok_or(MatchRefusalReason::UnsupportedGrammar)?;
    if body.contains('?') {
        return Err(MatchRefusalReason::UnsupportedGrammar);
    }
    // Never collect an unbounded split; the whole question is also byte-bounded.
    let parts: Vec<_> = body.split(" and ").take(MAX_MATCH_CONDITIONS + 1).collect();
    if !(2..=MAX_MATCH_CONDITIONS).contains(&parts.len()) {
        return Err(MatchRefusalReason::InvalidConditionCount);
    }
    let mut offset = "who ".len();
    let mut result = Vec::with_capacity(parts.len());
    for part in parts {
        let (prefix, predicate) = [
            ("was born in ", MatchPredicate::Birth),
            ("died in ", MatchPredicate::Death),
            ("is a citizen of ", MatchPredicate::Citizenship),
            ("was a citizen of ", MatchPredicate::Citizenship),
            ("works for ", MatchPredicate::Employer),
            ("worked for ", MatchPredicate::Employer),
        ]
        .into_iter()
        .find(|(prefix, _)| part.starts_with(prefix))
        .ok_or(MatchRefusalReason::UnsupportedGrammar)?;
        let object = question
            .get(offset + prefix.len()..offset + part.len())
            .ok_or(MatchRefusalReason::UnsupportedGrammar)?;
        match_endpoint(object)?;
        result.push(MatchConstraint {
            predicate,
            object: object.into(),
        });
        offset += part.len() + " and ".len();
    }
    Ok(result)
}

impl Store {
    pub fn match_question(&self, question: &str) -> MatchResult {
        match parse_match_question(question) {
            Ok(constraints) => {
                if self.has_reserved_separator_alias(question, &constraints) {
                    return MatchResult::Refused {
                        reason: MatchRefusalReason::UnsupportedGrammar,
                    };
                }
                self.match_constraints(&constraints)
            }
            Err(reason) => MatchResult::Refused { reason },
        }
    }

    // Aliases containing the reserved separator are outside this grammar. Do
    // not silently reinterpret such an alias as several shorter constraints.
    // Only full normalized alias equality is consulted, never claims or counts.
    // With at most three clauses this checks at most three spans, each bounded.
    fn has_reserved_separator_alias(
        &self,
        question: &str,
        constraints: &[MatchConstraint],
    ) -> bool {
        let question = question.trim_matches(' ');
        let lower = question.to_ascii_lowercase();
        let Some(body) = lower.strip_prefix("who ").and_then(|s| s.strip_suffix('?')) else {
            return true;
        };
        let mut offset = "who ".len();
        let mut spans = Vec::with_capacity(MAX_MATCH_CONDITIONS);
        for (part, constraint) in body.split(" and ").zip(constraints) {
            let end = offset + part.len();
            spans.push((end - constraint.object.len(), end));
            offset = end + " and ".len();
        }
        for (i, (start, _)) in spans.iter().enumerate() {
            for (_, end) in spans.iter().skip(i + 1) {
                let Some(text) = question
                    .get(*start..*end)
                    .filter(|s| s.len() <= MAX_ENDPOINT_BYTES)
                else {
                    continue;
                };
                let normalized = crate::normalize(text);
                if self.entities.values().any(|e| {
                    e.aliases.iter().any(|alias| {
                        alias.len() <= MAX_ENDPOINT_BYTES && crate::normalize(alias) == normalized
                    })
                }) {
                    return true;
                }
            }
        }
        false
    }

    /// Resolve every endpoint before consulting any claims, then intersect all
    /// directed conditions over the entire store before display truncation.
    /// Duplicate conditions are idempotent; the raw count must still be 2..3.
    pub fn match_constraints(&self, constraints: &[MatchConstraint]) -> MatchResult {
        if !(2..=MAX_MATCH_CONDITIONS).contains(&constraints.len()) {
            return MatchResult::Refused {
                reason: MatchRefusalReason::InvalidConditionCount,
            };
        }
        for constraint in constraints {
            if let Err(reason) = match_endpoint(&constraint.object) {
                return MatchResult::Refused { reason };
            }
        }
        // Collect all endpoint resolutions first; graph membership cannot choose
        // an entity, even when only one candidate would satisfy the conjunction.
        let links: Vec<_> = constraints
            .iter()
            .map(|c| self.verification_endpoint(&c.object))
            .collect();
        let mut resolved = Vec::with_capacity(constraints.len());
        let mut unknown = false;
        for (constraint, link) in constraints.iter().zip(links) {
            match link {
                EndpointLink::Known(object, object_resolution) => {
                    resolved.push(ResolvedMatchConstraint {
                        property: constraint.predicate.property(),
                        object,
                        object_resolution,
                    })
                }
                EndpointLink::Missing | EndpointLink::Ambiguous => unknown = true,
            }
        }
        if unknown {
            return MatchResult::Unknown {
                reason: MatchUnknownReason::ObjectNotUniqueOrMissing,
            };
        }
        // Prefer explicit-ID provenance deterministically if a duplicate edge
        // was specified using both an alias and its ID.
        resolved.sort_by(|a, b| {
            (&a.property, &a.object)
                .cmp(&(&b.property, &b.object))
                .then_with(|| {
                    (a.object_resolution == EndpointResolution::Alias)
                        .cmp(&(b.object_resolution == EndpointResolution::Alias))
                })
        });
        resolved.dedup_by(|a, b| a.property == b.property && a.object == b.object);
        let mut matches = Vec::new();
        let mut total_snapshot_matches = 0;
        for (subject, entity) in &self.entities {
            if !resolved.iter().all(|c| {
                entity
                    .claims
                    .get(&c.property)
                    .is_some_and(|objects| objects.contains(&c.object))
            }) {
                continue;
            }
            total_snapshot_matches += 1;
            if matches.len() < SET_DISPLAY_LIMIT {
                matches.push(SubjectMatch {
                    subject: subject.clone(),
                    witnesses: resolved
                        .iter()
                        .map(|c| MatchWitness {
                            subject: subject.clone(),
                            property: c.property.clone(),
                            object: c.object.clone(),
                            source_url: format!(
                                "https://www.wikidata.org/wiki/{}#{}",
                                subject.0, c.property.0
                            ),
                        })
                        .collect(),
                });
            }
        }
        let returned = matches.len();
        MatchResult::Matched {
            constraints: resolved,
            matches,
            total_snapshot_matches,
            returned,
            truncated: total_snapshot_matches > returned,
            scope: "local_snapshot",
            semantics: "Positive directed stored edges only; no global completeness or temporal/current-validity guarantee.",
        }
    }
}
