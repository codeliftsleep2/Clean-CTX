# PCG pre-authoring integration and observed agent use

Agent: **Agent MaxHeadRoom**. Date: 2026-11-11.

Authority: owner-directed operational integration and focused authoring scenario.
Disposition: **IMPLEMENTED; one controlled real-agent workflow observed**.
This is not evidence that every agent or unconfigured host now uses PCG.

## Entry points and responsibility

The existing [pre-authoring guidance](pcg-authoring.md) is the procedural source
of truth. Portable AGENTS.md, the local ignored `.clinerules/engineering.md`,
the agent documentation index, architecture procedure and tooling guide route
significant architectural work to it. Ordinary coding edits are explicitly
exempt. The existing automated runner reads AGENTS.md into its system prompt
(`scripts/agent/agent.mjs::runAgentSession`); no runner changes or CI jobs were
needed. Actual tool/profile availability remains host-specific.

Agents start with exact node context or deterministic track/FTS search, inspect
source-backed typed paths, carry applicable constraints into work, author GLS
declarations, then reconcile through pcg_documents_v1. No manual graph manifest,
parallel tag authority or approval step is introduced. Source truth outranks
metadata. DEPENDS_ON/BLOCKED_BY remain prerequisites/obligations; descriptive
USES_EVIDENCE/PRESERVES/IMPLEMENTS are not promoted into necessary truth.
SUPERSEDES and FALSIFIES remain distinct controlling replacement/counterevidence.

Final reconciliation compares relevant revisions and edges, not merely the global
generation. A newly relevant successor/falsifier matters even when the original
node's revision is unchanged. An unrelated generation change does not require
repeating the investigation. Incomplete or blocked context must be reported,
never presented as checked. PCG does not decide semantic applicability/conflict
or grant execution permission; the author evaluates the returned source text.

## Actual authoring sequence

The connected host does not advertise pcg_documents_v1. For this explicitly
authorized scenario, Agent MaxHeadRoom invoked that existing tool through a fresh
production stdio MCP session using the accepted binary and existing controlled
document profile. No product implementation, new database, corpus manifest,
Python helper, model delegation or simulated response was used.

Binary SHA-256:
`54afcb7843bb42a6c8fe7083c6f9f6d543bfa2bf5c681a67ac25fa5017f0194a`.
Existing profile: `target/pcg-175/documents-live-20261111/profile.json`.
Existing SQLite owner: `target/pcg-175/documents-live-20261111/documents.sqlite`.
Captured evidence: `target/pcg-175/adoption-20261111/`.
The prior scenario's three source documents were copied to `originals/` before
updating that controlled workspace; earlier response receipts were not overwritten.
Accepted Corpus 01/02/03 stores and production source were untouched.

1. Prepared a scenario-local chain with actual quoted contracts: DOC-068's body
   contains BINARY_V04_CONTRACT.md §1's exact round-trip requirements, including
   repeated/duplicated occurrences. DOC-042 contains CORE_OP_CONTRACT_MATRIX.md
   §4.1's operation cardinality/no-deduplication contract. Their source attribution
   names commit `6add8449c1a036d0599283fd6a982009d495e954`; whole working-source
   hashes were retained. These are real repository constraints presented in a
   controlled catalog, not newly approved repository identities.
2. DOC-081 initially asked whether a proposed binary-size optimization could
   discard duplicate canonical operations. No recommendation had been written.
   The agent called tools/list then tools/call pcg_documents_v1, context DOC-081.
   The tool returned ready/current/complete, source text for all three documents,
   and DOC-081 IMPLEMENTS DOC-068 DEPENDS_ON DOC-042.
3. The agent read that returned source text and identified the actual conflict:
   dropping duplicate operations violates exact round-trip equality and the
   preserve-every-occurrence rule. This conclusion was made by the author, not
   emitted by an automatic contradiction detector.
4. The agent authored DOC-081 with a revised recommendation: decline duplicate
   elimination under the current contract; any later optimization must preserve
   multiplicity/order and exact equality. Its GLS declaration retains MISSION /
   INVESTIGATING, IMPLEMENTS DOC-068 and adds PRESERVES DOC-042. It claims neither
   implementation success, validation, independent closure nor banking.
5. A second real tools/call context request reconciled that source. It returned
   ready/current/complete; DOC-068/042 revisions stayed equal to preflight.
6. Added scenario-local unrelated DOC-099, then queried DOC-081 context again.
   The global generation changed but the entire returned relevant documents and
   context objects remained equal. The agent retained the conclusions without
   reopening the investigation or seeking another approval.

This is observed behavior by the current agent through the production tool,
using deliberately constructed source declarations. It is not a blinded study,
an organically encountered production conflict, a new Rust regression, proof of
automated conflict detection, or adoption evidence for other agents.

## Retained identities and relevant comparison

| Observation | Generation |
| --- | --- |
| Pre-authoring context | `61f6c9c748c8253b3fac4280d84dd2784edf11260f85669957a1c271a643d5a5` |
| Authored document reconciliation | `dec981aa53ac6a67b25e7bf21b396a6596aa4d7075f5ee76e055e008e3234a4a` |
| Unrelated DOC-099 change | `765859acf23ef28a7ba84090e849d5c6305a2593fa98fcc905c1e2cb505c2d8c` |

Relevant controlling revisions, unchanged across all three observations:

- DOC-042: `6c3182657973f11ad0e3d3a2e6fb6290c0fba26ea54f11191f420d7fbc826d85`.
- DOC-068: `a017c95120fdac1494a39d44ade1aa03b5393ba647ce0ce579c3f74f2654d092`.

DOC-081 advanced from
`68d132ff61eb79c81d0e75b6eaa548f2ceaf84cfe13523b7bd24ffa7887edb92`
to `fc88a053f6e1d021361a6e5fd0fb7e91ae326e71c434ec8574f3dd653528889c`.
The author's new PRESERVES link is explicit and remains descriptive. The existing
IMPLEMENTS/DEPENDS_ON path and controlling evidence were unchanged. No further
architectural reasoning was required for DOC-099.

Requests, raw JSON-RPC responses, decoded payloads, process exit/binary/profile
identities and stderr are retained for `preflight`, `reconciliation` and
`unrelated-change`. `revision-comparison.json` and `unrelated-comparison.json`
retain exact comparisons; `authored-DOC-081.md` retains the actual authored proposal.
All three production sessions exited zero. No build or test campaign was run.

## Verification, delivery and limits

This change updates instructions and documents only. Focused static review checks
links/routing, relation semantics, relevant-change handling and strict UTF-8/no
BOM/whitespace. Normal commit encoding/size hooks remain enabled. Prior accepted
Rust/stdio suites were not repeated; the three calls above are adoption evidence,
not a replacement verification gate. No new CI job or reusable harness framework.

The local `.clinerules` instruction change is intentionally ignored, while its
portable projection and procedure are tracked. Deployment requires a host exposing
the existing document tool with an owner-configured profile. Instructions cannot
force compliance by an external agent, and an absent tool/profile must be reported
honestly. This session's explicit runtime authority does not authorize automatic
server launches or new profile creation in future tasks.
The tracked report's own declaration below is prepared source metadata; no
production-root document profile was configured or admitted by this mission.
Only the controlled scenario's declarations were actually reconciled.

Candidate consistency, attributed source status and revision history do not prove
revision authorization or mathematical contradiction detection. Freshness remains
boundary-observed rather than continuous/atomic. Broad repository-wide tests,
Clippy, builds and CI campaigns were not run. The six earlier untracked corpus
reports remain unrelated and are excluded from this delivery.

## Architecture evidence graph declaration

```architecture-graph
node: PCG-175-ADOPTION
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Pre-authoring agent instructions and controlled adoption evidence
track: REPOSITORY
artifact: docs/agent/PCG_175_AUTHORING_ADOPTION.md
verdict: Instructions integrated; one controlled agent workflow observed
note: No automatic conflict detection, global agent compliance or independent banking claimed.
implements:
  - AUTH.PCG_PREAUTHORING
uses_evidence:
  - PCG-175-DOCUMENTS
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
```
