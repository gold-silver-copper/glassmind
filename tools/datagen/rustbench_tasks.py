"""Training turns from rustbench's dev tasks (github.com/gold-silver-copper/rustbench).

Every dev task whose outcome cargo can judge (fix-error, fix-test, edit,
implement; not the ones that also check printed output) becomes one
conversation per prompt style. Its workspace becomes a source; its hidden
tests go under SOURCES/.rustbench-hidden/<source>, which the scorer copies
over a remedy before running the tests, as rustbench does. Held-out tasks
are never read: rustbench keeps them for measurement only.

    python3 tools/datagen/rustbench_tasks.py --rustbench DIR --sources DIR --out DIR
"""

import argparse
import json
import shutil
import tomllib
from pathlib import Path

CHECK = {"fix-error": "build", "fix-test": "test", "edit": "test", "implement": "test"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rustbench", required=True)
    ap.add_argument("--sources", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    sources, out = Path(a.sources).expanduser(), Path(a.out).expanduser()
    out.mkdir(parents=True, exist_ok=True)
    inputs, labels = [], []
    for toml in sorted(Path(a.rustbench).expanduser().glob("tasks/*/*/task.toml")):
        text = toml.read_text()
        if 'split = "dev"' not in text:
            continue
        task = tomllib.loads(text)
        check = CHECK.get(task["kind"])
        if task.get("split") != "dev" or not check or "stdout" in task.get("check", {}):
            continue
        name = f"rb-{task['suite']}-{task['id']}"
        dst = sources / name
        if dst.exists():
            shutil.rmtree(dst)
        shutil.copytree(toml.parent / "workspace", dst, ignore=shutil.ignore_patterns("target"))
        hidden = Path(".rustbench-hidden") / name
        if (sources / hidden).exists():
            shutil.rmtree(sources / hidden)
        if (toml.parent / "hidden").exists():
            shutil.copytree(toml.parent / "hidden", sources / hidden)
        else:
            (sources / hidden).mkdir(parents=True)
        kind = "rb_" + task["kind"].replace("-", "_")
        for style, prompt in sorted(task["prompt"].items()):
            conv = f"train/{name}/{style}"
            inputs.append({"id": conv, "sources": [name], "turns": [{"id": f"{conv}#0", "text": prompt}]})
            labels.append({"turn_id": f"{conv}#0", "conversation": conv, "split": "train",
                           "generator": f"rustbench-{style}", "source": name, "kind": kind,
                           "expected": {"type": "repaired", "check": check}, "hidden": str(hidden)})
    for fname, rows in (("inputs.jsonl", inputs), ("labels.jsonl", labels)):
        with open(out / fname, "w") as fo:
            for r in rows:
                fo.write(json.dumps(r) + "\n")
    print(f"{len(inputs)} conversations from {len({l['source'] for l in labels})} tasks -> {out}")


if __name__ == "__main__":
    main()
