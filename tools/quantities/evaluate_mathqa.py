#!/usr/bin/env python3
"""Independent exact-numeric MathQA dev adapter; only Problem enters Rust.

All official dev rows count. Non-numeric gold options are marked unscorable;
answers on them count wrong, refusals count refused. No formula execution.
"""
import argparse
from collections import Counter
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import re
import subprocess


def numeric_option(row):
    options = dict(re.findall(r'([a-e])\s*\)\s*(.*?)(?=\s*,?\s*[a-e]\s*\)|$)', row['options']))
    text = options[row['correct']].strip().rstrip(',').strip()
    # The numeric answer is expressed in the question's requested unit.
    text = text.replace(',', '').replace(' ', '').rstrip('%')
    if not re.fullmatch(r'[+-]?\d+(?:\.\d+|/\d+)?', text):
        return None
    try:
        return Fraction(text)
    except (ValueError, ZeroDivisionError):
        return None


def main():
    import pyarrow.parquet as pq
    ap = argparse.ArgumentParser()
    ap.add_argument('--data', type=Path, required=True)
    ap.add_argument('--binary', type=Path, required=True)
    ap.add_argument('--report', type=Path, required=True)
    ap.add_argument('--sample', action='store_true')
    args = ap.parse_args()
    rows = pq.read_table(args.data / 'validation.parquet').to_pylist()
    assert len(rows) == 4475
    indexed = list(enumerate(rows))[:100] if args.sample else list(enumerate(rows))
    questions = ''.join(json.dumps(row['Problem'])+'\n' for _,row in indexed)
    run = subprocess.run([str(args.binary.resolve()), '--jsonl'], input=questions,
                         text=True, capture_output=True, check=True, timeout=120)
    answers = [json.loads(line) for line in run.stdout.splitlines()]
    assert len(answers) == len(indexed)
    counts = Counter(); answered = []
    for (index,row), answer in zip(indexed, answers):
        gold = numeric_option(row)
        if gold is None: counts['non_numeric_gold'] += 1
        if answer['status'] == 'refusal':
            outcome = 'refused'
        else:
            try: correct = gold is not None and Fraction(answer['value']) == gold
            except (ValueError, ZeroDivisionError): correct = False
            outcome = 'right' if correct else 'wrong'
            answered.append(dict(index=index, question=row['Problem'], expected=str(gold),
                                 actual=answer, outcome=outcome))
        counts[outcome] += 1
    report = dict(dataset='MathQA', split='official-validation-first-100' if args.sample else 'official-validation',
                  total=len(indexed), **{k:counts[k] for k in ['right','refused','wrong','non_numeric_gold']},
                  metric='exact numeric value in requested units; no choice input; all rows count',
                  questions_sha256=hashlib.sha256(questions.encode()).hexdigest(),
                  binary_sha256=hashlib.sha256(args.binary.read_bytes()).hexdigest(), answered=answered)
    args.report.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='answered'},indent=2))

if __name__ == '__main__': main()
