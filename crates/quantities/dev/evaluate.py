#!/usr/bin/env python3
"""Synthetic development adapter. Never opens external benchmark files.

Oracles use Python Fraction, Decimal, datetime and calendar; they do not invoke
or import the Rust implementation. The fixed sample selects five per category.
"""
import argparse
import calendar
from collections import Counter, defaultdict
from datetime import date, timedelta
from decimal import Decimal
from fractions import Fraction
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import subprocess

VERSION = "quantities-synthetic-dev-v1"
SEED = 20261010
ROOT = Path(__file__).resolve().parent


def generate():
    rng = random.Random(SEED)
    rows = []

    def add(category, question, answer=None, unit=None):
        rows.append(dict(id=f"{category}-{sum(r['category'] == category for r in rows):04}",
                         category=category, question=question,
                         expected=None if answer is None else str(answer), unit=unit))

    # Independent published unit definitions expressed as Decimal, not the
    # rational numerator/denominator representation used by Rust.
    units = [("length", {"meters": "1", "kilometers": "1000", "centimeters": ".01", "inches": ".0254", "feet": ".3048", "miles": "1609.344"}),
             ("mass", {"kilograms": "1", "grams": ".001", "pounds": ".45359237", "ounces": ".028349523125"}),
             ("duration", {"seconds": "1", "minutes": "60", "hours": "3600", "days": "86400", "weeks": "604800"}),
             ("volume", {"liters": ".001", "milliliters": ".000001", "us gallons": ".003785411784"})]
    for i in range(80):
        a, b, c = [rng.randint(1, 200) for _ in range(3)]
        x, y = Fraction(a, b), Fraction(b, c)
        add("expression", f"Calculate ({a}/{b} + {b}/{c}) * {c} - {a}", (x+y)*c-a)
        left, right = Decimal(a) / 100, Decimal(b) / 100
        add("decimal", f"What is {left} plus {right} times 3?", Fraction(left + right*3))
        add("percent", f"What is {a} percent of {b}?", Fraction(a*b, 100))
        add("operator_precedence", f"Evaluate -{a} + {b} * ({c} - 7) / 3", -a+Fraction(b*(c-7), 3))
        for dimension, factors in units:
            source, target = rng.sample(list(factors), 2)
            value = Decimal(a) / 10
            expected = Fraction(value) * Fraction(Decimal(factors[source])) / Fraction(Decimal(factors[target]))
            question = f"Convert {value} {source} to {target}" if i % 2 else f"How many {target} are in {value} {source}?"
            add(f"units_{dimension}", question, expected, target)
        fahrenheit = Decimal(a) - 100
        add("temperature", f"Convert {fahrenheit} fahrenheit to celsius", (Fraction(fahrenheit)-32)*Fraction(5, 9), "celsius")
        add("speed", f"Convert {a} km/h to m/s", Fraction(a, 1) / Fraction(Decimal("3.6")), "m/s")
        year = [1900, 1999, 2000, 2004, 2024, 2100, 2400, 9998][i % 8]
        start = date(year, [1, 2, 3, 12][i % 4], min(28, i % 28+1))
        shift = rng.randint(1, 365)
        direction = 1 if i % 2 else -1
        result = start + timedelta(days=shift*direction)
        add("date_days", f"What date is {shift} days {'after' if direction == 1 else 'before'} {start}?", result)
        add("date_difference", f"How many days from {start} to {result}?", (result-start).days)
        add("weekday", f"What day of the week is {start}?", start.strftime("%A"))
        start = date(year, [1, 2, 3, 12][i % 4], calendar.monthrange(year, [1, 2, 3, 12][i % 4])[1])
        shift = i % 15 + 1
        index = start.year * 12 + start.month - 1 + shift*direction
        new_year, new_month = divmod(index, 12)
        result = date(new_year, new_month+1, min(start.day, calendar.monthrange(new_year, new_month+1)[1]))
        add("date_months", f"What date is {shift} months {'after' if direction == 1 else 'before'} {start}?", result)
        obj = ["apples", "coins", "books", "marbles"][i % 4]
        verb = ["gets", "loses", "buys", "gives away"][i % 4]
        initial = a+b
        result = initial+b if verb in ("gets", "buys") else initial-b
        add("inventory", f"Mira has {initial} {obj}. She {verb} {b} {obj}. How many {obj} does she have now?", result)
        add("groups", f"There are {a} boxes with {b} {obj} in each. How many {obj} are there in total?", a*b)
        add("sharing", f"{a*b} {obj} are shared equally among {b} children. How many {obj} does each child get?", a)
        add("price", f"Each book costs {left} dollars. What is the cost of {b} books?", Fraction(left*b), "dollars")
        # Deliberately broader language to expose unsupported coverage.
        add("sum_wording", f"What is the sum of {a} and {b}?", a+b)
        add("comparison", f"Ava has {a+b} {obj}. Ben has {b} {obj}. How many more {obj} does Ava have than Ben?", a)
        add("multi_event", f"Mira has {a+b} {obj}. She gets {c} {obj}. She gives away {b} {obj}. How many {obj} does she have now?", a+c)
        add("unit_mismatch", f"Convert {a} meters to kilograms")
        add("zero_divisor", f"Calculate {a} / ( {b} - {b} )")
        add("invalid_date", f"What date is {a} days after 1900-02-29?")
        add("unknown_unit", f"Convert {a} gallons to liters")
        add("object_mismatch", f"Mira has {a} apples. She gets {b} coins. How many apples does she have now?")
    return rows


def generate_extended():
    rng = random.Random(SEED + 1)
    rows = []

    def spell(n):
        ones = "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split()
        tens = "zero ten twenty thirty forty fifty sixty seventy eighty ninety".split()
        if n < 20:
            return ones[n]
        if n < 100:
            return tens[n//10] + ("-" + ones[n % 10] if n % 10 else "")
        if n < 1000:
            return ones[n//100] + " hundred" + (" and " + spell(n % 100) if n % 100 else "")
        return spell(n//1000) + " thousand" + (" " + spell(n % 1000) if n % 1000 else "")

    def add(category, question, answer=None, unit=None):
        rows.append(dict(id=f"{category}-{sum(r['category'] == category for r in rows):04}",
                         category=category, question=question,
                         expected=None if answer is None else str(answer), unit=unit))

    for i in range(60):
        a, b, c = [rng.randint(1, 200) for _ in range(3)]
        cardinal = rng.randint(0, 999999)
        add("number_words", f"What is {spell(cardinal)} plus {spell(a)}?", cardinal+a)
        add("unit_addition", f"What is {a} meters plus {b} centimeters in feet?",
            (Fraction(a) + Fraction(b, 100)) / Fraction(Decimal(".3048")), "feet")
        add("unit_addition_mismatch", f"What is {a} meters plus {b} kilograms in feet?")
        add("rate", f"A train travels {a} kilometers in {b} minutes. What is its speed in m/s?",
            Fraction(a*1000, b*60), "m/s")
        add("distance", f"A train travels at {a} mph for {b} minutes. How far does it travel in kilometers?",
            Fraction(a*b, 60) * Fraction(Decimal("1.609344")), "kilometers")
        year = [1600, 1904, 2000, 2096, 2400][i % 5]
        start = date(year, 2, 29)
        shift = i % 9 + 1
        result = date(year+shift, 2, min(29, calendar.monthrange(year+shift, 2)[1]))
        add("leap_year_shift", f"What date is {shift} years after {start}?", result)
        add("weeks", f"What date is {a} weeks before {start}?", start-timedelta(weeks=a))
        start = date(rng.randint(2, 9998), rng.randint(1, 12), rng.randint(1, 28))
        add("calendar_span", f"What date is {b} days after {start}?", start+timedelta(days=b))
        add("date_between", f"How many days between {start+timedelta(days=b)} and {start}?", b)
        add("kelvin", f"Convert {a} kelvin to fahrenheit", Fraction(a)*Fraction(9, 5)-Fraction(Decimal("459.67")), "fahrenheit")
        add("named_product", f"What is the product of {a}/{b} and {b}/{c}?", Fraction(a, c))
        add("recipient_mismatch", f"{a} coins are shared equally among {b} children. How many coins does each cat get?")
        add("entity_mismatch", f"Mira has {a} coins. Ava gets {b} coins. How many coins does she have now?")
        add("negative_inventory", f"Mira has {a} coins. She gives away {a+b} coins. How many coins does she have now?")
        add("overflow", f"What is {10**50+a} times {b}?")
        add("calendar_unit_ambiguity", f"Convert {a} months to days")
        # Deliberately outside the implemented sentence grammar. These are
        # answerable, so refusal counts as missing coverage, not a right answer.
        add("paraphrase_gap", f"Initially, Mira owned {a} coins; after receiving another {b}, what was her total?", a+b)
    for year in (100, 400, 1600, 1700, 1900, 2000, 2100, 2400, 9999):
        start = date(year, 2, 28)
        add("century_boundary", f"What date is 1 day after {start}?", start+timedelta(days=1))
    add("date_range", "What date is 1 day before 0001-01-01?")
    add("date_range", "What date is 1 day after 9999-12-31?")
    for i in range(1, 6):
        add("rational_cancellation", f"What is {10**30}/3 times 3/{10**30+i}?", Fraction(10**30, 10**30+i))
    return rows


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=Path(os.environ.get("CARGO_TARGET_DIR", "target")) / "debug/glassmind-quantities")
    parser.add_argument("--sample", action="store_true")
    parser.add_argument("--suite", choices=("core", "extended"), default="core")
    parser.add_argument("--generate-only", action="store_true")
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    rows = generate() if args.suite == "core" else generate_extended()
    version = VERSION if args.suite == "core" else "quantities-extended-dev-v1"
    stem = "synthetic-dev" if args.suite == "core" else "extended-dev"
    serialized = "".join(json.dumps(row, sort_keys=True) + "\n" for row in rows).encode()
    (ROOT / f"{stem}.jsonl").write_bytes(serialized)
    provenance = dict(source="locally generated deterministic English dev cases; no external benchmark items",
                      version=version, seed=SEED if args.suite == "core" else SEED+1, rows=len(rows), bytes=len(serialized),
                      sha256=hashlib.sha256(serialized).hexdigest(), license="MIT OR Apache-2.0 (workspace)",
                      oracle="Python stdlib Fraction, Decimal, datetime, calendar", python=platform.python_version(),
                      external_training_data_used=False, held_out_data_accessed=False)
    (ROOT / f"{stem}-provenance.json").write_text(json.dumps(provenance, indent=2)+"\n")
    if args.generate_only:
        print(json.dumps(provenance, indent=2))
        return
    if args.sample:
        counts = Counter()
        selected = []
        for row in rows:
            if counts[row["category"]] < 5:
                selected.append(row)
                counts[row["category"]] += 1
        rows = selected
    process = subprocess.run([str(args.binary.resolve()), "--jsonl"],
                             input="".join(json.dumps(r["question"])+"\n" for r in rows),
                             text=True, capture_output=True, timeout=120, check=True)
    answers = [json.loads(line) for line in process.stdout.splitlines()]
    if len(answers) != len(rows):
        raise RuntimeError(f"expected {len(rows)} answers, got {len(answers)}")
    totals, categories, failures = Counter(), defaultdict(Counter), []
    for row, answer in zip(rows, answers):
        if answer["status"] == "refusal":
            outcome = "right" if row["expected"] is None else "refusal"
            if outcome == "right":
                totals["correct_refusals"] += 1
        else:
            actual, expected = answer["value"], row["expected"]
            correct = actual == expected
            if expected is not None:
                try:
                    correct = Fraction(actual) == Fraction(expected)
                except (ValueError, ZeroDivisionError):
                    pass
            outcome = "right" if correct and answer.get("unit") == row["unit"] else "wrong"
        totals[outcome] += 1
        categories[row["category"]][outcome] += 1
        if outcome != "right":
            failures.append(dict(id=row["id"], question=row["question"], expected=row["expected"], actual=answer, outcome=outcome))
    report = dict(dataset=version, split="fixed-dev-sample" if args.sample else "full-dev", total=len(rows),
                  **{key: totals[key] for key in ("right", "refusal", "wrong", "correct_refusals")},
                  categories={k: {metric: v[metric] for metric in ("right", "refusal", "wrong")} for k, v in sorted(categories.items())},
                  failures=failures, provenance=provenance)
    if args.report:
        args.report.write_text(json.dumps(report, indent=2)+"\n")
    print(json.dumps({k: v for k, v in report.items() if k != "failures"}, indent=2))


if __name__ == "__main__":
    main()
