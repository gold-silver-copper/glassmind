//! `glassmind-oracle`: see `docs/tasks.md` and the crate docs.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use glassmind_oracle::answer::{Answerer, Request, error_line};
use glassmind_oracle::source::{self, Source};
use glassmind_oracle::tasks::{self, KINDS, Tier};

const USAGE: &str = "usage:
  glassmind-oracle tasks --source <name>=<path> [--source ...] [--per-kind N] [--seed S] --out <file.jsonl>
  glassmind-oracle fresh --source <name>=<path> <ident>...
  glassmind-oracle stats --source <name>=<path> [--source ...]
  glassmind-oracle answer --source <name>=<path> [--source ...] --in <requests.jsonl> --out <tasks.jsonl> [--seed S]";

#[derive(Default)]
struct Args {
    sources: Vec<(String, PathBuf)>,
    per_kind: usize,
    seed: u64,
    out: Option<PathBuf>,
    input: Option<PathBuf>,
    rest: Vec<String>,
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut parsed = Args {
        per_kind: 40,
        ..Args::default()
    };
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| args.next().ok_or(format!("{flag} needs a value"));
        match arg.as_str() {
            "--source" => {
                let v = value("--source")?;
                let (name, path) = v
                    .split_once('=')
                    .ok_or(format!("--source wants <name>=<path>, got {v}"))?;
                parsed.sources.push((name.to_string(), PathBuf::from(path)));
            }
            "--per-kind" => {
                parsed.per_kind = value("--per-kind")?
                    .parse()
                    .map_err(|e| format!("--per-kind: {e}"))?;
            }
            "--seed" => {
                parsed.seed = value("--seed")?
                    .parse()
                    .map_err(|e| format!("--seed: {e}"))?;
            }
            "--in" => parsed.input = Some(PathBuf::from(value("--in")?)),
            "--out" => parsed.out = Some(PathBuf::from(value("--out")?)),
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag}")),
            _ => parsed.rest.push(arg),
        }
    }
    if parsed.sources.is_empty() {
        return Err("at least one --source is required".into());
    }
    Ok(parsed)
}

fn load_all(args: &Args) -> Result<Vec<Source>, String> {
    args.sources
        .iter()
        .map(|(name, path)| {
            source::load(name, path).map_err(|e| format!("{name}: {}: {e}", path.display()))
        })
        .collect()
}

fn run() -> Result<(), String> {
    let mut argv = std::env::args().skip(1);
    let command = argv.next().ok_or(USAGE.to_string())?;
    let args = parse_args(argv).map_err(|e| format!("{e}\n{USAGE}"))?;
    match command.as_str() {
        "tasks" => {
            let out = args
                .out
                .clone()
                .ok_or(format!("tasks needs --out\n{USAGE}"))?;
            let sources = load_all(&args)?;
            let file = File::create(&out).map_err(|e| format!("{}: {e}", out.display()))?;
            let mut w = BufWriter::new(file);
            let mut total = 0;
            for source in &sources {
                for kind in KINDS {
                    let chosen = tasks::select(
                        tasks::candidates(source, kind, args.seed),
                        args.per_kind,
                        args.seed,
                    );
                    for task in chosen {
                        let line = serde_json::to_string(&task).map_err(|e| e.to_string())?;
                        writeln!(w, "{line}").map_err(|e| e.to_string())?;
                        total += 1;
                    }
                }
            }
            w.flush().map_err(|e| e.to_string())?;
            eprintln!("wrote {total} tasks to {}", out.display());
        }
        "answer" => {
            let input = args
                .input
                .clone()
                .ok_or(format!("answer needs --in\n{USAGE}"))?;
            let out = args
                .out
                .clone()
                .ok_or(format!("answer needs --out\n{USAGE}"))?;
            let sources = load_all(&args)?;
            let text =
                std::fs::read_to_string(&input).map_err(|e| format!("{}: {e}", input.display()))?;
            let file = File::create(&out).map_err(|e| format!("{}: {e}", out.display()))?;
            let mut w = BufWriter::new(file);
            let mut answerer = Answerer::new(&sources, args.seed);
            for (n, line) in text
                .lines()
                .enumerate()
                .filter(|(_, l)| !l.trim().is_empty())
            {
                let value = match serde_json::from_str::<Request>(line) {
                    Ok(request) => answerer.answer(&request),
                    Err(e) => {
                        let id = serde_json::from_str::<serde_json::Value>(line)
                            .ok()
                            .and_then(|v| v.get("id").cloned())
                            .unwrap_or(serde_json::Value::Null);
                        error_line(&id, &format!("line {}: {e}", n + 1))
                    }
                };
                writeln!(w, "{value}").map_err(|e| e.to_string())?;
            }
            w.flush().map_err(|e| e.to_string())?;
        }
        "fresh" => {
            let sources = load_all(&args)?;
            for ident in &args.rest {
                let taken = sources.iter().any(|s| s.taken.contains(ident));
                println!("{ident} {}", if taken { "taken" } else { "fresh" });
            }
        }
        "stats" => {
            for source in &load_all(&args)? {
                stats(source, args.seed);
            }
        }
        _ => return Err(USAGE.to_string()),
    }
    Ok(())
}

fn stats(source: &Source, seed: u64) {
    println!(
        "source {}: {} files parsed, {} skipped",
        source.name,
        source.files.len(),
        source.skipped.len()
    );
    let mut by_kind = std::collections::BTreeMap::new();
    for item in &source.items {
        *by_kind.entry(format!("{:?}", item.key.kind)).or_insert(0) += 1;
    }
    for (kind, n) in by_kind {
        println!("  items {kind:<10} {n}");
    }
    for kind in KINDS {
        let candidates = tasks::candidates(source, kind, seed);
        let preferred = candidates
            .iter()
            .filter(|c| c.tier == Tier::Preferred)
            .count();
        let zero = candidates.iter().filter(|c| c.tier == Tier::Zero).count();
        println!(
            "  tasks {kind:<26} {:>6} candidates, {preferred:>6} preferred, {zero:>5} zero",
            candidates.len()
        );
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
