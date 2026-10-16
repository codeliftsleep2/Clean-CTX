# Scoring and future execution protocol

## Primary metric

The denominator unit is one unique semantic relationship pair per relation.
Repeated call sites supporting the same pair are attached as evidence and count
once.

```text
recall(relation) = true_positive_expected_pairs / total_expected_pairs_for_relation
```

All four relations must independently satisfy `recall >= 0.90`:

- Dispatches
- Selects
- HandlesAction
- TriggersReducer

There is no macro-average rescue. Call-site recall may be reported as a
secondary diagnostic only.

## Identity normalization

A match requires the same relation plus the same normalized source and target:

```text
(relation, source.domain, source.kind, source.name, source.file,
 target.domain, target.kind, target.name, target.file)
```

Paths are repository-relative, use `/`, remove a leading `./`, and are compared
case-sensitively because the pinned Git tree is authoritative. Names and kinds
are exact and case-sensitive. Files identify declarations, not occurrence
sites. No alias, basename, same-spelling declaration, reverse edge, or
occurrence endpoint can substitute for the declared entity.

For `HandlesAction`, the source identity is the named `createEffect` property,
not its enclosing class. For `TriggersReducer`, the target identity is the
variable assigned the authoritative `createReducer(...)` call (`reducer` in
this corpus), qualified by its declaration file.

## Ambiguity and exclusions

Every discovered candidate is classified as `EXPECTED`,
`EXCLUDED_UNSUPPORTED`, `EXCLUDED_AMBIGUOUS`, or `REJECTED_NOT_SEMANTIC`.
Only `EXPECTED` contributes to recall. Exclusion decisions are frozen before
scoring and may not be changed because Clean-CTX misses a pair.

## True positives and false negatives

A true positive is an observed public relationship matching one EXPECTED tuple
exactly. Each missing EXPECTED tuple is a false negative. Evidence must report
the expected pair, source evidence, nearest observed evidence if any, and a
diagnostic classification. Diagnosis does not authorize implementation.

## False positives and precision

Capture every observed in-scope pair not in EXPECTED and manually classify it
as `TRUE_FALSE_POSITIVE`, `ORACLE_OMISSION`, `OUT_OF_SCOPE_RELATION`, or
`AMBIGUOUS`. Report diagnostic precision:

```text
precision = true_positive_expected_pairs / scored_observed_pairs
```

There is no precision acceptance threshold in v1, but pathological false
positives must be prominent. A genuine oracle omission invalidates the scored
run. It requires a reviewed new benchmark version; the frozen oracle must not
be edited in place during scoring.

## Separately authorized execution protocol

Do not execute this protocol as part of benchmark design.

1. Record the exact Clean-CTX commit, benchmark version, and oracle SHA-256.
2. Fetch the corpus from its canonical Git URL and detach exactly at
   `3d95daa95b5a53798f33b390815bdccad733995e`; reject a dirty checkout.
3. Mechanically verify the frozen scope and the expected count of 79 included
   non-test TypeScript files.
4. Start with a newly created empty evidence directory and fresh Clean-CTX
   workspace/index/cache state dedicated to this run. Do not reuse warm state.
5. Disable CBM and all optional semantic providers. Provider-disabled behavior
   is authoritative; any later CBM-enabled comparison is non-authoritative.
6. Use the registered public MCP interface only. Do not inspect private index,
   compiler, graph, debug, or test-only structures for scoring.
7. Use the repository-development high-fidelity public context/query path.
   Register or hydrate files in deterministic repository-relative lexical
   order, then query expected identities in oracle ID order: Dispatches,
   Selects, HandlesAction, TriggersReducer.
8. Capture every JSON-RPC request/response byte-for-byte and produce normalized
   observed-pair JSON without consulting private state.
9. Compare frozen EXPECTED tuples with observed tuples according to this file.
10. Write evidence beneath
    `target/ngrx-recall-benchmark/<timestamp>-<clean-ctx-short-sha>/` containing
    `provenance.json`, `requests.jsonl`, `responses.jsonl`, `observed.json`,
    `score.json`, `false-negatives.json`, and `false-positive-review.json`.

`provenance.json` must record corpus SHA, Clean-CTX SHA, benchmark version,
oracle checksum, scope, fidelity, provider-disabled state, fresh-state method,
query order, tool names, binary SHA-256, UTC timestamps, and host/platform.

If equivalent query orders produce different results, preserve both runs and
report a separate lifecycle defect. Never choose the passing order after
observing results.
