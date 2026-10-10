"""Calling Claude Opus 5.5 headless, isolated from the user's own Claude Code history.

Every call runs `claude -p` with its own config directory, no tools, no
settings, and no saved session, authenticated by a long-lived OAuth token
(never an API key). The token lives in ~/.cache/glassmind/oauth-token.
"""

import json
import os
import subprocess
import time
from pathlib import Path

MODEL = "claude-opus-5-5"
HOME = Path.home() / ".cache" / "glassmind"
CONFIG_DIR = HOME / "claude-gen"
TOKEN_FILE = HOME / "oauth-token"
USAGE_LOG = HOME / "usage" / "datagen.jsonl"


def env():
    token = TOKEN_FILE.read_text().strip()
    e = {k: v for k, v in os.environ.items() if not k.startswith(("ANTHROPIC_", "CLAUDE_CODE_", "CLAUDECODE", "OPENAI_", "GEMINI_"))}
    e["CLAUDE_CONFIG_DIR"] = str(CONFIG_DIR)
    e["CLAUDE_CODE_OAUTH_TOKEN"] = token
    return e


def ask(prompt, system, label, retries=4, timeout=1800):
    """Returns the model's text reply to `prompt`. Logs token usage under `label`."""
    CONFIG_DIR.mkdir(parents=True, exist_ok=True)
    USAGE_LOG.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        "claude", "-p", "--model", MODEL, "--output-format", "json",
        "--tools", "", "--no-session-persistence", "--setting-sources", "",
        "--system-prompt", system,
    ]
    last = None
    for attempt in range(retries):
        try:
            p = subprocess.run(cmd, input=prompt, capture_output=True, text=True, env=env(), timeout=timeout,
                               cwd=str(HOME))
            out = json.loads(p.stdout)
            if out.get("is_error"):
                raise RuntimeError(out.get("result", "error"))
            usage = out.get("usage", {})
            with USAGE_LOG.open("a") as f:
                f.write(json.dumps({"label": label, "time": time.time(), "usage": usage,
                                    "cost_usd": out.get("total_cost_usd")}) + "\n")
            return out.get("result", "")
        except Exception as e:  # retry on any failure, with backoff (usage limits included)
            last = e
            time.sleep(min(600, 30 * (2 ** attempt)))
    raise RuntimeError(f"claude call failed after {retries} tries: {last}")


def json_block(text):
    """The first JSON value in a reply, inside a ```json fence or bare."""
    if "```" in text:
        parts = text.split("```")
        for part in parts[1::2]:
            body = part[4:] if part.startswith("json") else part
            try:
                return json.loads(body)
            except json.JSONDecodeError:
                continue
    start = min([i for i in (text.find("["), text.find("{")) if i >= 0], default=-1)
    return json.loads(text[start:]) if start >= 0 else None
