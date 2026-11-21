# OPS-R1 - Mission Directive 24 engineering report

Agent MaxHeadRoom - 2026-11-20. Governing mission: OPS-001.

## Outcome

The selected fault-detection and test-hardening workstream is complete.

- M3 was detected at the intended acquisition-basis inequality assertion, before
  changed-context acquisition or currentness qualification could determine refusal.
- Three original preservation controls demonstrably passed with sharing disabled.
  Their hardened versions detect that fault, and pass with the accepted implementation.
- Six registered test-source files under src/tests were strengthened or decomposed. No production
  implementation changed relative to the accepted starting baseline.
- Thirty-six distinct focused tracked controls passed. Five affected controls were
  rerun after the final canonical-source-owner assertions; all five passed.
- Final scoped Clippy passed with zero warnings. Mutations remain isolated.

This closes this bounded workstream, not the entire OPS-R1 audit or all repository
verification. No CI policy, protected declaration, grant, commit, push or publication
was performed.

## Root causes and implemented corrections

| Finding | Implemented correction and source |
| --- | --- |
| Correct semantic output could come from independent acquisition, leaving claimed shared-proof coverage unestablished | `src/tests/ir/ops_r1_shared_acquisition_proof.rs:43`, `:130`, `:183` now require [1,1,2] acquisition work and one accepted qualification; attached-unit coverage also requires an attached proof |
| Directory-sensitive type acquisition could rely on fallback refusal without directly checking grouping distinction | `src/tests/ir/ops_r1_shared_acquisition_isolation.rs:156` checks equal selected config but unequal acquisition bases, alongside existing 3-versus-4 unit results |
| C# preparation count and top-level success did not establish successful batch semantics | `src/tests/mcp/csharp_call_authority_lifecycle.rs:86` requires Current authority, ordered successful items, one Go-to-Hit Calls edge per item, Method/builtin identities and exact canonical source owners |
| Recovery substring/nonempty checks were insufficient source and authority evidence | `src/tests/mcp/semantic_publication_recovery.rs:219` and `:290` check target source hash, applicable current authority, exact hydrated Class identity/count/source, and explicit TypeScript authority source ownership |
| A public query could reacquire state before retirement preservation was examined | `src/tests/mcp/semantic_retirement_controls_issue_162.rs:96` and `:124` inspect retained owner identity before the public-query comparison |
| One test combined eight cold scenarios and exceeded the existing process deadline | `src/tests/mcp/angular_resolver_registration_lifecycle_issue_162.rs:8` now registers four form-specific tests; helper `:28` retains both directions, fresh state/database conditions and every original directional assertion |

The original directional body is identical after indentation normalization, SHA-256
`302b4c488a284409ebcc6634239d50c495e9a5e90d56fe878521465783e91125`.
Four forms times two directions still means eight scenarios. There are no rustfmt
exclusions, increased diagnostic limits or removed behavioral assertions.

No generic fitness framework, new mandatory CI policy or production workaround
was introduced. Existing tracked tests and the existing bounded capture adapter
were reused. Small orchestration definitions record exact selections and evidence;
the Rust assertions remain the tests.

## Demonstrated M3 detection

The isolated fault maps effective strict=false to strict=true only during provisional
key construction. It preserves loaded config, project selection, source authority,
proof sealing, qualification, currentness and publication.

The original corrected test was byte-identical during this experiment. Its loaded
strict=false assertion and positive dot-path reuse assertions passed. It then failed
at `src/tests/ir/ops_r1_shared_acquisition_isolation.rs:59`, `assert_ne!` on the two
bases. Raw output contains equal serialized keys, both with strict=true. Acquisition
work was still [3,3,6], so the fifth acquisition had not executed.

| M3 evidence | Actual result |
| --- | --- |
| Source tree | 96117651f06c96a11a63aa3cab70b8e4ab903e2f75fcaf6622e5f037987cab92 |
| Executable SHA-256 | 752226a7682cbcd23be795145720138bb8281ef8ec40c3a70becb7151f897020 |
| Selected outcome | Exactly one test, 0 passed, 1 failed, 0 ignored; Rust exit 101 |
| Failure attribution | Explicit key inequality at original line 59 |
| Safety/identity | No resource/watchdog stop; source and executable verified after execution |
| Receipt SHA-256 | 9100596f15ab0bc7ee586145bc08ca7e108f46200039212fa1dda35d6a803a0e |

Receipt location:
`target/ops-r1/m3-isolated-23/target/ops-r1/p0-config_roots_and_route_changes_stay_independent-20261120-221227/receipt.json`.

This is attributable detection of M3. It is not a compiler failure, timeout, unrelated
earlier assertion failure or an inference from counters alone. No currentness
safeguard was weakened. Later formatting changed the accepted function's raw bytes;
its lexical tokens/string literals were reconciled separately, without relabeling
the original M3 execution as a run of the formatted successor.

## Demonstrated shared-proof reachability correction

One isolated M2 fault disables operation-slot selection, forcing independent project
acquisition without otherwise changing source/qualification truth contracts.

| Exact selected control | Original control under M2 | Hardened control under M2 | Accepted implementation |
| --- | --- | --- | --- |
| seed_is_not_sibling_disk_authority | Passed with [2,2,4] | Failed new reuse assertion, [2,2,4] versus [1,1,2] | Passed |
| standalone_capture_and_missing_module_qualification | Passed with [2,2,4] | Failed new missing-module reuse assertion | Passed |
| precached_attached_unit_replays_only_its_source | Passed with [2,2,4] | Failed new attached-unit reuse assertion | Passed |

The original control file was restored byte-for-byte only inside the isolated mutant
for the survivor comparison. The accepted control file retained its hardening.

- Hardened-M2 source: `2fe8c86f1b39d69ae286d4f24d4d5a79fb4e71768ab2c627ff0a06eaae461f58`.
- Hardened-M2 executable: `3d40c8b32a25a61b11cb0cba64c30aa245c5c29688a3914f7496bb3814e3faad`.
- Legacy-control M2 source: `1b088ca6914cba88981f5b7c59708ac73acf6a62aee01a57614664ea7cab8f9a`.
- Exact original control-file hash: `a05260c5402dea1594890d21a9b9306c4787fa0c71bc22508196854b37dc2230`.

Review records:
`target/ops-r1/reachability-24-hardened-fail-20261120-223310/review.json` and
`target/ops-r1/reachability-24-legacy-pass-20261120-223526/review.json`.
Each contains three exact single-test receipts. All six experiments completed
without safety stops. Expected mutant failures are recorded as failed Rust tests;
the experiment's successful attribution does not relabel them as test passes.

M1 was not applied: M3 provided a narrower, successfully attributed key-discrimination
experiment. Individual qualification safeguards can still overlap; detecting M2
does not establish independent mutation coverage for every byte/currentness guard.

## Regression, lifecycle and failure validation

The focused sequence retained all original selected obligations and reused existing
confirmed-deletion, acquisition-denial, wrong-kind, historical-retirement, recovery
refusal and C# preparation-failure tests. No duplicate lifecycle test was added.

Execution accounting is explicit:

1. Thirty-five controls passed on source
   `ac089ba19aeacf88e85f94a8752577546caca29b7a3f53d4049d646a431e3f8c`.
   This includes four bounded cold-registration tests replacing one combined test.
2. One additional existing C# failure control passed on that same source and binary:
   36 distinct controls, not a single 36-test invocation.
3. Canonical-owner assertions changed only two cross-consumer test files. The five
   affected positive/refusal controls were rerun on the final source and all passed.
   Other control bodies and production source were unchanged; those earlier runs
   retain their actual source identity.

Final source tree:
`c9a8c4eb39fdbde19a34e320cfcc5a8222adbc91940af8aa9ce9b09f24c1ab7f`.
Final executable:
`target/debug/deps/clean_ctx-c147437a0212f889.exe`, SHA-256
`ce6dc4454eea4bd667eda9bc7185209914ad7fc7fac0cd65e9a13d6eefb58c7c`.

All compilation captures used `cargo test --lib --all-features --no-run --message-format=json`;
all eight captured builds succeeded with zero compiler diagnostics. The metadata-only
supplemental build did not recompile source. Final scoped lint:
`cargo clippy --lib --tests --all-features -- -D warnings`, exit 0, zero warnings,
with final source identity verified before and after.
Receipt: `target/ops-r1/mission-24-clippy-final/receipt.json`.

The final accepted controls had no resource/watchdog stops, ignored tests, ambiguous
selections or identity drift. Each child used the existing 60-second deadline,
512 MiB observed working-set threshold, five-second stop wait and exact-PID monitor.

## Investigation failures preserved and resolved

- An initial new recovery assertion required TypeScript authority on ordinary
  declaration hydration. The failed execution established that this fixture uses
  the permitted LegacyEdit path. `query/entities.rs:101` selects that requirement;
  `semantic_compilation/retained.rs` defines applicable currentness. The corrected
  helper requires TypeScript authority for explicit provision and validates
  applicable retained authority plus exact source hash for ordinary hydration.
  It does not manufacture proof or change production admission.
- The original eight-scenario registration test hit WALL_CAPTURE_STOP at 60 seconds,
  with no semantic assertion failure. The stop is preserved as a resource stop.
  Decomposition retained every scenario/assertion and passed under unchanged limits.

These failures were investigated rather than erased, retried unchanged, treated as
architectural RED or concealed by weaker expectations. Their source variants, logs,
receipts and corrected successors are retained.

## Evidence and preservation

Artifact directory:
`verification/ops-r1/proposals/shared-acquisition/review-24/`.

- `validation-index.json`: execution, build, source and binary identities.
- `accepted-executions.csv`: 36 exact controls, latest applicable receipt per control.
- `final-control-matrix.csv`: current source anchors, detection strength and remaining limits.
- `hardening-tests.patch`: reviewable six-file test diff against the accepted starting inputs.
- `validation-evidence.zip` and `validation-evidence-members.csv`: 598 retained evidence
  members with SHA-256 values, including initial failures and raw successful outputs.
- `test-source-history.zip`: six test files across five reconciled source variants.
- `m3-exact-source.zip`, `m2-hardened-exact-source.zip`: isolated source snapshots.
- `verification-definition-history-final.zip`: exact executed adapters, manifests,
  monitor and capture definitions. Reconstructed historical bytes were retained
  only after matching the recorded SHA-256.
- `scoped-check-receipt.json` and `scoped-text-checks.csv`: scoped UTF-8/BOM,
  active-file line counts and unchanged-production/historical-evidence checks.

These scoped static checks are not represented as full repository guards or actual
document admission. Final source identity was verified again after validation.

Evidence archive SHA-256:
`855b73b7b17a2a2809921de150c40e82` is only a suffix; use the complete value in
`validation-evidence-sha256.txt` and `validation-index.json` for verification.

Accepted starting binary, M3 binary and hardened-M2 binary were preserved separately
under `target/ops-r1/accepted-amendment-18-control`, `m3-binary-control-24` and
`m2-hardened-binary-control-24`. Reusing the isolated build directory did not reuse
an identity: each stage has distinct source/executable/adapter receipts and archived
inputs. The original case-9 failure and earlier owner-accepted evidence remain intact.

## Limits and next phase

This work proves specific fault detection and selected semantic preservation.
It does not prove all-option key completeness, every overlapping refusal predicate,
all cancellation/panic schedules, all repository tests, full CI, production-scale
latency benefit, memory-leak absence or the original field-hang cause. Public written-registration evidence does not certify
resolver executable eligibility or completeness beyond its existing lower-bound
semantics. The previously
observed increase in qualification/currentness work remains separate from reduced
acquisition visits. Sampling and successful bounded completion are not leak proofs.

Next priority is selective guard-specific probes where significant masking remains,
followed by bounded attribution of total qualification/currentness costs and the
original real-workspace failure. Do not expand this into a manual audit of thousands
of tests or repeat an active Claude hang. No additional production repair is justified
by this test-hardening workstream.

This report continues OPS-001. Protected declarations/grants were not changed;
actual graph reconciliation and document admission remain required before publication.
Full repository gates remain CI-owned and were not run locally. No CI success,
field acceptance, protected admission, commit, push or publication is implied.
