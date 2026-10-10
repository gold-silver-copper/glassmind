"""Coding tasks for the harness, from real crates at fixed versions.

Each task is a copy of a crate with one thing broken, and an exact
expected outcome computed by cargo, never by glassmind:

- build_status: one use of a name mistyped, a `mut` or `&mut` dropped, or a
  `use` line deleted; `cargo check` fails. Expected: the problems.
- fix_build: the same breakage; expected: any change that makes
  `cargo check --all-targets` pass ("repaired").
- test_status: an operator flipped or an off-by-one in non-test code; it
  still builds, some test fails. Expected: the failing tests.
- fix_tests: the same; expected: any change that makes `cargo test` pass.

    python3 tools/datagen/harness_tasks.py --crates DIR --out DIR --per-crate N [--seed S]

Writes OUT/sources/<task source>/ (the broken copies) and OUT/tasks.jsonl.
"""

import argparse
import hashlib
import json
import random
import re
import shutil
import subprocess
from pathlib import Path

TARGET = Path.home() / ".cache" / "glassmind" / "harness-target"


def cargo(root, *args):
    env = dict(__import__("os").environ, CARGO_TARGET_DIR=str(TARGET / root.name))
    p = subprocess.run(["cargo", *args[:1], "-j", "4", *args[1:]], cwd=root, capture_output=True, text=True,
                       env=env, timeout=900)
    return p.returncode == 0, p.stdout


def build_problems(root, source):
    ok, out = cargo(root, "check", "--all-targets", "--message-format=json", "-q")
    seen, items = set(), []
    for line in out.splitlines():
        try:
            m = json.loads(line)
        except json.JSONDecodeError:
            continue
        if m.get("reason") != "compiler-message" or m["message"]["level"] != "error":
            continue
        msg = m["message"]
        prim = [s for s in msg["spans"] if s["is_primary"]]
        if not prim:
            continue
        code = (msg.get("code") or {}).get("code") or "error"
        key = (prim[0]["file_name"], prim[0]["line_start"], code)
        if key in seen:
            continue
        seen.add(key)
        items.append({"source": source, "file": prim[0]["file_name"], "line": prim[0]["line_start"],
                      "name": code, "kind": "problem", "message": msg["message"]})
    return ok, items


def failing_tests(root, source):
    ok, out = cargo(root, "test", "--no-fail-fast", "--", "--test-threads", "4")
    names = sorted({l[len("test "):-len(" ... FAILED")] for l in out.splitlines() if l.startswith("test ") and l.endswith(" ... FAILED")})
    return ok, [{"source": source, "file": "", "line": 0, "name": n, "kind": "test"} for n in names]


def rs_files(root, tests):
    """Library source files; with `tests`, test files too."""
    out = []
    for p in sorted(root.rglob("*.rs")):
        rel = p.relative_to(root).as_posix()
        if rel.startswith("target/") or rel.startswith("benches/") or rel.startswith("examples/"):
            continue
        if rel.startswith("tests/") and not tests:
            continue
        out.append(p)
    return out


def body_lines(text):
    """Line numbers outside `#[cfg(test)]` modules (a crude cut: from the
    attribute to the end of the file is test code)."""
    lines = text.split("\n")
    cut = next((i for i, l in enumerate(lines) if "#[cfg(test)]" in l), len(lines))
    return range(cut)


# --- mutations: each returns (new_text, description) or None -------------

def typo_use(text, rng):
    """Mistype one later use of a `let` binding (drop one letter)."""
    lines = text.split("\n")
    allowed = set(body_lines(text))
    cands = []
    for i, l in enumerate(lines):
        m = re.search(r"\blet (?:mut )?([a-z_][a-z0-9_]{3,})\b", l)
        if not m or i not in allowed:
            continue
        name = m.group(1)
        for j in range(i + 1, min(i + 25, len(lines))):
            if j not in allowed or lines[j].lstrip().startswith("//"):
                continue
            for u in re.finditer(r"\b%s\b" % re.escape(name), lines[j]):
                cands.append((j, u.start(), name))
    if not cands:
        return None
    j, col, name = rng.choice(cands)
    k = rng.randrange(1, len(name) - 1)
    typo = name[:k] + name[k + 1:]
    lines[j] = lines[j][:col] + typo + lines[j][col + len(name):]
    return "\n".join(lines), f"line {j + 1}: `{name}` mistyped as `{typo}`"


def drop_mut(text, rng):
    lines = text.split("\n")
    allowed = set(body_lines(text))
    cands = [i for i, l in enumerate(lines) if "let mut " in l and i in allowed]
    if not cands:
        return None
    i = rng.choice(cands)
    lines[i] = lines[i].replace("let mut ", "let ", 1)
    return "\n".join(lines), f"line {i + 1}: `mut` dropped"


def drop_ref_mut(text, rng):
    lines = text.split("\n")
    allowed = set(body_lines(text))
    cands = [i for i, l in enumerate(lines) if re.search(r"fn [a-z_0-9]+[^)]*: &mut ", l) and i in allowed]
    if not cands:
        return None
    i = rng.choice(cands)
    lines[i] = re.sub(r": &mut ", ": &", lines[i], count=1)
    return "\n".join(lines), f"line {i + 1}: a `&mut` parameter made `&`"


def drop_use(text, rng):
    lines = text.split("\n")
    allowed = set(body_lines(text))
    cands = [i for i, l in enumerate(lines) if re.match(r"\s*use [^;{]+;\s*$", l) and i in allowed]
    if not cands:
        return None
    i = rng.choice(cands)
    old = lines[i].strip()
    del lines[i]
    return "\n".join(lines), f"line {i + 1}: `{old}` deleted"


FLIPS = [(" + ", " - "), (" - ", " + "), (" < ", " <= "), (" <= ", " < "), (" > ", " >= "), (" >= ", " > "),
         (" == ", " != "), (" != ", " == "), ("true", "false"), ("false", "true"), (" && ", " || "),
         (" || ", " && ")]


def flip_logic(text, rng):
    lines = text.split("\n")
    allowed = set(body_lines(text))
    cands = []
    for i, l in enumerate(lines):
        s = l.strip()
        if i not in allowed or s.startswith("//") or s.startswith("#") or "assert" in s:
            continue
        for a, b in FLIPS:
            for m in re.finditer(re.escape(a), l):
                cands.append((i, m.start(), a, b))
    if not cands:
        return None
    i, col, a, b = rng.choice(cands)
    lines[i] = lines[i][:col] + b + lines[i][col + len(a):]
    return "\n".join(lines), f"line {i + 1}: `{a.strip()}` changed to `{b.strip()}`"


BUILD_BREAKERS = [typo_use, drop_mut, drop_ref_mut, drop_use]


def copy_crate(src, dst):
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst, ignore=shutil.ignore_patterns("target", ".git", ".cargo_vcs_info.json"))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--crates", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--per-crate", type=int, default=6)
    ap.add_argument("--seed", type=int, default=0)
    a = ap.parse_args()
    crates = sorted(p for p in Path(a.crates).expanduser().iterdir() if (p / "Cargo.toml").exists())
    out = Path(a.out).expanduser()
    (out / "sources").mkdir(parents=True, exist_ok=True)
    tasks = []
    for crate in crates:
        rng = random.Random(f"{a.seed}/{crate.name}")
        files = rs_files(crate, tests=False)
        made = {"build": 0, "test": 0}
        tries = 0
        while (made["build"] < a.per_crate or made["test"] < a.per_crate) and tries < a.per_crate * 12:
            tries += 1
            want = "build" if made["build"] < a.per_crate else "test"
            f = rng.choice(files)
            text = f.read_text()
            mut = rng.choice(BUILD_BREAKERS) if want == "build" else flip_logic
            r = mut(text, rng)
            if not r:
                continue
            new_text, what = r
            tag = hashlib.sha256(f"{crate.name}{f}{what}".encode()).hexdigest()[:8]
            name = f"{crate.name}-{want}-{tag}"
            dst = out / "sources" / name
            copy_crate(crate, dst)
            rel = f.relative_to(crate).as_posix()
            (dst / rel).write_text(new_text)
            ok, problems = build_problems(dst, name)
            if want == "build":
                if ok or not problems:
                    shutil.rmtree(dst)
                    continue
                info = {"crate": crate.name, "file": rel, "broken": what, "problems": [p["name"] + " " + p["message"] for p in problems]}
                keys = [{k: p[k] for k in ("source", "file", "line", "name", "kind")} for p in problems]
                tasks.append({"id": f"{name}/build_status", "source": name, "kind": "build_status", "args": {},
                              "expected": {"type": "checked", "check": "build", "passed": False, "problems": keys},
                              "info": info})
                tasks.append({"id": f"{name}/fix_build", "source": name, "kind": "fix_build", "args": {},
                              "expected": {"type": "repaired", "check": "build"}, "info": info})
            else:
                if not ok:
                    shutil.rmtree(dst)
                    continue
                tok, failing = failing_tests(dst, name)
                if tok or not failing:
                    shutil.rmtree(dst)
                    continue
                info = {"crate": crate.name, "file": rel, "broken": what, "failing": [t["name"] for t in failing]}
                tasks.append({"id": f"{name}/test_status", "source": name, "kind": "test_status", "args": {},
                              "expected": {"type": "checked", "check": "test", "passed": False, "problems": failing},
                              "info": info})
                tasks.append({"id": f"{name}/fix_tests", "source": name, "kind": "fix_tests", "args": {},
                              "expected": {"type": "repaired", "check": "test"}, "info": info})
            made[want] += 1
            print(f"{name}: {what}", flush=True)
    with open(out / "tasks.jsonl", "w") as fo:
        for t in tasks:
            fo.write(json.dumps(t) + "\n")
    print(f"{len(tasks)} tasks -> {out / 'tasks.jsonl'}")


if __name__ == "__main__":
    main()
