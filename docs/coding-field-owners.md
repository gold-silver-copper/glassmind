# Owner-specific field renaming

A new conservative route handles explicitly owner-qualified field renames when
multiple structs share the old spelling. The existing unique-spelling route is
unchanged. Bare ambiguous field requests continue to ask which field was meant.

The owner description must resolve to exactly one struct in one source/package.
The package graph supplies lexical module identity; the field declaration is
checked against its indexed owner, source file and line. `rename_fields.rs`
collects two position sets: all classified old-name tokens, and just those owned
by the selected struct. An edit is proposed only when the first set equals the
index's complete old-name token set. Only the second set is edited.

The first supported subset is deliberately small: non-generic named structs,
direct struct literals, field access on direct literals or explicitly typed free
function parameters (including references), and ordinary nested modules. Comments
and strings are untouched. Types are resolved from syntax and the package graph;
there is no nominal inference from variable names or function return types.

Imports, aliases, generics, impls/traits, closures, parameter shadowing, inferred
local receivers, call-result receivers, unclassified old-name tokens, shorthand,
old-name destructuring, raw-name variants, collisions, unsupported attributes,
and unknown macros refuse. Plain standard output macros can remain opaque only
when they contain no old-name identifiers or nested macros. The shared package
loader also refuses unsupported topology. This does not extend the native
verifier, snapshot checks, rollback behavior, or general Rust name resolution.

Frozen authored development contracts improve from 12/16 to 16/16: four newly
verified edits, twelve unchanged refusals. The first-three sample improves from
0/3 to 3/3. Every fixture compiled and executed its unchanged nonempty behavior
test before and after the request, with stdout markers. Successful patches are
byte-for-byte equal to independently specified expected source; separate values
check the target and homonymous other owner. Four supplemental safety probes
remain refusals. These small authored results are not the parent full765 result.

See [the handoff](../tools/coding-field-owners/HANDOFF.md) for provenance, exact
commands, hashes, rejected experiments and unchanged inherited failures.
