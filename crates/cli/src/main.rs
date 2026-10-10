//! The `glassmind` command: ask about Rust sources in English.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use glassmind_answer::ConversationInput;
use glassmind_english::English;
use glassmind_mind::{Mind, Session};
use glassmind_rules::Switches;
use glassmind_world_rust::index::{SourceIndex, index_source};
use glassmind_world_rust::{Rust, RustWorld};

/// Prints a line to stdout, ignoring a closed pipe (`println!` would panic).
macro_rules! say {
    ($($t:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stdout(), $($t)*);
    }};
}

mod answer;
mod chat;
mod details;
mod edit;
mod system;
mod train;

use system::{System, load_session, parse_source, save_session};

#[derive(Parser)]
#[command(
    name = "glassmind",
    about = "Ask about Rust code in English. No neural network inside."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Args, Clone)]
struct Common {
    /// A Rust source to talk about, as `name=path` or `path`. Repeatable.
    /// Defaults to the current directory.
    #[arg(long = "source", short = 's')]
    sources: Vec<String>,
    /// Keep the conversation in this file, loading it first if it exists.
    #[arg(long)]
    session: Option<PathBuf>,
    /// Show the whole trace: words, parse, meaning, references, readings
    /// dropped and why, what was done, and how the reply was built.
    #[arg(long)]
    explain: bool,
    /// Print the machine-readable outcome (JSON) instead of the reply.
    #[arg(long)]
    json: bool,
    /// Switch off these rules (comma-separated IDs).
    #[arg(long, value_delimiter = ',')]
    disable: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Say one thing and print the reply.
    Ask {
        #[command(flatten)]
        common: Common,
        /// Make the edit the reply proposes (files are written; nothing is committed).
        #[arg(long)]
        apply: bool,
        /// What to say.
        text: Vec<String>,
    },
    /// Talk interactively; Tab offers what can come next.
    Chat {
        #[command(flatten)]
        common: Common,
    },
    /// The coding harness: talk about a repository, build and test it, and
    /// fix what fails. Changes are only made when you say yes.
    Code {
        #[command(flatten)]
        common: Common,
        /// The repository (defaults to the current directory).
        repo: Option<String>,
    },
    /// Print what may come next after an unfinished input.
    Complete {
        #[command(flatten)]
        common: Common,
        text: Vec<String>,
    },
    /// Run conversations from a JSONL file and print one outcome per turn.
    Eval {
        /// Directory holding each source under its name.
        #[arg(long)]
        sources: PathBuf,
        /// Conversations, one JSON object per line.
        #[arg(long)]
        inputs: PathBuf,
        /// Switch off these rules (comma-separated IDs).
        #[arg(long, value_delimiter = ',')]
        disable: Vec<String>,
    },
    /// Act on one request in a repository, as a coding agent: a fix or an
    /// edit glassmind finds is made (asking for it is consent); anything else
    /// (an answer, a question, "I don't know how") is printed, and written to
    /// `--reply-file` if given. Nothing is committed.
    Do {
        /// The repository to work in.
        repo: PathBuf,
        /// Where to write the reply when no change is made.
        #[arg(long)]
        reply_file: Option<PathBuf>,
        /// Show the whole trace.
        #[arg(long)]
        explain: bool,
        /// The request.
        text: Vec<String>,
    },
    /// Learn the normalizer's edit model from turns answered right (eval
    /// outcomes and their labels), and write it as text.
    TrainNormalizer {
        #[arg(long)]
        inputs: PathBuf,
        #[arg(long)]
        outcomes: PathBuf,
        #[arg(long)]
        labels: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Learn the request-category tagger from turns answered right (eval
    /// outcomes against their labels).
    TrainIntents {
        #[arg(long)]
        inputs: PathBuf,
        #[arg(long)]
        outcomes: PathBuf,
        #[arg(long)]
        labels: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// List every rule with the round that added it.
    Rules {
        /// Also show what each grammar rule reads.
        #[arg(long)]
        detail: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Ask {
            common,
            apply,
            text,
        } => ask(&common, apply, &text.join(" ")),
        Command::Chat { common } => chat::run(&common, false),
        Command::Code { mut common, repo } => {
            common.sources.extend(repo);
            chat::run(&common, true)
        }
        Command::Complete { common, text } => complete(&common, &text.join(" ")),
        Command::Eval {
            sources,
            inputs,
            disable,
        } => eval(&sources, &inputs, Switches::without(disable)),
        Command::Do {
            repo,
            reply_file,
            explain,
            text,
        } => act(&repo, reply_file.as_deref(), explain, &text.join(" ")),
        Command::TrainNormalizer {
            inputs,
            outcomes,
            labels,
            out,
        } => train::normalizer(&inputs, &outcomes, &labels, &out).map(|report| say!("{report}")),
        Command::TrainIntents {
            inputs,
            outcomes,
            labels,
            out,
        } => train::intents(&inputs, &outcomes, &labels, &out).map(|report| say!("{report}")),
        Command::Rules { detail } => {
            for problem in system::rule_problems() {
                eprintln!("problem: {problem}");
            }
            for (rule, what) in system::all_rules_detailed() {
                if detail {
                    say!("{}\t{}\t{what}", rule.id, rule.round);
                } else {
                    say!("{}\t{}", rule.id, rule.round);
                }
            }
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("glassmind: {e}");
            ExitCode::FAILURE
        }
    }
}

fn open(common: &Common) -> System {
    let sources: Vec<(String, PathBuf)> = if common.sources.is_empty() {
        vec![parse_source(".")]
    } else {
        common.sources.iter().map(|s| parse_source(s)).collect()
    };
    System::open(&sources, Switches::without(common.disable.clone()))
}

fn ask(common: &Common, apply: bool, text: &str) -> Result<(), String> {
    let system = open(common);
    let mut session = match &common.session {
        Some(path) => load_session(path, system.source_refs())?,
        None => Session::new(system.source_refs()),
    };
    let turn = system.respond(&mut session, text);
    if common.json {
        let outcome = answer::outcome("ask", &turn, answer::canonical(&system.english, &turn));
        say!(
            "{}",
            serde_json::to_string(&outcome).map_err(|e| e.to_string())?
        );
    } else {
        if common.explain {
            say!("{}", turn.trace.render());
        }
        say!("{}", turn.spoken);
        if !matches!(turn.reply, glassmind_meaning::Reply::Proposal { .. }) {
            for line in details::details(&turn.reply) {
                say!("{line}");
            }
        }
    }
    if let Some(changes) = edit::changes(&answer::response(&turn.reply)) {
        if !common.json {
            say!("{}", edit::describe(&changes));
        }
        if apply {
            let roots = system
                .world
                .sources
                .iter()
                .map(|s| (s.name.clone(), s.root.clone()))
                .collect();
            let n = edit::apply(&changes, &roots)?;
            say!("Applied {n} changes. Nothing was committed.");
        } else if !common.json {
            say!("(Nothing changed. Run the same request with --apply to make these changes.)");
        }
    }
    if let Some(path) = &common.session {
        save_session(path, &session)?;
    }
    Ok(())
}

fn act(
    repo: &std::path::Path,
    reply_file: Option<&std::path::Path>,
    explain: bool,
    text: &str,
) -> Result<(), String> {
    use glassmind_meaning::Reply;
    use glassmind_meaning::interface::World as _;
    let sources = vec![parse_source(&repo.to_string_lossy())];
    let system = System::open(&sources, Switches::all_on());
    let mut session = Session::new(system.source_refs());
    let turn = system.respond(&mut session, text);
    if explain {
        say!("{}", turn.trace.render());
    }
    say!("{}", turn.spoken);
    for line in details::details(&turn.reply) {
        say!("{line}");
    }
    let mut inner = &turn.reply;
    while let Reply::Assuming { reply, .. } = inner {
        inner = reply;
    }
    let changes = match inner {
        Reply::Remedy { changes, left, .. } => !changes.is_empty() && left.is_empty(),
        Reply::Proposal { .. } => true,
        _ => false,
    };
    if changes {
        let mut fired = Vec::new();
        match system.world.apply(inner, &mut fired) {
            Ok(applied) => {
                if let Some(said) =
                    glassmind_meaning::interface::Language::say(&system.english, &applied)
                {
                    say!("{}", said.text);
                }
                return Ok(());
            }
            Err(e) => say!("could not make the changes: {e:?}"),
        }
    }
    if let Some(path) = reply_file {
        std::fs::write(path, format!("{}\n", turn.spoken)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn complete(common: &Common, text: &str) -> Result<(), String> {
    let system = open(common);
    for candidate in chat::candidates(&system, text) {
        say!("{candidate}");
    }
    Ok(())
}

fn eval(
    sources_dir: &std::path::Path,
    inputs: &std::path::Path,
    switches: Switches,
) -> Result<(), String> {
    let file = std::fs::File::open(inputs)
        .map_err(|e| format!("cannot open {}: {e}", inputs.display()))?;
    let english: English<Rust> = English::new(glassmind_english_rust::rules(), &switches);
    let mut indices: BTreeMap<String, Arc<SourceIndex>> = BTreeMap::new();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    for line in std::io::BufReader::new(file).lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let conversation: ConversationInput =
            serde_json::from_str(&line).map_err(|e| format!("bad conversation: {e}"))?;
        let mut shared = Vec::new();
        for name in &conversation.sources {
            let index = indices
                .entry(name.clone())
                .or_insert_with(|| Arc::new(index_source(name, &sources_dir.join(name))))
                .clone();
            shared.push(index);
        }
        let world = RustWorld::from_indices(shared, &switches);
        let mind = Mind {
            language: &english,
            world: &world,
            switches: &switches,
        };
        let mut session = Session::new(Vec::new());
        for turn in &conversation.turns {
            let result = mind.respond(&mut session, &turn.text);
            let outcome = answer::outcome(&turn.id, &result, answer::canonical(&english, &result));
            let json = serde_json::to_string(&outcome).map_err(|e| e.to_string())?;
            writeln!(out, "{json}").map_err(|e| e.to_string())?;
        }
    }
    out.flush().map_err(|e| e.to_string())
}
