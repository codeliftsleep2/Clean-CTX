# COMPACT-A research boundary

**Status:** A2 rejected by raw-source economics; A3 sparse positional repair approved

The current implementation must not proceed to model smoke or production
acceptance. A2's legend is only 146 cl100k / 147 o200k tokens; its inflation is
caused by incomplete nested positional encoding, fixed empty columns, repeated
call defaults, and fidelity-insensitive layouts. See `PHASED_A2_PLAN.md` for the
closed finding and `PHASED_A3_PLAN.md` for the approved successor gates.
`A3_GRAMMAR.md` is the Phase 0 wire and fidelity specification; it is not yet a
production contract.
`A3_PHASE0_BASELINE.md` records the reproduced A2 anatomy and the evidence for
moving to the A3 research encoder/decoder.

## Candidate lineage

- **A0** is the compact-JSON/key-substitution control. Its cold-schema result is
  38.60% cl100k and 37.78% o200k reduction. It is reversible but below the
  economic target, so it will not receive model evaluation.
- **A1** is a correctness-complete successor to SCHEMA v5: typed scoped records,
  positional columns, occurrence markers, compact graph rows, and separately
  framed exact bodies. It must decode to normalized CONTROL-FULL v2 before token
  measurement or reasoning evaluation. CONTROL-FULL is the semantic oracle,
  not the production cost denominator. Capture-time byte-exact raw source is
  the production cost control.

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
snapshots, restore, replay, and durable regenerated presentation historically
used A1. A2 uses the file-local `FILE-CONTEXT-DELTA v1` acknowledged-state
contract; workspace graph state is queried separately after apply when needed.

That historical A1 boundary is under correction because it embedded the full
workspace semantic-edge snapshot in every file response. The approved A2
boundary limits file content to fidelity-appropriate file structure, canonical
local calls/injections, and required exact bodies. Forward/reverse dependency,
multi-hop, framework, and cross-file provenance facts remain authoritative in
the workspace index and are requested through `workspace_query`. Persistence
and auxiliary state may retain them; model-visible file content must not copy
them automatically.

Every complete candidate is compared locally with the byte-exact raw source.
No model or remote token-count API is called. cl100k/o200k use bundled exact
BPE counts; Claude uses the calibrated local cl100k approximation and must
clear the documented uncertainty on both sides. A tie, an unsafe approximate
margin, unsupported approximate tokenizer, or tokenizer initialization failure
selects byte-exact raw source with no A1 wrapper/footer. Focused Edit is the
sole exception: a full raw document is not semantically equivalent because it
exposes every method body. Focused Edit therefore returns the complete focused
A1 projection with exact selected bodies and skeletons for all other methods.

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
`target/context-compression-verification/captures/compact-a2-token-records.json`.
Its primary comparison is capture-time raw source versus A2. CONTROL-FULL-to-A2
numbers are retained only as oracle-encoding diagnostics. These screening
counts decide whether any bounded model smoke is economically justified.
Each record also reports independent token anatomy for the legend, file/mode,
declarations, local calls, navigation, imports, aliases, and body frames. These
component counts diagnose representation cost and are not expected to sum
exactly to the whole payload because tokenizer merges cross component bounds.

After measurement, validate every generated capture without model calls:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Verify-CompactA1.ps1
```

Only after that passes, run the six-case, twelve-invocation high-risk smoke:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Run-CompactA1Smoke.ps1
```

The production comparison is a three-task paired A1-versus-raw smoke (six
cases, twelve model invocations including scoring). It refuses to spend model
tokens unless A1 first clears the raw-source economic screen:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Run-CompactA1VsRawSmoke.ps1 -Restart
```

The paired questions deliberately avoid requiring canonical IDs that do not
exist in raw source. They compare ownership/overloads, DI honesty, and ordered
call/spread reasoning using equivalent semantic expectations.

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
