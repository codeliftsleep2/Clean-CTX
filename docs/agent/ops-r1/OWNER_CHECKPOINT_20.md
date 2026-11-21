# OPS-R1 — Disposition 20 corrected case-9 execution

Agent MaxHeadRoom — 2026-11-20. One authorized execution, no retry.

## Outcome

Exactly one tracked test selected; **1 passed, 0 failed, 0 ignored**, exit 0.
Test identity:
`ir::typescript_resolution::ops_r1_shared_acquisition_controls::isolation::ops_r1_shared_config_roots_and_route_changes_stay_independent`.

The observed control satisfies its positive-equivalence and negative-divergence
assertions. This does not independently prove option-key completeness or
mutation-testing effectiveness.

## Execution and identities

```powershell
$env:OPS_R1_STAGE='green'
$env:OPS_R1_GREEN_SOURCE='amendment-18'
python .\verification\ops-r1\proposals\shared-acquisition\execution\capture.py --build-receipt .\target\ops-r1\shared-green-amendment-18-build-reviewed\build-receipt.json --case config_roots_and_route_changes_stay_independent --wall-seconds 60 --memory-mib 512 --stop-wait 5
```

The reviewed adapter's case tuple [1,9] was matched to the selection manifest and
the exact required test identity before execution. All six owner-pinned identities
were verified before launch. The current source set matched the normalized
per-file manifest and its aggregate digest. The adapter created a fresh output
directory and performed source/executable verification before and after execution.

- Source tree: `25d17848f53e607313b877bdb64f5d791d280e20f11dc935c244d880f1b1a8eb`.
- Test file: `7f87059494f87b780067875bf71e35de17417df6c988bdceff74764dba8ca582`.
- Executable: `target/debug/deps/clean_ctx-c147437a0212f889.exe`.
- Executable SHA-256: `e2d03e433a4137750f9511dcc5e1178541b049c53a66094799d9660cbe92b403`.
- Build receipt SHA-256: `714f362e9ab3d949fc4cbda84759872f80cc7b85020a6808b600b347415205b7`.
- Amendment-18 manifest SHA-256: `21734cd76b0d8a2a008b07964d31238b39354dce75af1456059ff71862d567a0`.
- Capture adapter SHA-256: `eab639d02d55659fb41ea307895f19a9bb4fb486a448860bb16ae47329ee14d3`.

## Observations and their evidence strength

| Observation | Result | Evidence category |
| --- | --- | --- |
| Equivalent dot-path physical/logical identity and basis | Assertions satisfied | Passing pinned tracked test; individual values not emitted |
| Intermediate acquisition counts | [3,3,6] | Assertion satisfied; no separate stage record |
| Dot-path successful qualification | Exactly one additional accepted qualification | Assertion satisfied |
| strict option | true for second context, false for fifth | Assertions satisfied; values not separately emitted |
| Changed acquisition basis | Different | Assertion satisfied |
| Final acquisition attempts/completions/unit visits | [4,4,8] | Explicit stderr aggregate and assertion |
| No accepted qualification for fifth request | Assertion satisfied | Passing tracked test |
| Returned project units | Two on every acquire | Helper assertion satisfied; no separate unit list |
| Qualification counters | [1,1,210,54,59] | Explicit stderr aggregate |

The qualification fields are attempted qualification, accepted qualification,
seal witness checks, recipient witness checks and qualification input visits.
No per-stage acquisition receipt, serialized group key or temporary-root path
was emitted; the passing assertions must not be relabeled as raw stage telemetry.

PID 5284. Monitor elapsed 1.6224716000142507 seconds; libtest reported 0.64 seconds.
Baseline observed working set 1,568,768 bytes; sampled peak 13,176,832 bytes over
16 samples. These samples do not establish true process peak or leak absence.
No watchdog/resource stop or monitoring failure occurred. Existing independent
watchdog, 60-second deadline, 512 MiB threshold, five-second stop wait and
exact-child termination policy remained unchanged.

## Evidence return

Folder: `target/ops-r1/p0-config_roots_and_route_changes_stay_independent-20261120-205902/`.

| File | SHA-256 |
| --- | --- |
| receipt.json | b9e462461a6741a609e1e770cf8f61e067ca7eb1ab6809068b3697590e0095dd |
| stdout.txt | 681ff7e9823a5df0582341ebeb40ee7bab7903456a3e94b741978ca03cbf6204 |
| stderr.txt | 87dc9af6de1f267eb2c7e167a6c47923a03dcc0d07d1fbc49a2fc2ab39df1eee |
| memory.jsonl | 33223aa3562450bcecc0041dca821547cb4519654497d367578dc05f046f8c30 |
| monitor-receipt.json | d9b03552b620e6a2eefd7fff7ca3847ea0ea9ca2d785cc9dd1528447ebf5158f |
| case-review.json | 8f3940dac1ff2c650ae0fa3a3a4f385e45f001abf98dda18a3d7b9c6b26fe5bf |

## Stop and preserved limitations

No source edits, builds, retries, other test executions, CI changes, commits,
pushes, protected grants or publication occurred. Original nine-case evidence
files were rehashed against the investigation index and remain unchanged. The
original ninth-case failure remains failed under its original identity; eight
earlier passes remain historical passes. Sixteen controls remain unexecuted.

Changed authoritative config bytes may cause currentness rejection independently
of grouping discrimination. This execution demonstrates the specified observable
behavior, not which guard caused every refusal, complete option-key coverage,
mutation-testing success, full preservation acceptance or mission closure.

This checkpoint continues the OPS-001 mission record; no fresh graph admission or
protected declaration change was performed. Applicable association/reconciliation
and document admission remain separate requirements before publication.

Stopped after the single authorized corrected-case execution. Resumption of the
sixteen remaining preservation controls requires the next owner authorization.
