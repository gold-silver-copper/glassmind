#!/bin/sh
# Run inside the recorded bounded scope, with candidate pinned beforehand.
set -eu
run=$1
binary=$run/candidate
if [ "${2:-all}" = all ]; then
python tools/coding-sequence-composition/run.py candidate "$binary" "$run/native" > "$run/native.log" 2>&1
python tools/coding-word-initials/run.py candidate "$binary" "$run/old-initials" > "$run/old-initials.log" 2>&1
python tools/coding-sequence-composition/run.py old-native "$binary" "$run/old-native" > "$run/old-native.log" 2>&1
fi
python tools/coding-implementation-grammar/parent/native.py --binary "$binary" --out "$run/independent" > "$run/independent.log" 2>&1
python tools/coding-implementation-grammar/parent/native.py --binary "$binary" --out "$run/unset-target" --unset-target > "$run/unset-target.log" 2>&1
python tools/coding-implementation-grammar/correction/extra-review.py --binary "$binary" --out "$run/extra" > "$run/extra.log" 2>&1
python tools/coding-sequence-composition/run.py train-sample "$binary" "$run/train-sample" > "$run/train-sample.log" 2>&1
python tools/coding-sequence-composition/run.py train-full "$binary" "$run/train-full" > "$run/train-full.log" 2>&1
python tools/coding-word-initials/verify.py "$run" native > "$run/native-verify.log" 2>&1
python tools/coding-word-initials/verify.py "$run" train-full > "$run/train-verify.log" 2>&1
