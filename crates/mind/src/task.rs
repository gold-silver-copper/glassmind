//! Durable action state shared by language frontends and verified executors.
use glassmind_meaning::{Action, Ontology, Reply, Trouble};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Ready,
    Revising,
    Applying,
    Completed,
    Blocked,
    Cancelled,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStage {
    Proposed,
    Applied,
    Revised,
}

/// Store typed observations, not a claim inferred from the wording of a reply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct TaskEvent<O: Ontology> {
    pub stage: TaskStage,
    pub result: Reply<O>,
}

/// The resolved action retains its target, kind and requested parameters.
/// The host supplies source freshness, persistence and execution boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Task<O: Ontology> {
    pub request: Action<O>,
    pub original_text: String,
    pub status: TaskStatus,
    pub events: Vec<TaskEvent<O>>,
    #[serde(default)]
    pub choices: Option<TaskChoices<O>>,
    #[serde(default)]
    pub revision: Option<TaskRevision<O>>,
}

impl<O: Ontology> Task<O> {
    pub fn proposed(request: Action<O>, original_text: String, result: Reply<O>) -> Self {
        let status = match &result {
            Reply::Proposal { spots, .. } if !spots.is_empty() => TaskStatus::Ready,
            Reply::Remedy { changes, left, .. } if left.is_empty() => {
                if changes.is_empty() {
                    TaskStatus::Completed
                } else {
                    TaskStatus::Ready
                }
            }
            _ => TaskStatus::Blocked,
        };
        let choices = match &result {
            Reply::Trouble(Trouble::Ambiguous { candidates }) if candidates.len() <= 32 => {
                let mut candidates = candidates.clone();
                candidates.sort();
                candidates.dedup();
                (candidates.len() > 1).then_some(TaskChoices {
                    candidates,
                    selected: None,
                })
            }
            _ => None,
        };
        Self {
            revision: None,
            choices,
            request,
            original_text,
            status,
            events: vec![TaskEvent {
                stage: TaskStage::Proposed,
                result,
            }],
        }
    }

    pub fn proposal(&self) -> Option<&Reply<O>> {
        if self.status != TaskStatus::Ready {
            return None;
        }
        self.events
            .last()
            .filter(|e| e.stage == TaskStage::Proposed)
            .map(|e| &e.result)
    }

    /// Only an actual verified application establishes completion.
    pub fn applied(&mut self, result: Reply<O>) {
        self.status = match &result {
            Reply::Applied {
                check: Some(_),
                problems,
                ..
            } if problems.is_empty() => TaskStatus::Completed,
            _ => TaskStatus::Blocked,
        };
        self.events.push(TaskEvent {
            stage: TaskStage::Applied,
            result,
        });
    }
}

/// An ordered request whose next step depends on completion of the current one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct TaskPlan<O: Ontology> {
    pub steps: Vec<PlanStep<O>>,
    pub current: usize,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct PlanStep<O: Ontology> {
    pub text: String,
    /// Unresolved interpretations, checked before any step is allowed to edit.
    pub interpretations: Vec<glassmind_meaning::Utterance<O>>,
    pub task: Option<Task<O>>,
    pub result: Option<Reply<O>>,
}

/// Persisted alternatives preserve the unresolved choice and the selected item.
/// Selection itself is not permission to bypass the host's proposal or verifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct TaskChoices<O: Ontology> {
    pub candidates: Vec<O::Item>,
    pub selected: Option<usize>,
}

/// Bounded repair evidence. Intermediate changes belong to an isolated trial;
/// only the host can turn a fully checked trial into an applicable proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct TaskRevision<O: Ontology> {
    pub initial: Vec<O::Item>,
    pub attempts: Vec<RevisionAttempt<O>>,
    pub stopped: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct RevisionAttempt<O: Ontology> {
    pub changes: Vec<O::Change>,
    pub before: Vec<O::Item>,
    pub after: Vec<O::Item>,
    pub source_after: String,
}
