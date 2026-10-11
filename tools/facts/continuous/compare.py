#!/usr/bin/env python3
"""Offline train/dev diagnostics, separate from the unchanged score command.

The Rust example consumes questions only. This report joins its outputs to
annotations after inference, checks transitions, and classifies error symptoms.
"""
import argparse
import collections
import hashlib
import json
from pathlib import Path


def status(row, prediction):
    if prediction["answers"] == [row[2]]:
        return "right"
    return "refused" if prediction["refusal"] else "wrong"


def symptom(row, prediction):
    if prediction["entity"] != row[0]:
        return "entity_mismatch"
    if prediction["relation"] != row[1]:
        return "relation_mismatch"
    if row[1].startswith("R"):
        return "inverse_answer_mismatch"
    return "snapshot_answer_mismatch"


def read_predictions(path, count):
    predictions = [json.loads(line) for line in path.read_text().splitlines()]
    if len(predictions) != count:
        raise ValueError(f"{path}: expected {count} predictions, got {len(predictions)}")
    return predictions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--split", choices=["train", "valid"], required=True)
    parser.add_argument("--before", type=Path, required=True)
    parser.add_argument("--after", type=Path, required=True)
    args = parser.parse_args()
    source = args.data / f"{args.split}.tsv"
    rows = [line.split("\t", 3) for line in source.read_text().splitlines()]
    before = read_predictions(args.before, len(rows))
    after = read_predictions(args.after, len(rows))
    transitions = collections.Counter()
    counts = {key: collections.Counter() for key in ("before", "after")}
    symptoms = {key: collections.Counter() for key in ("before", "after")}
    mismatched_right = {key: [] for key in ("before", "after")}
    changed = []
    for number, (row, old, new) in enumerate(zip(rows, before, after), 1):
        old_status, new_status = status(row, old), status(row, new)
        transitions[f"{old_status}->{new_status}"] += 1
        for key, prediction in (("before", old), ("after", new)):
            result = status(row, prediction)
            counts[key][result] += 1
            if result == "wrong":
                symptoms[key][symptom(row, prediction)] += 1
            if result == "right" and prediction["entity"] != row[0]:
                mismatched_right[key].append(number)
        if old_status != new_status:
            changed.append({"row": number, "before": old_status, "after": new_status,
                            "before_symptom": symptom(row, old)})
    files = {name: {"sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                    "bytes": path.stat().st_size}
             for name, path in (("annotations", source), ("before", args.before),
                                ("after", args.after))}
    print(json.dumps({"split": args.split, "total": len(rows), "counts": counts,
                      "transitions": transitions, "wrong_symptoms": symptoms,
                      "scored_right_with_entity_mismatch_rows": mismatched_right,
                      "status_changes": changed, "files": files}, indent=2))


if __name__ == "__main__":
    main()
