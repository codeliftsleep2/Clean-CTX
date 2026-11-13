# B9-R1 bounded delivery handoff

Agent: **Agent MaxHeadRoom**.

## Delivered boundary

The existing knowledge tool returns structural identities, typed relationships,
complete declared graph closures and exact revision-pinned component references.
Graph retrieval does not acquire source bodies. Individual/batch acquisition is
explicit, preserves mandatory qualifications and exact bytes, and reports invalid,
stale and historical reference outcomes without silently claiming completeness.
Explicit whole-source acquisition remains available. Knowledge response version 2
records the deliberate compatibility change; existing enrolled-evidence and other
token-budgeted interfaces remain separate.

Implementation commit: `df4d0414e0b48f78eba876e0338f00edbf1c1e58`.
Branch: `(fix)Performance-Regression-Issues`. The owner's later branch instruction
supersedes the earlier stacked-PR delivery protocol. No new branch is requested.
Reusable refresh scripts and production/host evidence were committed in
`be772ba2` (`docs: record B9-R1 production and host verification refresh`).
This final closeout records the owner's bounded architectural acceptance.

## Verified evidence

| Category | Actual result | Receipt under target/pcg-b9-r1 |
|---|---|---|
| Broader C# dependency/fidelity/regression tests | 72 passed, zero failed | failure-recheck-20261113-124828-813 |
| Scoped all-target/all-feature Clippy | Zero warnings, exit zero | same receipt, clippy.log |
| Document tests | 39 passed | delivery-refresh-20261113-134605-318 |
| Registered knowledge MCP tests | 10 passed | same receipt |
| All-feature product build | Exit zero | same receipt |
| Actual-root production verification | Exact graph and acquisition checks passed | same receipt, production/verification-summary.json |
| Actual host workflow | Seven calls covering all six modes; exit zero | host-refresh-20261113-142039 |
| Static host capture verification | Returned-reference provenance, qualifications and exact bytes verified | same receipt, verified-summary.json |

Rebuilt executable SHA-256:
`a555a0ad5374b7bc1b4db4a391ea0e389158880ac7d6c4c210c81f8f708266e0`.
Profile SHA-256:
`0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9`.

Graph: 218 identities, 221 typed relationships, 326 components. All eleven
expected closures preserve exact membership, relationships and witnesses.
Acquisition verifies 326 selected references and 430 content/qualification
occurrences. Only the three authorized governing/workflow revisions advance.
Generation:
`684f1ca847aaec8398506ca3e8d0ebd3bf8f5bbaacb26ba88ede062fb17c6d70`.

## Remaining obligations and scope limits

- The installed CBM provider times out on the nonexistent-project query even
  outside Clean-CTX. The unchanged live regression remains failed; see
  `B9_R1_CBM_PROVIDER_BLOCKER.md`. No unchanged retry is requested.
- Full repository gates belong to CI. No local full-gate run or handoff is requested.
  Focused evidence above is not a claim that CI passed.
- All sixteen frozen independent evaluations remain UNRUN pending valid isolation.
- Complete captured payloads and bounded host inspection do not prove universal
  model-visible completeness or comprehension. Reported context display truncation
  remains explicit; B10 large-response diagnostics remain suspended.
- No B10 resumption, representation redesign, provider installation repair,
  independent certification, new branch or additional production change is implied.

The owner accepted B9-R1 at the bounded architectural and implementation level
and authorized the final evidence commit and push on the existing performance
branch. Architectural work is complete. Preserve historical reports, failed
receipts and every limitation above. No completed verification is restarted.
After the final evidence commit is pushed, stop; internal document admission
governance belongs to the next separately authorized mission.
