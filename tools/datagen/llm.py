"""Codex GPT-6 Astra fast, through the user's ChatGPT login, never API keys."""

import json
import os
import subprocess
import time
from pathlib import Path

MODEL = "gpt-6-astra"
HOME = Path.home() / ".cache" / "glassmind"
USAGE_LOG = HOME / "usage" / "datagen.jsonl"


def env():
    """The environment for a child Codex: the user's ChatGPT login, with no
    API keys and none of this session's variables."""
    return {k: v for k, v in os.environ.items()
            if "API_KEY" not in k
            and not k.startswith(("ANTHROPIC_", "CLAUDE", "OPENAI_", "GEMINI_"))}


def ask(prompt, system, label, retries=4, timeout=1800):
    """Returns the model's text reply to `prompt`. Logs token usage under `label`."""
    USAGE_LOG.parent.mkdir(parents=True, exist_ok=True)
    tmp = HOME / "tmp"
    tmp.mkdir(parents=True, exist_ok=True)
    child_env = env()
    child_env["TMPDIR"] = str(tmp)
    cmd = [
        "codex", "--no-daemon", "exec", "--model", MODEL,
        "--ignore-user-config", "--ephemeral", "--json", "--sandbox", "read-only",
        "--skip-git-repo-check", "-c", 'service_tier="fast"',
        "-c", 'forced_login_method="chatgpt"', "-",
    ]
    last = None
    for attempt in range(retries):
        try:
            p = subprocess.run(cmd, input=system + "\n\nDo not use tools. Return only the requested answer.\n\n" + prompt,
                               capture_output=True, text=True, env=child_env, timeout=timeout,
                               cwd=str(HOME))
            events = [json.loads(line) for line in p.stdout.splitlines() if line.strip()]
            completed = [e for e in events if e.get("type") == "turn.completed"]
            replies = [e["item"]["text"] for e in events
                       if e.get("type") == "item.completed"
                       and e.get("item", {}).get("type") == "agent_message"]
            if p.returncode or not completed or not replies:
                raise RuntimeError("Codex did not complete a ChatGPT-authenticated turn: " + p.stderr[-1000:])
            usage = {key: sum(e.get("usage", {}).get(key, 0) for e in completed)
                     for key in ("input_tokens", "cached_input_tokens", "output_tokens")}
            with USAGE_LOG.open("a") as f:
                f.write(json.dumps({"label": label, "time": time.time(), "usage": usage,
                                    "model": MODEL, "service_tier": "fast", "auth": "chatgpt"}) + "\n")
            return replies[-1]
        except Exception as e:  # retry on any failure, with backoff (usage limits included)
            last = e
            time.sleep(min(600, 30 * (2 ** attempt)))
    raise RuntimeError(f"Codex call failed after {retries} tries: {last}")


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
