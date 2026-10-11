#!/usr/bin/env python3
"""Deterministic CLI smoke check of memory transitions and disk persistence.

These synthetic dev conversations are authored for this implementation, not
an independent dialogue benchmark. Checks inspect kinds, provenance and
remembered values instead of matching complete response snapshots.
"""
import argparse
import json
import pathlib
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    cases = [
        ("What is my name?", "missing", None, None),
        ("My name is Ada Lovelace.", "remembered", "Ada Lovelace", None),
        ("What is my name?", "recalled", "Ada Lovelace", 2),
        ("Actually, my name is Grace Hopper.", "remembered", "Grace Hopper", None),
        ("What about that?", "recalled", "Grace Hopper", 4),
        ("My favorite color is blue.", "remembered", "blue", None),
        ("My favorite drink is tea.", "remembered", "tea", None),
        ("What about my favorite color?", "recalled", "blue", 6),
        ("Actually, it's green.", "remembered", "green", None),
        ("What is my favorite color?", "recalled", "green", 9),
        ("And my favorite drink?", "recalled", "tea", 7),
        ("I prefer quiet rooms.", "remembered", "quiet rooms", None),
        ("What do I prefer?", "recalled", "quiet rooms", 12),
        ("No, I prefer noisy cafes.", "remembered", "noisy cafes", None),
        ("What about that?", "recalled", "noisy cafes", 14),
        ("What did I just say?", "recalled", "What about that?", 15),
        ("Hello", "social", "Grace Hopper", None),
        ("What did I just say?", "recalled", "Hello", 17),
        ("Forget my name", "social", None, None),
        ("What is my name?", "missing", None, None),
        ("And my favorite drink?", "recalled", "tea", 7),
        ("Forget my preferences", "social", None, None),
        ("What do I prefer?", "missing", None, None),
        ("A completely unsupported request", "refusal", None, None),
        ("What did I just say?", "recalled", "A completely unsupported request", 24),
    ]
    checks = 0

    def run(rows, session=None):
        nonlocal checks
        command = [str(args.binary.resolve()), "chat", "--json"]
        if session:
            command += ["--session", str(session)]
        result = subprocess.run(command, input="\n".join(row[0] for row in rows) + "\n", text=True, capture_output=True, check=True)
        replies = [json.loads(line) for line in result.stdout.splitlines()]
        if len(replies) != len(rows):
            raise RuntimeError(f"Expected {len(rows)} replies, received {len(replies)}")
        for row, reply in zip(rows, replies):
            utterance, kind, value, turn = row
            if reply["kind"] != kind or (value and value not in reply.get("text", "")) or (turn is not None and reply.get("source_turn") != turn):
                raise RuntimeError(f"Memory smoke failure for {utterance!r}: {reply}")
            checks += 1

    run(cases)
    session = args.output / "session.json"
    session.unlink(missing_ok=True)
    run([
        ("My name is İpek", "remembered", "İpek", None),
        ("MY FAVORITE drink IS Café con leche", "remembered", "Café con leche", None),
    ], session)
    run([
        ("And my name?", "recalled", "İpek", 1),
        ("What is my favorite drink?", "recalled", "Café con leche", 2),
        ("No, it's Oolong", "remembered", "Oolong", None),
    ], session)
    state = json.loads(session.read_text())
    if state["last_correction"]["previous"]["value"] != "Café con leche" or state["preferences"]["drink"]["value"] != "Oolong":
        raise RuntimeError("Correction provenance was not persisted")
    checks += 1
    run([
        ("/reset", "reset", None, None),
        ("What is my name?", "missing", None, None),
        ("What did I just say?", "recalled", "What is my name?", 1),
    ], session)
    run([("Actually, it's Alice", "missing", None, None)])
    run([
        ("Call me a taxi", "refusal", None, None),
        ("What is my name?", "missing", None, None),
        ("Call me tomorrow", "refusal", None, None),
        ("What is my name?", "missing", None, None),
        ("Call me Dr. Kim", "remembered", "Dr. Kim", None),
        ("What is my name?", "recalled", "Dr. Kim", 5),
    ])
    print(json.dumps({"synthetic_memory_smoke": {"right": checks, "wrong": 0}, "independent_benchmark": False}, indent=2))


if __name__ == "__main__":
    main()
