#!/usr/bin/env python3
"""Aggregate only this crate's synthetic development reports."""
import json
from pathlib import Path

root = Path(__file__).resolve().parent
reports = [json.loads((root / name).read_text()) for name in ("full-dev.json", "extended-full-dev.json")]
metrics = ("total", "right", "refusal", "wrong", "correct_refusals")
aggregate = {metric: sum(report[metric] for report in reports) for metric in metrics}
aggregate["answered_right"] = aggregate["right"] - aggregate["correct_refusals"]
aggregate["description"] = "Synthetic dev only; right includes explicitly expected refusals. No external benchmark score."
aggregate["sources"] = [report["provenance"] for report in reports]
(root / "aggregate.json").write_text(json.dumps(aggregate, indent=2) + "\n")
print(json.dumps(aggregate, indent=2))
if aggregate["wrong"]:
    raise SystemExit(1)
