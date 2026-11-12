# PCG-175-DESIGN-FREEZE: first-slice architecture contract

B3 source disposition: C02: the original authorization/disposition paragraphs below remain historical review evidence. Later source-attributed authorization and acceptance are scoped to M2a-1, M2a-2, M3 and M4a; they do not approve every proposal. Current bounded normative owners are [enrolled v1](../architecture/contracts/pcg-enrolled-v1.md), [acquisition](../architecture/contracts/pcg-acquisition-v1.md), [projection](../architecture/contracts/pcg-projection-v1.md) and [MCP](../architecture/contracts/pcg-mcp-v1.md). Original section headings preserve navigation. §9 pilot, §10 oracles/phase gates and the original final-approval conditions retain their historical/proposed subjects; no blanket supersession or independent acceptance is asserted.

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

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

## 3. Source enrollment contract

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

### 3.1 Choice and trusted local profile

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). Original section bytes and chronology remain at commit `187d0096`.

### 3.2 Closed manifest format

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). Original section bytes and chronology remain at commit `187d0096`.

### 3.3 Complete proposed enrollment example and normalization

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). Original section bytes and chronology remain at commit `187d0096`.

## 4. Semantic and relationship model

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

### 4.1 Roles and independent dimensions

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). Original section bytes and chronology remain at commit `187d0096`.

### 4.2 Relation contracts

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). Original section bytes and chronology remain at commit `187d0096`.

### 4.3 Correction A: single mandatory-evidence closure

Current normative owner: [AUTH.PCG_ACQUISITION_V1](../architecture/contracts/pcg-acquisition-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

## 5. Identity, revisions and provenance

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

## 6. SQLite persistence contract

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

### 6.1 Isolation and proposed DDL

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). Original section bytes and chronology remain at commit `187d0096`.

### 6.2 Transaction and lifecycle algorithms

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). Original section bytes and chronology remain at commit `187d0096`.

### 6.3 Correction C: admission-validated generation, targeted reads

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). Original section bytes and chronology remain at commit `187d0096`.

## 7. Applicability, closure and completeness

Current normative owner: [AUTH.PCG_ACQUISITION_V1](../architecture/contracts/pcg-acquisition-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

### 7.1 Scope and exact selectors

Current normative owner: [AUTH.PCG_ACQUISITION_V1](../architecture/contracts/pcg-acquisition-v1.md). Original section bytes and chronology remain at commit `187d0096`.

### 7.2 Deterministic evaluator pseudocode

Current normative owner: [AUTH.PCG_ACQUISITION_V1](../architecture/contracts/pcg-acquisition-v1.md). Original section bytes and chronology remain at commit `187d0096`.

### 7.3 Result states and precedence

Current normative owner: [AUTH.PCG_ACQUISITION_V1](../architecture/contracts/pcg-acquisition-v1.md). Original section bytes and chronology remain at commit `187d0096`.

## 8. Context projection and MCP API

### 8.1 Public choice and token contract

Current normative owner: [AUTH.PCG_PROJECTION_V1](../architecture/contracts/pcg-projection-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

### 8.2 Proposed request schema

Current normative owner: [AUTH.PCG_MCP_V1](../architecture/contracts/pcg-mcp-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

### 8.3 Proposed payload schema

Current normative owner: [AUTH.PCG_PROJECTION_V1](../architecture/contracts/pcg-projection-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

### 8.4 Seven representative public examples

Current normative owner: [AUTH.PCG_PROJECTION_V1](../architecture/contracts/pcg-projection-v1.md). Original section bytes and chronology remain at commit `187d0096`.

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

Current normative owner: [AUTH.PCG_ENROLLED_V1](../architecture/contracts/pcg-enrolled-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

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

```architecture-graph
node: PCG-175-DESIGN-FREEZE
kind: RESEARCH
status: INVESTIGATING
title: Historical corrected design review with later scoped v1 acceptance
track: REPOSITORY
artifact: docs/agent/PCG_175_DESIGN_FREEZE.md
note: Source record and its attributed disposition only; no new execution, authenticated approval, independent certification, reclosure or banking.
```
