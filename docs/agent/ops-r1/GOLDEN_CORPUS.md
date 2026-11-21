# OPS-R1 Golden Architecture Regression Corpus design

Agent MaxHeadRoom. Proposed logical corpus using tracked tests and existing CI.
No new executable, scheduler, generic framework or independent proof authority.

The machine-readable [golden design](../../../verification/ops-r1/golden-corpus-design.json)
records exact retained CI registration for seven entries, current authority/source
SHA-256 values, feature applicability, counterexamples, evidence and limits.
GARC-005 stays pending rather than receiving an invented registered identity.

Each approved entry will carry: GARC/AFF IDs, authority path and SHA-256 revision,
protected behavior, defect source/execution identity, production consumers,
positive/negative tests, package/target/full registered name, source/body hashes,
feature mode, required CI location, deterministic failure predicate, limitations
and RED/GREEN evidence associations. Hashes are observations, not frozen forever
without a governed successor procedure. Current source hashes are not historic ones.

| GARC | AFF / behavior | Exact source test anchor | Positive / negative counterexample | Historical and CI standing |
| --- | --- | --- | --- | --- |
| GARC-001 | 003/010; closed CoreIR registration | src/tests/ir/ci_runtime_core_query.rs::ci_runtime_core_query_warm_observation_avoids_recompiling_binary_owned_query | Same captures/instructions, own parse/cursor; query compilation becomes nonzero on historical code. core_ir_registration.rs adds exact definitions, custom/mismatch and concurrency. | Frozen original source SHA bdc4c262d8b5f799e7c5fa4de44cdd16a662d493124d173a9cbab11f125387b3 and RED archive. Current allfeatures suite includes witness. Isolated configs need explicit CI design. |
| GARC-002 | 004/010; supported feature boundaries | src/tests/ir/native_feature_evidence.rs::ci_runtime_native_typescript_evidence_preserves_source_ownership plus core_ir_registration.rs controls | Native source-owned evidence; C#-only/TS-only build must succeed without importing the other's layers. | Historical compiler failures and focused five-mode GREEN retained; mandatory CI isolated-mode enforcement proposed, not installed. |
| GARC-003 | 011/008; source-owned deletion | src/tests/mcp/angular_deleted_owner_membership_issue_162.rs::issue_162_deleted_owner_legacy_membership_is_retired | Initial/relocated/recreated membership; deleted owner cannot survive. semantic_retirement_controls_issue_162.rs covers error-not-deletion and independent supports. | Exact original RED/GREEN archived; formatting-approved successor identities preserved. Allfeature CI executes tracked successor. |
| GARC-004 | 008; actual config establishes restore enablement | src/tests/mcp/restore_registration_boundary_issue_162.rs::issue_162_unchanged_bootstrap_durable_registration_boundary | Enabled/absent-identity valid unchanged restore; config matrix rejects forged/stale/missing required evidence and preserves historical qualifications. | Exact RED archive and split GREEN ledger; current complete CI passes. No old capture relabeled patched. |
| GARC-005 | 001/002/005/007; absent-route progress | Proposed tracked cold no-resolver/absent-route regression, NOT YET AUTHORED | Empty lower-bound or correct explicit failure on stable input; independent owner/project growth detects unjustified repeated work. | Field report plus source confirmation. No invented test identity, allocation proof, behavioral RED or CI membership. P0 missing protection. |
| GARC-006 | 012/008; admitted document authority | src/tests/pcg/document_admission_authority.rs::b11_rejection_exposes_exact_accepted_governing_reference_despite_candidate_policy_edit | Existing accepted reference returned; forged candidate policy cannot rewrite rejection authority. diagnostics tests cover fail-closed location/publication. | Existing adversarial tests; actual new document admission remains operator-owned. Historical mutation of a publication pipeline bypass not established. |
| GARC-007 | 012; required execution membership | src/tests/architecture/ci_verify_r1.rs::ci_verify_r1_required_execution_and_failure_contracts | Valid attributed evidence accepted; missing/duplicate/ignored/empty/measurement execution cases rejected. | Existing actual Python verifier driven by fixture mutations; reuse in ordinary CI. Does not prove unlisted arbitrary regressions remain mandatory. |
| GARC-008 | 012; robust execution attribution | Existing consolidated parser fixtures/indexes and ci_verify_r1.rs | Real nested/interleaved Cargo/libtest evidence accepted; malformed/unattributed records rejected. | Frozen parser RED/GREEN evidence. Parser correction is not production query repair. |

## Identity and enforcement design

Resolve full registered identities from the machine census and actual module
registration, not namespace-to-file inference. A unique function-name match is a
candidate, not proof. Mark ambiguous/macro-generated cases unresolved. Required
configuration identities are package/target/name/feature mode; source presence is
insufficient. Empirical measurements are separately enumerated with mandatory
assertion companions; existing nine legacy ignores stay explicit, not generalized.

Add approved entries to the existing verification manifest and existing verifier.
It must fail on missing registration, missing outcome, unexpected ignore/duplicate,
wrong feature mode and changed protected assertion meaning. Source/body hashes aid
review but cannot mechanically prove assertion meaning; combine existing frozen
witness rules, AST facts and governed review of substantive changes.

No new mandatory GARC policy is applied before owner approval. Existing
verification.md remains the sole execution authority. Corpus design and proposed
membership need actual B11 review/admission where protection changes.
