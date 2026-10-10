//! Running a coding agent on a brief, in its own worktree.
//!
//! Claude Code agents run headless (`claude -p`) on the user's own login,
//! without saving their sessions, inside a bubblewrap sandbox that hides the
//! held-out data, the user's code and the user's Claude Code history, and
//! that cannot push anywhere.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use crate::exec::{Run, Sandbox};
use crate::state::{Config, Paths, Tokens};

/// What an agent run produced.
#[derive(Debug, Clone, Default)]
pub struct AgentRun {
    pub ok: bool,
    pub tokens: Tokens,
    /// The agent's final message.
    pub summary: String,
    pub log: String,
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// The sandbox for anything the loop runs on agent-written code.
pub fn sandbox(
    config: &Config,
    paths: &Paths,
    writable: Vec<PathBuf>,
    tmp: Option<PathBuf>,
) -> Option<Sandbox> {
    if !config.sandbox {
        return None;
    }
    let h = home();
    let mut writable = writable;
    writable.push(h.join(".cargo/registry"));
    writable.push(h.join(".cargo/git"));
    // Builds and results of checks on training sources (coding tasks).
    writable.push(h.join(".cache/glassmind/check"));
    // The login lives in ~/.claude and ~/.claude.json, which Claude Code
    // must be able to update; the history in them stays masked.
    writable.push(h.join(".claude"));
    writable.push(h.join(".claude.json"));
    let claude = h.join(".claude");
    Some(Sandbox {
        writable: writable.into_iter().filter(|p| p.exists()).collect(),
        masked: [
            "projects",
            "history.jsonl",
            "file-history",
            "sessions",
            "session-env",
            "paste-cache",
            "todos",
            "jobs",
        ]
        .iter()
        .map(|d| claude.join(d))
        .collect(),
        hidden: vec![
            h.join("code"),
            h.join(".ssh"),
            h.join(".config/gh"),
            h.join(".codex"),
            h.join(".gemini"),
            config.heldout.clone(),
            config.heldout_sources.clone(),
            paths.root.clone(),
        ],
        tmp,
    })
}

/// Sums the token use reported by `claude -p --output-format json`.
pub fn tokens_of(result: &Value) -> Tokens {
    let mut t = Tokens::default();
    if let Some(models) = result.get("modelUsage").and_then(Value::as_object) {
        for m in models.values() {
            let n = |k: &str| m.get(k).and_then(Value::as_u64).unwrap_or(0);
            t.input += n("inputTokens");
            t.output += n("outputTokens");
            t.cache_read += n("cacheReadInputTokens");
            t.cache_write += n("cacheCreationInputTokens");
        }
    }
    t.cost_usd = result
        .get("total_cost_usd")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    t
}

/// Runs Claude Code headless in `cwd` with `prompt`. `tools` limits the
/// tools it may use (empty: all, with permissions bypassed).
pub fn claude(
    config: &Config,
    paths: &Paths,
    slot: &Path,
    cwd: &Path,
    prompt: &str,
    tools: Option<&str>,
    writable: Vec<PathBuf>,
) -> Result<AgentRun, String> {
    let tmp = slot.join("tmp");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let mut args: Vec<String> = vec![
        "-p".into(),
        "--model".into(),
        config.model.clone(),
        "--output-format".into(),
        "json".into(),
        "--no-session-persistence".into(),
        "--setting-sources".into(),
        String::new(),
    ];
    match tools {
        Some(t) => {
            args.extend(["--tools".into(), t.into()]);
            args.extend(["--permission-mode".into(), "bypassPermissions".into()]);
        }
        None => args.push("--dangerously-skip-permissions".into()),
    }
    let mut writable = writable;
    writable.extend([slot.to_path_buf(), cwd.to_path_buf()]);
    let sb = sandbox(config, paths, writable, Some(tmp.clone()));
    // A usage limit is not a failed attempt: wait for it to reset and retry.
    let mut waited = 0;
    let (out, parsed) = loop {
        let out = Run::new("claude", &args, cwd)
            .remove_env_prefix("ANTHROPIC_")
            .remove_env_prefix("CLAUDE")
            .remove_env_prefix("OPENAI_")
            .remove_env_prefix("GEMINI_")
            .env("CLAUDE_CODE_TMPDIR", &tmp.to_string_lossy())
            .env("TMPDIR", &tmp.to_string_lossy())
            .env("CARGO_TARGET_DIR", &slot.join("target").to_string_lossy())
            // Four agents building at once: four jobs each keeps the machine responsive.
            .env("CARGO_BUILD_JOBS", "4")
            .stdin(prompt)
            .timeout(Duration::from_secs(config.agent_minutes * 60))
            .sandbox(sb.as_ref())
            .go()?;
        let parsed: Value = serde_json::from_str(out.stdout.trim()).unwrap_or(Value::Null);
        let limited = parsed.get("api_error_status").and_then(Value::as_u64) == Some(429);
        if !limited || waited >= 24 * 6 {
            break (out, parsed);
        }
        waited += 1;
        eprintln!("usage limit reached; waiting 10 minutes before retrying");
        std::thread::sleep(Duration::from_secs(600));
    };
    let summary = parsed
        .get("result")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let is_error = parsed
        .get("is_error")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    Ok(AgentRun {
        ok: out.ok && !is_error,
        tokens: tokens_of(&parsed),
        summary,
        log: format!(
            "timed out: {}\nstderr:\n{}\nstdout:\n{}",
            out.timed_out,
            out.stderr,
            out.stdout.chars().take(20_000).collect::<String>()
        ),
    })
}

/// Runs the configured agent on `brief` in `worktree`.
pub fn run_agent(
    config: &Config,
    paths: &Paths,
    slot: &Path,
    worktree: &Path,
    brief: &Path,
) -> Result<AgentRun, String> {
    if config.agent == "claude" {
        let prompt = format!(
            "You are a loop agent improving glassmind. Read your brief at {} and carry it out in this \
             repository ({}). Read docs/growing.md first. Leave your changes uncommitted. End with a short \
             summary of what you changed and why.",
            brief.display(),
            worktree.display()
        );
        claude(config, paths, slot, worktree, &prompt, None, Vec::new())
    } else {
        let args = vec![
            worktree.to_string_lossy().into_owned(),
            brief.to_string_lossy().into_owned(),
        ];
        let out = Run::new(&config.agent, &args, worktree)
            .timeout(Duration::from_secs(config.agent_minutes * 60))
            .go()?;
        Ok(AgentRun {
            ok: out.ok,
            tokens: Tokens::default(),
            summary: out.stdout.trim().to_owned(),
            log: out.stderr,
        })
    }
}
