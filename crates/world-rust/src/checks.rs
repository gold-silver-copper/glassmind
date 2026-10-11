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
    /// `cargo check` of every target but benches (which often need a
    /// nightly compiler): does the code (and its tests) compile?
    Build,
    /// `cargo test`: do the tests pass?
    Test,
    /// `cargo clippy` of the same targets: what does the linter warn about?
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
/// find: before 2, a check could read a build of another copy; before 3,
/// builds took in benches; before 4, a failed linter without readable
/// diagnostics could be cached as a pass; before 5, a failed target could
/// stop cargo before it reported errors in other independent targets.
const RESULTS: &str = "v5";

/// How long one tool run may take.
const TOOL_TIME: Duration = Duration::from_secs(600);

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

/// The targets checked: every one but benches, which often need a nightly
/// compiler (`#![feature(test)]`), so whether cargo gets to them depends on
/// the order it builds in. `--lib` only where there is a library.
fn targets(root: &Path) -> &'static [&'static str] {
    if root.join("src/lib.rs").exists() {
        &["--lib", "--bins", "--tests", "--examples"]
    } else {
        &["--bins", "--tests", "--examples"]
    }
}

/// Where scratch copies go: never the system temp directory, which on this
/// machine is in memory.
fn scratch_base() -> PathBuf {
    std::env::var_os("GLASSMIND_SCRATCH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_default()
                .join(".cache/glassmind/scratch")
        })
}

/// The most memory one check may use: tests of broken code can loop
/// allocating, and must not take the machine with them.
const TOOL_MEMORY: &str = "8G";

/// `cargo`, in a required scope with a memory limit and a
/// time limit, and in a process group of its own, so a timeout ends the
/// tests it started too.
fn cargo_command() -> Command {
    let mut command = Command::new("systemd-run");
    command
        .args(["--user", "--scope", "--quiet", "--collect"])
        .arg(format!("-pMemoryMax={TOOL_MEMORY}"))
        .arg("-pMemorySwapMax=0")
        .arg(format!("-pRuntimeMaxSec={}", TOOL_TIME.as_secs() + 30))
        .args([
            "setsid",
            "--wait",
            "timeout",
            "--kill-after=5s",
            "600",
            "cargo",
        ]);
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    command
}

/// Ends a check's whole process group.
fn kill_group(child: &mut std::process::Child) {
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{}", child.id())])
        .status();
    let _ = child.kill();
    let _ = child.wait();
}

/// How many cargo runs one process makes at once (each `-j 1`): requests
/// answered side by side share the machine.
const CARGO_AT_ONCE: usize = 1;

/// A turn to run cargo, held while it runs.
struct CargoTurn;

impl CargoTurn {
    fn take() -> CargoTurn {
        let (lock, freed) = &CARGO_RUNNING;
        let mut running = lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while *running >= CARGO_AT_ONCE {
            running = freed
                .wait(running)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        *running += 1;
        CargoTurn
    }
}

impl Drop for CargoTurn {
    fn drop(&mut self) {
        let (lock, freed) = &CARGO_RUNNING;
        let mut running = lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *running = running.saturating_sub(1);
        freed.notify_one();
    }
}

static CARGO_RUNNING: (std::sync::Mutex<usize>, std::sync::Condvar) =
    (std::sync::Mutex::new(0), std::sync::Condvar::new());

fn cargo(
    places: &Places,
    source: &str,
    root: &Path,
    subcommand: &str,
    args: &[&str],
) -> Result<(bool, String), String> {
    let _turn = CargoTurn::take();
    let mut command = cargo_command();
    if let Some(target) = &places.target {
        command.env("CARGO_TARGET_DIR", target.join(source));
    }
    // Doc tests and build scripts write temporary files: beside the scratch
    // copies, never in the system temp directory.
    let tmp = scratch_base().join(format!("tmp-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp);
    command.env("TMPDIR", &tmp);
    let mut child = command
        .arg(subcommand)
        .args(["-j", "1"])
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
            Ok(None) if start.elapsed() > TOOL_TIME + Duration::from_secs(10) => {
                kill_group(&mut child);
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

/// Every file of a source tree (not `target` or hidden entries), as
/// paths relative to `root`, in path order.
fn files(root: &Path) -> Vec<PathBuf> {
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
    files
}

/// A stable fingerprint of a source tree's files (FNV-1a over each path
/// and its bytes, in path order).
fn fingerprint(root: &Path) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for b in bytes {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for f in files(root) {
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
                // Report all independent targets even when one fails: in
                // particular, a library and its cfg(test) build can have
                // different errors. Cargo's default stops at the first.
                &[
                    targets(root),
                    &["--keep-going", "--message-format=json", "-q"],
                ]
                .concat(),
            )?;
            let found = findings(source, root, &out, &["error"]);
            // A failed build with no error read from it is not a pass.
            if !ok && found.is_empty() {
                return Err("the build failed in a way I cannot read".into());
            }
            Ok(found)
        }
        RustCheck::Lint => {
            let (ok, out) = cargo(
                places,
                source,
                root,
                "clippy",
                &[
                    targets(root),
                    &["--keep-going", "--message-format=json", "-q"],
                ]
                .concat(),
            )?;
            let found = findings(source, root, &out, &["error", "warning"]);
            if !ok && found.is_empty() {
                return Err("the linter failed in a way I cannot read".into());
            }
            Ok(found)
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

/// Continue compiler suggestions in an already edited scratch tree. Later
/// diagnostics refer to the edited text; compose them back onto the original
/// only after the requested check passes. Stop after three further rounds.
pub fn complete_suggestions(
    check: RustCheck,
    source: &str,
    original: &Path,
    scratch: &Path,
    initial: &[RustChange],
    mut remaining: Vec<Finding>,
) -> Option<Vec<RustChange>> {
    let roots = BTreeMap::from([(source.to_owned(), scratch.to_owned())]);
    let mut touched: BTreeSet<String> = initial.iter().map(|c| c.file.clone()).collect();
    let mut seen = BTreeSet::new();
    for _ in 0..3 {
        if remaining.is_empty() {
            break;
        }
        let mut places = BTreeSet::new();
        let fixes: Vec<RustChange> = remaining
            .iter()
            .flat_map(|f| &f.fixes)
            .filter(|c| places.insert((c.file.clone(), c.line, c.column, c.old.clone())))
            .cloned()
            .collect();
        if fixes.is_empty() || !seen.insert(fingerprint(scratch)) {
            return None;
        }
        touched.extend(fixes.iter().map(|c| c.file.clone()));
        apply(&fixes, &roots).ok()?;
        remaining = run(check, source, scratch).ok()?;
    }
    if !remaining.is_empty() {
        return None;
    }
    let mut changes = Vec::new();
    for file in touched {
        let old: Vec<char> = std::fs::read_to_string(original.join(&file))
            .ok()?
            .chars()
            .collect();
        let new: Vec<char> = std::fs::read_to_string(scratch.join(&file))
            .ok()?
            .chars()
            .collect();
        if old == new {
            continue;
        }
        let prefix = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
        let suffix = old
            .iter()
            .rev()
            .zip(new.iter().rev())
            .take(old.len().min(new.len()).saturating_sub(prefix))
            .take_while(|(a, b)| a == b)
            .count();
        let before = old.get(..prefix)?;
        let line = u32::try_from(before.iter().filter(|&&c| c == '\n').count() + 1).ok()?;
        let column = u32::try_from(before.iter().rev().take_while(|&&c| c != '\n').count()).ok()?;
        changes.push(RustChange {
            source: source.to_owned(),
            file,
            line,
            column,
            old: old
                .get(prefix..old.len().checked_sub(suffix)?)?
                .iter()
                .collect(),
            new: new
                .get(prefix..new.len().checked_sub(suffix)?)?
                .iter()
                .collect(),
        });
    }
    Some(changes)
}

/// Copies a source tree (without `target` and hidden entries) into `to`,
/// for trying changes without touching the original. The copies are dated
/// a day ahead, so cargo builds them anew whatever was built before.
pub fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    let _ = std::fs::remove_dir_all(to);
    overlay(from, to)
}

/// Copies every file under `from` over the tree at `to`, replacing files of
/// the same path and keeping the rest.
pub fn overlay(from: &Path, to: &Path) -> Result<(), String> {
    for f in files(from) {
        let into = to.join(&f);
        if let Some(dir) = into.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::copy(from.join(&f), &into)
            .and_then(|_| std::fs::File::open(&into))
            .and_then(|f| f.set_modified(SystemTime::now() + AHEAD))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Slips that leave code building but computing the wrong thing: an
/// operator or a truth value written for its neighbour, in pairs.
const SLIPS: &str = " + , - , < , <= , > , >= , == , != , && , || ,true,false";

/// How long undone slips are tried before giving up.
const SLIP_TIME: Duration = Duration::from_secs(120);

/// Changes that each undo one possible slip (see [`SLIPS`]) in the code
/// under `src` of the source at `root`, outside comments, attributes and
/// the test module at the end of a file. Files the failing `tests` name
/// most (by module or where they panicked) come first.
pub fn slips(source: &str, root: &Path, tests: &[RustItem]) -> Vec<RustChange> {
    let said = format!("{tests:?}");
    let mut files: Vec<PathBuf> = files(root)
        .into_iter()
        .filter(|f| f.starts_with("src") && f.extension().is_some_and(|e| e == "rs"))
        .collect();
    files.sort_by_key(|f| {
        std::cmp::Reverse(
            f.file_stem()
                .map_or(0, |s| said.matches(&*s.to_string_lossy()).count()),
        )
    });
    let slips: Vec<&str> = SLIPS.split(',').collect();
    let mut changes = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(root.join(&f)).unwrap_or_default();
        let code = text.lines().take_while(|l| !l.contains("#[cfg(test)]"));
        for (n, line) in code.enumerate() {
            if line.trim_start().starts_with(['/', '#']) {
                continue;
            }
            for (i, old) in slips.iter().enumerate() {
                for (at, _) in line.match_indices(old) {
                    changes.push(RustChange {
                        source: source.to_owned(),
                        file: f.to_string_lossy().into_owned(),
                        line: n as u32 + 1,
                        column: line.get(..at).map_or(0, |b| b.chars().count()) as u32,
                        old: (*old).to_owned(),
                        new: slips.get(i ^ 1).copied().unwrap_or_default().to_owned(),
                    });
                }
            }
        }
    }
    changes
}

/// The first slip (see [`slips`]) after whose undoing every test of the
/// source at `root` passes, each tried alone in a fresh scratch copy, for
/// at most [`SLIP_TIME`].
pub fn undo_slip(source: &str, root: &Path, tests: &[RustItem]) -> Option<RustChange> {
    let scratch = scratch(source);
    let roots = BTreeMap::from([(source.to_owned(), scratch.clone())]);
    let start = Instant::now();
    let found = slips(source, root, tests)
        .into_iter()
        .take_while(|_| start.elapsed() < SLIP_TIME)
        .find(|slip| {
            copy_tree(root, &scratch)
                .and_then(|()| apply(std::slice::from_ref(slip), &roots))
                .and_then(|_| run(RustCheck::Test, source, &scratch))
                .is_ok_and(|f| f.is_empty())
        });
    let _ = std::fs::remove_dir_all(&scratch);
    found
}

/// A scratch directory for trying changes to `source`, in the system's
/// temporary directory: a sandboxed run may write there and nowhere else
/// of its own.
pub fn scratch(source: &str) -> PathBuf {
    let base = scratch_base();
    // Of its own: requests answered side by side may try the same source.
    static COPIES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COPIES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    base.join(format!("{source}-{}-{n}", std::process::id()))
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
    fn slips_are_in_code_not_tests_named_files_first() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/test-slips");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("tests")).unwrap();
        std::fs::write(
            dir.join("src/lib.rs"),
            "// a < b\nfn f(a: u8) -> bool { a < 3 || is_true() }\n#[cfg(test)]\nmod t { fn g() -> bool { 1 == 2 } }\n",
        )
        .unwrap();
        std::fs::write(dir.join("src/kebab.rs"), "fn k() -> bool { false }\n").unwrap();
        std::fs::write(dir.join("tests/it.rs"), "fn h() -> bool { 1 == 2 }\n").unwrap();
        let failing = RustItem {
            source: "s".into(),
            file: String::new(),
            line: 0,
            name: "kebab::tests::test1".into(),
            kind: RustKind::Test,
            owner: None,
            detail: None,
        };
        let found = slips("s", &dir, &[failing]);
        let said: Vec<(&str, u32, u32, &str)> = found
            .iter()
            .map(|c| (c.file.as_str(), c.line, c.column, c.new.as_str()))
            .collect();
        assert_eq!(
            said,
            vec![
                ("src/kebab.rs", 1, 17, "true"),
                ("src/lib.rs", 2, 23, " <= "),
                ("src/lib.rs", 2, 27, " && "),
                ("src/lib.rs", 2, 34, "false"),
            ]
        );
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
