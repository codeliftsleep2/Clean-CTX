# OPS-R1 — Disposition 21 remaining preservation execution

Agent MaxHeadRoom — 2026-11-20. Sixteen authorized controls; no retries.

## Result

**16 passed, 0 failed, 0 ignored.** Each child selected exactly one tracked test.
Original selection order was preserved. No watchdog/resource stop, monitoring
failure, source drift or executable drift was observed. No builds or source
changes occurred. The sequence stopped after the sixteenth authorized case.

The selected cases are exactly entries 10–25 of the original 25-case preservation
manifest (excluding fitness). Entries 1–9 were reconciled against the retained
original bundle; no overlap, duplicate or omission was introduced. Each selector
was matched through the adapter case tuple to the exact manifest test identity.

## Execution identities and procedure

- Source tree: `25d17848f53e607313b877bdb64f5d791d280e20f11dc935c244d880f1b1a8eb`.
- Executable: `target/debug/deps/clean_ctx-c147437a0212f889.exe`.
- Executable SHA-256: `e2d03e433a4137750f9511dcc5e1178541b049c53a66094799d9660cbe92b403`.
- Build receipt SHA-256: `714f362e9ab3d949fc4cbda84759872f80cc7b85020a6808b600b347415205b7`.
- Manifest SHA-256: `21734cd76b0d8a2a008b07964d31238b39354dce75af1456059ff71862d567a0`.
- Adapter SHA-256: `eab639d02d55659fb41ea307895f19a9bb4fb486a448860bb16ae47329ee14d3`.

All pinned identities and the aggregate source digest were verified before and
after each case. The unchanged adapter also performed its normal source/binary
verification and exact single-test execution. Fresh evidence directories used
the existing monitor, independent watchdog, 60-second deadline, 512 MiB observed
working-set threshold, five-second stop wait and exact-child termination policy.

Sequential orchestration definition:
`verification/ops-r1/proposals/shared-acquisition/execution/invoke-remaining-21.py`.
It invokes the unchanged adapter; it does not implement a new test runner,
change test selection or monitor behavior, or build. Script SHA-256:
`a8a02e3f0e745664c1e8ef07f03155b467227887341d29710be670aed827b5b5`.

Command executed once:

```powershell
python .\verification\ops-r1\proposals\shared-acquisition\execution\invoke-remaining-21.py
```

## Individual outcomes

Exact full test identities, per-case receipt paths/hashes, PIDs, raw output hashes
and sampling details are in the bundle record and evidence index below.

| Reviewed selector | Outcome | Child seconds | Sampled peak MiB |
| --- | --- | ---: | ---: |
| source_and_package_changes_invalidate_proof | 1 passed | 0.107 | 2.34 |
| request_directory_and_inherited_project_remain_authoritative | 1 passed | 3.037 | 12.38 |
| multiple_projects_and_operations_do_not_share_authority | 1 passed | 0.302 | 11.95 |
| same_config_different_automatic_type_lookup_stays_distinct | 1 passed | 0.106 | 2.45 |
| concurrent_owners_and_nested_acquisition_are_isolated | 1 passed | 0.241 | 13.43 |
| bootstrap | 1 passed | 36.184 | 78.52 |
| hydration_and_explicit | 1 passed | 8.542 | 66.35 |
| worker_order_non_ts | 1 passed | 0.414 | 18.97 |
| worker_failure_non_ts | 1 passed | 0.105 | 2.37 |
| batch_prepublication | 1 passed | 0.106 | 2.35 |
| typescript_prepublication | 1 passed | 1.615 | 29.96 |
| typescript_persistence | 1 passed | 3.844 | 74.89 |
| resolver_positive | 1 passed | 54.781 | 39.79 |
| retirement | 1 passed | 3.462 | 33.40 |
| csharp | 1 passed | 0.710 | 22.12 |
| ambient_dependency | 1 passed | 0.105 | 2.33 |

Total child elapsed: 113.66022420010995 seconds, excluding orchestration and
identity-verification overhead. Bootstrap took 36.18 seconds and resolver-positive
54.78 seconds. These are observations from this capture, not a benchmark or
evidence of overall production performance improvement. Sparse sampling in short
cases does not establish true peak memory or absence of leaks.

## Evidence and attribution

Evidence directory: `target/ops-r1/remaining-21-20261120-210439/`.

- `bundle-review.json`: `baa94b215ba8a9e82b489862d39f9a3ba1fd40e81013aa3252ec3169245048cb`.
- `evidence-index.json`: `0a809ae679e727e7f349a9899806e91e215e6614df574ec65393c6d35b85a7a4`.
- Per-case adapter stdout/stderr and original child evidence remain retained.

All sixteen receipt hashes and raw stdout/stderr/memory hashes were checked.
Original nine-case evidence was checked against the Disposition 17 index and
remains byte-identical. Production files and historical fitness source remain
unchanged relative to that index.

Preservation accounting across execution identities:

- Eight original passes remain attributable to the preceding GREEN source.
- The original case-9 failure remains preserved and failed under that identity.
- Corrected case 9 passed separately on amendment 18.
- These sixteen passed on amendment 18; zero selected controls remain unexecuted.

This is a reconciled set of preservation observations, not a claim that all 25
were rerun together against the amended executable. Historical passes were not
relabeled as new-source executions.

## Remaining boundaries

The corrected configuration control does not independently prove option-key
completeness: changed authoritative config bytes may cause currentness rejection.
No mutation testing, fault injection, full regression suite, CI change, protected
grant, graph admission, commit, push or publication occurred. Selected preservation
success does not establish field behavior, leak absence, overall performance
acceptance or closure of the broader OPS-R1 audit.

This checkpoint continues the OPS-001 mission record. Actual applicable governing
association, graph reconciliation and document admission remain separate before
publication; protected declarations were not changed.

Stopped after the authorized sequence. Return these observations for the next
owner disposition; no further execution or implementation is inferred.
