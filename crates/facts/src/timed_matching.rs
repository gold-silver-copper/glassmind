//! Shared-subject matching within recorded strict interval interiors.
use crate::temporal::{CalendarDate, TemporalIndex, TemporalWitness};
use crate::{
    EdgeWitness, EndpointResolution, EntityId, MatchConstraint, MatchPredicate, MatchRefusalReason,
    MatchResult, MatchUnknownReason, ResolvedMatchConstraint, SET_DISPLAY_LIMIT,
    parse_match_question,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalMatchRefusalReason {
    InvalidDate,
    Question(MatchRefusalReason),
    UnsupportedProperty,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TemporalSubjectMatch {
    pub subject: EntityId,
    pub witnesses: Vec<TemporalWitness>,
}
/// The date applies to every witness. Zero is no complete recorded interval
/// match in this snapshot, never evidence of global absence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TemporalMatchResult {
    Matched {
        date: CalendarDate,
        constraints: Vec<ResolvedMatchConstraint>,
        matches: Vec<TemporalSubjectMatch>,
        total_recorded_interval_matches: usize,
        returned: usize,
        truncated: bool,
        scope: &'static str,
        semantics: &'static str,
    },
    Unknown {
        reason: MatchUnknownReason,
    },
    Refused {
        reason: TemporalMatchRefusalReason,
    },
}
impl TemporalIndex {
    pub fn match_at_str(&self, date: &str, question: &str) -> TemporalMatchResult {
        match CalendarDate::parse(date) {
            Ok(date) => self.match_at(&date, question),
            Err(_) => TemporalMatchResult::Refused {
                reason: TemporalMatchRefusalReason::InvalidDate,
            },
        }
    }
    pub fn match_at(&self, date: &CalendarDate, question: &str) -> TemporalMatchResult {
        match parse_match_question(question) {
            Ok(cs) => {
                if self.store().has_reserved_separator_alias(question, &cs) {
                    return TemporalMatchResult::Refused {
                        reason: TemporalMatchRefusalReason::Question(
                            MatchRefusalReason::UnsupportedGrammar,
                        ),
                    };
                }
                self.match_constraints_at(date, &cs)
            }
            Err(reason) => TemporalMatchResult::Refused {
                reason: TemporalMatchRefusalReason::Question(reason),
            },
        }
    }
    pub fn match_constraints_at(
        &self,
        date: &CalendarDate,
        constraints: &[MatchConstraint],
    ) -> TemporalMatchResult {
        use TemporalMatchResult::*;
        if !(2..=crate::MAX_MATCH_CONDITIONS).contains(&constraints.len()) {
            return Refused {
                reason: TemporalMatchRefusalReason::Question(
                    MatchRefusalReason::InvalidConditionCount,
                ),
            };
        }
        if constraints.iter().any(|c| {
            !matches!(
                c.predicate,
                MatchPredicate::Employer | MatchPredicate::Citizenship
            )
        }) {
            return Refused {
                reason: TemporalMatchRefusalReason::UnsupportedProperty,
            };
        }
        let resolved = match self.store().resolve_match_constraints(constraints) {
            Ok(cs) => cs,
            Err(MatchResult::Unknown { reason }) => return Unknown { reason },
            Err(MatchResult::Refused { reason }) => {
                return Refused {
                    reason: TemporalMatchRefusalReason::Question(reason),
                };
            }
            // Resolution never constructs a matched result; fail closed.
            Err(MatchResult::Matched { .. }) => {
                return Refused {
                    reason: TemporalMatchRefusalReason::Question(
                        MatchRefusalReason::UnsupportedGrammar,
                    ),
                };
            }
        };
        let mut total = 0;
        let mut matches = Vec::new();
        for subject in self.store().matching_subjects(&resolved) {
            let witnesses: Option<Vec<_>> = resolved
                .iter()
                .map(|c| {
                    self.interval_witness(
                        date,
                        &EdgeWitness {
                            subject: subject.clone(),
                            property: c.property.clone(),
                            object: c.object.clone(),
                            subject_resolution: EndpointResolution::ExplicitId,
                            object_resolution: c.object_resolution,
                            source_url: format!(
                                "https://www.wikidata.org/wiki/{}#{}",
                                subject.0, c.property.0
                            ),
                        },
                    )
                    .ok()
                })
                .collect();
            if let Some(witnesses) = witnesses {
                total += 1;
                if matches.len() < SET_DISPLAY_LIMIT {
                    matches.push(TemporalSubjectMatch {
                        subject: subject.clone(),
                        witnesses,
                    });
                }
            }
        }
        let returned = matches.len();
        Matched {
            date: date.clone(),
            constraints: resolved,
            matches,
            total_recorded_interval_matches: total,
            returned,
            truncated: total > returned,
            scope: "local_snapshot_recorded_bounded_intervals",
            semantics: "Each condition has one original recorded claim whose strict interval interior contains the requested day; zero matches is not global absence or current truth.",
        }
    }
}
