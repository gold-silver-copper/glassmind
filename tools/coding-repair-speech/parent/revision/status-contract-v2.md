# Explicit status polarity clarification (version 2)

The frozen 30 input cases and all native mutation/repair assertions are unchanged.
Version1 required Question::Passes for every status question, including "Are there
any compiler errors?". That proxy is semantically incorrect for error presence.
The public Question::Problems(Build) type represents this question. world-rust's
existing dispatch at src/lib.rs:839 executes the same check for Problems/Passes,
returning diagnostic facts instead of an inverted Boolean. A valid answer does
not grant permission to change source.

Version2 requires exactly Ask(Problems(Build)) for status_2. The other five status
cases still require exactly Ask(Passes(the original check)). No broader alternatives
are accepted. Apply both versions to every original baseline/candidate and revised
retained output, report both scores, and preserve original files and grades. All
negative Command-preview, no-unrequested-mutation, exact-positive-repair and native
assertions are identical. The original candidate remains rejected by four lost
positives regardless of this clarification. Inputs were frozen before candidate
inspection; this note is a separate post-review correction of a type proxy, not an
independent preregistered metric. Full765's scoring is untouched.
