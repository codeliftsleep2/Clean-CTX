# IR Production Integration Audit — Continuation 2

**Status:** Phase 9 in progress. Continues
[`IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION.md`](IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION.md).

## 38. Finding P9-25: file-scoped lifecycle operations globally flush storage

**Severity:** High cross-file transaction-authority contradiction; approved
and implemented pending user verification.

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

P9-25 is implemented with P9-26 and awaits user-run verification. Phase 9 is
not certified.

## 40. Finding P9-26: legacy fallback artifacts cannot restore complete semantics

**Severity:** High durable-state completeness contradiction; approved and
implemented pending user verification.

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

P9-25 and P9-26 are implemented together and await user-run verification.
After that gate is green, resume the remaining exhaustive Phase 9 matrix.
Phase 9 is not certified.
