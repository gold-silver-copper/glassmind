//! The interactive mode, with grammar-driven Tab completion.

use glassmind_meaning::interface::{Language, Next, World};
use glassmind_mind::Session;
use std::cell::RefCell;
use std::rc::Rc;

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::{Context, Editor, Helper, Highlighter, Hinter, Validator};

use crate::system::{System, load_session, save_session};
use crate::{Common, open};

/// What may come next at the end of `line`: grammar words, and the world's
/// names wherever the grammar expects a name. Filtered by the partly typed
/// last word, if any.
pub fn candidates(system: &System, line: &str) -> Vec<String> {
    let (done, partial) = split_partial(line);
    let partial_lower = partial.to_lowercase();
    let mut out = Vec::new();
    for next in system.english.next(done) {
        match next {
            Next::Word(w) => {
                if w.starts_with(&partial_lower) {
                    out.push(w);
                }
            }
            Next::Name => {
                for name in system.world.names() {
                    if name.to_lowercase().starts_with(&partial_lower) {
                        out.push(name);
                    }
                }
            }
            Next::Number => {}
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Splits a line into the finished part and the word being typed.
fn split_partial(line: &str) -> (&str, &str) {
    let start = line
        .char_indices()
        .rev()
        .find(|&(_, c)| c.is_whitespace() || c == '`')
        .map_or(0, |(i, c)| i + c.len_utf8());
    line.split_at(start)
}

#[derive(Helper, Hinter, Highlighter, Validator)]
struct Completion {
    system: Rc<RefCell<System>>,
}

impl Completer for Completion {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line = line.get(..pos).unwrap_or(line);
        let (done, _) = split_partial(line);
        let pairs = candidates(&self.system.borrow(), line)
            .into_iter()
            .map(|c| Pair {
                display: c.clone(),
                replacement: format!("{c} "),
            })
            .collect();
        Ok((done.len(), pairs))
    }
}

/// Talks interactively; in the coding harness (`code`), shows the details
/// under each reply (problems, proposed changes) and reads the sources again
/// after changes are made.
pub fn run(common: &Common, harness: bool) -> Result<(), String> {
    let system = Rc::new(RefCell::new(open(common)));
    let shared = system.clone();
    let system = system.borrow();
    let mut session: Session<_> = match &common.session {
        Some(path) => load_session(path, system.source_refs())?,
        None => Session::new(system.source_refs()),
    };
    let sources: Vec<String> = system
        .world
        .sources
        .iter()
        .map(|s| s.name.clone())
        .collect();
    say!(
        "glassmind: talking about {} ({} names). Tab shows what can come next; Ctrl-D quits.",
        sources.join(", "),
        system.world.names().len()
    );
    let config = rustyline::Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
    let mut editor: Editor<Completion, rustyline::history::DefaultHistory> =
        Editor::with_config(config).map_err(|e| e.to_string())?;
    drop(system);
    editor.set_helper(Some(Completion {
        system: shared.clone(),
    }));
    loop {
        match editor.readline("> ") {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                let _ = editor.add_history_entry(line.as_str());
                let turn = shared.borrow().respond(&mut session, &line);
                if common.explain {
                    say!("{}", turn.trace.render());
                }
                say!("{}", turn.spoken);
                for line in crate::details::details(&turn.reply) {
                    say!("{line}");
                }
                if harness && matches!(turn.reply, glassmind_meaning::Reply::Applied { .. }) {
                    // The sources changed: read them again.
                    *shared.borrow_mut() = open(common);
                }
                if let Some(path) = &common.session {
                    save_session(path, &session)?;
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}
