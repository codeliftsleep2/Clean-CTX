# B2 subsystem migration and lossless retrieval

Agent: **Agent MaxHeadRoom**. Work started 2026-11-11; delivery 2026-11-12.
Baseline: B0/B1 commit `9400d476`. This is implementation and source-migration
evidence, not a second normative owner or independent certification.

## Delivered boundaries

[B2_NODES.csv](B2_NODES.csv) records 51 subsystem declarations: 17 authorities
and 34 existing invariants. [B2_RELATIONSHIPS.csv](B2_RELATIONSHIPS.csv) records
44 new source-backed relationships. The migration-record declaration below adds
one node and two preservation edges separately. All 36 baseline catalog nodes
and their 15 relationships are retained: the expected expanded graph is 88 nodes
and 61 relationships. Actual admitted membership, revisions, source hashes and
generation are recorded in [B2_ADMISSION.csv](B2_ADMISSION.csv), from read-only
inspection of the document database after production reconciliation. Production
query receipts remain distinct from this membership inspection.

The 15 focused owners under [contracts](../contracts/) govern persistence and
recovery, scoped durable compatibility, occurrence-aware deltas, exact mutation,
context batches and response channels, workspace batches, cycle witnesses,
model-facing presentation, token economics, cache hints, trusted roots, network
effects, proxy transforms and secret scrubbing. The compact semantic composition
contract and operational tooling guide remain cohesive in place.

The current invariant families admitted here cover persistence/publication,
workspace queries, optional CBM lifecycle and shared cache identity, delta
lifecycle, fidelity/currentness, provider health versus authority, and MCP/edit
contracts. Angular, other language/domain rules and historical debt are not
silently admitted with those families. A returned family source can contain
unadmitted sections; complete source text does not confer graph standing on them.

[B2_PROVENANCE.csv](B2_PROVENANCE.csv) maps each declaration or extracted section
to its original path, commit and available byte hashes. Original headings remain
as navigation. Replaced sections have one current owner, not a retained normative
copy. Shipped grammar examples preserve their original pre/post wording; B4
marker optimization and Tier C acknowledged-schema transport remain deferred.
The two economics clauses have one separate owner and are linked from the
presentation contract. The enclosing original section hash is a reconstruction
witness, not a claim that the split section remains contiguous in one file.

Whole-source byte hashes capture the original Windows worktree, which includes
mixed LF/CRLF files. The additional Git-blob hashes identify normalized committed
sources for reproducibility. Static comparison verifies each extracted section's
recorded hash against its original LF or CRLF section bytes, every relocated
clause (including the separate economics owner), all activated invariant bodies,
and all 14 pre-existing declarations in modified source files. Unmodified sources
retain their existing declarations. No encoding conversion hides a content change.

Source enforcement vocabulary and cited tests remain unchanged. IMPLEMENTED
means the source-described operative contract; no node is newly FROZEN,
VALIDATED, RECLOSED or BANKED. Historical operator/maintainer results retain
their attribution. The migration does not execute their witnesses again.

## Production preflight and source reconciliation

The seven B1 starting nodes were retrieved before authoring through actual
production `pcg_documents_v1` stdio. All were ready, current and complete with no
diagnostics at generation
`3390705a9314fe78f1e99c09a16bdec7180773c362d8416541159793d7390458`.
Applicable constraints included canonical sequence/multiplicity, physical 0x04
identity, distinct model projection, source-qualified refinement and evidence
versus closure. Typed preservation paths were inspected as constraints, never
execution permission.

The same ignored actual-root profile, document database, launch directory and
finite limits are retained. No new profile, corpus, ingestion mechanism, SQLite
schema or traversal was introduced. The profile SHA-256 remains
`0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9`.
Token budget is 65,536; representative requests use maxNodes 128/maxEdges 512,
inside profile maxima 512/2,048. Accepted enrolled-evidence stores, user root
configuration and six pre-existing untracked PCG reports are excluded.

[B2_CONTEXTS.csv](B2_CONTEXTS.csv) records actual final-production downstream
queries, exact final MCP-result token costs, returned identities, completeness,
diagnostics and generation. The contexts cover persistence/serialization,
semantic completeness/history, workspace batch/cycle behavior, optional CBM and
project cache ownership, MCP fidelity/channels, projection/economics, trusted
roots and proxy/security boundaries. Source dependencies remain present even
when expensive. Any incomplete result is recorded as such, never credited as a
checked prerequisite chain. The receipt CSV is outside Markdown ingestion so
recording the final generation does not create a self-referential source hash.

Baseline node IDs, declarations and relationship endpoints/kinds remain intact.
Shared family sources gain declarations, so their source hashes and node
revisions legitimately advance. The final comparisons distinguish that
source-normalization change from the earlier same-generation transport test.
No controlling canonical invariant was weakened to fit a context budget.

An initial expanded-session attempt returned the first context, then correctly
refused a subsequent query with `document_source_changed` while the assessment
README navigation was still being edited. That interrupted capture is retained
separately under `target/pcg-b2/expanded`; it is not final-source verification.
The final session runs after all Markdown authoring is finished.

## Compatible response projection

The public contract is [explicit lossless context transport](../PCG_DOCUMENT_AUTHORING.md#explicit-lossless-context-transport).
Default and explicit `inline_v1` retain version 1; only explicit context-only
`source_refs_v1` selects version 2. There is no automatic format switch. Unknown
or null format and requesting references for history fail as invalid parameters.

Each response pools exact `(path, sha256)` identities with exact text equality
on reuse. Different paths remain distinct even when bytes agree. Node revisions,
declarations, prior revisions, relationship provenance, witness paths and evidence
associations remain individual. Repeated artifact string payloads use the same
pool; absent/null text is preserved. No lossy summary or omission is introduced.

Production lifecycle: registered tools/call handler validates negotiation;
Session query performs the unchanged source capture, whole-candidate validation,
SQLite publication and traversal; store revisions retain full original sources;
the response-local projector lifts payload strings; the encoder serializes and
meters the final MCP result; overflow clears the pool, documents and context
paths/relationships together; COMPLETION verifies observed source boundaries;
the transaction commits; `send_pcg_result` emits those exact serialized bytes.
The temporary pool has no session/persistent owner and drops after serialization.
History remains separate and unchanged. Source and database authority are not
redefined by transport identities.

AF-PERF-001 scope: one context response. Expensive complete-set work is the
existing traversal/source construction, followed by one exact pool construction.
Each distinct payload identity is validated/constructed once; repeated identities
are compared exactly. Cost is O(payload bytes + occurrences × log(unique
identities)); there is no cross-operation cache or invalidation machinery.
Tracked deterministic coverage checks one pool construction for 1, 8 and 64
occurrences, alongside path/hash separation and conflict rejection. This is
bounded transport work coverage, not a global semantic-producer certification.

AF-PERF-002 disposition: **FIX** for demonstrated repeated transport payloads.
The benefit is measured below, including final JSON/reference overhead. Existing
source construction costs remain; no latency, allocation or total-operation
speedup is claimed. One small projection module and explicit negotiation are
proportionate to the measured reductions. No persistent state or schema migration
is required. Single-source regressions in token cost are retained honestly.

## Same-generation B1 comparison

[B2_RETRIEVAL_COMPARISON.csv](B2_RETRIEVAL_COMPARISON.csv) is generated by the
independent [production consumer](../../../verification/pcg-175/compare-document-source-responses.py).
All fourteen responses use the unchanged B1 generation above, same tokenizer
`o200k`, same profile and budgets. These are exact serialized MCP **result**
counts, including inner JSON escaping and the content wrapper; transport RPC ID
framing is outside the established `serialized_mcp_result_v1` accounting boundary.

| Starting node | Inline | Source refs | Tokens saved | Saving |
| --- | ---: | ---: | ---: | ---: |
| AUTH.CANONICAL_IR | 34,087 | 29,241 | 4,846 | 14.217% |
| AUTH.BINARY_V04 | 23,210 | 18,755 | 4,455 | 19.194% |
| AUTH.QUESTION_MATRIX | 8,350 | 8,643 | -293 | -3.509% |
| AUTH.LAYER_CERTIFICATION | 23,585 | 19,552 | 4,033 | 17.100% |
| WIRE-001 | 3,466 | 3,622 | -156 | -4.501% |
| AUTH-001 | 5,126 | 5,271 | -145 | -2.829% |
| INV.C22 | 2,495 | 2,642 | -147 | -5.892% |

The first B1 campaign used the earlier generation beginning `2b0f0e89` and
reported four more tokens per query (for example 34,091). The current inline
comparison uses the final B1 generation beginning `3390705a`; it is byte-identical
to this task's pre-edit production binary capture. No four-token difference is
credited to deduplication.

For all seven pairs, independent expansion equals the entire legacy JSON value:
every node, revision, declaration, edge, path, exact source and artifact
association. Path/hash attribution and source SHA-256 were verified. The new
binary's default result also matches the pre-edit binary's exact result bytes.
Repeated source contexts contain 8/6, 4/3 and 5/3 nodes/unique sources; contexts
without repeated sources pay explicit reference metadata overhead. The finite
ceiling was not raised, and no dependency was pruned to manufacture savings.

## Verification categories and limits

Tracked Rust regression: the formatted
`src/tests/mcp/pcg_document_sources.rs`, SHA-256
`98e7a3c4b115f18b1bcfc7a90537fb447f25a69ebf8ae86d1adfc9af9dc35e40`,
first failed against the original implementation at the intended invalid-request
boundary. It was stashed, the implementation restored, and the exact unchanged
test restored before GREEN. An earlier formatting-altered attempt was discarded
as the final RED/GREEN pair; the corrected pair is the evidence used here.

Focused all-feature Rust commands passed:

```powershell
cargo test --package clean-ctx --lib --all-features negotiated_sources_reconstruct_legacy
cargo test --package clean-ctx --lib --all-features documents::
cargo test --package clean-ctx --lib --all-features pcg_documents::
cargo clippy --package clean-ctx --lib --all-features -- -D warnings
cargo build --package clean-ctx --bin clean-ctx --all-features
```

The primary regression passed; the two module filters ran 22 and 6 tests,
respectively (the latter includes the primary). No ignored required tests.
Clippy had zero warnings. The production binary was built once, SHA-256
`b81789484379020d8bb96f540058f32e34bf765e28eca645f369ea8b69e2affa`.
Raw logs and stdio captures are retained under `target/pcg-b2/`, not mistaken for
tracked tests or committed reusable definitions.

Operator/production evidence: seven preflight queries; fourteen paired queries;
final B2 representative contexts and reconciliation. The unchanged existing
Rust tokenizer helper counts exact result files. These are live stdio receipts,
not Rust regression results or independent field certification. No Claude pilot
was needed for this explicitly negotiated document-consumer path.

Static verification covers source relocation, invariant/declaration preservation,
identity/relationship endpoints, changed links, strict UTF-8/no BOM, active file
sizes and normal commit guards. Broad unrelated Cargo suites, all-target final
repository gate and CI jobs were not run or claimed passed. The task explicitly
limits this delivery to focused checks; the authoritative full gate remains
[verification.md](../../agent/verification.md).

## Remaining holds and B3 handoff

- C15 all-feature CI discrepancy remains separate implementation work. CI is unchanged.
- C02/C03 historical PCG approval/banner dispositions and broader PCG contract
  admission remain B3; no accepted evidence interface or corpus was rewritten.
- Language/domain campaigns, Angular generation78/resolver/runtime boundaries,
  later performance histories and field evidence remain their assigned batches.
- B4 marker-vocabulary optimization and Tier C host acknowledgement are deferred;
  proxy Unicode/rule-language completeness is not certified by the two bounded repairs.
- The full final repository gate is unrun here. Focused B2 evidence does not
  certify uninspected producer paths or replace an owner-required broader gate.

The branch is stacked on `docs/pcg-foundation-b0b1` while upstream PR #179 is
actually open; completed work is not recreated because hosted status differs
from the task's shorthand. Commit and PR delivery are recorded in the final response.

```architecture-graph
node: PCG-175-B2
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Production subsystem document migration and exact source-reference transport
track: REPOSITORY
artifact: docs/architecture/knowledge-normalization/B2_PROGRESS.md
preserves:
  - AUTH.CANONICAL_IR
  - AUTH.BINARY_V04
note: Source migration and scoped deterministic and production stdio evidence; no independent certification or global final-gate claim.
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
```
