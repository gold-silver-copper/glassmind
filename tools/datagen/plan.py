"""Plans conversations backwards from tasks, so every turn's label is exact.

Reads oracle tasks, and writes conversation plans: which turns, what each
turn must ask for (in plain terms, for a phrasing model), and the exact
expected response of each turn. Multi-turn conversations add follow-ups
about the same item (asked with "it" or "that"), corrections, context
requests, a "why" after an edit, and nicknames the user defines.

    python3 tools/datagen/plan.py --tasks T.jsonl --split train \
        --generators terse,chatty,newcomer --source tds=PATH ... --out plans.jsonl
"""

import argparse
import hashlib
import json
import os
import random
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

ASK = {
    "callers": "Ask which functions call `{name}`.",
    "callees": "Ask which functions `{name}` calls.",
    "count_callers": "Ask how many functions call `{name}`.",
    "locate": "Ask where `{name}` is defined.",
    "describe": "Ask what `{name}` does or what it is for.",
    "fields_of": "Ask what fields the struct `{name}` has.",
    "structs_with_field_type": "Ask which structs have a field whose type is (or contains) `{type}`.",
    "structs_with_field_named": "Ask which structs have a field named `{field}`.",
    "rename_field": "Ask to rename the field `{old}` of the struct `{struct}` to a better name that you choose (snake_case, not already obvious from the old one).",
    "rename_function": "Ask to rename the function `{old}` to a better name that you choose (snake_case).",
    "fields_of_function": "Ask what fields `{name}` has, as if you believed it was a struct (it is really a function).",
    "ambiguous_rename_field": "Ask to rename the field `{old}` to a better name that you choose, WITHOUT saying which struct it belongs to.",
}

REFER = {
    "callers": "Ask which functions call it.",
    "callees": "Ask which functions it calls.",
    "count_callers": "Ask how many functions call it.",
    "locate": "Ask where it is defined.",
    "describe": "Ask what it does.",
    "fields_of": "Ask what fields it has.",
}


def instruction(task):
    kind = task["kind"]
    args = task["args"]
    if kind == "no_such_name":
        q = args.get("question", "locate")
        return ASK[q].format(**{"name": args["name"]})
    return ASK[kind].format(**args)


def stable(*parts):
    return int(hashlib.sha256("/".join(map(str, parts)).encode()).hexdigest()[:12], 16)


def oracle_answer(sources, requests):
    """Computes full tasks for `requests` with the oracle; returns {id: task or None}."""
    if not requests:
        return {}
    work = Path(os.environ.get("GLASSMIND_WORK", REPO / "data" / "work"))
    work.mkdir(parents=True, exist_ok=True)
    rin = work / f"requests-{stable(*[r['id'] for r in requests])}.jsonl"
    rout = rin.with_suffix(".out.jsonl")
    rin.write_text("".join(json.dumps(r) + "\n" for r in requests))
    cmd = ["cargo", "run", "-q", "--release", "-j", "8", "-p", "glassmind-oracle", "--", "answer"]
    for s in sources:
        cmd += ["--source", s]
    cmd += ["--in", str(rin), "--out", str(rout)]
    subprocess.run(cmd, check=True, cwd=REPO)
    out = {}
    for line in rout.read_text().splitlines():
        t = json.loads(line)
        out[t["id"]] = None if "error" in t else t
    return out


def turn(conv_id, i, move, kind, text, expected, task=None, extra=None):
    t = {"turn_id": f"{conv_id}#{i}", "move": move, "kind": kind, "instruction": text,
         "expected": expected, "task_id": task["id"] if task else None}
    if task:
        t["info"] = task.get("info", {})
        t["args"] = task["args"]
    if extra:
        t.update(extra)
    return t


def is_rename(kind):
    return kind in ("rename_field", "rename_function", "ambiguous_rename_field")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tasks", required=True)
    ap.add_argument("--split", required=True)
    ap.add_argument("--generators", required=True)
    ap.add_argument("--source", action="append", default=[])
    ap.add_argument("--out", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--multi", type=float, default=0.35, help="multi-turn conversations per single-turn one")
    a = ap.parse_args()
    rng = random.Random(a.seed)
    generators = a.generators.split(",")
    tasks = [json.loads(l) for l in open(a.tasks) if l.strip()]
    tasks.sort(key=lambda t: t["id"])
    plans = []

    def gen_for(key):
        return generators[stable(a.seed, key) % len(generators)]

    # Single-turn conversations: one per task.
    for t in tasks:
        cid = f"{a.split}/{t['id']}"
        plans.append({"conv_id": cid, "split": a.split, "generator": gen_for(cid), "source": t["source"],
                      "turns": [turn(cid, 0, "task", t["kind"], instruction(t), t.get("expected"), t)]})

    by_source = {}
    for t in tasks:
        by_source.setdefault(t["source"], []).append(t)
    n_multi = int(len(tasks) * a.multi)
    requests = []
    drafts = []
    for k in range(n_multi):
        source = rng.choice(sorted(by_source))
        pool = by_source[source]
        cid = f"{a.split}/multi/{source}/{k}"
        shape = rng.choice(["follow", "follow", "follow", "correct", "context", "forget", "why", "alias"])
        if shape == "follow":
            first = rng.choice([t for t in pool if t["kind"] in ("callers", "callees", "locate", "describe", "count_callers")])
            name = first["args"]["name"]
            kinds = [k2 for k2 in ("locate", "describe", "callers", "callees", "count_callers") if k2 != first["kind"]]
            follow = rng.sample(kinds, rng.choice([1, 1, 2]))
            reqs = [{"id": f"{cid}#{i + 1}", "source": source, "kind": k2, "args": {"name": name}} for i, k2 in enumerate(follow)]
            requests += reqs
            drafts.append((shape, cid, source, first, reqs))
        elif shape == "correct":
            locs = [t for t in pool if t["kind"] in ("locate", "describe", "callers")]
            first = rng.choice(locs)
            other = rng.choice([t for t in locs if t["kind"] == first["kind"] and t["id"] != first["id"]] or [first])
            drafts.append((shape, cid, source, first, [other]))
        elif shape in ("context", "forget"):
            first = rng.choice([t for t in pool if not is_rename(t["kind"])])
            drafts.append((shape, cid, source, first, []))
        elif shape == "why":
            renames = [t for t in pool if t["kind"] in ("rename_field", "rename_function")]
            if renames:
                drafts.append((shape, cid, source, rng.choice(renames), []))
        elif shape == "alias":
            structs = [t for t in pool if t["kind"] == "fields_of"]
            if structs:
                drafts.append((shape, cid, source, rng.choice(structs), []))

    answered = oracle_answer(a.source, requests)
    for shape, cid, source, first, rest in drafts:
        turns = [turn(cid, 0, "task", first["kind"], instruction(first), first.get("expected"), first)]
        if shape == "follow":
            ok = [answered.get(r["id"]) for r in rest]
            if not all(ok):
                continue
            for i, t in enumerate(ok, start=1):
                turns.append(turn(cid, i, "refer", t["kind"],
                                  REFER[t["kind"]] + " Refer to the item from before as 'it', 'that' or 'that function'/'that struct'; do NOT write its name.",
                                  t["expected"], t))
        elif shape == "correct":
            other = rest[0]
            if other["id"] == first["id"]:
                continue
            name = other["args"]["name"]
            turns.append(turn(cid, 1, "correct", other["kind"],
                              f"Correct yourself: say you actually meant `{name}` instead of the name you used before (for example 'sorry, I meant `{name}`'). Do not repeat the whole question.",
                              other["expected"], other))
        elif shape == "context":
            turns.append(turn(cid, 1, "show_context", "meta",
                              "Ask the assistant to show you what it is keeping in its context / what it remembers of this conversation.",
                              {"type": "meta", "act": "show_context"}))
        elif shape == "forget":
            turns.append(turn(cid, 1, "forget", "meta", "Tell the assistant to forget that (the last thing).",
                              {"type": "meta", "act": "forget"}))
        elif shape == "why":
            turns.append(turn(cid, 1, "why", "meta", "Ask why it wants to change that (the change it just proposed).",
                              {"type": "meta", "act": "explain_action", "turn": 0}))
        elif shape == "alias":
            s = first["args"]["name"]
            turns = [turn(cid, 0, "alias_fact", "meta",
                          f"Tell the assistant you will call the struct `{s}` by a short nickname of one or two plain lowercase English words that you choose (for example 'the hero'). Report the nickname.",
                          {"type": "meta", "act": "noted"}, first, {"alias_of": s}),
                     turn(cid, 1, "alias_use", "fields_of",
                          "Ask what fields it has, naming it ONLY by the nickname you defined (not its real name).",
                          first["expected"], first)]
        plans.append({"conv_id": cid, "split": a.split, "generator": rng.choice(generators), "source": source,
                      "turns": turns})

    with open(a.out, "w") as f:
        for p in plans:
            f.write(json.dumps(p) + "\n")
    print(f"{len(plans)} conversations, {sum(len(p['turns']) for p in plans)} turns -> {a.out}")


if __name__ == "__main__":
    main()
