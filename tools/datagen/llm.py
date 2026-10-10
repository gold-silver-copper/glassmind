"""Calling Claude Opus 5.5 headless, isolated from the user's own Claude Code history.

Every call runs the plain `claude` CLI on the user's own login (never an API
key), with no tools, no settings, and no saved session, so nothing is added
to the user's Claude Code history.
"""

import json
import os
import subprocess
import time
from pathlib import Path

MODEL = "claude-opus-5-5"
HOME = Path.home() / ".cache" / "glassmind"
USAGE_LOG = HOME / "usage" / "datagen.jsonl"


def env():
    """The environment for a child `claude`: the user's own login, with no
    API keys and none of this session's variables."""
    return {k: v for k, v in os.environ.items()
            if not k.startswith(("ANTHROPIC_", "CLAUDE_CODE_", "CLAUDECODE", "OPENAI_", "GEMINI_"))}


def ask(prompt, system, label, retries=4, timeout=1800):
    """Returns the model's text reply to `prompt`. Logs token usage under `label`."""
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
