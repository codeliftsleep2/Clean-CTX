# PCG-175-DESIGN-FREEZE: first-slice architecture contract

Agent: **Agent MaxHeadRoom**

Parent: Issue #175. Predecessor: [accepted discovery](PCG_175_ARCHITECTURE_INVESTIGATION.md).

Authorization: design text only. **No implementation, tests, migration, staging or publication is authorized.**

Disposition: **DESIGN CORRECTION READY** for owner approval of corrections A-C. Final owner approval remains withheld; no implementation stage is authorized. Every new format, table, command, tool, record and test in this document is **PROPOSED / UNIMPLEMENTED / UNRUN**. “Freeze” means the proposed first-slice specification is reviewable; it does not mean its proposals have already become production policy.

## 1. Source identity and evidence authority

### 1.1 Reference, candidate and publication identities

| Identity | Repository / selected ref | Commit | Tree |
| --- | --- | --- | --- |
| GLS reference | `realworldbooks/GoalLine-Stand-2026`, discovery `main` | `f48228537ac7c44b7c47a71b5cf19c099130e468` | `55eb8a9221cf60cf994094ee39bd56ebbcb8cccc` |
| Clean-CTX discovery reference | `codeliftsleep2/Clean-CTX`, `(fix)Performance-Regression-Issues` | `8ce0a6865ff8a2b55866895c6a239f6bcc3245a3` | `10d8ecadd0a151ea0638e2a066bd50b40ad022e0` |
| Inspected design candidate | Same owner-selected performance branch, resolved during this mission | `8ce0a6865ff8a2b55866895c6a239f6bcc3245a3` | `10d8ecadd0a151ea0638e2a066bd50b40ad022e0` |
| Publication identity | Future owner-approved implementation target | **NOT SELECTED** | **NOT SELECTED** |

**OBSERVED:** GitHub commit/branch APIs verified these identities. The selected branch had not advanced at inspection: candidate-versus-reference relevant drift is empty, so SQLite, dispatch, source identity, provenance, scope, security, tokenizer and canonical-owner findings use one compatible Clean-CTX tree. The clean source mirror's `HEAD` and tree match the candidate; `git diff HEAD -- src proxy` reported no changed files. The accepted discovery's cloud-object comparison covers its 41 inspected files; this mission additionally read current query schemas/dispatch, canonical endpoints, tokenizer and startup seams. Exact pilot artifact hashes/spans were independently obtained from pinned cloud contents in this mission. No branch was switched or modified.

The 230-commit `main` lag is an attributed discovery observation, not a new calculation. The local GLS checkout remains excluded. Its pinned cloud reference is not advanced to a later tip. [GLS reference](https://github.com/realworldbooks/GoalLine-Stand-2026/tree/f48228537ac7c44b7c47a71b5cf19c099130e468), [Clean-CTX candidate](https://github.com/codeliftsleep2/Clean-CTX/tree/8ce0a6865ff8a2b55866895c6a239f6bcc3245a3).

**OBSERVED, DF-CORRECTION (2026-11-10):** the target document exists. Local HEAD/tree still equal the candidate above, with no source/proxy diff. This correction retains the immutable design baseline; it makes no new claim about the live cloud branch tip. Only this design document is modified in the correction mission.

### 1.2 Source anchors and limitations

**OBSERVED, candidate snapshot:** `src/mcp/sqlite_store.rs::{open,migrate,save_context}`, `sqlite_store/semantic_state.rs::save_context_transaction`, `state.rs::McpState::new` and `buffered_store.rs` own existing source persistence. `state/publication.rs::SemanticPublicationTicket::commit` owns source publication ordering. `tool_handlers/query/request.rs::WorkspaceQueryOperation` is a closed eight-operation source query enum. `tool_schemas.rs::workspace_query_operation_properties` and batch variants describe source/entity requests, while `query.rs` can hydrate and publish source-semantic state. `tool_dispatch.rs::{get_registry,dispatch_tools_call}`, `tool_handlers/registry.rs::create_default_registry`, `tools.rs` and `tool_annotations.rs` establish public registration/schema/effect parity. `src/tokenizer.rs` exposes exact local cl100k/o200k and approximate Claude/Llama counters. `layers/meta/canonical_endpoint.rs` distinguishes source/hash/span-backed declarations from invocation/route/string endpoints. These are the compatibility basis for §§2, 7, 8 and 10.

**OBSERVED, GLS reference snapshot:** `architecture_graph.py::{load_graph,validate,check_tree}` and `mission_control_intelligence.py::{report_context,analyze}` support the accepted evidence/metadata distinction. They do not authenticate a runtime approver or certify exhaustive prose coverage. The discovery's GLS trace remains the reference; this mission does not recreate GLS governance in Clean-CTX.

**INFERRED:** A source-owned, explicit evidence projection composes existing responsibilities without extending source IR into decision authority. **UNVERIFIED:** executable test results, live MCP/Claude behavior, receipt payload availability, enterprise security and measured PCG performance. Only reports describing earlier runs have been read; no receipt PASS is reexecuted or independently certified here. Existing source-inspected tests are not newly executed evidence. All remaining normative text is **PROPOSED** unless labelled OBSERVED.

## 2. First-slice responsibility boundary

```text
Local owner profile (scope + exact approved manifest digest)
                      |
Authoritative Git artifacts <-- reviewed sidecar references exact bytes
                      |
             explicit operator import
                      v
       isolated, rebuildable SQLite generation
                      |
    explicit read query -> applicability/closure/witness checks
                      v
          cited, token-bounded MCP result

Canonical IR / source persistence / source publication clocks
          remain independent source-owned systems
```

PCG owns enrolled relationship representation, deterministic retrieval, revision provenance and truthful coverage of the named corpus. Artifacts own decision/evidence prose and its domain truth. The local owner owns enrollment and scope; approval attribution in an artifact remains an assertion of that source. SQLite owns only a derived projection, generation publication and indexes. Canonical IR owns structural source facts and typed endpoints. Existing MCP persistence owns per-file baselines/deltas/semantic restoration and its established deletion/recovery lifecycle. No PCG read invokes restore, replay, semantic hydration, source compilation, cache mutation or pending persistence flush.

First deployment: one OS account, one trusted profile, one canonical repository root, one corpus per query, no cross-corpus edges. Sources are locally available immutable Git blobs in that repository plus their current working-file witnesses. No network access occurs during import/query. The cloud commit supplies pilot identity; cloud GitHub is not a new runtime dependency. The first slice supports source-attributed continuity for other repositories/document subjects using the same format, without AF-PERF-specific evaluator branches.

Deferred: runtime ledger records/writes/approval, autonomous extraction, automatic injection, embeddings, enterprise tenancy, multi-root joins, arbitrary external URI ingestion and independent banking/reclosure execution. A source adapter may preserve descriptive reclosure/banking citations, but cannot issue either decision. No source-semantic/global cache, lock, retry, limit or publication change is part of this contract.

## 3. Source enrollment contract

### 3.1 Choice and trusted local profile

| Alternative | Identity/review/provenance/drift/security/parser tradeoff | First slice |
| --- | --- | --- |
| A: inline declarations | Strong artifact proximity, but changes AF-PERF authorities; fence parsing, source revision churn and declaration review required. Source text still cannot self-authenticate approval. | Defer. |
| B: reviewed sidecar | Stable IDs and exact byte references without rewriting evidence; adds explicit enrollment review and hash maintenance; strict JSON parser; digest pin prevents an agent from silently shrinking coverage. | **SELECT.** |
| C: generated proposal | Reduces initial authoring effort but uncertain applicability/authority; extraction/parser/model revisions and review remain necessary. Never confers approval. | Defer extraction; proposed output could later use B's review boundary. |

Trusted profile v1 is an owner-provided strict JSON object, selected for server reads by `CLEAN_CTX_PCG_PROFILE` containing an absolute path. Omitted variable means PCG is disabled. The profile is separate from evidence and cannot be selected/replaced by a tool request. Its required fields are `schema_version:1`, `partition_id`, `owner_label`, `workspace_id`, canonical absolute `workspace_root`, absolute `database_path`, absolute `manifest_path`, `approved_manifest_sha256`, `policy_revision`, `history_allowed` (boolean), and `limits` (positive integer ceilings `records`, `edges`, `evidence_bytes`, `token_budget`). Unknown fields reject. IDs are case-sensitive ASCII `[A-Za-z0-9][A-Za-z0-9._:-]{0,127}`. No caller-supplied owner/tenant field is accepted. The isolated DB and profile must not be source evidence or source artifacts indexed as claims.

The owner reviews the manifest and pins SHA-256 of its exact UTF-8 bytes in the profile. Editing either document does not cause automatic enrollment. Import requires the exact profile-approved digest; server reads pin a profile fingerprint and check its on-disk bytes/permissions at request start/end. A changed/revoked profile yields `unavailable/profile_changed`, requiring explicit owner restart/reimport as appropriate. Same-OS-account malicious profile replacement is outside the trusted-local threat model, not defeated by this digest. No source-declared `owner`, `approved` or `confidence` field overrides profile trust.

Ordinary queries compare the admitted manifest digest in the generation header with the trusted profile; they do not reparse/rehash the whole sidecar. An unapproved sidecar edit cannot replace the admitted mapping or revoke it implicitly; owner revocation/change is expressed through the trusted profile. Import and explicit audit read the exact approved sidecar and reject absence/digest mismatch. This avoids smuggling full-corpus manifest work into every read without introducing another authority owner or cache.

Proposed maintenance entry point: `clean-ctx pcg import --profile <absolute-path>`; it reads `manifest_path` and never edits it. `--rebuild` constructs the same projection in the same DB transaction after clearing derived rows. Mutually exclusive `--verify-only` performs the explicit read-only full-generation audit in §6.3, without importing or publishing. This remains one operator maintenance surface, not a retrieval MCP operation or scheduled background task. No command is implemented or run by this mission. Server startup opens an existing projection read-only; it never creates, migrates, imports or repairs it.

### 3.2 Closed manifest format

Manifest v1 is strict UTF-8 without BOM. Reject duplicate JSON member names, invalid UTF-8, unsupported version, unknown members, comments, nonfinite numbers and nonintegral offsets. No Unicode normalization. Object order is immaterial to normalized semantic hashes; arrays that represent sets are sorted/deduplicated as specified in §5. Owner approval pins raw bytes, so even whitespace changes require review of a new digest. All source/record/selector/evidence/edge IDs are unique in their own corpus namespace. No executable expressions or document-supplied filesystem roots exist.

| Required top-level field | Exact content |
| --- | --- |
| `schema_version`, `parser_version` | Integer `1`, string `pcg-json-v1`. A parser/schema mismatch rejects import; no best-effort projection. |
| `corpus_id`, `source_namespace` | Stable scoped IDs; namespace is an origin label, not a directory or CBM project. |
| `enrollment` | `{owner_label,policy_revision,complete}`; `complete` boolean is an owner-reviewed corpus declaration, not proof of all prose/world truth. Must match profile attribution/policy. |
| `sources` | Array of `{id,path,commit,blob,sha256,check}`. `path` is repo-relative slash-separated; `commit` and `blob` are full hex Git object IDs; SHA-256 covers exact blob bytes; `check` is `current` or `historical`. |
| `selectors` | Array of `{id,purpose,workspace,file?,entity?,operation?,task?,from,until}`; purpose `coverage` or `applicability`; exact scalar selectors, UTC half-open interval, null unbounded ends. Semantics in §7. |
| `evidence` | Array of `{id,source,start,end,sha256,heading?}`. Half-open UTF-8 byte span; nonempty, scalar-boundary aligned, quote hash exact; optional heading must occur exactly once in the span and start at its first byte. |
| `records` | Array of `{id,role,required,primary,authority,verification,lifecycle,subject,disposition,evidence_refs}`. Role `claim/decision/constraint/evidence`; primary evidence ID; authority `{class,attributed_to}`; verification `{outcome,subject,execution}`; lifecycle enum §4; subject/disposition are exact owner-reviewed labels, not a replacement decision body; evidence_refs array `{id,required}`. No free-form decision body field. |
| `edges` | Array of `{id,kind,from,to?,to_selector?,evidence,gate?}`. Exactly one target form; `gate` only for `BLOCKED_BY`; semantics §4. Provenance evidence ID is mandatory. |

`verification.execution` is `not_run`, `report_of_execution` or `executed_artifact`; the last requires the actual enrolled executable receipt artifact, not a report naming it. `subject` identifies the exact proposition evaluated, e.g. `equivalence` versus `counterexample_detection`. `disposition` is descriptive source vocabulary (`PROTECTED`, `DEFER`, `REJECT`, etc.), never an approval capability. `evidence_refs` need not repeat `primary`; primary is always required. All definitions/endpoints resolve at import, including optional ones; later loss of optional source evidence is handled by §4.3. `USES_EVIDENCE` has no required-edge attribute; mandatory witness selection is exclusively §4.3. A target record's `required` flag never promotes a descriptive edge into required closure.

Each corpus has at least one coverage selector. A binding is an explicit `CONSTRAINS` edge from a decision/constraint to an applicability selector; coverage selectors cannot be binding targets. Exact records queried by ID need not have a binding, but cannot thereby become applicable constraints. No inline expression evaluation, glob, regex, lexical ranking or implicit inheritance is admitted.

At least one source has `check=current`; all such sources share one exact enrollment commit, which becomes `generation.source_commit`. Historical source rows may name earlier commits. Import/resource ceilings debit raw manifest bytes, examined definitions and actual source bytes before unbounded parsing/reads; ceiling failure is explicit, never partial enrollment. Namespace and `enrollment.complete` are inert until the trusted profile's exact digest admits this manifest.

### 3.3 Complete proposed enrollment example and normalization

This is a complete **unimplemented one-record corpus example**, not the complete A-E pilot and not an owner-approved manifest. Hashes/locators are OBSERVED cloud witnesses (§9). Its record IDs and classification/mapping are PROPOSED. It contains no duplicated decision prose.

```json
{
  "schema_version":1,"parser_version":"pcg-json-v1","corpus_id":"af-perf-175","source_namespace":"codeliftsleep2:Clean-CTX",
  "enrollment":{"owner_label":"local-owner","policy_revision":"local-source-attribution-v1","complete":true},
  "sources":[
    {"id":"delta-report","path":"docs/agent/AF_PERF_002_DELTA_PREVIOUS_REPAIR.md","commit":"8ce0a6865ff8a2b55866895c6a239f6bcc3245a3","blob":"606e9da096cb7c272252246296d2d8f8cfa882f3","sha256":"0177375cf4b24b6ddf4b741da1ceff1d94ed840c00a7a0ae60cf4349b30dea51","check":"current"},
    {"id":"delta-owner","path":"src/mcp/tool_handlers/core/delta.rs","commit":"8ce0a6865ff8a2b55866895c6a239f6bcc3245a3","blob":"9b1849de120228c7786ff0124bfcb89c40f32d81","sha256":"b50b5fca0fd8550df1be1910e35e6c536b759b92872819cfcc00dfee1aa46722","check":"current"}
  ],
  "selectors":[
    {"id":"delta-coverage","purpose":"coverage","workspace":"cleanctx","file":"src/mcp/tool_handlers/core/delta.rs","operation":"delta_code_context","task":"modify","from":null,"until":null},
    {"id":"delta-modify","purpose":"applicability","workspace":"cleanctx","file":"src/mcp/tool_handlers/core/delta.rs","operation":"delta_code_context","task":"modify","from":null,"until":null}
  ],
  "evidence":[{"id":"delta-final","source":"delta-report","start":22798,"end":25013,"sha256":"3c95b9640d858ac6963dc5dbb910bf0a41c0c61b272dad29a506d75f0c575e0f","heading":"## Final composed acceptance: CLOSED / FIX / PROTECTED"}],
  "records":[{"id":"delta-capture","role":"decision","required":true,"primary":"delta-final","authority":{"class":"explicitly_approved_decision","attributed_to":"owner approval reported by Agent MaxHeadRoom"},"verification":{"outcome":"supported","subject":"delayed_capture_contract","execution":"report_of_execution"},"lifecycle":"active","subject":"delta_previous_authority_capture","disposition":"PROTECTED","evidence_refs":[]}],
  "edges":[{"id":"delta-capture-binding","kind":"CONSTRAINS","from":"delta-capture","to_selector":"delta-modify","evidence":"delta-final"}]
}
```

Normalized representation is exactly: partition from profile; corpus/name/version; a source revision for each `(source-id,commit,blob,sha256,path,check)`; evidence revision for the primary span; one record revision of `delta-capture`; selectors `delta-coverage` and `delta-modify`; one binding revision; and a generation fingerprint. `primary_text` is derived from bytes `[22798,25013)` of the report, not authored in the manifest. `delta-owner` is a **corpus currentness witness**, not a source body stored by default: all declared sources are validated at import; query witness scope is §7.2. Only enrolled excerpts are persisted. Normalized authority includes `assurance="source_attributed"` regardless of the source's approval label. Freshness is `not_checked` until query validation. Evidence execution remains `report_of_execution`; the 110-pass receipt itself is not magically enrolled.

Reject absolute/traversing paths, `.git` traversal, symlinks/reparse-point escapes, sources outside the pinned repository, Git fetch/promisor fallback, missing blobs, hash mismatch, ambiguous heading, wrong byte range, conflicting IDs, unknown edge vocabulary, invalid intervals or unsupported selectors. Git object reads use argument arrays and literal validated OIDs/paths; never shell-built revspecs or hooks. Current working sources must also be regular files beneath the profile root; Git LF/CRLF conversion is **not** normalized away. A mismatch requires reviewed re-enrollment or historical-only interpretation, not a rewritten hash. Initial/replacement invalid import publishes no generation.

## 4. Semantic and relationship model

### 4.1 Roles and independent dimensions

| Dimension | Stored source-attributed values | Query treatment |
| --- | --- | --- |
| Role | `claim`, `decision`, `constraint`, `evidence` | Determines endpoint/selection rules; evidence records identify propositions about actual artifacts, not an extra authority layer. |
| Origin/authority | `hypothesis`, `unverified_extraction`, `attributable_observation`, `verified_claim`, `explicitly_approved_decision` plus attributed actor | Stored classification; always expose assurance `source_attributed`. No authenticated runtime approval exists in v1. Only reviewed profile enrollment permits reliance on this attribution. |
| Verification | `unknown`, `supported`, `contradicted`, `inconclusive`; exact subject and execution category | Stored as what the source says; not recomputed by executing tests. `supported/equivalence` and `supported/counterexample_detection` are distinct. |
| Lifecycle | `active`, `superseded`, `retracted`, `falsified` | Stored source assertion, validated with links. `falsified` retains historical result; it does not erase evidence. |
| Freshness | No persisted “current truth” flag | Derived each read: `current`, `stale`, `missing`, `inaccessible`, `invalid`, `not_checked`. Required old decisions remain visible as blocked obligations when stale. |

`required` means necessary to the selected operation's context, not permission to act. A required applicable decision/constraint must be emitted with complete source span/conditions or the result cannot be complete. Optional records cannot override controlling requirements. Proposal/claim origins cannot satisfy approval gates. Independent reclosure/banking have no executable v1 status shortcut: preserve their actual evidence citations and distinct subjects; `VALIDATES` cannot establish either.

### 4.2 Relation contracts

All edges have source evidence and immutable endpoint revisions; applicability is checked before root selection. Once required closure selects an edge, its target is included even if its own selector differs: a prerequisite cannot disappear through applicability filtering. An explicit out-of-corpus/root prerequisite is unsupported and blocks import rather than being silently severed.

| Kind / direction / endpoints | Meaning, required closure and lifecycle | Negative / cycle control |
| --- | --- | --- |
| `DEPENDS_ON`: decision/constraint/claim -> decision/constraint/claim | Necessary premise. Required source forces target and recursive required closure. Target must be active, current, noncontradicted, and have at least source-attributed observation authority; hypotheses/unverified extraction cannot discharge a premise. | Descriptive evidence does not substitute. Dependency cycles are import-invalid; no fixed-point semantics. |
| `BLOCKED_BY`: decision/constraint/claim -> claim/decision/constraint | Unresolved gate. Gate is exactly `{outcome:"supported",subject:<exact>,authority:<class>}`, with class one of `attributable_observation`, `verified_claim`, `explicitly_approved_decision`. Satisfaction requires matching target verification subject, active/current target and authority at least that class in this explicit order. Target and gate evidence are required. | No status-name or confidence shortcut; hypothesis never satisfies. Combined dependency/blocker cycles reject import. |
| `CONSTRAINS`: decision/constraint -> applicability selector | Declares applicability binding, not an extra record prerequisite. Every matching active `required:true` bound record is a required root. | No bound inferred from title, proximity or namespace; unknown matching blocks completeness. Not a record-cycle edge. |
| `USES_EVIDENCE`: any record -> evidence record | Always descriptive. Neither this edge nor its target's `required` flag creates mandatory closure. Required witnesses must be named by primary/explicit `evidence_refs.required=true`, or be required independently by §4.3. The target record/edge may be offered as optional support. | A failed experiment may be cited; no premise, verification or approval promotion. Descriptive cycles terminate by visited IDs. |
| `VALIDATES`: evidence/claim -> claim/decision/constraint | Source asserts support for exact target revision/subject. Incoming verification must be reported when its target is required, including its primary/required evidence. | Does not mutate target state, authorize implementation or reclose/bank; a detector may validate divergence rather than equivalence. |
| `FALSIFIES`: evidence/claim -> claim/decision/constraint | Source asserts counterevidence to target; incoming falsifiers are mandatory for a required target. Active contradictory support must be exposed, not ranked away. | No automatic erasure or approval; a falsified target can remain history. Cycles never imply refutation is resolved. |
| `SUPERSEDES`: decision/constraint/claim -> same role | Source claims current control over same `subject`; old target must be `superseded`, effective applicability must explicitly cover the relevant overlap, and successor attribution must meet profile policy. | Exactly one applicable active head for a single-valued subject; cycles/forks/conflicting overlap reject import. History is never a required premise merely because superseded. |
| `PRESERVES`, `IMPLEMENTS`: descriptive record -> record | Imported respect/realization relationship with evidence. Optional history unless provenance of a required relation/condition. | Neither is a prerequisite, successful verification, reclosure or banking. Cycles allowed with visited traversal. |

No caller may choose a weaker closure edge set. `constraints`, `closure`, `impact` and `provenance` all collect mandatory incoming validation/falsification and explicit conflict peers, then their required evidence. Opposing `VALIDATES`/`FALSIFIES` assertions on the same subject are an unresolved conflict unless a source-backed lifecycle/supersession explicitly resolves them. Distinct decision subjects can coexist; same-subject/disposition records retain their distinct logical IDs while identical citation payloads are emitted once. No record receives independent approval merely because a second source repeats it.

Required **historical evidence** is not a required **true premise**: a falsified claim may be emitted to explain a rejection without blocking that rejection. Only controlling active authority and DEPENDS_ON/BLOCKED_BY satisfaction require the corresponding active/noncontradicted state. A historical detector result can be valid evidence of past divergence while the equivalence proposition remains falsified.

### 4.3 Correction A: single mandatory-evidence closure

**ME-01 (normative):** `USES_EVIDENCE` is always descriptive. Mandatory evidence is the least fixed point below, computed independently of optional support/history and before token packing. No new manifest edge attribute or SQL column is added. `record.required` selects applicable decision/constraint roots under §7; it is not an instruction to pull every evidence record in the corpus, or to promote a descriptive target.

```text
mandatory_closure(required_roots, required_bindings, mode):
  M_records := required_roots; M_edges := required_bindings; M_evidence := empty
  repeatedly process each new required record/relationship once:
    record r -> add r.primary and each evidence_ref where required=true to M_evidence
    DEPENDS_ON/BLOCKED_BY from r -> add edge and target record (with its satisfaction check)
    incoming VALIDATES/FALSIFIES to r's exact proposition revision -> add assertion edge
      and its source record as required explanation, not as a true premise or approver
    required conflict/supersession/impact paths selected by the mode -> add their edges/records
    every edge/binding in M_edges -> add its provenance evidence to M_evidence
    never follow USES_EVIDENCE, PRESERVES or IMPLEMENTS as a mandatory edge
  deduplicate immutable evidence revisions and produce required source witness set
  validate each M_evidence revision once; encode/reuse its shared citation if deliverable
  evaluate optional refs/descriptive paths separately, without weakening M_* or its coverage
```

Required paths in the fourth step are precisely those establishing applicable controlling heads/conflicts or the requested reverse-impact result in §7; it is not an open-ended relevance rule. Provenance mode traces this mandatory proof graph; descriptive paths remain optional unless a record is explicitly requested as an additional required root. A cycle in descriptive/assertion explanation traversal terminates through visited sets without asserting premise satisfaction. Validation of an evidence revision means checking immutable source/span/hash/permission; §7's two working-file observations are separate source-freshness checks, not repeated evidence revision validation. Shared evidence bytes are reused only within the query.

| Witness or relationship | Mandatory treatment |
| --- | --- |
| Required record primary; explicit `required:true` ref | Always included, even if no USES_EVIDENCE edge exists. |
| Optional ref; USES_EVIDENCE to optional or `required:true` evidence record | Optional unless independently selected by this algorithm. To require its witness, put that exact evidence ID in the required record's `evidence_refs` with `required:true`. |
| Traversed required relationship/binding provenance | Always included, including provenance of incoming VALIDATES/FALSIFIES and required impact paths. Optional descriptive edge provenance is needed only if that edge is emitted. |
| Incoming VALIDATES/FALSIFIES | Both are enumerated for every required target revision; assertion source, its primary/required refs and edge provenance are mandatory. No silent removal of counterevidence. |
| Historical evidence supporting an active decision | Mandatory if selected above; current access and exact historical identity still required. Its old failure is evidence for the rejection, not a currently true prerequisite. `includeHistory:false` cannot remove it. |
| Missing/stale/inaccessible/unauthorized/invalid mandatory source or witness | `blocked` with safe diagnostics and evidence coverage false; confirmed projection row corruption is `unavailable`. Work exhaustion before determination is `incomplete`. No complete actionable bundle. |
| Mandatory evidence exceeds budget | `blocked/required_budget_exceeded`; no truncation of that witness into a complete answer. Minimal control result or explicit too-small-budget protocol error follows §8. |

**EVIDENCE-01/02:** every emitted required record has its primary and explicit required witnesses accounted for; every emitted required relationship has its provenance accounted for. If the full required bundle is not deliverable, return no partial `required_records`/required relationships as actionable authority; use permitted diagnostic IDs instead. **EVIDENCE-03/04:** descriptive evidence grants neither dependency nor authority, and historical failure stays distinct from premise truth. **EVIDENCE-05:** inaccessible or missing mandatory evidence forbids complete actionable context. **EVIDENCE-06:** validate each immutable evidence revision once and serialize it once per explicitly necessary representation: one canonical citation JSON fragment and its one MCP text-string escaped form. Reuse those operation-local fragments through budget candidates/final emission; do not reserialize the same quote per reference or packing attempt. Only one shared citation occurs in the emitted payload. **EVIDENCE-07:** omission of optional source evidence is observable through §8's omission codes, cannot downgrade required completeness, and cannot promote optional facts. Known projection corruption is a projection failure, not mere optional evidence omission.

Positive examples: (1) replay rejection's primary C-accepted plus explicit required refs C-reject/W-replay are mandatory; `USES_EVIDENCE replay-counterexample` adds no prerequisite. Even if its optional record is omitted, C-reject's required span demonstrates divergence; detector PASS never proves equivalence. (2) two required records and their required edge share one evidence revision: primary, required ref and edge provenance all resolve to one validation/citation; an incoming historical FALSIFIES assertion adds its distinct source witness without requiring the refuted proposition to be true.

Proposed negative oracles (not authored tests): `required_primary_missing_blocks`, `required_ref_missing_without_uses_edge_blocks`, `descriptive_required_target_does_not_promote`, `required_edge_provenance_missing_blocks`, `incoming_falsifier_cannot_be_omitted`, `optional_evidence_omission_preserves_required_completeness`, `shared_evidence_once_per_query`, `divergence_pass_is_not_equivalence`. G3-G5/G8 in §10.1 own these specifications.

## 5. Identity, revisions and provenance

Hash function `H` is SHA-256 of strict UTF-8 canonical JSON: object keys sorted by UTF-8 byte order, no insignificant whitespace, no Unicode normalization, JSON strings escaped only as required for valid JSON, integers decimal, booleans/null standard, and set-valued arrays sorted by their canonical encodings. Source byte hashes never use this JSON transform. These are versioned v1 hash rules, not existing source identity rules.

| Identity | Definition and ownership |
| --- | --- |
| Logical record | `(partition_id,corpus_id,record_id)`; owner assigns stable ID, independent of title/path/hash. |
| Source identity | `(partition,corpus,source_namespace,source_id)`; source ID survives a reviewed rename. |
| Source revision | `H({source_id,path,commit,blob,sha256,check})`; source OID and SHA-256 both verified. |
| Evidence revision | `H({evidence_id,source_revision,start,end,quote_sha256})`; citation is exact half-open bytes; line numbers/headings are orientation, not identity. |
| Record revision | `H({record_id,role,required,primary_evidence_revision,authority,verification,lifecycle,subject,disposition,sorted_evidence_refs})`; no edge recursion in this hash. |
| Selector/binding revision | Selector hash covers exact dimensions/interval/purpose; `CONSTRAINS` revision covers source record revision, selector hash and provenance evidence revision. |
| Relationship assertion revision | `H({edge_id,kind,from_record_revision,to_record_revision,gate,provenance_evidence_revision})`; endpoints resolve within one candidate generation. |
| Generation | `H({partition,corpus,raw_manifest_sha256,policy_revision,parser_version,sorted_source_revisions,sorted_record_revisions,sorted_edge_or_binding_revisions,sorted_selector_revisions})`. No timestamp participates. |

Idempotency key is `(partition,corpus,manifest_sha256,policy_revision,parser_version)`; same key/same normalized fingerprint is a no-op, different fingerprint is invalid/nondeterministic import. A parser upgrade changes the key and generation and requires reviewed version support, never reinterprets old rows silently. The same source body at different origins/partitions stays distinct. Same-name records in different corpora remain distinct; no cross-project hash deduplication.

Nonsemantic edits still change a source witness and dependent record revision if cited bytes/source revision changes; a reviewer may preserve logical IDs but must approve a new manifest digest. A changed decision requires an explicit revised source assertion, correct lifecycle/supersession and reviewed enrollment; it is never a summary overwrite. Rename preserves source/record logical IDs but changes source revision, locators and exact file selectors; old path history remains historical. New missing/deleted sources cause failed import or blocked query, never automatic deletion of a constraint. Removing an enrolled record or coverage rule requires a reviewed replacement manifest; the prior generation is retained only under owner policy, not used as silent current fallback.

For a substantively changed controlling conclusion, enroll a distinct successor record ID plus SUPERSEDES to the predecessor, both represented in the new generation with their exact source revisions. A prior generation's original lifecycle assertion remains unchanged. Same-ID reimport may update nonsemantic witnesses while preserving the conclusion, but must not hide substantive replacement. `includeHistory` exposes only historical/superseded records explicitly enrolled in the requested active generation; it never scans retained older generations under an obsolete approval policy. Missing unenrolled history is not reconstructed.

For permissions, history means superseded/retracted/falsified records or retained `check=historical` source revisions, not a current report's still-valid observation that a past run failed. `includeHistory` controls optional history and explicit historical roots; direct requests for those roots require it true plus profile permission. Required historical evidence supporting a current conclusion is included even when optional history is false, but still requires profile history permission. Withheld required history blocks the current bundle; a caller cannot bypass history policy by naming a record ID or following an edge.

Currentness witnesses: immutable commit/blob membership, source and excerpt hashes, observed profile bytes/permissions, admitted manifest digest compared to profile/header, policy identity, current-file observations for `check=current`, and source-owned canonical endpoint proof when an entity selector is requested. These are identity and boundary observations as §7.2 defines, not continuous-currentness proof. Required `historical` sources use retained local immutable blobs and current history permission; absence means missing history, not inferred truth. A stored old excerpt cannot prove a currently missing source. Supersession preserves prior source assertions; lawful deletion can remove historical projection data because the authoritative source retention policy, not SQLite, owns that obligation.

## 6. SQLite persistence contract

### 6.1 Isolation and proposed DDL

Choose an **isolated PCG database** supplied by the trusted profile. Sharing the existing DB under new table names saves a connection but couples schema versioning, purge/backup, corruption and source-context lifecycle. An isolated file avoids touching `ContextStore`, source baseline replacement, durable semantic transactions and publication clocks. Initial schema version is `1`; SQLite owns derived rows only. Writer uses `foreign_keys=ON`, `journal_mode=WAL`, `synchronous=FULL`, `busy_timeout=0`; no implicit retry. Server opens `SQLITE_OPEN_READ_ONLY`, `query_only=ON`, foreign keys enabled; never use SQLite immutable-file mode against a live WAL DB.

Concrete proposed DDL follows. `p,c,g` mean partition/corpus/generation. JSON fields are strictly validated and canonicalized by the importer; no dependency on SQLite JSON extensions. All declared FKs are enabled. SHA fields are lowercase 64 hex validated by importer as well as length checks; Git OIDs use the repository's verified object format. Cross-table role/endpoint/lifecycle rules in §§3-4 are admission checks in the same transaction, not implied by SQL alone.

```sql
CREATE TABLE pcg_schema(version INTEGER PRIMARY KEY CHECK(version=1));
INSERT INTO pcg_schema VALUES(1);
CREATE TABLE corpus(
 p TEXT NOT NULL,c TEXT NOT NULL,namespace TEXT NOT NULL,
 PRIMARY KEY(p,c)
);
CREATE TABLE generation(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL CHECK(length(g)=64),
 manifest_sha TEXT NOT NULL CHECK(length(manifest_sha)=64),policy TEXT NOT NULL,
 parser TEXT NOT NULL,source_commit TEXT NOT NULL,fingerprint TEXT NOT NULL,
 enrollment_complete INTEGER NOT NULL CHECK(enrollment_complete IN(0,1)),
 PRIMARY KEY(p,c,g),UNIQUE(p,c,manifest_sha,policy,parser),
 FOREIGN KEY(p,c) REFERENCES corpus(p,c) ON DELETE CASCADE
);
CREATE TABLE active_generation(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,
 PRIMARY KEY(p,c),FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g)
);
CREATE TABLE source_revision(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,sid TEXT NOT NULL,srev TEXT NOT NULL,
 path TEXT NOT NULL,git_commit TEXT NOT NULL,blob TEXT NOT NULL,
 sha256 TEXT NOT NULL CHECK(length(sha256)=64),
 check_mode TEXT NOT NULL CHECK(check_mode IN('current','historical')),
 PRIMARY KEY(p,c,g,sid,srev),UNIQUE(p,c,g,sid),
 FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g) ON DELETE CASCADE
);
CREATE TABLE evidence_revision(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,eid TEXT NOT NULL,erev TEXT NOT NULL,
 sid TEXT NOT NULL,srev TEXT NOT NULL,bstart INTEGER NOT NULL CHECK(bstart>=0),
 bend INTEGER NOT NULL CHECK(bend>bstart),quote_sha TEXT NOT NULL CHECK(length(quote_sha)=64),
 quote_utf8 BLOB NOT NULL CHECK(length(quote_utf8)=bend-bstart),
 PRIMARY KEY(p,c,g,eid,erev),UNIQUE(p,c,g,eid),
 FOREIGN KEY(p,c,g,sid,srev) REFERENCES source_revision(p,c,g,sid,srev) ON DELETE CASCADE
);
CREATE TABLE record_revision(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,rid TEXT NOT NULL,rrev TEXT NOT NULL,
 role TEXT NOT NULL CHECK(role IN('claim','decision','constraint','evidence')),
 required INTEGER NOT NULL CHECK(required IN(0,1)),eid TEXT NOT NULL,erev TEXT NOT NULL,
 authority TEXT NOT NULL CHECK(authority IN('hypothesis','unverified_extraction','attributable_observation','verified_claim','explicitly_approved_decision')),
 attribution TEXT NOT NULL,verification TEXT NOT NULL CHECK(verification IN('unknown','supported','contradicted','inconclusive')),
 verification_subject TEXT NOT NULL,execution TEXT NOT NULL CHECK(execution IN('not_run','report_of_execution','executed_artifact')),
 lifecycle TEXT NOT NULL CHECK(lifecycle IN('active','superseded','retracted','falsified')),
 subject TEXT NOT NULL,disposition TEXT NOT NULL,
 PRIMARY KEY(p,c,g,rid,rrev),UNIQUE(p,c,g,rid),
 FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE TABLE evidence_ref(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,rid TEXT NOT NULL,rrev TEXT NOT NULL,
 eid TEXT NOT NULL,erev TEXT NOT NULL,required INTEGER NOT NULL CHECK(required IN(0,1)),
 PRIMARY KEY(p,c,g,rid,rrev,eid,erev),
 FOREIGN KEY(p,c,g,rid,rrev) REFERENCES record_revision(p,c,g,rid,rrev) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE TABLE selector(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,xid TEXT NOT NULL,xrev TEXT NOT NULL,
 purpose TEXT NOT NULL CHECK(purpose IN('coverage','applicability')),
 workspace TEXT NOT NULL,file_path TEXT,entity_json TEXT,operation TEXT,task TEXT,
 effective_from TEXT,effective_until TEXT,normalized_json TEXT NOT NULL,
 PRIMARY KEY(p,c,g,xid),UNIQUE(p,c,g,xid,xrev),
 FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g) ON DELETE CASCADE
);
CREATE TABLE binding(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,bid TEXT NOT NULL,brev TEXT NOT NULL,
 rid TEXT NOT NULL,rrev TEXT NOT NULL,xid TEXT NOT NULL,xrev TEXT NOT NULL,eid TEXT NOT NULL,erev TEXT NOT NULL,
 PRIMARY KEY(p,c,g,bid),
 FOREIGN KEY(p,c,g,rid,rrev) REFERENCES record_revision(p,c,g,rid,rrev) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,xid,xrev) REFERENCES selector(p,c,g,xid,xrev),
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE TABLE relation(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,aid TEXT NOT NULL,arev TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN('DEPENDS_ON','BLOCKED_BY','USES_EVIDENCE','VALIDATES','FALSIFIES','SUPERSEDES','PRESERVES','IMPLEMENTS')),
 src TEXT NOT NULL,src_rev TEXT NOT NULL,dst TEXT NOT NULL,dst_rev TEXT NOT NULL,
 eid TEXT NOT NULL,erev TEXT NOT NULL,gate_json TEXT,
 PRIMARY KEY(p,c,g,aid),
 CHECK((kind='BLOCKED_BY' AND gate_json IS NOT NULL) OR (kind<>'BLOCKED_BY' AND gate_json IS NULL)),
 FOREIGN KEY(p,c,g,src,src_rev) REFERENCES record_revision(p,c,g,rid,rrev) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,dst,dst_rev) REFERENCES record_revision(p,c,g,rid,rrev),
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE INDEX record_subject ON record_revision(p,c,g,subject,lifecycle);
CREATE INDEX selector_scope ON selector(p,c,g,purpose,workspace,file_path,operation,task);
CREATE INDEX binding_selector ON binding(p,c,g,xid,rid);
CREATE INDEX relation_forward ON relation(p,c,g,src,kind,dst);
CREATE INDEX relation_reverse ON relation(p,c,g,dst,kind,src);
CREATE INDEX evidence_source ON evidence_revision(p,c,g,sid,srev);
CREATE INDEX evidence_required ON evidence_ref(p,c,g,rid,rrev,required,eid,erev);
CREATE INDEX source_check ON source_revision(p,c,g,check_mode,sid);
```

`source_commit` is the current repository enrollment anchor, with individual historical source commits retained in source rows. Import metadata/fingerprint/parser/policy/source rows are the rebuild manifest; an additional authoritative ledger or audit-event table is unnecessary. `CONSTRAINS` is stored as `binding`, not a fake subject record or nullable-FK relation. One source ID has one selected revision per generation; enrolling multiple historical revisions of one artifact uses explicit distinct source IDs referencing the same origin path and immutable commits. Identity continuity remains in the origin namespace/record IDs, not path alias guessing.

### 6.2 Transaction and lifecycle algorithms

Import: validate trusted profile, raw manifest digest, syntax, paths, exact Git source bytes/excerpts, secret policy, selectors, records, edges, cycles and conflicts in bounded operation-local memory; capture all source witnesses. Open writer; `BEGIN IMMEDIATE` (busy fails explicitly); validate schema and the active head expected at preparation; insert corpus/generation and source/evidence/record/selector/binding/edge rows; verify all constraints, complete normalized fingerprints/membership and FK consistency; reread each distinct current file once; verify profile/manifest digest again; publish `active_generation` in the same transaction; `COMMIT`. An observed mismatch rejects import, but an unobserved change-and-restore does not. Filesystem observations neither freeze working files through commit nor establish atomic multi-file currentness. Success is returned only after commit.

Reimport: identical key/fingerprint returns the same generation without replacing rows; a different manifest or source replacement produces a wholly separate generation. CAS of the active head must match the head observed before preparation; a stale importer reports `import_head_changed` with no retry. A profile-approved manifest change makes the old generation ineligible for complete queries even before import; there is no fallback to outdated approved evidence. Invalid source replacement/semantic validation rolls back; old rows remain historical but cannot masquerade as the newly requested corpus.

Crash/interruption: before commit, all inserted rows/head changes roll back; after commit, a new process sees the complete committed generation. Tests must kill real fixture import processes before/after commit. A process-local flag or successful SQL call before commit is not durable acceptance. WAL read transactions may continue on a previously pinned valid generation; they report its identity and never claim the newest head. Source/profile changes detected during a read prevent a complete result.

Restart: no runtime ledger recovery. Validate database-open metadata and exact active generation identity as §6.3 specifies, then apply targeted query checks. This is not a full-generation rehash at restart or connection-open. Missing DB/schema, unknown version or detected corrupt rows produce `unavailable`; never auto-create/repair. Explicit rebuild: under one writer transaction delete active pointer and derived corpus generations in FK-safe order (relations/bindings/refs/records/selectors/evidence/sources/generations), rebuild from approved retained sources, publish head and commit. Failure rolls back the entire rebuild, leaving old committed DB intact. Readers pin their old SQLite snapshot; new reads see only the whole new head. Rebuild does not recover missing authoritative artifacts.

Migration: v1 has no implicit upgrade from any existing Clean-CTX schema or unknown PCG version. Future schema changes require a separately approved migration. Because this DB is derived, the initial supported upgrade route is explicit transactional rebuild into a supported schema after owner backup/permission review; never invoke existing `SqliteStore::migrate`. Corruption must be handled by closing affected connections, returning generic unavailable diagnostics and explicit operator replacement/rebuild after readers stop; no automatic database deletion, WAL removal or success fallback. Projection backup is optional recovery convenience; the retained approved manifest/profile and authoritative source revisions are essential.

Concurrency: independent read-only connections use read transactions; imports serialize through SQLite's single writer and head CAS, with zero application retries. Read transactions pin one generation; query SQL always includes `p,c,g`. No read updates metrics/timestamps or flushes persistence. Import/resource ceilings come from the trusted profile and are specific to PCG, not new global query caps. Immutable evidence is validated once per revision; distinct current files are observed once per specified phase, never per referencing record. Only accessed row hashes are recomputed during retrieval. Persistent semantic memoization is not introduced.

### 6.3 Correction C: admission-validated generation, targeted reads

**INTEGRITY-01 (normative): choose Option B.** Fully validate the generation at admission and publish it atomically; each query validates identity plus all accessed applicability/proof rows and required evidence. Do not recompute the full-generation fingerprint on ordinary retrieval or hide that scan in database-open validation. Admission/application checks establish the semantic invariants; SQLite transactions preserve their all-or-nothing publication, not their semantic truth. Generation IDs identify admitted content and do not attest that every stored row remains intact.

| Responsibility | Exact boundary / required checks |
| --- | --- |
| Import | Validate entire manifest/source/record/selector/relationship universe, exact memberships, hashes, closure/cycle/conflict rules and FK consistency; compute complete fingerprint. Reject any failure before publication. |
| Transactional publication | Insert immutable rows and head within one transaction; compare expected head; acknowledge only committed generation. Never edit published generation rows in place. |
| DB open / restart | Read schema version, required table/index definitions, active-head/header, approved manifest/policy/parser/partition compatibility and SQLite errors encountered opening/accessing them. No implicit `integrity_check`/full table scan. Metadata mismatch refuses service; success is not a corpus-integrity certificate. |
| Per-request generation identity | Pin one SQLite read transaction and exact `(p,c,g)`; compare requested generation, active header, profile-approved manifest digest, policy/parser/schema and stored fingerprint identity. Do not recompute all constituent hashes here. |
| Accessed-row verification | Recompute hashes and validate typed structure/partition/references for every examined applicability selector/binding/conflict candidate, required closure record, traversed assertion/ref and selected citation. Confirm endpoint revision/FK references when traversed. Known malformed/missing required rows or hash mismatch are projection corruption -> unavailable. Any optional row actually accessed is checked too. |
| Source/evidence witnesses | Apply §§4.3/7.2; exact immutable required evidence once, all corpus-wide `check=current` witnesses at both working-file boundaries, and required historical sources with permission. No full off-path historical evidence rehash. |
| Explicit full audit | Owner invokes proposed `--verify-only`: read-only pinned generation, complete row/membership/fingerprint/FK/semantic verification against approved manifest and retained immutable evidence. Report all limitations/failures; no repair, new generation, periodic worker or authority-cache receipt. Audit does not grant continuous filesystem freshness. |
| Corruption/rebuild | Any detected projection integrity failure refuses the affected read/handle pending operator recovery. A separate audit failure requires owner stop/disable affected readers and explicit rebuild; no cross-process failure notification is claimed. Rebuild re-admits exact authoritative sources; no automatic success fallback. |

| Alternative | Guarantees / detection / cost / lifecycle |
| --- | --- |
| A: full verification every query | Recomputes all row memberships/hashes against the expected generation and detects off-path changes visible in the pinned snapshot, if verification itself completes. O(total generation data) even for a small query; restart and concurrent imports still use one pinned generation. It does not freeze files or authenticate an approver. |
| B: admission plus targeted verification (**selected**) | Same full admission/atomic publication; per-request proof-row checks, full applicable-universe evaluation and required witnesses. Restart has metadata/targeted checks, not a hidden O(N) audit. Concurrent imports cannot mix generation rows within the read transaction. Unaccessed corruption/tampering may remain undetected until an audit or affected access; no persistent validation cache is introduced. |

**INTEGRITY-02 (limitation):** B does not detect all off-path row corruption, valid-SQL deletion of an otherwise required binding/edge, forged selector prefilter fields, index omissions or tampering that leaves header metadata unchanged. Such damage can affect an absence/closure answer without being discovered by that query; a matching generation ID is not a proof of intact membership. `complete_within_registered_scope` is conditional on the trusted-local, admission-validated generation remaining immutable outside the admitted writer. It is not a cryptographic or full-storage-integrity certificate. Source hashing cannot repair missing graph membership. Stronger untrusted-storage/tamper guarantees would require owner review, not a silent completeness claim or performance exemption. This restriction is exposed as `coverage.projection_integrity="admission_and_accessed_rows"` in §8.

The accepted trusted-local deployment allows only the admitted importer to write the projection; same-account malicious mutation is outside that trust assumption, while accidental/off-path damage remains an explicit detection limit. The evaluator must still exhaust the registered applicability candidates; B authorizes no intentional top-k, skipped binding or partial-index completeness shortcut. If that deployment assumption is unacceptable for the implementation, stop for owner disposition rather than advertise B as tamper-proof.

No schema tables/record/edge attributes change. The two additional design-text indexes in §6.1 bound required-ref lookup and enumeration of current witness sources; existing selector_scope, binding_selector, forward/reverse relation, primary-key and evidence_source indexes remain necessary. Positive controls: original admission/rebuild fingerprint equality, repeated targeted reads with unchanged immutable generation, and pinned old/new readers under concurrent import. Negative controls: `targeted_required_row_corruption_unavailable`, `off_path_corruption_detected_on_audit_or_access`, `removed_binding_not_certified_by_generation_id`, `restart_does_not_rehash_generation`, `concurrent_import_never_mixes_generation`. Off-path tests must record that targeted reads may not detect damage; they must not pretend B provides A's guarantees.

## 7. Applicability, closure and completeness

### 7.1 Scope and exact selectors

Request supplies `workspaceRoot` and corpus ID; root must equal the configured canonical workspace root, not enlarge it. Corpus/partition/owner/policy derive from the profile. The caller must name the exact registered generation; mismatch gives `unavailable/generation_mismatch` rather than silently selecting another tree. Scope is a named enrolled source universe, not every document in a repository or all knowledge relevant to a task.

Subject = `{workspace,file?,entity?,operation?,task?,at}`. File is a validated repo-relative path; operation is an exact MCP operation label (or a source-owned noncoding operation label), task is a reviewed category such as `modify`, `explain`, `investigate`. No lexical synonyms. `at` is explicit canonical UTC `YYYY-MM-DDTHH:MM:SSZ`, interpreted within `[from,until)`; v1 has no implicit wall-clock default. Historical effective-date queries do not bypass current source permissions.

A selector is conjunction of supplied dimensions; several bindings/coverage selectors form a union. Missing selector field means unrestricted on that dimension within this workspace. Missing **request** dimension needed by a selector yields UNKNOWN, not false. A known mismatching dimension makes the conjunction false even if another field is unknown. Coverage requires at least one definitely matching coverage selector AND `enrollment.complete=true`; an unknown potentially applicable required binding still prevents complete output. Coverage describes the reviewed enrolled universe only; it does not certify exhaustive prose extraction.

Evaluate the union per record: any TRUE binding makes that record applicable even if another binding is UNKNOWN; UNKNOWN blocks completeness only when no binding for that same record is TRUE. A FALSE coverage selector cannot certify its scope. These are ordinary three-valued AND/OR rules, not a heuristic confidence threshold.

Canonical entity selector v1 supports **declarations only** using `{kind:"declaration",file,source_sha256,start,end,name,declaration_kind}` mapped exactly to the candidate's `CanonicalDeclaration`. Resolve against existing source-owned current `WorkspaceIndex` canonical endpoints under a read guard, with matching source bytes/hash/span/name/kind; normalize physical path through configured root mapping. Do not invent an entity from a namespace or caller label. No available matching typed endpoint (including after a fresh process before source indexing), ambiguity, or unsupported invocation/route/string endpoint yields `incomplete/identity_unavailable`; no hydration/compilation fallback. File-level bindings still enumerate from the file carried by the entity reference. This explicitly limits first-slice entity readiness while supporting deterministic file/operation/workspace requests across restart without IR acquisition.

Conflict key is source-declared `(subject,role)` for decisions/constraints, with overlap evaluated on selectors/effective intervals. Different active dispositions or verification contradictions in overlapping applicable scope block an actionable result; missing dimensions yield unknown overlap, not permission to ignore a peer. No timestamp/confidence/ranking arbitration. Reviewed explicit supersession with full controlling scope may resolve the conflict. Source-only approval attribution remains visible even when the local owner trusts enrollment.

### 7.2 Deterministic evaluator pseudocode

```text
query(q):
  validate closed request schema; authorize exact root/corpus against trusted profile
  recheck profile bytes/permissions; open read-only SQLite transaction
  pin p,c,g; compare schema/policy/manifest/header identities under INTEGRITY-01
  resolve requested subject and optional source-owned canonical declaration proof
  evaluate coverage selectors and ALL potentially relevant applicability bindings
    SQL may prefilter only definitely unequal dimensions; absent dimensions remain candidates
    verify accessed selector/binding/conflict rows; never full-generation-rehash here
  mark roots: matching active required decisions/constraints; retain unknown required matches
  seed closure/provenance/impact roots from exact record IDs when mode requires them
  compute mandatory_closure from §4.3, with complete required impact paths for impact mode
    verify every accessed required record/edge/ref hash and expected revision; debit once
    detect corrupt required cycles, missing endpoints and conflicting heads
  START witness phase: all enrolled current sources plus required historical sources
    check exact immutable identity/required excerpts once per revision, and permissions
    read/hash each distinct current working file once; record only what was observed
  evaluate gates, lifecycle, currentness, verification subjects and conflicts
  enforce ceilings before expansion/read; incomplete work never becomes absence proof
  order full required bundle; try optional support/history only from remaining work/budget
    descriptive paths cannot enlarge required closure; record optional omission codes
  build/count draft whole result, dropping optional items only
  COMPLETION witness phase: recheck profile and each distinct current file/permission once
    required historical source permission/availability rechecked, immutable bytes reused
  if any required observation now fails: discard draft authority bundle, classify failure
    build/count safe control result; do not retry or serialize the old complete draft
  otherwise classify with explicit observation/integrity limitations and required fit
  serialize/emit exactly counted final result; close transaction
```

`limits.records` debits each visited record plus each examined selector/binding/conflict candidate once; `limits.edges` debits each inspected relation/binding/ref once; `limits.evidence_bytes` debits actual bytes offered by Git/filesystem validation in the prescribed phases, not merely excerpt/output lengths. Read bounded chunks and stop before crossing the ceiling; never read a whole oversized source first. A limit hit during mandatory work sets the affected coverage false and `incomplete/work_limit` unless a known higher-priority block applies. Optional-only exhaustion records omission without changing completed mandatory coverage. No hidden depth/top-k cutoff exists. Ordered IDs, explicit stacks/queues and visited sets determine traversal; SQLite row order is never assumed.

All `check=current` sources remain required **corpus freshness witnesses**, including witness-only code files, regardless of `includeHistory`. This retains the existing corpus-wide freshness invariant: enrollment makes no finer relevance proof allowing an arbitrary witness to be skipped for one record. Their observed drift blocks current completeness even when their quote is not selected. Historical sources are mandatory only through §4.3; optional historical loss is recorded as omission. Imports validate all declared historical sources; ordinary queries do not hash off-path optional historical sources. The cost of checking every corpus-current witness is explicit in §10.3 and may dominate retrieval; targeted row verification must not be advertised as eliminating it. Narrowing this source policy would need an earned relevance contract/owner decision, not an implicit optimization.

**FRESHNESS-01 (normative):** “Currentness was observed against the enrolled source witnesses at the specified retrieval boundaries. This does not establish continuous immutability or an atomic multi-file snapshot.” Immutable Git/source/excerpt identity proves which historical bytes were cited; it does not prove that current working files stayed unchanged. START is after required closure discovery, before source interpretation; COMPLETION is after draft projection/token packing, before final serialization. Source/evidence identity is validated once per immutable revision; current working paths are deduplicated by observed canonical physical source identity and observed once per phase. Conflicting current witnesses for one physical path reject admission; distinct historical revisions of that path remain legal. A detected alias change/escape is invalid. Permission/availability is checked at each applicable phase, not inferred from old SQLite access.

| Observation | Classification and consequence |
| --- | --- |
| Current bytes differ from enrolled witness | `stale`; required corpus freshness fails -> blocked/source_stale. Historical Git blob availability does not cure it. |
| Required path/blob deleted, or permission lost | `missing` / `inaccessible`; blocked with non-disclosing source_missing/source_inaccessible diagnostics. Profile authority loss itself is unavailable. |
| Invalid UTF-8, unsafe path/type, malformed source/span or incompatible identity | `invalid`; blocked/source_invalid, unless the fault is confirmed projection corruption (unavailable). |
| Mandatory observation not completed within work limits | `not_checked`; incomplete, never a successful freshness observation. |
| Failure found after draft retrieval but in COMPLETION | Invalidate complete draft; clear actionable required bundle, classify observed failure (source_changed_during_read plus specific source reason), remeter control result. Do not emit the old draft or loop until it appears stable. |
| Change occurs and restores identical bytes/permissions between reads | May be undetected; both observations can match and a bounded complete result is permitted with the explicit limitation. No deterministic negative oracle may require its detection. |

**FRESHNESS-02 (TOCTOU limit):** changes after a file's last observation, including during final serialization/emission, can escape detection. Two files can each match at their own observation times without ever forming one jointly atomic snapshot; even each individual read is not a filesystem snapshot against an uncooperative concurrent writer. Return the identity of bytes actually hashed and the boundary-only assurance, never continuous/atomic currentness. A future stronger requirement would need a source-provider snapshot/version guarantee or coordinated ownership mechanism; neither is designed or added here. No change to existing Clean-CTX source-currentness, replay, delta or publication mechanisms is authorized.

Required proposed controls and exact expectations: `source_modified_before_start_blocks` (persistent mismatch detected); `source_modified_between_observations_blocks` (mismatch present at COMPLETION detected); `source_change_restore_between_observations_not_claimed_detected` (barrier-controlled unobserved ABA may complete, with continuous/atomic flags false); `source_deleted_during_query_blocks` (absence present at COMPLETION); `source_permission_revoked_during_query_blocks` (revocation effective at an attempted check; restored/unobserved permission changes may escape); `independent_source_observations_are_not_atomic` (detect whichever mismatches are observed, never certify a joint instant); `historical_blob_does_not_cure_stale_working_source`; and `completion_failure_discards_complete_draft`. These are design oracles only, not newly written/executed tests.

### 7.3 Result states and precedence

| State, in highest-first precedence | Exact condition |
| --- | --- |
| `unavailable` | Trusted scope/config/policy permission cannot be established, projection/schema/generation unreadable/incompatible, approved head mismatch or corruption. Emit no evidence payload. Unauthorized root is rejected before lookup with generic `-32602` instead of disclosing corpus existence. |
| `blocked` | Known applicable required authority is stale/missing/inaccessible/invalid, a required premise/gate fails, required cycle/conflict is detected, or the known required serialized bundle cannot fit. Preserve only safe diagnostic context; never imply permission to proceed. Other unknown coverage remains reported. |
| `incomplete` | No known higher-priority block, but enrollment/coverage, identity, applicability, required traversal or evidence checks cannot be completed within limits. Empty/partial results cannot certify absence. |
| `complete_within_registered_scope` | All named corpus/identity/applicability/required closure/observed-freshness/policy checks succeed and the full required bundle fits. Empty result requires exhaustive enrolled-scope applicability evaluation. The assurance assumes intact admission-validated generation membership under INTEGRITY-02 and establishes only FRESHNESS-01 observations, never continuous/atomic currentness or full-storage integrity. It proves no fact outside that boundary and grants no execution authorization. |

An unsatisfied condition discovered before a work limit can yield `blocked` with incomplete coverage; it does not certify all blockers were found. Optional stale history is warning-labelled and omitted if unauthorized; it cannot downgrade a valid complete required bundle unless its contradiction is itself required. `USES_EVIDENCE` never converts evidence into a mandatory prerequisite. Negative controls include a missing task field, omitted required binding, unenrolled file, unknown entity, removed source, hidden history, cycle, stale generation, exact empty covered scope and early traversal exhaustion.

## 8. Context projection and MCP API

### 8.1 Public choice and token contract

| Interface | Candidate source fit / risk | Decision |
| --- | --- | --- |
| Evidence mode in `workspace_query` | Closed source operation enum, source scope/coverage schema and batch hydration/publication path would need non-source branches. Metadata resembling source completeness could mislead consumers. | Do not extend in v1. |
| Dedicated `pcg_query_v1` | One separately versioned read contract; no source hydration, explicit corpus generation and distinct coverage/authority semantics. Requires normal catalog/registry/schema/annotation parity. | **SELECT.** |

Annotations: read-only, non-destructive, idempotent, closed-world hint for the enrolled local corpus; hints are not authorization. Tool accepts no writes/approval/import/maintenance operation. Existing tool requests/responses, dictionary/cache hints, default compression, canonical IR and source SQLite stay unchanged. Only the enabled new catalog entry is added; when PCG is disabled there is no entry. A configured but unavailable projection leaves the entry discoverable and returns unavailable. No automatic context injection or source-facing footers are added.

Required payload is exactly §4.3's mandatory closure: selected controlling constraints/decisions, complete cited directive spans/conditions/exceptions, required proof/counterevidence/provenance, blockers/conflicts, authority/verification/lifecycle/freshness and coverage. Reuse one citation text body per immutable evidence revision. Optional refs/descriptive links/history cannot enlarge that mandatory set implicitly. Record omitted optional categories through `budget.optional_omissions`; `optional_truncated` marks actual known optional packing/work omissions, not a changed mandatory completeness class. Withheld/unrequested history is reported as a category policy, regardless of whether hidden records exist; never disclose their IDs/counts. Source spans must include complete conditions; unverified LLM summaries cannot replace them.

Order: required roots then required dependencies/counterevidence by logical ID; edges by `(kind,from,to,id)`; citations by `(source-id,source-revision,start,end)`; optional support then history by logical ID/revision. No model ranking. Escape source excerpts as JSON data and label them untrusted evidence; never concatenate them into system/tool instructions.

Response uses one MCP text block containing canonical JSON payload; no duplicated `structuredContent`. Publish `inputSchema`; do not advertise MCP `outputSchema` requiring a structured channel absent from this v1 contract. The documented payload schema below governs the JSON inside the text block. Normal tool result is exactly `{"content":[{"type":"text","text":PAYLOAD_JSON}],"isError":false}`. JSON-RPC framing/request ID is not context material; **the entire serialized MCP result**, including escaped payload JSON, wrapper, all metadata/citations and isError, is counted with exact local `cl100k` or `o200k`. Approximate tokenizers and tokenizer initialization failure return a control error; no raw/source fallback.

Budget limit and tokenizer appear in the payload; actual used count is deliberately not inserted into the same counted object (avoids self-referential accounting). Tests/operators count the emitted result independently. Drop optional complete items, remeter the final result, never slice mandatory text. If required bundle does not fit, return `blocked/required_budget_exceeded`, `required_material_fit:false`, coverage and bounded permitted IDs/diagnostics; `required_records` may be empty and the bundle is explicitly undelivered. Diagnostic ID lists may truncate with `diagnostics_truncated:true`; never truncate the required bundle and call it delivered. If even the minimal control result cannot fit, return `-32602/token_budget_too_small` outside the evidence-result budget; this protocol error is no usable context or completeness claim. No pagination/resumption in v1.

Under EVIDENCE-01/02, undelivered mandatory closure leaves `required_records` and its required relationships empty rather than emitting partial actionable authority. Safe diagnostic IDs may describe the deficit. Optional omissions use unique sorted category codes `not_requested`, `budget`, `unavailable`, `work_limit`, with no hidden-source identifier/count. Required omission instead changes result state/coverage/fit; it is never disguised as an optional omission. The count includes the added omission/assurance fields and any reclassified control result.

### 8.2 Proposed request schema

Draft 2020-12; `additionalProperties:false` applies at every object. The exact operation condition is enforced by the `oneOf`: constraints uses only subject; closure/impact/provenance also require records. Generation, policy, corpus IDs and references are never inferred from a local namespace. Byte paths are repo-relative and validated beyond JSON schema as §3 specifies.

```json
{
 "$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","additionalProperties":false,
 "required":["workspaceRoot","corpus","generation","mode","subject","tokenizer","limits","includeHistory"],
 "properties":{
  "workspaceRoot":{"type":"string","minLength":1},"corpus":{"$ref":"#/$defs/id"},"generation":{"type":"string","pattern":"^[0-9a-f]{64}$"},
  "mode":{"enum":["constraints","closure","impact","provenance"]},
  "subject":{"type":"object","additionalProperties":false,"required":["workspace","at"],"properties":{
   "workspace":{"$ref":"#/$defs/id"},"file":{"type":"string","minLength":1},"operation":{"$ref":"#/$defs/id"},"task":{"$ref":"#/$defs/id"},"at":{"type":"string","pattern":"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"},
   "entity":{"type":"object","additionalProperties":false,"required":["kind","file","source_sha256","start","end","name","declaration_kind"],"properties":{
    "kind":{"const":"declaration"},"file":{"type":"string","minLength":1},"source_sha256":{"type":"string","pattern":"^[0-9a-f]{64}$"},"start":{"type":"integer","minimum":0},"end":{"type":"integer","minimum":1},"name":{"type":"string","minLength":1},"declaration_kind":{"type":"string","minLength":1}
   }}
  }},
  "records":{"type":"array","minItems":1,"uniqueItems":true,"items":{"$ref":"#/$defs/id"}},
  "tokenizer":{"enum":["cl100k","o200k"]},"includeHistory":{"type":"boolean"},
  "limits":{"type":"object","additionalProperties":false,"required":["records","edges","evidence_bytes","token_budget"],"properties":{
   "records":{"type":"integer","minimum":1},"edges":{"type":"integer","minimum":1},"evidence_bytes":{"type":"integer","minimum":1},"token_budget":{"type":"integer","minimum":1}
  }}
 },
 "oneOf":[{"properties":{"mode":{"const":"constraints"}},"not":{"required":["records"]}},{"properties":{"mode":{"enum":["closure","impact","provenance"]}},"required":["records"]}],
 "$defs":{"id":{"type":"string","pattern":"^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$"}}
}
```

Semantic request validation: real UTC calendar/time, `end>start`, entity/file consistency, unique IDs, configured limits and profile root; no caller-supplied corpus/partition expansion. `includeHistory:true` with history disallowed is a generic permission error, not silently ignored. Unknown record IDs return `incomplete/record_not_enrolled` after authorized scope validation, not complete emptiness.

### 8.3 Proposed payload schema

The following schema defines all payload fields and closed record/citation/issue shapes. `required_records` and `support/history` carry typed record objects, not executable instructions. A canonical empty `required_records` never alone establishes complete state. Evidence text occurs once in `citations`; each required record cites its primary and required reference IDs. All arrays are present, even when empty; no null array masquerades as not-checked.

```json
{
 "$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","additionalProperties":false,
 "required":["version","scope","generation","authority_policy","subjects","required_records","support","history","relationships","conflicts","missing_stale_requirements","coverage","budget","citations","result_state","reasons"],
 "properties":{
  "version":{"const":1},"scope":{"type":"object","additionalProperties":false,"required":["partition","corpus","workspace"],"properties":{"partition":{"type":"string"},"corpus":{"type":"string"},"workspace":{"type":"string"}}},
  "generation":{"type":["string","null"]},"authority_policy":{"type":"string"},"subjects":{"type":"array","items":{"type":"string"}},
  "required_records":{"$ref":"#/$defs/records"},"support":{"$ref":"#/$defs/records"},"history":{"$ref":"#/$defs/records"},
  "relationships":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["id","revision","kind","from","to","citation"],"properties":{"id":{"type":"string"},"revision":{"type":"string"},"kind":{"enum":["DEPENDS_ON","BLOCKED_BY","CONSTRAINS","USES_EVIDENCE","VALIDATES","FALSIFIES","SUPERSEDES","PRESERVES","IMPLEMENTS"]},"from":{"type":"string"},"to":{"type":"string"},"citation":{"type":"string"}}}},
  "conflicts":{"$ref":"#/$defs/issues"},"missing_stale_requirements":{"$ref":"#/$defs/issues"},"reasons":{"$ref":"#/$defs/issues"},
  "coverage":{"type":"object","additionalProperties":false,"required":["enrollment","identity","applicability","required_closure","evidence","currentness_boundary","freshness_guarantee","continuous_currentness","atomic_filesystem_snapshot","observations","projection_integrity"],"properties":{
   "enrollment":{"type":"boolean"},"identity":{"type":"boolean"},"applicability":{"type":"boolean"},"required_closure":{"type":"boolean"},"evidence":{"type":"boolean"},"currentness_boundary":{"const":"pinned_revision_two_observations"},
   "freshness_guarantee":{"const":"observed_boundaries_only"},"continuous_currentness":{"const":false},"atomic_filesystem_snapshot":{"const":false},"projection_integrity":{"const":"admission_and_accessed_rows"},
   "observations":{"type":"object","additionalProperties":false,"required":["working_start","working_completion"],"properties":{"working_start":{"enum":["matched","failed","incomplete","not_checked"]},"working_completion":{"enum":["matched","failed","incomplete","not_checked"]}}}
  }},
  "budget":{"type":"object","additionalProperties":false,"required":["limit","tokenizer","accounting","required_material_fit","optional_truncated","diagnostics_truncated","optional_omissions"],"properties":{"limit":{"type":"integer","minimum":1},"tokenizer":{"enum":["cl100k","o200k"]},"accounting":{"const":"serialized_mcp_result_v1"},"required_material_fit":{"type":"boolean"},"optional_truncated":{"type":"boolean"},"diagnostics_truncated":{"type":"boolean"},"optional_omissions":{"type":"array","uniqueItems":true,"items":{"enum":["not_requested","budget","unavailable","work_limit"]}}}},
  "citations":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["id","source","commit","blob","source_sha256","start","end","quote_sha256","text","freshness","execution"],"properties":{"id":{"type":"string"},"source":{"type":"string"},"commit":{"type":"string"},"blob":{"type":"string"},"source_sha256":{"type":"string"},"start":{"type":"integer","minimum":0},"end":{"type":"integer","minimum":1},"quote_sha256":{"type":"string"},"text":{"type":"string"},"freshness":{"$ref":"#/$defs/freshness"},"execution":{"enum":["not_run","report_of_execution","executed_artifact"]}}}},
  "result_state":{"enum":["complete_within_registered_scope","incomplete","blocked","unavailable"]}
 },
 "$defs":{
  "freshness":{"enum":["current","stale","missing","inaccessible","invalid","not_checked"]},
  "issues":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["code","records"],"properties":{"code":{"enum":["profile_changed","projection_unavailable","generation_mismatch","policy_mismatch","record_not_enrolled","enrollment_incomplete","scope_not_enrolled","identity_unavailable","applicability_unknown","work_limit","source_stale","source_missing","source_inaccessible","source_invalid","premise_unsatisfied","gate_unsatisfied","required_cycle","authority_conflict","required_budget_exceeded","source_changed_during_read"]},"records":{"type":"array","items":{"type":"string"}}}}},
  "records":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["id","revision","role","required","authority","verification","lifecycle","freshness","subject","disposition","citations"],"properties":{
   "id":{"type":"string"},"revision":{"type":"string"},"role":{"enum":["claim","decision","constraint","evidence"]},"required":{"type":"boolean"},
   "authority":{"type":"object","additionalProperties":false,"required":["class","attributed_to","assurance"],"properties":{"class":{"enum":["hypothesis","unverified_extraction","attributable_observation","verified_claim","explicitly_approved_decision"]},"attributed_to":{"type":"string"},"assurance":{"const":"source_attributed"}}},
   "verification":{"type":"object","additionalProperties":false,"required":["outcome","subject","execution"],"properties":{"outcome":{"enum":["unknown","supported","contradicted","inconclusive"]},"subject":{"type":"string"},"execution":{"enum":["not_run","report_of_execution","executed_artifact"]}}},
   "lifecycle":{"enum":["active","superseded","retracted","falsified"]},"freshness":{"$ref":"#/$defs/freshness"},"subject":{"type":"string"},"disposition":{"type":"string"},"citations":{"type":"array","minItems":1,"items":{"type":"string"}}
  }}}
 }
}
```

Additional semantic invariants: returned IDs/revisions/hash/byte ranges must equal normalized admitted records; citations must resolve and hash their exact text; no dangling citation/relationship; repeated IDs reject; complete state requires every coverage bit true plus `required_material_fit=true`, no required issues/conflicts and no required omission. `unavailable` has no citations/records/history/relationships. A known source failure sets evidence=false; incomplete enumeration sets the corresponding coverage bit false. `blocked` budget overflow may retain true acquisition coverage while required material fit is false. Unknown result-state strings or future schema versions reject at the client boundary.

Here “coverage bit” means the five boolean checks enrollment/identity/applicability/required_closure/evidence, not the deliberately false continuous/atomic assurance fields. Complete additionally requires both working observation states `matched`. A phase is matched only when all required current-file/permission observations in that phase match; an observed failure takes `failed` precedence over unfinished observations, otherwise exhaustion is `incomplete`, and an unattempted phase is `not_checked`. Known projection corruption sets unavailable; `projection_integrity` never means a full-storage audit was performed. Every result exposes boundary-only freshness and admission/accessed-row integrity limitations, including a complete-negative result. These added v1 design fields require no manifest or SQL schema change.

### 8.4 Seven representative public examples

Metavariables here identify **proposed example values**, not executed MCP data: `G` is the generation computed from the reviewed full pilot manifest, `ROOT` the owner profile root, and `BASE` the common request fields `{workspaceRoot:ROOT,corpus:"af-perf-175",generation:G,tokenizer:"o200k",includeHistory:false,limits:{records:256,edges:1024,evidence_bytes:1048576,token_budget:8192}}`. These finite fixture values must be admitted by the test profile; they are not global/default caps. Each row is one call to `pcg_query_v1` with BASE plus the shown fields, and specifies the expected payload fields in addition to schema-required scope/generation/policy/subjects/empty arrays.

`subjects` in v1 is exactly one string: canonical JSON of the normalized request subject, including its explicit effective time and any resolved declaration reference. Relationship `from` is a logical record ID; `to` is a record ID except for `CONSTRAINS`, where kind unambiguously identifies a selector ID. Citations use evidence IDs scoped by the returned generation. Protocol errors are JSON-RPC `-32602` with fixed non-disclosing reason for invalid request/unauthorized scope/unsupported tokenizer/too-small control budget; tokenizer creation or denied secret material uses `-32603` with fixed `pcg_control_unavailable` and no source detail. These errors are not evidence results or complete-negative answers.

| Example | Request addition | Expected result / public contract |
| --- | --- | --- |
| 1 Exact source owner | `mode:"constraints", subject:{workspace:"cleanctx",file:"src/mcp/tool_handlers/core/delta.rs",operation:"delta_code_context",task:"modify",at:"2026-11-09T23:00:00Z"}` | Required A/B records, their full source citations/conditions; assurance source-attributed, no edit approval; complete only if full reviewed pilot coverage and all witnesses fit. |
| 2 Dependency closure | `mode:"closure",records:["delta-capture"],subject:<example 1>` | A plus a separately enrolled same-guard/currentness constraint if linked by DEPENDS_ON; typed edges/citations and gate status, no history-as-premise. |
| 3 Reverse impact | `mode:"impact",records:["delta-currentness"],subject:<example 1>` | Incoming DEPENDS_ON/BLOCKED_BY consumers A/B and transitive paths, plus all applicable required constraints. Reverse index completeness is for G only; missing corpus enrollment => incomplete. |
| 4 Provenance | `mode:"provenance",records:["replay-reuse"],subject:{workspace:"cleanctx",file:"src/mcp/tool_handlers/persistence/replay.rs",operation:"replay_history",task:"explain",at:"2026-11-09T23:00:00Z"}` | Rejection, registered-deletion counterexample and exact active regression references; report-of-execution stays such. Complete only if the whole required bundle fits; otherwise example 7's rule applies. No claim of equivalence. |
| 5 Complete negative | constraints on fixture `unconstrained.ts` with an explicit matching reviewed coverage selector, all dimensions supplied, no bound records | `result_state:"complete_within_registered_scope"`, all coverage bits true, `required_records:[]`, `citations:[]`, `reasons:[]`, required material fits. Without the explicit coverage selector this example is not complete. |
| 6 Incomplete enrollment | Same request as 5, but approved fixture manifest has `enrollment.complete:false` | `incomplete`; enrollment=false, reason `enrollment_incomplete`; empty results are not absence proof. An unapproved manifest edit is rejected earlier, never used as this fixture. |
| 7 Required overflow | Example 1, fixture token budget chosen above minimal-control size but below independently counted required envelope | `blocked`; `required_material_fit:false`, reason `required_budget_exceeded`, no truncated actionable authority bundle. All emitted metadata still fits the limit. Below control size => protocol error, not a complete empty result. |

## 9. AF-PERF evidence pilot

All pilot records/edges below are **PROPOSED / UNIMPLEMENTED**. Source hashes and byte spans are **OBSERVED** pinned cloud bytes. `C` denotes candidate commit `8ce0a6865ff8a2b55866895c6a239f6bcc3245a3`; each row's full path, C and SHA-256 constitute its exact source attribution. No generated execution receipt is invented. Byte ranges are half-open over original UTF-8 including original newline bytes.

### 9.1 Exact source catalogue

| Key | Source path at C / SHA-256 | Locator / excerpt SHA-256 |
| --- | --- | --- |
| A-final | `docs/agent/AF_PERF_002_DELTA_PREVIOUS_REPAIR.md`; `0177375cf4b24b6ddf4b741da1ceff1d94ed840c00a7a0ae60cf4349b30dea51` | `[22798,25013)`, `## Final composed acceptance: CLOSED / FIX / PROTECTED`; `3c95b9640d858ac6963dc5dbb910bf0a41c0c61b272dad29a506d75f0c575e0f`. |
| B-rationale | `docs/agent/AF_PERF_002_DELTA_PREVIOUS_DESIGN.md`; `33a658c004b618cdead72d166e02b1e1b9a86c14ab3d99f76b5642b20ca38427` | `[489,1923)`, `## Exact registered lock and authority trace`; `20138b0426b4d17904d9a52e87eb9aaa7a0121e1bc2d35d3545d481a4c3e3470`. Historical pre-repair trace; current deferral comes from A-final, not this old early-capture description. |
| C-reject | `docs/agent/AF_PERF_002_REPLAY_RENDER_DESIGN.md`; `42753c4e68efaa8c2555e885d16f8b225e60b7621744bb099225f4eddfb243bf` | `[13420,21998)`, `## Executed design: REJECT proposed Candidate presentation reuse`; `9c55be6ddb1a7bbf45a17e93f76f7a537cbd4e0491695044c78138264a7eca9f`. |
| C-accepted | `docs/agent/AF_PERF_002_GITDIFF_DISCOVERY.md`; `8e44a70059f9d1a4b9d75d3600316235e25a1245259547d2e4c13b2b1ea68836` | `[250,2163)`, `## Rejected replay apparatus retirement`; `d6d9233a049d6d9e3218c8a99e6b3623523629a89cb53423918294521895eb3b`. Owner acceptance and active regression are reported separately from the earlier recommendation. |
| D-disposition | `docs/agent/AF_PERF_002_GITDIFF_DISCOVERY.md`; `8e44a70059f9d1a4b9d75d3600316235e25a1245259547d2e4c13b2b1ea68836` | `[14045,15898)`, `### Ranked disposition and owner recommendation`; `bf8474cf7e697d217a9a96d0fa2e7e2f6a3555f1bea2d3a6f5613337647c6b4c`. |
| D-work | Same Git report/hash | `[11230,14045)`, `### Production-path work evidence`; `63cc6102e6bedb8c063c634ce02e71b2ae9bba3fb93886e3b0940c2847694055`. |
| E-failed | Same Git report/hash | `[8538,9839)`, `## First attempt: first-emission versus warm cache hints`; `b72f6ac369d7547aef5f09b6258bd2568702c54e48c3246ade659fc468e103eb`. |
| E-final | Same Git report/hash | `[9839,10604)`, `## Executed cleanup and discovery return`; `10f4a173989666fcc6bcf950d0896702d82d37a09895c72ee3c557e258d5589e`. |
| W-delta | `src/mcp/tool_handlers/core/delta.rs`; `b50b5fca0fd8550df1be1910e35e6c536b759b92872819cfcc00dfee1aa46722`; blob `9b1849de120228c7786ff0124bfcb89c40f32d81` | Current corpus witness, no whole source body persisted. Protects the actual delayed capture/second-read owner. |
| W-replay | `src/tests/mcp/replay_footer_observation_af_perf_002.rs`; `6e8cdc00a7d5bef7e23fcf672f45de4520c4e618af41b64074e4b42b0c8e6408`; blob `d847cb3d1eb1b23c0cde7f54ba227672ed3c0c8d` | Current witness; full `[0,5448)` is available for explicitly enrolled regression evidence, with identical whole-source/excerpt hash. No execution inferred from its existence. |

All source rows used for current requirements have `check=current`; B-rationale can be a historical source/evidence row, explicitly labelled pre-repair. Required A/B interpretation uses A-final and W-delta. The full pilot must enroll owner-reviewed coverage/bindings for each operation/task/file combination; examples must not acquire universal coverage merely by listing these cases. Exact record hashes/generation are computed only after that manifest is reviewed; this document does not manufacture approval fingerprints.

### 9.2 Records, relationships and expected context

| Case / exact proposed IDs | Authority, applicability and relationships | Verification / lifecycle / query / expected behavior / negative |
| --- | --- | --- |
| A `delta-capture`; `delta-currentness` | Source-approved decision attribution from A-final; `delta-currentness` is a constraint citing the same complete final span. Bind both to exact delta owner + `delta_code_context` + tasks `modify/explain` through separate exact selectors. `delta-capture DEPENDS_ON delta-currentness`; both use source A-final; W-delta required current witness. | Supported/delayed_capture_contract/report_of_execution; active; freshness evaluated. Query example 1/2. Context must preserve zero discarded snapshots on cached return, same original guard, both reads, necessary cached construction and changed-source capture. Contradictory early-capture task must surface PROTECTED and reauthorization need. Remove a condition/span or change W-delta => blocked, never a weaker complete instruction. |
| B `delta-second-read` | Source-attributed DEFER from A-final, rationale B-rationale explicitly historical. Same file/operation tasks as A. `delta-second-read DEPENDS_ON delta-currentness`; `USES_EVIDENCE` a historical rationale evidence record, no historical early-capture prerequisite. | Supported/retain_currentness_boundary/report_of_execution for the documented decision, not proof of a new optimization. Active, query-time freshness. Query constraints, task modify, intent expressed by exact subject `delta_second_source_read`. Context: second read/hash occurs under existing guard; completed capture repair did not authorize removal. Set same bytes/mtime but alter W-delta or remove required currentness evidence => blocked; a DEFER cannot become FIX. |
| C `replay-reuse`; `replay-equivalence`; `replay-counterexample`; `replay-later-footer` | C-accepted supplies owner-acceptance attribution; C-reject supplies divergence, later-footer constraint; W-replay supplies active test. `replay-counterexample FALSIFIES replay-equivalence`; `replay-later-footer CONSTRAINS` replay owner/operation; `replay-reuse DEPENDS_ON replay-later-footer`. `replay-reuse USES_EVIDENCE replay-counterexample` is descriptive; mandatory C-reject/W-replay witnesses are explicit `evidence_refs.required=true` on replay-reuse. The counterexample record is independently mandatory only if explicitly requested or selected as incoming FALSIFIES when tracing equivalence. | Rejection supported/reuse_rejection/report_of_execution, active. Equivalence claim contradicted/equivalence/falsified; counterexample supported/counterexample_detection; test source itself not_run. Query example 4 or replay constraints/modify. Context includes registered deletion's later dictionary/footer difference and exact test function `af002_replay_footer_observes_dictionary_after_economics`. Treat detector PASS as equivalence or retired apparatus as active => oracle failure. No new replay-after-delete success policy. |
| D `git-body-copy`; `git-duplicate-set` | Two separate source-attributed DEFER decisions, both exact D-disposition primary and required D-work evidence refs. Bind to `diff_commits` plus exact tasks/file subjects for body ownership and working-tree enumeration. `USES_EVIDENCE git-measured-work` is descriptive, never authorization to optimize. | Supported/measured_deferral/report_of_execution, active. Query constraints on diff_commits/modify. Shared citation emitted once; separate subjects remain separate records. Context preserves 26..965-byte observed bodies, eight-entry/72-byte largest measured set, no construction-inclusive benefit, duplicate/order/security/currentness obligations. Reclassify logical bytes as allocation/latency savings or collapse two subjects => negative fails. |
| E `git-first-packet`; `git-final-packet`; `delta-final-reported` | Attributed observations from E-failed, E-final and A-final. `USES_EVIDENCE` links later conclusions to these exact evidence records; no SUPERSEDES edge claims that the failed run became successful. Their subjects include distinct exact packet IDs. | First: supported/fixture_failure_report/report_of_execution, active historical finding; final: supported/completed_discovery_report/report_of_execution. Query provenance/history for specific packet IDs; history permission required. Context distinguishes `gitdiff-discovery-20261109-184618` fixture failure from `gitdiff-discovery-20261109-190157` reported 48-pass completion; A-final reports 110 passes/55 groups separately. Missing actual receipt remains report-only. Swapping packet outcomes or reporting these as newly run gates => negative fails. |

The catalogue supplies exact locators for the mandatory pilot evidence, including C's owner-acceptance attribution. Materializing a reviewed manifest means applying the closed format and these exact witnesses, computing their defined revisions/generation and obtaining the owner's digest pin; it does not authorize discovering a missing decision body. Additional evidence requires the same exact-source review. No unapproved graph records are created by this design.

Pilot normalization constants: workspace `cleanctx`, policy `local-source-attribution-v1`, both interval ends null, tasks `modify` and `explain` represented by separate selectors. A/B bind `src/mcp/tool_handlers/core/delta.rs` + `delta_code_context`; C binds `src/mcp/tool_handlers/persistence/replay.rs` + `replay_history`. D's body-copy owner is `src/gitdiff/engine.rs::diff_two_contents` (candidate trim site at line 329), duplicate-set owner `src/gitdiff/workspace.rs::collect_changed_files` (set at line 153); use separate exact file + `diff_commits` selectors/queries. These locations were source-inspected in the candidate; line numbers are orientation, not a substitute for artifact hashes or record IDs.

All A/B/C/D controlling decisions are `role=decision, required=true, authority.class=explicitly_approved_decision` with source-attributed owner approval, active lifecycle and `report_of_execution` for the **reported evidence**, never runtime approval. A primary=A-final/disposition=PROTECTED; B primary=A-final/disposition=DEFER, required A-final and W-delta, optional B-rationale; C primary=C-accepted/disposition=REJECT, required C-reject and enrolled full W-replay test-definition citation; D primary=D-disposition/disposition=DEFER, required D-work. Decision subjects are respectively `delta_previous_authority_capture`, `delta_second_source_read`, `replay_earlier_candidate_reuse`, `git_trim_to_owned_body`, `git_worktree_duplicate_set`; keep the two D subjects separate. Attributed actor is `owner decision reported by Agent MaxHeadRoom`; neither graph nor importer authenticates that actor.

Auxiliary constraints `delta-currentness` and `replay-later-footer` are required source-attributed verified-claim constraints, supported respectively for `source_currentness` and `later_dictionary_observation`, primary A-final and C-reject. `replay-equivalence` is a noncontrolling hypothesis claim, contradicted/equivalence, lifecycle falsified, primary C-reject; `replay-counterexample` is descriptive attributable-observation evidence (`required:false`), supported/counterexample_detection, active, primary C-reject. Its quote is nevertheless mandatory for replay-reuse via the explicit C-reject evidence ref. `git-measured-work` is attributable-observation evidence, supported/logical_work_counts, active, primary D-work. E evidence records are attributable observations with the exact packet subjects/outcomes in the case table; their witnesses are mandatory only by §4.3, not merely because a required record has a descriptive link to them. A W-replay test-definition evidence record has unknown/test_definition/not_run; its source text is not itself a VALIDATES edge proving execution. These constants complete the pilot's normalized mapping without authoring a second decision body.

## 10. Acceptance, security and implementation gates

### 10.1 Executable acceptance specification

No test file/fixture is written or executed in this mission. These are exact **proposed** tracked assets. Register each `src/tests/pcg/<name>.rs` from its owning `src/pcg/<module>.rs` using `#[path="../tests/pcg/<name>.rs"]`; MCP tests from the handler use the correct source-relative path into `src/tests/mcp/`. Proposed fixture root is `verification/pcg-175/fixtures/` (versioned definition); tests materialize isolated Git workspaces/DBs under `target/**` or temporary directories. Runtime AF-PERF assertions use exact approved source spans; tiny fixtures establish correctness only, not enterprise performance. No source tests in ignored paths satisfy the gate.

| Group / proposed tracked file and test functions | Exact positive evidence / required negative fixture |
| --- | --- |
| G1 `src/tests/pcg/enrollment.rs`: `stable_ids_reimport_exact`, `reject_conflicting_identity`, `rename_preserves_logical_id`, `reject_parser_and_locator_mismatch` | `valid-v1.json` exact mapping; `duplicate-key.json`, `duplicate-record.json`, `conflicting-revision.json`, malformed offsets/ambiguous headings, parser-v2 and rename manifest. Assert same logical IDs, distinct correct revisions; no generation on invalid input. |
| G2 `src/tests/pcg/persistence.rs`: `atomic_import_rollback`, `process_crash_restart`, `idempotent_rebuild`, `unknown_schema_and_corruption_refuse`, `concurrent_reader_import_head_cas` | Exact row/fingerprint/head oracle after real fixture subprocess kill at precommit/postcommit barriers; source replacement fails without partial rows; two independent writer connections, one stale CAS loses; old reader snapshot and fresh process see whole generations. No kill of production processes. |
| G3 `src/tests/pcg/provenance.rs`: `exact_citation_bytes`, `source_drift_and_missing_evidence`, `source_approval_is_attribution`, `packet_subjects_never_conflate` | Byte hashes and ranges exact; same-size/mtime-preserved edits, absent blobs, forged approved field, detection-versus-equivalence and failed-versus-final receipt fixtures. No authenticated-approval flag emerges. |
| G4 `src/tests/pcg/query.rs`: `applicability_three_valued`, `closure_and_incoming_counterevidence`, `reverse_impact_closed_scope`, `covered_empty_vs_unenrolled`, `cycles_and_unknown_identity_refuse` | All selector dimensions/interval endpoints, typed canonical declaration from real registered source compression when needed, missing state after restart, uncovered files/tasks, cycle and contradictory decision fixtures. Assert exact state/coverage/paths; no lexical absence proof. |
| G5 `src/tests/pcg/projection.rs`: `deterministic_complete_envelope`, `account_exact_serialized_result`, `required_overflow_blocks`, `optional_only_truncation` | Permute insertion order; count full escaped MCP result with cl100k/o200k; choose boundaries using independently counted full/control envelopes; mandatory conditions/citations never disappear in a complete result; approximate tokenizer rejects. |
| G6 `src/tests/mcp/pcg_query_v1.rs`: `catalog_registry_schema_parity`, `registered_constraints_query`, `fresh_process_readonly_reopen`, `legacy_responses_unchanged` | Actual stdio server -> dispatcher -> registry -> handler -> evaluator -> SQLite -> complete response. Import via explicit CLI, terminate, launch fresh server, query known G. Compare full legacy response/canonical/cache/durable identities before/after PCG; disabled catalog unchanged. Internal-only evaluator tests cannot satisfy this group. |
| G7 `src/tests/pcg/security.rs`: `trusted_profile_scope_refusal`, `history_revocation_no_disclosure`, `injection_is_data`, `secret_input_not_persisted`, `reparse_and_git_escape_refusal` | Foreign root/corpus, manifest/profile replacement, hidden history, forged instructions/approval, secret-bearing excerpt, path traversal/reparse/symlink/promisor/remote fetch controls. Assert no hidden locators/hashes/titles in errors/logs/DB/WAL. |
| G8 `src/tests/pcg/af_perf.rs`: `protected_delta_and_deferred_read`, `rejected_replay_counterexample`, `two_git_deferrals`, `historical_packets_keep_outcomes` | Full A-E reviewed manifest and immutable source catalogue. Assert exact IDs/dispositions/required spans/evidence execution categories/source freshness. Remove coverage, change source, swap packet subject, treat detector as equivalence or historical test as active; each must fail the expected oracle visibly. |

Fixture definitions also include `gates.json` (exact prerequisite subjects), `coverage.json` (explicit covered-empty scope), `history.json` (supersession and retention), and `expected-results.json` (canonical required IDs, byte citations, state/coverage for each case). These names are planning contracts, not existing files. Complete A-E materialization and its original negative controls must be reviewed before production import work, including all computed revisions/generation; no invented placeholders enter an executed gate.

Correction acceptance mapping (all **PROPOSED / UNRUN**, no test files added):

| Correction / exact proposed owner | Positive oracle / negative specifications |
| --- | --- |
| A: G3 `provenance.rs`, G4 `query.rs`, G5 `projection.rs`, G8 `af_perf.rs` | Execute the two positive examples and all eight named controls in §4.3; require the shared-witness validation/encoding/citation counts and optional-omission response. Replay's explicit C-reject ref is mandatory even without USES_EVIDENCE. `optional_work_limit_preserves_required_completeness` separately exhausts only optional expansion and must retain the full complete required bundle. |
| B: G3 `provenance.rs`, G6 `pcg_query_v1.rs` | All eight named controls in §7.2. Barriers schedule edits at actual observation points. ABA/restored permissions not observed by either read must not be asserted detected; the oracle checks boundary-only wording and false continuous/atomic flags. Two sources changing independently never prove one snapshot. Final observed failure must discard the counted complete draft. |
| C: G2 `persistence.rs`, G4 `query.rs`, G6 `pcg_query_v1.rs` | All five named controls in §6.3. A valid generation yields the same required answers as explicit full verification. Corrupt required accessed rows fail; off-path valid-SQL deletion/corruption can evade an ordinary read, but explicit audit/affected access detects what it actually examines. A matching generation ID is not a full-integrity attestation. Restart counters must not hide full-row rehash. No benchmark result is implied. |

The off-path and ABA controls expose limits, not tolerated missing mandatory witnesses in an intact admitted projection. No test may turn an unobserved event into a claimed detected failure, or turn a source-inspected/test-definition PASS into a new execution receipt.

### 10.2 Trusted-local security and data handling

Profile and manifest are local owner-controlled files with OS permissions restricting untrusted modification. Tool arguments, source text and SQLite excerpts are untrusted data. Workspace paths are locators, not tenant principals. Reject unknown corpus/root before discovery; all joins and errors respect exact p/c/g. Local read access to source is required at each query phase; inaccessible evidence does not inherit old access through a cached excerpt. History requires explicit policy permission and currently readable authoritative revision; permission loss returns generic blocked/unavailable without sensitive locators/hashes. No enterprise multi-tenant security claim exists.

Secret minimization: enroll only necessary reviewed spans and witness hashes, no full source bodies, transcripts, arbitrary prose metadata or source credentials. The first-slice policy is exactly `reviewed-public-repository-evidence-v1`: only owner-reviewed nonsecret repository artifacts, plus deterministic deny checks on **all to-be-persisted or emitted text/metadata**. The deny set is Rust-regex syntax: `-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----`, `(?:gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|AKIA[0-9A-Z]{16}|sk-[A-Za-z0-9_-]{20,})`, `(?i)authorization\s*:\s*bearer\s+\S+`, and `(?i)(?:api[_-]?key|client[_-]?secret|password|access[_-]?token)\s*[:=]\s*["']?[^\s"']{8,}`. Scan normalized metadata and exact quote bytes after strict UTF-8 decoding; no matches are persisted/logged. These deliberately conservative signatures can reject benign examples; owner review cannot bypass a required rejection without a separately reviewed policy revision. A match rejects the whole import/query material with a generic control error, no partial redaction, no evidence payload and no falsely complete clause. This finite deny set does not prove arbitrary text secret-free; the reviewed nonsecret-source deployment restriction is essential. Source hashes/paths may also be sensitive; do not log them on refused scope/permission or secret failure. Optional proxy scrubbing, currently disabled by default, is not this boundary.

A document cannot choose commands, profile, roles, permissions, SQL, tokenizer fallback or approval. Excerpts remain JSON strings labelled source-attributed; fake system instructions/fenced PASS examples cannot satisfy gates. A cooperating client must treat retrieved text as evidence, never higher-priority instructions. The server can enforce typed transitions and data access, not prove a model obeys its instructions. Runtime promotion is absent, not merely protected by an `approved=false` default.

Retention: source owner controls permitted historical retention. The projection may retain old committed generations only while policy allows; local owner can run explicit rebuild with policy excluding old history. Before claiming physical erasure, stop readers, replace/rebuild the derived DB and handle WAL/backups/OS recovery copies under owner retention policy; logical SQL DELETE alone is not erasure evidence. No automatic purge or competing durable decision ledger. Rebuild requires approved manifest plus retained Git source blobs, never summaries or reported receipt paths. Corrupt DB cannot authenticate its own repair.

### 10.3 Performance and bounded work

Measure cold, warm and fresh-process restart retrieval; import/reimport/rebuild; DB and WAL bytes per source/record revision; end-to-end memory and serialized output; required closure and distinct source validation bytes; local tokenization costs and complete-baseline token savings. Record fixture/corpus hashes, exact configuration/tokenizer, hardware, source populations and full distributions, not only favorable medians. Required-authority recall, wrong-authority promotion, wrong-scope disclosure and false-complete counts are separate correctness metrics; each prohibited outcome must be zero in established acceptance populations. No measured PCG efficiency claim is made here.

Operation scope is one import generation or one explicit query. **COST-01 (normative): no per-query O(total enrolled corpus) integrity validation unless a concrete correctness invariant requires it.** Selected Option B removes unconditional generation-wide row/byte rehash; it does not waive complete applicable-universe evaluation, required closure/evidence, or the retained all-current-witness corpus freshness invariant in §7.2. Those costs remain separate and observable.

| Work symbol | Exact unit |
| --- | --- |
| N | Total enrolled records in the pinned generation. |
| A | Indexed applicability/coverage/conflict candidates actually examined, including unknown dimensions; may approach the whole registered selector universe. N alone does not bound arbitrarily many selectors per record. |
| R | Distinct required closure records, including mandatory assertion explanations. |
| E | Distinct traversed/examined relationships, bindings and evidence refs. Required-ref index avoids scanning unrelated optional refs for each record. |
| S | Distinct source witnesses checked: all enrolled current sources, plus required historical sources and any optional sources actually offered. S need not be proportional to R. |
| B | Distinct mandatory excerpt bytes validated/projected once, not full source-file bytes. |
| T | Tokens in the final serialized MCP result, including wrappers and metadata. |
| Additional necessary units | `E_all` = all generation assertions; `D` = total generation row/hash bytes for a full verification; `L_access` = accessed metadata bytes rehashed; `W` = actual source bytes read/hashed in prescribed phases; `P` = total serialized/tokenizer input work across draft, optional packing and final control/result candidates. W can greatly exceed B, and P can exceed T, especially on overflow. |

Let `logK=log(N+E_all+1)` denote indexed lookup cost, with candidate/output sorting reported separately. Anticipated work, not measured performance:

| Alternative / boundary | Expected work |
| --- | --- |
| A, full verification each retrieval | O(D + N + E_all + (A+R+E)logK + S + W + B + P), plus deterministic ordering. Every query pays corpus row/hash cost even if A/R/E/S/B/T stay fixed. |
| B, selected targeted retrieval | O((A+R+E)logK + L_access + S + W + B + P), plus deterministic ordering. No unconditional N/D/E_all verification term; generation/schema/profile/header lookup is bounded metadata work. |
| Admission / explicit full audit | Full manifest/definition membership and semantic work O(N+E_all+registered selectors), full normalized byte/hash work D, and actual source/immutable-evidence bytes for that operation. Audit is owner-invoked, not amortized into an unreported startup/query scan. |

Complete applicability can legitimately cost O(A) with A near corpus size; it is not full-corpus byte hashing. Source checks can likewise grow with N if all-current S/W grow: the existing generation-wide freshness promise is the concrete reason those witnesses remain mandatory, and no narrower relevance proof exists in this format. Record integrity costs cover only accessed rows, independently of both operations. A required-only successful result can have P proportional to T; optional repacking/overflow must report actual cumulative P rather than claiming final output size bounds all tokenization. Memory includes operation-local candidate/closure sets, deduplicated evidence and serialization buffers; no numerical memory/latency advantage is asserted. Stream full audit hashing where possible; do not turn a full scan into a new semantic cache.

Proposed N/2N/4N measurements: (1) hold A/R/E/S/B and required output fixed while adding off-path optional historical records; assert zero off-path row/quote hashing under B and observe index/restart costs; (2) grow A with required universe deliberately broad, retaining exhaustive applicability; (3) grow R/E/B and shared-witness fanout, asserting one evidence validation/citation per revision; (4) grow current S/W independently, demonstrating both required observations without hiding source cost; (5) compare explicit A audit/read cost with B on identical corpus identities. Measure cold/warm/restart, import/rebuild/audit, DB/WAL, full serialization/tokenization P/T and peak memory with original negative controls. Work counters and enclosing-operation costs are future evidence, not measured here.

Request ceilings are explicit PCG-only controls. No per-candidate authority-set reconstruction, new persistent authority/cache/retry mechanism or claimed benefit from counts alone is authorized. AF-PERF-001 fitness and AF-PERF-002 evidence-backed FIX/MONITOR/ACCEPT remain mandatory for later production changes.

### 10.4 Separately authorized implementation stages

All file names below that do not exist are **proposed permitted paths**, not discovered current paths. New/modified code must remain <=615 lines, preferably <=600, with semantic decomposition where necessary. Candidate seam counts: `main.rs` 432, `lib.rs` 108, `mcp/state.rs` 593, `mcp/tools.rs` 542, `tool_schemas.rs` 320, handler registry 216, annotations 98. Do not overflow `state.rs`; place initialization/read-handle helpers in a focused module. No stage may edit replay/delta/source-cache/source-persistence/publication code to make PCG easier.

| Stage / permitted production paths | Original negatives / positive and restart evidence | Compatibility, stop and owner gate |
| --- | --- | --- |
| M2a-1 enrollment/persistence | New `src/pcg/{mod,profile,enrollment,identity,store,import,security,cli}.rs`; minimal exports `src/lib.rs` and explicit CLI dispatch `src/main.rs`. G1/G2/G3/G7 original rejection/atomicity oracles; exact import/reopen/rebuild generation, no real user DB. | Existing no-argument server/CLI/persistence unchanged. Stop on incompatible source baseline, unavailable trust/source, schema/lifecycle expansion or failed negative. Owner accepts evidence before next stage. |
| M2a-2 authority retrieval | New `src/pcg/{selectors,query,closure,provenance}.rs`; source-owned canonical declaration adapter isolated in `src/mcp/pcg_subject.rs` if needed, no source IR edits. G4/G8, incoming falsifiers/impact, stale/unknown/empty controls; fresh-process file queries complete and entity-proof absence incomplete. | No source hydration, compilation or publication fallback. Stop if required scope/identity/completeness cannot be represented. Owner accepts matrix/fitness before projection. |
| M3 token projection | New `src/pcg/{projection,wire}.rs`, read-only calls to existing tokenizer. G5; independently counted whole public result at required/optional boundaries; restart results deterministic for pinned source/profile. | No changes to existing code economics/fidelity/raw fallback. Stop on missing conditions, approximate counts or falsely complete overflow. Owner accepts exact envelope contract. |
| M4a read-only MCP | New `src/mcp/tool_handlers/pcg.rs`, `src/mcp/pcg_state.rs`; narrow wiring `src/mcp/{mod.rs,state.rs,tools.rs,tool_schemas.rs,tool_annotations.rs}`, `tool_handlers/{mod.rs,registry.rs}` as needed. Existing dispatcher/router stays unchanged. G6 actual registry/stdio path and full legacy identity controls; explicit CLI import then fresh server retrieval. | Add PCG descriptor/handler only when profile enabled; maintain state size and lock ordering. Stop on any legacy response/authority mutation, unresolved MCP parity or field-only failure. Owner accepts local gates, then batches any justified Claude-only field gate. |

At each stage, required tracked tests remain under `src/tests/**`; reusable operator driver/fixtures under `verification/pcg-175/**`; generated outputs under `target/**`. New tests are registered through normal `#[path]` conventions, never hidden ignored definitions. New correctness failures need original valid RED before correction when applicable; a compile error or live harness is not RED. Stage authorization does not permit advancing automatically, modifying protected optimizations or broadening profile/security/API scope. Runtime ledger/approval/extraction/injection/embeddings/hosting remain deferred beyond these stages.

### 10.5 Proposed freeze decisions, limitations and disposition

Ready for owner review: sidecar enrollment and exact-digest profile trust; single-root/source-attributed local deployment; stable IDs and byte/revision hash rules; explicit role/authority/verification/lifecycle/freshness distinctions; typed closure/gate/conflict semantics; isolated SQLite DDL and atomic generation lifecycle; three-valued applicability and enrolled-scope completeness; dedicated `pcg_query_v1` schema/channel/token contract; A-E corpus/oracles; G1-G8 and four gated implementation stages.

Owner choices are acceptance/rejection of these concrete proposed contracts, location/OS permissions of the local profile/database, profile resource ceilings/history retention and the materialized pilot manifest/digest. Approval of corrections A-C remains required. No authority principal, runtime approval or enterprise ACL policy is silently frozen. Unavailable typed canonical endpoint proof yields incomplete; restart supports complete file/operation/workspace results only under the stated intact-generation and observed-freshness assumptions, not unacquired canonical entities. ABA/TOCTOU and off-path projection damage can escape targeted observation; the API states both limits. All corpus-current witnesses still have an explicit cost, not a claimed subgraph-only performance bound.

Final disposition: **DESIGN CORRECTION READY**. The next action is owner approval of the corrected design freeze. Do not advance automatically into M2a-1 or any implementation mission.

Only this design document was authored during this mission; predecessor discovery and source files remain untouched. No tests, builds, formatter, validator, benchmark, server or migrations were executed. Bounded static source/hash/byte inspections are not executable acceptance. The user-owned documentation gates remain unrun:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
```

Future implementation must use `docs/agent/verification.md` as the authoritative Final Verification Gate, including all-feature tracked tests/zero-warning Clippy/AF-PERF fitness. Actual results and live/operator evidence must remain separately reported. No test PASS, performance result, MCP integration or production authority is claimed by this document.

### 10.6 DF-CORRECTION change report and consistency review

| Correction | Sections changed | Contract decision | Remaining limitation |
| --- | --- | --- | --- |
| A — Evidence closure | §§3.2-3.3, 4.2-4.3, 6.2, 7.2, 8.1/8.3, 9.2, 10.1 | ME-01 least-fixed-point closure; USES_EVIDENCE always descriptive; explicit primary/required refs and required edge provenance; incoming support/refutation included; one shared citation. | Source approval stays attributed. Required historical access/budget failure blocks; optional omission is observable and cannot waive mandatory material. |
| B — Currentness | §§5, 6.2, 7.2-7.3, 8.3, 10.1/10.5 | FRESHNESS-01/02: immutable identity separate from two per-file boundary observations; explicit final-draft invalidation and non-atomic response fields. | Unobserved ABA/permission restoration, final serialization gap and independent multi-file changes are not continuously/atomically detected. |
| C — Integrity cost | §§3.1, 5, 6.1-6.3, 7.2-7.3, 8.3, 10.1/10.3/10.5 | INTEGRITY-01/02 selects B; complete admission, atomic publication, metadata-open checks, targeted rows and explicit audit; two small indexes, no new tables/cache. | Off-path damage including deleted bindings can evade a read until audit/access. Completeness assumes intact admitted membership. Full applicable-universe work and all-current source checks remain measured costs. |

Cross-contract review: unchanged manifest/record/edge attributes map to existing evidence_ref and provenance FKs; required-ref/current-source indexes support the corrected access paths. Mandatory closure, token failure, omission codes and the payload schema agree. Currentness fields assert observations only; projection integrity fields assert admission/accessed-row checks only. Negative coverage includes omission, unobserved ABA and undetected off-path corruption without false detection claims. There is no unresolved specification contradiction identified within the authorized correction scope; the explicit trust/detection limits remain owner-review matters, not implemented guarantees. Only this document changed; no implementation, executable tests, migration, staging, commit, push or publication occurred.
