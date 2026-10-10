//! Turning a reply into the machine-readable answer format.

use glassmind_answer::{ItemKey, ItemKind, Outcome, Refusal, Response, Stage};
use glassmind_meaning::{Reply, Trouble};
use glassmind_mind::{Progress, Turn};
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
            RustKind::Trait => ItemKind::Trait,
            RustKind::Field => ItemKind::Field,
        },
    }
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
        Reply::Trouble(t) => match t {
            Trouble::UnknownWord { word, .. } => Response::Refuse {
                refusal: Refusal::UnknownWord { word: word.clone() },
            },
            Trouble::UnknownShape { .. } | Trouble::TooAmbiguous => Response::Refuse {
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
            Trouble::Unsupported => Response::Refuse {
                refusal: Refusal::Unsupported,
            },
        },
    }
}

pub fn outcome(turn_id: &str, turn: &Turn<Rust>) -> Outcome {
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
    }
}
