"""Reconstruct source features from original text and frozen predicted history."""
from pathlib import Path
from collections import defaultdict, deque
import json
HERE = Path(__file__).resolve().parent
WORK = HERE.parents[1]
SUB = WORK/'contextledger-run/terminal-originals/worktree/tools/dialogue/context-ledger'
data = sorted(json.loads((WORK/'entityledger-source/train.json').read_text()),key=lambda d:d['dialogue_id'])
baseline = json.loads((WORK/'contextledger-source/baseline-train.json').read_text())
sources = defaultdict(list)
for line in (SUB/'artifacts/source-spans.jsonl').open():
    row = json.loads(line)['row']
    sources[row['id'],row['turn']].append(row)
def tokens(text):
    result = []; word = ''
    for char in text:
        if char.isalnum(): word += char
        else:
            if word: result.append(word); word = ''
            if not char.isspace(): result.append(char)
    if word: result.append(word)
    return result
def mask(text, values):
    text = text.lower()
    for value in sorted({v.lower() for v in values if v},key=lambda v:(-len(v.encode()),v)):
        text = text.replace(value,' <entity> ')
    return text
verified = 0
for d, snapshots in zip(data,baseline,strict=True):
    by_turn = {s['turn_index']:s for s in snapshots}
    history = deque(); system_count = 0; user = ''; prior = None
    for ti, turn in enumerate(d['turns']):
        if turn['speaker']=='USER':
            prior = by_turn[ti]; user = turn['utterance']; continue
        rows = sources.pop((d['dialogue_id'],ti),[])
        raw = turn['utterance'].encode()
        current = [raw[r['raw']['start']:r['raw']['end']].decode() for r in rows]
        state = []; values = [v for _,v in history]+current
        if prior:
            state += ['active='+s for s in prior['active_services']]
            for f in prior['frames']:
                state.append('intent='+f['service']+':'+f['state']['active_intent'])
                for slot, vs in sorted(f['state']['slot_values'].items()):
                    state.append('filled='+f['service']+':'+slot); values += vs
        for row in rows:
            a,b = row['raw']['start'],row['raw']['end']
            left = tokens(mask(raw[:a].decode(),values)); right = tokens(mask(raw[b:].decode(),values))
            context = ['bias','len='+str(min(16,len(tokens(raw[a:b].decode()))))]
            context += [f'left{i}={v}' for i,v in enumerate(reversed(left[-3:]))]
            context += [f'right{i}={v}' for i,v in enumerate(right[:3])]
            context += ['ctx='+v for v in left+right]
            if len(user.encode())<=4096 and len(tokens(user))<=128:
                words = tokens(mask(user,values))
                context += ['previous_user='+v for v in words]
                context += ['previous_bigram='+a+'_'+b for a,b in zip(words,words[1:])]
            context += state
            assert context==row['context'],(d['dialogue_id'],ti,context,row['context'])
            verified += 1
        system_count += 1
        history.extend((system_count,v) for v in current)
        while len(history)>128 or (history and system_count-history[0][0]>=32): history.popleft()
assert not sources and verified==34223
report = dict(source_contexts_reconstructed=verified,original_conversations=len(data),
              features='Actual source text, actual preceding USER, bounded predicted-span mask history and frozen baseline state; no annotation values used',
              scope='Reconstructs all reported source contexts; does not independently rerun full BIO decoding or retrain weights')
(HERE/'context-review.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
