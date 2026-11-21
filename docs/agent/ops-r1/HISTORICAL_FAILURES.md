# OPS-R1 historical architectural failure registry

Agent MaxHeadRoom. Evidence-based initial registry; not a complete defect history.
Read source assertions and retained evidence, not test names alone. Existing
functional tests remain valuable even where the architectural counterexample was absent.

The [authority/evidence revision snapshot](../../../verification/ops-r1/authority-and-evidence-revisions.json)
records 28 source hashes and verified member/archive hashes for four selected
historical bundles. It proves custody of those sources, not new execution.

| Failure | Root cause / violated boundary | Shared exposure | Existing verification and escape | Preventive enforcement / evidence standing |
| --- | --- | --- | --- | --- |
| HF-001 repeated complete CoreIR query compilation | Immutable binary-owned TS/C# definitions entered per-observation dynamic compilation; AF-PERF bounded work/reuse distinction. | CoreIRPass and capture consumers; custom query path must remain dynamic. | Output/equivalence tests can pass with an extra Query::new. Source parsing still required, so parse-count checks alone cannot expose query compilation. | GARC-001 / AFF-003: unchanged warm query compilation count zero plus independent parse/cursor and exact-definition controls. Historical tracked RED and GREEN archives exist. |
| HF-002 feature-isolation failures | Conditional compilation and cross-layer feature leakage; all-feature wiring concealed C#-only/TS-only failures. | Query registration, native TS evidence, Angular/DotNet test helpers. | CI all-feature execution and broad default-feature lint did not compile supported isolated modes. Tests behind those modes could disappear rather than fail in all-feature CI. | GARC-002 / AFF-004: separate supported feature compile/registration inventory. Historical compiler RED is valid build-failure evidence, not behavioral RED. Focused five-mode GREEN exists. |
| HF-003 deleted-owner relationship survival | Shared retirement failed to remove established-deleted source contributions while preserving independent support. | Legacy Route ResolvedBy query, typed registrations and other shared semantic families. | Original registration implementation exposed a pre-existing legacy query defect. Positive membership and target identity tests do not exercise owner relocation/deletion/recreation. | GARC-003 / AFF-011: exact original legacy query witness plus deletion/acquisition failure/sibling/multiple-support controls; frozen RED/GREEN preserved. |
| HF-004 restore enablement derived from optional identity | Configuration authority conflated with optional Angular compatibility identity, rejecting valid unchanged native registration authority. | Current and historical configuration-aware restore; low-level identity wrapper intentionally narrower. | Matching-identity/happy restore tests did not exercise enabled configuration with absent routing identity. | GARC-004 / AFF-008: unchanged-source boundary witness plus config/native/current/history/forged/stale matrix. Exact RED retained; first patched 20-pass/one-new-test-error capture stays distinct from later two-test GREEN. |
| HF-005 Angular absent-route query hang/memory growth | Confirmed broad preparation before selector; retained batches/fresh project acquisition are credible related mechanisms. Exact blocking/leak attribution remains unmeasured. | A01–A13 census; no universal defect claim. | Registration-count growth held owners fixed; output/185 synthetic checks did not vary workspace/project scale. | GARC-005 / AFF-001/002/005/007: independent-axis cold absent/no-resolver reproduction. Owner-reported three field failures plus static confirmation; tracked behavioral RED still pending. |
| HF-006 governed document admission bypass/gap | Protected membership/rule/authority provenance can be bypassed if guards substitute for actual admission. | Architectural document authoring and publication. | UTF-8, formatting and file-size checks cannot validate source-owned graph authority. Existing B11 adversarial tests protect specific transactions; not proof every author invoked admission. | GARC-006 / AFF-012: existing actual rejection/provenance tests plus pipeline enforcement design. Historical production incident proving each bypass is NOT established in this packet; classify known governance failure family and prevention gap, not invented historical RED. |
| HF-007 CI work duplication and measurement contamination | Equivalent correctness/fitness execution plus repeated empirical sampling and repeated Cargo build work; verification lacked responsibility separation. | CI workflow, fitness runner, measurement companions. | Correct tests can all pass while scheduler repeats them. Counts and correctness do not protect execution responsibility. | GARC-007 / AFF-012: exact membership/outcome reconciliation, explicit measurement/companion lists, reject duplicate/ignored/zero selections. CI-VERIFY-R1 receipts and run history retained. |
| HF-008 parser attribution failure | Merged Cargo headers/nested libtest output attributed incorrectly; successful tests could yield false gate failure. | CI evidence reconciler. | Simple fabricated output missed interleaving and child executions. | GARC-008 / AFF-012: real frozen output plus adversarial parser cases; original execution remains independent from parser correction. |

## Exact supporting sources

- HF-001: src/tests/ir/ci_runtime_core_query.rs and
  src/tests/compression/core_ir_registration.rs; verification/ci-verify-r1/
  core-query-red-index.json, core-query-verification-index.json and their archives.
- HF-002: feature-isolation-red-evidence.zip, feature-isolation-green-index.json,
  feature-isolation-repair-ledger.json and owner-disposition-03.txt in that directory.
- HF-003: src/tests/mcp/angular_deleted_owner_membership_issue_162.rs,
  semantic_retirement_controls_issue_162.rs; historical-witness-evidence.zip and
  ANG_R2_RETIREMENT_REPAIR.md under verification/live-acceptance/issue-162/ang-r2.
- HF-004: src/tests/mcp/restore_registration_boundary_issue_162.rs,
  restore_registration_config_matrix_issue_162.rs; restore-validator-red-index.json
  and restore-validator-green-index.json in that same mission directory.
- HF-005: existing field report/independent source trace in ACTIVE_INCIDENT.md;
  src/tests/mcp/angular_resolver_registration_lifecycle_issue_162.rs growth body.
- HF-006: src/tests/pcg/document_admission_authority.rs and diagnostics counterpart;
  actual DOC-270/VERIFY-001 admission receipts remain distinct from those tests.
- HF-007/008: src/tests/architecture/ci_verify_r1.rs,
  scripts/verify-ci-membership.py, retained CI membership and parser RED/GREEN indexes.

The [escape analysis](ESCAPE_ANALYSIS.md) separates source-confirmed gaps from
historical execution proof. Exact hashes/revisions belong in the authority and
historical evidence manifest, not hand-retyped or silently updated assertions.
