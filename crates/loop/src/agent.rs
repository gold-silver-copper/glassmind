//! Running a coding agent on a brief, in its own worktree.
//!
//! Codex agents use the user's ChatGPT login, GPT-6 Astra and the fast tier.
//! API keys are removed. Each ephemeral worker has a private Codex home and
//! a sandbox hiding held-out data, unrelated code and account history.

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
    // Agents' own builds and results of checks on training sources (the
    // scorer keeps its own where agents cannot see them).
    writable.push(h.join(".cache/glassmind/check"));
    Some(Sandbox {
        readable: Vec::new(),
        writable: writable.into_iter().filter(|p| p.exists()).collect(),
        masked: Vec::new(),
        hidden: vec![
            h.join("code"),
            h.join(".ssh"),
            h.join(".config/gh"),
            h.join(".codex"),
            h.join(".claude"),
            h.join(".claude.json"),
            h.join(".gemini"),
            config.heldout.clone(),
            config.heldout_sources.clone(),
            paths.root.clone(),
        ]
        .into_iter()
        .chain(config.private.iter().cloned())
        .collect(),
        tmp,
    })
}

/// Token usage from Codex's JSONL events; cached input is part of input_tokens.
fn tokens_of(event: &Value) -> Tokens {
    let usage = event.get("usage").unwrap_or(&Value::Null);
    let count = |key: &str| usage.get(key).and_then(Value::as_u64).unwrap_or(0);
    let cached = count("cached_input_tokens");
    Tokens {
        input: count("input_tokens").saturating_sub(cached),
        cache_read: cached,
        output: count("output_tokens"),
        ..Tokens::default()
    }
}

/// Runs Codex with ChatGPT authentication only. A legacy Claude configuration
/// fails explicitly instead of launching an unintended provider.
pub fn model_agent(
    config: &Config,
    paths: &Paths,
    slot: &Path,
    cwd: &Path,
    prompt: &str,
    tools: Option<&str>,
    writable: Vec<PathBuf>,
) -> Result<AgentRun, String> {
    if config.agent != "codex" || config.model != "gpt-6-astra" {
        return Err("configure agent=codex and model=gpt-6-astra; no provider fallback".into());
    }
    let tmp = slot.join("tmp");
    let codex_home = slot.join("codex-home");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&codex_home).map_err(|e| e.to_string())?;
    // Let Codex itself use the login; never read or copy its contents.
    let auth = home().join(".codex/auth.json");
    let login = codex_home.join("auth.json");
    if !login.exists() {
        std::os::unix::fs::symlink(&auth, &login).map_err(|e| e.to_string())?;
    }
    let args: Vec<String> = [
        "--no-daemon",
        "exec",
        "--ignore-user-config",
        "--ephemeral",
        "--model",
        "gpt-6-astra",
        "-c",
        "service_tier=\"fast\"",
        "-c",
        "forced_login_method=\"chatgpt\"",
        "-c",
        "model_reasoning_effort=\"high\"",
        "--dangerously-bypass-approvals-and-sandbox",
        "--json",
        "-",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let mut writable = writable;
    writable.extend([slot.to_path_buf(), cwd.to_path_buf()]);
    let mut sb = sandbox(config, paths, writable, Some(tmp.clone()));
    if let Some(s) = &mut sb {
        s.readable.push(auth);
        if tools.is_some() {
            // Review workers need no repository writes.
            s.writable.retain(|p| p != cwd);
            s.readable.push(cwd.to_path_buf());
        }
    }
    let prompt = format!(
        "{prompt}\nRead prompts/grow-fast.md for current policy. \
        Use the ChatGPT account only, never an API key or credential-file reads. \
        Do not launch more agents. Set TMPDIR for cargo; use a memory-capped scope, \
        an isolated process group, -j 1 and --test-threads 4. Leave changes uncommitted."
    );
    let mut run = Run::new("codex", &args, cwd)
        .remove_env_prefix("ANTHROPIC_")
        .remove_env_prefix("CLAUDE")
        .remove_env_prefix("OPENAI_")
        .remove_env_prefix("GEMINI_")
        .env("CODEX_HOME", &codex_home.to_string_lossy())
        .env("TMPDIR", &tmp.to_string_lossy())
        .env("CARGO_TARGET_DIR", &slot.join("target").to_string_lossy())
        .env("CARGO_BUILD_JOBS", "1")
        .stdin(&prompt)
        .timeout(Duration::from_secs(config.agent_minutes * 60))
        .memory("8G")
        .sandbox(sb.as_ref());
    for (key, _) in std::env::vars().filter(|(k, _)| k.contains("API_KEY")) {
        run = run.remove_env_prefix(&key);
    }
    let out = run.go()?;
    let mut summary = String::new();
    let mut tokens = Tokens::default();
    let mut completed = false;
    let mut failed = false;
    for line in out.stdout.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match event.get("type").and_then(Value::as_str) {
            Some("turn.completed") => {
                completed = true;
                tokens.add(&tokens_of(&event));
            }
            Some("turn.failed" | "error") => failed = true,
            Some("item.completed") => {
                if let Some(item) = event.get("item")
                    && item.get("type").and_then(Value::as_str) == Some("agent_message")
                {
                    summary = item
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                }
            }
            _ => {}
        }
    }
    Ok(AgentRun {
        ok: out.ok && completed && !failed && !summary.is_empty(),
        tokens,
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
    if config.agent == "codex" || config.agent == "claude" {
        let prompt = format!(
            "You are a loop agent improving glassmind. Read your brief at {} and carry it out in this \
             repository ({}). Read docs/growing.md first. Leave your changes uncommitted. End with a short \
             summary of what you changed and why.",
            brief.display(),
            worktree.display()
        );
        model_agent(config, paths, slot, worktree, &prompt, None, Vec::new())
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
