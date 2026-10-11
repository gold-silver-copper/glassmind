"""Independent endpoint/rule-count audit of retained TRAIN proposals; no inference."""
import argparse
import collections
import hashlib
import json
from pathlib import Path


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--train", type=Path, required=True)
    p.add_argument("--reports", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    a = p.parse_args()
    dialogues = json.loads(a.train.read_text())
    traces = [json.loads(x) for x in (a.reports / "train-reference-proposals.jsonl").read_text().splitlines()]
    audit = [json.loads(x) for x in (a.reports / "audit.jsonl").read_text().splitlines()]
    gate = json.loads((a.reports / "gate.json").read_text())
    assert len(traces) == len(audit) == 165
    counts = collections.Counter()
    source_counts = collections.Counter()
    stats = collections.defaultdict(lambda: {s: {"ids": set(), "rows": 0, "mismatches": 0} for s in ("FIT", "CAL")})
    for trace, row in zip(traces, audit):
        assert all(row[k] == v for k, v in trace.items())
        d = dialogues[trace["conversation"]]
        part = "CAL" if int(hashlib.sha256(d["dialogue_id"].encode()).hexdigest(), 16) % 5 == 0 else "FIT"
        assert row["partition"] == part and row["dialogue_id"] == d["dialogue_id"]
        counts[part + "_reference_proposals"] += 1
        if trace["punctuation_value"] is not None:
            assert row["status"] == "existing_punctuation_priority"
            counts[row["status"]] += 1
            continue
        value = trace["value"]
        location = None
        for ti in reversed(range(trace["turn"])):
            text = d["turns"][ti]["utterance"]
            end = len(text)
            while end >= len(value):
                start = text.rfind(value, 0, end)
                if start < 0:
                    break
                stop = start + len(value)
                if (start == 0 or not text[start-1].isalnum()) and (stop == len(text) or not text[stop].isalnum()):
                    location = (ti, start, stop)
                    break
                end = stop - 1
            if location is not None:
                break
        assert location is not None
        ti, start, end = location
        turn = d["turns"][ti]
        text = turn["utterance"]
        gold = set()
        for frame in turn["frames"]:
            for slot in frame.get("slots", []):
                if [frame["service"], slot["slot"]] in trace["sources"] and slot["start"] == start:
                    gold.add(slot["exclusive_end"])
        source_counts[part + ("_exact_original_endpoint" if gold == {end} else "_missing_source_annotation" if not gold else "_different_endpoint")] += 1
        assert row["source_turn"] == ti
        assert row["start"] == len(text[:start].encode())
        assert row["original_end"] == len(text[:end].encode())
        counts[row["status"]] += 1
        for proposal in row["proposals"]:
            # Decode byte offsets strictly: invalid UTF-8 boundaries cannot pass.
            proposed = text.encode()[row["start"]:proposal["end"]].decode()
            assert proposed == proposal["value"] and proposed.startswith(value)
            char_end = start + len(proposed)
            matched = gold == {char_end}
            assert proposal["endpoint_match"] == matched
            assert proposal["gold_ends"] == sorted(len(text[:x].encode()) for x in gold)
            assert proposal["missing_annotation"] == (not gold)
            counts[part + "_generated_proposals"] += 1
            counts[part + "_endpoint_mismatches"] += not matched
            counts[part + "_missing_annotations"] += not gold
            stat = stats[(tuple(row["key"]), proposal["length"])][part]
            stat["ids"].add(d["dialogue_id"])
            stat["rows"] += 1
            stat["mismatches"] += not matched
    assert dict(counts) == gate["totals"], (dict(counts), gate["totals"])
    candidates = json.loads((a.reports / "candidates.json").read_text())
    lengths = {tuple(c["key"]): c["lengths"] for c in candidates}
    accepted = []
    for rule in gate["rules"]:
        by_part = stats[(tuple(rule["key"]), rule["length"])]
        reasons = []
        if len(lengths[tuple(rule["key"])]) > 1:
            reasons.append("conflicting_FIT_discovery_lengths")
        for part, minimum in (("FIT", 5), ("CAL", 3)):
            stat = by_part[part]
            expected = {"conversations": len(stat["ids"]), "rows": stat["rows"], "mismatches": stat["mismatches"]}
            assert rule[part] == expected
            if expected["conversations"] < minimum:
                reasons.append(part + "_insufficient_actual_conversations")
            if expected["mismatches"]:
                reasons.append(part + "_endpoint_mismatches")
        assert rule["rejections"] == reasons and rule["accepted"] == (not reasons)
        if not reasons:
            accepted.append(rule)
    assert not accepted and gate["qualifying_rules"] == gate["qualifying_CAL_instances"] == 0
    original = json.loads((a.reports / "source-endpoints.json").read_text())
    assert dict(source_counts) == original["counts"]
    result = {"decision": "REJECT_BEFORE_DEV", "trace_rows": len(traces), "independently_checked_rules": len(gate["rules"]), "counts": dict(counts), "source_endpoints": dict(source_counts), "native_inference_repeated": False, "limitation": "Independent source/endpoint arithmetic on retained TRAIN replay; classifiers previously saw CAL."}
    a.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
