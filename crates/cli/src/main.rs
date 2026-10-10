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
mod system;

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
        /// What to say.
        text: Vec<String>,
    },
    /// Talk interactively; Tab offers what can come next.
    Chat {
        #[command(flatten)]
        common: Common,
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
        Command::Ask { common, text } => ask(&common, &text.join(" ")),
        Command::Chat { common } => chat::run(&common),
        Command::Complete { common, text } => complete(&common, &text.join(" ")),
        Command::Eval {
            sources,
            inputs,
            disable,
        } => eval(&sources, &inputs, Switches::without(disable)),
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

fn ask(common: &Common, text: &str) -> Result<(), String> {
    let system = open(common);
    let mut session = match &common.session {
        Some(path) => load_session(path, system.source_refs())?,
        None => Session::new(system.source_refs()),
    };
    let turn = system.respond(&mut session, text);
    if common.json {
        let outcome = answer::outcome("ask", &turn);
        say!(
            "{}",
            serde_json::to_string(&outcome).map_err(|e| e.to_string())?
        );
    } else {
        if common.explain {
            say!("{}", turn.trace.render());
        }
        say!("{}", turn.spoken);
    }
    if let Some(path) = &common.session {
        save_session(path, &session)?;
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
            let outcome = answer::outcome(&turn.id, &result);
            let json = serde_json::to_string(&outcome).map_err(|e| e.to_string())?;
            writeln!(out, "{json}").map_err(|e| e.to_string())?;
        }
    }
    out.flush().map_err(|e| e.to_string())
}
