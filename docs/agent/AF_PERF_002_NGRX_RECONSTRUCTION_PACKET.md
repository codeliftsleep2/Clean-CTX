# AF-PERF-002 / #173: reconstruction characterization packet

Agent: Agent MaxHeadRoom
Status: authored UNRUN; characterization only, no optimization or repair budget.
A2 remains MONITOR for its accepted decision; A3 regression protection is separate.

## Actual boundaries instrumented

WorkspaceIndex::rebuild_ngrx_import_refinements still retracts/rebuilds immediately.
New cfg(test) observations count object_member_value_bindings invocations, stored
forward-edge visits, eligible binding records and four cloned strings' logical byte
lengths; MAP candidate/binding comparisons; switch-value candidate/binding comparisons;
and one_hop_named_reexport read attempts/parse invocations. Existing direct export
read/parse counters remain distinct. No read is cached, skipped or moved, and no
predicate, traversal order, authority, publication or visibility rule changes.
Logical cloned string lengths are not allocator bytes or all reconstruction copies.

Tracked index_ngrx_reconstruction_af_perf_002.rs uses actual add_edges, with fixture
capture outside the measured empty-mutation interval. Growth1/2/4/8/16/64/256
independently varies unrelated forward edges E, binding records B, and candidate
count M. MAP and switch branches are separate; fixed-M/fixed-B axes expose joins
without a Cartesian campaign. Intended assertions characterize current E+B collection
and M*B comparison behavior; they are not future optimization budgets or RED claims.

MAP/switch candidates deliberately lack reducer/action declaration authority.
Expected refusal is explicit: no dispatch or refined map fact may be invented.
These negative index-input fixtures are not claimed producer-captured public query
integration or positive MAP/switch certification. Existing populated direct-action
controls retain supported facts, consumed export mutation/repair and immediate
retirement behavior. A new legitimate one-hop import/export fixture requires an
exact supported dispatch, then barrel mutation refusal and byte repair, with exact
layer, endpoints and owner/target files. Reexport read/parses must be visible separately.

## Operator packet

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-ngrx-reconstruction.ps1
```

Expected four tracked NgRx tests (two existing plus two new), nine af174 controls,
then focused all-feature/all-target warning-fatal lint, formatting, size and encoding.
Receipt preserves actual results and source hashes. Compile/fixture failure is not
a characterization result. No agent Cargo/test/build/formatter/lint/runtime ran.
Fast local checks cover script parsing, UTF-8/no BOM and authored line ceilings only.

## Decision boundary and remaining work

Return observed collection/join/IO counts and timing before choosing FIX/MONITOR/
ACCEPT; DESIGN REVIEW remains temporary. Count growth does not by itself establish
benefit from skipping or sharing validation. Empty-input reconciliation already
has measured export-currentness obligations; those cannot be removed incidentally.

This packet does not characterize every populated positive MAP/switch family,
root-retirement visibility/failure interleavings, delegated declaration clone/sort
cost, parser byte/allocation cost or all reexport shapes. These remain explicit
scope gaps. No reconciliation deferral, batching across public mutations, cache,
index redesign or production repair is authorized. Subsequent design must include
construction/maintenance cost and semantic preservation under the four owner criteria.
Source accounting and the remaining non-C# subsystem queue remain independent.
