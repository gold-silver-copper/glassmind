//! Assembling glassmind: English, the Rust world and the dialogue layer.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use glassmind_english::English;
use glassmind_meaning::Ontology;
use glassmind_mind::{Mind, Session, SourceRef, Turn};
use glassmind_rules::{RuleInfo, Switches};
use glassmind_world_rust::index::{SourceIndex, index_source};
use glassmind_world_rust::{Rust, RustWorld};

/// Every rule in the system, for listing and switching off.
pub fn all_rules() -> Vec<RuleInfo> {
    let mut rules: Vec<RuleInfo> = glassmind_english_rust::rules()
        .iter()
        .map(|r| r.info)
        .collect();
    rules.extend(glassmind_world_rust::RULES.iter().copied());
    rules.extend(glassmind_mind::RULES.iter().copied());
    rules
}

/// Every rule with a short description of what it reads (grammar rules)
/// or which layer it belongs to.
pub fn all_rules_detailed() -> Vec<(RuleInfo, String)> {
    let mut out: Vec<(RuleInfo, String)> = glassmind_english_rust::rules()
        .iter()
        .map(|r| {
            let rhs: Vec<String> = r
                .rhs
                .iter()
                .map(|s| match s {
                    glassmind_grammar::Sym::Cat(c) => format!("{c:?}"),
                    glassmind_grammar::Sym::Word(w) => format!("{w:?}"),
                    glassmind_grammar::Sym::Class(c) => format!("<{c}>"),
                })
                .collect();
            let mode = if r.speak { "" } else { " (read only)" };
            (r.info, format!("{:?} -> {}{mode}", r.lhs, rhs.join(" ")))
        })
        .collect();
    out.extend(
        glassmind_world_rust::RULES
            .iter()
            .map(|r| (*r, "world rule".to_owned())),
    );
    out.extend(
        glassmind_mind::RULES
            .iter()
            .map(|r| (*r, "dialogue rule".to_owned())),
    );
    out
}

/// Problems that make the rule set unusable, such as duplicate IDs.
pub fn rule_problems() -> Vec<String> {
    let mut problems = glassmind_grammar::Grammar::problems(&glassmind_english_rust::rules());
    let all = all_rules();
    for id in glassmind_rules::duplicate_ids(all.iter()) {
        let message = format!("duplicate rule id {id}");
        if !problems.contains(&message) {
            problems.push(message);
        }
    }
    problems
}

/// A source given on the command line as `name=path`, or just `path`.
pub fn parse_source(arg: &str) -> (String, PathBuf) {
    match arg.split_once('=') {
        Some((name, path)) => (name.to_owned(), PathBuf::from(path)),
        None => {
            let path = PathBuf::from(arg);
            let name = path
                .canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "source".to_owned());
            (name, path)
        }
    }
}

/// The parts of a running glassmind.
pub struct System {
    pub english: English<Rust>,
    pub world: RustWorld,
    pub switches: Switches,
}

impl System {
    pub fn new(indices: Vec<Arc<SourceIndex>>, switches: Switches) -> Self {
        System {
            english: English::new(glassmind_english_rust::rules(), &switches),
            world: RustWorld::from_indices(indices, &switches),
            switches,
        }
    }

    pub fn open(sources: &[(String, PathBuf)], switches: Switches) -> Self {
        let indices = sources
            .iter()
            .map(|(name, root)| Arc::new(index_source(name, root)))
            .collect();
        System::new(indices, switches)
    }

    pub fn mind(&self) -> Mind<'_, Rust> {
        Mind {
            language: &self.english,
            world: &self.world,
            switches: &self.switches,
        }
    }

    pub fn respond(&self, session: &mut Session<Rust>, text: &str) -> Turn<Rust> {
        self.mind().respond(session, text)
    }

    pub fn source_refs(&self) -> Vec<SourceRef> {
        self.world
            .sources
            .iter()
            .map(|s| SourceRef {
                name: s.name.clone(),
                root: s.root.to_string_lossy().into_owned(),
            })
            .collect()
    }
}

/// Loads a saved session, or starts one if the file does not exist.
pub fn load_session<O: Ontology>(
    path: &Path,
    sources: Vec<SourceRef>,
) -> Result<Session<O>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| format!("cannot read session {}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Session::new(sources)),
        Err(e) => Err(format!("cannot read session {}: {e}", path.display())),
    }
}

pub fn save_session<O: Ontology>(path: &Path, session: &Session<O>) -> Result<(), String> {
    let text = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| format!("cannot write session {}: {e}", path.display()))
}
