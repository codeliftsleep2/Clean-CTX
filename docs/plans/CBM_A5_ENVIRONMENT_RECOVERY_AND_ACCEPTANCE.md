# A5 environment recovery and live pinned acceptance

Date: 2026-10-11 UTC. No production changes. Phase B and Issue #142 not started.

## Environment recovery

Debian GNU/Linux 13.6, x86-64, kernel 6.18.44; account `agent`, UID 1000.
Account database and HOME both identify `/home/agent`. The provider's normal
cache is `/home/agent/.cache/codebase-memory-mcp`; no CBM_CACHE_DIR override.

Option A provisioning was unavailable: exposed platform tools do not create a
replacement environment. Option B succeeded through the platform's normal
`request_permissions` mechanism granting write access to `/home/agent/.cache`.
No privileged remount command, HOME substitution, provider-specific cache
override or global account configuration change was used.

Preflight after the grant observed `ST_RDONLY == false`, writable access and a
successful temporary create/remove probe. The provider directory was absent;
the provider created it naturally during first bootstrap. It is now a canonical
directory owned by agent with mode 0700. Provider-owned `_config.db`, logs and
fixture graph databases were naturally created and retained, not deleted.

The platform grant is **turn-scoped**. A subsequent user message expired the
first grant. A launch then failed with `cache-private`, explicitly reporting
`chmod 0700 failed (errno 30)`. The same normal-path permission was reacquired,
preflight rechecked, and successful acceptance resumed. This second environment
failure is preserved at `target/cbm-a5/20261011-210446/`; it is not adapter failure.
Future runs must establish writability again. The harness now probes before
provider execution so an expired grant is reported at preflight.

Historical first read-only failure remains recorded separately in
[A5 original evidence](CBM_A5_LIVE_ACCEPTANCE_EVIDENCE.md).

## Artifact

Reused without rebuilding:
`/workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp`.
Source revision `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`.
Pre- and post-run SHA-256:
`6dfaa906ed310795a97ca9b992d4d42b6118a8cf740ea7b4550df6d8146b24f0`.
Source/build/runtime provenance is [Gate 0](CBM_A5_GATE0_ARTIFACT_PROVENANCE.md).
Clean-CTX commit is A4 `593a2c7a3dc0e6a74d27648be68eb023b6028530` on the required
feature branch with verified required-parent ancestry. The previously authorized
all-feature Clean build was reused; no extra build, tests or Clippy runs.

## Runs, fixture scope and configuration

First recovered bootstrap: `target/cbm-a5/20261011-204502/`.
Complete safe matrix: **`target/cbm-a5/20261011-210512/`**.

Final matrix roots:

- Primary: `/workspace/Clean-CTX/target/cbm-a5/20261011-210512/primary`.
- Additional: `/workspace/Clean-CTX/target/cbm-a5/20261011-210512/additional`.

Each final fixture has its own Git repository boundary and one TypeScript
function, `a5PrimaryProbe` or `a5AdditionalProbe`. Primary `.clean-ctx.json`
contains the exact `cbm.binary_path` and additional_roots entry. No existing
user configuration was replaced. The first recovered run lacked fixture Git
boundaries; its list_projects branch metadata reflects the enclosing Clean-CTX
branch. Final fixtures use independent repository boundaries and `master`
metadata. Both runs' provider state is owned by this acceptance, and retained.
The final run is cold in daemon/session state but **warm in account cache state**.

The prepared A3 matrix was extended narrowly for A4 recovery. PowerShell uses
the existing ProcessStartInfo/stdin/stdout session pattern. Linux strace observes
the unmodified pinned provider; no provider wrapper or direct CLI graph query.
Each case has a 90-second deadline, with 12 MiB trace and 1 MiB saved stderr
observation limits. Final traces total 6,743,330 bytes; final matrix took about
15 seconds. System-call tracing affects timing; no production timeout change.

Exact final execution command, working directory `/workspace/Clean-CTX`:

```bash
XDG_CACHE_HOME=/workspace/.tooling/powershell-cache XDG_CONFIG_HOME=/workspace/.tooling/powershell-config XDG_DATA_HOME=/workspace/.tooling/powershell-data /workspace/.tooling/powershell/opt/microsoft/powershell/7/pwsh -NoProfile -File verification/cbm/scripts/Invoke-A5BootstrapAcceptance.ps1 -BinaryPath /workspace/Clean-CTX/target/debug/clean-ctx -ContinueMatrix
```

XDG paths initialize PowerShell under the workspace. Provider source resolves
HOME/CBM_CACHE_DIR, not XDG; its normal cache remained under the real account.

## Case matrix

| Case | Result | Evidence class | Key observation |
| --- | --- | --- | --- |
| 1. First frontend/bootstrap | PASS | PROVEN BY LIVE PINNED ARTIFACT | Normal cache created, daemon bootstrapped, admission completed, native fixture context returned. |
| 2. Required live registry | PASS | PROVEN BY LIVE PINNED ARTIFACT | All seven required tools present once with object input schemas; 17 total tools. |
| 3. Compatible existing daemon | PASS | PROVEN BY LIVE PINNED ARTIFACT | B admitted while daemon PID 78060 remained shared with A. |
| 4. Two frontend/session isolation | PASS | PROVEN BY LIVE PINNED ARTIFACT | A and B independently answered list_projects; B retirement left A usable. |
| 5. Canonical account/cache/root routing | PASS | PROVEN BY LIVE PINNED ARTIFACT | Normal cache, primary CWD, one frontend for two roots, root-explicit results returned distinct fixture symbols. |
| 6. Direct frontend retirement | PASS | PROVEN BY LIVE PINNED ARTIFACT | B direct child retired/reaped, A and daemon remained live and answered. |
| 7. Observational recovery | PASS | PROVEN BY LIVE PINNED ARTIFACT | One killed A frontend led to one replacement, complete admission and one list_projects dispatch. |
| 8. Mutation non-replay | NOT RUN — SAFETY BOUNDARY | PROVEN BY LOCAL FIXTURE / NOT ESTABLISHED LIVE | No safe bounded uncertain mutation stimulation established; A4 deterministic authority unchanged. |
| 9. Exit during admission | PASS | PROVEN BY LIVE PINNED ARTIFACT | Only C's direct frontend killed before admission; degraded status, native context and A sibling remained usable. |
| 10. Incompatible cohort refusal | NOT RUN — SAFETY BOUNDARY | PROVEN BY LOCAL FIXTURE / NOT ESTABLISHED LIVE | No approved safe isolated mismatch seam; compatible account/cohort untouched. |
| 11. Registered Clean MCP path | PASS | PROVEN BY LIVE PINNED ARTIFACT | cbm_proxy list_projects/search_graph reached real provider and returned nonempty fixture evidence. |

PASS is limited to the stated observations, not general Phase B compatibility,
coverage or freshness. Full refusal/mutation/live negative-result behavior was
not established by the safe matrix.

## Admission and registry trace

Final A trace observed:

1. Clean PID **78055** → frontend **78056**, exact executable and primary CWD.
2. Initialize ID **1**, protocol `2024-11-05`; provider replied with that version,
   serverInfo `codebase-memory-mcp`/`0.11.0` and tools capability object.
3. No-ID `notifications/initialized` sent.
4. tools/list ID **2** and complete 17-tool response captured.
5. Clean's current-code capability validator accepted the response. This
   internal validation step is **PROVEN BY CURRENT CODE**, externally supported
   by the successful transition and complete matching live registry.
6. Background index_repository IDs **3** and **4** occurred only after tools/list;
   registered status was available and indexing complete.

Required: index_repository, search_graph, query_graph, trace_path,
get_architecture, list_projects, detect_changes. Additional tools:
get_code_snippet, get_file_outline, get_graph_schema, compare_graphs,
search_code, delete_project, index_status, check_index_coverage, manage_adr,
ingest_traces. Presence is not full schema compatibility.

Raw bounded trace is preserved. Provider stdout was reassembled across its
4 KiB write chunks by writer PID, fd and newline; derived registry/RPC JSON
files preserve complete evidence rather than treating partial writes as frames.
A has **two** full registries (original and replacement); B has one; C has none.

## Daemon/session ownership and cache

Daemon **78060** bootstrapped from A's frontend and normal account cache. B's
frontend **78210**, owned by Clean **78209**, admitted without another daemon;
snapshots preserve both private frontends plus the same daemon. Both answered
observational requests. Closing B through normal Clean EOF retirement killed
only its direct frontend; A subsequently answered list_projects.

After the last original frontend was intentionally killed for recovery, the
provider daemon independently stopped: trace records
`daemon.runtime_stopping reason=last_committed_client_disconnected` and
`daemon.stop`. Replacement frontend **78470** bootstrapped daemon **78474**.
This is observed provider final-session policy, not Clean daemon ownership.
Final Clean shutdown killed/reaped only its frontend; provider again stopped
its own last-session daemon. No global lifecycle command or account repair.

PID 1 in this container retains exited daemon PIDs as **zombies** (PPID 1, STAT Z),
including 76762, 78060 and 78474. They are not live daemons or owned frontend
leaks. Clean cannot reap an adopted non-child; no unrelated/process-global
cleanup was attempted. Postflight executable-based snapshots omit zombies;
separate ps observations establish this declared environment limitation.

Cache realpath and ownership/mode were observed; project databases map to
the explicit fixture identities. Exact internal cohort/ABI identity is not
exposed by initialize. Compatibility is evidenced by live reuse/admission of
the same exact artifact, not an invented structured fingerprint response.

## Recovery and controlled admission exit

Recovery stimulus killed only direct frontend 78056, with no daemon signal.
Registered cbm_proxy(list_projects), a proven A4 observational operation,
encountered the terminal transport and caused exactly one frontend replacement.
A trace captures fresh initialize **1**, initialized, registry **2**, then a
single list_projects **3**. Result `_meta.cbm_error` is null and status available.
Original terminal/EOF is observed by child exit and transport lifecycle;
successful recovery does not expose an original error in the final MCP response.

This was controlled between requests. It establishes safe recovery from an
already-dead frontend; it does **not** prove replay after an original request was
accepted mid-flight. Only one actual provider list_projects dispatch occurred
for this recovered demand. There was no uncertain mutation or repeated stress.

C Clean **78545** had direct frontend **78546** killed before initialize returned.
C trace has initialize request only, no response/initialized/registry/indexing.
It reaped that child, reported degraded, and returned exact native TypeScript
context. A still answered list_projects. No structured refusal was emitted;
none is claimed preserved in this signal-induced path.

Registered project-explicit search_graph calls returned one matching function
each, with distinct primary/additional qualified prefixes, through normal
cbm_proxy compression. list_projects exposed the acceptance-owned projects,
including earlier fixture state. No successful-empty result substituted for
these positive fixture queries. Failure-result projection generally remains
covered by deterministic A4 evidence, not proved here.

## Changes, limits and next gate

Provider source/artifact unchanged. No production edits, daemon takeover,
global install/update or private provider cache. New/updated repository artifacts
are reports, acceptance documentation and the operator harness. Fixture Git
metadata, local Clean persistence/cache and provider-owned account fixture state
were generated. Account data was retained; no persistent provider state deleted.

Windows live deployment/lifecycle is not established by this Linux A5 run.
Phase B tool/request/result compatibility is not generally established;
paging/continuation, watcher freshness/completeness remain unestablished.
Issue #142 was neither fixed nor tested. Mid-flight accepted-request replay,
live mutation uncertainty and incompatible cohort behavior remain limitations.

**PHASE B REQUIRES SEPARATE AUTHORIZATION**

**PHASE A ACCEPTED WITH DECLARED LIVE LIMITATIONS**
