#!/usr/bin/env python3
"""Fetch only the allowed prefix of pinned official CLINC150 data.

Stops on the first disallowed top-level section header, BEFORE consuming
any of its items. No complete upstream JSON or test file is saved.
The official file's later oos_train section is deliberately not accessed.
"""
import argparse
import collections
import hashlib
import json
import pathlib
import urllib.request

REVISION = "828f8093932c8fe6ca7936c3d2e52903b1c523de"
URL = f"https://raw.githubusercontent.com/clinc/oos-eval/{REVISION}/data/data_full.json"
ALLOWED = {"train", "val", "oos_val"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=pathlib.Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    sections = {}
    current = None
    consumed = 0
    with urllib.request.urlopen(URL, timeout=60) as response:
        upstream_bytes = int(response.headers.get("Content-Length", "0"))
        for line in response:
            consumed += len(line)
            if line.startswith(b'    "'):
                key = line.split(b'"', 2)[1].decode("ascii")
                if key not in ALLOWED:
                    break
                current = key
                sections[current] = [b"["]
            elif current:
                if line.startswith(b"    ]"):
                    sections[current].append(b"]")
                    current = None
                else:
                    sections[current].append(line)
    if set(sections) != ALLOWED:
        raise RuntimeError("Upstream layout changed: allowed splits are incomplete")
    parsed = {key: json.loads(b"".join(value)) for key, value in sections.items()}
    if {key: len(value) for key, value in parsed.items()} != {
        "train": 15000, "val": 3000, "oos_val": 100
    }:
        raise RuntimeError("Unexpected official split sizes")
    train_counts = collections.Counter(label for _, label in parsed["train"])
    val_counts = collections.Counter(label for _, label in parsed["val"])
    if len(train_counts) != 150 or set(train_counts.values()) != {100} or set(val_counts) != set(train_counts) or set(val_counts.values()) != {20}:
        raise RuntimeError("Unexpected official per-label counts")
    overlap = len({text for text, _ in parsed["train"]} & {text for text, _ in parsed["val"] + parsed["oos_val"]})
    files = {}
    for split, rows in [("train", parsed["train"]), ("val", parsed["val"] + parsed["oos_val"])]:
        content = "".join(json.dumps({"text": text, "label": label}, ensure_ascii=False) + "\n" for text, label in rows).encode()
        (args.output / f"{split}.jsonl").write_bytes(content)
        files[split] = {"rows": len(rows), "bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()}
    metadata = {
        "source": URL, "revision": REVISION, "license": "CC-BY-3.0",
        "license_url": f"https://github.com/clinc/oos-eval/blob/{REVISION}/LICENSE",
        "attribution": "Larson et al. (2019), An Evaluation Dataset for Intent Classification and Out-of-Scope Prediction",
        "upstream_bytes": upstream_bytes, "prefix_bytes_consumed_including_stop_header": consumed,
        "training_note": "Only 15000 in-scope training rows used. Later oos_train is never accessed.",
        "test_policy": "Stopped at first disallowed section header before reading any test item.",
        "official_exact_train_val_text_overlap": overlap,
        "files": files,
    }
    (args.output / "source.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(json.dumps(metadata, indent=2))


if __name__ == "__main__":
    main()
