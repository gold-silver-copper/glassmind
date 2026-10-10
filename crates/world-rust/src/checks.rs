//! Checking a Rust source with its own tools: building it, running its
//! tests and its linter, reading what they report, offering the fixes the
//! compiler suggests (only those that make the build pass in a scratch
//! copy), and applying changes the user accepted.
//!
//! What the tools report becomes items: a compiler error or warning is an
//! item of kind `Problem` (its code as the name, its message as the
//! detail), a failing test one of kind `Test`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{RustItem, RustKind};

/// The ways the Rust world checks a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RustCheck {
    /// `cargo check --all-targets`: does the code (and its tests) compile?
    Build,
    /// `cargo test`: do the tests pass?
    Test,
    /// `cargo clippy --all-targets`: what does the linter warn about?
    Lint,
}

/// One change to a file: replace `old`, written at `line` (1-based) and
/// `column` (0-based, in characters), with `new`. `old` may span lines.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RustChange {
    pub source: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub old: String,
    pub new: String,
}

/// A problem a tool reported, with the changes the compiler suggests for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub item: RustItem,
    pub fixes: Vec<RustChange>,
}

/// How far ahead copies of a source are dated.
const AHEAD: Duration = Duration::from_secs(24 * 60 * 60);

/// The version of the kept results, raised when checks change what they
/// find: before 2, a check could read a build of another copy.
const RESULTS: &str = "v2";

/// How long one tool run may take.
const TOOL_TIME: Duration = Duration::from_secs(15 * 60);

/// Runs `cargo SUBCOMMAND -j 4 ARGS...` in `root`, returning whether it
/// succeeded and its stdout.
/// Where checks keep their builds and their results.
#[derive(Debug, Clone, Default)]
pub struct Places {
    /// Builds go here, one directory per source (shared by its scratch
    /// copies), instead of each source's own `target`.
    pub target: Option<PathBuf>,
    /// Results are kept here by check, source and a fingerprint of its
    /// files, so the same code is only ever checked once.
    pub cache: Option<PathBuf>,
}

impl Places {
    /// From GLASSMIND_CARGO_TARGET and GLASSMIND_CHECK_CACHE, if set.
    pub fn from_env() -> Places {
        Places {
            target: std::env::var_os("GLASSMIND_CARGO_TARGET").map(PathBuf::from),
            cache: std::env::var_os("GLASSMIND_CHECK_CACHE").map(PathBuf::from),
        }
    }
}

fn cargo(
    places: &Places,
    source: &str,
    root: &Path,
    subcommand: &str,
    args: &[&str],
) -> Result<(bool, String), String> {
    let mut command = Command::new("cargo");
    if let Some(target) = &places.target {
        command.env("CARGO_TARGET_DIR", target.join(source));
    }
    let mut child = command
        .arg(subcommand)
        .args(["-j", "4"])
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run cargo: {e}"))?;
    let mut out = child.stdout.take();
    let reader = std::thread::spawn(move || {
        let mut s = String::new();
        if let Some(o) = out.as_mut() {
            let _ = std::io::Read::read_to_string(o, &mut s);
        }
        s
    });
    let mut err = child.stderr.take();
    let err_reader = std::thread::spawn(move || {
        let mut s = String::new();
        if let Some(e) = err.as_mut() {
            let _ = std::io::Read::read_to_string(e, &mut s);
        }
        s
    });
    let start = Instant::now();
    let success = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None) if start.elapsed() > TOOL_TIME => {
                let _ = child.kill();
                return Err("cargo took too long".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => return Err(e.to_string()),
        }
    };
    let _ = err_reader.join();
    let out = reader
        .join()
        .map_err(|_| "cannot read cargo's output".to_owned())?;
    Ok((success, out))
}

fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn number(v: &Value, key: &str) -> u32 {
    v.get(key)
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .unwrap_or(0)
}

/// The text a span covers, from the source file.
fn covered(root: &Path, span: &Value) -> Option<String> {
    let file = std::fs::read_to_string(root.join(text(span, "file_name"))).ok()?;
    let start = usize::try_from(number(span, "byte_start")).ok()?;
    let end = usize::try_from(number(span, "byte_end")).ok()?;
    file.get(start..end).map(str::to_owned)
}

/// Compiler messages of `levels` from `cargo ... --message-format=json`
/// output, once each, with their suggested fixes.
fn findings(source: &str, root: &Path, out: &str, levels: &[&str]) -> Vec<Finding> {
    let mut seen = BTreeSet::new();
    let mut found = Vec::new();
    for line in out.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if v.get("reason").and_then(Value::as_str) != Some("compiler-message") {
            continue;
        }
        let Some(msg) = v.get("message") else {
            continue;
        };
        if !levels.contains(&text(msg, "level").as_str()) {
            continue;
        }
        let spans = msg
            .get("spans")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let Some(primary) = spans
            .iter()
            .find(|s| s.get("is_primary").and_then(Value::as_bool) == Some(true))
        else {
            continue;
        };
        let code = msg
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(Value::as_str)
            .unwrap_or("error")
            .to_owned();
        let item = RustItem {
            source: source.to_owned(),
            file: text(primary, "file_name"),
            line: number(primary, "line_start"),
            name: code,
            kind: RustKind::Problem,
            owner: None,
            detail: Some(text(msg, "message")),
        };
        if !seen.insert(item.clone()) {
            continue;
        }
        // The first suggestion that can be applied as it stands.
        let mut fixes = Vec::new();
        for child in msg
            .get("children")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let suggested: Vec<&Value> = child
                .get("spans")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|s| {
                    s.get("suggested_replacement").is_some_and(|r| !r.is_null())
                        && matches!(
                            s.get("suggestion_applicability").and_then(Value::as_str),
                            Some("MachineApplicable" | "MaybeIncorrect")
                        )
                })
                .collect();
            if suggested.is_empty() {
                continue;
            }
            for s in suggested {
                if let Some(old) = covered(root, s) {
                    fixes.push(RustChange {
                        source: source.to_owned(),
                        file: text(s, "file_name"),
                        line: number(s, "line_start"),
                        column: number(s, "column_start").saturating_sub(1),
                        old,
                        new: text(s, "suggested_replacement"),
                    });
                }
            }
            break;
        }
        found.push(Finding { item, fixes });
    }
    found
}

/// Failing tests from `cargo test` output (`test name ... FAILED`), each
/// with what it printed when it failed (the panic and assertion lines).
fn failing_tests(source: &str, out: &str) -> Vec<RustItem> {
    let mut why: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut current: Option<&str> = None;
    for line in out.lines() {
        if let Some(name) = line
            .strip_prefix("---- ")
            .and_then(|l| l.strip_suffix(" stdout ----"))
        {
            current = Some(name);
        } else if line.starts_with("failures:") || line.starts_with("note: run with") {
            current = None;
        } else if let Some(name) = current
            && !line.trim().is_empty()
        {
            why.entry(name).or_default().push(line.trim());
        }
    }
    let mut tests: Vec<RustItem> = out
        .lines()
        .filter_map(|l| l.strip_prefix("test ")?.strip_suffix(" ... FAILED"))
        .map(|name| RustItem {
            source: source.to_owned(),
            file: String::new(),
            line: 0,
            name: name.to_owned(),
            kind: RustKind::Test,
            owner: None,
            detail: why.get(name).map(|lines| lines.join(" ")),
        })
        .collect();
    tests.sort();
    tests.dedup();
    tests
}

/// A stable fingerprint of a source tree's files (FNV-1a over each path
/// and its bytes, in path order), skipping `target` and hidden entries.
fn fingerprint(root: &Path) -> u64 {
    let mut files = Vec::new();
    let mut stack = vec![PathBuf::new()];
    while let Some(rel) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(root.join(&rel)) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "target" {
                continue;
            }
            let child = rel.join(&name);
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(child);
            } else {
                files.push(child);
            }
        }
    }
    files.sort();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for b in bytes {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for f in files {
        eat(f.to_string_lossy().as_bytes());
        eat(&std::fs::read(root.join(&f)).unwrap_or_default());
    }
    h
}

/// Runs a check of the source at `root`, returning what it found. With
/// GLASSMIND_CHECK_CACHE set, results are kept there by the check, the
/// source's name and a fingerprint of its files, so the same code is only
/// ever checked once.
pub fn run(check: RustCheck, source: &str, root: &Path) -> Result<Vec<Finding>, String> {
    run_in(&Places::from_env(), check, source, root)
}

/// [`run`], with builds and results kept where `places` says.
pub fn run_in(
    places: &Places,
    check: RustCheck,
    source: &str,
    root: &Path,
) -> Result<Vec<Finding>, String> {
    let cache = places.cache.as_ref().map(|dir| {
        dir.join(format!(
            "{check:?}-{source}-{:016x}-{RESULTS}.json",
            fingerprint(root)
        ))
    });
    if let Some(path) = &cache
        && let Ok(text) = std::fs::read_to_string(path)
        && let Ok(found) = serde_json::from_str(&text)
    {
        return Ok(found);
    }
    // When every copy of a source builds in one directory, cargo may take
    // a build newer than the files for a build of them: the tools run on a
    // fresh copy, newer than any build, so another copy's build (one with
    // the tests fixed, say) is never taken for this one's.
    let found = match &places.target {
        Some(target) => {
            let copy = target
                .join("copies")
                .join(format!("{source}-{}", std::process::id()));
            let found =
                copy_tree(root, &copy).and_then(|()| run_tools(places, check, source, &copy));
            let _ = std::fs::remove_dir_all(&copy);
            found?
        }
        None => run_tools(places, check, source, root)?,
    };
    if let Some(path) = &cache {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string(&found) {
            let _ = std::fs::write(path, text);
        }
    }
    Ok(found)
}

fn run_tools(
    places: &Places,
    check: RustCheck,
    source: &str,
    root: &Path,
) -> Result<Vec<Finding>, String> {
    match check {
        RustCheck::Build => {
            let (ok, out) = cargo(
                places,
                source,
                root,
                "check",
                &["--all-targets", "--message-format=json", "-q"],
            )?;
            let found = findings(source, root, &out, &["error"]);
            // A failed build with no error read from it is not a pass.
            if !ok && found.is_empty() {
                return Err("the build failed in a way I cannot read".into());
            }
            Ok(found)
        }
        RustCheck::Lint => {
            let (_, out) = cargo(
                places,
                source,
                root,
                "clippy",
                &["--all-targets", "--message-format=json", "-q"],
            )?;
            Ok(findings(source, root, &out, &["error", "warning"]))
        }
        RustCheck::Test => {
            let build = run_tools(places, RustCheck::Build, source, root)?;
            if !build.is_empty() {
                return Ok(build);
            }
            let (ok, out) = cargo(
                places,
                source,
                root,
                "test",
                &["--no-fail-fast", "--", "--test-threads", "4"],
            )?;
            let failing = failing_tests(source, &out);
            // Failed tests with none read from the output is not a pass.
            if !ok && failing.is_empty() {
                return Err("the tests failed in a way I cannot read".into());
            }
            Ok(failing
                .into_iter()
                .map(|item| Finding {
                    item,
                    fixes: Vec::new(),
                })
                .collect())
        }
    }
}

/// Makes every change, or none: each must still find its `old` text where
/// it says. `roots` maps a source name to its directory.
pub fn apply(changes: &[RustChange], roots: &BTreeMap<String, PathBuf>) -> Result<usize, String> {
    let mut by_file: BTreeMap<PathBuf, Vec<&RustChange>> = BTreeMap::new();
    for c in changes {
        let root = roots
            .get(&c.source)
            .ok_or_else(|| format!("unknown source {}", c.source))?;
        by_file.entry(root.join(&c.file)).or_default().push(c);
    }
    let mut rewritten = Vec::new();
    for (path, mut file_changes) in by_file {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let mut chars: Vec<char> = text.chars().collect();
        // Where each line starts, in characters.
        let mut starts = vec![0];
        for (i, c) in chars.iter().enumerate() {
            if *c == '\n' {
                starts.push(i + 1);
            }
        }
        // Later changes first, so earlier offsets stay valid.
        file_changes.sort_by_key(|c| std::cmp::Reverse((c.line, c.column)));
        for c in file_changes {
            let line_start = usize::try_from(c.line)
                .ok()
                .and_then(|l| l.checked_sub(1))
                .and_then(|l| starts.get(l))
                .copied()
                .ok_or_else(|| format!("{}:{} no longer exists", path.display(), c.line))?;
            let start = line_start + usize::try_from(c.column).unwrap_or(usize::MAX);
            let old: Vec<char> = c.old.chars().collect();
            let end = start.saturating_add(old.len());
            if chars.get(start..end) != Some(old.as_slice()) {
                return Err(format!(
                    "{}:{}:{} no longer holds `{}`; nothing was changed",
                    path.display(),
                    c.line,
                    c.column + 1,
                    c.old
                ));
            }
            chars.splice(start..end, c.new.chars());
        }
        rewritten.push((path, chars.into_iter().collect::<String>()));
    }
    for (path, text) in &rewritten {
        std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    Ok(changes.len())
}

/// Copies a source tree (without `target` and hidden directories) into
/// `to`, for trying changes without touching the original. The copies are
/// dated a day ahead, so cargo builds them anew whatever was built before.
pub fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    let _ = std::fs::remove_dir_all(to);
    let mut stack = vec![PathBuf::new()];
    while let Some(rel) = stack.pop() {
        let dir = from.join(&rel);
        std::fs::create_dir_all(to.join(&rel)).map_err(|e| e.to_string())?;
        for entry in std::fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .flatten()
        {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') || name_str == "target" {
                continue;
            }
            let rel_child = rel.join(&name);
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(rel_child);
            } else {
                std::fs::copy(from.join(&rel_child), to.join(&rel_child))
                    .and_then(|_| std::fs::File::open(to.join(&rel_child)))
                    .and_then(|f| f.set_modified(SystemTime::now() + AHEAD))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

/// A scratch directory for trying changes to `source`.
pub fn scratch(source: &str) -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".cache/glassmind/scratch")
        .join(format!("{source}-{}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_failing_tests() {
        let out = "running 2 tests\ntest tests::adds ... FAILED\ntest tests::subs ... ok\n";
        let tests = failing_tests("s", out);
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "tests::adds");
    }

    #[test]
    fn applies_changes_across_lines_or_none() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/test-checks");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("a.rs"),
            "fn f(v: &Vec<u32>) {\n    v.push(1);\n}\n",
        )
        .unwrap();
        let roots = BTreeMap::from([("s".to_owned(), dir.clone())]);
        let change = |line, column, old: &str, new: &str| RustChange {
            source: "s".into(),
            file: "a.rs".into(),
            line,
            column,
            old: old.into(),
            new: new.into(),
        };
        assert!(apply(&[change(1, 8, "&mut ", "&")], &roots).is_err());
        assert_eq!(
            apply(
                &[
                    change(1, 8, "&", "&mut "),
                    change(2, 6, "push(1);\n}", "push(2);\n}")
                ],
                &roots
            ),
            Ok(2)
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("a.rs")).unwrap(),
            "fn f(v: &mut Vec<u32>) {\n    v.push(2);\n}\n"
        );
    }
}
