#!/usr/bin/env python3
"""Utterance-only parity exercise. Expected data are old predictions, never gold."""
import argparse
import hashlib
import json
import pathlib
import subprocess
import time

p = argparse.ArgumentParser()
p.add_argument('--source', type=pathlib.Path, required=True)
p.add_argument('--run', type=pathlib.Path, required=True)
p.add_argument('--binary', type=pathlib.Path, required=True)
p.add_argument('--count', type=int, choices=[10, 179], required=True)
a = p.parse_args()
a.run.mkdir(parents=True, exist_ok=True)
inputs = json.loads((a.source / 'inputs.json').read_bytes())[:a.count]
expected = json.loads((a.source / 'baseline-predictions.json').read_bytes())[:a.count]
assert all(set(t) == {'speaker', 'utterance'} for c in inputs for t in c)
bundle = a.run.parent / 'bundle'
bundle.mkdir(exist_ok=True)
for name in ['model.json', 'schema.json']:
    destination = bundle / name
    if not destination.exists():
        destination.symlink_to(a.source / name)
manifest = dict(version=1, source='google-research-datasets/dstc8-schema-guided-dialogue@e852981ae34990f4358979625854259302feaa78; reused accepted glassmind model', license='CC-BY-SA-4.0', scope='provisional predictions, selected three-service SGD scope; no external actions')
for name in ['model', 'schema']:
    manifest[name + '_sha256'] = hashlib.sha256((a.source / (name + '.json')).read_bytes()).hexdigest()
(bundle / 'manifest.json').write_text(json.dumps(manifest, indent=2))
started = time.monotonic()
proc = subprocess.Popen([str(a.binary), str(bundle)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
commands = (a.run / 'commands.jsonl').open('w')
responses = (a.run / 'responses.jsonl').open('w')
latencies = []

def command(session, op, **kwargs):
    request = dict(session=session, command=dict(op=op, **kwargs))
    line = json.dumps(request, ensure_ascii=False)
    commands.write(line + '\n')
    start = time.monotonic()
    proc.stdin.write(line + '\n'); proc.stdin.flush()
    response = proc.stdout.readline()
    latencies.append(time.monotonic() - start)
    assert response, proc.poll()
    responses.write(response)
    return json.loads(response)

def ok(session, op, **kwargs):
    response = command(session, op, **kwargs)
    assert response['ok'], response
    return response['result']

user_count = restart_count = after_restart = system_boundaries = 0
prediction_bytes = 0
history_sizes = []
try:
    empty = ok('a', 'show')
    assert empty['provisional'] is True and empty['draft'] is None and empty['total_turns'] == empty['user_turns'] == 0
    assert empty == ok('a', 'show')
    for conversation, predictions in zip(inputs, expected, strict=True):
        assert ok('a', 'reset') == empty
        assert ok('b', 'reset') == empty
        boundaries = [i for i, t in enumerate(conversation[:-1]) if t['speaker'] == 'SYSTEM']
        midpoint = min(boundaries, key=lambda i: abs(i - len(conversation) // 2)) if boundaries else max(0, len(conversation) // 2 - 1)
        system_boundaries += int(bool(boundaries))
        prediction_iter = iter(predictions)
        users = 0
        for i, turn in enumerate(conversation):
            op = turn['speaker'].lower()
            snap = ok('a', op, utterance=turn['utterance'])
            resumed = ok('b', op, utterance=turn['utterance'])
            assert snap == resumed
            assert snap['provisional'] and snap['total_turns'] == i + 1
            users += int(op == 'user')
            assert snap['user_turns'] == users and snap['last_speaker'] == turn['speaker']
            if op == 'user':
                assert snap['draft'] == next(prediction_iter)
                user_count += 1
                after_restart += int(i > midpoint)
                prediction_bytes += len(json.dumps(snap['draft']).encode())
            assert ok('a', 'show') == snap
            if i == midpoint:
                h = ok('b', 'save')['history']
                assert set(h) == {'version', 'bundle_identity', 'turns'}
                assert h['turns'] == conversation[:i+1]
                history_sizes.append(len(json.dumps(h, ensure_ascii=False).encode()))
                ok('b', 'reset')
                assert ok('b', 'restore', history=h) == snap
                restart_count += 1
        assert next(prediction_iter, None) is None
    # Reversed conversation order through a single resettable session.
    reverse_users = 0
    for conversation, predictions in reversed(list(zip(inputs, expected, strict=True))):
        ok('reverse', 'reset'); prediction_iter = iter(predictions)
        for t in conversation:
            s = ok('reverse', t['speaker'].lower(), utterance=t['utterance'])
            if t['speaker'] == 'USER':
                assert s['draft'] == next(prediction_iter); reverse_users += 1
    # Different conversations interleaved against their independently frozen outputs.
    interleaved_users = 0
    for i in range(min(10, len(inputs))): ok(str(i), 'reset')
    prediction_iters = [iter(v) for v in expected[:10]]
    for turn_index in range(max(map(len, inputs[:10]))):
        for i, conv in enumerate(inputs[:10]):
            if turn_index < len(conv):
                t = conv[turn_index]
                s = ok(str(i), t['speaker'].lower(), utterance=t['utterance'])
                if t['speaker'] == 'USER':
                    assert s['draft'] == next(prediction_iters[i]); interleaved_users += 1
    # Restart at every boundary of the first conversation, including SYSTEM turns.
    every_boundary = 0
    ok('boundaries', 'reset')
    for t in inputs[0]:
        s = ok('boundaries', t['speaker'].lower(), utterance=t['utterance'])
        h = ok('boundaries', 'save')['history']
        ok('boundaries', 'reset'); assert ok('boundaries', 'restore', history=h) == s
        every_boundary += 1
    # Failures preserve the session, both predicted state and original input bytes.
    before = ok('a', 'show'); h = ok('a', 'save')['history']
    for bad in ['', ' \t', 'x\0y', '😀'*1024+'x']:
        assert not command('a', 'user', utterance=bad)['ok']
        assert ok('a', 'show') == before and ok('a', 'save')['history'] == h
    for bad in [dict(h, frames=[]), dict(h, version=2), dict(h, bundle_identity='bad'), dict(h, turns=[dict(speaker='USER',utterance='hi',annotations={})])]:
        assert not command('a', 'restore', history=bad)['ok']; assert ok('a','show') == before
    proc.stdin.close(); assert proc.wait(timeout=10) == 0, proc.stderr.read()
    report = dict(conversations=len(inputs), exact_user_predictions=user_count, midpoint_restarts=restart_count,
                  system_midpoints=system_boundaries, user_predictions_after_restart=after_restart,
                  reverse_order_predictions=reverse_users, interleaved_predictions=interleaved_users,
                  every_boundary_restarts=every_boundary, no_failed_mutations=True,
                  wall_seconds=time.monotonic()-started, example_requests=len(latencies),
                  request_median_ms=sorted(latencies)[len(latencies)//2]*1000,
                  request_max_ms=max(latencies)*1000, midpoint_history_max_bytes=max(history_sizes),
                  input_sha256=hashlib.sha256((a.source/'inputs.json').read_bytes()).hexdigest(),
                  model_sha256=manifest['model_sha256'], schema_sha256=manifest['schema_sha256'],
                  semantic_accuracy_measured=False)
    (a.run/'report.json').write_text(json.dumps(report,indent=2)); print(json.dumps(report,indent=2))
finally:
    if proc.poll() is None:
        proc.terminate()
        try: proc.wait(timeout=10)
        except subprocess.TimeoutExpired: proc.kill(); proc.wait()
    commands.close(); responses.close()
