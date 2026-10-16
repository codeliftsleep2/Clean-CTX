# A5 live acceptance — blocked cloud account cache

Date: 2026-10-11 UTC. Historical read-only-environment run; Phase A acceptance
was **not established by this run**. Subsequent environment recovery and safe
live acceptance are recorded in
[the current A5 report](CBM_A5_ENVIRONMENT_RECOVERY_AND_ACCEPTANCE.md).

## Artifact and branch

- Source revision: `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`.
- Executable: `/workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp`.
- Pre-run and post-run SHA-256, identical:
  `6dfaa906ed310795a97ca9b992d4d42b6118a8cf740ea7b4550df6d8146b24f0`.
- Gate 0 provenance: [record](CBM_A5_GATE0_ARTIFACT_PROVENANCE.md).
- Clean-CTX branch `feature/current-cbm-pinned-upgrade`, A4 commit
  `593a2c7a3dc0e6a74d27648be68eb023b6028530`; required parent `4c1627f...`
  ancestry and all five A0–A4 commits reverified before execution.
- The authorized current Clean-CTX build completed with exit 0 in 10.85 seconds:

```bash
cd /workspace/Clean-CTX
PATH=/workspace/.tooling/cargo/bin:$PATH RUSTUP_HOME=/workspace/.tooling/rustup CARGO_HOME=/workspace/.tooling/cargo cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4
```

Clean executable SHA-256:
`aa3071beedfceef072cfadadb86dc1c8affe8cb1982f2987ae485d7f4009b06f`.
No tests, Clippy, provider rebuild or production changes were performed.

## Environment and harness

Debian GNU/Linux 13.6, x86-64; account `agent`, UID 1000.

Primary fixture:
`/workspace/Clean-CTX/target/cbm-a5/20261011-204103/primary`.
Additional fixture:
`/workspace/Clean-CTX/target/cbm-a5/20261011-204103/additional`.
Each contains one distinct TypeScript function. Only primary has an acceptance
configuration: `cbm.binary_path` is the exact pinned executable and
`additional_roots` contains the additional fixture. No cache override or PATH
shadowing was supplied. Fresh fixture configuration introduced these fields;
no existing user configuration was overwritten.

Normal provider cache `/home/agent/.cache/codebase-memory-mcp` was absent before
and after the run. A read-only `os.statvfs('/home/agent/.cache')` observation
established `ST_RDONLY == true`. Thus this account cannot create the normal
absent provider cache on this filesystem. No alternative cache root was used.

The prepared `verification/cbm/A3_ACCEPTANCE.md` first-frontend case was exercised
with a tracked bootstrap-stage PowerShell harness, following the existing
ProcessStartInfo/stdin/stdout MCP-session pattern. It is deliberately not a
claim of a complete eleven-case harness. The first failed acceptance assertion
stopped the matrix. Linux `strace` observed the unmodified Clean-CTX process and
its direct provider, recording process/chdir/write events. It was not a provider
wrapper, fake frontend or substitute provider CLI query. Trace instrumentation
can affect timing; no general timing claim is made.

PowerShell 7.4.19 was extracted locally under `/workspace/.tooling/powershell`.
The Microsoft signing key fingerprint was
`BC528686B50D79E339D3721CEB3E94ADBE1229CF`; the repository InRelease signature,
Packages.gz hash and package SHA-256 were verified before extraction.
Package hash: `bbd6fdb8e97bd7533b21cb01d7ca876a084bf485fb46f26fd3f70e775f089fc2`.
The initial GitHub release-metadata request returned 403; Microsoft's signed
Debian repository supplied the verified package instead. Initial PowerShell
startup also encountered the read-only home filesystem. Its own XDG data,
configuration and cache directories were relocated under `/workspace/.tooling`.
Pinned provider source uses HOME/CBM_CACHE_DIR rather than these XDG variables;
the provider cache remained the normal account path.

Executed command (the script was subsequently renamed to describe its scope):

```bash
XDG_CACHE_HOME=/workspace/.tooling/powershell-cache XDG_CONFIG_HOME=/workspace/.tooling/powershell-config XDG_DATA_HOME=/workspace/.tooling/powershell-data /workspace/.tooling/powershell/opt/microsoft/powershell/7/pwsh -NoProfile -File verification/cbm/scripts/Invoke-A5LiveAcceptance.ps1 -BinaryPath /workspace/Clean-CTX/target/debug/clean-ctx
```

Current tracked script name: `Invoke-A5BootstrapAcceptance.ps1`. Syntax validation
passed before the live run. Per-case observation limit was 90 seconds; this case
completed and cleaned up in approximately one second. Evidence totals 15,089
bytes; trace is 9,045 bytes and captured stderr 729 bytes.

## Case matrix

| Case | Result | Evidence class | Key observation |
| --- | --- | --- | --- |
| 1. First frontend/bootstrap | FAIL | PROVEN BY LIVE PINNED ARTIFACT | Cache canonicalization failed; provider exited before admission. Classified ENVIRONMENT DEFECT. |
| 2. Required live registry | BLOCKED | NOT ESTABLISHED | No initialize response or tools/list reached. |
| 3. Compatible daemon | BLOCKED | NOT ESTABLISHED | Bootstrap did not create an admitted session/daemon. |
| 4. Two frontend isolation | BLOCKED | NOT ESTABLISHED | No admitted first session. |
| 5. Canonical cache/root routing | BLOCKED | PROVEN BY LIVE PINNED ARTIFACT / NOT ESTABLISHED | Correct primary CWD observed; cache canonicalization/cohort and extra-root routing unestablished. |
| 6. Direct frontend retirement | BLOCKED | PROVEN BY LIVE PINNED ARTIFACT / NOT ESTABLISHED | Failed child reaped and workers exited; planned admitted-session/sibling retirement not reached. |
| 7. Observational recovery | BLOCKED | NOT ESTABLISHED | Initial admission failed; no eligible A4 recovery exercised. |
| 8. Mutation non-replay | NOT RUN — SAFETY BOUNDARY | NOT ESTABLISHED | No safe live uncertainty mechanism established; no uncertain indexing mutation attempted. |
| 9. Controlled exit during admission | BLOCKED | NOT ESTABLISHED | Provider exited naturally during startup; no controlled termination case run. |
| 10. Incompatible cohort refusal | NOT RUN — SAFETY BOUNDARY | NOT ESTABLISHED | No approved isolated mismatch seam established; account/cache state left untouched. |
| 11. Representative registered CBM MCP path | BLOCKED | NOT ESTABLISHED | Registered get_cbm_status was used; representative successful graph/proxy consumption was not reached. |

The failed acceptance assertion does not establish an adapter defect. It records
that this environment cannot currently satisfy first-frontend bootstrap.

## Actual live admission and cleanup trace

**PROVEN BY LIVE PINNED ARTIFACT**, from `A.trace`:

1. Clean-CTX PID **76320** launched the exact provider as direct child **76321**.
2. Child `chdir` to the canonical primary fixture succeeded.
3. Clean wrote initialize request ID **1**, protocol `2024-11-05`, clientInfo
   `clean-ctx` version `0.9.3`, capabilities `{}` to the owned provider pipe.
4. Provider wrote to stderr:
   `codebase-memory-mcp: exact executable identity could not be verified (cache-canonicalize)`.
5. Provider exited **1**. No provider stdout response, initialized notification,
   tools/list or background index_repository request was observed.
6. Clean reaped child 76321 using wait4. Stdout/stderr workers **76322/76323**
   exited. Clean logged `Admission failed: CBM connection lost: CBM exited`.
7. Registered `get_cbm_status` returned `_meta.cbm_status = "degraded"`, no graph
   version and text `CBM not available.` It did not claim provider availability.
8. Harness closed Clean stdin. Clean logged normal worker shutdown and exited
   **0**. No force kill was needed. Postflight found no matching Clean/provider
   processes. Trace contains no daemon launch or global shutdown command.

This trace does not establish the successful A3 sequence. It establishes truthful
unavailability and directly owned failed-child cleanup in this startup failure.
No registry tools were observed; required-tool presence remains **NOT ESTABLISHED**.
No structured provider refusal was emitted in this path. The stderr diagnostic
was retained as raw evidence, not promoted into a typed build/ABI/cohort refusal.

**PROVEN BY PROVIDER SOURCE**: `src/main.c:1424–1461` resolves the normal cache,
tries to canonicalize/create it, and returns CACHE_CANONICALIZE if it is not a
canonical directory. `main.c:3036–3044` prints the observed diagnostic and exits
before the daemon/session handshake. This explains why no structured JSON-RPC
admission response was emitted. The exact failing filesystem syscall was not
captured; attributing that individual syscall to EROFS is **NOT ESTABLISHED**.
The absent cache and read-only parent filesystem are directly observed environment
facts and suffice to establish a missing bootstrap prerequisite.

## Ownership, recovery and remaining limits

Daemon bootstrap/reuse, cohort identity, admitted sibling isolation, final-session
daemon policy and canonical cache admission are **NOT ESTABLISHED**. No daemon
was killed, private cohort created, installation replaced or account repaired.
The direct frontend cleanup described above is partial A2 live evidence only.
A4 terminal → replacement → admission → safe replay was **not exercised**.
Deterministic A4 mutation/recovery evidence remains unchanged.

Native Clean initialize and registered status requests worked after provider
startup failed. A native source-context request was scheduled after the bootstrap
assertion and therefore did not run; native context correctness is not claimed.

Windows live deployment/lifecycle is not established by this Linux A5 run.
Phase B tool/request/result compatibility is not generally established;
paging/continuation, watcher freshness and completeness remain unestablished.
Issue #142 was neither implemented nor tested.

## Preserved evidence and changes

Run directory: `target/cbm-a5/20261011-204103/`.
Files: `A.trace`, `A.stderr`, `rpc.json`, `preflight.json`, `postflight.json`,
`case1-status.json`, `case1-processes.json`, fixture configuration/source and
Clean-owned fixture persistence/cache outputs. No provider account cache appeared.
All provider artifact hashes remained unchanged. New tracked candidates are this
report, the Gate 0 report and the bootstrap harness; no production files changed.

Next prerequisite: an acceptance environment where the normal operator-owned
account cache path is writable. Do not substitute a private CBM_CACHE_DIR,
rewrite provider configuration or repeat the matrix without resolving that
environment prerequisite. No Clean-CTX repair is proposed by this evidence.

**A5 BLOCKED — ACCEPTANCE ENVIRONMENT FAILURE**
