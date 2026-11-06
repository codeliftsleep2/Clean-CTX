# AF-PERF-002 A1: default publication copy boundaries

Agent: Agent MaxHeadRoom
Status: characterization executed GREEN; A1 ownership copies classified below.
Handler-only clone-elision correction CLOSED / FIX / PROTECTED after genuine RED,
exact semantic GREEN, focused/mutation and final 65-test composed GREEN.
See AF_PERF_002_A1_HANDLER_CLONE_REPAIR.md.
Original characterization retained below; later approved handler-only correction
is tracked separately. No representation or ownership-model redesign.

## Owner scope and baseline

Owner accepts exporter/transport/symbol tracing without repair. Stats formatting,
replay, A3, NgRx, hierarchy and scrubber protected/closed decisions remain intact.
A1 is a separate characterization, not a reopening of those repairs. The former
zero published_semantics accessor count describes only its measured interval.
It cannot establish absence of copies elsewhere or outside that interval.

All new source observations are cfg(test); no existing clone, move, admission
predicate, lock, currentness test or persistence/publication order is changed.
Existing receipts stay frozen. Their exact source hashes precede this test-only
instrumentation and are not misrepresented as verification of the new bytes.

## Actual default reachability and independent operations

1. workspace_query -> hydration publication -> compiled artifact ->
   published_semantics -> validated publication ticket -> index admission.
   C# keeps the artifact in Arc for later source-bound generation projection.
   The accessor clones edges/dependencies/hash to supply a separate publication
   owner; a TypeScript hydration call currently also uses this accessor even
   though its artifact retention differs. Query entities has its own C# retained
   artifact accessor branch and moves non-C# into_parts results instead.
2. Dirty C# query -> project_csharp_exact_calls_for_generation -> current artifacts
   -> staged Projection per owner -> workspace index replacement -> generation
   coverage commit -> live IR load and semantic snapshot ownership per projection.
   IR is cloned from the retained artifact; selected non-exact base edges are
   cloned before new generation facts are projected. Each staged edge/dependency
   vector is cloned for the index, then its original is moved into a snapshot.
3. compress_code_context successful IR -> candidate/current_parts -> hierarchy and
   economics -> durable candidate clone -> authority-ordered durable publication
   -> live IR clone -> non-C# index clone -> retained snapshot clone -> response.
   C# current_parts has a separate stamp/dependency-currentness boundary and its
   own generation IR clone; it cannot borrow a result from reconciliation merely
   because both consume retained artifacts.
4. Required durable baseline -> persistence_ir::baseline clones IR for physical
   owner identity normalization -> binary encoding -> SQLite transactional save.
   This is separate from the handler's durable_candidate clone. Persistence off
   and auto-save off alter which boundaries are reached, not their authority.

These are source-traced production paths, not claims of executed integration for
this new packet. The packet invokes actual registered public tools to measure
these paths. No standalone helper output substitutes for public reachability.

## Ownership and failure requirements

Hydration captures expected dependencies/hash/authority before ticket commit.
Validation rereads current source and dependencies before admitting the candidate;
failed publication must not satisfy clean acquisition. These captures are not
silently considered redundant. Existing failure control deliberately injects
publication failure and checks the production acquisition result.

Compression validates typed authority and persists the canonical IR/semantic
baseline before live IR, index/snapshot and presentation cache updates. Existing
failed_compress_baseline_commit_publishes_no_candidate_live_state injects a real
SQLite save failure after an old owner and peer are published. It verifies exact
prior IR/hash/version, peer IR, fidelity, semantic edges, index registrations and
cached presentation survive; restore still reads the prior durable baseline.
New observations isolate the failed request before the existing state assertions.
Copies before a failure may be candidate isolation work; later publication counts
must not be conflated with candidate construction. No rollback protocol is changed.

C# reconciliation constructs all fallible projections before the index write;
then it replaces index owners and commits coverage under that lock, followed by
separate IR/snapshot updates. This does not establish a global transaction spanning
all those owners/locks. This first packet does not inject every reconciliation
race/failure or prove whole-operation atomicity. Those remain explicit gaps;
removing staged ownership cannot be justified by successful cases alone.

## Measurement definitions

Each named clone site records calls, edge/dependency objects, dependency path/hash
string lengths and explicit cloned source-hash lengths. Endpoint name/file lengths
are counted as copied owned string payload. Static domain/type/layer references
are not counted as cloned strings. Canonical endpoint occurrences are reported
separately; their deeper proof/path payload is not included in the endpoint-byte
subtotal. IR sites report cloned instruction objects and file-ID string lengths,
not instruction operand bytes, capacities or heap allocations.

These are executed logical-work observations once the tests run. They are NOT
allocator events, allocator bytes, RSS, capacity, memcpy traffic or total graph
memory. Endpoint bytes are a lower bound on deep edge-clone payload. Dependency
string lengths exactly cover the two owned fields at the named dependency-copy
sites, excluding container/allocator overhead. Output/index-derived structures,
retained authority clones, resolution units, source snapshots, internal index
endpoint/key materialization and persistence edge conversion are not fully counted
by this packet. Do not add these subtotals and label them complete allocation cost.

Separate sink counters count calls and moved edge/dependency objects admitted to
workspace index and generation/retained snapshots. They measure dependent ownership
publications, not new clones. Public elapsed time includes all enclosing operations
and test-observation/capture/serialization overhead; no visit-to-latency attribution.
Synchronous/thread-local observations exclude unattributed compilation workers.

## Authored fixture dimensions and controls

C# public query: 2/4/8/16 admitted owners independently; 1/2/4/8/16/64/256 calls
at two owners; 16-owner/16-call joint case. Cold hydration, explicitly dirtied
retained reconciliation and unchanged warm repeats are recorded separately. The
admitted file count is asserted. The existing exact C# identity/provenance oracle
must pass, not only builtin Run/Save output. Dirty phase requires the actual
reconciliation-index site per admitted owner and zero full semantic parses.
Warm phase requires zero work at the selected clone sites and unchanged ordered
serialized Caller snapshot; that is not a universal zero-clone claim.

Successful public TypeScript compression: independent 1/2/4/8/16/64/256 method
population, requested name lengths 8/16/64/256, repeated requests 1/2/4/8/16,
and joint 16-method/64-name-byte/four-request fixture. Both explicit persistence-off
and persistence-on/auto-save-on variants are exercised. Actual source bytes and
reached clone/sink counts are printed. Each request must succeed through IR;
retained source hash and nonempty semantic/live IR ownership are required. These
are copy-boundary controls, not new exhaustive TypeScript semantic certification.
Existing exact semantic and lifecycle protections remain independent.

No future repair budget or desired representation is imposed. Counters document
current copies even if necessary. Original characterization evidence is retained
separately, not retroactively labeled optimization RED.

## User-owned execution packet

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-a1-publication-work.ps1
```

Expected six tracked executions: two new public-copy tests, one existing durable
failure test, one existing C# acquisition failure test and two retained-artifact
reuse controls. Then all-target/all-feature warning-fatal Clippy, format/size and
strict UTF-8 gates; exact source hashes/logs remain under target/af-perf-002.
All are UNRUN in this return. Agent performed only bounded text/line/encoding and
PowerShell syntax review, with explicit UTF-8 reads/writes. No agent runtime,
formatter, Cargo command, server or profiler was started.

Return after execution: actual per-site/per-axis work, distinct sink publications,
copy-versus-move and failure observations, source identity and smallest defensible
ACCEPT/MONITOR/FIX-proposal disposition. No copy elision, shared storage, mutation
in place, cache, transaction redesign or production correction is approved here.
If deeper byte/allocator attribution is needed for a consequential site, scope that
measurement explicitly rather than claiming these counters already provide it.

Syntax census is still 9763 original / 13 scoped / 9750 unresolved plus 96 callback
supplements. This production-operation progress does not dispose syntax records.
Broader discovery remains independent; A1 does not claim whole-system coverage.


## Executed return: actual production operation evidence

Agent MaxHeadRoom inspected **a1-publication-work-20261106-074024**. Six tracked
executions pass, zero failed/ignored: new public characterization two tests 134.27s,
durable publication failure control 4.54s, acquisition failure control 0.64s and
two retained-artifact controls 1.39s/0.75s. Warning-fatal all-target/all-feature
Clippy passes (reported 1m05s); format logs empty; size passes 36 active files;
strict encoding passes 2131 text files. **All 1313 recorded source hashes match**
at inspection. inspection.json retains 63 operation rows plus the isolated failure
interval. No production/test changes follow that execution in this return.
This is focused characterization GREEN, not optimization RED or whole-workspace/
binary/field certification. No agent runtime was started.

Thirty rows cover compression with persistence off/on; eleven fixtures each have
cold-query, dirty-query and warm-query rows. All named copy/publish site data is
retained, not merely elapsed times or the legacy accessor counter.

### Successful compression: independent dimensions

N is instruction objects copied at EACH handler IR clone site. E is edge objects
copied at EACH handler index/snapshot site. Both explicit persistence variants
have identical N/E and named endpoint-byte counts. Persistence adds one separate
normalization IR clone; it does not remove either handler IR clone.

| Methods | Requested name bytes | Requests | N per IR clone site | E per edge clone site | Endpoint bytes per edge clone site | Public us off / on |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 8 | 1 | 9 | 5 | 685 | 1797070 / 956357 |
| 2 | 8 | 1 | 13 | 7 | 974 | 962571 / 1015649 |
| 4 | 8 | 1 | 21 | 11 | 1552 | 965904 / 974554 |
| 8 | 8 | 1 | 37 | 19 | 2708 | 1174592 / 1097216 |
| 16 | 8 | 1 | 69 | 35 | 5038 | 1116561 / 1271567 |
| 64 | 8 | 1 | 261 | 131 | 19054 | 2028585 / 2387817 |
| 256 | 8 | 1 | 1029 | 515 | 75586 | 6110833 / 7609209 |
| 1 | 16 | 1 | 9 | 5 | 709 | 854065 / 1005604 |
| 1 | 64 | 1 | 9 | 5 | 853 | 837501 / 1149424 |
| 1 | 256 | 1 | 9 | 5 | 1429 | 858628 / 896696 |
| 1 | 8 | 2 | 18 | 10 | 1370 | 1685460 / 2004273 |
| 1 | 8 | 4 | 36 | 20 | 2740 | 3422122 / 3684503 |
| 1 | 8 | 8 | 72 | 40 | 5480 | 6937022 / 7672011 |
| 1 | 8 | 16 | 144 | 80 | 10960 | 13729787 / 14973485 |
| 16 | 64 | 4 | 276 | 140 | 30904 | 4515826 / 5176989 |

At 256 methods, the off path copies 1029 instructions twice (2058 at these sites),
while the on path copies them three times (3087). Each publishes 515 edges into
the workspace index and retains a separate 515-edge snapshot: 1030 measured edge
copies, with 151172 named endpoint string bytes plus 128 explicit source-hash bytes.
The accessor remains [0,0,0] in these rows. These are not allocator-byte totals.
At fixed one method, increasing names from 8 to 256 leaves nine instructions/five
edges per site but grows endpoint payload 685 -> 1429 bytes. Byte density and item
population are independent. Repeated requests multiply each copy site linearly;
Q16 copies 144 instruction objects per site and 80 edges per edge site, publishing
16 index owners and 16 retained snapshots over time. These are replacements over
requests, not 16 simultaneously retained new owners.

Within these controlled fixtures N=Q*(4*M+5), E=Q*(2*M+3). These formulas describe
the generated source, not universal compiler bounds. All canonical endpoint
subcounts are zero here and all TS dependency vectors are empty: deep retained
proof bytes and positive TypeScript dependency density remain uncharacterized.
Public timings contain compilation/rendering/tokenization/IO and test capture;
they neither isolate clone cost nor prove that removing one clone improves latency.

### C# acquisition and retained generation publication

Cold hydration reaches the accessor: F2/C1 clones 7 edges/905 endpoint bytes;
F16/C1 clones 21 edges/2637 bytes; F2/C256 clones 262 edges/30740 bytes. Each owner
has one accessor call and validation dependency-vector clone (empty in this cold
fixture), then an index publication. Initial accessor calls are real work that the
earlier dirty-only interval omitted.

Dirty reconciliation has ZERO accessor calls while the following copies execute.
Endpoint/dependency bytes below belong only to reconciliation-index clones; base
edge cloning and IR cloning are distinct. Index and generation-snapshot sinks each
receive the stated edges/dependencies; the snapshot takes staged originals by move.

| Owners F | Calls C | Base edges cloned | IR instructions cloned | Index edges cloned | Dependencies cloned | Index endpoint / dependency bytes | Dirty public us |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 1 | 5 | 10 | 8 | 1 | 1060 / 119 | 98768 |
| 4 | 1 | 7 | 14 | 10 | 3 | 1304 / 357 | 186691 |
| 8 | 1 | 11 | 22 | 14 | 7 | 1792 / 833 | 331172 |
| 16 | 1 | 19 | 38 | 22 | 15 | 2792 / 1791 | 647247 |
| 2 | 2 | 6 | 11 | 10 | 1 | 1332 / 119 | 93829 |
| 2 | 4 | 8 | 13 | 14 | 1 | 1876 / 119 | 89800 |
| 2 | 8 | 12 | 17 | 22 | 1 | 2964 / 119 | 85116 |
| 2 | 16 | 20 | 25 | 38 | 1 | 5140 / 119 | 96874 |
| 2 | 64 | 68 | 73 | 134 | 1 | 18196 / 119 | 103979 |
| 2 | 256 | 260 | 265 | 518 | 1 | 70420 / 119 | 219536 |
| 16 | 16 | 34 | 53 | 52 | 15 | 6872 / 1791 | 782879 |

All eleven dirty rows satisfy this fixture's base-edge F+C+2, IR 2*F+C+5,
index-edge F+2*C+4 and dependency F-1 counts. This is not evidence that A1 copies
necessarily multiply F*C. Each owner is projected once; the invocation population
is concentrated in Caller. The existing A2 traversal measurements remain separate.
C# dependencies here represent the retained Target obligation for other owners;
peer-density and arbitrary framework dependency populations are not exhausted.

At F2/C256, 260 base edge records are cloned before exact projection; 518 projected
edges are cloned into the index while 518 staged originals move into snapshots.
The measured index copy alone accounts for 70420 endpoint string bytes plus 119
dependency bytes and 128 explicit source-hash bytes. It coexists with 265 copied
IR instructions. Zero accessor calls clearly do not imply zero publication copies.

All eleven warm repeats have zero work at the selected copy sites AND zero measured
index/snapshot publications. Warm times range as recorded in inspection.json;
this does not certify zero unrelated response/internal getter copies. Exact C#
identity/provenance oracles and unchanged ordered Caller snapshots pass. Dirty
repeats require zero full semantic parses; retained-artifact reuse controls pass.

### Failure atomicity and ownership evidence

The real failed durable save interval executes handler-durable-candidate and
persistence-normalization once each, five instructions each. It reaches NONE of
the measured live-IR, handler-index or handler-snapshot clone sites and has no
publication sink entries. The existing control proves the prior owner/peer IR,
hash/version/fidelity/edges/registrations/cache and prior durable restore remain
correct. The separate acquisition failure control remains incomplete rather than
claiming successful C# acquisition. These are executed finite failure facts, not
whole C# multi-lock atomicity or every concurrent publication schedule.

## Smallest defensible dispositions

| Work unit | Evidence and disposition |
| --- | --- |
| Artifact accessor supplying independently retained publication | ACCEPT for the observed ownership boundary. Distinct artifact/index owners need owned state under current representation; no unused clone result demonstrated. Nonretained hydration alternatives remain unmeasured. |
| Index and retained snapshot edge/dependency ownership | ACCEPT current distinct-owner requirement; MONITOR aggregate payload/positive proof density. A cheap move/share rewrite is not established by these counts. No cross-owner storage redesign proposed. |
| Generation IR clone before replacing realtime instructions | MONITOR. Measured real payload, but immutable retained authority and projected live ownership differ. In-place mutation/reuse is not authorized or justified here. |
| Persistence normalization clone | ACCEPT current identity-normalization ownership boundary. No proposal to change durable physical-owner semantics or mutate live IR for encoding. |
| Handler durable_candidate clone | Measured unnecessary copy candidate: FIX PROPOSED / owner review, not implemented/protected. It executes even when persistence is off, and is copied again before durable use when on. Narrow elimination proof below. |

### Proposed handler-only correction; no production edit yet

In successful core/compress.rs only, remove `let durable_candidate = ir.clone()`
and pass existing `&ir` at the same persist_read_baseline_with_authority call.
Do not skip, gate or move that call: authority checks, lock acquisition, error paths,
physical-owner normalization and durable-before-live ordering stay unchanged.
No new helper, conditional, cache, shared state or API is needed.

Source equivalence proof: CompiledIR consists of file_id, instructions and version.
Between the old clone and the persistence call, only ir.file_id is changed to the
presentation alias. Rendering/economics borrow it immutably; instructions/version
are unchanged. The persistence helper either returns after the SAME authority/
checkpoint/store checks, or invokes persistence_ir::baseline(compiled, owner_path).
That existing helper clones the supplied IR and overwrites file_id unconditionally
with the SAME durable owner path BEFORE encoding. Subsequent persistence observes
compiled.version, not the incoming file_id. Compatibility identity derives from
unchanged source/config/owner arguments. Thus the normalized durable IR is identical
whether input file_id is its original form or the alias. Instruction/version/source
identity and all currentness observations remain the same. The main handler already
borrows ir in this closure for live loading, so a separate snapshot is not required
merely to provide an immutable reference at the earlier persistence call.

Expected construction-inclusive effect at measured sites: persistence off 2*N -> N
instruction-object copies; persistence on 3*N -> 2*N. This is a finite local copy
elimination, not incremental compilation or ownership redesign. It removes the
unconsumed clone when persistence is off and the redundant input copy when on.
No elapsed-time or allocator-byte benefit is claimed before/after; the existing
1029/144-object growth establishes the relevant work, not enterprise pressure.
The real persistence-failure test must continue preserving prior live/durable state.

Before any authorized implementation: author a genuine zero-redundant-candidate
production-path budget, execute original RED, then minimal wiring and unchanged
GREEN; retain original A1 characterization separately. Require exact durable IR/
binary/owner/version and public responses for differing alias/physical-owner cases,
persistence off/on, auto-save off, Edit-required checkpoint and failed save. Retain
source/currentness failures and protected C1 economics behavior without reworking
it. Add finite clone-site enforcement, isolated real restored-clone mutation RED/
restored GREEN and affected gates. If a helper consumer reads the incoming file_id
before normalization or any observation/order changes are discovered, stop and
return that evidence. This is a proposal under characterization-only authorization.

## Remaining limits and closure boundary

No production correction was applied, no new optimization budget authored, no
commit/push or further runtime command requested in this return. A1 is now measured
on these consequential default paths, with allocator/RSS, IR operand/proof bytes,
index internal materialization, positive TS dependency density, concurrent schedules
and broader publication callers still explicit gaps. No automatic requirement to
remove every clone follows from observation. A4 remains an independent obligation.
All protected/closed dispositions and syntax accounting remain unchanged.


## Owner implementation disposition

Owner approves only removal of the redundant handler durable_candidate clone and
passing existing &ir at the unchanged unconditional persistence call. Original A1
characterization remains frozen GREEN; new deterministic RED budget and exact
public/durable identity controls are authored UNRUN before production editing.
See AF_PERF_002_A1_HANDLER_CLONE_REPAIR.md for the exact Before command and scope.
Other publication-copy ACCEPT/MONITOR decisions remain unchanged.


## Approved handler-only repair completion

The separate approved handler durable_candidate correction is CLOSED / FIX /
PROTECTED. Final af-perf-001-20261106-102529 passes 65 tracked tests with 230
matching hashes; original budget/normalizer bytes remain unchanged, and the
production diff is the approved clone removal plus borrowed &ir argument only.
See AF_PERF_002_A1_HANDLER_CLONE_REPAIR.md for exact RED/GREEN/mutation/source
identity. Original characterization remains frozen and all other A1 ACCEPT/MONITOR
classifications and coverage limits remain unchanged. No whole-A1 closure inferred.
