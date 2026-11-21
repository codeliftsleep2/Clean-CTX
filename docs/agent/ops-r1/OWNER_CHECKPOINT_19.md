# OPS-R1 — Disposition 19 corrected-test compilation receipt

Agent MaxHeadRoom — 2026-11-20. Compilation only; execution withheld.

## Result

The single authorized build succeeded: exit 0, 120.4678828 seconds, zero compiler
messages, zero errors and zero warnings. No tests executed. No production source
or test assertions changed under this disposition.

Exact command, through the amended source-verifying capture launcher:

```powershell
$env:OPS_R1_GREEN_SOURCE='amendment-18'
pwsh -NoProfile -File .\verification\ops-r1\proposals\shared-acquisition\execution\invoke-build-capture.ps1 -Stage green -OutputDirectory target/ops-r1/shared-green-amendment-18-build-reviewed
```

Underlying command: `cargo test --lib --all-features --no-run --message-format=json`.
The output directory was absent before launch. Source verification ran before
and after compilation through the existing capture path; no bypass was used.

## Source and verification identities

- Corrected source tree: `25d17848f53e607313b877bdb64f5d791d280e20f11dc935c244d880f1b1a8eb`.
- Corrected test raw SHA-256: `7f87059494f87b780067875bf71e35de17417df6c988bdceff74764dba8ca582`.
- Approved test patch: `7a4f68c8491e3d4edbf76d99ca50a2e7d18bb230e33c958cbde6f128660129de`.
- New separate manifest: `verification/ops-r1/proposals/shared-acquisition/execution/reviewed-green-amendment-18-sources.json`.
- Manifest SHA-256: `21734cd76b0d8a2a008b07964d31238b39354dce75af1456059ff71862d567a0`.
- Amended adapter: `verification/ops-r1/proposals/shared-acquisition/execution/capture.py`.
- Adapter SHA-256: `eab639d02d55659fb41ea307895f19a9bb4fb486a448860bb16ae47329ee14d3`.
- Unchanged build capture script: `af7ee07da3d00df45d21bef0c2b5a78165dd0ed5e112aedc9ec6735092ea42a0`.

Exact adapter diff: `target/ops-r1/disposition-19-amendment/adapter.patch`, SHA-256
`fcbbc1a0c8946e0c488db5fc81bdc78b21aa437f433e1ae60452bceed24592bd`.
Pre-amendment adapter bytes are retained alongside it as `capture-before.py`.
Static checks and historical manifest hashes are in that directory's
`static-receipt.json`.

The adapter adds a hash-pinned amendment-18 manifest and selects it for corrected
GREEN by default. `OPS_R1_GREEN_SOURCE=amendment-15` retains explicit selection of
the previous GREEN manifest; unknown GREEN amendment values are refused. RED
continues selecting its original manifest. Receipt metadata reports the selected
manifest's actual hash. Cargo, test selections, watchdog, monitor, executable
verification and evidence capture mechanisms are unchanged.

All three historical source manifests remain byte-identical. The new manifest
contains normalized per-file source hashes and retains the original source-tree
verification algorithm. Production source and the historical fitness source were
compared against the pre-test-correction evidence index and remain unchanged.
Historical receipts were not rewritten. Historical behavior is retained through
explicit old-manifest selection and preserved adapter bytes, not claimed as a
newly rerun historical execution.

## Executable and evidence

Executable: `target/debug/deps/clean_ctx-c147437a0212f889.exe`.
SHA-256: `e2d03e433a4137750f9511dcc5e1178541b049c53a66094799d9660cbe92b403`.

Evidence folder: `target/ops-r1/shared-green-amendment-18-build-reviewed/`.

| Evidence | SHA-256 |
| --- | --- |
| build-receipt.json | 714f362e9ab3d949fc4cbda84759872f80cc7b85020a6808b600b347415205b7 |
| build.jsonl | 196e23015e266c63f990574ae1af9fa70159fd644d4c79ea9a04f705dd5c2a3b |
| build-stderr.txt | d163c6ab381ee46a2cd996da93b656cc3dcf718d8b61c5d256b9c442db8d8bcb |

`build-review.json` records parsed diagnostics, original receipt and evidence
hashes. This is actual compilation evidence, not test execution, architectural
GREEN, mutation-testing success, CI success or production integration evidence.

## Preserved boundaries and next checkpoint

Original case-9 failure remains failed under its historical source identity;
eight preceding controls retain their successful receipts; sixteen remain
unexecuted. Neither corrected case 9 nor the preservation bundle was run.

Fault-detection limitation remains: changing config bytes may independently
invalidate currentness witnesses. This test does not independently certify
option-key completeness, and no mutation experiment was performed. The positive
control's unnecessary-reacquisition detection and negative control's unsafe-reuse
detection are still execution obligations, not established outcomes.

This checkpoint continues the governing OPS-001 mission record without changing
its protected declaration. Fresh graph reconciliation/admission, CI, authority
grants, commits, pushes and publication were not performed. Applicable document
association/admission remains separate before publication.

Stopped after the single compilation. Next owner decision is whether to authorize
bounded execution of corrected case 9 against this exact source/executable and
amended adapter. No execution is inferred from successful compilation.
