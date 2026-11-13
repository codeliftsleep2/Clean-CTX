# PCG-175 Corpus 02: approved union enrollment and real-use result

Agent: **Agent MaxHeadRoom**

Date: 2026-11-11. Authority: owner-approved Corpus 02 enrollment and five explicit
MCP queries. Disposition: **USEFUL — both reviewed architectural domains recovered
from one complete generation**. No PCG implementation or prior verification
campaign was reopened.

## Enrollment and preservation

The approved complete-union manifest was imported through the existing explicit
CLI, using the accepted binary. Actual generation exactly matches prediction:

`e304524614d3df0d6512efbaea0a30f31b94b1eaec54c7e6120c26be6f5f83f9`.

| Identity | Verified value |
| --- | --- |
| Source commit | `6add8449c1a036d0599283fd6a982009d495e954` |
| Source tree | `d1540017265328e4bbb0b4ea31d6a8bc7172cb1a` |
| Approved manifest SHA-256 | `58f334bedb47ad26e2decbcbc23a7940f0c10bca758e709bde8766300a73afe5` |
| Approved profile SHA-256 | `cc5501390d5443e20eb38369b20cdd9754f61df2f7d8a14efc9b6ddbe1ca3896` |
| Executed binary SHA-256 | `0959d317624514e5cb9e1d79e36e5ba1c48587f7ac84496d31b7edec1d29f76c` |

Manifest/profile digests, exact Git commit/blob membership, all source/excerpt
hashes and the predicted generation were checked before enrollment. A read-only
post-import comparison confirmed every stored source, evidence, record, selector,
relation and binding revision equals the proposal. This corroborates record
revisions rather than relying solely on the generation label.

The isolated `target/pcg-175/corpus02/workspace/` contains the 13 selected source
files as exact Git bytes at the pinned commit. Its detached HEAD resolves that
commit through local alternate objects; no fetch, remote write or source commit
was needed. The reused startup configuration disables CBM, proxy auto-start,
source persistence and cache. The separate approved PCG store remains enabled.
This isolates PCG retrieval from implicit source compilation/indexing.

Union storage is `target/pcg-175/corpus02/storage/projection.sqlite`. It contains
17 records, 37 evidence revisions, four selectors and 32 graph edges/bindings.
Import exited zero and reported 37 evidence validations, 435,106 Git bytes and
870,212 working-file bytes over the two 13-source observations. These are one
run's accounting values, not scaling or latency claims.

All 72 previously inventoried Pilot 01 review/execution/storage files retain
their exact hashes. The original manifest, profile, database, accepted responses
and receipts were neither replaced nor reenrolled. Original source files also
retain their preflight working-byte hashes. Complete-union import, rather than
an implicit merge, is the supported additive strategy.

## Actual MCP boundary and results

A fresh production stdio server process was launched with the approved profile
and isolated project root. Initialize and tools/list succeeded; `pcg_query_v1`
was registered once, read-only, without an added structured output channel.
Five explicit tools/call requests ran in that session, then the process exited
zero. Requests, raw JSON-RPC responses, exact MCP result fragments, decoded
payloads, stderr and counting output are retained under
`target/pcg-175/corpus02/execution/`.

| Query | Result | Required records | Relationships | Citations | Whole-result o200k tokens / limit |
| --- | --- | --- | --- | --- | --- |
| IR constraints | complete_within_registered_scope | 11 | 16 | 20 | 14,866 / 32,768 |
| IR closure: ir-edit-recovery | complete_within_registered_scope | 11 | 16 | 20 | 14,866 / 32,768 |
| IR provenance: ir-compatible-adoption | complete_within_registered_scope | 11 | 16 | 20 | 14,866 / 32,768 |
| IR impact: ir-occurrence-preservation | complete_within_registered_scope | 11 | 16 | 20 | 14,866 / 32,768 |
| Original CBM constraints question | complete_within_registered_scope | 7 | 15 | 18 | 13,649 / 32,768 |

Each IR result is 58,399 bytes, SHA-256
`08a90657f618c757e42e5aad032bd9200f2b8b898b3dd1a6c9f221e9b3ef2d97`.
The four modes returned identical result bytes: their common mandatory scope
already includes all ten IR records plus `cbm-semantic-authority`, and all five
necessary prerequisite edges. Different modes did not provide extra selection
power for this deliberately broad, single-question scope.

Every result reports all five completeness checks true, START and COMPLETION
matched, required material fit true, no conflicts, missing/stale requirements or
reason codes. Optional omissions are exactly `["not_requested"]`;
optional/diagnostic truncation is false. No budget increase, mandatory evidence
drop or manifest alteration was made.

Counts were obtained by the existing standalone o200k counter from the exact
serialized MCP result bytes extracted from JSON-RPC framing. Escaped text and
the content/isError wrapper are included; JSON-RPC framing is excluded, as v1
defines. Counter binary SHA-256:
`92ca8701af027627e51f87a02d566a65241d859e1174d57e034ad98a35864382`.
The counter was reused, not rebuilt; its retained identity is provenance, not a
new source-to-counter build certification. No cl100k population was authorized
or run in this pilot.

## Independent citation, relationship and preservation assessment

The artifact assessment checked every emitted quote against its pinned blob
bytes, half-open offsets, quote SHA-256, whole-source SHA-256, path, commit and
blob OID. It checked record and relationship revisions against the proposal,
edge endpoints/kinds/provenance, all required primary/reference witnesses and
one citation per evidence ID. All checks passed for all five responses.

The IR outputs contain all 11 applicable bindings and five prerequisites.
The descriptive `PRESERVES` link to the old semantic-authority record is stored
but not returned in these mandatory-only results. Its omission does not remove
the explicitly bound old record or its required authority citation. This is an
observed retrieval limit for optional graph exposition, not a missing necessary
premise. No USES_EVIDENCE relationship was promoted into a prerequisite.

The CBM payload equals Pilot 01's retained constraints payload after removing
only the generation field. Its records, relationships, exact citations, scope,
coverage and budget metadata are unchanged. Token count differs from Pilot 01's
13,642 because the serialized generation value changed. Original target project
identity, project-owned partitions, target invalidation, success-only refresh,
error-versus-empty, public/internal routing and semantic-authority evidence all
remain retrievable from the union.

Union main-database SHA-256 before and after the five reads:
`d5de15d7a4ecbe3a058564324aaf253016e04fa33e03592f94e7ef0d65bfcee2`.
Binary/counter bytes remained unchanged. Read-only inspection did not run another
PCG full audit, and a main-file comparison is not a physical-erasure or complete
storage-side-effect certificate.

## Source-defensible engineering answer

The returned evidence supports these seven conclusions without reopening source
files. Evidence IDs below refer to the retained response citations, whose exact
locators and hashes are in the approved proposal catalogue.

1. **Identity and multiplicity survive.** Typed owner namespaces must resolve;
   unique definitions and optional singular facts reject duplicates. Ordered
   facts/payloads preserve order and every allowed repeated occurrence. A
   transformation must preserve identity, apply a complete reference remap, or
   explicitly authorize removal without leaving references. Checked failure must
   not return a partial hierarchy as success. Evidence: ir-approved-defaults,
   ir-identity-namespaces, ir-operation-matrix, ir-cardinality,
   ir-validation-transform and ir-semantic-boundaries.
2. **Semantic serialization and replay are exact, distinct contracts.** Binary
   0x04 preserves file/version, operations/operands, order, duplicates, typed
   families, bodies and spans. Malformed/unsupported/trailing input fails
   structurally. Delta dv:2 independently checks placement, expected tuple,
   typed identity and occurrence transactionally; durable application requires
   the server-owned target semantic snapshot. Evidence: ir-binary-equality,
   ir-binary-failures, ir-binary-decoder, ir-replay-invariant and
   ir-semantic-boundaries.
3. **Durability follows policy and coherent ownership.** Automatic checkpointing
   differs from manual-mode session-only reads. Edit authority and accepted
   mutations retain their stated durability requirements. Required durable
   publication aligns canonical IR, source identity, fidelity, semantic state
   and compatibility identities from one owner epoch before live publication;
   older in-flight owner operations cannot overwrite newer authority. Reads do
   not flush another file's pending state. Evidence: ir-checkpoint-invariant and
   ir-owner-publication. These current policy clauses qualify the older broad
   certification wording rather than silently imposing always-durable reads.
4. **Loading is not authority.** Structural/source/fidelity checks and separate
   canonical/semantic configuration and exact producer identities precede their
   respective live adoption boundaries. Semantic dependencies must still be
   current; canonical validity alone does not validate a derived snapshot.
   Missing legacy identity is unproven, not automatically blessed or deleted.
   Compatible reuse is supported. Evidence: ir-compatibility-invariant and
   ir-compatibility-checks.
5. **Recovery completion is distinct from current adoption.** Resolve file-scoped
   pending edit intents before authoritative load/compile/checkpoint/hydration.
   Edits prove exact prior bytes and candidate semantics through staged recovery,
   replacement and aligned commit. An older recovered transaction retains its
   identities; incompatibility still prevents live adoption. Evidence:
   ir-durable-recovery, ir-owner-publication and ir-compatibility-invariant.
6. **Compatibility input cannot manufacture semantics.** Lossy binary 0x01–0x03
   is rejected; older supported hierarchy remains decode-only; legacy deltas
   require unambiguous targets; incomplete fallbacks remain inspection/quarantine
   only. New compatibility support requires evidence and approval. Inspection,
   decoding and historical reported PASS never grant current semantic authority.
   Evidence: ir-binary-legacy, ir-quarantine and ir-compatibility-invariant.
7. **Presentation and discovery remain separate authorities.** Minimized LLM
   output does not replace canonical semantic state or edit fidelity. CBM is
   discovery infrastructure; WorkspaceIndex owns the stated live semantic
   authority. Cache presence/freshness is not proof of compatible IR adoption.
   Evidence: ir-projection-separation, ir-semantic-boundaries and
   overview-authority.

## Usefulness and limitations

**USEFUL for the reviewed architectural question.** A reader receives the seven
claim groups, complete mandatory conditions, prerequisite paths and exact source
text in one explicit response within the fixed budget. The same enrolled
generation also recovers the prior CBM answer unchanged. This establishes useful
architectural continuity beyond fixture-only acceptance; it does not quantify
time saved, compare a blinded baseline, or demonstrate enterprise efficiency.

The corpus is manually reviewed, source-attributed and question-scoped. Exact
operation/task labels and trusted profile configuration are required; this is
not natural-language discovery, autonomous extraction or repository-wide prose
completeness. All four IR modes select the same broad roots, so they do not
demonstrate narrower relevance discrimination. Optional descriptive cross-topic
relationships are not visible in these outputs; finer scope or richer graph
exposition would need separate design review if needed, not an in-pilot repair.

Freshness remains two observed working-file boundaries, not continuous
immutability or an atomic multi-file snapshot. Projection integrity remains
admission plus accessed-row verification. Source approvals and historical gate
claims remain source-attributed; no tests were rerun or authenticated runtime
approval created. The pinned snapshot says nothing about later commits.

Only this result report and ignored Corpus 02 execution/evidence files were
created during execution. No production code, accepted corpus definition,
original Pilot 01 enrollment or existing verification asset changed. No builds,
tests, CI, benchmarks, commits, pushes or publication occurred.

## Retained evidence

- [Enrollment identity](../../target/pcg-175/corpus02/execution/enrollment-identity.json)
- [Importer output](../../target/pcg-175/corpus02/execution/import.stdout.json)
- [MCP session identity](../../target/pcg-175/corpus02/execution/mcp-session.json)
- [Five result summaries](../../target/pcg-175/corpus02/execution/mcp-results.json)
- [IR evidence payload](../../target/pcg-175/corpus02/execution/constraints.payload.json)
- [Exact IR MCP result](../../target/pcg-175/corpus02/execution/constraints.result.json)
- [CBM preservation payload](../../target/pcg-175/corpus02/execution/cbm-preservation.payload.json)
- [Independent artifact assessment](../../target/pcg-175/corpus02/execution/independent-assessment.json)
- [Approved proposal](PCG_175_CORPUS02_PROPOSAL.md)
