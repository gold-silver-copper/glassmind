//! Checks that the crate dependency graph keeps glassmind's layers apart.
//!
//! - The meaning layer depends on no language, no world and no grammar.
//! - The grammar building blocks depend on no language and no world.
//! - A language crate (`glassmind-<language>`) depends on no world.
//! - A world crate (`glassmind-world-*`) depends on no language and no grammar.
//! - The dialogue layer (`glassmind-mind`) depends on no language and no world.
//! - Every layer may use `glassmind-rules`, which depends on nothing.
//! - The oracle depends on no part of glassmind except the answer format.
//!
//! Dependencies are followed transitively. Run with `cargo run -p glassmind-depcheck`.

use std::collections::{BTreeMap, BTreeSet};
use std::process::{Command, ExitCode};

use serde_json::Value;

/// The layer a workspace crate belongs to, read from its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    /// Rule identity and switches: shared by every layer, depends on nothing.
    Rules,
    /// Small learned models: usable by every layer, depend on nothing.
    Models,
    Meaning,
    Grammar,
    Language,
    World,
    Bridge,
    Mind,
    Answer,
    Oracle,
    Tool,
}

/// Language crates in the workspace. Add a name here when a new language crate is added.
const LANGUAGES: &[&str] = &["english"];

fn layer(name: &str) -> Layer {
    let Some(rest) = name.strip_prefix("glassmind-") else {
        return Layer::Tool;
    };
    match rest {
        "rules" => Layer::Rules,
        "models" => Layer::Models,
        "meaning" => Layer::Meaning,
        "grammar" => Layer::Grammar,
        "mind" => Layer::Mind,
        "answer" => Layer::Answer,
        "oracle" => Layer::Oracle,
        "loop" | "depcheck" => Layer::Tool,
        _ if rest.starts_with("world-") => Layer::World,
        _ if LANGUAGES.contains(&rest) => Layer::Language,
        _ if LANGUAGES.iter().any(|l| rest.starts_with(&format!("{l}-"))) => Layer::Bridge,
        _ => Layer::Tool,
    }
}

/// Whether a crate in layer `from` may depend on a crate in layer `to`.
fn allowed(from: Layer, to: Layer) -> bool {
    use Layer::*;
    if to == Rules {
        return from != Rules;
    }
    if to == Models {
        return !matches!(from, Rules | Models);
    }
    match from {
        Rules | Models => false,
        Meaning => false,
        Grammar => false,
        Answer => false,
        Language => matches!(to, Meaning | Grammar),
        World => matches!(to, Meaning),
        Mind => matches!(to, Meaning),
        Bridge => matches!(to, Meaning | Grammar | Language | World),
        Oracle => matches!(to, Answer),
        Tool => true,
    }
}

/// A string field of a JSON object, if present.
fn field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn main() -> ExitCode {
    let output = match Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1"])
        .output()
    {
        Ok(o) if o.status.success() => o.stdout,
        Ok(o) => {
            eprintln!(
                "cargo metadata failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("cannot run cargo metadata: {e}");
            return ExitCode::FAILURE;
        }
    };
    let meta: Value = match serde_json::from_slice(&output) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cannot read cargo metadata: {e}");
            return ExitCode::FAILURE;
        }
    };
    let members: BTreeSet<String> = meta
        .get("workspace_members")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    let mut names = BTreeMap::new();
    for package in meta
        .get("packages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let (Some(id), Some(name)) = (field(package, "id"), field(package, "name")) {
            names.insert(id.to_owned(), name.to_owned());
        }
    }
    // Direct normal and build dependencies between workspace crates.
    let mut direct: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for node in meta
        .get("resolve")
        .and_then(|r| r.get("nodes"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(id) = field(node, "id") else {
            continue;
        };
        if !members.contains(id) {
            continue;
        }
        let deps = node
            .get("deps")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|dep| {
                dep.get("dep_kinds")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .any(|k| field(k, "kind") != Some("dev"))
            });
        let entry = direct.entry(id.to_owned()).or_default();
        for dep in deps {
            if let Some(pkg) = field(dep, "pkg")
                && members.contains(pkg)
            {
                entry.insert(pkg.to_owned());
            }
        }
    }
    let mut failures = 0;
    for id in direct.keys() {
        let name = names.get(id).cloned().unwrap_or_default();
        let from = layer(&name);
        let mut seen = BTreeSet::new();
        let mut stack: Vec<String> = direct.get(id).into_iter().flatten().cloned().collect();
        while let Some(dep) = stack.pop() {
            if !seen.insert(dep.clone()) {
                continue;
            }
            stack.extend(direct.get(&dep).into_iter().flatten().cloned());
        }
        for dep in seen {
            let dep_name = names.get(&dep).cloned().unwrap_or_default();
            let to = layer(&dep_name);
            if !allowed(from, to) {
                eprintln!("{name} ({from:?}) must not depend on {dep_name} ({to:?})");
                failures += 1;
            }
        }
    }
    if failures == 0 {
        println!("dependency layers ok: {} workspace crates", direct.len());
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
