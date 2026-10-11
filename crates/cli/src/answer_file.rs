//! Query-only answer exports. No proposals, remedies or guessed readings.
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use glassmind_meaning::interface::{Language, World};
use glassmind_meaning::{Question, Reply, Thing, Trouble};
use glassmind_world_rust::Rust;

use crate::system::System;

#[derive(Clone, Copy, Debug)]
enum Format {
    Number,
    Type,
    Name,
}

struct Export<'a> {
    question: &'a str,
    format: Format,
    destination: PathBuf,
}

// Full directive grammar; every word must have a role. The question is parsed
// separately by the real English grammar, without normalizers or models.
fn parse(text: &str) -> Result<Export<'_>, String> {
    let (question, directive) = text
        .split_once('?')
        .ok_or("expected a question followed by an output directive")?;
    let directive = directive
        .trim()
        .strip_suffix('.')
        .unwrap_or(directive.trim());
    let mut words: Vec<&str> = directive.split_whitespace().collect();
    if words
        .first()
        .is_some_and(|w| w.eq_ignore_ascii_case("please"))
    {
        words.remove(0);
    }
    let Some(verb) = words.first() else {
        return Err("missing output directive".into());
    };
    if !["write", "put", "save"]
        .iter()
        .any(|v| verb.eq_ignore_ascii_case(v))
    {
        return Err("unsupported output directive".into());
    }
    words.remove(0);
    if words.first().is_some_and(|w| ["just", "only"].contains(w)) {
        words.remove(0);
    }
    if words.first() == Some(&"the") {
        words.remove(0);
    }
    let format = match words.first() {
        Some(&"number") => Format::Number,
        Some(&"name") => Format::Name,
        Some(&"type" | &"type,") => Format::Type,
        _ => {
            return Err(
                "unsupported answer format (number, unique name or declared type required)".into(),
            );
        }
    };
    words.remove(0);
    if matches!(format, Format::Type) {
        for phrase in [
            &["exactly", "as", "written", "in", "the", "source,"][..],
            &[
                "spelled", "exactly", "as", "it", "is", "in", "the", "source,",
            ][..],
            &["exactly", "like", "in", "the", "code"][..],
        ] {
            if words.starts_with(phrase) {
                words.drain(..phrase.len());
                break;
            }
        }
    }
    if !words
        .first()
        .is_some_and(|w| ["to", "into", "in"].contains(w))
    {
        return Err("missing output destination".into());
    }
    words.remove(0);
    if words.first() == Some(&"a") {
        words.remove(0);
    }
    if words.first() == Some(&"file") {
        words.remove(0);
    }
    if words
        .first()
        .is_some_and(|w| ["named", "called"].contains(w))
    {
        words.remove(0);
    }
    let mut path = *words.first().ok_or("missing filename")?;
    for quote in ['`', '\'', '"'] {
        if let Some(rest) = path.strip_prefix(quote) {
            path = rest.strip_suffix(quote).ok_or("unclosed filename quote")?;
            break;
        }
    }
    let destination = PathBuf::from(path);
    words.remove(0);
    if !words.is_empty()
        && !matches!(
            words.as_slice(),
            ["in" | "at", "the", "crate", "root"] | ["in" | "at", "crate", "root"]
        )
    {
        return Err("unconsumed output instruction".into());
    }
    validate_path(&destination)?;
    Ok(Export {
        question: question.trim(),
        format,
        destination,
    })
}

fn validate_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("destination must be a relative path without traversal".into());
    }
    // Reject lexical dot components as well (Path::components normalizes them).
    if path
        .to_string_lossy()
        .split('/')
        .any(|p| p.is_empty() || p == "." || p == "..")
    {
        return Err("destination contains an empty or dot component".into());
    }
    Ok(())
}

fn identity(path: &Path) -> Option<PathBuf> {
    if let Ok(p) = path.canonicalize() {
        return Some(p);
    }
    let parent = path.parent()?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    Some(parent.canonicalize().ok()?.join(path.file_name()?))
}

pub fn transcript_conflicts(repo: &Path, transcript: &Path, text: &str) -> bool {
    parse(text).is_ok_and(|e| {
        identity(&repo.join(e.destination)).is_some_and(|p| identity(transcript) == Some(p))
    })
}

pub fn handle(
    system: &System,
    repo: &Path,
    transcript: Option<&Path>,
    explain: bool,
    text: &str,
) -> Option<Result<String, String>> {
    // Reserve output requests even when malformed, so a rejected compound
    // request cannot fall through to the action/repair route.
    let tokens = system.english.tokens(text);
    if !tokens.iter().enumerate().any(|(i, token)| {
        let quoted = text
            .get(token.start..token.end)
            .is_some_and(|raw| raw.starts_with(['`', '\'', '"']));
        let verb = !quoted
            && ["write", "put", "save"]
                .iter()
                .any(|verb| token.text.eq_ignore_ascii_case(verb));
        if !verb {
            return false;
        }
        let prefix = text.get(..token.start).unwrap_or_default().trim_end();
        let previous = i.checked_sub(1).and_then(|p| tokens.get(p));
        let boundary = prefix.is_empty()
            || prefix.ends_with(['?', '.', '!', ';'])
            || previous.is_some_and(|p| {
                ["and", "then", "please"]
                    .iter()
                    .any(|word| p.text.eq_ignore_ascii_case(word))
            });
        if !boundary {
            return false;
        }
        // A verb in a function name is not an instruction. Outside a question,
        // reserve only value-output instructions, not ordinary "write a function".
        prefix.contains('?')
            || tokens
                .iter()
                .skip(i + 1)
                .map(|t| t.text.to_ascii_lowercase())
                .find(|word| !["just", "only", "the", "a"].contains(&word.as_str()))
                .is_some_and(|word| {
                    [
                        "number", "name", "type", "answer", "count", "value", "result",
                    ]
                    .contains(&word.as_str())
                })
    }) {
        return None;
    }
    Some((|| {
        let export = parse(text)?;
        if explain {
            say!(
                "Answer export: {:?} to {}",
                export.format,
                export.destination.display()
            );
        }
        if transcript.is_some_and(|p| transcript_conflicts(repo, p, text)) {
            return Err("answer and --reply-file must have different destinations".into());
        }
        if system
            .world
            .sources
            .iter()
            .any(|s| !s.skipped.is_empty() || s.files.is_empty())
        {
            return Err("source index is incomplete".into());
        }
        if let Some(question) = glassmind_english_rust::declared_types::question(export.question) {
            if !matches!(export.format, Format::Type) {
                return Err("declared type requires type output".into());
            }
            let question = question?;
            let mut fired = Vec::new();
            let answer = system
                .world
                .declared_type(&question, &mut fired)
                .map_err(|e| format!("cannot establish declared type: {e:?}"))?;
            if explain {
                say!("Metadata question: {question:?}");
                say!("Resolved answer: {answer:?}");
                for rule in fired {
                    say!(
                        "World rule: {} ({})",
                        rule.id,
                        glassmind_rules::provenance(rule.round)
                    );
                }
            }
            let bytes = format!("{}\n", answer.spelling);
            publish(repo, &export.destination, bytes.as_bytes()).map_err(|e| e.to_string())?;
            return Ok(format!(
                "Wrote {} to {}.",
                answer.spelling,
                export.destination.display()
            ));
        }
        let questions =
            glassmind_english_rust::metadata_questions(&system.english, export.question);
        if questions.is_empty() {
            return Err("question is not fully understood".into());
        }
        let mut answers = BTreeSet::new();
        let mut fired = Vec::new();
        for question in questions {
            if explain {
                say!("Metadata question: {question:?}");
            }
            if !matches!(question, Question::Count(_) | Question::Identify(_)) {
                return Err("only metadata counts and unique names can be exported".into());
            }
            let mut unknown = false;
            let question = question.map_things(&mut |thing| match thing {
                Thing::Named(name) => {
                    let items = system.world.lookup(name, &mut fired);
                    match items.as_slice() {
                        [item] => Thing::Item(item.clone()),
                        _ => {
                            unknown = true;
                            thing.clone()
                        }
                    }
                }
                Thing::Previous => {
                    unknown = true;
                    thing.clone()
                }
                _ => thing.clone(),
            });
            if unknown {
                return Err("unknown or ambiguous reference".into());
            }
            // A described reference must also resolve uniquely, not denote a
            // union of same-named owners.
            let utterance = glassmind_meaning::Utterance::Ask(question.clone());
            for thing in utterance.things_within(&|_| false) {
                if let Thing::Fitting(d) = thing {
                    match system
                        .world
                        .answer(&Question::Identify(d.clone()), &mut fired)
                    {
                        Ok(Reply::Extent(e)) if e.members.len() == 1 => {}
                        _ => return Err("unknown or ambiguous described reference".into()),
                    }
                }
            }
            let reply = match system.world.answer(&question, &mut fired) {
                Ok(reply) => reply,
                // Lexical overloads such as field-have/member-have are checked
                // against the actual owner kind by the world.
                Err(Trouble::WrongKind { .. }) => continue,
                Err(_) => return Err("world cannot establish this answer".into()),
            };
            let answer = serialize(export.format, &reply)?;
            if explain {
                say!("Resolved answer: {reply:?}");
            }
            answers.insert(answer);
        }
        if answers.len() != 1 {
            return Err("no unique type-compatible answer".into());
        }
        let answer = answers.into_iter().next().ok_or("no answer")?;
        if explain {
            for rule in fired.into_iter().collect::<BTreeSet<_>>() {
                say!(
                    "World rule: {} ({})",
                    rule.id,
                    glassmind_rules::provenance(rule.round)
                );
            }
        }
        publish(repo, &export.destination, answer.as_bytes()).map_err(|e| e.to_string())?;
        Ok(format!(
            "Wrote {} to {}.",
            answer.trim_end(),
            export.destination.display()
        ))
    })())
}

fn serialize(format: Format, reply: &Reply<Rust>) -> Result<String, String> {
    match (format, reply) {
        (Format::Number, Reply::Count { count, .. }) => Ok(format!("{count}\n")),
        (Format::Name, Reply::Extent(e)) => match e.members.as_slice() {
            [item] => Ok(format!("{}\n", item.name)),
            _ => Err("name output requires exactly one item".into()),
        },
        _ => Err("answer does not match requested format".into()),
    }
}

// Pin every directory by descriptor and refuse symlinks. /proc/self/fd allows
// safe relative operations with std alone on Linux (unsafe code is forbidden).
// hard_link publishes the complete staged file atomically and never replaces.
#[cfg(target_os = "linux")]
fn publish(repo: &Path, destination: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    const O_DIRECTORY: i32 = 0o200000;
    const O_NOFOLLOW: i32 = 0o400000;
    let open_dir = |p: &Path| {
        OpenOptions::new()
            .read(true)
            .custom_flags(O_DIRECTORY | O_NOFOLLOW)
            .open(p)
    };
    let mut dir = open_dir(repo)?;
    if let Some(parent) = destination.parent() {
        for part in parent.components() {
            dir =
                open_dir(&PathBuf::from(format!("/proc/self/fd/{}", dir.as_raw_fd())).join(part))?;
        }
    }
    let pinned = PathBuf::from(format!("/proc/self/fd/{}", dir.as_raw_fd()));
    let name = destination
        .file_name()
        .ok_or_else(|| std::io::Error::other("missing filename"))?;
    let target = pinned.join(name);
    if fs::symlink_metadata(&target).is_ok() {
        return Err(std::io::Error::other(
            "destination already exists; overwrite is forbidden",
        ));
    }
    let stage = pinned.join(format!(
        ".glassmind-answer-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file: File = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&stage)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::hard_link(&stage, &target)
    })();
    let _ = fs::remove_file(stage);
    result
}

#[cfg(not(target_os = "linux"))]
fn publish(_repo: &Path, _destination: &Path, _bytes: &[u8]) -> std::io::Result<()> {
    Err(std::io::Error::other(
        "safe answer publication is only supported on Linux",
    ))
}
