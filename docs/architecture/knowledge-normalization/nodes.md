# Independently addressable proposition candidates

Agent: Agent MaxHeadRoom. Baseline and provenance: [report](README.md),
[source hashes](inventory.csv), eight `sections-*.csv` indexes.
All identities here are planning reservations, except the existing
`PCG-175-ADOPTION` source declaration. These tables are not graph declarations.

## How to interpret the candidate set

The unit is the proposition in the second column, not the file. Section names
are exact source locators; the section indexes provide baseline line intervals.
An authority candidate's proposed GLS pair is **AUTHORITY / FROZEN**, conditional
on migration ratification of that exact standing contract. This is not an
approval claim or instruction to mark every source FROZEN. A row carrying an
unresolved current contract must remain a proposal until reconciled.

Invariant candidates and source names are enumerated in [invariants.md](invariants.md).
Discovery evidence is enumerated in [findings.md](findings.md). Individually
allocated DOC records in inventory.csv are **RESEARCH / INVESTIGATING** unless
explicitly overridden here; their actual reported outcomes remain attributed in
source prose, not inferred from test-like filenames. Cohesive investigations can
remain whole evidence nodes. Mixed-source allocations are held until the focused
propositions below replace them; they are not aggregate current authorities.

No new CONTRACT, DECISION, EVIDENCE or historical-status taxonomy is introduced.
No report of local GREEN becomes RECLOSED or BANKED. IMPLEMENTS must target a
MISSION, IMPLEMENTATION or AUTHORITY; do not point it at an INVARIANT merely
because an invariant is controlling. PRESERVES can express the latter relationship.

## Foundation, state and public contracts

Paths in this table are relative to the repository root.

| Proposed ID | Proposition and precise scope | Source and section | Disposition / granularity |
| --- | --- | --- | --- |
| AUTH.CORE_OPS | Typed ownership, cardinality, ordered occurrences, checked projection and allowed transformations of canonical CoreOp | docs/architecture/CORE_OP_CONTRACT_MATRIX.md §§2–8 | Keep cohesive matrix intact; invariant aliases constrain its consumers, not another copy of matrix rows |
| AUTH.BINARY_V04 | decode(encode(valid CompiledIR)) equals the whole value, with physical 0x04 strict decoding and evidence-gated compatibility | docs/architecture/BINARY_V04_CONTRACT.md §§1,4–7,9 | Keep intact; historical 0x03 loss in §3 remains evidence inside source |
| AUTH.SEMANTIC_MODEL | Composition of source evidence, neutral resolution, generation freeze, domain refinement, completeness and fidelity | docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md Governing rule; Production model; Acquisition, history, and batches; Completeness authority | Keep short composition contract; specific invariants own their own independent constraints |
| AUTH.QUESTION_MATRIX | A material capability is complete only through real authoritative/current public questions with explicit completeness | docs/Clean-CTX_Canonical_Question_Matrix_Standard.md §§1–3 and required deliverables | Cohesive standard; examples are illustrative, not automatically supported semantics |
| AUTH.LAYER_CERTIFICATION | Declared support requires identity, lifecycle, queryability, adversarial/ecological evidence and dual sign-off; material changes require recertification | docs/LANGUAGE_META_LAYER_CERTIFICATION.md §§2–4,7–8,11–12 | Keep gates together; separate any specific campaign's certification evidence |
| AUTH.FINAL_VERIFICATION | Single authoritative final-gate definition and distinction between tracked tests, operator evidence and final acceptance | docs/agent/verification.md Final Verification Gate; Finalization requirements; AF-PERF-001 mandatory bounded-work gate | Keep intact; hold C15 CI implementation discrepancy, do not alter gate |
| AUTH.TOOL_WORKFLOW | Clean-first discovery/context, explicit workspace root, namespace and exactness rules; provider fallback is operational, not authority | docs/agent/tooling.md §§1–9,11 | Logical declarations can share source initially; broad file cost favors later coherent tool-family split |
| AUTH.SOURCE_ENCODING | Strict UTF-8 without BOM, preserve Unicode and enforce through existing guards | docs/ENCODING_POLICY.md Authoritative rule / location note and enforcement sections | Tracked projection, explicitly subordinate to local encoding rule; do not assert independent overriding authority |
| AUTH.DELTA_SEQUENCE | Explicit dv:2 preserves sequence/version/occurrences; acknowledgement is code-side, never a substitute for complete provide context | docs/COMPILER_IR.md Delta Transport / Corrected Sequence Delta Envelope and State Replay; IRDELTA-001/002/003 | Extract contract from compiler walkthrough; keep delta vs physical binary/hierarchy revisions distinct |
| AUTH.DURABLE_COMPATIBILITY | Restore admits canonical and semantic durable authority only with scoped compatible source/producer evidence | docs/architecture/DURABLE_AUTHORITY_COMPATIBILITY_MIGRATION_PLAN_2026-10-03.md §§2–3, completed phase record; PERSIST-001/002 | Extract current contract, preserve completed migration as implementation evidence; no whole-executable fingerprint policy |
| AUTH.PERSISTENCE_RECOVERY | Source baseline, semantic snapshot, edit intent, replay and scoped deletion recover atomically under their production owner | docs/ARCHITECTURE_OVERVIEW.md Persistence Layer / transactional semantic persistence; Phase9 certification §§1,3 | Extract current lifecycle; audit findings remain historical evidence, not separate success authorities |
| AUTH.PRESENTATION_VNEXT | Model-visible structural grammar is separate from canonical storage, exact bodies and on-demand workspace facts | docs/architecture/SCHEMA_VNEXT_PROPOSAL.md §§2–3, shipped grammar sections | Extract shipped contract; keep candidate experiments/measurements separate, no filename-derived proposal status |
| AUTH.TOKEN_ECONOMICS | Choose among complete alternatives under the same selected tokenizer; raw wins ties; no character-count saving claims | docs/architecture/SCHEMA_VNEXT_PROPOSAL.md §§1–3; docs/ARCHITECTURE_OVERVIEW.md current content boundary | Separate from historical performance percentages and exact PCG token packing; reuse rather than duplicate selector policy |
| AUTH.CONTEXT_BATCH | One to eight ordered failure-isolated files reuse one authoritative evaluator; shared request fields do not imply shared edit authority | docs/architecture/PROVIDE_CODE_CONTEXT_BATCH_MIGRATION_PLAN_2026-09-29.md §§1,3, target contract | Extract exact request/evaluation contract; retain checkpoint history as evidence |
| AUTH.CONTEXT_CHANNELS | Mirrored, structured and indexed output select explicit consumption channels without silent code loss | docs/architecture/PROVIDE_CODE_CONTEXT_BATCH_RESPONSE_MODE_OPTIMIZATION_PLAN_2026-09-29.md §§1–3,9–10 | Keep host capability precondition; no guessed compact mode |
| AUTH.WORKSPACE_BATCH | Requirements union before interpretation; independent outcomes share final acquired generation | docs/architecture/WORKSPACE_QUERY_BATCH_MIGRATION_PLAN_2026-09-28.md §§1,3 and final target contract | Extract from long migration; no truth improvement from item order |
| AUTH.CYCLE_WITNESS | has_cycle is scoped indexed-evidence-only with explicit relation policy and cycle witness, not exhaustive source search | docs/architecture/HAS_CYCLE_WITNESS_PROPOSAL.md Implementation outcome; phase0 audit relation inventory; WSC-007 | Current implementation contract despite proposal filename; exhaustive compilation remains separate |
| AUTH.CACHE_HINTS | MCP cache hints describe exact returned state and differ from proxy HTTP cache_control | docs/CONFIGURATION.md `_meta.cache_hints` Contract and Two Cache Systems | Extract only actual hint/transport semantics; installation guide stays outside graph |
| AUTH.TRUSTED_ROOTS | workspaceRoot establishes authorized roots; withinPath only narrows; returned paths and project slugs are distinct | docs/SECURITY.md Source and workspace boundaries; WSC-004 / CBM-ID-001 | Focused security proposition, no expansion of filesystem authority |
| AUTH.NETWORK_EFFECTS | Core MCP is local stdio; optional CBM and upstream HTTP proxy have separate effects and storage | docs/SECURITY.md Security Posture; Network; Data Handling | Keep explicit effects; no aggregate air-gap/DLP certification |
| AUTH.PROXY_TRANSFORMS | Proxy transforms and tool-output filtering run at their declared request boundary with explicit cache strategy | docs/PROXY.md Request Flow / Transform Pipeline / Tool Output Filtering / Cache Breakpoint Strategy | Extract from platform/auth setup examples; preserve platform-specific evidence as history |
| AUTH.PROXY_SECRETS | Secret scrubbing protects only the proxy boundary and must propagate equal-length substitutions and existing rule triggers | docs/PROXY.md Secret Scrubbing; AF_PERF_002_PROXY_SCRUB_DISCOVERY.md approved comparison contract; AF_PERF_002_PROXY_PREFILTER_DISCOVERY.md approved eight-root scope | Ratify focused contract, not universal secret coverage; Unicode/rule-language completeness remains open |

## PCG contracts and historical stage identities

| Proposed ID | Proposition and scope | Source and section | Proposed metadata / disposition |
| --- | --- | --- | --- |
| AUTH.PCG_GLS | Exact seven kinds, twelve statuses, thirteen directions, identity grammar and whole-candidate constraints | PCG_175_SOURCE_DOCUMENT_GRAPH.md Inspected reference and exact taxonomy; src/pcg/documents/{model,validate}.rs | AUTHORITY/FROZEN after ratification; no new transition matrix, tags or generic edge |
| AUTH.PCG_DOCUMENTS | Source-owned declarations, query-time complete candidate validation, atomic SQLite history/FTS, complete-source context and boundary-observed freshness | docs/architecture/PCG_DOCUMENT_AUTHORING.md all sections | AUTHORITY/FROZEN; cohesive user/contract source; distinguish reconciliation write effect from source mutation |
| AUTH.PCG_PREAUTHORING | Retrieve typed transitive constraints, assess actual applicability, author then reconcile relevant revisions/edges | docs/agent/pcg-authoring.md Retrieve first; Author and reconcile; Unavailable tool or profile | AUTHORITY/FROZEN; procedural authority only, no permission or automatic contradiction verdict |
| AUTH.PCG_ENROLLED_V1 | Explicit owner-enrolled immutable evidence projection with source-attributed approval and deferred runtime ledger/injection | PCG_175_DESIGN_FREEZE.md §§2–6; M2a-1 implementation §B | AUTHORITY/FROZEN only after C02 source disposition reconciliation; not superseded wholesale by document catalog |
| AUTH.PCG_ACQUISITION_V1 | Deterministic applicability/evidence closure with explicit required proof and fail-closed source witnesses | PCG_175_DESIGN_FREEZE.md §7; M2a-2 implementation §§1,3 | AUTHORITY/FROZEN; not source-code hydration or inferred canonical entity proof |
| AUTH.PCG_PROJECTION_V1 | Required evidence precedes optional enrichment; entire response is locally token-metered; required overflow blocks without partial bundle | PCG_175_DESIGN_FREEZE.md §8; M3 implementation §§1,3–5 | AUTHORITY/FROZEN; do not transplant catalog example limits as global policy |
| AUTH.PCG_MCP_V1 | Conditional, explicit, read-only evidence query transports exact metered result bytes with session-pinned scope | M4a implementation §§2,5–6,8 | AUTHORITY/FROZEN; source-document reconcile has different write annotation |
| PCG-175-M2A1 | Enrollment and derived persistence bounded implementation | M2a-1 implementation and acceptance §§A–G | IMPLEMENTATION/IMPLEMENTED proposed; preserve existing named mission; acceptance record is separate evidence, not banking |
| PCG-175-M2A2 | Deterministic evidence acquisition implementation | M2a-2 implementation §§1–7 | IMPLEMENTATION/IMPLEMENTED proposed; scoped results only |
| PCG-175-M3 | Token-bounded context projection implementation | M3 implementation §§1–8 | IMPLEMENTATION/IMPLEMENTED proposed; no independent closure inferred |
| PCG-175-M4A | Conditional evidence MCP integration implementation | M4a implementation §§1–9 | IMPLEMENTATION/IMPLEMENTED proposed; do not duplicate accepted stage as new M4b work |
| PCG-175-M4B | Runtime ledger/integration scope is not frozen; missing authenticated actor/approval/retention contracts | M4b scope reconciliation Exact unresolved decisions | MISSION/PROPOSED proposed; do not use BLOCKED without explicit, correctly disposed blocker endpoints |
| PCG-175-DOCUMENTS | Source-document graph implementation and scoped Rust/stdio evidence | Source document graph Implemented lifecycle and final-source verification | IMPLEMENTATION/IMPLEMENTED proposed; reported 17 tests and operator stdio are distinct, neither rerun here |
| PCG-175-ADOPTION | Portable instructions plus one controlled real-agent authoring/reconciliation sequence | PCG_175_AUTHORING_ADOPTION.md Actual authoring sequence; retained identities | Existing IMPLEMENTATION/IMPLEMENTED declaration, preserve exactly; not universal adoption |

All abbreviated PCG filenames above are under `docs/agent/`. Existing compatible
source-named PCG mission identities are retained in inventory.csv, including
PCG-175-DISCOVERY and PCG-175-DESIGN-FREEZE. No duplicate DOC reservation is
assigned to the same proposition. PCG-175-M2A1-ACCEPTANCE and
PCG-175-M2A1-OPERATOR-RECEIPTS remain separate source-attributed evidence records.
No evidence record inherits a stage's implementation status automatically.

## Neutral and framework authority scopes

Each proposed authority below owns a written/proven source relation, not the
runtime Angular or .NET behavior implied by a familiar name. Proposed kind/status:
AUTHORITY/FROZEN after confirming the source's explicit owner approval and envelope.
Do not promote the entire #162 campaign to validated support from these rows.

| ID | Independently meaningful contract | Source in docs/agent/ (exact title family) |
| --- | --- | --- |
| AUTH.ANGULAR_FOUNDATION | Neutral TS identity, API/declaration proof and Angular-owned interpretation remain separate | ISSUE_162_FOUNDATION_DESIGN_FREEZE.md §§1–10; SEMANTIC_BOUNDARY_REASSESSMENT owner decision |
| AUTH.TS_RETAINED_EVIDENCE | Retain demanded source-qualified TypeScript facets across compilation/durable boundaries; no whole program checker authority | ISSUE_162_RETAINED_AUTHORITY_CONTRACT.md Requirement and bounded scope; MODULE_DECLARATION_EVIDENCE_DECISION approved decision |
| AUTH.ANGULAR_CANONICAL_RELATIONS | Canonical endpoints retain exact declaration identity across relation publication/retraction | ISSUE_162_CANONICAL_RELATION_PUBLICATION_DECISION.md Owner decision and required lifecycle |
| AUTH.ANGULAR_CONSTRUCTOR_TOKENS | Constructor dependency eligibility requires independently proved runtime value/type and native token role | ISSUE_162_CONSTRUCTOR_TOKEN_EVIDENCE_DECISION.md Owner decision and current implementation |
| AUTH.ANGULAR_STRING_TOKENS | Exact source-bound string DI token identity with approved API-surface guardrail | ISSUE_162_STRING_TOKEN_IDENTITY_DECISION.md approval / guardrail and verification |
| AUTH.ANGULAR_MEMBER_IDENTITY | Member projections retain exact declaration identity, independent same-name owners and lifecycle | ISSUE_162_MEMBER_IDENTITY_DECISION.md Proposed bounded decision; Member retirement |
| AUTH.ANGULAR_PUBLIC_ENTRY | Owning package and public entry witnesses establish native API identity, not terminal spelling | ISSUE_162_PUBLIC_ENTRY_AUTHORITY_DECISION.md Located architectural decision; Scope and lifecycle |
| AUTH.ANGULAR_PROVIDER_STRATEGY | Strategy relation endpoints and token registration are separately proved | ISSUE_162_PROVIDER_STRATEGY_IDENTITY_DECISION.md bounded contract and limits |
| AUTH.ANGULAR_PROVIDER_FACTORY | Factory endpoint identifies exact admitted callable declaration without claiming execution/satisfaction | ISSUE_162_PROVIDER_FACTORY_IDENTITY_DECISION.md bounded decision and callable boundary |
| AUTH.ANGULAR_PROVIDER_ALIAS | useExisting joins exact token identities, without runtime winner semantics | ISSUE_162_PROVIDER_EXISTING_DECISION.md contract and limits |
| AUTH.ANGULAR_PROVIDER_VALUE | Supported useValue references admit native reference evidence without generalized evaluation | ISSUE_162_PROVIDER_VALUE_REFERENCE_DECISION.md bounded W04 V1–V4 contract |
| AUTH.ANGULAR_DIRECTIVE_PROVIDER | Direct native Directive declaration may assert its own written providers | ISSUE_162_PROVIDER_DIRECTIVE_OWNER_DECISION.md P1.D1 contract |
| AUTH.ANGULAR_VIEW_POSITION | Component viewProviders position is distinct from providers membership, not effective visibility | ISSUE_162_VIEW_PROVIDER_POSITION_DECISION.md bounded P1.VP1 contract |
| AUTH.ANGULAR_ARRAY_COMPOSITION | Supported provider-array structural composition retains source path and owner evidence | ISSUE_162_PROVIDER_COMPOSITION_DECISION.md structural-authority boundary |
| AUTH.ANGULAR_IMPORTED_ARRAY | Imported provider-array membership requires source-qualified import/export/declaration proof | ISSUE_162_PROVIDER_IMPORTED_COMPOSITION_DECISION.md bounded C2 contract |
| AUTH.ANGULAR_OWNER_COMPOSITION | Directive/NgModule composed membership preserves native owner role independently | ISSUE_162_PROVIDER_OWNER_COMPOSITION_DECISION.md bounded C3 contract |
| AUTH.ANGULAR_ROUTE_PROVIDERS | Exact route object's written provider membership does not establish activation or hierarchy | ISSUE_162_ROUTE_PROVIDER_DECISION.md P1.R1 decision and limits |
| AUTH.ANGULAR_CONFIG_PROVIDERS | Native ApplicationConfig written provider membership does not select active application | ISSUE_162_APPLICATION_CONFIG_PROVIDER_DECISION.md bounded decision and bootstrap context |
| AUTH.ANGULAR_BOOTSTRAP_REFERENCE | Source-declared bootstrap root/config references require exact call/API/declaration proof | ISSUE_162_BOOTSTRAP_REFERENCE_DECISION.md bounded contract; discovery decision remains separate |
| AUTH.ANGULAR_INLINE_BOOTSTRAP | Inline bootstrap provider membership preserves exact source owner and ordinary arguments | ISSUE_162_BOOTSTRAP_INLINE_PROVIDER_DECISION.md bounded architectural decision |
| AUTH.ANGULAR_CONFIG_COMPOSITION | Named merge inputs prove written composition only, not runtime merge/precedence | ISSUE_162_CONFIG_COMPOSITION_DECISION.md bounded contract |
| AUTH.ANGULAR_NESTED_CONFIG | Named merged configs may be written inputs with exact identity/cycle handling | ISSUE_162_NESTED_CONFIG_INPUT_DECISION.md bounded contract; gen71 lifecycle |
| AUTH.ANGULAR_LEGACY_BOOTSTRAP | Native factory/member call argument0 proves written NgModule association | ISSUE_162_LEGACY_BOOTSTRAP_DECISION.md bounded decision; no legacy discovery expansion |
| AUTH.ANGULAR_EAGER_COMPONENT | Ordinary component field selects exact native Component declaration; path is presentation | ISSUE_162_ROUTE_COMPONENT_MEMBERSHIP_DECISION.md bounded decision |
| AUTH.ANGULAR_LAZY_COMPONENT | Admitted loadComponent callback selects exact public export with independent native role | ISSUE_162_LAZY_ROUTE_COMPONENT_DECISION.md bounded decision; no runtime loading meaning |
| AUTH.ANGULAR_LOAD_CHILDREN | Ordinary loadChildren written selection distinguishes native Module and RouteConfig targets | ISSUE_162_LOAD_CHILDREN_DECISION.md bounded decision; 85-file timeout remains open |
| AUTH.ANGULAR_CAN_ACTIVATE | Exact canActivate Array membership is written use, not guard execution/applicability | ISSUE_162_ROUTE_GUARD_MEMBERSHIP_DECISION.md approved written-use contract |
| AUTH.ANGULAR_GUARD_KINDS | Four additional guard kinds have separate explicit relations, not an umbrella union | ISSUE_162_ROUTE_GUARD_KINDS_DECISION.md approved relations / public kind distinction |
| AUTH.ANGULAR_RESOLVER | Unqualified ResolvedBy written membership does not answer public key identity | ISSUE_162_ROUTE_RESOLVER_MEMBERSHIP_DECISION.md approved repair; later gen78 bounded acceptance |

Resolver admission is held on C04 disposition reconciliation. No `AUTH` candidate
here grants new discovery, runtime evaluation, effective injector visibility or
full framework certification. C#/EF/NgRx/template controlling invariant candidates
already exist in invariants.md; their implementation records receive evidence
identities instead of competing AUTH copies. Cohesive Spring reference remains
an explanatory producer/query boundary, with its lifecycle constrained by the
shared semantic invariants.

## Negative evidence and decisions that must remain independently recoverable

| Proposed ID | Exact proposition / source | Kind / proposed status | Admission hold or scope |
| --- | --- | --- | --- |
| EXP-001-A2 | COMPACT-A2 production encoding fails raw-source economics; verification/context-compression/compact-a/PHASED_A2_PLAN.md Status and outcomes | EXPERIMENT/FALSIFIED | Subject is suitability as production encoding, not all codec correctness |
| EXP-002-TABLE | Scoped type/callee table increases complete payload on measured rows; A3_PHASE3C_TABLE.md Outcome / Decision | EXPERIMENT/FALSIFIED | Retain negative result and removal; do not reconstruct table |
| EXP-003-REPLAY | Earlier selected Candidate text is equivalent to later replay presentation under intervening alias deletion | EXPERIMENT/FALSIFIED | AF_PERF_002_REPLAY_RENDER_DESIGN.md Executed design; counterexample falsifies equivalence only |
| DOC-188 | Executed replay design investigation and semantic counterexamples | RESEARCH/INVESTIGATING | Keep source intact as evidence; separate from EXP-003's refuted hypothesis |
| DOC-165 | Owner rejects replay reuse, retires apparatus, defers two measured Git candidates | RESEARCH/INVESTIGATING | GITDIFF_DISCOVERY final decision; exact no-repair propositions below |
| PERF-001-REPLAY | No replay reuse repair is queued under retained later alias/footer observation semantics | MISSION/FROZEN | Proposed explicit owner no-repair decision source DOC-165; no optimization authorization |
| PERF-002-STARTUP | Preserve startup projection; count-removal rejected, projection reuse deferred | MISSION/FROZEN | STARTUP_PROJECTION_DESIGN final return; do not map DEFER/REJECT to new GLS statuses |
| PERF-003-OVERLOAD | Signature-array retrieval with find-then-copied replaces initial FIFO proposal within accepted residual cost bound | IMPLEMENTATION/IMPLEMENTED | OVERLOAD_REPAIR final composed result; dense same-signature equality excluded |
| PERF-004-DOCTRINE | Optimize only demonstrated material benefit with preserved semantics and proportional total complexity | AUTHORITY/FROZEN | AGENTS.md AF-PERF-002; SYSTEM_WIDE_INVESTIGATION Governing owner principle |
| PERF-005-BOUNDED | Modified semantic operations declare scope/scaling/reuse and protect deterministic work bounds | INVARIANT/FROZEN | AGENTS.md AF-PERF-001; AF_PERF_001_DESIGN and verification mandatory gate; no blanket exception |
| EXP-004-CROSSROOT | General symmetric cross-root DbContext hypothesis is falsified; original field cause remains insufficient | EXPERIMENT/FALSIFIED | ISSUE_146_CROSS_ROOT investigation final disposition, not a successful repair or attribution of company cause |
| DOC-371 | A5 cloud account/cache environment failed before product acceptance | RESEARCH/INVESTIGATING | Preserve failed execution; DOC-369 is a later different environment/run, not a status rewrite |
| ADR-001 | Owner-aware file-local Calls inspection for Option C | AUTHORITY/FROZEN proposed | FORWARD_THINKING_OPTION_C.md ADR-001, not the entire partly implemented document |

Protected repair records in the performance inventory retain exact source outcome
and final section in the section index. Each can become a scoped IMPLEMENTATION /
IMPLEMENTED node after its proposition/source record is ratified: A1 redundant
handler clone, A3 lazy buckets, A4 original-input lookup, NgRx collector, C1 count
reuse, hierarchy placement, scrub comparison/prefilter, stats format, edit exact-end,
diff shortcut/source ownership, overload retrieval, encoder borrowing/reverse map,
SQLite text ownership, manifest demand and delta delayed capture. None implies
latency gains, whole-system discovery completion or acceptance of deferred paths.

## Relationship admission boundary

[relationships.md](relationships.md) lists supported directions and explicit holds.
Do not synthesize a complete edge graph from this taxonomy or from shared words.
For each actual declaration, preserve source location/witness and ensure every
endpoint is admitted in the same candidate or an earlier batch. No dependent
implementation may use a refuted or superseded proposition as a necessary premise.
