# B9 design disposition

Agent: **Agent MaxHeadRoom**, 2026-11-12. Owner-authorized B9-A/B/C correction.
This is the implementation disposition, not a competing normative authority.
The governing source remains `docs/architecture/PCG_DOCUMENT_AUTHORING.md`, with
the agent workflow routed through `docs/agent/pcg-authoring.md`.

## Verified starting point and preflight

B8 commit `76e0d0a3622ed76f60e5abcaba179ab81716efc9`, PR #187 OPEN/draft,
head `feat/pcg-b8-document-budget`, base `docs/pcg-b7-assessment`, verified live.
B9 branch: `feat/pcg-b9-knowledge-units`. No upstream branch is modified.
The accepted executable SHA-256 is
`d6af36341dc15b96548b9c3e404e96f7af215dea082490650bc792cafb4f2dcc`.
The actual graph has 218 identities, 221 typed relationships and generation
`abb04e766d381da87b1dcd7ec18c7b22d6c98949276d83d523f209f0f6c5e0da`.
Read-only baseline acquisition verifies all 153 declaring source hashes.

Production preflight retrieved 19 complete/current contexts using that executable
and the unchanged actual-root profile. Subjects cover canonical IR, binary identity,
semantic authority, declaration acquisition, workspace composition, certification,
PCG authoring/enrolled compatibility, performance doctrine, Angular, C# history and
the retained performance contexts. Raw evidence: `target/pcg-b9/preflight/**`.
In particular PCG preauthoring consumes AUTH.PCG_DOCUMENTS, which depends on
AUTH.PCG_GLS; USES_EVIDENCE/PRESERVES are not promoted to necessary premises.
Source-attributed historical status remains distinct from executed verification,
authenticated approval, reclosure and banking. The host does not advertise the
conditional document tool; the authorized production stdio route supplies preflight.

## Problem and chosen representation

B8 demonstrated whole-document indexing per identity and full-source acquisition
for discovery. The new representation separates a source artifact, an identity,
its exact source-owned content components, typed relationships and evidence.
Existing identity/lifecycle/relationship semantics and canonical revision hashes
remain. The separate enrolled interface is unchanged. No universal ontology is added.

Extend the existing flat `architecture-graph` declaration grammar with an explicit
knowledge scope and an ordered content-component list. Each entry assigns a typed
role, optional source-owned semantic label and exact source selector. Selectors
identify either the explicitly declared complete source (`whole`) or an explicitly
named pair of source markers. Paths remain repository-relative and link-safe.
Human headings are not extraction rules. Markers formalize existing authored
sections without rewriting their prose. Multiple components retain shared
definitions, qualifications, rationale and supporting evidence when needed.

Scope classifications distinguish proposition, decision, invariant/constraint,
historical finding, complete authority/contract, complete evidence and multi-section
knowledge. They do not change existing public graph kinds. Every baseline identity
gets an explicit source-owned disposition; missing coverage never defaults to an
inferred complete proposition. Whole reports remain whole where their identity
legitimately denotes the report. Shared-file invariant identities get own boundaries.

Component roles distinguish statements, subjects, scope, outcomes, rationale,
alternatives/rejected alternatives, rejection reasons, consequences, constraints,
verification obligations, observations, counterexamples, qualifications, historical
applicability and evidence. Roles and labels annotate exact existing source content;
they do not manufacture facts or infer current execution from historical reports.
Only source-supported components are authored. Exact prose retrieval and structured
component predicates are separately reported capabilities.

Rust validates unique ordered marker pairs, missing/ambiguous/malformed selectors,
UTF-8 intervals, source identity and nonempty exact content. No fixed chunks,
token limits, heading guesses or generated summaries. Full source SHA plus exact
byte interval/content SHA bind every component to its observed source revision.
Moving a marked interval preserves node identity; changing its source legitimately
advances the existing source-bound revision. Existing historical revisions remain.

## SQLite ownership and migration

Evolve only the document catalog schema, recognizing the exact accepted v1 schema.
Schema v2 separates deduplicated `(path, source_sha, body)` artifacts from immutable
document revisions. Legacy full-source responses rejoin the exact original body;
their revision calculation and historical meaning remain intact. New knowledge
unit/component tables reference those revisions and exact artifact intervals.
Indexed metadata and component predicates support deterministic joins; per-component
FTS records expose their declared match origin. No second editable truth is created.

Migrate inside the existing immediate transaction. Validate schema/scope first,
preserve all members, generations, publications, predecessors, edges, evidence and
legacy FTS data, validate source/revision hashes and foreign-key integrity, and
commit only at the existing completion boundary. Failure/termination rolls back.
Unsupported/corrupt schemas refuse explicitly. Old binaries refuse v2 rather than
silently reading a different representation. No automatic destructive downgrade.

## Public compatibility and normal workflow

Add `pcg_knowledge_v1` to the same profile-controlled registry and session owner.
It is the normal documented architectural workflow. Preserve `pcg_documents_v1`
as an explicitly legacy whole-document/evidence interface, including default and
explicit inline, explicit context source references and exact reconstruction.
Do not route the new interface through a legacy full-source response transformation.

- Search: identity metadata, revision/provenance and typed component match origins
  with exact acquisition references; no automatic source-body materialization.
  SQLite FTS matches admitted component content and explicitly indexed metadata.
  Kind/status/track/component predicates execute in SQLite. No semantic relevance
  inference from a source-level match.
- Get: exact ordered components for one identity, or an explicit typed component
  selection, including required qualifications and provenance. Completeness is
  stated against the requested operation, not inferred applicability.
- Related: actual typed edge tuples, direction, provenance and endpoint identities;
  no complete source bodies for endpoints. No new inferred relationships.
- Context: reuse the existing deterministic typed traversal, separating membership/
  witnesses from legacy materialization. Return all selected units/components,
  evidence associations, typed edges and paths. Existing finite node/edge refusal
  returns no partial authoritative bundle. Preserve exact access to original evidence.
- Source: independently acquire an identity-associated complete original artifact;
  exact source bytes/hash remain available without contaminating proposition search.

All paths reuse existing source observations, profile rechecks, scope/security,
SQLite transaction ownership and explicit failure contracts. Search cannot claim
complete discovery over unmigrated identities. Incomplete selected knowledge is
reported explicitly, never silently omitted. No document token budget is introduced.

## Work scope and verification

The shared row-integrity check validates source bytes once per distinct declaring
artifact and compares exact revision metadata/evidence, without constructing legacy
full-source response objects. Newly published generations still build and verify
both indexes atomically. Stable knowledge operations verify the knowledge index;
explicit legacy operations and the enrolled same-root guard retain their legacy
FTS integrity check. The new interface does not consume or certify the unrelated
legacy search projection. No persistent trust/cache is added and source observations,
complete candidate validation and database constraints remain unchanged.

Operation scope is one query/reconciliation. Source acquisition and complete
candidate validation retain existing boundaries. Marker indexes are built once
per distinct source per operation, not rescanned for each sibling. New publication
deduplicates source ownership and projects components once. Costs scale with
distinct source bytes, declared components, indexed text and selected graph edges;
no persistent cache or new invalidation/lifecycle authority is introduced.
Tracked tests cover deterministic shared-source work counts and semantic isolation.

Use tracked RED/GREEN tests for shared-source isolation, exact/multi-interval content,
match attribution, selector failures, drift, atomicity, migration, closure/witnesses,
legacy compatibility and unrelated budget preservation. Actual-root production
witnesses cover every original query, the eight PassPipeline identities, replay
rejection/counterexample, C# #156 chronology, Angular membership/runtime distinctions,
representative authorities and graph-wide preservation. Existing B8 captures supply
the original PassPipeline witness; B9 captures must establish changed behavior.

The full repository gate and independent evaluations are not presumed green.
Frozen B7 scenarios/answer keys remain untouched and contaminated runs remain UNRUN.
No GLS, runtime pipeline, language producer or host integration change is authorized.
