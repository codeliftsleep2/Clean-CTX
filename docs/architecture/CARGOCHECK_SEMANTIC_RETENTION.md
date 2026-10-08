# CargoCheck semantic retention and omission disclosure

This phase follows incremental parsing commit `060935c`. It implements the
owner-approved retention and nested selection policies without changing numeric
budgets, process ownership, capture, invocation, or environment policy.

## Online diagnostic selection

The compiler maintains at most 64 error candidates and 64 warning candidates.
Each severity pool keeps its first 48 distinct admitted candidates and rotates
its last 16. Final selection consumes errors first, then warnings within the
remaining part of the 64-diagnostic quota; each reduced quota uses its own 3:1
head/tail ratio. Selected items preserve producer order within each severity.
Notes/help remain child diagnostics rather than top-level competitors. Other
top-level records are counted and disclosed as omitted.

All valid compiler-message records continue contributing to parser and severity
counts after retention fills. `peak_candidates` measures the resident candidate
collection, not transient deserialization of the current bounded frame. This
phase bounds candidate and nested object counts; the later result-budget phase
must enforce the approved 512 KiB serialized result limit and trim large strings.
It does not introduce an unlimited identity index or code/file histogram.

## Exact-repeat truth

Repeat matching uses a private pre-redaction diagnostic record, excluding
parent/child rendered presentation. All locations, coordinates, messages, child
semantics, and optional fields remain part of that conservative equality check.
Unknown metadata can prevent compaction; it cannot fabricate a repeat. Private
identity data stays inside the producer boundary and is never serialized,
logged, or spooled. Sanitized display collisions are not exact-repeat evidence.

Repeat matching is scoped to currently retained candidates. When a candidate is
evicted, its identity and occurrence count are discarded. A later recurrence may
be admitted again; the result does not claim global deduplication. Compaction
facts count only repeated occurrences represented by final retained diagnostics.
All unrepresented occurrences, including repeats of evicted diagnostics and
lower-priority candidates omitted during final selection, count as omissions.

```text
diagnostics_seen = diagnostics_retained + exact_repeats_collapsed
                 + diagnostics_omitted
diagnostics_omitted = errors_omitted + warnings_omitted + other_omitted
```

Retained severity counts measure distinct result entries. Omitted severity
counts measure producer occurrences. Selection/repeat scope, actual quotas used,
head/tail counts, and peak candidate count are explicit result facts.

## Nested selection and disclosure

Primary and related spans retain the first four and eight respectively in
producer order. Children retain the first six and last two; retained children
keep up to eight spans, matching the existing related-span detail budget.
Each retained child discloses original span count and omitted spans. Diagnostic
facts disclose primary/related span totals and omissions, original child count,
child omissions, and the actual child head/tail split.

Suggestions are selected from eligible retained spans, including retained child
spans. Machine-applicable edits take priority, followed by producer order. A
distinct edit is identified by source file, complete line/column range,
replacement, and applicability before redaction. At most four distinct edits are
selected; duplicate occurrences on retained spans share an edit slot. Occurrence
counts and distinct edit counts are separate facts.

Suggestion totals and replacement-byte totals cover every original parent and
child span, including omitted spans and omitted children. Retained/omitted
occurrence counts and omitted replacement bytes therefore describe the complete
validated record. Machine-applicable seen/retained counts disclose whether that
priority could be satisfied within the independent span and child budgets.
All strings in retained diagnostic and child details remain sanitized.

## Verification

Five tracked `retention_regression_` tests were observed failing at their intended
assertions before production edits. The tests and module registration were
stashed, then restored unchanged for GREEN. They cover column-sensitive identity,
pre-redaction identity, bounded resident candidates, suggestion omissions on
discarded spans, and machine-applicable suggestion priority.

Eight additional tracked cases cover mixed severities, warning quota head/tail,
evicted-repeat accounting, presentation-independent identity, child identity,
nested omission totals/bytes, distinct edit slots, and zero retention capacity.

Owner-authorized Linux results:

- RED: 0 passed, 5 failed, 3,531 filtered out.
- Unchanged GREEN: 5 passed, 0 failed, 3,531 filtered out.
- Combined CargoCheck checks: 63 passed, 0 failed, 0 ignored, 3,481 filtered out.
- Package all-target/all-feature Clippy with `-D warnings`: passed, exit 0.

Windows verification for this phase remains owner-run using:

```text
cargo test --locked -p clean-ctx --lib --all-features -j 4 diagnostics::cargo_check::
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
```

No full test suite or Claude pilot was run. The full suite remains exclusively
the CI gate.

## Next boundary

Sanitized fallback evidence hardening is implemented in the subsequent
`CARGOCHECK_FALLBACK_EVIDENCE.md` phase. Final result budgets are implemented in `CARGOCHECK_RESULT_BUDGETS.md`. CLI
projection/exit codes, typed MCP integration, production lifecycle verification,
and the final bundled Claude pilot remain later work.
