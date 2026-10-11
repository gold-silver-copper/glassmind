#!/usr/bin/env python3
"""Fetch ONLY official train/dev URLs; retain the complete single-service slice.

No repository archive, recursive tree, test directory, or held-out URL is used.
Only four downloads are concurrent. Nonselected train/dev dialogues are discarded.
"""
import argparse
import concurrent.futures
import hashlib
import json
from pathlib import Path
import urllib.request

VERSION = "e852981ae34990f4358979625854259302feaa78"
REPO = "google-research-datasets/dstc8-schema-guided-dialogue"


def download(path):
    assert path == "LICENSE.txt" or path.split("/")[0] in ("train", "dev")
    url = f"https://raw.githubusercontent.com/{REPO}/{VERSION}/{path}"
    data = urllib.request.urlopen(url, timeout=60).read()
    return data, {"path": path, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path)
    parser.add_argument("--service", default="RideSharing_1")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    manifest = {"repository": REPO, "version": VERSION, "license": "CC-BY-SA-4.0",
                "selection": {"train_contains_service": args.service,
                              "dev_services_exactly": [args.service]}, "files": [], "splits": {}}
    license_data, entry = download("LICENSE.txt")
    (args.output / "LICENSE.txt").write_bytes(license_data)
    manifest["files"].append(entry)
    for split in ("train", "dev"):
        schema_data, entry = download(split + "/schema.json")
        manifest["files"].append(entry)
        schema = [s for s in json.loads(schema_data) if s["service_name"] == args.service]
        if len(schema) != 1:
            raise ValueError("Service must exist in BOTH train and dev")
        (args.output / (split + "-schema.json")).write_text(json.dumps(schema, indent=2) + "\n")
        listing = json.load(urllib.request.urlopen(
            f"https://api.github.com/repos/{REPO}/contents/{split}?ref={VERSION}", timeout=60))
        names = sorted(f["name"] for f in listing
                       if f["name"].startswith("dialogues_") and f["name"].endswith(".json"))
        selected = []
        def fetch(name):
            data, record = download(split + "/" + name)
            dialogues = [d for d in json.loads(data)
                         if (args.service in d["services"] if split == "train"
                             else d["services"] == [args.service])]
            return dialogues, record
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            for dialogues, record in pool.map(fetch, names):
                selected.extend(dialogues)
                manifest["files"].append(record)
        selected.sort(key=lambda d: d["dialogue_id"])
        target = args.output / (split + ".json")
        target.write_text(json.dumps(selected, ensure_ascii=False) + "\n")
        manifest["splits"][split] = {
            "dialogues": len(selected), "user_turns": sum(t["speaker"] == "USER" for d in selected for t in d["turns"]),
            "retained_bytes": target.stat().st_size,
            "sha256": hashlib.sha256(target.read_bytes()).hexdigest(), "shards_scanned": len(names)}
        print(split, manifest["splits"][split], flush=True)
    manifest["downloaded_bytes"] = sum(f["bytes"] for f in manifest["files"])
    (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
