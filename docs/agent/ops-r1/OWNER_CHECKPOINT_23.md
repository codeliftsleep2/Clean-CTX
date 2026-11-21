# OPS-R1 - Disposition 23 isolated M3 preparation

Agent MaxHeadRoom - 2026-11-20. Preparation only; compilation and execution withheld.

## Result and candidate location

Prepared an isolated M3 source candidate at:
`C:/Users/MNasty/Desktop/RustContextLayerAI/target/ops-r1/m3-isolated-23`.

The accepted working source, accepted executable and accepted case-9 receipt
remain unchanged. Only the isolated copy of
`src/ir/typescript_resolution/shared_acquisition.rs` was mutated. No test source,
configuration fixture, qualification, currentness, sealing, publication or CI
implementation was changed. No build, test or benchmark ran.

| Identity | SHA-256 |
| --- | --- |
| Accepted source tree | 25d17848f53e607313b877bdb64f5d791d280e20f11dc935c244d880f1b1a8eb |
| Isolated mutant source tree | 96117651f06c96a11a63aa3cab70b8e4ab903e2f75fcaf6622e5f037987cab92 |
| Exact approved M3 proposal | af2bc315dccfee30a64d193f3d27fa569ffdb8ccfa77d11a887ff84b31eb8d88 |
| Original production file, raw | 65116e3a7a30390f7f04ba157673cb47ec18456d19fe3644dab3868f11817cd8 |
| Isolated mutated file, raw | f94f1646cced272e3b8bea41c9cfe15cadd4993b7e896e8468ef25287a1ab20f |
| Unchanged corrected test, raw | 7f87059494f87b780067875bf71e35de17417df6c988bdceff74764dba8ca582 |
| Accepted executable | e2d03e433a4137750f9511dcc5e1178541b049c53a66094799d9660cbe92b403 |
| Accepted case-9 receipt | b9e462461a6741a609e1e770cf8f61e067ca7eb1ab6809068b3697590e0095dd |

Exact mutation source:
`verification/ops-r1/proposals/shared-acquisition/review-22/M3-collapse-strict-false.patch`.
It passed git apply --check and was applied only within the isolated repository.
Normalized source-set comparison establishes exactly one file difference from
the accepted tree. Raw test bytes match in both locations.

## Isolation provenance and deviation

A local shared-object, no-checkout clone returned exit 128 before source copying;
the captured exception did not retain Git's stderr, so no failure cause is claimed.
No accepted files or Git metadata were changed by a successful clone operation.

Preparation continued using independent Git metadata initialized within the
isolated directory, a detached accepted HEAD
`0dd8d7503af3116241ad7c1e23c293b1d5849cc1`, a private index initialized from that HEAD,
and a read-only object-alternates reference to the existing object database.
The candidate was populated from the actual accepted working bytes, including
the pinned untracked source files; the older committed HEAD is not represented
as the accepted source snapshot. No fetch, commit, push or checkout of the
accepted worktree occurred. Removing/moving the accepted object database would
break this local alternates relationship; this candidate is not a standalone
durable historical archive or publication artifact.

The isolated target directory does not yet exist, and no accepted executable was
copied into it. Any future build must use the isolated target path, with no
inherited CARGO_TARGET_DIR/CARGO_BUILD_BUILD_DIR redirect to accepted artifacts.
This is output isolation, not a change to Cargo features or execution policy.

## Exact changed symbol and effect

`shared_acquisition.rs:150`, function `basis`, gains only the six added lines in
the pinned patch, after config/supplied-input eligibility checks and before JSON
key serialization. For an effective strict value equal to JSON false, it clones
Project, changes only that clone's options["strict"].value to JSON true and calls
basis with the clone. The recursive call terminates because true does not match
the false predicate. Config identity, option origin directory and every other
field remain unchanged in the clone. The original Project is never mutated.

The provisional key for strict=false is therefore intentionally equivalent to
strict=true under otherwise equal context. This is a deliberate isolated defect
in one grouping distinction, not a proposed product correction.

The patch does not change `Project::load`, `project_config::document`, source
bytes, selected config, `reusable_sources`, `ProjectProof::qualify`, Witness::current,
program publication, any test expectation, build command or safety policy.
Static scope review establishes these boundaries; compilation remains unverified.

## Fixture preconditions and expected detection

Exact unchanged test:
`ir::typescript_resolution::ops_r1_shared_acquisition_controls::isolation::ops_r1_shared_config_roots_and_route_changes_stay_independent`.
Source: `src/tests/ir/ops_r1_shared_acquisition_isolation.rs:8`.

| Key dimension | Second context | Fifth context |
| --- | --- | --- |
| Requested owner | b.ts | b.ts |
| Actual selected config | Same tsconfig.json | Same tsconfig.json |
| Effective strict value before mutation function | true | false |
| Option origin | Same declaring config directory | Same declaring config directory |
| Project root/base, paths, mode, conditions | Same explicit bundler/module setup | Same |
| Root set, aliases, source bytes, request parent/extension | Same | Same |
| Ownership/config candidate structure | Same discovered root-member context | Same |
| Capture demand policy | controls | controls |

This table is source-backed static reasoning, not captured runtime key telemetry.
The only config mutation is strict=true to strict=false. No project references,
root declarations or derived module-resolution settings change. Sources and roots
stay unchanged. First/third/fourth contexts do not have strict=false, so M3 should
leave their basis calculation unchanged. In particular, the fourth equivalent
dot-path request should still satisfy the positive sharing assertions.

Expected precise failure: line 59, the explicit assertion
`assert_ne!(basis(&changed, &b, &changed_project).unwrap(), second_basis)`.
The preceding loaded strict=false assertion at line 58 should still pass. The
later changed acquisition has not run when this assertion executes. Thus a failure
at this exact assertion would independently demonstrate detection of M3 through
key discrimination, without relying on config-currentness rejection.

A failure in an earlier assertion, compiler error, timeout, resource stop or
selection mismatch is not the intended detection evidence. Unexpected results
require investigation and a stop, not edits or automatic retries.

The accepted case-9 pass remains the control. No M3 failure has yet been observed.
Even successful detection of M3 would establish this one false/true distinction,
not completeness of every effective option, origin or authority key field.

## Proposed verification infrastructure - not applied

Review artifacts under
`verification/ops-r1/proposals/shared-acquisition/review-23/`:

- `isolation-receipt.json`: candidate identity and preparation provenance.
- `reviewed-green-m3-23-sources.json`: separate proposed mutant source pin.
- `isolated-adapter-pin.patch`: proposed isolation-only registry/selection change.
- `execution-proposal.json`: exact proposed identities and test selector.
- `review-index.json`: artifact/source evidence hashes.

Proposed manifest SHA-256:
`c5348d3dd24e4903610cd7a137037e4fbe6e59b1eaab2206016908e28d26ef1f`.
Proposed adapter patch SHA-256:
`71c2ce412c5eb41907f4cd2d73fdd39ebdbcc077e12b8f789d3ae9d8a55b1358`.
Hypothetical amended adapter SHA-256:
`f45f674d43d099ffd33889dee4c203e1f79fbd7393c210d5d9b1d96269f05b7b`.

These are proposals, not an installed or executed adapter. The isolated adapter
still has its accepted hash and will reject the mutant under its old source pin.
Future approval must include installing the separate manifest and applying this
adapter amendment only in the isolated candidate. No unpinned override is proposed.
All existing historical manifest selections and monitor behavior remain intact.

The proposed manifest was statically reconciled using the unchanged verifier's
tracked-path plus manifest-path enumeration and CRLF normalization; it reproduces
the mutant aggregate source hash exactly. Its shorter path list includes the
original required untracked source paths; all tracked source paths remain covered
by the existing git ls-files enumeration. No source verification was bypassed.

## Proposed staged handoff - DO NOT RUN under Disposition 23

After separate approval of the isolated pin installation, the proposed build is:

```powershell
Set-Location 'C:\Users\MNasty\Desktop\RustContextLayerAI\target\ops-r1\m3-isolated-23'
$env:CARGO_TARGET_DIR=Join-Path (Get-Location) 'target'
$env:OPS_R1_GREEN_SOURCE='m3-23'
pwsh -NoProfile -File .\verification\ops-r1\proposals\shared-acquisition\execution\invoke-build-capture.ps1 -Stage green -OutputDirectory target/ops-r1/shared-m3-23-build-reviewed
```

Underlying command remains `cargo test --lib --all-features --no-run --message-format=json`.
A completed no-run build and separately approved exact executable/build receipt
would then be prerequisites to this proposed one-case execution:

```powershell
$env:OPS_R1_STAGE='green'
$env:OPS_R1_GREEN_SOURCE='m3-23'
python .\verification\ops-r1\proposals\shared-acquisition\execution\capture.py --build-receipt .\target\ops-r1\shared-m3-23-build-reviewed\build-receipt.json --case config_roots_and_route_changes_stay_independent --wall-seconds 60 --memory-mib 512 --stop-wait 5
```

The execution command is not authorized now. It retains the existing exact-case
selection, independent watchdog, 60-second wall deadline, 512 MiB observed memory
stop, five-second stop wait, exact-child termination and partial evidence capture.
No accepted binary, receipt or expected test outcome would be overwritten.

## Stop and governance

Accepted source, test and immutable controls remain unchanged; M1/M2 were not
applied. No build, test, benchmark, CI change, commit, push or publication occurred.
The only mutation applied is M3 within the isolated source snapshot.

This record continues OPS-001 without changing protected declarations or grants.
Actual applicable graph reconciliation and document admission remain separate
before publication; no fresh admission is claimed. The current PCG preflight
coverage limitation remains explicit. Stopped for owner review before compilation
or test execution.
