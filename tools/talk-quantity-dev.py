#!/usr/bin/env python3
"""Run the existing quantities development suites through the main talk command.

Labels are read only by this evaluator; glassmind receives question text.
This measures routing of synthetic quantity questions, not general dialogue.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("data", nargs="+", type=Path)
    args = parser.parse_args()
    rows = [json.loads(line) for path in args.data for line in path.read_text().splitlines()]
    if not rows:
        raise SystemExit("empty development set")
    run = subprocess.run([str(args.binary.resolve()), "talk", "--json"],
                         input="\n".join(row["question"] for row in rows) + "\n",
                         capture_output=True, text=True, timeout=120, check=True)
    replies = [json.loads(line) for line in run.stdout.splitlines()]
    if len(replies) != len(rows):
        raise RuntimeError("missing conversation replies")
    counts = Counter()
    failures = []
    for row, reply in zip(rows, replies):
        answered = reply.get("kind") == "calculated"
        if row["expected"] is None:
            outcome = "right" if reply.get("kind") == "refusal" else "wrong"
            counts["expected_refusals"] += outcome == "right"
        elif not answered:
            outcome = "refused" if reply.get("kind") == "refusal" else "wrong"
        else:
            outcome = "right" if (reply.get("value") == row["expected"]
                                    and reply.get("unit") == row.get("unit")) else "wrong"
        counts[outcome] += 1
        if outcome != "right":
            failures.append(dict(question=row["question"], expected=row["expected"],
                                 actual=reply, outcome=outcome))
    report = dict(dataset="talk routing of quantities synthetic and extended dev v1",
                  independent_benchmark=False, total=len(rows),
                  right=counts["right"], refused=counts["refused"], wrong=counts["wrong"],
                  expected_refusals=counts["expected_refusals"], failures=failures)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({key: value for key, value in report.items() if key != "failures"}))
    if counts["wrong"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
