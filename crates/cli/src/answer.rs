//! Turning a reply into the machine-readable answer format.

use glassmind_answer::{Change, ItemKey, ItemKind, MetaAct, Outcome, Refusal, Response, Stage};
use glassmind_meaning::{Action, Reply, Thing, Trouble};
use glassmind_mind::{Progress, Turn};
use glassmind_world_rust::checks::RustCheck;
use glassmind_world_rust::{Rust, RustItem, RustKind};

pub fn key(item: &RustItem) -> ItemKey {
    ItemKey {
        source: item.source.clone(),
        file: item.file.clone(),
        line: item.line,
        name: item.name.clone(),
        kind: match item.kind {
            RustKind::Function => ItemKind::Function,
            RustKind::Struct => ItemKind::Struct,
            RustKind::Enum => ItemKind::Enum,
            RustKind::Variant => ItemKind::Variant,
            RustKind::Trait => ItemKind::Trait,
            RustKind::Field => ItemKind::Field,
            RustKind::Problem => ItemKind::Problem,
            RustKind::Test => ItemKind::Test,
        },
    }
}

/// A check's name in the answer format.
pub fn check_name(check: RustCheck) -> String {
    match check {
        RustCheck::Build => "build",
        RustCheck::Test => "test",
        RustCheck::Lint => "lint",
    }
    .to_owned()
}

pub fn response(reply: &Reply<Rust>) -> Response {
    match reply {
        Reply::Extent(e) => Response::Items {
            items: e.members.iter().map(key).collect(),
        },
        Reply::Location(item) => Response::Items {
            items: [key(item)].into(),
        },
        Reply::Account { item, .. } => Response::Describe { item: key(item) },
        Reply::Count { count, .. } => Response::Count { count: *count },
        Reply::Assuming { reply, .. } => response(reply),
        Reply::Proposal {
            action:
                Action::Rename {
                    thing: Thing::Item(item),
                    name: Some(name),
                    ..
                },
            spots,
        } => Response::Edit {
            changes: spots
                .iter()
                .map(|s| Change {
                    source: s.source.clone(),
                    file: s.file.clone(),
                    line: s.line,
                    column: s.column,
                    old: item.name.clone(),
                    new: name.0.clone(),
                })
                .collect(),
        },
        Reply::Forgotten => Response::Meta {
            act: MetaAct::Forget,
        },
        Reply::Noted(_) => Response::Meta {
            act: MetaAct::Noted,
        },
        Reply::Recall(_) => Response::Meta {
            act: MetaAct::ShowContext,
        },
        Reply::Proposal { .. } | Reply::Patch { .. } => Response::Refuse {
            refusal: Refusal::Unsupported,
        },
        Reply::Checked { check, problems }
        | Reply::Applied {
            check: Some(check),
            problems,
            ..
        } => Response::Checked {
            check: check_name(*check),
            passed: problems.is_empty(),
            problems: problems.iter().map(key).collect(),
        },
        Reply::Applied { check: None, .. } => Response::Meta {
            act: MetaAct::Noted,
        },
        // Changes that leave problems are not a fix: offering them is
        // honest, but it does not answer "make it pass".
        Reply::Remedy { left, .. } if !left.is_empty() => Response::Refuse {
            refusal: Refusal::Unsupported,
        },
        Reply::Remedy { check, changes, .. } => Response::Remedy {
            check: check_name(*check),
            changes: changes
                .iter()
                .map(|c| Change {
                    source: c.source.clone(),
                    file: c.file.clone(),
                    line: c.line,
                    column: c.column,
                    old: c.old.clone(),
                    new: c.new.clone(),
                })
                .collect(),
        },
        // Asking back whether it meant a reading is honest, not an answer.
        Reply::Clarify { .. } => Response::Ask {
            candidates: Default::default(),
        },
        Reply::Declined => Response::Meta {
            act: MetaAct::Declined,
        },
        Reply::Reason { turn, .. } => Response::Meta {
            act: MetaAct::ExplainAction {
                turn: u32::try_from(*turn).unwrap_or(u32::MAX),
            },
        },
        Reply::Trouble(t) => match t {
            Trouble::UnknownWord { word, .. } => Response::Refuse {
                refusal: Refusal::UnknownWord { word: word.clone() },
            },
            Trouble::UnknownShape { .. } => Response::Refuse {
                refusal: Refusal::UnknownShape,
            },
            Trouble::NoSuchName { name } => Response::Refuse {
                refusal: Refusal::NoSuchName {
                    name: name.0.clone(),
                },
            },
            Trouble::WrongKind { .. } => Response::Refuse {
                refusal: Refusal::WrongKind,
            },
            Trouble::Ambiguous { candidates } => Response::Ask {
                candidates: candidates.iter().map(key).collect(),
            },
            Trouble::Unsupported | Trouble::ActionFailed { .. } => Response::Refuse {
                refusal: Refusal::Unsupported,
            },
            Trouble::NothingPending => Response::Refuse {
                refusal: Refusal::NothingPending,
            },
        },
    }
}

pub fn outcome(turn_id: &str, turn: &Turn<Rust>, canonical: Option<String>) -> Outcome {
    Outcome {
        turn_id: turn_id.to_owned(),
        response: response(&turn.reply),
        stage: match turn.progress {
            Progress::Words => Stage::Words,
            Progress::Sentence => Stage::Sentence,
            Progress::Reference => Stage::Reference,
            Progress::Action => Stage::Action,
            Progress::Done => Stage::Done,
        },
        understood: u32::try_from(turn.understood).unwrap_or(u32::MAX),
        tokens: u32::try_from(turn.tokens).unwrap_or(u32::MAX),
        rules: turn.rules().iter().map(|r| r.id.to_owned()).collect(),
        text: turn.spoken.clone(),
        canonical,
    }
}

/// What English says for the reading the turn was answered from, before
/// its names were looked up: the canonical form of the request.
pub fn canonical(english: &glassmind_english::English<Rust>, turn: &Turn<Rust>) -> Option<String> {
    let reading = turn.trace.chosen.and_then(|i| turn.trace.readings.get(i))?;
    english
        .speak(
            glassmind_english::Cat::Utterance,
            &glassmind_english::Sem::Utterance(reading.meaning.clone()),
        )
        .map(|s| s.text)
}
