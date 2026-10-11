"""Compare the pilot normalizer with the compiled current Rust normalization."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import unicodedata

root = Path(__file__).resolve().parent
process = subprocess.Popen([str(root / 'normalization-probe')], stdout=subprocess.PIPE,
                           stderr=subprocess.PIPE, text=True)
separators = re.compile(r'[\W_]+')
total = different = 0
examples = []
hash_stream = hashlib.sha256()
for line in process.stdout:
    ordinal, actual = line.rstrip('\n').split('\t')
    codepoint = int(ordinal)
    expected = separators.sub(' ', f'A{chr(codepoint)}B'.lower()).strip()
    total += 1
    if actual != expected:
        different += 1
        hash_stream.update(line.encode())
        if len(examples) < 32:
            examples.append({'codepoint': f'U+{codepoint:04X}',
                             'name': unicodedata.name(chr(codepoint), 'UNASSIGNED'),
                             'rust': actual, 'python': expected})
rust_unicode = process.stderr.read().strip()
assert process.wait() == 0
report = {'scope': 'every Unicode scalar inserted between A and B; current Rust Store normalization versus pilot Python regex',
          'scalars': total, 'different': different,
          'difference_stream_sha256': hash_stream.hexdigest(),
          'rust_unicode': rust_unicode, 'python_unicode': unicodedata.unidata_version,
          'examples': examples,
          'conclusion': 'Require native Rust normalization before runtime use; this is not an answer-accuracy measurement.'}
(root / 'normalization-report.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k: v for k, v in report.items() if k != 'examples'}, indent=2))
