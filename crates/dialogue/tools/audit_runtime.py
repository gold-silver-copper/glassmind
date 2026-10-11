#!/usr/bin/env python3
"""Integration audit: annotation-free runtime must reproduce every dev prediction.

Also verifies that the runtime rejects an annotation-bearing turn, rather than
silently accepting fields that could leak oracle state into a later extension.
"""
import argparse
import json
from pathlib import Path
import subprocess

p = argparse.ArgumentParser()
p.add_argument("--binary", required=True)
p.add_argument("--data", required=True)
p.add_argument("--schema", required=True)
p.add_argument("--model", required=True)
p.add_argument("--predictions", required=True)
p.add_argument("--scratch", type=Path, required=True)
a = p.parse_args()
a.scratch.mkdir(parents=True, exist_ok=True)
predictions = {(x["dialogue_id"], x["turn_index"]): x["state"] for x in json.load(open(a.predictions))}
runtime_file = a.scratch / "runtime-input.json"
command = [a.binary, "predict", "--schema", a.schema, "--model", a.model, "--input", str(runtime_file)]
count = 0
dialogues = json.load(open(a.data))
for dialogue in dialogues:
    inputs = [{"speaker": t["speaker"], "utterance": t["utterance"]} for t in dialogue["turns"]]
    runtime_file.write_text(json.dumps(inputs))
    result = subprocess.run(command, check=True, capture_output=True, text=True, timeout=30)
    outputs = [json.loads(line) for line in result.stdout.splitlines()]
    expected = [predictions[(dialogue["dialogue_id"], index)] for index, t in enumerate(inputs) if t["speaker"] == "USER"]
    assert outputs == expected, "annotation-free inference diverges"
    count += len(outputs)
runtime_file.write_text(json.dumps([{"speaker":"USER", "utterance":"hello", "frames":[]}]))
result = subprocess.run(command, capture_output=True, text=True, timeout=30)
assert result.returncode != 0 and "unknown field" in result.stderr
report = {"dialogues":len(dialogues), "identical_annotation_free_predictions":count,
          "annotation_bearing_runtime_input_rejected":True}
print(json.dumps(report, indent=2))
