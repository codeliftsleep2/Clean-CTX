# OPS-R1 disposition 11: concrete scoped-acquisition patch packet

Agent: **Agent MaxHeadRoom**. Date: 2026-11-20.
Standing: Candidate B is owner-selected. The patches below are prepared, **not
applied, compiled, executed, admitted or delivered**. Disposition 09 remains the
architectural contract; Disposition 11 authorizes preparation only.

## Exact packet and source identity

Baseline HEAD: `0dd8d7503af3116241ad7c1e23c293b1d5849cc1`.
Baseline diagnostic tree: `393f7d7b0ba31dbe0f78cceb43539dccdb120598a154af8c7a7c415b33a661c1`.
The old binary `34d2a9ebd67ce949fd27cd373ba013996ba7eb5a9e80573ff5428b995cf89222`
belongs to that earlier diagnostic tree. It is not a binary for these patches.

Authoritative proposal identities and per-file before/after hashes:
`verification/ops-r1/proposals/shared-acquisition/proposal-index.json`.

Apply order, if separately approved:

1. `observations-and-fitness.patch`: operation-local counters and one public-path
   tracked fitness witness. Acquisition still runs independently per owner.
2. **Stop for actual RED evidence and owner review.**
3. `shared-acquisition.patch`: receipt capture, cache attribution, group ownership,
   recipient qualification and immutable program reuse.
4. `proof-controls.patch`: concern-based native proof and isolation regressions.

| Patch | SHA-256 | Candidate source-tree SHA-256 |
|---|---|---|
| observations-and-fitness.patch | `e5f1ba75dda8ade21e6316e3008a8c70791e591b5a062796cd70d9102b08ef3a` | `73bba237a6f178dcf424cb449f033335044c5e835cf56c0f94363d2213cc3a05` |
| shared-acquisition.patch | `4f30d7e91935e9863081856b885ebd27683cfa28c50ed49932cced606674efa7` | `f248e13ff89dbd20e663e28a685d79e21709ffb76f8019517647ca9911d1dea5` |
| proof-controls.patch | `fe7f27e48d3e0f1974deed66b4105b93e3e604a2370d4c5ca4edec365e390a6a` | `a62cd801bb5c8808f1ceaa96c1cd1d0ef21e92475d06faaf2c2f5307a7466cd6` |

No patch has been applied to the working source. Applicability checks used a
disposable index/object directory under target; the real index and source files
were not changed by those checks. All proposed code files are below 615 lines.

## Source-file and API manifest

The index lists exact files and hashes for each sequential stage. The source/API
responsibilities are:

| Boundary | Proposed changes |
|---|---|
| `semantic_compilation/executor.rs` | Fresh context per compile invocation; same Rayon pool, indexed job order and catch-unwind boundary. |
| `semantic_compilation/compile.rs` | Internal operation-aware entry; standalone entry and request structure remain available. |
| `typescript_resolution/capture.rs` | Optional operation input; private owner Resolver and exact seed; no serialized authority-field changes. |
| `typescript_resolution/acquisition_inputs.rs` | Internal source/input receipts and nested consumption scopes; no serde representation or public proof authority. |
| `typescript_resolution/host.rs` | Retain source receipts, including failed reads; supplied versus disk provenance; preserve the Witness enum and truth predicates. |
| `typescript_resolution/resolver.rs` | Unit/Project values retain receipts; cache hits attribute dependencies; inherited project context retains its selection receipt. |
| `typescript_resolution/program.rs` | Scope at actual demand; seal accounted success; local failure memoization; qualified immutable reuse. |
| `typescript_resolution/shared_acquisition.rs` | Operation slots, immutable program/input proof and role-specific qualification. Work counters are test-only. |
| `ambient_modules.rs`, `ambient_groups.rs` | Consume the immutable program map without introducing a full input copy for proof construction. |
| `src/tests/mcp/ops_r1_acquisition_fitness.rs` | Same tracked RED/GREEN public query and work assertion. |
| `src/tests/ir/ops_r1_shared_acquisition_*.rs` | Shared support, proof controls and isolation controls; normal tracked module registration. |

No publication, persistence, compatibility identity, producer-generation or
protected-contract implementation file is modified. `publication.rs` remains
byte-identical to the diagnostic baseline. Owner-specific global/augmentation
analysis can still iterate or clone units as required by its existing result
contracts; this patch does not claim constant total owner work.

## Receipt attribution and qualification implementation

InputLog retains the existing cumulative Witness values for owner validation.
Additional internal receipts describe observations, immutable source bytes and
dependency references. They are accumulated in nested scopes at the original
read/parse/project-selection points; finishing a scope references its receipt
from its parent rather than copying a complete authority list.

Host read hits, Unit hits, Project hits and inherited context attribute their
recorded dependencies. Negative cached reads also retain proof. Empty/unrecorded
receipts and inputs acquired before recording was enabled are explicitly
unaccounted and cannot seal reusable success. A nonempty owner witness list is
never treated as sufficient proof.

The program scope attributes the original project-selection receipt without a
second project selection. Existing acquisition produces an immutable ordered unit
map. Sealing visits the recorded dependency nodes, not the source tree: no new
filesystem inventory, program construction or second unit-map traversal is used
to discover its input proof. Complete source receipts and accounted status follow
the existing per-unit read path. Only accounted successful acquisitions enter a
shared slot; justified missing-module evidence does not become a failure or a
claim of universal completeness.

The provisional key retains the selected config, origin-bearing options,
ownership/reference selection, compiler version, roots/aliases, request directory,
source kind and capture demand policy. Paths alone are insufficient. Source
requests whose physical seed route is not established by the private Host stay
independent. Different request directories stay independent initially.

Qualification checks only the recipient's consumed cache/seed overlaps against
the immutable source map, then validates applicable recorded witnesses. It does
not rescan all project units or copy the whole source map into each owner.
Recipient bindings are small override maps over the immutable acquired map.
Required proof-node/currentness traversal is measured separately from acquisition.

For a leader seed used by a sibling, qualification obtains real disk-backed
dependency evidence and exact byte equality. For the recipient's own supplied
source, exact bytes remain supplied authority; it does not inherit a foreign disk
Content witness as authority for its seed. Unrelated private cached values are
neither overwritten nor exported. Attached Unit hits replay that unit's source
receipt, not an entire other program's proof.

The recipient retains its own declaration/reference/demand state and later
program groups/augmentations. A shared proof carries only acquisition-owned
evidence. Existing capture-currentness and artifact identity handling remain in
the owner path. Materialized witness order/multiplicity is not certified by static
inspection; the selected retained-authority/persistence and refusal tests remain
required evidence before acceptance.

## Concurrency, ownership and failure isolation

One context owns one small slot map per executor invocation. No context is stored
on McpState, the executor, a persistent artifact or a global variable. Successful
proofs contain immutable units/inputs, not a Resolver or publication ticket.
Receipt references point to earlier consumed input records; they do not reference
the operation context or mutable owner, preventing an operation/proof ownership
cycle in the proposed representation.

The map lock only obtains a per-group slot. An empty group's acquisition owner
holds that slot through the first acquisition. Once a proof exists, the slot lock
is released before qualification; qualified owners compile concurrently. This is
per-group acquisition coordination, not serial owner semantic compilation or a
new worker-count policy. In-flight waiting cost remains an implementation cost.

Only top-level private acquisitions participate in slot coordination. Nested
acquisitions stay independent so two group leaders cannot form a lock cycle.
This is reuse ineligibility, not a new depth limit or semantic refusal. Existing
same-project recursive acquisition refuses before acquiring a slot.

Failed/partial acquisition leaves no shared failure/success entry. Poison recovery
does not make another owner's failure sticky. Existing owner-local failure
memoization remains unchanged. A stale existing success causes independent
acquisition; it is not silently rewritten as a current snapshot.

| Event | Preserved response |
|---|---|
| Different seed bytes, config, roots/route, package or lookup basis | Independent acquisition; no private cache overwrite. |
| Unaccounted cache provenance | No shared success; original private behavior remains. |
| Failure after some units were consumed | No proof export; another owner can acquire independently. |
| Source/config/package changes | Existing proof/currentness checks disqualify stale reuse/publication. |
| Owner retirement/replacement | Existing source checks and publication tickets remain controlling. |
| Cancellation/retry | No new behavior or cross-operation failure state. |
| Non-TS job / explicit standalone call | Existing compilation path; no TS acquisition reuse. |

## Architectural work accounting

The tracked witness measures attempted/completed complete acquisitions and actual
program-unit visits independently from owner compilation/publication. Separate
qualification observations cover attempts, successful reuse, sealing witness
checks, recipient witness checks and proof-node visits. These counters are not
production time/memory budgets.

Project/config reads and route checks remain part of the existing private
selection/read paths. Native work instrumentation and existing observations count
reads, currentness and acquisition separately; counters do not establish constant
qualification cost. Existing directory-witness validation still re-enumerates
directories, and content validation still reads/hashes bytes. This can retain
O(owners * witnesses) work. No currentness cache or deduplication waiver is added.

Proof sealing adds one validation pass for an eligible first acquisition; each
recipient adds qualification/proof handling before its existing final checks.
Those are real costs to include in post-repair assessment. Fewer acquisition
visits alone do not establish overall benefit or explain the field hang.

## Tracked tests and cross-consumer selections

Exact names, source paths, package `clean-ctx`, lib target, all-feature mode and
case names are in `test-selections.json` next to the patches. Source/module mapping
is static; executable registration/nonempty selection remains unverified.

The new tests cover pre-cached Host/Unit/Project proof, owner-only exclusion,
supplied/disk authority, divergent overlays and private caches, config/roots/routes,
source/package mutation, failed/partial acquisition, unaccounted provenance,
request-directory automatic types, inherited project context, genuinely distinct
projects/operations, concurrent owners, recursive/nested acquisition, standalone
capture and justified missing-module behavior. The same public fitness test
asserts empty lower-bound response, four owner compilations/publications and one
complete four-unit project acquisition.

Reused selections cover positive resolver registration, bootstrap dependency
mutation, explicit tools/ordinary hydration recovery, TS contributor persistence
and prepublication refusal, retirement/reappearance, non-TS worker ordering/error
isolation, batch supersession and C# reconciliation. No resolver-only test is
represented as complete cross-consumer protection. Full repository gates remain
CI-owned; this packet does not add mandatory CI membership or a new golden runner.

## Historical RED procedure: planned, not authorized to execute

After separate approval, verify the original diagnostic source identity and apply
only `observations-and-fitness.patch`. The exact first-stage candidate digest is
in the index. The expected defective behavior remains four independent
acquisitions and sixteen visits; earlier owner-1/4/16 receipts stay historical
successful observations, not this test's executed RED.

Compilation-only handoff, requiring its own approval:

```powershell
pwsh -NoProfile -File .\verification\ops-r1\proposals\shared-acquisition\execution\invoke-build-capture.ps1 -Stage red
```

Underlying Cargo remains exactly:
`cargo test --lib --all-features --no-run --message-format=json`.
Return `target/ops-r1/shared-red-build-reviewed/build-receipt.json` for review.
Do not infer execution permission from compilation.

After separate approval of the actual executable/build identities:

```powershell
$env:OPS_R1_STAGE='red'
python .\verification\ops-r1\proposals\shared-acquisition\execution\capture.py --build-receipt .\target\ops-r1\shared-red-build-reviewed\build-receipt.json --case fitness --wall-seconds 60 --memory-mib 512 --stop-wait 5
```

Expected RED is one selected test failing the unchanged acquisition assertion with
actual `[4,4,16]` versus required `[1,1,4]`, while public lower-bound and four-owner
assertions pass. A timeout, memory stop, compile error, other assertion or missing
selection is not that RED. The monitor returns nonzero on the expected failing
test too: stop, preserve stdout/stderr/fixtures/stages/work counts and return the
actual receipt for review. Do not automatically advance to GREEN.

## Corrected GREEN and preservation procedure: planned only

After RED review and separate application approval, apply the next two exact
patches in order. Confirm the final candidate digest. Build once:

```powershell
pwsh -NoProfile -File .\verification\ops-r1\proposals\shared-acquisition\execution\invoke-build-capture.ps1 -Stage green
```

Return `target/ops-r1/shared-green-build-reviewed/build-receipt.json`. Only after
the new binary/build identities and execution are approved, run the same fitness:

```powershell
$env:OPS_R1_STAGE='green'
python .\verification\ops-r1\proposals\shared-acquisition\execution\capture.py --build-receipt .\target\ops-r1\shared-green-build-reviewed\build-receipt.json --case fitness --wall-seconds 60 --memory-mib 512 --stop-wait 5
```

Expected GREEN: one passed, zero failed/ignored; `[1,1,4]`; four owner
compilations/publications; unchanged empty lower-bound semantics. The witness
source hash is identical across RED and GREEN. Then execute only the approved
additional exact cases from the manifest using the same already-built executable
and capture command, changing `--case` to that reviewed name. There is no Cargo
invocation per test and no automatic retry/continuation after abnormal evidence.

The thin adapter imports the existing reviewed monitor bytes, pins the stage
source manifest, records adapter/monitor identities and preserves the raw monitor
receipt separately. The original independent watchdog, exact PID, 60 s / 512 MiB
/ 5 s controls and partial stdout/stderr/memory preservation remain unchanged.
Public fitness reuses the existing P0 sampler for partial stage counters in the
isolated operator child. Direct native controls emit final operation counters;
they do not all emit P0 fixture/query/stage files. A killed native control has
incomplete internal counters and must be reported that way.

The poison-recovery control deliberately catches an injected `failed leader`
panic; its panic-hook diagnostic is expected only if the test itself finishes
passed. Do not normalize an unexpected nonzero result as that expected event.

## Cost, checks and remaining gates

Observed prior all-feature no-run compilation: 590.986 seconds. Plan two staged
builds; about twenty minutes is a conservative planning reference if both cost as
much as that observation, not a measured prediction. Incremental build and new
case costs remain unknown. The 26 selected cases reuse one GREEN executable;
every child is bounded separately. Aggregate wall cost includes source/binary
hashing and legitimate currentness I/O, not just the 60-second child limit.
No timing research campaign or second full all-feature test run is proposed.

Static results are recorded in `static-checks.json`: sequential patch
applicability against disposable indexes; bounded rustfmt stdin/stdout formatting
and syntax parsing; Python AST and PowerShell parse; unchanged Witness truth,
authority fields and publication source; strict UTF-8 and code-size inspection.
These do not establish compilation, borrow/type correctness, zero Clippy warnings,
RED/GREEN, performance improvement, actual document admission or CI success.

Source/authority and directive hashes are in `source-authority-pins.json`.
The original pre-repair evidence archive and hashes are preserved unchanged.
All new packet files are covered by the governing OPS-001 checkpoint/manifest
association. Fresh actual graph reconciliation/admission is still required before
applicable publication; no authorization grant or protected contract change was
generated. No actual refusal is claimed without running the checker.

Deliberate eligibility restrictions are same request-directory/source-policy and
established seed route, plus independent nested acquisition. These implement
unknown-equivalence fallback, not a waiver or new public contract. No deliberate
deviation from the selected architecture is proposed. If verification exposes
identity/retention differences that violate existing contracts, stop and return
the exact conflict; no silent witness sorting, gate exception or source correction
is authorized by this packet.

No production patch application, build, test execution, CI change, protected
publication, grant, commit or push occurred under Disposition 11.
