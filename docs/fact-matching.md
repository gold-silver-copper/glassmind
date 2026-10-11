# Match facts across conditions

With a local facts bundle loaded, ask for subjects sharing two or three positive
relationships:

```text
Match facts: Who was born in Q220 and died in Q90?
Match facts: Who was born in Harbor City and is a citizen of Maple Republic?
```

The first example returns Q1029284 in the demonstrated snapshot. The second uses
illustrative names; aliases must exist unambiguously in your loaded index.
Start conversation with `glassmind talk --facts-dir PATH`. This command does not
change personal memory or story facts and does not infer names from earlier turns.

The standalone command needs only an index:

```sh
glassmind-facts match --index index.json 'Who was born in Q220 and died in Q90?'
```

Use `Who CLAUSE and CLAUSE?`, optionally adding a third clause. Supported clauses
are `was born in OBJECT`, `died in OBJECT`, `is a citizen of OBJECT`,
`was a citizen of OBJECT`, `works for OBJECT`, and `worked for OBJECT`. Keywords
ignore ASCII case; internal spaces follow these forms. Objects are full aliases
or canonical QIDs. Aliases containing the reserved word `and` are unsupported.
Negation, alternatives, conditionals, current-time claims and appended instructions
are unsupported. Limits are 4096 bytes per question and 256 per endpoint.

Every subject must have every directed edge. Results are sorted by ID and show
up to ten subjects, with the full local count and a truncation flag. Each returned
subject has a witness for every distinct constraint: subject, property, object
and a source link. Reordering or repeating a condition does not create matches.
The standalone JSON includes the index hash; conversation adds bundle provenance.

An unknown or ambiguous alias is distinct from a valid query with no matches.
An empty result means no matching subject in the loaded store. The snapshot can
be incomplete, stale or wrong; recorded edges do not verify dates or current
status. The demonstrated 500-entity snapshot was seeded by validation subjects,
so its examples are not independent retrieval benchmarks. Ordinary facts scores
remain unchanged. [Independent validation and limitations](../tools/facts/matching/PARENT.md).
