//! The scorer: compares glassmind's outcomes with the expected responses.
//!
//! Wrong is worse than "I don't know": a confident wrong answer costs
//! [`WRONG`], far more than any refusal, and a refusal costs less the
//! further the request got.

use std::collections::BTreeMap;

use glassmind_answer::{Outcome, Refusal, Response, Stage};
use serde::{Deserialize, Serialize};

/// The cost of a confident wrong answer.
pub const WRONG: f64 = 10.0;

/// The label of one turn: what glassmind should have done.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    pub turn_id: String,
    pub conversation: String,
    /// `train`, `heldout` or `valid`.
    pub split: String,
    pub generator: String,
    pub source: String,
    /// The task kind, or the conversation move (`reference`, `meta`, ...).
    pub kind: String,
    pub expected: Response,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Right,
    Refused,
    Wrong,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scored {
    pub turn_id: String,
    pub verdict: Verdict,
    pub cost: f64,
}

/// The cost of a refusal that stopped at `stage` after understanding
/// `understood` of `tokens` tokens.
pub fn refusal_cost(stage: Stage, understood: u32, tokens: u32) -> f64 {
    let base = match stage {
        Stage::Words => 1.0,
        Stage::Sentence => 0.8,
        Stage::Reference => 0.6,
        Stage::Action | Stage::Done => 0.5,
    };
    let progress = if tokens == 0 {
        0.0
    } else {
        f64::from(understood) / f64::from(tokens)
    };
    match stage {
        Stage::Words | Stage::Sentence => base - 0.1 * progress,
        _ => base,
    }
}

fn same_refusal(expected: &Refusal, got: &Refusal) -> bool {
    match (expected, got) {
        (Refusal::NoSuchName { name: a }, Refusal::NoSuchName { name: b }) => {
            a.eq_ignore_ascii_case(b)
        }
        (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
    }
}

/// Judges one outcome against its label.
pub fn judge(expected: &Response, outcome: &Outcome) -> Verdict {
    let got = &outcome.response;
    match (expected, got) {
        (Response::Refuse { refusal: e }, Response::Refuse { refusal: g }) => {
            if same_refusal(e, g) {
                Verdict::Right
            } else {
                Verdict::Refused
            }
        }
        // Not answering is honest whatever was expected.
        (_, Response::Refuse { .. }) => Verdict::Refused,
        // Asking back is honest for a question, and right when it was expected
        // with the same candidates.
        (Response::Ask { candidates: e }, Response::Ask { candidates: g }) => {
            if e == g {
                Verdict::Right
            } else {
                Verdict::Refused
            }
        }
        (_, Response::Ask { .. }) => Verdict::Refused,
        (e, g) => {
            if e == g {
                Verdict::Right
            } else {
                Verdict::Wrong
            }
        }
    }
}

pub fn score(label: &Label, outcome: Option<&Outcome>) -> Scored {
    let Some(outcome) = outcome else {
        return Scored {
            turn_id: label.turn_id.clone(),
            verdict: Verdict::Refused,
            cost: 1.0,
        };
    };
    let verdict = judge(&label.expected, outcome);
    let cost = match verdict {
        Verdict::Right => 0.0,
        Verdict::Wrong => WRONG,
        Verdict::Refused => refusal_cost(outcome.stage, outcome.understood, outcome.tokens),
    };
    Scored {
        turn_id: label.turn_id.clone(),
        verdict,
        cost,
    }
}

/// Totals over a set of scored turns.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Totals {
    pub turns: usize,
    pub right: usize,
    pub refused: usize,
    pub wrong: usize,
    pub cost: f64,
}

impl Totals {
    pub fn add(&mut self, s: &Scored) {
        self.turns += 1;
        self.cost += s.cost;
        match s.verdict {
            Verdict::Right => self.right += 1,
            Verdict::Refused => self.refused += 1,
            Verdict::Wrong => self.wrong += 1,
        }
    }

    /// Share of turns answered right.
    pub fn accuracy(&self) -> f64 {
        if self.turns == 0 {
            0.0
        } else {
            self.right as f64 / self.turns as f64
        }
    }
}

/// A full score report: every turn, and totals by split, generator and kind.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub turns: BTreeMap<String, Scored>,
    pub by_split: BTreeMap<String, Totals>,
    pub by_generator: BTreeMap<String, Totals>,
    pub by_kind: BTreeMap<String, Totals>,
    pub total: Totals,
}

pub fn report(labels: &[Label], outcomes: &[Outcome]) -> Report {
    let by_id: BTreeMap<&str, &Outcome> =
        outcomes.iter().map(|o| (o.turn_id.as_str(), o)).collect();
    let mut r = Report::default();
    for label in labels {
        let s = score(label, by_id.get(label.turn_id.as_str()).copied());
        r.by_split.entry(label.split.clone()).or_default().add(&s);
        r.by_generator
            .entry(label.generator.clone())
            .or_default()
            .add(&s);
        r.by_kind.entry(label.kind.clone()).or_default().add(&s);
        r.total.add(&s);
        r.turns.insert(label.turn_id.clone(), s);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use glassmind_answer::{ItemKey, ItemKind};

    fn item(name: &str) -> ItemKey {
        ItemKey {
            source: "s".into(),
            file: "f.rs".into(),
            line: 1,
            name: name.into(),
            kind: ItemKind::Function,
        }
    }

    fn outcome(response: Response, stage: Stage) -> Outcome {
        Outcome {
            turn_id: "t".into(),
            response,
            stage,
            understood: 1,
            tokens: 2,
            rules: Vec::new(),
            text: String::new(),
        }
    }

    #[test]
    fn wrong_costs_more_than_any_refusal() {
        let expected = Response::Items {
            items: [item("a")].into(),
        };
        let wrong = judge(
            &expected,
            &outcome(
                Response::Items {
                    items: [item("b")].into(),
                },
                Stage::Done,
            ),
        );
        let refused = outcome(
            Response::Refuse {
                refusal: Refusal::UnknownShape,
            },
            Stage::Words,
        );
        assert_eq!(wrong, Verdict::Wrong);
        assert_eq!(judge(&expected, &refused), Verdict::Refused);
        assert!(refusal_cost(Stage::Words, 0, 5) < WRONG);
        assert!(refusal_cost(Stage::Reference, 5, 5) < refusal_cost(Stage::Words, 0, 5));
        // Editing on a guess when the user had to be asked is wrong.
        let ask = Response::Ask {
            candidates: [item("a"), item("b")].into(),
        };
        let edit = outcome(
            Response::Edit {
                changes: Default::default(),
            },
            Stage::Done,
        );
        assert_eq!(judge(&ask, &edit), Verdict::Wrong);
    }
}
