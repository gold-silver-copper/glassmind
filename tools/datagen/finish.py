"""Checks phrased conversations and writes the eval inputs and labels.

1. Tooling checks: a turn that must name its item names it, a turn that
   must refer back does not; chosen rename targets are checked as fresh by
   the oracle, which fills in the exact changes.
2. An LLM check (Claude Opus 5.5, a separate checking prompt): does each
   turn ask for exactly what its plan says? Conversations with a failing
   turn are dropped.
3. Writes `inputs.jsonl` (what glassmind sees) and `labels.jsonl` (what the
   scorer compares with).

    python3 tools/datagen/finish.py --phrased P.jsonl --source tds=PATH ... --out-dir DIR [--no-llm-check]
"""

import argparse
import json
import re
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import llm
from plan import oracle_answer

CHECK = """You check data for a code-question tool. Each item has a PLAN (what the user's message must ask for) and the user's TEXT.
Judge whether TEXT asks for exactly what PLAN says, about exactly the named code item(s), so that a correct answer to TEXT is the correct answer to PLAN.
Typos, slang, terseness and indirectness are fine. Fail it if it asks for something different or additional, names a different item, or is a statement where a request is required.
Reply with only a JSON array of objects {"id": "...", "ok": true|false, "why": "short reason if not ok"}."""


def names_in(turn):
    args = turn.get("args") or {}
    out = []
    for k in ("name", "old", "type", "field", "struct"):
        if isinstance(args.get(k), str):
            out.append(args[k])
    return out


def tooling_ok(conv):
    for t in conv["turns"]:
        text = t.get("text") or ""
        low = text.lower()
        move = t["move"]
        names = names_in(t)
        if not text.strip():
            return False, "empty"
        if move in ("task", "correct") and t["kind"] != "meta":
            key = names[:1] if move == "correct" else names
            if not all(re.search(r"(?<![A-Za-z0-9_])" + re.escape(n.lower()) + r"(?![A-Za-z0-9_])", low) for n in key):
                return False, f"turn {t['turn_id']} does not name {key}"
        if move in ("refer", "alias_use") and any(n.lower() in low for n in names):
            return False, f"turn {t['turn_id']} names the item it should refer to"
        if move == "alias_fact" and not t.get("nickname"):
            return False, "no nickname"
    return True, ""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--phrased", required=True)
    ap.add_argument("--source", action="append", default=[])
    ap.add_argument("--out-dir", required=True)
    ap.add_argument("--no-llm-check", action="store_true")
    a = ap.parse_args()
    out_dir = Path(a.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    convs = [json.loads(l) for l in open(a.phrased) if l.strip()]
    dropped = {}

    # Rename targets: let the oracle check the chosen name and fill the changes.
    requests = []
    for c in convs:
        for t in c["turns"]:
            if t["kind"] in ("rename_field", "rename_function"):
                new = (t.get("new_name") or "").strip().strip("`")
                args = dict(t["args"], new=new)
                requests.append({"id": t["turn_id"], "source": c["source"], "kind": t["kind"], "args": args})
    answered = oracle_answer(a.source, requests)
    kept = []
    for c in convs:
        ok, why = tooling_ok(c)
        for t in c["turns"]:
            if t["kind"] in ("rename_field", "rename_function"):
                task = answered.get(t["turn_id"])
                if task is None or not t.get("new_name") or t["new_name"].strip("`") not in t["text"]:
                    ok, why = False, "rename target missing, taken or not in text"
                else:
                    t["expected"] = task["expected"]
        if ok:
            kept.append(c)
        else:
            dropped[c["conv_id"]] = why

    # LLM check, in batches of turns.
    if not a.no_llm_check:
        items = [{"id": t["turn_id"], "plan": t["instruction"] + (f" (chosen name: {t.get('new_name') or t.get('nickname')})" if t.get("new_name") or t.get("nickname") else ""),
                  "text": t["text"], "earlier": [u["text"] for u in c["turns"][:i]]}
                 for c in kept for i, t in enumerate(c["turns"])]
        batches = [items[i:i + 40] for i in range(0, len(items), 40)]

        def check(batch):
            try:
                return llm.json_block(llm.ask(json.dumps(batch, indent=1), CHECK, "check")) or []
            except Exception as e:
                print(f"check batch failed: {e}", flush=True)
                return [{"id": b["id"], "ok": False, "why": "check failed"} for b in batch]

        verdicts = {}
        with ThreadPoolExecutor(4) as ex:
            for result in ex.map(check, batches):
                for v in result:
                    if isinstance(v, dict) and "id" in v:
                        verdicts[v["id"]] = v
        still = []
        for c in kept:
            bad = [verdicts.get(t["turn_id"], {"ok": False, "why": "unchecked"}) for t in c["turns"]]
            bad = [v for v in bad if not v.get("ok")]
            if bad:
                dropped[c["conv_id"]] = "llm check: " + "; ".join(str(v.get("why")) for v in bad)
            else:
                still.append(c)
        kept = still

    with open(out_dir / "inputs.jsonl", "w") as fi, open(out_dir / "labels.jsonl", "w") as fl, open(out_dir / "conversations.jsonl", "w") as fc:
        for c in kept:
            fi.write(json.dumps({"id": c["conv_id"], "sources": [c["source"]],
                                 "turns": [{"id": t["turn_id"], "text": t["text"]} for t in c["turns"]]}) + "\n")
            for t in c["turns"]:
                fl.write(json.dumps({"turn_id": t["turn_id"], "conversation": c["conv_id"], "split": c["split"],
                                     "generator": c["generator"], "source": c["source"],
                                     "kind": t["kind"] if t["move"] == "task" else f"{t['move']}:{t['kind']}",
                                     "expected": t["expected"]}) + "\n")
            fc.write(json.dumps(c) + "\n")
    (out_dir / "dropped.json").write_text(json.dumps(dropped, indent=1))
    print(f"kept {len(kept)} conversations ({sum(len(c['turns']) for c in kept)} turns), dropped {len(dropped)}")


if __name__ == "__main__":
    main()
