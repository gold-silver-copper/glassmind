"""Phrases planned conversations as a user would type them, with Claude Opus 5.5.

Each generator is a different prompt family (persona, style and template).
The held-out generator's prompt is kept outside the repository while the loop
runs, so loop agents cannot imitate it; pass it with --persona-file.

    python3 tools/datagen/phrase.py --plans plans.jsonl --out phrased.jsonl [--persona-file F]
"""

import argparse
import json
import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import llm

COMMON = """You write what a user types to a tool that answers questions about a Rust codebase and makes small edits.
You are given conversation plans: for each turn, an instruction saying what the user asks for, with facts about the code.
Write each turn as the user would type it. Rules:
- Ask for exactly what the instruction says; no more, no less. Do not add a second question.
- Write code identifiers exactly as given (users copy-paste them), with or without backticks as the persona would.
- Never write the answer, and never mention facts the user would not know (file paths, line numbers) unless the persona would.
- Turns of one conversation follow each other; later turns may lean on earlier ones as instructed.
- When an instruction asks you to choose a new name or a nickname, report it in "new_name" or "nickname".
Reply with only a JSON array, one object per conversation, in the order given:
[{"conv_id": "...", "turns": [{"text": "...", "new_name": "optional", "nickname": "optional"}]}]"""

PERSONAS = {
    # Added after loop round 10, written from the brief's requirement that
    # generators imitate how people actually type (short, lowercase, missing
    # words, typos), not from any held-out example.
    "hurried": """Persona: someone typing fast into a chat box on a laptop while in the middle of something else.
Very short and all lowercase. Often drops articles, "is"/"are", "do"/"does" and question marks ("what fields Player have", "where spawn_wave").
Frequent typos in ordinary English words: about one word in five has a dropped, doubled or swapped letter ("fucntion", "whcih", "defiend", "teh").
Never a typo inside a code identifier. Sometimes shorthand: u, pls, w/, fn, smth. Sometimes the words come in an odd order.""",
    "terse": """Persona: a senior engineer firing quick queries into a terminal tool between meetings.
Mostly lowercase, as few words as possible, articles and question marks often dropped ("callers of x", "where's foo defined").
Sometimes abbreviations (fn, func). An occasional typo in an ordinary English word, never in identifiers.
Vary the wording from turn to turn; do not reuse one template.""",
    "chatty": """Persona: a friendly developer pair-programming with a colleague, writing complete, polite sentences.
Sometimes a short preamble ("hey", "quick question", "ok so") or a reason ("I'm refactoring the store"), sometimes "please"/"thanks".
Normal capitalization and punctuation. Vary sentence shapes: direct questions, "can you tell me ...", "I'd like to know ...".""",
    "newcomer": """Persona: someone who joined the team this week and is still finding their way around the code.
Slightly unsure and indirect ("I'm trying to figure out where ...", "is there anything that ends up calling ...", "what is ... even for?").
Sometimes filler words, sometimes slightly non-native English (missing articles, odd prepositions). Mixed capitalization.""",
}


def prompt_for(batch):
    items = []
    for plan in batch:
        turns = []
        for t in plan["turns"]:
            target = (t.get("info") or {}).get("target") or {}
            facts = {k: v for k, v in target.items() if k in ("kind", "signature", "doc", "owner", "struct")}
            turns.append({"instruction": t["instruction"], "facts": facts})
        items.append({"conv_id": plan["conv_id"], "turns": turns})
    return "Conversation plans:\n" + json.dumps(items, indent=1) + "\n\nReply with the JSON array only."


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--plans", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--persona-file", help="generator name and prompt for a generator not in this file: NAME=PATH")
    ap.add_argument("--batch", type=int, default=12)
    ap.add_argument("--workers", type=int, default=4)
    a = ap.parse_args()
    personas = dict(PERSONAS)
    if a.persona_file:
        name, path = a.persona_file.split("=", 1)
        personas[name] = Path(path).read_text()
    plans = [json.loads(l) for l in open(a.plans) if l.strip()]
    done = set()
    if Path(a.out).exists():
        done = {json.loads(l)["conv_id"] for l in open(a.out) if l.strip()}
    todo = [p for p in plans if p["conv_id"] not in done]
    batches = []
    for gen in sorted({p["generator"] for p in todo}):
        mine = [p for p in todo if p["generator"] == gen]
        batches += [(gen, mine[i:i + a.batch]) for i in range(0, len(mine), a.batch)]
    lock = threading.Lock()
    print(f"{len(todo)} conversations in {len(batches)} batches", flush=True)

    def work(job):
        gen, batch = job
        system = COMMON + "\n\n" + personas[gen]
        try:
            reply = llm.ask(prompt_for(batch), system, f"phrase:{gen}")
            data = llm.json_block(reply)
        except Exception as e:
            print(f"batch failed ({gen}): {e}", flush=True)
            return 0
        by_id = {d.get("conv_id"): d for d in data or [] if isinstance(d, dict)}
        n = 0
        with lock, open(a.out, "a") as f:
            for plan in batch:
                d = by_id.get(plan["conv_id"])
                if not d or len(d.get("turns", [])) != len(plan["turns"]):
                    continue
                out = dict(plan)
                out["turns"] = [dict(t, **{k: v for k, v in pt.items() if k in ("text", "new_name", "nickname")})
                                for t, pt in zip(plan["turns"], d["turns"])]
                f.write(json.dumps(out) + "\n")
                n += 1
        return n

    with ThreadPoolExecutor(a.workers) as ex:
        total = 0
        for i, n in enumerate(ex.map(work, batches)):
            total += n
            print(f"batch {i + 1}/{len(batches)}: {total} conversations phrased", flush=True)


if __name__ == "__main__":
    main()
