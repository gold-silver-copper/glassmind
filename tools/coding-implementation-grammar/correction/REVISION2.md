# Restore a bare identifier named function

Independent source review and an authored CLI fixture found that `implement
function` refuses a valid function literally named `function`. The imperative
parser consumes that identifier as an optional kind word before target binding.
The frozen accepted grammar permits a bare Rust identifier here and does not
include this extra optional kind. Existing declaration syntax already allows
an explicit `function` after its target and is unaffected.

Remove the optional kind consumption from imperative parsing. Add contrasts for
bare/quoted/polite/location forms, bound declaration syntax, and refusal of the
out-of-grammar two-identifier form. Keep the six prior declaration refusals,
all original accepted forms, and the native generator unchanged. Run native
implementation of the same authored function with the supported bare variants.

The prior revision's full765 run was interrupted after this independent defect
was reproduced, before using a full DEV result to select a patch. Its original
source, binary, logs and stop record are preserved in revision1/manifest.json.
All required routing/native/TRAIN/full765 gates remain unchanged and must pass
again for the revised binary. Do not relabel the interrupted run as a pass.
