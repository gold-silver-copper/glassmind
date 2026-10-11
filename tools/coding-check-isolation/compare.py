#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Compare full authored runs, masking only OS-assigned panic thread IDs."""
import argparse
import json
from pathlib import Path
import re

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('baseline', type=Path)
p.add_argument('candidate', type=Path)
p.add_argument('output', type=Path)
a = p.parse_args()
before = json.loads(a.baseline.read_text())
after = json.loads(a.candidate.read_text())
assert before['fixture_sha256'] == after['fixture_sha256']
rows = []
for old, new in zip(before['cases'], after['cases'], strict=True):
    assert old['case'] == new['case']
    stable = lambda text: re.sub(r'\([0-9]+\) panicked', '(PID) panicked', text)
    rows.append(dict(case=old['case'], baseline_pass=old['passed'], candidate_pass=new['passed'],
        diagnostics_equal_except_thread_id=stable(old['stdout']) == stable(new['stdout']),
        baseline_changed_entries=len(old['delta']), candidate_changed_entries=len(new['delta'])))
report = dict(baseline_passed=before['passed'], candidate_passed=after['passed'], total=after['total'],
              fixture_sha256=after['fixture_sha256'], baseline_binary_sha256=before['binary_sha256'],
              candidate_binary_sha256=after['binary_sha256'], cases=rows)
a.output.write_text(json.dumps(report, indent=2)+'\n')
assert after['total'] == 16
assert all(r['candidate_pass'] and r['diagnostics_equal_except_thread_id'] for r in rows)
print(f"{before['passed']}/16 -> {after['passed']}/16; all diagnostic responses preserved")
