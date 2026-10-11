#!/usr/bin/env python3
"""Training-only bAbI preparation and scoring. Python never reasons about answers.

Split complete episodes before expanding questions, so overlapping contexts never
cross train/dev. Only English 1k training members may be opened from the archive.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
URL = "http://www.thespermwhale.com/jaseweston/babi/tasks_1-20_v1-2.tar.gz"
MIRROR = "https://s3.amazonaws.com/text-datasets/babi_tasks_1-20_v1-2.tar.gz"
ARCHIVE_SHA256 = "84f5296ab9a1ad0dc9464e08c491d65cd08830fca3acae9ab86f75e0fb81573c"
SEED = "glassmind-stories-v1"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def prepare(args):
    archive = args.archive
    existing = archive.exists()
    if not archive.exists():
        archive.parent.mkdir(parents=True, exist_ok=True)
        urllib.request.urlretrieve(args.url, archive)
    archive_digest = digest(archive.read_bytes())
    if archive_digest != ARCHIVE_SHA256:
        raise ValueError("archive checksum differs from the pinned bAbI v1.2 source")
    records = []
    out = args.output
    out.mkdir(parents=True, exist_ok=True)
    # Never extractall, never open a held-out member, even for validation.
    with tarfile.open(archive, "r:gz") as tar:
        for member in tar:
            path = PurePosixPath(member.name)
            if not (member.isfile() and len(path.parts) == 3
                    and path.parts[0] == "tasks_1-20_v1-2"
                    and path.parts[1] == "en"
                    and re.fullmatch(r"qa\d+_[a-z0-9-]+_train\.txt", path.name)):
                continue
            stream = tar.extractfile(member)
            if stream is None:
                raise ValueError("missing training member")
            data = stream.read()
            (out / path.name).write_bytes(data)
            records.append({"member": member.name, "bytes": len(data), "sha256": digest(data)})
    if len(records) != 20:
        raise ValueError(f"expected 20 training members, got {len(records)}")
    provenance = {
        "dataset": "Facebook bAbI tasks 1-20 v1.2, English 1k",
        "license": "CC-BY-3.0", "copyright": "2015-present Facebook, Inc.",
        "license_source": "https://huggingface.co/datasets/facebook/babi_qa",
        "official_url": URL, "download_url": args.url,
        "archive_bytes": archive.stat().st_size,
        "archive_sha256": archive_digest,
        "training_bytes": sum(r["bytes"] for r in records),
        "files": sorted(records, key=lambda r: r["member"]),
        "split": "SHA256(seed/task/episode_index) rank; lowest floor(episodes/5) dev; all other episodes train",
        "seed": SEED,
        "existing_archive": existing,
        "download_note": "Official URL returned HTTP 404 on 2026-10-10; public S3 mirror byte identity verified by SHA256",
    }
    args.provenance.write_text(json.dumps(provenance, indent=2) + "\n")
    print(json.dumps({k: v for k, v in provenance.items() if k != "files"}, indent=2))


def episodes(path):
    episode = []
    for line in path.read_text().splitlines():
        number, text = line.split(" ", 1)
        if number == "1" and episode:
            yield episode
            episode = []
        episode.append(text)
    if episode:
        yield episode


def items(directory, split, sample):
    files = sorted(directory.glob("qa*_train.txt"), key=lambda p: int(p.name.split("_")[0][2:]))
    if len(files) != 20:
        raise ValueError("expected all 20 English training files")
    for path in files:
        task = int(path.name.split("_")[0][2:])
        stories = list(episodes(path))
        ranked = sorted(range(len(stories)), key=lambda i: digest(f"{SEED}/{task}/{i}".encode()))
        dev = set(ranked[:len(stories) // 5])
        selected = []
        for index, story in enumerate(stories):
            if (index in dev) != (split == "dev"):
                continue
            context = []
            for turn, text in enumerate(story):
                if "\t" in text:
                    question, answer, *_ = text.split("\t")
                    selected.append({"id": f"{task}/{index}/{turn}", "task": task,
                                     "story": context.copy(), "question": question.strip(),
                                     "expected": answer.strip()})
                else:
                    context.append(text)
        selected.sort(key=lambda row: digest(f"{SEED}/sample/{row['id']}".encode()))
        yield from selected[:sample] if sample else selected


def train(args):
    payload = []
    counts = {}
    for path in sorted(args.data.glob("qa*_train.txt")):
        task = int(path.name.split("_")[0][2:])
        stories = list(episodes(path))
        ranked = sorted(range(len(stories)), key=lambda i: digest(f"{SEED}/{task}/{i}".encode()))
        dev = set(ranked[:len(stories) // 5])
        counts[task] = {"train_episodes": len(stories)-len(dev), "dev_episodes": len(dev)}
        for i, story in enumerate(stories):
            if i not in dev:
                payload.append(json.dumps({"story": [text for text in story if "\t" not in text]}) + "\n")
    if len(counts) != 20:
        raise ValueError("expected all 20 training tasks")
    subprocess.run([str(args.binary.resolve()), "--train", str(args.model)],
                   input="".join(payload), text=True, check=True, timeout=550)
    provenance = {"seed": SEED, "split": "train", "questions_or_answers_used": False,
                  "training_stories": len(payload), "per_task": counts,
                  "training_input_sha256": digest("".join(payload).encode()),
                  "model_sha256": digest(args.model.read_bytes()),
                  "binary_sha256": digest(args.binary.read_bytes())}
    args.model.with_suffix(".provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    print(json.dumps(counts, indent=2))


def normalize(text, task):
    value = text.strip().lower()
    # Lists are sets; path sequences are ordered and must remain so.
    return ",".join(sorted(value.split(","))) if task == 8 else value


def evaluate(args):
    rows = list(items(args.data, args.split, args.sample))
    # Answer keys, supporting fact IDs, and task IDs never reach the Rust process.
    payload = "".join(json.dumps({"story": row["story"], "question": row["question"]}) + "\n" for row in rows)
    command = [str(args.binary.resolve()), "--jsonl"]
    if args.model:
        command += ["--model", str(args.model)]
    process = subprocess.run(command, input=payload,
                             text=True, capture_output=True, timeout=550, check=True)
    replies = [json.loads(line) for line in process.stdout.splitlines()]
    if len(replies) != len(rows):
        raise ValueError("one reply required per question")
    totals = Counter()
    tasks = defaultdict(Counter)
    outcomes = []
    for row, reply in zip(rows, replies, strict=True):
        answer = reply.get("answer")
        status = ("refusal" if answer is None else "correct" if
                  normalize(answer, row["task"]) == normalize(row["expected"], row["task"]) else "wrong")
        totals[status] += 1
        tasks[row["task"]][status] += 1
        outcomes.append({**row, **reply, "status": status})
    report = {"split": args.split, "seed": SEED, "sample_per_task": args.sample,
              "model_sha256": digest(args.model.read_bytes()) if args.model else None,
              "binary_sha256": digest(args.binary.read_bytes()),
              "item_ids_sha256": digest("\n".join(row["id"] for row in rows).encode()),
              "total": len(rows), "totals": {s: totals[s] for s in ("correct", "refusal", "wrong")},
              "per_task": {k: {s: v[s] for s in ("correct", "refusal", "wrong")} for k, v in tasks.items()}}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    if args.outcomes:
        args.outcomes.write_text("".join(json.dumps(row) + "\n" for row in outcomes))
    print(json.dumps(report, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    prep = commands.add_parser("prepare")
    prep.add_argument("--archive", type=Path, default=ROOT / "data/babi/babi.tgz")
    prep.add_argument("--url", default=MIRROR)
    prep.add_argument("--output", type=Path, default=ROOT / "data/babi/tasks_1-20_v1-2/en")
    prep.add_argument("--provenance", type=Path, default=ROOT / "tools/stories/source.json")
    prep.set_defaults(run=prepare)
    ev = commands.add_parser("eval")
    ev.add_argument("--data", type=Path, default=ROOT / "data/babi/tasks_1-20_v1-2/en")
    ev.add_argument("--binary", type=Path, default=ROOT / ".cache/stories-target/debug/glassmind-stories")
    ev.add_argument("--split", choices=("train", "dev"), default="dev")
    ev.add_argument("--sample", type=int, default=20, help="questions per task; 0 means complete split")
    ev.add_argument("--report", type=Path, required=True)
    ev.add_argument("--outcomes", type=Path)
    ev.add_argument("--model", type=Path)
    ev.set_defaults(run=evaluate)
    learning = commands.add_parser("train", help="learn state/action counts in Rust from training assertions only")
    learning.add_argument("--data", type=Path, default=ROOT / "data/babi/tasks_1-20_v1-2/en")
    learning.add_argument("--binary", type=Path, default=ROOT / ".cache/stories-target/debug/glassmind-stories")
    learning.add_argument("--model", type=Path, required=True)
    learning.set_defaults(run=train)
    args = parser.parse_args()
    args.run(args)


if __name__ == "__main__":
    main()
