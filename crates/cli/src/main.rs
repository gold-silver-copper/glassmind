//! The `glassmind` command: ask about Rust sources in English.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
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
mod coding;
mod details;
mod edit;
mod everyday;
mod system;
mod talk_facts;
mod talk_stories;
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
    /// Conversation: remember preferences, follow stories, calculate units and dates.
    #[command(
        long_about = "Remember names and preferences, follow story facts, and calculate exact answers.\n\nTry: Mary went to the kitchen. / Mary picked up the apple. / Where is the apple?\nEnter each statement or question on its own line. Use --session FILE to continue later.\n/reset clears memory and story facts; /quit leaves the conversation."
    )]
    Talk {
        /// Persist memory and story facts between runs; /reset clears both.
        #[arg(long)]
        session: Option<PathBuf>,
        /// Local facts bundle: manifest.json, index.json, relation.json, optional labels.json.
        #[arg(long)]
        facts_dir: Option<PathBuf>,
        /// Optional learned everyday-intent model (no services are executed).
        #[arg(long)]
        model: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
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
        /// Conversations answered at once (they are independent; the
        /// outcomes come out in input order whatever this is).
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Answer only the conversations that use one of these rules in an
        /// earlier outcome file (`--only-using-from`): the others cannot
        /// change when only these rules are switched off.
        #[arg(long, value_delimiter = ',')]
        only_using: Vec<String>,
        #[arg(long)]
        only_using_from: Option<PathBuf>,
    },
    /// Act on one request in a repository, as a coding agent: a fix or an
    /// edit glassmind finds is made (asking for it is consent); anything else
    /// (an answer, a question, "I don't know how") is printed, and written to
    /// `--reply-file` if given. Nothing is committed.
    Do {
        /// The repository to work in.
        repo: PathBuf,
        /// Where to write the final answer or result of applying changes.
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
        Command::Talk {
            session,
            model,
            facts_dir,
            json,
        } => everyday::run(session, model, facts_dir, json),
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
            jobs,
            only_using,
            only_using_from,
        } => (|| {
            // The turns whose earlier outcome used one of the rules.
            let only = match only_using_from {
                Some(path) if !only_using.is_empty() => {
                    let text = std::fs::read_to_string(&path)
                        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
                    let mut ids = BTreeSet::new();
                    for line in text.lines().filter(|l| !l.trim().is_empty()) {
                        let o: glassmind_answer::Outcome =
                            serde_json::from_str(line).map_err(|e| format!("bad outcome: {e}"))?;
                        if o.rules.iter().any(|r| only_using.contains(r)) {
                            ids.insert(o.turn_id);
                        }
                    }
                    Some(ids)
                }
                _ => None,
            };
            eval(&sources, &inputs, Switches::without(disable), jobs, only)
        })(),
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
                    if let Some(path) = reply_file {
                        std::fs::write(path, format!("{}\n", said.text))
                            .map_err(|e| e.to_string())?;
                    }
                }
                return Ok(());
            }
            Err(e) => {
                let message = format!("could not make and verify the changes: {e:?}");
                if let Some(path) = reply_file {
                    std::fs::write(path, format!("{message}\n")).map_err(|e| e.to_string())?;
                }
                return Err(message);
            }
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

/// One conversation's outcomes, as JSON lines.
type Answered = Result<Vec<String>, String>;

fn eval(
    sources_dir: &std::path::Path,
    inputs: &std::path::Path,
    switches: Switches,
    jobs: usize,
    only: Option<BTreeSet<String>>,
) -> Result<(), String> {
    let text = std::fs::read_to_string(inputs)
        .map_err(|e| format!("cannot read {}: {e}", inputs.display()))?;
    let conversations: Vec<ConversationInput> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).map_err(|e| format!("bad conversation: {e}")))
        .collect::<Result<_, _>>()?;
    let conversations: Vec<ConversationInput> = match &only {
        Some(ids) => conversations
            .into_iter()
            .filter(|c| c.turns.iter().any(|t| ids.contains(&t.id)))
            .collect(),
        None => conversations,
    };
    let english: English<Rust> = English::new(glassmind_english_rust::rules(), &switches);
    // Every source indexed once, before the conversations share them.
    let names: BTreeSet<&String> = conversations.iter().flat_map(|c| &c.sources).collect();
    let indices: BTreeMap<String, Arc<SourceIndex>> = names
        .into_iter()
        .map(|name| {
            (
                name.clone(),
                Arc::new(index_source(name, &sources_dir.join(name))),
            )
        })
        .collect();
    let answer_one = |conversation: &ConversationInput| -> Result<Vec<String>, String> {
        let shared = conversation
            .sources
            .iter()
            .filter_map(|n| indices.get(n).cloned())
            .collect();
        let world = RustWorld::from_indices(shared, &switches);
        let language = coding::CodingLanguage {
            english: &english,
            switches: &switches,
        };
        let mind = Mind {
            language: &language,
            world: &world,
            switches: &switches,
        };
        let mut session = Session::new(Vec::new());
        conversation
            .turns
            .iter()
            .map(|turn| {
                let result = mind.respond(&mut session, &turn.text);
                let outcome =
                    answer::outcome(&turn.id, &result, answer::canonical(&english, &result));
                serde_json::to_string(&outcome).map_err(|e| e.to_string())
            })
            .collect()
    };
    // Workers take the next conversation in turn; each one's outcomes are
    // kept at its place, so the output keeps the input order.
    let next = std::sync::atomic::AtomicUsize::new(0);
    let done: Vec<std::sync::Mutex<Option<Answered>>> = conversations
        .iter()
        .map(|_| std::sync::Mutex::new(None))
        .collect();
    std::thread::scope(|s| {
        for _ in 0..jobs.max(1) {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let (Some(conversation), Some(slot)) = (conversations.get(i), done.get(i))
                    else {
                        break;
                    };
                    let answered = answer_one(conversation);
                    if let Ok(mut slot) = slot.lock() {
                        *slot = Some(answered);
                    }
                }
            });
        }
    });
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    for slot in done {
        let lines = slot
            .into_inner()
            .map_err(|_| "a worker failed".to_owned())?
            .unwrap_or_else(|| Err("a conversation was not answered".into()))?;
        for line in lines {
            writeln!(out, "{line}").map_err(|e| e.to_string())?;
        }
    }
    out.flush().map_err(|e| e.to_string())
}
