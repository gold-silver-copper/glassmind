# Check a relationship in a local facts bundle

With a facts bundle enabled, `talk` accepts an explicit check:

```sh
./target/release/glassmind talk --facts-dir scratch/facts-bundle
```

```text
Check fact: Was Q100303 born in Q2533654?
```

Use full indexed names or explicit Wikidata IDs. Supported question forms are:

- `Was SUBJECT born in OBJECT?`
- `Did SUBJECT die in OBJECT?`
- `Is SUBJECT a citizen of OBJECT?` (also `Was`)
- `Does SUBJECT work for OBJECT?` (also `Did`)

A supported reply identifies the exact directed relationship in the loaded records.
JSON output (`talk --json`) includes `kind: fact_verification`, a typed `status`,
the subject/property/object witness, endpoint resolution, and bundle provenance.
The standalone command is also available:

```sh
./target/release/glassmind-facts verify --index scratch/facts-bundle/index.json \
  'Was Q100303 born in Q2533654?'
```

`Supported` means that the edge exists in this snapshot. The index does not retain
statement dates, qualifiers or ranks, so the reply does not establish current
citizenship, employment or exhaustive world truth. An absent relationship or
missing/ambiguous name is `Unknown`, never a negative factual conclusion.
Unsupported syntax or competing interpretations produces `Refused`.

The parser resolves complete endpoint names before inspecting edges. Names may
contain predicate words such as “born in”; two viable interpretations still refuse,
even when only one relationship exists. Display labels cannot supply aliases.
Questions are bounded to4,096 UTF-8 bytes and endpoints to256 bytes. Negation,
conjunctions, conditions, temporal modifiers and additional actions remain outside
this deliberately limited grammar; reserved words can also exclude valid names.

Checks require explicit names and do not bind “he,” “she” or “it” to earlier facts.
They preserve stored personal preferences and story state. Without `--facts-dir`,
the explicit command explains that a bundle is required. Ordinary question routing
and the existing `List answers to:` command retain their behavior.

Independent authored checks match47/48 original status/witness expectations,
including all18 positive witnesses. One compound subject returns Refused where the
original fixture expected Unknown. The full ordinary2,821-question QA output remains
byte-identical; this feature is additional explicit verification coverage, not an
improvement in that benchmark. See the [review and original results](../tools/facts/verification/PARENT.md)
for the grammar revision, retained disagreements, provenance and validation.
