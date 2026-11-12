# #174 / AF-PERF-002: ordered-index membership design

Agent: Agent MaxHeadRoom
Status: eight direct budget/invariant/memory tests verified GREEN; static/fitness/preservation/lint and isolated mutation proof pending.
Generation 78 remains implemented/targeted GREEN/pushed; independent hardening only.

## Eight-test invariant and memory GREEN; next focused gate

Verified index174-after-20261103-175559: eight passed, zero failed/ignored,
1.11s. Exact source hashes and logs are preserved in
`target/af-perf-002/index174-invariant-green-evidence/receipt.json`. Vec/set
correspondence, physical-versus-publisher ownership, exact ordering, clone
divergence and synthetic NgRx add/remove controls pass.

At 256 long-name/path entities and two publishers, companion string capacity
is 156566 bytes, with 1024 stored key values and aggregate set capacity 2432.
Duplicate density 1 versus 4 does not increase retained companion payload.
This is capacity/payload accounting, not exact allocator or hash-table bytes.
Existing physically distinct occurrences remain after publisher retirement;
that existing removal behavior is preserved. Map capacities remain allocated.

The new bounded AST gate checks known insertion helpers for scans and known
ordered/companion writers for unclassified mutations. Its scope is finite, not
a complete Rust call graph. Mandatory fitness now selects nine af174 tests and
hashes workspace production/test sources. The static test and focused gate are
UNRUN. Next operator command:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-index174-focused.ps1
```

This runs mandatory fitness, affected index preservation, all-feature warning-fatal
Clippy, format, size and encoding checks. Isolated production mutation sensitivity
remains a separate required step after clean GREEN. No agent test/build/lint/runtime
was started. Declaration scans, reconciliation frequency and other enclosing costs
remain outside this repair; #174 is not closed.

## Direct linear-budget GREEN; invariant/memory controls authored, UNRUN

Verified index174-after-20261103-175243: four pass, zero failed/ignored, 0.72s.
Receipt source hashes match; original RED test bytes are unchanged. Name identity,
physical occurrence and publisher-key dimensions all reach 256 with zero counted
Vec membership comparisons and no more than 512 logical companion probes per
bucket. At n=8 the old 28-comparison failure becomes <=16 probes. Timing is a small
instrumented observation, not whole-index speed or closure of enclosing costs.
Evidence: target/af-perf-002/index174-direct-green-evidence/receipt.json.

New tracked index_membership_lifecycle_af_perf_002.rs independently compares every
companion to the exact Vec set image after mutations, asserts no Vec duplicates,
checks multiple publishers of one physically located occurrence, exact order,
remove/readd/root retirement, physical owner deletion, None identity, divergent
clone and real NgRx synthetic add/target retirement/rebuild/caller removal.
Existing source-removal policy is preserved, including physical occurrences whose
file differs from removed publishers; no new last-publisher retirement is invented.

Memory fixture varies 1/2/4/8/16/64/256 and duplicate densities 1/4 with two
publishers, long names/paths and clone/removal. Reports companion string capacity,
key/header sizes, set/map capacities and retained buckets. This is explicit payload/
capacity accounting, not exact allocator heap bytes; no allocation claim is made
until results are observed. New assertions are UNRUN.

Next focused command uses the same wrapper; it now selects eight tracked tests
(four unchanged budget/order tests plus four new invariant/lifecycle/memory tests):

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-index174-membership.ps1 -Phase After
```

Static scan detection, blocking coverage drift, intentional mutation sensitivity,
affected existing semantic regressions and AF/lint gates remain pending. Enclosing
declaration/reconciliation/other growing work remains separately attributed and
#174 stays open. No production behavior changed during receipt inspection; only
new tracked tests and their module registration were added. No agent runtime ran.

## Three intended budgets RED; approved companion repair authored, UNRUN

Verified index174-before-20261103-174737: three intended failures, one order/clone
control passes, zero ignored, 0.07s. Each growth dimension reaches n=8 with 28 actual
membership comparisons against budget 16; larger points were not reached. Baseline
logs/source/regression SHA-256: target/af-perf-002/index174-observed-red/receipt.json.
Original four tests were isolated during edits and restored byte-exactly.

WorkspaceIndex now owns the three approved private membership maps. Registration
moves to index/membership.rs; paired helpers own entity/name insertion, publisher
append/removal, physical occurrence removal and affected-name cleanup. Existing
Vecs still provide ordered query data. No query reconstructs companions or treats
set presence as semantic authority. Normal add_edges/remove_file and synthetic
NgRx registration/retraction now call paired helpers; Clone/new cover initialization.
File retirement still removes exactly the physical occurrences the old code removed;
no cross-publisher retirement semantics were changed. Declaration storage and all
reconciliation timing/policy remain untouched.

Test-only logical membership probes count contains and successful insertion decisions
separately, not actual hash equality/collision costs. Source clones remain a memory
tradeoff; full capacity/payload and long-path measurements still required. No memory
acceptance is claimed before those measurements. Static invariant/ownership/memory
and mutation enforcement are remaining acceptance work, not satisfied by merely
passing a helper budget. The next command runs only unchanged four regressions:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-index174-membership.ps1 -Phase After
```

All post-edit runtime gates UNRUN. After direct GREEN, complete representation/
normal+synthetic lifecycle and enclosing-operation coverage, memory report, static
scan detection and isolated production mutation proof, affected semantics/AF/lint.
#174 remains open through those checks and independently reported enclosing costs.
No agent formatter/Cargo/test/build/runtime ran. Generation 78 remains baseline.

## Owner memory-cost refinement and enclosing-path acceptance

Owner requests evaluating key ownership/string duplication without adding interning,
reference-counted keys or another abstraction. EntityKey is currently an owned
(String,String,String) tuple. The approved companion maps therefore duplicate
string storage when cloning keys; safe borrowed references into independently
mutated HashMap/Vec/String owners cannot be retained in WorkspaceIndex without
self-reference/lifetime/invalidation complexity. Do not introduce that complexity
for this repair or silently change public EntityKey ownership.

Use existing key borrows for membership lookups, and allocate/clone only when a
new companion entry is actually inserted. Repeated duplicate participation must
not clone owned key/path strings before determining existing membership where
ordinary borrowed lookup permits it. Move already-owned temporary keys into their
final companion location where ownership flow permits, without changing the Vec's
retained order/storage or adding new key wrappers. Do not duplicate whole EntityRef
values; occurrence membership stores only the exact Option<String> file identity.

Measure memory after representative 1/2/4/8/16/64/256 growth and duplicate densities:
map/set lengths and capacities, owned String len/capacity for keys and occurrence
paths, size_of headers, clone behavior and retained capacity after removal. Report
payload/header lower-bound accounting separately from actual allocator/hash-table
allocation overhead; std collection capacity alone is not an exact heap-byte
measurement. Do not label a calculated payload estimate an allocator measurement.
Include long names/paths and multiple publishers so short synthetic keys do not
hide duplication. If exact allocation measurement requires new dependencies or a
process-wide allocator, return that requirement rather than introducing it merely
for a memory check. Straightforward owned sets remain recommended if the observed
tradeoff is reasonable; no arbitrary universal memory cap is invented.

#174 remains open until enclosing add_edges and synthetic insertion/retraction
operations have justified scaling. Companion probe counts alone cannot hide
register_declaration scans, refined-entity membership scans, file-key merge scans,
reconciliation or another upstream/downstream growing traversal. Count/attribute
these independently, enforce the approved repaired subset and return remaining
mechanisms explicitly. This approval does not silently extend to declaration or
adjacency redesign or reconciliation deferral. Acceptance reports distinguish the
finite repair from closure of the broader #174 investigation.

## Owner-approved representation; intended-budget baseline authored, UNRUN

Owner approves the three private HashSet companions subject to paired writers,
exact Vec ordering/ownership preservation, invariant tests, measured work and
mutation/static enforcement. No membership repair is applied before intended RED.

Rechecked writer references immediately before baseline: register_entity and
add_edges publisher merge; remove_file normal occurrence/name/file removal;
register_refined_entity and drop_ngrx_refined_entities synthetic writers. Clone/
new initialize/copy ownership; declarations remain a separate excluded structure.
Any additional writer discovered during implementation blocks acceptance until
classified and covered. index.rs decomposition will follow registration ownership,
not hide mutation state in a generic collection registry.

New tracked index_membership_budget_af_perf_002.rs adds three independent actual
add_edges growth regressions through 1/2/4/8/16/64/256. Their linear budget counts
existing comparisons plus future companion membership probes, bounded by two
units per registration per companion in these exact fixtures. Legitimate hash
cost/collisions and separate reconciliation are not claimed constant. The fourth
test preserves duplicate suppression, exact publisher Vec order, remove/readd and
divergent clone order. Full cross-publisher and synthetic invariant/memory controls
remain implementation acceptance requirements, not certified by these four tests.

Operator baseline:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-index174-membership.ps1 -Phase Before
```

Expected result: three intended quadratic-budget failures and one semantic control
pass. Compile/fixture failures are not valid RED. Logs, exact selected source/test
bytes, command phase and SHA-256 are preserved under target/af-perf-002/index174-*.
Only test registration and a test-only future membership-probe field were added;
production scans/storage/semantics are unchanged. After valid RED, isolate/restore
regression bytes, implement approved paired helpers/companions, then test unchanged
budgets before broader required invariant/preservation/static/mutation/lint checks.
No agent formatter/Cargo/test/build/runtime ran. Generation 78 remains baseline.

## Smallest proposed representation

Keep all existing public/query iteration Vecs and their insertion/retention order.
Add three private membership companions owned by WorkspaceIndex:

- name_members: HashMap<String, HashSet<EntityKey>>, exactly the keys in each name_index Vec.
- occurrence_members: HashMap<EntityKey, HashSet<Option<String>>>, exactly the entity.file values in each entities Vec. Preserve literal file strings and None semantics; no new normalization.
- publisher_members: HashMap<String, HashSet<EntityKey>>, exactly the keys in each file_map publisher Vec. Publisher ownership remains separate from physical occurrence identity.

HashSet iteration is never used to produce public answers. Its insert/remove result
controls whether the corresponding Vec append occurs. No global EntityRef equality,
edge identity, canonical coordinates, sorting or public API/serde changes. Memory
cost is O(name keys + physical occurrences + publisher keys) extra membership
storage with duplicated keys/files; measure it before acceptance, not claim free reuse.

Alternative: BTreeSet companions provide worst-case logarithmic membership at higher
comparison/key-order cost. Recommended HashSet matches existing HashMap/edge HashSet
usage and expected-linear work; it does not promise worst-case O(1). Batch-local
sets alone cannot avoid rescanning accumulated cross-file occurrence/name buckets
on every independent add_edges call, so do not solve the measured occurrence case.

## Synchronization invariant

At every completed public mutation, each companion equals the exact set image of
its corresponding Vec, and no Vec has duplicate members under its existing identity.
Vectors remain the authoritative ordered storage; companions accelerate membership
and never certify semantic authority. Use narrow private helper methods to update
both representations, not a generic indexed-collection registry. Debug/test checks
compare images after mutations; production queries never silently rebuild caches.

## Mutation lifecycle traced

| Existing path | Required synchronized operation |
| --- | --- |
| WorkspaceIndex::new / Default | initialize all maps empty |
| add_edges -> register_entity | insert name/key membership, preserve first Vec append; insert physical occurrence once; still record every asserting publisher even if physical occurrence already exists |
| add_edges final file_map merge | publisher_members insertion gates ordered file_map append; avoid repeated duplicate participation scans |
| remove_file | remove publisher bucket and membership; remove only physical occurrences that current implementation removes; remove matching occurrence-member entries; drop empty identity state; clean only affected name buckets/members |
| remove_files_in_root | existing remove_file loop uses identical helpers; no reconciliation deferral or new batch policy |
| register_refined_entity | replace its independent occurrence/file-key contains scans with the same paired membership helpers; preserve its existing early-exists behavior and ownership |
| drop_ngrx_refined_entities | update occurrence/file/name companions for every Vec retain/remove, preserving surviving order and existing refinement ownership behavior |
| Clone | derive Clone copies Vecs and companions together, no sharing/locks |
| declarations/file_declarations | separate authority and storage; not migrated or silently counted as fixed by this slice |
| file_map() accessor | read-only reference to existing ordered Vec map remains unchanged |
| MCP state creation/reset | state constructs WorkspaceIndex::new; no reset/clear method was found in inspected index modules; any future mutation must use paired helpers |
| persistence/restore | WorkspaceIndex currently derives Debug/Clone, not serde; semantic snapshots restore via normal publication/add_edges. No serialized companion or durable-format bump proposed |

Searched entities/name_index/file_map mutation references in index.rs and index/*;
normal and NgRx paths above are the identified writers. Before implementation,
recheck call sites and compiler access boundaries; any additional writer requires
explicit synchronization coverage, not an assumption that private state is safe.
No change to existing source-removal ownership semantics is authorized here.

## Complexity and deterministic budgets

Let E be edge participations, U unique publisher keys, O distinct physical entity
occurrences and K name/identity pairs. Target expected O(E+U+O+K) membership operations
for insertion over the participating facts, excluding separate edge reconciliation,
source validation and allocation/hash byte costs. Legitimate Vec retain work during
removal is proportional to affected buckets and preserves order; do not require O(1)
removal or claim #55/#173 global paths are all repaired.

Track actual membership probes/inserts per companion and reject reintroduced Vec
membership scans. For controlled 1/2/4/8/16/64/256 insertion fixtures, require probes
bounded by registration attempts + unique file-key merge attempts at each call
boundary, independent of existing bucket length. Hash collisions/key bytes are not
captured by logical-probe counts; static mutation tests and runtime semantic tests
are complementary. Include same name across different domain/type keys, same exact
key across files, duplicate participation, multiple publishers naming one physical
entity, add/readd/removal/root retirement, clone then divergent mutation, and NgRx
synthetic registration/retraction. Check exact Vec order and set/Vec invariant after
each operation. Existing file-local cleanup, ambiguity/edge occurrence and generation-
78 admission semantics remain independent authority.

## Acceptance before production merge

1. Owner approves this private representation/lifecycle design or selects BTreeSet.
2. Add intended scaling RED against existing scans; keep the first baseline as
   characterization, not retroactively call its passing observations RED.
3. Isolate/restore valid regression bytes, implement only this paired membership
   slice, decompose active index.rs along registration ownership (already 613 lines).
4. Run unchanged budgets and affected mutation/ownership/order semantic tests,
   expanded AF gate and focused lint. No historical full campaign automatically.
5. Extend AST known sinks/callers to registration/refinement membership paths and
   classify new mutations; intentional isolated production scan reintroduction must
   fail both relevant deterministic budget and static coverage gate. Restore source
   and run clean GREEN. No generic runtime fitness or authority registry.

## Separate decisions and risk

This does not optimize declaration storage, adjacency queries, witness validation,
NgRx reconciliation frequency or snapshot mutexes. Each needs its own measured
slice. No concurrent protocol change: caller synchronization still owns exclusive
mutation. Companion divergence is a correctness risk; direct mutation must be
encapsulated and test invariants exercise normal and synthetic paths. Do not use
membership presence as currentness, provenance or declaration proof. No cache TTL,
cross-call authority reuse or semantic generation change follows.

## Focused run 181031: semantic/static GREEN, lint refusal corrected

Operator run index174-focused-20261103-181031 passed 34 mandatory fitness
executions (14 AF, five resolver, six delegated, nine af174), 115 existing index
tests and one populated reconciliation control. Size and scoped format passed.
Clippy refused three warnings: relocated registration documentation left at its
old location, an obsolete unused comparison helper, and a needless generic borrow
in characterization code. The doc now follows its method; the unused helper is
removed (comparison counter fields remain), and the borrow is removed. No warning
suppression or membership-policy change.

Refresh focused evidence with the same focused command, including formatting.
No agent test/build/lint/runtime ran. Isolated mutation proof remains pending.

## Focused run 182051: lint GREEN; encoding restoration

Clippy completed cleanly in 35.32s and format passed. The final encoding gate
refused mojibake in index.rs comments. Byte-level comparison with HEAD establishes
UTF-8 punctuation had been decoded through Windows-1252 and re-encoded during
earlier file editing; this was real on-disk corruption, not console rendering.
Restored 34 comment lines from their exact HEAD Unicode text using byte-captured
git output and explicit UTF-8 writes. Noncomment bytes are unchanged. Future edits
must retain explicit encoding and avoid legacy shell text round-trips.

Only the encoding gate needs rerunning for this comment restoration; previous
semantic/static/lint observations remain applicable to unchanged executable code.
No agent runtime or gate was started. Isolated mutation proof remains pending.

## Encoding owner-reported GREEN; isolated mutation handoff

Owner reports encoding PASS after comment restoration. New operator driver
run-index174-mutation.ps1 isolates exact current sources in a local clone under
target, reintroduces a real name Vec membership scan, and requires both production
work-count and static AST RED from the mandatory af174 test selection. Compilation
failure alone cannot qualify. Finally restore original bytes and require all nine
tests GREEN. Preserve source hashes, original/mutated bytes, diff and logs. Active
checkout/index are untouched. Isolated Cargo artifacts may require a fresh build.
Driver syntax checked only; execution remains UNRUN and operator-owned.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-index174-mutation.ps1
```

Broader coverage mapping and adversarial audit remain outstanding independently.
No #174 closure or system-wide completeness is claimed.

## Isolated production mutation verified RED -> restored GREEN

Run index174-mutation-20261103-183035 produced exactly the two intended failures:
name-identity growth budget and AST insertion-scan enforcement. Seven other af174
tests passed; zero ignored; mutated execution 0.42s. Restoring original production
bytes produced nine passes, zero failures/ignored, 0.85s. Receipt reports exit 101
for intended RED, active source unchanged and isolated source restored. Inspection
verified current relevant Rust hashes and exact isolated membership restoration.
Evidence: target/af-perf-002/index174-mutation-20261103-183035/receipt.json,
mutation.diff, mutation-tests.log and restored-tests.log. This is tracked-test
production sensitivity evidence, not binary acceptance or whole-index complexity
certification. No agent test/build/runtime ran.

The finite membership repair now has original intended RED, linear-growth GREEN,
ordered/publisher/clone/synthetic invariants, scoped AST enforcement, isolated
mutation RED/restored GREEN, focused index/fitness preservation and clean Clippy/
format plus owner-reported encoding PASS. Extra string capacity remains 156566
bytes at 256 long-name/path entities and two publishers, excluding allocator/table
overhead. Remaining key clones during map entry are a known transient allocation
cost, not claimed eliminated by retained-memory measurements. No interning added.

#174 remains open: enclosing declaration registration, refinement traversal and
reconciliation costs need separate characterization/disposition. Broader audit
coverage map, adversarial second pass, blocking coverage drift beyond the finite
known writer scope, and witness/currentness/lock designs remain outstanding.

```architecture-graph
node: DOC-168
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Finite ordered-index membership repair and outstanding broader audit
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_INDEX_MEMBERSHIP_DESIGN.md
preserves:
  - PERF-005-BOUNDED
uses_evidence:
  - DOC-145
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
```
