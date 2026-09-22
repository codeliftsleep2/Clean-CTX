# COMPACT-A research boundary

**Status:** A1 production integration implemented; verification pending

## Candidate lineage

- **A0** is the compact-JSON/key-substitution control. Its cold-schema result is
  38.60% cl100k and 37.78% o200k reduction. It is reversible but below the
  economic target, so it will not receive model evaluation.
- **A1** is a correctness-complete successor to SCHEMA v5: typed scoped records,
  positional columns, occurrence markers, compact graph rows, and separately
  framed exact bodies. It must decode to normalized CONTROL-FULL v2 before token
  measurement or reasoning evaluation.

## A1 hard edit boundary

Bodies are never line-, delimiter-, or summary-decoded. Each body segment is
keyed by canonical method ID and carries exact start/end spans plus an explicit
UTF-8 byte length. The decoder reads that exact byte count, rejects truncation or
trailing corruption, and preserves CRLF, Unicode, whitespace, braces, and marker-
like text byte-for-byte. Focused Edit includes only the resolved canonical target
set; all-body Edit includes every available body. Semantic-only modes include no
body segment and must request Edit or Verbatim when source is required.

No A1 token result is valid until malformed/truncated body frames are rejected
deterministically and every decoded body/span equals the normalized oracle.

The focused A1 body-frame gate is green. It covers multiple canonical methods,
explicit byte lengths, exact spans, CRLF, Unicode, marker-like body contents,
truncation, and corrupt-length rejection. The next A1 gate is typed scoped
declarations and ordered occurrence groups; graph records follow only after that
roundtrip is exact.

The typed scoped declaration/occurrence gate is green. The next tracked stage
uses positional call and semantic-edge records while retaining occurrence order,
canonical caller IDs, written callee names, arity, spread, unresolved status,
typed endpoints, independent endpoint files, layer, and call evidence.

The scoped graph gate is green. The integrated A1 gate now composes the scoped
declaration and graph records with separately length-framed bodies, then
regenerates stable typed navigation metadata from decoded identities. Acceptance
requires equality with the complete normalized CONTROL-FULL v2 oracle, including
navigation, exact body bytes, and numeric spans.

The first bounded reasoning smoke exposed locality failures despite exact
roundtrip: DI occurrences and behavioral families were too distant from their
typed owners, while compact edge endpoints were too easy to collapse into one
provenance statement. A1 therefore carries a non-authoritative navigation index:
`N.D` groups ordered DI occurrences by canonical owner, and `N.V` groups every
behavior family by canonical method. Edge columns explicitly distinguish `S`
and `O`, including independent endpoint files. These entries repeat existing
facts for navigation only and add no semantic assertions.

COMPACT-A encodes the frozen CONTROL-FULL v2 semantic object with a versioned
schema and shorter field keys. Unknown fields are escaped rather than omitted,
so schema growth cannot silently lose facts.

Cold payloads carry the complete `A1` key legend. Token measurements must count
that legend on every response unless a future transport contract explicitly
guarantees schema reuse; decoder-only measurements are invalid for candidate
selection.

The decoded target is a normalized semantic object, not byte-identical JSON.
Equality therefore ignores object-key order and whitespace while preserving:

- array order, duplicates, and nested occurrence-group boundaries;
- typed IDs, ownership, unresolved-target status, and provenance;
- null versus present values;
- exact body strings byte-for-byte, including line endings and Unicode;
- numeric body start/end spans exactly.

The verified research decoder remains under `src/tests/ir/**`. The production
encoder now lives in `src/ir/compact_a.rs` and is assembled through the shared
MCP content boundary. Full provide/compress, delta baselines and post-apply
snapshots, restore, replay, and durable regenerated presentation use A1.
CONTROL-FULL-DELTA v2 remains the acknowledged-state delta contract.

Every complete candidate is compared locally with the byte-exact raw source.
No model or remote token-count API is called. cl100k/o200k use bundled exact
BPE counts; Claude uses the calibrated local cl100k approximation and must
clear the documented uncertainty on both sides. A tie, an unsafe approximate
margin, unsupported approximate tokenizer, or tokenizer initialization failure
selects byte-exact raw source with no A1 wrapper/footer.

Production verification is owned by the repository Final Verification Gate in
`docs/agent/verification.md`. The tracked suite now covers the production A1
renderer against the verified decoder, local exact/approximate economics,
strictly cheaper selection, byte-exact CRLF/Unicode raw fallback, focused Edit
bodies, full provide/compress, delta baseline/apply, and restore/replay. The
older capture/measurement scripts below remain research evidence and do not
replace the tracked gate.

The focused user-run test passed. This establishes codec reversibility for the
representative correctness-rich fixture and validates that normalization ignores
object presentation while retaining ordered arrays, exact bodies, and spans. It
does not yet establish roundtrip coverage across the full corpus or production
lifecycle modes.

The next deterministic matrix is now represented by tracked cases for Low,
Medium, High, all-body Edit, focused-body Edit, delta-shaped state,
serialization/restore/replay, schema growth, CRLF/Unicode body bytes, and exact
spans. These remain zero-model-call tests. Real multi-language corpus capture and
token measurement follow only after this matrix is green.

After the deterministic matrix passes, measure the candidate locally with no
model calls:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Measure-CompactA.ps1
```

The script writes per-capture cl100k/o200k comparisons to
`target/context-compression-verification/captures/compact-a1-token-records.json`.
These screening counts decide whether any bounded model smoke is economically
justified; they do not select the codec for production.

After measurement, validate every generated capture without model calls:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Verify-CompactA1.ps1
```

Only after that passes, run the six-case, twelve-invocation high-risk smoke:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Run-CompactA1Smoke.ps1
```

The smoke is resumable and intentionally excludes the remaining 30 baseline
cases. Use `-Restart` only to discard and rerun its six saved results.

After a presentation-only locality repair, rerun only the three affected
families (six model invocations including scoring):

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Run-CompactA1RepairSmoke.ps1 -Restart
```

`-Restart` clears only that result lane's saved prompt/answer files. This is
required after changing a candidate payload; otherwise a resumable run would
correctly reuse the previous answer rather than evaluate the new presentation.

## Production edge-case matrix

The tracked heavy deterministic pass compiles real TypeScript/Angular and C#
source through the production compiler before A1 encoding. It covers multiple
bound arrows, nested RxJS/object callbacks, microtask callbacks, spread,
unresolved callees, Angular constructor DI/meta edges, C# overload identity,
nested types, multiple lambdas, ASP.NET route/action meta edges, and exact Edit
bodies/spans. Java and Spring
remain explicitly deferred from this heavier pass; their existing independent
tests still apply, but they are not evidence of A1 production-edge coverage.
