# Documentation Reconciliation — 2026-10-16

Investigator: **Agent MaxHeadRoom**

## A. Audit identity

| Field | Value |
| --- | --- |
| Branch | feature/post-cbm-native-correctness-fixes |
| HEAD | d039453bcc5825f47917e1956669bd3446a06cc5 |
| Main at audit | b5a152e8343808ab84c105d77b46458593608d03 |
| Merge base | b5a152e8343808ab84c105d77b46458593608d03 |
| Date | 2026-10-16 |
| Scope | README.md plus Markdown under docs/** and verification/**; targeted production comments/source authorities |

The branch is directly ahead of main and contains the accepted #133–#142
sequence, current CBM pinned upgrade, Angular/template work, C#/.NET repairs,
apply_edit reconciliation, #142 cross-cutting acceptance, and 0.10.0 release
preparation. The release-candidate authority gate passed.

Pre-existing working-tree changes existed in Cargo.lock and
docs/changelogs/CHANGELOG.md. This reconciliation did not modify or revert them.

## B. Executive verdict

**DOCUMENTATION RECONCILED — MINOR FOLLOW-UPS REMAIN**

Before reconciliation, public/current prose materially overstated tuple
identity, fidelity preservation, evergreen verification, quantitative
performance, and CBM runtime ownership. Historical plans also lacked a clear
precedence model. The current authority boundary is now explicit and
navigable. Minor follow-up remains because several large mixed-era subsystem
guides contain historical mechanics; they are now subordinate to the spine but
were not exhaustively rewritten, preserving blame and history.

## C. Document inventory

Static inventory covered 215 Markdown files: root README (1), docs top-level
(29), docs/agent (10), docs/architecture (22), docs/plans (67),
docs/changelogs (38), and verification (48).

### Current authority and public/operator documents

| Path | Classification | Required action | Action taken |
| --- | --- | --- | --- |
| README.md | CURRENT USER GUIDE | Correct public identity, fidelity, CBM, status, and numeric claims | Updated |
| docs/README.md | CURRENT AUTHORITY INDEX | Make normative precedence obvious | Rebuilt |
| docs/ARCHITECTURAL_INVARIANTS.md | CURRENT AUTHORITY | Add missing authority invariants without duplication | Updated |
| docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md | CURRENT AUTHORITY | Missing centerpiece | Created |
| docs/ARCHITECTURE_OVERVIEW.md | CURRENT SUBSYSTEM OVERVIEW / MIXED HISTORY | Prevent it overriding semantic authority | Status banner added |
| docs/CONFIGURATION.md | CURRENT USER GUIDE | Check keys/examples; no material semantic-authority edit | Retained |
| docs/COMPILER_IR.md | CURRENT AUTHORITY | Check IR/projection role | Retained |
| docs/ENCODING_POLICY.md | CURRENT AUTHORITY | Check precedence | Retained |
| docs/SECURITY.md | CURRENT USER GUIDE | Check scope/security posture | Retained |
| docs/PROXY.md | CURRENT USER GUIDE | Check proxy/filter/cache claims | Retained; README numbers removed |
| docs/DEVELOPER_DOCUMENTATION.md | CURRENT DEVELOPER GUIDE / MIXED DETAIL | Subordinate to normative spine | Indexed as guide |
| docs/CLAUDE_INTEGRATION_RULES.md | CURRENT OPERATOR GUIDE | Verify CBM fallback/tool forms | Retained |
| docs/DIFF_COMMITS_GUIDE.md | CURRENT USER GUIDE | Check public diff semantics | Retained |
| docs/EDIT_TYPE.md | CURRENT USER GUIDE | Check editing boundary | Retained |
| docs/TROUBLESHOOTING.md | CURRENT USER GUIDE | No authority conflict found | Retained |
| docs/agent/tooling.md | CURRENT OPERATIONAL AUTHORITY | Verify tool/CBM/query contracts | Retained |
| docs/agent/verification.md | CURRENT VERIFICATION AUTHORITY | Preserve user-run gate | Retained |
| docs/agent/architecture.md | CURRENT PROCEDURAL GUIDE | Preserve bug/audit procedure | Retained |
| docs/agent/releases.md | CURRENT PROCEDURAL GUIDE | Preserve release workflow | Retained |
| docs/agent/incremental-migration.md | CURRENT PROCEDURAL GUIDE | Preserve migration workflow | Retained |

### Framework documents

| Path | Classification | Required action | Action taken |
| --- | --- | --- | --- |
| docs/ANGULAR_META_LAYER.md | CURRENT REFERENCE WITH HISTORICAL PHASES | Clarify current-vs-history boundary | Banner added |
| docs/ANGULAR_ECOSYSTEM_DEEPENING.md | CURRENT REFERENCE WITH HISTORY | Index correctly | Retained |
| docs/DOTNET_META_LAYER.md | CURRENT REFERENCE WITH HISTORICAL PHASES | Clarify authority boundary | Banner added |
| docs/SPRING_META_LAYER.md | CURRENT FRAMEWORK REFERENCE | No material contradiction found | Retained |

### Architecture collection

| Paths | Classification | Required action | Action taken |
| --- | --- | --- | --- |
| BINARY_V04_CONTRACT.md; CORE_OP_CONTRACT_MATRIX.md | CURRENT AUTHORITY | Surface in index | Indexed |
| IR_ARCHITECTURE_CERTIFICATION.md; IR_PRODUCTION_INTEGRATION_AUDIT*.md | INVESTIGATION / EVIDENCE | Preserve point-in-time proof | Retained |
| IR_ARCHITECTURE_LOCKDOWN_PLAN.md; migration-plan files | IMPLEMENTED — HISTORICAL | Do not rewrite as current | Group-classified |
| HAS_CYCLE_*; SCHEMA_VNEXT_PROPOSAL.md; WORKSPACE_QUERY_FIDELITY_AUTO_COMPILE_PROPOSAL.md | PROPOSAL / INVESTIGATION | Prevent precedence ambiguity | Group-classified |
| SEMANTIC_INTELLIGENCE_ARCHITECTURE_HARDENING_INVESTIGATION.md; SEMANTIC_EVIDENCE_SUFFICIENCY_AND_RECOVERABILITY_INVESTIGATION.md; IR_PROJECTION_RECONNAISSANCE_REPORT.md | INVESTIGATION / EVIDENCE | Preserve rationale | Group-classified |
| LLM_CONTEXT_COMPRESSION_RESEARCH.md | INVESTIGATION / HISTORICAL RESEARCH | Keep numbers historical | Group-classified |

### Plans and acceptance records

All 67 files under docs/plans/** were inventoried. Their default status is
historical plan, investigation/evidence, implemented acceptance record, or
future proposal; none overrides the documentation spine.

| Paths/group | Classification | Action taken |
| --- | --- | --- |
| ISSUE_133_* through ISSUE_142_* | IMPLEMENTED — HISTORICAL / ACCEPTANCE EVIDENCE | Preserved; #142 final record bannered |
| ISSUES_133_141_POST_CBM_RECONCILIATION.md and POST_CBM_133_141_NATIVE_DEFECT_RECONCILIATION.md | IMPLEMENTED — HISTORICAL | Preserved |
| CURRENT_CBM_PINNED_UPGRADE_IMPLEMENTATION_PLAN.md and CBM_A*/B* records | IMPLEMENTED — HISTORICAL / EVIDENCE | Preserved; current topology distilled |
| CBM_SUBPROCESS_SESSION_DAEMON_RECONCILIATION_INVESTIGATION.md | INVESTIGATION / EVIDENCE | Preserved; ownership truth distilled |
| QUERY_HISTORY_INDEPENDENCE_AND_DISCOVERY_COMPLETENESS_INVESTIGATION.md | SUPERSEDED — HISTORICAL | Banner added |
| FINAL_RELEASE_BLOCKER_AUDIT_2026-10-16.md | INVESTIGATION / POINT-IN-TIME EVIDENCE | Banner added |
| ANGULAR_TEMPLATE_CORRECTNESS_AND_LINKAGE_PLAN.md; invocation plans; apply-edit plans | IMPLEMENTED — HISTORICAL | Preserved |
| HEURISTICS_ENGINE_V2.md; TYPE_AWARE_COMPRESSION_PLAN.md; other unimplemented proposals | PROPOSAL / FUTURE unless their own status says implemented | Preserved |

### Changelogs, verification, and ledgers

| Paths | Classification | Action taken |
| --- | --- | --- |
| docs/changelogs/** | HISTORICAL RELEASE RECORD | All 38 retained; active changelog user change preserved |
| verification/** | OPERATOR DEFINITIONS / EVIDENCE | All 48 retained and classified as non-test evidence |
| docs/agent/DISCOVERY_REGISTRY.md | INVESTIGATION / APPEND-ONLY EVIDENCE | Retained |
| docs/agent/*HANDOFF*.md; docs/SESSION_HANDOFF_2026-08-28.md | HISTORICAL HANDOFF | Retained |
| docs/PERFORMANCE.md | SUPERSEDED — HISTORICAL BENCHMARK | Existing banner retained; public claims removed/qualified |
| dated top-level audits | INVESTIGATION / EVIDENCE | Retained |

## D. Material stale claims found

| Document | Old claim | Current production truth | Evidence | Severity | Resolution |
| --- | --- | --- | --- | --- | --- |
| README.md | Semantic tuple was canonical identity and same entity across files | Tuple groups occurrences; exact declaration identity is separately proven | WorkspaceIndex occurrence storage; final release audit Model C trace | High | Corrected |
| README.md | Selector spelling resolves to the component | Only exact published selector authority plus one unique candidate yields typed refinement | angular_templates catalog/refinement and template tests | High | Corrected |
| README.md | All fidelities preserve task semantics without qualification | Bounded projections may omit declared categories; Edit/Verbatim are separate exact contracts | projection metadata, CTX-001, #141 acceptance | High | Corrected |
| README.md | Broad benchmark percentages were current product guarantees | Results are fixture/fidelity/history dependent | docs/PERFORMANCE.md historical banner and benchmark context | Medium | Removed from public table; linked as history |
| README.md | All tests pass / clean lint as evergreen status | Verification is revision- and run-specific | release acceptance and verification policy | Medium | Replaced with named RC/version evidence wording |
| README.md | Clean-CTX subprocess represented CBM server/runtime ownership | Child is an owned frontend/session; daemon/runtime is shared and provider-owned | CBM lifecycle source and pinned-upgrade investigation | High | Corrected |
| Historical #142 investigation | Candidate architecture could be mistaken for pending/current authority | Decisions were approved and implemented | ISSUE_142 final acceptance and d3e92dfa | Medium | Superseded banner |
| Mixed framework docs | Phase plans could be read as current full authority | Current sections remain useful; phase schedules are history | current source/tests plus later acceptance records | Medium | Banners and index classification |

## E. New or updated architectural authority

The current spine is:

    README.md
      -> docs/README.md
      -> docs/ARCHITECTURAL_INVARIANTS.md
      -> docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md
      -> docs/agent/tooling.md and docs/agent/verification.md
      -> framework references
      -> historical plans/evidence

The semantic architecture now states Model C identity, acquisition/freeze/
refinement order, query-history independence, batch union semantics, earned
completeness, dependency validity, projection fidelity, domain ownership,
Angular template authority, C#/.NET binding rules, CBM topology/optionality,
apply_edit authority, and evidence-class separation.

## F. Historical documents deliberately preserved

Implementation plans, issue acceptance reports, CBM migration evidence,
benchmarks, changelogs, investigations, handoffs, and verification definitions
remain because they explain decisions, alternatives, and the evidence available
at a named time. Rewriting them into current-tense authority would falsify
history. The index and targeted banners establish precedence instead.

## G. Quantitative claim audit

| Current public claim checked | Status/evidence | Action |
| --- | --- | --- |
| Four language layers and three framework meta-layers | CURRENT + EVIDENCED by Cargo feature declarations | Retained |
| Package version 0.10.0 | CURRENT + EVIDENCED by Cargo.toml | Retained |
| provide_code_context batch 1–8 | CURRENT + EVIDENCED by schema/tooling | Retained |
| workspace_query operation list | CURRENT + EVIDENCED by tool schema/tooling | Retained |
| 61–97% compression | HISTORICAL / fixture-dependent | Removed from README |
| Up to 53% delta savings | HISTORICAL benchmark, not universal | Removed from current feature/table claim |
| 26 filters | Volatile count; not needed for contract | Removed from README |
| About 90% prompt-cache savings | UNVERIFIED as universal claim | Removed from README |
| 20+/30+ relation counts | Volatile marketing counts | Removed from README |
| All tests passing / zero warnings | Point-in-time evidence only | Removed as evergreen README status |
| Rust 1.85+ | Current public prerequisite; not runtime-reverified in this static audit | Retained with uncertainty noted |

Historical benchmark/changelog numbers remain where dated and classified.

## H. Remaining uncertainty

- No runtime command verified installation snippets, minimum supported Rust, or
  host-specific IDE behavior during this audit.
- CBM was unavailable to the active Clean-CTX tools; production topology was
  established from current adapter source and accepted pinned-upgrade evidence,
  not a live provider session.
- Large mixed-era subsystem guides may contain low-severity stale mechanics
  outside the audited authority keywords. Their status is now subordinate, but
  a future guide-by-guide modernization could improve ergonomics.
- No external link checker was run; internal paths touched by this change were
  checked statically.

## I. Runtime verification not performed

No Cargo test, check, build, Clippy, rustfmt, audit, benchmark, Clean-CTX binary,
external/live MCP pilot, operator harness, model test, server, or release build
was run. The already-connected Clean-CTX tool surface was used only for
read-only status/context inspection; CBM reported unavailable. Other work was
static Git/filesystem inspection, text search, patch application, and static
diff/encoding/link inspection. The user-owned final verification command
remains in docs/agent/verification.md.

## Production discrepancies

No production behavior was changed. No release-blocking contradiction was found
between the accepted release-candidate implementation and the reconciled
authority model. Generic graph families intentionally remain lower-bound; that
is a contract, not a documentation workaround.
