# B0/B1 production migration progress

Agent: **Agent MaxHeadRoom**. Date: 2026-11-11.

Source normalization and actual production reconciliation succeeded. All seven
representative contexts were ready, catalog-current and complete, with no diagnostics.
The accepted assessment baseline is `655f0a75`. This is a migration record, not
another normative owner for the migrated propositions.

## Scope and source ownership

All 73 invariant/debt sections now have one focused current owner across 13
families under `docs/invariants/`. The original catalog preserves legacy headings,
enforcement vocabulary and governance procedure as navigation. Exact original
commit/path/line ranges, whole-source hash and per-section hash are retained in
[B0_B1_PROVENANCE.csv](B0_B1_PROVENANCE.csv). The original normative text remains
in Git, rather than as a second current copy. C13 is the only intentional change
to a relocated invariant's descriptive body.

The assessment hash identifies its original mixed LF/CRLF worktree bytes. Git's
normalized LF blob has SHA-256
`4e2e78e09a6e99af276ef1fbd467cd718bd7e34bbeaa7cc670ae79199921f8d3`;
both identities are recorded explicitly. Static comparison confirms the other
72 section bodies are unchanged, including their captured section-byte witnesses
after restoring trailing section separators. The old catalog has no remaining
duplicate `Invariant` table cells. Unicode was preserved; no encoding normalization
was used to conceal a substantive change.

The B1 graph declares 28 foundational invariants and five authority contracts:
canonical IR composition, cohesive CoreOp matrix, cohesive Binary 0x04 matrix,
question matrix and layer certification. Later-family sections remain current
source contracts but have no graph declaration yet. Stable compatible identities
are retained; C-22, mechanism reuse and meta-layer completeness use the explicit
aliases established by the assessment.

GLS IMPLEMENTED records an already source-described mechanism or operative
governance contract, never a new witness execution. DOCUMENTED mechanism reuse
remains INVESTIGATING in GLS; its source convention is not promoted to machine
enforcement. No node is newly FROZEN, VALIDATED, RECLOSED or BANKED. Source
ENFORCED/STRUCTURAL/DOCUMENTED/RESOLVED and qualified attributes are preserved.

## Production profile and authoring route

The owner-controlled, ignored `.clean-ctx/pcg-documents/profile.json` scopes the
actual repository workspace to `document_roots:["docs"]`. The separate generated
document database is `.clean-ctx/pcg-documents/documents.sqlite`; accepted evidence
corpus stores and profiles are untouched. At configuration the docs tree contained
402 filesystem entries and 9,389,303 file bytes. The existing scanner debits entries
and reads Markdown at both source-observation boundaries.

Finite profile ceilings: 2,048 examined entries, 512 nodes, 2,048 edges, 33,554,432
observed bytes and 65,536 result tokens. This accommodates the measured tree and
foundational context plus B2 headroom without making these limits global defaults.
Changes in physical scope require the existing lifecycle's new database rule;
future operational ceiling adjustments remain explicit owner configuration.

The original count is 402 entries beneath docs plus the scanned root itself;
the byte total bounds all files, while the scanner reads Markdown sources and
their declared artifacts. Profile/database paths are excluded from tracked source
and lie outside the scanned document roots.

The accepted production binary hash is
`54afcb7843bb42a6c8fe7083c6f9f6d543bfa2bf5c681a67ac25fa5017f0194a`.
No build is needed. This host does not advertise `pcg_documents_v1` directly.
The existing production stdio interface does advertise it under this actual-root
profile. `verification/pcg-175/invoke-workspace-documents.ps1` provides explicit
authoring queries and retained raw responses. It creates no source declarations,
fixture workspace, ingestion service or replacement graph.

The launch directory's separate ignored `.clean-ctx.json` disables optional CBM,
proxy, source persistence and source cache for document-authoring sessions. The
process configuration root is that launch directory; every document query and
the pinned document profile explicitly use the actual repository workspace.
This distinction avoids changing the user's normal root configuration or opening
the existing evidence-corpus profile. All processes are bounded and retired.

Initial actual-root reconciliation returned ready/current generation
`1d64ff7f4eb08edb58b30fef76c638ad0602eec050f7bbdbd6c1abeb7f4c8cf1`.
Its catalog contained only NORM-001 and PCG-175-ADOPTION. Both complete contexts
were retrieved before source editing; neither had typed relationships. This is
production workspace evidence, not the earlier controlled DOC-081 scenario.

## C13 source comparison and preserved limits

The canonical enum has 30 variants. Binary 0x04 has 31 opcode values because
exact and spread Call have different physical opcodes. The dedicated
`src/tests/ir/binary_wire_v04.rs::all_variants_with_duplicates` fixture covers all
30 variants and both Call forms; its round-trip assertion compares file identity,
version and the complete ordered instruction stream, including duplicates.

The randomized generator in `round_trip_randomized.rs` covers 25 variants and
100 seeds, not all current variants. It omits the four interface-specific
operations and BaseTypeRef. The older `round_trip.rs` fixture also covers 25;
its hierarchical test uses a partial oracle. No test was executed here, and no
all-format completeness claim is inferred. WIRE-001's invariant is unchanged;
only its stale enforcement description and witness references are corrected.
The original sentence, “All 20 CoreOp variants are covered,” remains inspectable
at the original section/source hash and commit above.
The static variant-by-variant comparison is retained in
[B0_B1_C13_COVERAGE.csv](B0_B1_C13_COVERAGE.csv). This is coverage-source inspection,
not a new test result.

## Actual admission and retrieval results

The first foundational publication is generation
`2b0f0e892dd23c92027de58977c33b3c2d104babd4508b04840e81e264144dc5`.
It reconciles the complete candidate, including 33 new foundational nodes and
the two pre-existing nodes. Source validation succeeds; no independent approval,
test execution or closure is inferred from that success. The exact admitted
source declarations and proposed later holds are distinct in
[B0_B1_NODES.csv](B0_B1_NODES.csv).

| Context start | Returned nodes | Unique source files | Traversed edges | Exact serialized o200k tokens |
| --- | ---: | ---: | ---: | ---: |
| AUTH.CANONICAL_IR | 8 | 6 | 7 | 34,091 |
| AUTH.BINARY_V04 | 4 | 3 | 3 | 23,214 |
| AUTH.QUESTION_MATRIX | 2 | 2 | 1 | 8,354 |
| AUTH.LAYER_CERTIFICATION | 5 | 3 | 4 | 23,589 |
| WIRE-001 | 1 | 1 | 0 | 3,470 |
| AUTH-001 | 1 | 1 | 0 | 5,130 |
| INV.C22 | 1 | 1 | 0 | 2,499 |

The complete returned node/file lists, exact serialized result hashes and costs
are retained in [B0_B1_RETRIEVAL.csv](B0_B1_RETRIEVAL.csv). These measurements name
the publication above; final migration-record reconciliation may advance the
global generation without changing foundational source revisions or edges.

The production response meter covers the serialized MCP result, excluding only
outer JSON-RPC framing. Its exact bytes were extracted from the existing
`send_pcg_result` framing without reserialization or trailing newline and counted
by the already-built Rust `clean-ctx-measure-helper.exe` o200k counter. Counter
SHA-256: `92ca8701af027627e51f87a02d566a65241d859e1174d57e034ad98a35864382`.
This uses the existing tokenizer implementation; no helper was compiled or new
token approximation introduced.

The admitted foundational typed edges are in
[B0_B1_RELATIONSHIPS.csv](B0_B1_RELATIONSHIPS.csv). They include canonical
composition's operation premise and preserved boundaries, Binary's CoreOp premise
and IRWIRE preservation, and certification's question-matrix prerequisite and
completion/mechanism constraints. No inferred supersession, blocker, validation,
reclosure or banking edge is introduced. The migration record separately preserves
the two cohesive matrices; it does not become their authority.

No representative request exceeded the profile's token/node/edge ceilings. No
broad historical investigation, original catalog payload, experiment ledger or
operator-definition collection was attached to these contexts. Complete retained
matrices still contain their relevant phase/compatibility notes. Family files can
also include adjacent current rules awaiting B2; they are not claimed admitted
merely because their text is returned.

Measured limitation: canonical context returns the same canonical family for three
selected nodes, and binary context returns the same identity family for two.
Certification context returns the completion family for three nodes. The API
currently represents these as separate node documents containing complete sources.
This explains the difference between nodes and unique files and the transitive
cost. Focused families substantially bound a single-invariant source; multiple
declarations do not supply section-only retrieval or source-payload deduplication.
The largest measured context is within the explicit budget; no genuine edge was
removed and no PCG retrieval change was made.

Retained local receipts: `target/pcg-b0b1/preflight/`, `constraints/`,
`foundation/` and the final reconciliation directory. These contain actual
requests, raw RPC results, decoded payloads, process exit/stderr, binary/profile
hashes and relevant revision/edge comparisons. No SQLite data or machine-local
configuration is tracked. The final source generation is reported with delivery,
avoiding a self-referential generation embedded inside its own source revision.

## Using this production authoring route

When the host exposes the tool, query `pcg_documents_v1` with the actual workspace
and existing owner profile. This session cannot add it dynamically to its host
tool catalog; stdio availability is observed, direct-host availability is not.
An operator can use the existing binary and this bounded route instead:

```powershell
$repo = (Get-Location).Path
$owner = Join-Path $repo '.clean-ctx/pcg-documents'
# Write an explicit JSON query array under ignored target state, for example:
$query = @(@{workspaceRoot=$repo; mode='context'; id='AUTH.BINARY_V04'; tokenizer='o200k'; tokenBudget=65536; maxNodes=128; maxEdges=512})
[IO.File]::WriteAllText((Join-Path $repo 'target/pcg-b0b1/authoring-queries.json'), (ConvertTo-Json -InputObject $query), [Text.UTF8Encoding]::new($false, $true))
pwsh -NoProfile -File verification/pcg-175/invoke-workspace-documents.ps1 -WorkspaceRoot $repo -Profile (Join-Path $owner 'profile.json') -Binary (Join-Path $repo 'target/debug/clean-ctx.exe') -LaunchDirectory (Join-Path $owner 'authoring') -Requests target/pcg-b0b1/authoring-queries.json -OutputRoot target/pcg-b0b1/authoring-fresh
```

Use a fresh output directory. The optional `-TokenCounter` parameter accepts the
existing Rust counter when exact serialized-cost observations are wanted.
This command actually starts a bounded production stdio session. Current-task
owner authorization covers this migration's sessions; future sessions remain
subject to the existing runtime-execution rule. Instructions do not grant
blanket launch permission or change graph scope.

## Remaining migration scope

- B2: workspace completeness/history/provider/cache/persistence and public tool
  lifecycle contracts; CI all-feature discrepancy C15 remains in its implementation
  scope, not a prerequisite for B1 source admission.
- B3: historical PCG approval/banner discrepancies C02/C03 and broader authoring
  contract admission. Existing accepted evidence-corpus stores remain unchanged.
- B4: Angular generation78 dispositions, resolver keys/effective runtime meaning,
  other language/domain contracts and final campaign acceptance.
- B5/B6: performance dispositions, focused historical evidence and field receipts.

PR #178 was reported landed by the task; the live GitHub response at this run
still shows OPEN. The exact locally accepted assessment commit is retained as
the source baseline. This discrepancy does not delay production migration.

## Source-owned migration record

```architecture-graph
node: PCG-175-B0B1
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Production foundational document graph migration and scoped query evidence
track: REPOSITORY
artifact: docs/architecture/knowledge-normalization/B0_B1_PROGRESS.md
preserves:
  - AUTH.CORE_OPS
  - AUTH.BINARY_V04
note: Source normalization and actual stdio reconciliation; no new runtime test or independent certification.
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
```
