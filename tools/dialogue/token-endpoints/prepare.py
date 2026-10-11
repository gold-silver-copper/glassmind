"""Prepare selected TRAIN speaker/utterance-only input, never DEV/test."""
import argparse
import json
import pathlib

p = argparse.ArgumentParser()
p.add_argument('--root', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args()
dialogues = json.loads((a.root / 'selected/train.json').read_text())
assert len(dialogues) == 3174
assert sum(t['speaker'] == 'USER' for d in dialogues for t in d['turns']) == 34022
a.output.parent.mkdir(parents=True, exist_ok=True)
a.output.write_text(json.dumps([[{'speaker': t['speaker'], 'utterance': t['utterance']}
                               for t in d['turns']] for d in dialogues], ensure_ascii=False))
