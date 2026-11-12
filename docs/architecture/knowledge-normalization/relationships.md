# Typed relationship plan and admission holds

Agent: Agent MaxHeadRoom. Direction is always **source -> target**. Every edge
below is proposed, not admitted. Source sections name why the edge holds; shared
keywords, directory proximity, timestamps and filenames establish no edge.

These are the highest-value supported relationships for foundational admission
and the documented historical chains. The inventories do not imply edges between
all allocated source records. Unlisted edges remain unestablished, not absent.
Source paths and exact section intervals resolve through inventory.csv and the
eight section indexes. Proposed node identities resolve through nodes.md,
invariants.md, findings.md and inventory.csv; conflicting umbrella reservations
are held rather than admitted as duplicate propositions.

## Foundational constraints and consumers

| Source -> typed relation -> target | Evidence and exact meaning | Batch / condition |
| --- | --- | --- |
| AUTH.BINARY_V04 DEPENDS_ON AUTH.CORE_OPS | Binary contract §2 explicitly preserves CoreOp identity/cardinality; §1 includes exact duplicate occurrences | B1; complete value equality, not mere opcode presence |
| AUTH.BINARY_V04 PRESERVES IRWIRE-001 | Binary governing round-trip equality realizes the named invariant without changing it | B1 |
| AUTH.CORE_OPS PRESERVES IRPAT-001 | Matrix Transformation contract requires surviving declaration identities; consumptive pattern changes cannot orphan references | B1 |
| AUTH.DELTA_SEQUENCE PRESERVES IRDELTA-001 | Compiler IR Corrected Sequence Delta Envelope and invariant require exact sequence/duplicate replay | B1/B2 |
| AUTH.DELTA_SEQUENCE PRESERVES IRDELTA-002 | Explicit delta result remains code-side; provide context remains complete presentation | B2 |
| AUTH.PRESENTATION_VNEXT PRESERVES ARCH-003 | Shipped contract §§2–3 keeps canonical authority separate from model text | B1/B2 |
| AUTH.PRESENTATION_VNEXT PRESERVES CONTENT-001 | Visible content exactness describes actual text, not auxiliary IR | B2; no complete-source implication from High fidelity |
| AUTH.TOKEN_ECONOMICS DEPENDS_ON AUTH.PRESENTATION_VNEXT | §3 selects among complete alternatives and preserves exact bodies/required meaning | B2 |
| AUTH.SEMANTIC_MODEL PRESERVES AUTH-001 | Governing rule explicitly prohibits manufacturing either side of an authority join | B1 |
| AUTH.SEMANTIC_MODEL PRESERVES WSC-012 | Model C section separates grouping tuple, occurrence and exact declaration identity | B1 |
| AUTH.SEMANTIC_MODEL PRESERVES WSC-013 | Acquisition/history/batches section explicitly requires history-independent truth | B1 |
| AUTH.SEMANTIC_MODEL PRESERVES WSC-014 | Completeness authority names requirements/scope/generation/dependencies, not populated caches | B1 |
| AUTH.SEMANTIC_MODEL PRESERVES SEMDEP-001 | Validity and lifecycle section binds derived validity to contributing dependencies | B1 |
| AUTH.DURABLE_COMPATIBILITY PRESERVES PERSIST-002 | Frozen migration invariants and scoped compatibility gates reject incompatible authority restoration | B2 |
| AUTH.PERSISTENCE_RECOVERY PRESERVES PERSIST-001 | Checkpoints remain policy; accepted durable mutations and scoped recovery keep their authority | B2; extract current contract before admitting old overview text |
| AUTH.CONTEXT_BATCH PRESERVES MCP-003 | Batch plan requires one existing evaluator per item, preserving singular evaluation authority | B2 |
| AUTH.CONTEXT_CHANNELS DEPENDS_ON AUTH.CONTEXT_BATCH | Response-mode plan changes serialized channels for the existing batch, not file evaluation | B2 |
| AUTH.CONTEXT_CHANNELS PRESERVES MCP-001 | Explicit modes preserve host-consumable result-envelope completeness | B2 |
| AUTH.WORKSPACE_BATCH PRESERVES WSC-009 | Migration contract isolates outcomes and shares post-acquisition generation | B2 |
| AUTH.CYCLE_WITNESS PRESERVES WSC-007 | Scoped witness contract is indexed evidence, not complete workspace compilation | B2 |
| AUTH.TRUSTED_ROOTS PRESERVES WSC-004 | Security source-boundary table forbids widening trusted scope through withinPath | B2 |
| AUTH.LAYER_CERTIFICATION DEPENDS_ON AUTH.QUESTION_MATRIX | Certification Gate2 explicitly requires the canonical question matrix | B1 |
| AUTH.LAYER_CERTIFICATION PRESERVES INV.MECHANISM_REUSE_001 | Governing Principles explicitly imports shared-mechanics/specialized-semantics doctrine | B1 |
| AUTH.LAYER_CERTIFICATION PRESERVES INV.META_LAYER_COMPLETENESS_001 | Certification definition and supported envelope require actual authoritative public support | B1 |
| AUTH.QUESTION_MATRIX PRESERVES ARCH-004 | Completion requires real production query reachability, not implemented internal state | B1 |
| PERF-004-DOCTRINE PRESERVES PERF-005-BOUNDED | AGENTS AF-PERF-002 explicitly does not weaken AF-PERF-001 gates or permit authority changes | B5; only doctrine, not universal system acceptance |

## PCG production contract chain

| Source -> typed relation -> target | Source evidence / scope | Batch / condition |
| --- | --- | --- |
| AUTH.PCG_DOCUMENTS DEPENDS_ON AUTH.PCG_GLS | Authoring grammar imports exact GLS vocabulary/candidate consistency; source graph report enumerates it | B3; both endpoints together if newly extracted |
| AUTH.PCG_PREAUTHORING CONSUMES_AUTHORITY AUTH.PCG_DOCUMENTS | Procedure invokes existing document API and reconciles source declarations; cannot invent graph permissions | B3 |
| AUTH.PCG_DOCUMENTS PRESERVES ARCH-003 | Catalog holds document knowledge separately from canonical source IR and source-code tools | B3 |
| AUTH.PCG_ACQUISITION_V1 DEPENDS_ON AUTH.PCG_ENROLLED_V1 | M2a-2 consumes the existing immutable admitted evidence projection, without enrollment redesign | B3 |
| AUTH.PCG_PROJECTION_V1 DEPENDS_ON AUTH.PCG_ACQUISITION_V1 | M3 required closure/applicability is input to token packing; optional enrichment cannot replace required evidence | B3 |
| AUTH.PCG_MCP_V1 DEPENDS_ON AUTH.PCG_PROJECTION_V1 | M4a production lifecycle transports the exact M3 metered payload | B3 |
| PCG-175-M2A1 IMPLEMENTS AUTH.PCG_ENROLLED_V1 | M2a-1 Implementation and ownership traces import, derived persistence and lifecycle | B3 |
| PCG-175-M2A2 IMPLEMENTS AUTH.PCG_ACQUISITION_V1 | M2a-2 Scope and call path / semantics and completeness | B3 |
| PCG-175-M3 IMPLEMENTS AUTH.PCG_PROJECTION_V1 | M3 Implemented boundary / required evidence and optional enrichment | B3 |
| PCG-175-M4A IMPLEMENTS AUTH.PCG_MCP_V1 | M4a Production lifecycle and exact-byte boundary | B3 |
| PCG-175-DOCUMENTS IMPLEMENTS AUTH.PCG_DOCUMENTS | Source document graph Implemented lifecycle and final-source verification | B3 |
| PCG-175-ADOPTION IMPLEMENTS AUTH.PCG_PREAUTHORING | Adoption Entry points and Actual authoring sequence describe instruction integration and one controlled use | B3; new edge proposed, existing source declaration currently has no such edge |
| PCG-175-ADOPTION USES_EVIDENCE PCG-175-DOCUMENTS | Source document graph report provides accepted document-query implementation context used by the controlled adoption sequence | B3; implementation and reported evidence share one cohesive source record, not two duplicate identities |
| PCG-175-M2A1-ACCEPTANCE USES_EVIDENCE PCG-175-M2A1-OPERATOR-RECEIPTS | M2a-1 acceptance explicitly names subsequent operator-receipt corroboration | B3; not independent reclosure or banking |

No blanket SUPERSEDES from document catalog to enrolled v1 is justified. They are distinct
interfaces. No edge from the controlled DOC-081/068/042 scenario to newly named
production binary/CoreOp authorities is admitted; the scenario has explicit
attribution and separate scope, not newly approved production IDs.

## Domain relationships with explicit source support

| Source -> typed relation -> target | Evidence / scope | Batch / condition |
| --- | --- | --- |
| AUTH.ANGULAR_FOUNDATION PRESERVES AUTH-001 | Foundation design separates neutral language proof, API identity and native Angular interpretation | B4 |
| AUTH.ANGULAR_FOUNDATION PRESERVES INV.MECHANISM_REUSE_001 | Mechanism reuse review and accepted question-driven boundary retain semantic owners | B4 |
| AUTH.ANGULAR_CANONICAL_RELATIONS DEPENDS_ON AUTH.TS_RETAINED_EVIDENCE | Module declaration decision and canonical relation decision require source-qualified retained endpoint evidence | B4 |
| AUTH.ANGULAR_STRING_TOKENS PRESERVES AUTH.ANGULAR_CONSTRUCTOR_TOKENS | String-token decision explicitly does not treat earlier constructor approval as primitive-token authorization | B4; preservation of earlier scope, not inference that string requires constructor semantics |
| AUTH.ANGULAR_IMPORTED_ARRAY DEPENDS_ON AUTH.ANGULAR_ARRAY_COMPOSITION | Imported C2 trace says direct/same-source prerequisites passed before imported-stage RED; decision extends source-qualified membership | B4 |
| AUTH.ANGULAR_OWNER_COMPOSITION DEPENDS_ON AUTH.ANGULAR_IMPORTED_ARRAY | Owner C3 trace observes direct-owner and matching composed-Component prerequisites; combined C2/C3 decision covers same imported proof | B4; only imported composed forms, not every direct owner |
| AUTH.ANGULAR_NESTED_CONFIG DEEPENS AUTH.ANGULAR_CONFIG_COMPOSITION | Named-input decision preserves generation70 leaf-input envelope and adds independently proved merge results | B4; no replacement or runtime evaluation |
| AUTH.ANGULAR_LAZY_COMPONENT PRESERVES AUTH.ANGULAR_EAGER_COMPONENT | Lazy decision explicitly preserves generation73 eager semantics and requires separate neutral callback/export proof | B4 |
| AUTH.ANGULAR_LOAD_CHILDREN PRESERVES AUTH.ANGULAR_LAZY_COMPONENT | Shared neutral selection acceptance preserves loadComponent while separately proving Module/RouteConfig native targets | B4 |
| AUTH.ANGULAR_GUARD_KINDS DEEPENS AUTH.ANGULAR_CAN_ACTIVATE | Four-kind decision explicitly preserves canonical canActivate without umbrella union | B4 |
| AUTH.ANGULAR_RESOLVER PRESERVES AUTH.ANGULAR_GUARD_KINDS | Resolver decision preserves guard memberships and does not infer resolver approval from guards | B4; resolve C04 first |
| AUTH.ANGULAR_RESOLVER USES_EVIDENCE DOC-228 | Generation78 bounded acceptance proposal reports later bounded local/binary checks | B4; DOC-228 is source-record identity, not full issue certification |

GEN78 evidence identity above is resolved in inventory.csv; it must refer to
`ISSUE_162_GENERATION78_BOUNDED_ACCEPTANCE_PROPOSAL.md`. If a later file allocation
changes, retain the committed identity map rather than guess a new DOC number.
Provider, bootstrap and route decisions frequently cite the same bounded binary
sessions; do not manufacture independent corroboration by counting each citation
as a separate run. No generic edge from every domain source to every invariant
is proposed.

## Historical replacement, falsification and evidence

| Proposed typed relationship | Repository evidence | Required subject / admission action |
| --- | --- | --- |
| DOC-188 FALSIFIES EXP-003-REPLAY | Replay design Executed design reports later alias deletion changes original footer/text cache while reused earlier text does not | Source evidence is counterexample; target is equivalence hypothesis, not whole replay subsystem |
| PERF-001-REPLAY USES_EVIDENCE DOC-188 | Gitdiff discovery owner rejection/retirement cites failed reuse design and retains focused observation regression | Owner decision is no repair; no request to rescue smaller reuse |
| PERF-002-STARTUP USES_EVIDENCE DOC-196 | Startup design measured projection costs and error-contract consequence of count removal | Keep DEFER and REJECT as prose dispositions with conditions; not new statuses |
| PERF-003-OVERLOAD SUPERSEDES PERF-006-FIFO | Original retrieval design opening explicitly identifies historical FIFO proposal superseded by owner-approved signature arrays | Add PERF-006-FIFO MISSION/SUPERSEDED with exact initial proposal artifact; admit both atomically, exactly one successor |
| DOC-444 USES_EVIDENCE EXP-004-CROSSROOT | Original-field report distinguishes falsified general hypothesis from insufficient original causal boundary | Hold until exact general-hypothesis vs counterevidence subjects are extracted; field cause remains unresolved |
| DOC-434 USES_EVIDENCE DOC-452 | #142 accepted decisions arose from explicitly historical query-history investigation | Evidence direction does not make historical unresolved alternatives necessary truth |
| DOC-421 USES_EVIDENCE DOC-420 | Structured inject repair explicitly resumes preserved blocked evidence gate with new structured prerequisite | Do not set old blocked run to success or assert RECLOSES |
| DOC-427 DEPENDS_ON DOC-428 | Legacy transition repair explicitly consumes class-side source binding established by Prereq2 | Prefer focused prerequisite AUTH extraction before actual declaration; opaque research record alone should not become whole-contract premise |
| DOC-369 USES_EVIDENCE DOC-371 | A5 recovery report retains previous failed cloud environment and states subsequent bounded acceptance | Distinct run/environment subjects; no product FALSIFIES or automatic supersession of the failed receipt |

PERF-006-FIFO is a proposed **MISSION / SUPERSEDED** node for the original
same-name FIFO retrieval proposal in
`docs/agent/AF_PERF_002_OVERLOAD_RETRIEVAL_DESIGN.md`. The complete report remains
historical evidence. Superseding that exact proposal does not supersede unrelated
snapshot/diff policies or make original comparison fixtures current production.

## Relationships deliberately unestablished

- No BANKS, RECLOSES or PRODUCES_AUTHORITY edge is established from descriptive
  “accepted”, “closed”, “protected”, test PASS, agent attestation or owner-reported
  results. Those require exact independent subjects and evidence not inferred here.
- A supported VALIDATES edge requires a specific evidence subject, oracle and
  outcome. A tracked harness README or authored scenario is not a passing run.
  This assessment does not newly authenticate target receipts, so report-level
  USES_EVIDENCE is generally the honest relationship.
- No BLOCKED_BY is inserted merely because prose says blocked, final field pending,
  environment failed or approval missing. Candidate-state rules require concrete
  unresolved endpoints and only RECLOSED/BANKED satisfies them. Preserve holds in
  prose until those subjects exist; do not strengthen metadata to pass validation.
- No wholesale SUPERSEDES of PCG design freeze, all #162 progress, all failed
  investigations, whole performance source corpus or the entire #146 field report.
  Split proposition identities before partial replacements.
- No authority edge to CBM, Roslyn, Angular runtime or remote documentation from
  a name alone. External implementations can corroborate/falsify development
  behavior without becoming production semantic authority.
- No relationship from a new node to every heading in its source. Section indexes
  preserve discoverability, not evidence of semantic dependence.

These holds are part of the migration design. They prevent technically valid
metadata from asserting relationships the repository does not establish.
