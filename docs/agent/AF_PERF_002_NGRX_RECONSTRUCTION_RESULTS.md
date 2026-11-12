# AF-PERF-002 / #173: measured reconstruction and bounded design return

Agent: Agent MaxHeadRoom
Status: characterization GREEN; production repair PROPOSED ONLY.
No reconciliation deferral, lookup redesign or validation reuse is implemented.

## Actual execution

Operator ngrx-reconstruction-20261104-123958: four tracked NgRx tests19.19s
and nine existing membership controls0.27s, zero failed/ignored. Warning-fatal
all-target/all-feature Clippy28.27s, empty format logs, size5 active files and
strict encoding2065 files pass. Receipt hashes match current source. inspection.json
preserves56 parsed reconstruction rows and three reexport currentness observations.
No agent test/build/lint/runtime was started. These are small index-operation fixtures,
not enterprise measurements or full public-query/capture integration certification.

## Measured work

E is unrelated forward facts, B eligible object-member binding records, M candidate
map/switch records. Each interval is one actual empty add_edges mutation following
fixture setup. No timings include building the fixture. Refusals are explicit.

| Branch | E | B | M | Collector visits | Cloned string bytes | Join comparisons | us |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| no consumers | 256 | 0 | 0 | 256 | 0 | 0 | 33 / 32 |
| no consumers | 0 | 256 | 0 | 256 | 34596 | 0 | 231 / 208 |
| map | 0 | 16 | 256 | 16 | 2124 | 4096 | 82702 |
| map | 0 | 256 | 16 | 256 | 34596 | 4096 | 8891 |
| switch-value | 0 | 16 | 256 | 16 | 2124 | 4096 | 40468 |
| switch-value | 0 | 256 | 16 | 256 | 34596 | 4096 | 2845 |

The paired no-consumer timings come from the two fixture modes; neither has a
selected branch. Across controlled growth1/2/4/8/16/64/256, collector visits equal
E+B and joins equal M*B. String bytes count the logical lengths of four cloned
strings per binding, not capacity, allocator overhead or all reconstruction clones.
All negative candidates correctly refuse unsupported refined facts. This does not
certify positive map/switch provenance or ambiguity policy exhaustively.

Equal4096 comparison counts have very different elapsed time when M and B trade
places. Target/module resolution, declaration lookup and other per-candidate work
remain inside the interval. No syscall/canonicalization attribution is established;
those costs require separate instrumentation before proposing a bucket repair.
Repeated candidate export reads are zero in these unsupported fixtures, so these
rows do not quantify populated positive export-validation costs.

## Currentness and independent protection

The one-hop fixture establishes an exact supported dispatch through import/export,
then edits the consumed barrel to export nothing and requires immediate retirement
on empty add_edges; restoring bytes restores the exact original endpoint/layer/
asserting-owner/target-file fact. Each phase observes one reexport read and parse,
separate from exports_const/function counters. Existing direct-export mutation and
byte-repair controls also pass. Empty input is still not a semantic no-op.

The nine #174 controls pass; they protect different facts and do not certify
complete enclosing reconciliation complexity. Root-removal and publication failure/
visibility protocols are not changed or inferred from this index-call packet.

## Smallest independently approvable candidate: demand-local binding collection

Propose replacing unconditional object_member_value_bindings construction with one
explicit owned lazy slot local to a single rebuild invocation. Only existing map
and switch-value branches can initialize it, after their existing target-resolution
gates reach consumption. Both use the same immutable ordered Vec result once built.
No map/index/cache, IO receipt, cross-call reuse or authority policy is introduced.
Other branches still validate exports and rebuild immediately as before.

Proof obligation before approval: verify every value_bindings use (map loop and
switch-value selection) and show eligible binding facts cannot change before a
later lazy initialization. Collection reads only forward ReferencesValue facts at
exact typescript_object_members layer. Earlier refined insertion emits ngrx-layer
facts, so it appears unable to change this set, but this must be reviewed against
all insertion/retraction paths, not assumed from mutable self ownership. Preserve
original binding occurrence/order, duplicates and all existing refusal predicates.
An alternative is a pre-pass demand check followed by one original-position build;
that preserves the exact observation point but adds an M scan and may construct
for candidates that refuse before needing bindings. Compare the two explicitly.

Expected benefit: no-demand operations avoid E+B collector visits and B four-string
clones entirely. Demanding operations retain existing complete collection and M*B
joins. Additional state is one local Option<Vec<...>>, destroyed at rebuild end;
no maintenance/invalidation protocol. This is a scaling-prevention and allocation
opportunity with measured growing wasted work, not demonstrated enterprise latency
benefit. Small observed costs and construction tradeoff remain owner decision inputs.

### Four-criterion disposition

- Demonstrated problem: measured full collector traversal and cloning with no
  consumer; no semantic use of that result in the measured operations.
- Significant benefit: eliminating that measured work is credible for growing E/B
  and repeated mutations, but benefit on real workloads is unmeasured. No current
  claim justifies a broad reconciliation redesign.
- Semantic preservation: local ownership and unchanged consumer matching are
  plausible; observation-position/insertion proof and positive-family controls
  remain required before implementation.
- Proportional complexity: one explicit local slot is smaller than indexed join
  buckets or a cache. If proof/API overhead expands, reassess rather than broaden.

Classification: collector DESIGN REVIEW, narrow FIX candidate requiring owner
approval; map/switch M*B joins DESIGN REVIEW/MONITOR pending candidate-path cost
attribution and positive density/refusal characterization. Reconciliation frequency
and export-currentness stay separately unresolved; no implied deferral approval.

## Next decision and permanent protection if approved

Approve only the call-local collector slice after proof review, or retain MONITOR.
Before any repair, author deterministic zero-unreachable-work regression RED against
the current implementation. Require unchanged GREEN after repair, one collection
per reached consumer operation, correct mixed-branch sharing and fresh-call behavior,
positive/refused/duplicate/retirement controls, construction-inclusive timing/memory,
static known-sink coverage and isolated real production mutation RED/restored GREEN.
Current passing characterization is not retroactively optimization RED.

No new runtime command is needed simply to record this return. A2 remains MONITOR,
A3 protection remains separate, and broader subsystem discovery continues. Original
syntax accounting9763 preserved/13 scoped/9750 pending plus96 callback supplements
is unchanged;56 observed operation rows do not imply additional census dispositions.

```architecture-graph
node: DOC-174
kind: RESEARCH
status: INVESTIGATING
title: Measured NgRx M times B joins with production repair proposed only
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_NGRX_RECONSTRUCTION_RESULTS.md
consumes_authority:
  - PERF-004-DOCTRINE
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
```
