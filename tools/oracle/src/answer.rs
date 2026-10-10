//! Answering given requests: the task line `tasks` would write for the same
//! kind and arguments, or why the request is not a valid task.
//!
//! The output keeps the request's `id`. A rename's `args.new`, when given,
//! must be a fresh identifier and fills every change's `new`.

use std::collections::BTreeMap;

use glassmind_answer::{ItemKind, Refusal, Response};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::source::Source;
use crate::tasks::{self, KINDS, Task};

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: String,
    pub source: String,
    pub kind: String,
    #[serde(default)]
    pub args: Map<String, Value>,
}

/// Valid candidates of each kind per source, by the name part of their id.
#[derive(Default)]
pub struct Answerer<'a> {
    sources: BTreeMap<&'a str, &'a Source>,
    cache: BTreeMap<(String, String), BTreeMap<String, Task>>,
    /// The `tasks` seed, which decides `no_such_name` info.
    seed: u64,
}

impl<'a> Answerer<'a> {
    pub fn new(sources: &'a [Source], seed: u64) -> Self {
        Answerer {
            sources: sources.iter().map(|s| (s.name.as_str(), s)).collect(),
            cache: BTreeMap::new(),
            seed,
        }
    }

    /// The output line for one request: a task, serialized exactly as
    /// `tasks` writes it, or `{"id", "error"}`.
    pub fn answer(&mut self, request: &Request) -> String {
        let error = |why: String| error_line(&json!(request.id), &why);
        match self.task(request) {
            Ok(task) => serde_json::to_string(&task).unwrap_or_else(|e| error(e.to_string())),
            Err(why) => error(why),
        }
    }

    fn task(&mut self, request: &Request) -> Result<Task, String> {
        let source = *self
            .sources
            .get(request.source.as_str())
            .ok_or(format!("unknown source {}", request.source))?;
        if !KINDS.contains(&request.kind.as_str()) {
            return Err(format!("unknown task kind {}", request.kind));
        }
        let arg = |name: &str| -> Result<&str, String> {
            request
                .args
                .get(name)
                .and_then(Value::as_str)
                .ok_or(format!("missing string args.{name}"))
        };
        let kind = request.kind.as_str();
        let key = match kind {
            "rename_field" => format!("{}.{}", arg("struct")?, arg("old")?),
            "rename_function" | "ambiguous_rename_field" => arg("old")?.to_string(),
            "structs_with_field_type" => arg("type")?.to_string(),
            "structs_with_field_named" => arg("field")?.to_string(),
            _ => arg("name")?.to_string(),
        };
        if kind == "no_such_name" && source.taken.contains(&key) {
            return Err(format!("{key} is an identifier in the source"));
        }
        let seed = self.seed;
        let found = self
            .cache
            .entry((source.name.clone(), kind.to_string()))
            .or_insert_with(|| {
                tasks::candidates(source, kind, seed)
                    .into_iter()
                    .filter_map(|c| {
                        let suffix = c.task.id.splitn(3, '/').nth(2)?.to_string();
                        Some((suffix, c.task))
                    })
                    .collect()
            })
            .get(&key)
            .cloned();
        let mut task = match found {
            Some(task) => task,
            None if kind == "no_such_name" => no_such_name(source, &key),
            None => return Err(why_invalid(source, kind, &key, request)),
        };
        task.id.clone_from(&request.id);
        if kind == "no_such_name"
            && let Some(args) = task.args.as_object_mut()
        {
            match request.args.get("question") {
                Some(q) => args.insert("question".into(), q.clone()),
                None => args.remove("question"),
            };
        }
        if kind.starts_with("rename_")
            && let Some(new) = request.args.get("new").and_then(Value::as_str)
        {
            check_fresh(source, new)?;
            if let Some(args) = task.args.as_object_mut() {
                args.insert("new".into(), json!(new));
            }
            if let Response::Edit { changes } = &task.expected {
                let changes = changes
                    .iter()
                    .cloned()
                    .map(|mut c| {
                        c.new = new.to_string();
                        c
                    })
                    .collect();
                task.expected = Response::Edit { changes };
            }
        } else if kind == "ambiguous_rename_field"
            && let Some(new) = request.args.get("new").and_then(Value::as_str)
            && let Some(args) = task.args.as_object_mut()
        {
            args.insert("new".into(), json!(new));
        }
        Ok(task)
    }
}

/// `{"id": ..., "error": ...}` with `id` first.
pub fn error_line(id: &Value, why: &str) -> String {
    format!("{{\"id\":{id},\"error\":{}}}", json!(why))
}

fn is_identifier(word: &str) -> bool {
    let mut chars = word.chars();
    chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
        && chars.all(|c| c == '_' || c.is_alphanumeric())
        && word != "_"
}

fn check_fresh(source: &Source, new: &str) -> Result<(), String> {
    if !is_identifier(new) {
        return Err(format!("new name {new} is not an identifier"));
    }
    if tasks::is_keyword(new) {
        return Err(format!("new name {new} is a keyword"));
    }
    if source.taken.contains(new) {
        return Err(format!(
            "new name {new} is already an identifier in the source"
        ));
    }
    Ok(())
}

/// A `no_such_name` task for a fresh name the oracle did not synthesize.
fn no_such_name(source: &Source, name: &str) -> Task {
    Task {
        id: String::new(),
        source: source.name.clone(),
        kind: "no_such_name".into(),
        args: json!({ "name": name }),
        expected: Response::Refuse {
            refusal: Refusal::NoSuchName {
                name: name.to_string(),
            },
        },
        info: json!({}),
    }
}

fn kind_word(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Function => "function",
        ItemKind::Struct => "struct",
        ItemKind::Enum => "enum",
        ItemKind::Trait => "trait",
        ItemKind::Field => "field",
        ItemKind::Problem => "problem",
        ItemKind::Test => "test",
    }
}

/// Why a name does not resolve to one item of an allowed kind.
fn why_not_unique(source: &Source, name: &str, allowed: &[ItemKind]) -> String {
    let named = source.named(name);
    match named.as_slice() {
        [] => format!("no item is named {name}"),
        [one] => match source.item(*one) {
            Some(item) if !allowed.contains(&item.key.kind) => {
                format!("{name} is a {}", kind_word(item.key.kind))
            }
            _ => format!("{name} is not valid for this task"),
        },
        many => format!("{name} is not unique ({} items)", many.len()),
    }
}

fn why_invalid(source: &Source, kind: &str, key: &str, request: &Request) -> String {
    use ItemKind::{Enum, Field, Function, Struct, Trait};
    match kind {
        "callers" | "callees" | "count_callers" | "fields_of_function" => {
            why_not_unique(source, key, &[Function])
        }
        "locate" => why_not_unique(source, key, &[Function, Struct, Enum, Trait, Field]),
        "describe" => why_not_unique(source, key, &[Function, Struct, Enum, Trait]),
        "fields_of" => why_not_unique(source, key, &[Struct]),
        "structs_with_field_type" => format!("no struct has a field whose type mentions {key}"),
        "structs_with_field_named" => format!("no struct has a field named {key}"),
        "ambiguous_rename_field" => format!("{key} is not a field of two or more structs"),
        "rename_function" => {
            if source
                .unique(key)
                .is_some_and(|i| source.item(i).is_some_and(|it| it.key.kind == Function))
            {
                format!(
                    "renaming {key} is not exact: it occurs other than as a declaration, call, path, method or use"
                )
            } else {
                why_not_unique(source, key, &[Function])
            }
        }
        "rename_field" => {
            let old = request
                .args
                .get("old")
                .and_then(Value::as_str)
                .unwrap_or("");
            let strukt = request
                .args
                .get("struct")
                .and_then(Value::as_str)
                .unwrap_or("");
            let fields = tasks::fields_by_name(source)
                .remove(old)
                .unwrap_or_default();
            let owners: Vec<String> = fields
                .iter()
                .filter_map(|f| source.item(*f)?.owner.clone())
                .collect();
            if owners.is_empty() {
                format!("no struct has a field named {old}")
            } else if owners.len() > 1 {
                format!("{old} is a field of {} structs", owners.len())
            } else if !owners.iter().any(|o| o == strukt) {
                format!("{old} is a field of {}, not {strukt}", owners.join(", "))
            } else {
                format!(
                    "renaming {strukt}.{old} is not exact: it occurs other than as a declaration, field access or non-shorthand struct field"
                )
            }
        }
        _ => format!("{key} is not valid for {kind}"),
    }
}
