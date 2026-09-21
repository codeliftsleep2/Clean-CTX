# IR Production Integration Audit — Continuation

**Status:** Phase 9 in progress. This document continues
[`IR_PRODUCTION_INTEGRATION_AUDIT.md`](IR_PRODUCTION_INTEGRATION_AUDIT.md)
after P9-15/P9-16 were user-verified green.

## 23. Finding P9-17: registered `apply_edit` description advertises a removed contract

**Severity:** High externally visible tool-contract contradiction; Option 1
repaired and user-verified.

The registered `apply_edit` input schema correctly requires `filePath` and an
`operations` array containing `replace_body`, `delete`, `insert_after`, or
`insert_before`. Its registered description begins by advertising a different
generic text editor: `insert_line`, `new_text`, `old_text`, arbitrary
replacement, and file creation. None of those parameters or behaviors exists
in the production handler.

The real handler requires previously tracked byte-exact state, refuses new or
unowned files, resolves structural units, validates expected unit text, applies
the approved P9-15/P9-16 source-identity and durable transaction contract, and
returns operation-specific byte spans. The runtime prompt and authoritative
tooling guide describe this structural contract; only the public registered
tool description still exposes the obsolete generic contract.

This can cause MCP clients and LLMs to send requests that the registered schema
and handler reject, or to believe `apply_edit` can create arbitrary files. It
also omits the now-authoritative stale-source and durability preconditions.

### Alternatives

1. Replace the obsolete description with the existing structural operation
   contract and explicitly summarize tracked-state, byte-exact, stale-source,
   and transactional durability behavior. Keep the correct input/output schema
   and production handler unchanged.
2. Restore the advertised generic editor behavior. This conflicts with the
   approved structural, tracked-state, and byte-exact edit architecture.
3. Introduce a separately named generic file-edit tool. That is a new public
   capability outside the Phase 9 repair and requires separate review.

### Recommendation

Choose option 1. This is a bounded public-description correction: it makes MCP
discovery truthful without changing the approved handler, operation schema, or
edit behavior. Add registered tool-list coverage that rejects the obsolete
vocabulary and asserts the structural contract.

## 24. Approval gate and next audit action

The obsolete generic-editor description was replaced with the approved
structural, tracked-state, byte-exact, stale-source, staged-durability, span,
and full-body contract. Input/output schemas and production behavior are
unchanged. Tracked `tool_list()` coverage protects both required and forbidden
vocabulary. The exhaustive Phase 9 audit has resumed; Phase 9 is not certified.

## 25. Finding P9-18: `replay_history` bypasses durable edit-intent recovery

**Severity:** High transactional lifecycle contradiction; Option 1 repaired
and user-verified.

The approved P9-15 crash-recovery contract makes a durable edit intent the
authority for deciding whether exact prior bytes remain authoritative or exact
target bytes must be committed before durable state is loaded. Registered
`restore_context` now invokes that recovery boundary before reading and
publishing a persisted context.

Registered `replay_history` directly calls `load_durable_context` and then
installs canonical IR, source hash, fidelity, semantic edges, aliases,
`WorkspaceIndex`, and compact output. It does not inspect or recover a pending
edit intent. A crash after exact target-source replacement but before the
target semantic commit can therefore be followed by `replay_history`, which
publishes the older durable baseline while the source file contains the exact
target bytes. This recreates the source/live/durable split P9-15 was designed
to prevent and leaves the recovery intent unresolved.

The bypass affects current and targeted history replay, subsequent deltas and
edits, source-hash authority, complete semantic edges, and the truthfulness of
the registered replay response.

### Alternatives

1. Route `replay_history` through the same deterministic pending-edit recovery
   boundary as `restore_context` before any durable load or live mutation.
   Prior-byte recovery proceeds with history replay; target-byte recovery first
   commits the exact target semantics, after which the requested durable state
   is loaded normally. Irreconcilable or persistence failure remains structural
   and leaves live ownership unchanged.
2. Reject every replay while an edit intent exists and require callers to run
   `restore_context` first. This is safe but makes recovery depend on a separate
   public operation even though replay already claims durable restoration.
3. Keep ignoring edit intents. This violates the approved staged-edit recovery
   and transactional publication contracts.

### Recommendation

Choose option 1. Durable edit-intent recovery is a storage lifecycle boundary,
not a tool-specific feature. Reusing it preserves one deterministic crash
decision while leaving replay sequence selection and response semantics
unchanged.

**Implementation update (2026-09-20):** Pending-edit recovery now belongs to
the shared `McpState` durable-semantic lifecycle. `apply_edit`,
`restore_context`, and registered `replay_history` invoke that one boundary.
Replay cannot load or publish durable state until recovery succeeds. Registered
coverage includes unchanged replay, exact-prior recovery, exact-target recovery
after restart, targeted sequence replay, failed target persistence, and
irreconcilable source bytes.

## 26. Finding P9-19: delta durable-baseline paths bypass edit recovery

**Severity:** High cross-cutting transactional contradiction; Option 1 repaired
and user-verified.

The required post-P9-18 inspection found that `delta_code_context` and
`apply_delta` reach durable baseline helpers without first resolving a pending
edit intent. `ensure_persisted_baseline` directly reads physical binary and
calls `load_durable_context`; `ensure_apply_baseline` delegates to it. The
first-baseline branch of `delta_code_context` can instead compile current disk
bytes and call `persist_baseline`, replacing durable canonical and edge state
without consulting the recovery intent.

After restart during the P9-15 source-replaced/semantic-not-committed window,
`delta_code_context` can therefore treat the exact target source as a new
baseline and overwrite durable ownership through ordinary compilation rather
than committing the authoritative recovery artifacts. In a surviving session,
delta validation/application can read durable state while an uncleared intent
still owns the crash decision. Both paths violate the rule that pending edit
intent resolution precedes any authoritative durable semantic load or
replacement.

### Alternatives

1. Require the shared recovery boundary at the registered delta entry points
   before candidate compilation, durable baseline inspection/replacement, delta
   generation, or application. Recovery failure returns structurally before
   live or durable mutation. Keep `dv:2`, pending target-edge authority, and
   delta response semantics unchanged.
2. Reject all delta operations whenever an edit intent exists and require
   `restore_context` first. This is safe but unnecessarily makes recovery
   dependent on a separate public call.
3. Permit delta baseline logic to reconcile independently. This duplicates or
   bypasses the approved deterministic recovery authority.

### Recommendation

Choose option 1. The shared boundary already owns the exact recovery decision;
delta handlers should invoke it rather than infer state from disk or durable
baselines. Tracked registered coverage should include first-baseline generation
after restart, existing-baseline generation, and application failure paths,
proving no compile, durable replacement, pending delta transition, live state,
or `WorkspaceIndex` mutation occurs before recovery succeeds.

**Implementation update (2026-09-20):** Both registered delta entry points now
resolve the shared durable edit intent before compilation, baseline access,
pending target-edge lookup, generation, or application. A resolved intent is
hydrated from its checked durable canonical/edge state before delta work, so a
recovered target cannot be overwritten as a synthetic first baseline. Recovery
failure returns structurally without compiling or changing live, durable,
index, or pending-delta ownership. Registered coverage exercises prior and
target recovery, restart, corrected `dv:2` generation, duplicate application,
and persistence failure for generation and application.

## 27. Approval gate and next audit action

P9-18 and P9-19 were user-verified green. The exhaustive Phase 9 audit has
resumed; Phase 9 is not certified.

## 28. Finding P9-20: remaining context producers bypass recovery ownership

**Severity:** High cross-cutting lifecycle contradiction; repaired and user-
verified on 2026-09-20.

The post-P9-19 registered-path audit found four remaining ways to publish,
checkpoint, index, or delete semantic state without coherently owning a pending
edit intent:

- Non-verbatim `compress_code_context` compiles current disk bytes and can
  replace the durable baseline and live canonical/index state without first
  resolving the intent.
- `provide_code_context` reads source and creates alias/durable mappings before
  strategy selection, then several strategies compile or publish canonical IR,
  semantic edges, pending deltas, and `WorkspaceIndex` state without recovery.
- `save_context` can checkpoint session canonical/edge state while an intent
  still owns whether prior or target bytes are authoritative.
- Registered `workspace_query` hydration compiles discovered files and publishes
  semantic edges into `WorkspaceIndex` without checking file-scoped recovery.

Deletion has the inverse ownership defect. `delete_context_transactionally`
deletes the context baseline, delta history, and semantic-edge snapshot but not
the file's `edit_intents` row. Registered `delete_context` can therefore report
complete deletion while leaving durable crash-recovery ownership behind. The
row has no foreign key that would remove it with the context.

Read-only source diff, stats, list, and history-metadata operations do not
publish durable canonical or semantic state and have not been shown to require
this boundary. Verbatim compression returns source bytes without publishing a
semantic baseline and remains distinct from state-producing compression.

### Alternatives

1. Make shared pending-edit preflight mandatory for every registered path that
   compiles and publishes canonical/edge/index state or writes a semantic
   checkpoint: non-verbatim compression, provision strategies, `save_context`,
   and per-file workspace hydration. Recovery and checked durable hydration
   complete before aliases, mappings, compilation, checkpointing, pending
   transitions, or index mutation. Extend transactional deletion to remove the
   file's edit intent as owned durable state, without first committing a target
   that is immediately being deleted.
2. Reject those operations whenever an intent exists and require explicit
   `restore_context`; deletion would still need transactional intent removal.
3. Allow each operation to infer or overwrite recovery state independently,
   recreating the split-authority defects repaired in P9-15 through P9-19.

### Recommendation

Choose option 1. Recovery is a file-scoped storage lifecycle boundary; semantic
producers and checkpoint writers must enter through it, while deletion must
atomically remove it. Implement one reusable preflight that recovers and
hydrates checked durable state before publication, preserving each handler's
response, fidelity, compact rendering, full-body fallback, and transaction
semantics. Add the edit-intent row to the SQLite deletion transaction and
remove any staged recovery artifact only after durable deletion commits.

Tracked registered coverage should prove each affected path cannot compile,
checkpoint, publish aliases/edges/index state, or overwrite baselines before
successful recovery; failure is mutation-free; deletion removes baseline,
history, edge snapshot, and intent together while preserving source bytes; and
unaffected read-only operations retain their current behavior.

**Implementation update (2026-09-20):** `McpState` now exposes one semantic-
publication preflight that deterministically resolves any durable edit intent
and checked-hydrates the recovered canonical and complete edge state before a
caller may continue. Non-verbatim compression, all provision strategies,
file-scoped checkpointing, registered delta entry points, and per-file
workspace hydration enter through this authority before compilation or
publication. Workspace hydration propagates recovery failures to the
registered MCP response rather than misclassifying them as ordinary candidate
compile misses. Verbatim compression remains byte-only and outside the gate.

Transactional deletion now owns the baseline, `dv:2` history, semantic-edge
snapshot, and edit-intent row in one SQLite commit. Only after that commit does
the handler remove the staged recovery artifact and live session/index owners;
failure rolls the intent and all other durable rows back together. Registered
coverage exercises mutation-free failure and successful recovery for
compression, provision, save, and workspace hydration, plus deletion success
and failure with exact source and peer ownership preserved.

## 29. Approval gate and next audit action

P9-18 through P9-20 were user-verified green. The exhaustive audit resumed;
Phase 9 is not certified.

## 30. Finding P9-21: read-only context history creates identity ownership

**Severity:** High session-identity contract contradiction; repaired and user-
verified on 2026-09-20.

The P9-20 audit explicitly left read-only history metadata outside the shared
semantic-publication preflight unless implementation evidence showed that it
mutated authoritative state. Registered `context_history(filePath)` does so:
before reading either session history or durable metadata, it calls
`get_or_create_alias`. That method takes the mutable path dictionary and
creates a session alias for any supplied string, including a file that has
never been compiled, validated, persisted, or otherwise tracked.

This contradicts the registered description, which says the tool views
history for tracked files, and makes a nominal read operation establish path
identity ownership. The invented alias can affect later alias allocation,
dictionary/footer output, path lookup, and handlers that distinguish a known
alias from an unowned path. It also bypasses the P9-20 rule that alias creation
which establishes semantic ownership occurs only after recovery and checked
hydration. No canonical IR, fidelity, source hash, semantic-edge state, or
durable mapping is installed with this alias, so the resulting session identity
is incomplete by construction.

### Alternatives

1. Make `context_history` strictly non-mutating. Use `alias_for_path` to inspect
   an existing session owner; if none exists, report no live IR baseline while
   still reading durable history by the supplied durable file identity. Never
   allocate an alias merely to answer history metadata.
2. Route `context_history` through recovery and checked durable hydration before
   creating an alias. This makes a read-only metadata request restore and
   publish semantic state, changing the operation's public meaning and failure
   surface.
3. Retain alias allocation and document the side effect. This preserves the
   defect and permits partial identity ownership without canonical semantics.

### Recommendation

Choose option 1. History inspection does not require publication. Existing
session identity should be observed without mutation, durable history should
remain keyed by the requested durable identity, and an untracked file should
not become tracked merely because its history was queried. Registered coverage
should prove that existing tracked history remains unchanged, unknown paths do
not change dictionary/alias state, repeated reads are idempotent, and no
pending edit intent is recovered or otherwise mutated by this read-only tool.

**Implementation update (2026-09-20):** The registered handler now observes an
existing alias with `alias_for_path` and never allocates session identity.
Committed durable history metadata is read directly from SQLite by the supplied
file identity without flushing buffered writes, recovering edit intents, or
hydrating semantic state. The legacy session-only metadata store remains a
non-mutating fallback when durable persistence is unavailable. The tool
description now states the read-only ownership contract. Registered coverage
proves tracked and durable-only history, unknown-path idempotence and alias
ordering, and preservation of pending edit intent and all semantic owners.

## 31. Approval gate and next audit action

The exhaustive Phase 9 audit resumed after user verification of P9-21. Phase 9
is not certified.

## 32. Finding P9-22: read-only context stats commits and mutates state

**Severity:** High cross-file durability and observation-contract
contradiction; repaired and user-verified on 2026-09-20.

P9-20 left read-only stats outside the semantic-publication recovery boundary
unless implementation evidence showed authoritative mutation. Registered
`context_stats` performs two such mutations before returning its dashboard.

First, it calls `McpState::flush_persistence`. `BufferedStore` documentation
explicitly names `context_stats` as a flush boundary, so merely viewing the
dashboard can commit any queued save or clear operation, including unrelated
files. The response does not identify those commits, cannot report individual
failures truthfully, and does not run file-scoped pending-edit recovery before
making queued durable semantic changes authoritative.

Second, when proxy metrics are available, the handler writes fetched proxy
counters into live `session_stats` and then also applies them to its local
dashboard clone. Repeated reads can therefore change the session statistics
they purport to observe. The registered description and architecture overview
both describe `context_stats` as a view/dashboard operation, including an
explicit diagram label of `context_stats (read)`.

Affected contracts include unrelated buffered persistence ownership,
file-scoped edit recovery, truthful failure reporting, repeatable statistics,
session metrics, and the read-only classification used by the Phase 9
operation matrix.

### Alternatives

1. Make `context_stats` strictly observational. Remove it as a persistence
   flush boundary; read only committed SQLite statistics and current in-memory
   statistics. Apply fetched proxy values to the local response snapshot only,
   without recording them back into session ownership. Pending buffered work
   remains owned by its producing lifecycle and is committed only by an
   explicit/approved durability boundary.
2. Redefine `context_stats` as an explicit global flush-and-refresh operation,
   expose every affected persistence result, and run recovery for every file
   involved. This is a substantially different public operation and couples
   observation to unrelated lifecycle transitions.
3. Keep flushing and live counter mutation as undocumented implementation
   details. This retains non-idempotent reads and bypasses the approved
   file-scoped transaction model.

### Recommendation

Choose option 1. A dashboard read should not become an implicit global commit
or rewrite its own source metrics. Registered coverage should prove queued
writes and clears remain pending, committed SQLite rows are still reported,
proxy observations affect only the current response, repeated reads are
idempotent, pending edit intents and semantic owners remain unchanged, and no
other file is committed or cleared by a stats request.

**Implementation update (2026-09-20):** `context_stats` no longer flushes the
buffered store. It reads committed SQLite statistics and clones current session
statistics without changing either owner. Proxy observations are applied only
to the response-local clone and never recorded into authoritative
`session_stats`. The registered description now states the observational
contract. Registered coverage retains queued saves and clears, proves committed
rows remain visible, isolates proxy-derived response data, and preserves edit
intent, canonical, hash, fidelity, edge, and `WorkspaceIndex` ownership.

## 33. Approval gate and next audit action

P9-22 was user-verified green. The exhaustive Phase 9 audit resumed; Phase 9
is not certified.

## 34. Finding P9-23: persisted-context listing commits buffered lifecycle work

**Severity:** High cross-file durability and observation-contract
contradiction; repaired and user-verified on 2026-09-20.

The audit of buffered persistence ownership found no registered production
write that became stranded solely because P9-22 removed `context_stats` as a
flush boundary. The remaining production `queue_append_delta` caller performs
an explicit flush in the same accepted-delta lifecycle, while no registered
production caller was found for `queue_save_context` or `queue_clear_file`.
This means the removed stats flush was not the authoritative commit owner for
current registered writes.

The same audit found a different implicit boundary. Registered
`list_sessions` calls `BufferedStore::list_contexts`, and `list_contexts`
unconditionally calls `flush()` before reading committed SQLite rows. A tool
described as showing tracked sessions/files can therefore commit every queued
save, delta, or clear across unrelated files merely because persistence was
listed. The response reports only the resulting rows; it neither identifies
the lifecycle operations it committed nor reports their individual durability
outcomes. It also bypasses the file-scoped recovery and transaction ownership
required of semantic mutations.

The buffer's `load_latest` and `delta_count` read methods contain the same
read-causes-flush pattern. No current registered production call path was found
through those methods during this checkpoint, but they remain an obsolete
bypass surface that can recreate the defect if reused.

### Alternatives

1. Make persisted-context enumeration strictly observational. Remove the
   implicit flush from `list_contexts` and read only committed SQLite rows.
   Remove read-triggered flushing from `load_latest` and `delta_count`, or make
   those methods inaccessible as lifecycle authorities, so buffered writes can
   commit only at explicit producing-operation boundaries.
2. Redefine `list_sessions` as a global flush-and-list operation, exposing and
   recovering every affected file before commit. This changes a read tool into
   a cross-file lifecycle command and substantially expands its response and
   failure contract.
3. Keep the implicit flush as an implementation detail. This preserves a
   hidden, non-file-scoped durability boundary and permits future callers to
   depend accidentally on reads for commits.

### Recommendation

Choose option 1. Reads should observe committed state and never become fallback
commit owners. Each registered producer must either commit synchronously at its
approved lifecycle boundary or retain explicitly pending work with a named
owner. Tracked registered coverage should prove that `list_sessions` leaves
queued saves, deltas, and clears pending; returns only committed contexts;
does not recover edit intents or mutate semantic/session/index ownership; and
is idempotent. Focused storage coverage should prevent `load_latest` and
`delta_count` from silently flushing queued work. Existing explicit delta
commit behavior must remain unchanged.

**Implementation update (2026-09-20):** Persisted-context reads now inspect
committed SQLite state directly. `list_contexts`, `load_latest`,
`load_context_with_deltas`, and `delta_count` do not flush or otherwise publish
queued work. Registered `list_sessions` documents this committed-state-only
contract. Queue size no longer creates an implicit commit boundary; the
producing lifecycle must invoke its explicit commit, including the existing
accepted-delta boundary. Tracked storage coverage proves queued saves, deltas,
and clears remain pending across repeated reads.

## 35. Approval gate and next audit action

P9-23 was user-verified green with P9-24. Phase 9 is not certified.

## 36. Finding P9-24: purge commits unrelated buffered lifecycle work

**Severity:** High cross-file durability and operation-authority
contradiction; repaired and user-verified on 2026-09-20.

Implementation inspection for approved P9-23 found another global flush
boundary before production code was changed. Registered `purge_old_deltas`
calls `BufferedStore::purge_old_deltas`, which unconditionally calls `flush()`
before performing the requested age-based history purge. A request authorized
to remove old delta history can therefore commit unrelated queued saves,
deltas, or clears for any file. Those commits are absent from the operation's
response and do not pass through their producing file's approved lifecycle
boundary.

The legacy `ContextStore::clear_file` implementation also flushes the complete
queue before clearing one file. Registered `delete_context` no longer uses
that path and instead performs its approved atomic file-scoped deletion, but
the legacy method remains an obsolete cross-file commit surface.

This is distinct from P9-23: making reads observational does not remove a
mutating operation's authority to commit unrelated work.

### Alternatives

1. Scope `purge_old_deltas` strictly to its requested purge transaction and
   remove its global pre-flush. Remove or constrain the legacy buffered
   `clear_file` implementation so it cannot commit unrelated queued work.
   Existing explicitly owned accepted-delta commits remain unchanged.
2. Redefine purge as a global flush-then-purge lifecycle operation and expose
   every affected file's recovery, commit result, and failure. This greatly
   expands a maintenance operation's public authority and response contract.
3. Retain the pre-flush as an undocumented convenience. This preserves the
   cross-file commit bypass and prevents buffered writes from having one
   explicit producing owner.

### Recommendation

Choose option 1 and implement it in the same bounded storage decomposition as
P9-23. Registered coverage should prove purge changes only qualifying committed
history, leaves every queued operation pending, and preserves unrelated files.
Focused storage coverage should prove legacy file clearing cannot flush other
files' queued work. The registered transactional `delete_context` contract
must remain unchanged.

**Implementation update (2026-09-20):** `purge_old_deltas` now operates only
on committed SQLite history and never flushes the queue. The legacy
`ContextStore::clear_file` implementation removes the requested committed
context plus queued operations demonstrably owned by that file, without
publishing peer work. Registered transactional `delete_context` remains the
sole production deletion contract. Focused coverage preserves peer committed
and queued state across purge and legacy clear operations.

## 37. Approval gate and next audit action

P9-23 and P9-24 were user-verified green as one bounded storage-authority
repair. The remaining registered-operation, semantic-family, lifecycle, and
obsolete-path matrix audit resumed. Phase 9 is not certified.

The audit continues in
[`IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION_2.md`](IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION_2.md).
