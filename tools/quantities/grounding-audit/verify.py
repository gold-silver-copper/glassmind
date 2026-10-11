import json,os
from pathlib import Path
R=Path(os.environ.get('AUDIT_RUN',str(Path.cwd().parent/'groundingaudit-run/audit')));S=Path.cwd().parent/'groundingaudit-source'
def read(p):return [json.loads(l) for l in p.read_text().splitlines()]
assert read(R/'replay200-original.jsonl')==read(R/'replay200-expected.jsonl')
assert len(read(R/'replay200-original.jsonl'))==200
assert read(R/'gold-all-original.jsonl')==read(S/'train-executions.jsonl')
assert len(read(R/'gold-all-original.jsonl'))==29837
if (R/'replay200-helper.jsonl').exists():assert read(R/'replay200-helper.jsonl')==read(R/'replay200-original.jsonl')
(R/'native-verification.json').write_text(json.dumps(dict(first200_exact=True,all29837_gold_execution_exact=True,helper200_exact=(R/'replay200-helper.jsonl').exists()))+'\n')
