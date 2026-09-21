# IR Production Integration Audit — Continuation 2

**Status:** Phase 9 in progress. Continues
[`IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION.md`](IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION.md).

## 38. Finding P9-25: file-scoped lifecycle operations globally flush storage

**Severity:** High cross-file transaction-authority contradiction; repaired
and user-verified on 2026-09-20.

The post-P9-24 registered-path audit found that removing implicit commits from
reads, purge, and legacy clear did not eliminate the global commit surface.
Several registered semantic lifecycle paths call `BufferedStore::flush()`
before performing their own file-scoped durable work:

- `save_context` flushes before its requested checkpoint;
- `restore_context` and `replay_history` flush before loading durable state;
- persisted `compress_code_context` flushes before committing its candidate;
- delta baseline and application persistence helpers flush before their
  file-scoped atomic operations;
- staged edit and edit-recovery persistence paths flush before committing or
  reconciling their requested file.

`flush()` does not commit only the requesting operation's state. It drains the
entire pending queue and first attempts to reimport every fallback file. A
checkpoint, restore, compilation, delta, or edit request for one file can
therefore publish queued or fallback lifecycle work belonging to unrelated
files. Those additional commits are not represented in the requesting MCP
response and do not pass through the affected files' recovery, validation, or
transaction boundary.

Current registered production queueing is narrow: the remaining
`queue_append_delta` caller immediately invokes `flush()` in the accepted-delta
path, and no registered producer currently calls `queue_save_context` or
`queue_clear_file`. That limits ordinary queue exposure but does not make the
boundary sound. Fallback reimport remains global even with an empty queue, and
the public buffered/trait surface can recreate cross-file pending work.

### Alternatives

1. Replace global pre-flushes with scoped durable APIs. Registered semantic
   operations commit only their own canonical, delta, edge, edit-intent, or
   recovery transaction. Retire buffered queueing from production semantic
   authority where synchronous atomic APIs already exist. Give fallback
   reimport an explicit separately owned recovery boundary rather than running
   it opportunistically inside unrelated requests.
2. Retain buffering but add file/operation-selective flush and fallback
   recovery keyed by exact durable identity and transition. This preserves the
   mechanism but adds ownership, ordering, partial-queue, and failure
   complexity.
3. Keep global flushes and document every semantic operation as a potential
   cross-file durability boundary. This conflicts with the approved
   file-scoped transactional model and makes responses incomplete.

### Recommendation

Choose option 1. The approved production paths already use synchronous atomic
SQLite methods for canonical IR plus semantic edges, accepted deltas,
transactional deletion, and staged edits. Their preceding global flushes
should not be lifecycle authorities. The remaining accepted-delta compatibility
branch should commit its exact delta directly rather than queueing and globally
flushing. Fallback import should become an explicit startup or maintenance
recovery operation with truthful results and must not execute as a side effect
of a file-scoped MCP request.

Tracked registered coverage should inject unrelated queued and fallback work
before every affected operation and prove that only the requested file's
transaction changes. Failure coverage should prove unrelated pending/fallback
ownership remains intact. Existing exact `0x04`, `dv:2`, semantic-edge,
edit-intent, byte-exact edit, and live-publication boundaries must remain
unchanged.

**Implementation update (2026-09-20):** Registered save, restore, replay,
compression, delta baseline/application, staged-edit, edit-recovery, deletion,
and startup paths no longer invoke the global buffered flush. Each continues
through its existing scoped SQLite operation. The accepted-delta compatibility
branch now appends its exact delta directly and reports structural persistence
failure before live publication. No registered production caller queues
buffered saves, deltas, or clears; the remaining adapter is documented as
non-authoritative and requires an explicit owner-controlled commit.

## 39. Approval gate and next audit action

P9-25 was user-verified green with P9-26. Phase 9 is not certified.

## 40. Finding P9-26: legacy fallback artifacts cannot restore complete semantics

**Severity:** High durable-state completeness contradiction; repaired and
user-verified on 2026-09-20.

P9-25 requires fallback reimport to move behind an explicit recovery boundary
that performs the required file-scoped recovery and validation. The existing
fallback artifact format cannot satisfy that contract:

- fallback `save_context` JSON stores file path, fidelity, compact output,
  binary IR, source hash, and token counts, but no complete semantic-edge
  snapshot or aligned semantic version;
- fallback `append_delta` JSON stores only context ID, payload, and edit type,
  but no authoritative target semantic-edge snapshot, target source hash, or
  complete transition identity required by P9-12;
- fallback `clear_file` carries only a path and lacks the transactional durable
  ownership/result required by registered `delete_context`.

The importer disables foreign keys, accepts missing fields through empty/default
values, imports artifacts in directory enumeration order, and deletes each file
after its individual operation succeeds. It cannot prove a coherent physical
`0x04` canonical baseline plus semantic-edge snapshot, checked `dv:2`
transition, or transactional deletion. Canonical IR cannot reconstruct missing
framework/meta-layer edges, and approved restore/apply contracts forbid source
recompilation or synthesis as a fallback.

Consequently, merely renaming the current importer as an explicit maintenance
operation would make an incomplete legacy format authoritative and contradict
P9-10 through P9-12.

### Alternatives

1. Retire legacy fallback artifacts from semantic recovery. Remove fallback
   creation from the buffered flush failure path, leave any existing artifacts
   untouched, and provide an explicit inspection boundary that reports each
   artifact as unsupported/incomplete without importing or deleting it.
   Production scoped SQLite failures remain structural failures owned by the
   requesting operation. A future complete, versioned fallback protocol would
   require separate review.
2. Introduce a new versioned fallback transaction format containing physical
   `0x04`, complete aligned semantic edges, semantic version/hash/identity, and
   checked `dv:2` transition authority. New artifacts could be recovered, but
   existing legacy artifacts would still have to be rejected because their
   missing semantics cannot be reconstructed.
3. Recompile source or infer semantic edges while importing legacy artifacts.
   This violates the approved durable restore and authoritative-edge contracts
   and can silently produce state different from the failed transaction.

### Recommendation

Choose option 1 for the bounded Phase 9 repair. No registered production
caller remains that needs buffered fallback once P9-25 converts the accepted
delta compatibility write to a scoped direct commit. Retiring creation and
quarantining existing artifacts avoids pretending incomplete state is safely
recoverable. The explicit inspection result should identify artifact paths,
legacy operation kinds where parseable, and the reason recovery is refused;
it must not mutate SQLite, live/session ownership, source files, or the
artifacts themselves.

**Implementation update (2026-09-20):** Buffered failure no longer writes
legacy fallback JSON and ordinary flush no longer imports fallback artifacts.
Failed legacy batches remain pending and report zero committed operations.
`inspect_legacy_fallbacks` is the sole fallback-artifact boundary: it reports
path, parseable operation/identity, available metadata, and the semantic-
authority rejection reason without modifying the artifact, SQLite, source, or
live state. Existing legacy artifacts remain quarantined in place.

## 41. Approval gate and next audit action

P9-25 and P9-26 were user-verified green together. The remaining exhaustive
Phase 9 matrix audit resumed. Phase 9 is not certified.

## 42. Remaining registered-operation audit

The final registered paths were traced from dispatch through their actual
owners and consumers after P9-25/P9-26 verification.

| Registered operation/family | Authority and lifecycle | Consumer/exposure | Final status |
|---|---|---|---|
| `diff_code_context` | Reads authorized source bytes and owns only its AST-diff baseline in `LocalStateCache`; it does not publish canonical IR, durable semantics, aliases, or `WorkspaceIndex` state. | Registered rolling diff response | Confirmed distinct from the semantic persistence lifecycle. |
| `diff_commits` | Reads validated Git refs under the authorized workspace and constructs a bounded multi-file diff without installing canonical/session ownership. | Registered manifest and change-count response | Confirmed read/compute path; no semantic persistence bypass. |
| `index_repository` | Delegates explicitly to CBM indexing with caller-supplied repository identity and mode. CBM project/index ownership is separate from canonical IR and SQLite semantic contexts. | Registered CBM response | Confirmed external index lifecycle, not an alternate IR producer. |
| CBM graph/status/proxy tools | Inline dispatch is the intentional CBM boundary; project selection and graph cache/index state remain CBM-owned. The inline-name/registry parity contract prevents double dispatch. | Registered graph/status responses | Confirmed distinct ownership; no canonical IR publication. |
| `workspace_query` | Reads `WorkspaceIndex`; eligible queries may invoke one discovery/hydration cycle. Every discovered file enters the shared P9-20 pending-edit recovery and checked semantic publication boundary before index mutation. | Registered scoped entity/edge/graph responses | Confirmed production integration; recovery failures propagate structurally. |
| Semantic definition and fact families | Default compiler, ordered production passes, exhaustive shared validation, checked typed projection, physical `0x04`, complete edge snapshot, and compact renderer remain the authoritative path. | Context, delta, restore, replay, edit, and workspace responses | Confirmed across definitions, signatures, modifiers, control summaries, patterns, side effects, execution contexts, relationships, bodies/spans, and explicit interfaces. |
| Delta/replay | Production emits `dv:2`; accepted durable transitions require server-owned target-edge authority and persist scoped canonical/edge state before live publication. Legacy input remains compatibility-only and cannot become durable without approved authority. | Registered delta/apply/replay responses | Confirmed occurrence/order preservation and transactional ownership. |
| Persistence lifecycle | Save, restore, replay, replacement, deletion, reset, edit, and recovery use file-scoped durable transactions. Reads, stats, listing, purge, and history inspection cannot publish pending work. | Registered persistence/context responses | Confirmed after P9-08 through P9-26. |
| Legacy buffered/fallback surface | No registered production semantic path queues or globally flushes buffered work. Incomplete fallback artifacts are quarantine/inspection-only and cannot enter SQLite or live semantics. | No semantic MCP exposure | Obsolete production authority removed; future versioned fallback remains separately reviewable. |

## 43. Final obsolete/bypass review

The final static production-path review found:

- no registered caller of unchecked hierarchical projection;
- no legacy delta emission from production generation;
- no lossy `filter_map` canonical tuple reconstruction in registered paths;
- no empty/noncanonical registered persistence write;
- no restore/replay source-recompilation fallback;
- no class/interface normalization in the corrected production path;
- no registered semantic global-buffer flush dependency;
- no opportunistic fallback creation or import;
- no read-only context operation that creates identity or commits lifecycle
  work;
- no live canonical, semantic-edge, or `WorkspaceIndex` publication before its
  approved durable commit boundary.

The unchecked hierarchy convenience and legacy delta types remain internal
compatibility/convenience surfaces with no registered production consumer.
They are not production authorities and do not weaken the checked MCP path.

## 44. Phase 9 certification boundary

**Status:** Certification pending user-run production-path verification.

The code-level production-integration matrix is complete:

- every registered IR/MCP operation was classified and traced through its
  actual production ownership and consumer path;
- every canonical semantic family was traced through production compilation,
  validation/transformation, session ownership, persistence/lifecycle, checked
  projection, and external exposure where applicable;
- all high-severity findings P9-01 through P9-26 were either repaired and
  user-verified or explicitly retained as non-authoritative future capability;
- no obsolete registered production bypass remains for the audited contracts;
- physical `0x04`, `dv:2`, semantic-edge, interface, edit-fidelity, full-body,
  and compact-rendering boundaries remain distinct and coherent;
- the applicable repository gate was reported green by the user after the last
  bounded repair.

Final Phase 9 certification remains withheld until the required operator
verification artifacts can exercise every approved production boundary and the
user reports their results. Phase 10 must not begin automatically.

## 45. Finding P9-27: fallback inspection production reachability

**Severity:** Verification-blocking production-reachability gap; approved and
implemented pending user verification.

The P9-26 repair introduced non-mutating
`BufferedStore::inspect_legacy_fallbacks`, but that method is crate-internal and
has no registered MCP operation, CLI command, or other executable production
entry point. The required Phase 9 operator scenario cannot exercise the
approved inspection boundary through the real production binary. Calling it
from a scratch program or tracked test would verify implementation code, not
registered production reachability, and would violate the requested harness
contract.

Alternatives:

1. Add a registered read-only MCP maintenance operation for legacy fallback
   inspection. It reports artifact path, parseable kind/identity/metadata, and
   the unsupported semantic-authority reason, while performing no recovery or
   mutation. This makes the approved boundary operator-verifiable.
2. Add a separately named CLI maintenance command with the same read-only
   contract. This avoids enlarging MCP but creates a second executable routing
   surface and would not satisfy a registered-MCP lifecycle scenario.
3. Declare fallback inspection internal-only and waive the requested live
   production-path verification. This leaves P9-26 without operator-reachable
   evidence and contradicts the current verification request.

Recommendation: option 1. The operation is a truthful read-only view over an
already-approved maintenance boundary and provides the smallest way to verify
that quarantined artifacts cannot mutate semantic state. Its public name,
schema, response, and registration require explicit approval before
implementation.

**Implementation update (2026-09-20):** The registered read-only
`inspect_legacy_fallbacks` MCP operation now routes through the existing
`BufferedStore::inspect_legacy_fallbacks` authority. It returns every
discoverable artifact, including malformed artifacts, with its path,
parseable legacy operation/identity/metadata, explicit `recoverable: false`,
and the incomplete-authority reason. The handler performs no fallback import,
flush, recovery, deletion, rewrite, SQLite write, or semantic/session/index
publication. Tracked registered-path coverage preserves that non-mutation
contract.

The operator-only package under `target/phase9-verification/` drives the
registered MCP operation and the remaining Phase 9 lifecycle scenarios through
a freshly built production binary. Those artifacts are explicitly not tracked
tests, CI evidence, or substitutes for the repository verification gate.

## 46. Verification handoff

P9-27 and the Phase 9 operator fixtures are ready for user-run verification.
Phase 9 certification remains withheld until the user reports the tracked
repository gate and the operator scenarios green. Phase 10 must not begin
automatically.
