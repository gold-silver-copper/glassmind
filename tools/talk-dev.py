#!/usr/bin/env python3
"""Fixed synthetic dev conversation for calculation routing and memory interaction.

This checks product integration, not independent language generalization.
Expected arithmetic uses Fraction and dates use datetime. Runtime input is
utterance text only; expected values and kinds never enter the subprocess.
"""
import argparse
from collections import Counter
from datetime import date, timedelta
from fractions import Fraction
import json
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--model", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    command = [str(args.binary.resolve()), "talk", "--json"]
    if args.model:
        command += ["--model", str(args.model.resolve())]
    outcomes = []

    def check(rows, session):
        run = subprocess.run(command + ["--session", str(session)],
                             input="\n".join(text for text, _ in rows) + "\n",
                             text=True, capture_output=True, timeout=30, check=True)
        replies = [json.loads(line) for line in run.stdout.splitlines()]
        if len(rows) != len(replies):
            raise RuntimeError("the CLI must return one reply per utterance")
        for (text, expected), actual in zip(rows, replies):
            kind = actual.get("kind")
            correct = all(actual.get(key) == value for key, value in expected.items()
                          if key != "contains")
            correct &= expected.get("contains", "") in actual.get("text", "")
            if expected.get("kind") == "calculated":
                correct &= bool(actual.get("method")) and bool(actual.get("text"))
            if correct:
                status = "right"
            elif expected.get("kind") == "calculated" and kind in {"refusal", "intent"}:
                status = "refused"
            else:
                status = "wrong"
            outcomes.append(dict(question=text, expected=expected, actual=actual,
                                 outcome=status))

    def calc(value, unit=None):
        return dict(kind="calculated", value=str(value), unit=unit)

    with tempfile.TemporaryDirectory(prefix="talk-dev-", dir=args.output.parent) as tmp:
        session = Path(tmp) / "session.json"
        check([
            ("My name is Mira", dict(kind="remembered", contains="Mira")),
            ("Calculate 2 + 3 * 4", calc(2 + 3 * 4)),
            ("What did I just say?", dict(kind="recalled", source_turn=2,
                                         contains="Calculate 2 + 3 * 4")),
            ("What is my name?", dict(kind="recalled", source_turn=1, contains="Mira")),
            ("Convert 1/3 hours to minutes", calc(Fraction(1, 3) * 60, "minutes")),
            ("What about my name?", dict(kind="recalled", source_turn=1, contains="Mira")),
            ("My favorite number is 12", dict(kind="remembered", contains="12")),
            ("What is my favorite number?", dict(kind="recalled", source_turn=7, contains="12")),
            ("What is twenty-one plus three?", calc(21 + 3)),
            ("Actually, it's 42", dict(kind="remembered", contains="42")),
            ("What is my favorite number?", dict(kind="recalled", source_turn=10, contains="42")),
            ("Convert 32 fahrenheit to celsius", calc(0, "celsius")),
            ("What date is 1 day after 2024-02-28?", calc(date(2024, 2, 28) + timedelta(days=1))),
            ("What day of the week is 2024-02-29?", calc(date(2024, 2, 29).strftime("%A"))),
            ("What is 25 percent of 80?", calc(Fraction(25 * 80, 100))),
            ("What date is 1 year after 2024-02-29?", calc(date(2025, 2, 28))),
        ], session)
        # A second CLI process must recover memory and recall the calculation.
        check([
            ("What did I just say?", dict(kind="recalled", source_turn=16,
                                         contains="What date is 1 year after 2024-02-29?")),
            ("What is my name?", dict(kind="recalled", source_turn=1, contains="Mira")),
            ("What is my favorite number?", dict(kind="recalled", source_turn=10, contains="42")),
            ("/reset", dict(kind="reset")),
            ("What is my name?", dict(kind="missing")),
            ("Calculate 1/2 + 1/3", calc(Fraction(1, 2) + Fraction(1, 3))),
            ("What did I just say?", dict(kind="recalled", source_turn=2,
                                         contains="Calculate 1/2 + 1/3")),
        ], session)
        # Unsupported/invalid requests must not become numeric answers. With
        # a model an honest service-intent proposal is also allowed here.
        invalid = ["Calculate 1 / 0", "Convert 3 meters to kilograms",
                   "What date is 1 day after 1900-02-29?",
                   "Calculate 2 + 2 and book me a flight"]
        run = subprocess.run(command, input="\n".join(invalid) + "\n", text=True,
                             capture_output=True, timeout=30, check=True)
        replies = [json.loads(line) for line in run.stdout.splitlines()]
        if len(replies) != len(invalid):
            raise RuntimeError("missing invalid-input replies")
        for text, reply in zip(invalid, replies):
            outcomes.append(dict(question=text, actual=reply,
                                 outcome="right" if reply.get("kind") in {"refusal", "intent"}
                                 else "wrong"))
    counts = Counter(row["outcome"] for row in outcomes)
    report = dict(dataset="talk-calculations-memory-synthetic-dev-v1",
                  independent_benchmark=False, total=len(outcomes),
                  right=counts["right"], refused=counts["refused"], wrong=counts["wrong"],
                  model=str(args.model) if args.model else None, outcomes=outcomes)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({key: value for key, value in report.items() if key != "outcomes"}))
    if counts["wrong"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
