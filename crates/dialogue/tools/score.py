#!/usr/bin/env python3
"""Call pinned official SGD metrics, including fuzzy noncategorical matching.

Gold is read only here and in the adapter's post-prediction scoring path.
The input predictions must cover every user frame of the declared slice/sample.
"""
import argparse
import json
import warnings
with warnings.catch_warnings():
    warnings.simplefilter("ignore")
    import upstream_metrics as metrics

p = argparse.ArgumentParser()
p.add_argument("--data", required=True)
p.add_argument("--schema", required=True)
p.add_argument("--predictions", required=True)
p.add_argument("--limit", type=int)
p.add_argument("--output", required=True)
a = p.parse_args()
schema, = json.load(open(a.schema))
dialogues = sorted(json.load(open(a.data)), key=lambda d: d["dialogue_id"])
if a.limit is not None:
    dialogues = dialogues[:a.limit]
preds = json.load(open(a.predictions))
predictions = {(p["dialogue_id"], p["turn_index"]): p for p in preds}
assert len(predictions) == len(preds), "duplicate predictions"
totals = {}
frames = 0
intents = 0
strict = 0
for d in dialogues:
    assert d["services"] == [schema["service_name"]]
    for index, t in enumerate(d["turns"]):
        if t["speaker"] != "USER":
            continue
        frame, = t["frames"]
        prediction = predictions.pop((d["dialogue_id"], index))
        assert prediction["service"] == schema["service_name"]
        frames += 1
        intents += metrics.get_active_intent_accuracy(frame, prediction)
        scores = metrics.get_average_and_joint_goal_accuracy(frame, prediction, schema, True)
        strict += metrics.get_average_and_joint_goal_accuracy(frame, prediction, schema, False)["joint_goal_accuracy"]
        for key, value in scores.items():
            if value != metrics.NAN_VAL:
                totals.setdefault(key, []).append(float(value))
assert not predictions, "unexpected predictions"
report = {"service":schema["service_name"], "scope":"complete single-service dev slice" if a.limit is None else "fixed first sorted dialogue IDs",
          "dialogues":len(dialogues), "frames":frames, "active_intent_correct":int(intents),
          "active_intent_accuracy":intents/frames, "strict_joint_correct":int(strict),
          "strict_joint_accuracy":float(strict/frames),
          "metrics":{k:{"sum":sum(v),"count":len(v),"mean":sum(v)/len(v)} for k,v in totals.items()}}
with open(a.output,"w") as f:
    json.dump(report,f,indent=2)
    f.write("\n")
print(json.dumps(report,indent=2))
