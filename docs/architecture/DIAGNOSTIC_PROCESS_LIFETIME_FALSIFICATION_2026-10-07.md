# Diagnostic Process-Lifetime Falsification

**Date:** 2026-10-07  
**Investigator:** Agent MaxHeadRoom  
**Status:** Windows and GitHub Actions Ubuntu survived bounded process-ownership falsification with explicit platform limitations.  
**Scope:** OS-level ownership and cancellation semantics for a closed diagnostic process tree. No production supervisor, Cargo command, Clean-CTX runtime, or MCP experiment.  
**Starting decision chain:** Semantic compilation survived; executable resolution survived with an absolute-path constraint; parser truthfulness survived.

## Evidence vocabulary

- **PROVEN** — established by controlled evidence within its stated scope.
- **DOCUMENTED** — guaranteed by an authoritative platform source.
- **OBSERVED** — measured in the stated environment without universal generalization.
- **INFERRED** — reasoned from observed and documented evidence.
- **UNRESOLVED** — required evidence has not been obtained.

---

## 1. Verdict

**PROCESS OWNERSHIP SURVIVES WITH REQUIRED ARCHITECTURAL LIMITATIONS**

Windows process ownership survived the bounded local falsifier. GitHub Actions Ubuntu process-group ownership survived all six bounded Linux cases. No ordinary-descendant kill condition was observed.

The positive Linux claim is deliberately narrow: **OBSERVED on GitHub Actions `ubuntu-latest` in run 37665077341**, not proven for all Linux or Unix environments. Ordinary inherited descendants remained in the dedicated session/process group across two generations and after root exit. Group SIGTERM and SIGKILL reached the ordinary tree. The controller observed completion only after all recorded fixture PIDs were absent and the process group itself was no longer signalable.

Two limitations are architectural requirements, not footnotes. A descendant can explicitly escape with `setsid()`, so the first operation contract must exclude producer behavior that intentionally detaches. Linux process groups also lack the Windows Job Object's observed terminate-on-owner-close behavior; the first Linux contract guarantees bounded cancellation while the controller lives, not cleanup after controller crash.

Consequently:

- Windows Job Object and Linux session/process-group ownership remain viable for a closed diagnostic operation.
- The execution core remains viable only with the narrowed no-detachment operation contract and explicit Linux controller-crash limitation.
- Process ownership no longer blocks design-freeze consideration.
- The next gate is direct-MCP secret-boundary falsification; MCP selection remains later.

---

## 2. Environments Tested

### Windows

```text
OS: Microsoft Windows NT 10.0.19045.0
PowerShell: 7.6.6
Fixture runtime: PowerShell plus bounded Add-Type Win32 interop
Ownership primitive: Windows Job Object
Root creation: CreateProcessW(CREATE_SUSPENDED | CREATE_NO_WINDOW)
Containment sequence: create suspended → assign to Job → resume thread
Job limit: JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
```

Classification: **OBSERVED on this Windows environment**.

### Linux

```text
GitHub Actions target: ubuntu-latest
Runner image: ubuntu-24.04, version 20261004.327.1
OS: Ubuntu 24.04.5 LTS
Kernel: 6.17.0-1022-azure
Platform: Linux-6.17.0-1022-azure-x86_64-with-glibc2.39
Python: 3.12.3
Workflow: .github/workflows/investigate-diagnostic-process-lifetime.yml
Fixture runtime: Python 3 + Linux process groups/sessions
Workflow SHA: 3cfef027e48ce0bf52e26a2b85b756349f030be3
Branch: feature/claude-native-tool-integrations
Run: https://github.com/codeliftsleep2/Clean-CTX/actions/runs/37665077341
Artifact: diagnostic-process-lifetime-linux-evidence, ID 11502970229
Artifact SHA-256: c27d1d6a5e3de9dec1cae4d5efcb55d607d610d1d2e04f0960f23eab8ee49cc4
```

Classification: **OBSERVED on this GitHub Actions Ubuntu environment**.

No universal claim about Linux distributions, kernels, macOS, BSD, or other Unix-family systems is made.

---

## 3. Windows Ownership Primitive

The experiment used an unnamed Windows Job Object configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.

The controller:

1. created the Job;
2. configured terminate-on-final-handle-close;
3. created the root PowerShell process suspended;
4. assigned the suspended root to the Job;
5. resumed its initial thread;
6. allowed the root to create child and grandchild processes normally;
7. queried Job accounting and individual PID membership;
8. applied cooperative cancellation or `TerminateJobObject` according to the case; and
9. waited until Job accounting reported zero active processes.

Process-handle inheritance was disabled during root creation. The Job handle therefore was not intentionally inherited into the owned tree.

The fixture remained under `target/tmp/process-lifetime-falsifier/windows/` and did not touch production code.

Microsoft documents that Job Objects manage associated processes as a unit, child processes join the Job by default, `TerminateJobObject` terminates associated processes, and `KILL_ON_JOB_CLOSE` terminates associated processes when the final Job handle closes. [Microsoft Job Objects](https://learn.microsoft.com/en-US/windows/win32/procthread/job-objects)

---

## 4. Windows Results

### W1 — Normal completion

**Setup:** Root, child, and grandchild ran in normal mode. Every node wrote durable start and normal-exit markers.

**Observed behavior:** All six expected markers appeared. Job accounting reached zero active processes without cancellation.

**Classification:** **OBSERVED**.

**Authority consequence:** The Job boundary did not interfere with natural completion, and the controller could distinguish it from cancellation.

### W2 — Graceful cancellation

**Setup:** All three marked nodes polled an isolated cooperative cancellation file. The controller created the file after all start markers existed.

**Observed behavior:** Parent, child, and grandchild each wrote a graceful-request marker and a normal-exit marker. Job accounting reached zero.

**Classification:** **OBSERVED** for the cooperative fixture.

**Authority consequence:** Windows Job termination itself is not a graceful cancellation protocol. Graceful cancellation requires a separate producer-appropriate cooperative channel. The Job remains the forced-termination and ownership boundary.

**Limitation:** This does not prove Cargo, rustc, or build scripts support an equivalent graceful request.

### W3 — Forced cancellation

**Setup:** Root, child, and grandchild remained active. The controller verified every marked PID with `IsProcessInJob`, called `TerminateJobObject`, and polled Job accounting.

**Observed behavior:** Every marked PID was in the Job. Six processes were active before termination because the PowerShell fixture introduced helper processes. Job accounting reached zero after termination.

**Classification:** **OBSERVED**.

**Authority consequence:** Forced cancellation reached the complete observed owned tree, including runtime helpers. The correct invariant is Job emptiness, not an assumed process count.

### W4 — Parent exits while child remains

**Setup:** Parent spawned child and grandchild, wrote its normal-exit marker, and exited while descendants remained in linger mode.

**Observed behavior:** The parent PID was no longer alive. The marked child and grandchild remained members of the Job. Job accounting reported four active processes, including runtime helpers. `TerminateJobObject` reduced the active count to zero.

**Classification:** **OBSERVED**.

**Authority consequence:** Root-process death did not destroy descendant ownership. The ownership authority belonged to the Job, not the original root PID.

### W5 — Nested descendant ownership

**Setup:** Parent → child → grandchild used ordinary process creation with no breakaway flags.

**Observed behavior:** `IsProcessInJob` returned true for all three marked PIDs. Forced termination reduced active Job membership to zero.

**Classification:** **OBSERVED**, consistent with Microsoft's documented default child association.

**Authority consequence:** Ordinary nested descendants are included automatically.

### W6 — Breakaway boundary

**Setup:** No hostile breakaway executable was created. The Job was configured with neither `BREAKAWAY_OK` nor `SILENT_BREAKAWAY_OK`.

**Observed behavior:** Ordinary descendants did not escape.

**Documented behavior:** Microsoft states that children join the Job by default. Explicit breakaway uses `CREATE_BREAKAWAY_FROM_JOB`, and breakaway is prevented when the relevant Job does not permit it. `SILENT_BREAKAWAY_OK` would instead allow children to leave automatically and therefore must not be enabled. [AssignProcessToJobObject](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject), [Nested Jobs](https://learn.microsoft.com/windows/win32/procthread/nested-jobs)

**Classification:** Ordinary containment **OBSERVED + DOCUMENTED**; explicit hostile breakaway not executed.

**Authority consequence:** The future runner must not enable breakaway flags for the outer diagnostic Job. Preventing an intentionally hostile process from exploiting unrelated OS facilities remains outside the CargoCheck threat model.

---

## 5. Windows Spawn-Race Analysis

### Can the root create descendants before ownership is established?

Yes, if the architecture uses:

```text
start running process
→ assign running process to Job
```

That sequence contains an architectural race.

### Can the race be eliminated?

Yes for the investigated primitive. The fixture used:

```text
CreateProcessW(CREATE_SUSPENDED)
→ AssignProcessToJobObject
→ ResumeThread
```

The root's initial thread could not execute producer code before Job assignment. Ordinary descendants created after resume inherited Job membership.

Classification:

- Existence of `CREATE_SUSPENDED`: **DOCUMENTED**.
- Successful suspended-create/assign/resume behavior here: **OBSERVED**.
- Closure of the root-code spawn race for this sequence: **INFERRED from observed and documented behavior**.

### Architectural requirement

The Windows implementation must not use `std::process::Command::spawn` followed by late Job assignment if it cannot otherwise guarantee suspension. It needs a Windows-specific creation boundary that establishes the Job before execution resumes.

---

## 6. Windows Parent-Exit Behavior

Ownership remained after the root exited.

The child and grandchild remained in the same Job and were terminable through that Job. This was **OBSERVED** in W4 and is consistent with the Job Object being an OS-owned process collection rather than a parent-PID traversal.

Parent exit therefore does not hit kill condition C.

---

## 7. Windows Controller-Crash Behavior

The fixture tested the underlying crash-cleanup primitive by closing the final controller-owned Job handle while root, child, and grandchild were active.

Observed:

- the root was created with handle inheritance disabled;
- the final Job handle was closed;
- all three marked PIDs disappeared;
- zero marked processes remained alive after the close.

Classification: **OBSERVED**.

Microsoft documents that `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` terminates all associated processes when the last Job handle closes. Classification: **DOCUMENTED**.

### Constraints

- The Job handle must not be inherited or duplicated into descendants or unrelated processes.
- A deliberately leaked duplicate Job handle delays terminate-on-close.
- Normal controller shutdown should explicitly drive cancellation and observe completion rather than depend only on handle destruction.
- Abrupt process termination should close controller handles through OS process teardown, subject to no leaked duplicate handle.

The experiment did not forcibly crash the controller process itself. It tested the same final-handle-close primitive directly. An actual-controller-crash probe remains production-hardening evidence rather than an execution-core blocker, given the documented handle semantics.

---

## 8. Windows Breakaway Boundary

### Ordinary descendant behavior

Ordinary descendants joined the Job automatically and remained owned after root exit.

### Explicit escape behavior

Windows exposes explicit breakaway mechanisms. They require process-creation behavior and Job policy that permit breakaway. Silent breakaway is especially incompatible with truthful ownership.

### Required policy

- Do not set `JOB_OBJECT_LIMIT_BREAKAWAY_OK`.
- Do not set `JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK`.
- Treat inability to assign the suspended root as launch failure; do not resume unowned execution.
- Record whether the root was already in another Job and whether a valid nested Job hierarchy was established.
- Keep the outer Job handle non-inheritable.

### Remaining unknowns

- Interaction with every possible pre-existing host Job policy.
- Ordinary Cargo helper behavior when a helper attempts to create its own Job.
- Windows environments older than nested-Job support; they are outside the presently observed Windows 10 environment.

These require compatibility testing but did not falsify the Windows ownership primitive.

---

## 9. Linux Ownership Primitive

The executed fixture used:

```text
subprocess.Popen(..., start_new_session=True)
→ root becomes leader of a dedicated session/process group
→ ordinary descendants inherit PGID and SID
→ os.killpg(PGID, SIGTERM or SIGKILL)
```

It recorded PID, PPID, PGID, SID, start markers, signal markers, exit markers, known-PID liveness, and process-group existence.

For cancellation completion, the controller required both:

```text
all recorded fixture PIDs fail os.kill(pid, 0)
and
os.killpg(PGID, 0) raises ProcessLookupError
```

This is stronger than root-process exit and stronger than signal delivery alone. It is still bounded evidence: known-PID checks are fixture-specific, while group nonexistence is the candidate production-relevant observation. PID/PGID reuse remains a general race that a production design must handle within a tightly bounded operation lifecycle.

Classification: **OBSERVED** in run 37665077341.

---

## 10. Linux Results

| Case | Setup and observed behavior | Authority consequence | Classification |
|---|---|---|---|
| L1 normal completion | Parent 2092, child 2093, and grandchild 2094 shared PGID/SID 2092; all wrote normal-exit markers; known PIDs and group were absent at completion | Ordinary descendants inherit the dedicated boundary through two generations | OBSERVED |
| L2 graceful group cancellation | Parent 2095, child 2096, and grandchild 2097 shared PGID/SID 2095; all wrote SIGTERM markers; no known PID or group remained | SIGTERM reached the complete ordinary owned group in this fixture | OBSERVED |
| L3 forced group cancellation | Parent 2098, child 2099, and grandchild 2100 shared PGID/SID 2098; SIGKILL left no known PID or group | Forced group termination removed the complete ordinary tree | OBSERVED |
| L4 parent exits / descendants linger | After parent 2101 exited, child 2102 and grandchild 2103 remained live in PGID/SID 2101; the group remained signalable; SIGKILL removed both and the group | Root exit did not collapse ownership authority | OBSERVED |
| L5 nested grandchild | The L1, L2, and L3 trees all showed grandchild inheritance without explicit registration | Multi-generation ordinary ownership survives | OBSERVED |
| L6 explicit detachment | Child 2105 changed PGID/SID from 2104 to 2105 with `setsid()`; child and grandchild 2106 survived termination of PGID 2104; fixture cleanup of PGID 2105 removed both | Explicit detachment escapes the original boundary and must be excluded or separately controlled | OBSERVED |

All cases exited successfully, but the verdict follows from the recorded ownership semantics rather than the workflow exit code.

### Critical-question answers

1. Ordinary descendants inherited the root's dedicated PGID/SID: **OBSERVED**.
2. Inheritance persisted across parent, child, and grandchild: **OBSERVED**.
3. Descendants remained controllable after root exit: **OBSERVED**.
4. SIGTERM reached the complete ordinary group: **OBSERVED**.
5. SIGKILL reached the complete ordinary group: **OBSERVED**.
6. The controller established bounded completion from known-PID absence plus group nonexistence: **OBSERVED**.
7. A descendant explicitly detached with `setsid()`: **OBSERVED**.
8. The detached subtree was outside the original group's control: **OBSERVED**.
9. Preventing arbitrary hostile detachment would require a stronger mechanism such as sandboxing, tracing, or cgroups: **INFERRED**; no such mechanism is authorized by this result.
10. Explicit detachment is not expected for the first closed `CargoCheck` operation: **INFERRED**, requiring later real-toolchain compatibility evidence.
11. If the controller disappears, the process group supplies no observed automatic cleanup: **UNRESOLVED by crash injection and treated as an explicit weaker contract**.
12. Session/process-group ownership is sufficient for the first `CargoCheck` threat model only under the no-detachment contract and live-controller guarantee: **INFERRED**.

### Linux Parent-Exit Result

Parent 2101 exited while child 2102 and grandchild 2103 remained alive. Both descendants retained PGID/SID 2101, and `killpg(2101, 0)` still found the group. `killpg(2101, SIGKILL)` then terminated the remaining descendants, after which their recorded PIDs and the process group were absent.

**Authority consequence:** root exit does not end the operation or remove live-controller authority over ordinary descendants. **Classification: OBSERVED.**

### Linux Group-Completion Result

The controller did not equate successful signal delivery or root exit with completion. It polled all fixture-known PIDs with `kill(pid, 0)` and the group with `killpg(pgid, 0)`, completing only when all PID probes and the group probe returned `ProcessLookupError`.

**Authority consequence:** the controller could establish strongly enough in the bounded fixture that ordinary owned work was gone. The known-PID half is not a universal discovery mechanism, and production PGID-reuse/race handling remains a design concern. **Classification: OBSERVED for the fixture; INFERRED for a future production implementation.**

---

## 11. Linux Detachment Boundary

L6 established the distinction between:

- ordinary inherited group/session membership; from
- explicit detachment that leaves the original process group.

Before `setsid()`, child 2105 had PGID/SID 2104. Afterward it led PGID/SID 2105, and its grandchild inherited 2105. Both survived termination and verified disappearance of the original group.

The fixture then terminated detached PGID 2105 through its deliberately recorded identity. That cleanup is fixture hygiene. It does not show that a future runner can discover arbitrary detached descendants.

Therefore:

- ordinary descendant inheritance is **OBSERVED** and sufficient within scope;
- explicit detachment escape is **OBSERVED**;
- absence of detachment in supported `CargoCheck` execution is **INFERRED**, not yet demonstrated;
- hostile or arbitrary executable containment remains outside the architecture's threat model;
- the operation must fail truthfully or be excluded if a supported producer requires detachment.

---

## 12. Linux Controller-Crash Behavior

**Explicitly weaker than Windows; bounded crash injection remains UNRESOLVED.**

A process group supplies a target for live-controller group signaling. It does not by itself establish Windows-equivalent terminate-on-final-owner-handle semantics.

The experiment established only:

```text
controller alive
→ controller can signal and observe disappearance of the ordinary process group

controller gone
→ no automatic cleanup guarantee established
```

This is an **explicit product limitation** for the first design, not an architecture kill condition for a closed `CargoCheck` operation. The result does not authorize `PR_SET_PDEATHSIG`, subreapers, cgroups, systemd scopes, or pidfd-based supervision. A later product requirement for cleanup after controller crash would require a separate architectural decision and evidence.

---

## 13. Cross-Platform Semantic Contract

The smallest common contract supported by the bounded Windows and Ubuntu evidence is:

```text
One diagnostic operation owns an explicit OS process boundary.

The platform establishes the ownership boundary as part of root creation or
before the root can create producer descendants.

Ordinary descendants enter the boundary automatically.

Cancellation is not complete until:
- the owned boundary contains no live work; or
- cleanup is explicitly reported incomplete.

Graceful cancellation is an optional producer-specific attempt.
Forced boundary termination is the final ownership mechanism.

Process-lifetime facts remain independent of compiler diagnostic facts.

The supported operation does not intentionally detach or request escape from
the ownership boundary.

Controller-crash cleanup is platform-specific and must be declared separately:
Windows has the observed Job close behavior; the first Linux contract does not.
```

The semantic contract is common; the mechanisms are intentionally not symmetric.

---

## 14. Cross-Platform Asymmetries

| Difference | Status | Classification |
|---|---|---|
| Windows Job membership versus Linux process-group/session membership | Both survived bounded fixtures | Acceptable implementation asymmetry |
| Windows terminate-on-final-handle-close | Observed/documented | Acceptable stronger Windows guarantee |
| Linux owner-crash cleanup | No automatic cleanup established | Requires explicit product limitation |
| Windows explicit breakaway flags | Documented | Acceptable with outer-Job policy |
| Linux explicit `setsid()` detachment | Observed escape | Requires narrowed operation contract |
| Windows graceful termination | Separate cooperative channel | Acceptable asymmetry |
| Linux SIGTERM group delivery | Complete ordinary fixture group received SIGTERM | Acceptable platform mechanism |
| Completion observation | Windows Job active count versus Linux group nonexistence plus bounded checks | Acceptable, subject to production race analysis |

---

## 15. Architecture Kill Conditions

### Windows

| Kill condition | Tested? | Observed? |
|---|---:|---:|
| A. Ordinary descendants escape | Yes, bounded ordinary tree | No |
| B. Unavoidable spawn/assignment race | Yes, suspended creation path | No |
| C. Parent exit destroys ownership | Yes | No |
| D. Cannot terminate complete owned tree | Yes | No |
| E. Cleanup completion cannot be observed | Yes, Job active count | No |
| F. Requires unrestricted interception/sandbox | Yes within ordinary-operation threat model | No |
| G. Requires general-purpose process manager | Yes within fixture scope | No |

### Linux

| Kill condition | Tested? | Observed? |
|---|---:|---:|
| A. Ordinary descendants escape group | Yes, bounded ordinary tree | No |
| B. Parent exit makes descendants uncontrollable | Yes | No |
| C. Group cancellation fails | Yes, SIGTERM and SIGKILL | No |
| D. Cleanup completion cannot be established | Yes, known PIDs plus group nonexistence | No within fixture scope |
| E. Reliable ordinary ownership requires a general sandbox | Yes within stated threat model | No |
| F. Supported diagnostic tooling normally detaches | No real Cargo execution permitted | UNRESOLVED; contract excludes detachment pending compatibility evidence |

L6 deliberately detached and escaped. That is a confirmed boundary, not kill condition A, because it was explicit escape behavior outside ordinary inheritance and outside the first operation contract.

---

## 16. Required Architectural Modifications

Supported by Windows evidence:

1. Create the Windows root suspended.
2. Establish Job ownership before resuming producer code.
3. Configure `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
4. Do not allow ordinary or silent breakaway on the outer Job.
5. Do not inherit or leak the Job handle.
6. Treat assignment failure as launch failure and never resume an unowned root.
7. Keep graceful cancellation separate from forced Job termination.
8. Wait for observable Job emptiness before reporting cancellation complete.
9. Represent cleanup failure explicitly rather than reporting the operation ended.
10. Preserve root exit status separately from owned-tree cleanup status.

Required by the Ubuntu evidence:

11. Create the Linux root as leader of a dedicated session/process group.
12. Target graceful and forced signals at the group, never only the root PID.
13. Treat root exit as process outcome, not owned-operation completion.
14. Do not report cancellation complete until the group is no longer observable; report incomplete cleanup explicitly on timeout or observation failure.
15. Restrict supported operations to producers that do not intentionally call `setsid()`, `setpgid()`, double-fork into another session, or otherwise detach.
16. State that the first Linux design does not guarantee descendant cleanup if the controller itself crashes.
17. Keep controller-crash strengthening as a separate decision; do not silently introduce sandboxing, cgroups, tracing, or a general supervisor.

---

## 17. Remaining Unknowns

### Blocks execution-core design freeze

- Production-level Linux root-creation race analysis: `start_new_session=True` worked in the Python fixture, but the Rust implementation mechanism is not selected or authorized here.
- Decision acceptance of the explicit Linux controller-crash limitation.
- Decision acceptance of the no-detachment operation contract.

### Blocks production hardening only

- Actual Windows controller-crash injection rather than direct final-handle close.
- Compatibility with host processes already in restrictive Jobs.
- Cargo/rustc/helper compatibility with the no-breakaway outer Job.
- Cargo/rustc/helper compatibility with the Linux no-detachment contract.
- High-concurrency Job completion notifications.
- Linux process-group observation and PID/PGID reuse hardening under concurrency.
- Grace-period policy and global timeout values.

### Later concern

- macOS and BSD ownership semantics.
- Containers and remote execution.
- Toolchains beyond the supported operation contract.
- Hostile-process sandboxing, which remains a non-goal.

---

## 18. Effect on Diagnostic Architecture

**Execution core remains viable only with a narrowed operation contract.**

The bounded evidence supports a cross-platform semantic ownership contract implemented by platform-specific mechanisms. It does not support arbitrary commands, hostile children, detached background services, `CargoRun`, or `CargoTest`. The first candidate remains closed `CargoCheck`, subject to later compatibility evidence that its supported producer tree does not intentionally detach.

Process ownership no longer falsifies the execution core or blocks consideration of the next architecture gate. This report does not authorize production implementation and does not resolve Linux controller-crash hardening.

---

## 19. Updated Falsification Chain

```text
semantic compilation       → survived bounded synthetic falsification
executable resolution      → survived with absolute-path constraint
parser truthfulness        → survived bounded synthetic falsification
Windows process ownership  → survived bounded local falsification
Linux process ownership    → survived bounded GitHub Actions Ubuntu falsification with no-detachment and live-controller limitations
```

MCP selection and secrecy remain unchanged and unstarted.

---

## 20. Next Gate

The next highest-leverage experiment is the **direct-MCP secret-boundary falsifier**, before selection reliability. This report does not execute or authorize that experiment automatically.

Reason: selection reliability can demote MCP from primary UX while leaving the execution core valuable through CLI or another adapter. Failure of the claimed pre-Claude sanitization boundary removes a major security advantage used to justify direct MCP execution and materially changes the architecture's comparative score.

Only after the secrecy boundary survives should MCP selection reliability decide whether MCP is the preferred interface or merely an available adapter.

The Linux evidence does not change that ordering. It clears the process-ownership blocker while adding explicit limitations that the secrecy experiment does not depend on.

---

## 21. Ubuntu Evidence Preservation

The investigative fixture and workflow were committed in `2c5817d4`. GitHub does not expose a newly introduced `workflow_dispatch` workflow by filename until it exists on the default branch, so dispatch returned HTTP 404 without starting a run. Commit `3cfef027` added a branch-and-path-scoped push trigger; pushing that commit executed the fixture without changing the default branch.

Preserved remote evidence:

```text
Run URL: https://github.com/codeliftsleep2/Clean-CTX/actions/runs/37665077341
Job URL: https://github.com/codeliftsleep2/Clean-CTX/actions/runs/37665077341/job/112942114102
Workflow SHA: 3cfef027e48ce0bf52e26a2b85b756349f030be3
Branch: feature/claude-native-tool-integrations
Event: push
Conclusion: success
Artifact: diagnostic-process-lifetime-linux-evidence
Artifact ID: 11502970229
Artifact download URL: https://github.com/codeliftsleep2/Clean-CTX/actions/runs/37665077341/artifacts/11502970229
Artifact SHA-256: c27d1d6a5e3de9dec1cae4d5efcb55d607d610d1d2e04f0960f23eab8ee49cc4
Artifact retention: 7 days
```

The downloaded artifact is also present locally under ignored `target/tmp/process-lifetime-falsifier/github-run-37665077341/` for report transcription. It is evidence, not a tracked test or CI gate.
