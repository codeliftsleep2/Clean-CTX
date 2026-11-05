# AF-PERF-002: graph traversal setup and observation boundaries

Agent: Agent MaxHeadRoom
Status: source traced; characterization authored UNRUN. No repair authorized.

## Active entry points and delegated work

mcp/tool_handlers/query/graph.rs handles workspace_query transitive_dependencies
and has_cycle. For an exact identity, dependency preparation first traverses the
index and discards that returned Vec, then invokes hydration. Later resolved
answer construction traverses again unless the distinct C# dependency catalog
serves it. These observations straddle hydration and cannot be assumed to share
current authority or output. No removal or cross-boundary reuse is proposed.

WorkspaceIndex::transitive_dependencies_with_scope first checks forward membership,
then scoped start occurrence admission where relevant. Only after those checks does
it call build_node_index: clone/deduplicate all entity/forward/reverse identity keys,
sort unique nodes, clone keys again into a numeric map. BFS allocates a visited Vec
for all nodes, but builds adjacency only for expanded reachable nodes. Depth-limited,
unlimited, missing and scoped-negative paths therefore have independent costs.
Direct Calls remains excluded by the approved relation vocabulary.

Node sorting is O(V log V), plus key hashing/string lengths and cloned payload;
the BFS boundary is output/reachability-sensitive but whole-index setup is not.
A tiny reachable component can pay for unrelated retained components. This is a
structural risk until actual work and proportional alternatives are evaluated.
No direct-key BFS, cache or representation change is approved by this discovery.

Cycle witness production separately enumerates all nodes and admissible edges,
sorts outgoing edges for deterministic witness selection, then performs iterative
three-color DFS. Complete-set observation can be necessary for whole-graph absence
and deterministic witness selection. Do not label it the same defect as narrow
reachability. Witness provenance and scope filtering are correctness requirements.

## Next measured slice

One tracked test calls the actual WorkspaceIndex traversal, independently varying
unrelated disconnected edges and invocation count over1/2/4/8/16/64/256 plus16x16.
A fixed Start->Reached->Last component supplies exact ordered BFS tuple oracles.
Depth1, unlimited, missing start and target-only sink paths are tested independently.
Setup add_edges is outside timing and observations; no synthetic authority helper
bypasses the index operation. This is not a public MCP acquisition/currentness test.

Test-only counters observe complete node-index builds, input identity key counts
across three maps (before deduplication), unique sorted node totals and actual
expanded adjacency edge visits before relation/scope filtering. Key visits count
logical entries, not bytes or allocator events. Node totals are summed over calls,
not peak memory. Sorting comparisons and BFS visited initialization are not separately
counted. Descriptive elapsed times include all index traversal work and counters.
No future optimization budget or arbitrary threshold is imposed.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-traversal-work.ps1
```

Expected38 tracked executions: one new characterization,27 existing graph controls,
ten cycle-policy controls; then all-feature warning-fatal lint/format/size/encoding.
Logs and source hashes remain under target. Agent performed bounded source/text/
script syntax inspection only; no Cargo, formatter, lint, tests or runtime started.

## Open dimensions and accountable coverage

Scoped dense/out-of-scope population, candidate identity ambiguity, graph depth,
width, density, duplicate edge occurrences, removal/currentness, public hydration
multiplicity and failure paths remain independent obligations. Existing passing
controls are not relabeled as exhaustive current behavioral characterization.
Graph utilities also contain older recursive find_cycles; its other production
callers and output multiplicity remain untraced. No whole-graph safety claim follows.

Disposition STRUCTURAL RISK / MONITOR pending execution and design evidence.
Hierarchy placement remains protected; C2 MONITOR. Original9763 records preserved,
13 scoped outcomes/9750 pending plus96 callback supplements are unchanged. Source
path discovery is distinct from syntax accounting and behavioral completeness.


## Executed whole-index setup characterization

Operator traversal-work-20261104-174619 verifies38 tracked passes: one new
characterization0.35s,27 graph controls0.01s,10 cycle-policy controls0.01s; zero
failed/ignored. Warning-fatal all-feature/all-target Clippy1m13s, empty format-check
log, size and strict encoding2098 files pass. All 1303 receipt source hashes
match before documentation updates; inspection.json preserves56 parsed rows.
Agent inspected evidence only and did not start runtime work.

### Unrelated population, one successful invocation

| Unrelated edges U | Input key visits | Unique nodes built | Depth1 expanded edge visits | Unlimited expanded edge visits |
| ---: | ---: | ---: | ---: | ---: |
|1|11|5|1|2|
|2|15|7|1|2|
|4|23|11|1|2|
|8|39|19|1|2|
|16|71|35|1|2|
|64|263|131|1|2|
|256|1031|515|1|2|

Counts equal4*U+7 input keys and2*U+3 unique nodes in this fixture. These are
logical map-entry visits and summed output lengths of setup, not actual hash/string
comparisons, peak memory, allocator bytes or sort comparisons. The returned exact
BFS identity list stays Reached for depth1 and Reached,Last for unlimited; unrelated
nodes never enter results. U256 descriptive timings1722us/2041us include the complete
index operation. No enterprise latency or isolated cost fraction is inferred.

### Repeated invocation, fixed U1

Q1/2/4/8/16/64/256 invokes complete node construction Q times, visits11*Q input
keys and builds5*Q unique-node entries. Depth1 expands Q edges; unlimited expands
2*Q. Joint U16/Q16 observes16 builds,1136 input-key visits,560 node entries and
16/32 expanded edge visits. This is repeated setup per fresh operation, not proof
that state may be reused across queries or hydration boundaries.

Absent and target-only Last starts perform zero node builds/key/edge work at every
axis point. Source forward-membership guards already avoid this negative-path cost;
do not describe all negative queries as expensive. Scope/admission negatives are
not newly measured by this packet. Independent reachable breadth/depth/density,
duplicate occurrences, scoped retained population and public hydration remain open.

## Bounded candidate and disposition

Measured finding: unrelated retained identities increase whole-set construction
for a fixed small successful reachability request. Classification DESIGN REVIEW
for this setup slice; no production correction authorized. Existing cycle witness
sorting remains independent, necessary-for-contract work until separate evidence.

Smallest candidate for owner review: use an operation-local queue and visited set
of exact EntityKeys in transitive_dependencies_with_scope, reading existing forward
adjacency directly, instead of constructing a global numeric node table and visited
Vec. Preserve the current start guards, exact relation/provenance filtering, depth
semantics, root exclusion, duplicate suppression and first-discovery BFS order.
No state may survive a call or cross hydration. Keep cycle build_node_index and
shared graph utility behavior unchanged; no global graph rewrite or cache follows.

Source proof prerequisite: show every stored eligible edge object is representable
by the existing node universe (entities/forward/reverse) before dropping numeric-map
filtering. Preserve unresolved-object behavior and exact query order. Direct-key
hashing/cloning may cost more on dense reachable graphs; measure reachable fraction,
small workloads and capacity/payload before choosing. No implementation or claimed
post-repair benefit exists. This representation tradeoff deserves separate approval,
unlike the earlier two-site reuse of an already-built hierarchy map.

Syntax accounting remains9763 preserved/13 scoped/9750 pending plus96 callback
supplements;38 passes and56 behavioral rows do not advance syntax dispositions.
Public acquisition/currentness and cycle witness absence cannot be inferred from
these direct-index operation fixtures. Broader discovery remains incomplete.


## Follow-up: dense reachable fraction and lifecycle controls authored

No direct-key BFS implementation is authorized or introduced. Next characterize the
existing operation for chain, star and fully connected directed graphs at1/2/4/8/
16/64/256 nodes. Each graph runs depth1 and unlimited queries with exact ordered
identity lists. Dense256 has65280 actual relationships; fixture construction remains
outside observations/timing. This distinguishes setup cost from unavoidable scans
of reached dense adjacency and output growth. It is not an enterprise workload.

Additional same-index lifecycle assertions cover duplicate publishers, provenance-
scoped edge filtering, duplicate publisher removal, full local-owner retirement,
outside-only start refusal, negative-depth equivalence and reinsertion. No out-of-
scope edge may extend the admitted walk. Existing graph/cycle controls remain intact.
These new tests are authored UNRUN; source inspection is not runtime certification.

Normal add_edges registers both endpoint occurrences then writes each accepted
relationship into forward and reverse. Refinement insertion also writes both
adjacencies. remove_file drops exact asserting occurrences from both before retiring
unreferenced entities/buckets. This supports, but does not alone exhaustively prove,
the invariant that surviving forward objects belong to the numeric node universe.
Synthetic/manual index mutation and all publication paths must remain explicit
before any traversal representation substitution; do not simply drop a filtering
step because ordinary insertion appears paired.

Ordering source trace: numeric IDs come from sorted keys, but BFS enqueues existing
forward Vec occurrences in their supplied order; sorting does not reorder that Vec.
Root is marked visited before enqueue, and later duplicate discoveries are ignored.
A candidate replacement must reproduce these facts exactly, including cycle and
self-loop root exclusion. Hash-set iteration must never become output order.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-traversal-density.ps1
```

Expected39 tracked executions (two characterization/semantic tests plus27 graph
and10 cycle controls) and focused gates. It reruns the small earlier characterization
because the same protected file and counter boundary are extended; no completed
repair campaign is repeated. Only bounded syntax/text checks were agent-run.
Complete-operation capacity/allocation comparison with an alternative remains
unmeasured, so the candidate stays DESIGN REVIEW, no repair budget yet.


## Reachable density and scoped lifecycle execution verified

traversal-density-20261104-175647 passes39 tracked executions: two characterization/
lifecycle tests1.85s,27 graph controls0.01s and10 cycle-policy controls0.01s. Zero
failed/ignored. All-feature/all-target warning-fatal Clippy23.34s, empty format-check,
size and strict encoding2099 files pass. All 1303 receipt source hashes match
before this documentation update. inspection.json retains42 parsed density rows;
56 unrelated/invocation rows remain in the original test log. No agent runtime.

| Shape,256 nodes | Depth | Input keys | Unique nodes | Expanded edges | Results | us |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
|chain|1|766|256|1|1|971|
|chain|unlimited|766|256|255|255|1361|
|star|1|512|256|255|255|1038|
|star|unlimited|512|256|255|255|1083|
|dense|1|768|256|255|255|1369|
|dense|unlimited|768|256|65280|255|71458|

Each interval builds one node table. The dense graph retains65280 directed edges;
its depth1 query expands only255. Unlimited visits65280 despite the same255 output
identities. Increasing elapsed time here cannot be blamed solely on global setup:
that setup remains768 input keys and256 nodes for both queries. Actual adjacency,
filtering, endpoint-key creation, hashing and visited work vary. No assertion that
every expanded edge is globally unavoidable follows; no alternate algorithm was
measured. Counter overhead contributes to all descriptive single samples.

All42 shape/depth rows pass exact ordered identity lists. The scoped lifecycle
control independently passes duplicate publisher preservation/removal, negative-
depth equivalence, local retirement, outside-only start refusal and reinsertion.
The original self-loop rows exclude the start identity. This is executed index
behavior, not public MCP hydration, declaration/currentness or failure certification.

### Representation decision return

Retain DESIGN REVIEW; do not implement direct-key BFS on these data alone. There
is measured irrelevant whole-index work for narrow reachable components. However,
the existing numeric traversal uses integer visited checks for expanded edges;
a naive direct-key replacement can add visited hashing of owned string tuples on
every dense edge. Removing setup does not prove construction-inclusive improvement
for reachable-dense inputs. Both designs already construct target keys for lookup,
but their visited representation and key retention costs differ. Quantify rather
than assuming these costs cancel.

Recommend a separately approved, operation-local comparison prototype only if the
owner prioritizes this repair: compare exact original and candidate outputs over
current sparse/dense/scoped fixtures; record setup, edge key/hash work, visited and
queue capacities, retained string payload and full-operation timing. No prototype,
new helper/API or production algorithm is authored here. No adaptive threshold or
cache is proposed. If dense/small costs outweigh narrow-query benefit, MONITOR is
an acceptable result. Cycle witness and cross-hydration observations stay separate.

This completes the currently authored characterization packet, not full traversal
coverage. Public preparation/answer multiplicity, all producers/synthetic mutation
paths, ambiguity/authority failures, independent provenance density and allocation
bytes remain gaps. Other production subsystems and the9750 pending syntax records
must continue independently. No new runtime gate is requested merely to record
these results; protected repairs remain unchanged.
