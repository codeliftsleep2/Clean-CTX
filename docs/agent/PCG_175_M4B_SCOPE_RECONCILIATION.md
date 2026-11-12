# PCG-175-M4B: frozen-scope reconciliation

B3 source disposition: C03: the later source-document catalog is a distinct implemented scope, not this proposed runtime ledger. Its source-attributed statuses do not authenticate actors, approvals or transitions. The unresolved decisions below remain operative holds on ledger implementation; no blocker endpoint or frozen ledger design is invented. The catalog may describe this mission without satisfying it.

Agent: **Agent MaxHeadRoom**

Date: 2026-11-11 (America/New_York).
Authority: owner-directed M4b reconciliation following acceptance of M4a,
including the explicit instruction to report undefined or prohibited scope.

Disposition: **M4B IMPLEMENTATION SCOPE NOT FROZEN**. The approved design freeze
defines no M4b implementation stage. No production/test/definition file changed,
and no verification campaign was reopened. This is a scope discrepancy, not a
demonstrated defect in the accepted implementation.

## Frozen requirement-to-implementation mapping

| Source | Actual contract | Current disposition |
| --- | --- | --- |
| [Design freeze §10.4](PCG_175_DESIGN_FREEZE.md#104-separately-authorized-implementation-stages) | Four named stages: M2a-1, M2a-2, M3 and M4a. No M4b row, permitted paths, interfaces or exit criteria. | All named stages are owner-accepted. There is no remaining frozen M4b implementation obligation to assign. |
| Design freeze §2 | Runtime ledger records/writes/approval, automatic injection and other expanded capabilities are deferred. | Preserve the accepted read-only boundary; do not infer a write capability from the new mission label. |
| Design freeze §4.1 | Approval assurance remains source-attributed; no authenticated runtime approval exists in v1. | M4a preserves this distinction. A graph classification or profile label cannot become an authenticated approval event. |
| Design freeze §10.4 closing paragraph | “Runtime ledger/approval/extraction/injection/embeddings/hosting remain deferred beyond these stages.” | These are explicitly outside the frozen implementation sequence. |
| [Discovery §14](PCG_175_ARCHITECTURE_INVESTIGATION.md#14-incremental-implementation-milestones--planning-only), M2b/M4b row | “Only after authenticated owner/approval, consent, retention and correction contracts”; earned runtime ledger, UC-2/3 without self-promotion. | Planning-only, gated future work. It does not supply those contracts or a frozen M4b MCP integration design. |
| Accepted [M4a report](PCG_175_M4A_IMPLEMENTATION.md) | Explicit conditional tool, pinned profile, existing projection, exact metered bytes, failure propagation, legacy compatibility and real stdio reachability. | Already completed and accepted. Relabeling or repeating this work would not implement a distinct M4b obligation. |

The current owner instruction supplies M4b mission authority. The reason for
stopping is the missing frozen scope and its explicit prerequisite contracts,
not the older report's historical statement that M4b was unauthorized.
The freeze's historical approval status likewise does not revoke the owner's
subsequent acceptance of the completed stages.

## Exact unresolved decisions

If M4b means the discovery's runtime ledger, a separately reviewed design must
establish the authenticated actor/capability and revocation boundary, exact
proposal/approval/correction subjects and transitions, consent, retention/erasure,
durable ledger ownership/transactions, and public API/acceptance criteria before
implementation. Existing source-attributed enrollment and a derived SQLite
projection do not establish these contracts.

If M4b instead means further read-only MCP work, the missing contract is its exact
remaining behavior and exit evidence beyond accepted M4a. No such stage appears
in the freeze. Canonical entity proof, optional historical enrichment, proxy or
Claude field work must not be silently substituted as a new mission.

## Preserved contracts and negative controls

Exact M3 result bytes, explicit read-only invocation, session-pinned trust,
required closure/failure semantics and the disabled legacy catalog remain
unchanged. No source hydration, import, enrollment, approval write or fabricated
canonical identity was introduced.

The accepted M4a report retains evidence for strict argument/scope refusal,
profile-pin refusal, unavailable projection, required-budget/stale-source empty
bundles, exact-byte dispatch and legacy compatibility. Its five focused cases,
four existing compatibility cases and two real-stdio sessions were not rerun.
These remain accepted prior evidence, not new M4b execution results. No new
negative test was justified because no M4b behavior or defect was established.

## Changed files, execution and final identity

Only this reconciliation report was added. Existing uncommitted M4a changes were
left untouched; no staging, commit, ref switch, push or publication occurred.

Performed bounded document/section searches, native reads, Git status/identity and
byte-hash comparisons. No tests, builds, formatter, Clippy, binaries, servers,
operator driver, benchmark or broad gate was launched. Unrun verification is not
failure and is not claimed green.

Local HEAD: `b65561ed6169ce860a7f07c32a08382de5b5e725`, the accepted M3 checkpoint.
Current M4a remains an accepted uncommitted worktree on top of that commit.
All 18 entries in `target/pcg-175/m4a-verify/source.sha256` still match current
source/definition bytes. Inventory SHA-256:
`0de354231f559154dc040eda0c5ea8205f509ce6b24850cc5f42e6f7ed299c56`.
No new runtime source revision was produced.

Inspected design-freeze SHA-256:
`73fd5147ea202906e46eacd7740d719265c407732d01c6a63434e184b4a5ffb4`.
Inspected M4a report SHA-256:
`f5ba3c9e6552444250730ac50a3e16c4a392698d83fec3e7b154ae779eb0f5f5`.

## Recommendation

Accept this scope reconciliation; **do not claim M4b implemented or acceptance
ready**. The frozen read-only MCP boundary is already satisfied by accepted M4a.
A distinct M4b requires a concrete design amendment resolving the decisions above.
Preserve prior acceptance and verification closure while that scope is defined.

```architecture-graph
node: PCG-175-M4B
kind: MISSION
status: PROPOSED
title: Unfrozen runtime-ledger scope and unresolved prerequisite decisions
track: REPOSITORY
artifact: docs/agent/PCG_175_M4B_SCOPE_RECONCILIATION.md
note: Source record and its attributed disposition only; no new execution, authenticated approval, independent certification, reclosure or banking.
```
