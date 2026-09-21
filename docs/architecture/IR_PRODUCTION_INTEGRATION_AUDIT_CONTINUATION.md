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

**Severity:** High transactional lifecycle contradiction; Option 1 implemented
pending user verification.

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

**Severity:** High cross-cutting transactional contradiction; architectural
approval required.

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

## 27. Approval gate and next audit action

P9-18 is implemented pending user verification. The exhaustive audit is paused
at P9-19, and no delta production code has been modified for that finding.
After approval and user verification of the applicable repairs, resume the
remaining operation, semantic-family, lifecycle, and obsolete-path matrix.
Phase 9 is not certified.
