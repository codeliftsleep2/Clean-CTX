# AF-PERF-002: enclosing insertion return and declaration design

Agent: Agent MaxHeadRoom
Status: characterization verified; declaration repair PROPOSED, NOT IMPLEMENTED.
Generation 78 remains baseline. Discovery incomplete. #174 remains open.

## Actual packet evidence

enclosing-packet3-20261103-184148: two characterization tests PASS (0.04s),
nine existing af174 controls PASS (0.21s), zero failed/ignored. All 16 source hashes
in receipt.json match current files. Format/size were part of the operator packet;
affected instrumentation lint/encoding refresh remains outstanding. Evidence is
under target/af-perf-002/enclosing-packet3-20261103-184148. These assertions establish
existing quadratic behavior, not an optimization RED or repaired complexity budget.

| n | One declaration identity, n files: occurrence comparisons | n unique declarations, one file: file-key comparisons | Rebuilds in either growth fixture |
| ---: | ---: | ---: | ---: |
| 1 | 0 | 0 | 1 |
| 2 | 1 | 1 | 2 |
| 4 | 6 | 6 | 4 |
| 8 | 28 | 28 | 8 |
| 16 | 120 | 120 | 16 |
| 64 | 2016 | 2016 | 64 |
| 256 | 32640 | 32640 | 256 |

The independently unselected comparison dimension is zero in each fixture.
Both series exactly follow n(n-1)/2. Repaired membership probes remain bounded:
at n=256 unique declarations they are [512,512,512]; shared identity across files
gives [257,512,512]. Thus declaration scans coexist with bounded membership
helpers at the same enclosing add_edges boundary; the cost was not eliminated
from the complete operation. Observed n=256 elapsed times were 10994us for unique
per-file keys and 4968us for shared identity. Instrumented small-fixture times
do not establish enterprise latency or consequential wall-time attribution.

For 1/2/4/8/16/64/256 empty insertions, reconciliation invocations equal call
count. Repeated duplicate declaration insertions do likewise, with declaration
comparisons [n,n] against the single stored declaration. Candidate count is zero;
this packet does not measure populated no-op reconstruction or authorize skipping
a currentness observation on an apparently unchanged index.

## Architectural history and source-wide pattern search

register_declaration independently maintains ordered occurrence and publisher-key
Vecs using linear membership. It has the same structural failure as the three
repaired membership dimensions, but declaration authority is a separate lifecycle.
The inspected declaration file's latest commit is 04c4cad7 (Issue #139 reconciliation).
That attribution does not prove who introduced every line or why. No claim of a
single historical cause is justified without line-history review.

A targeted source-wide lexical census of contains/any calls excludes test trees
and preserves complete raw matches plus source hashes under
target/af-perf-002/membership-pattern-census. It is candidate discovery, not type-
resolved AST enforcement or exhaustive complexity classification. Initial reviewed
instances follow; none is silently repaired:

| Exact path/function | Pattern / dimensions | Disposition |
| --- | --- | --- |
| workspace/index/declarations.rs::register_declaration | append after occurrence/file-key scan; independently growing dimensions measured above | MEASURED DEFECT; bounded FIX design below, approval required |
| workspace/index.rs::resolve_selector | for matching incoming edges, scan accumulated subject_keys before append | STRUCTURAL RISK / MONITOR; selector fan-in vs distinct subjects, negative lookups and duplicate assertions need counts |
| cbm/client.rs::extract_trace_edges | direct caller/callee pairs -> scan growing JSON edge Vec for exact pair -> append | STRUCTURAL RISK / MONITOR; unique and duplicate trace sizes, producer limits and enclosing transport calls need characterization |
| angular_meta/config_composition.rs::project | input pairs -> canonical endpoint pair scan in growing output | STRUCTURAL RISK / MONITOR; owners/arguments/unique pairs independent, proof cost may dominate; do not replace canonical pair identity |
| angular_meta/reactive_forms_normalize.rs::merge_unique | scan accumulated strings for each incoming value | STRUCTURAL RISK / MONITOR; field density and repeated merges, exact order and string identity matter |
| angular_meta/reactive_forms_validators.rs::push_rule | repeated validator expressions scan accumulated unique rules | STRUCTURAL RISK / MONITOR; actual validator-list sizes and caller multiplicity unmeasured |
| dotnet_meta/general.rs name extraction | repeated source matches -> growing name Vec membership -> append | STRUCTURAL RISK / MONITOR; actual extraction entry callers and names per input unresolved |
| compression/opcodes.rs primitive opcode lookup | scan fixed vocabulary | Different bounded work unit; not evidence of growing-membership defect |
| workspace/index/membership.rs HashSet contains | contains is a set operation | Not the prohibited Vec-scan pattern; existing finite fitness remains authoritative |

Presence of the idiom alone does not justify extra persistent state or a generalized
index rewrite. Continue caller/type/lifecycle classification of census matches.
Batch-local sets may suit local ordered accumulators; persistent companions require
paired mutation ownership. No shared abstraction is selected for unrelated cases.

## Smallest proposed declaration repair: separate owner decision

Keep declaration Vecs authoritative; add two private companions:

- declaration_occurrence_members: HashMap<EntityKey, HashSet<Option<String>>>.
- declaration_publisher_members: HashMap<String, HashSet<EntityKey>>.

These are distinct from relation endpoint companions: membership in entities does
not establish declaration authority. Only existing self-Defines admission in
add_edges may invoke register_declaration; preserve semantic-role registration
exclusion, exact entity/key/file identity, ordering, cross-publisher behavior and
existing removal semantics. Do not infer declaration provenance from a target or
borrow another companion's membership.

Known writers are register_declaration and remove_declarations_in_file, plus
WorkspaceIndex new/Clone ownership. Recheck all writers before implementation.
Helpers synchronize both Vec and companion at each completed mutation. Removing
an asserting publisher still removes exactly the physical occurrence selected by
current file comparison, preserving other publishers and existing orphan behavior.
Root retirement/replacement continue calling those helpers; no batch deferral.
No serialization change: WorkspaceIndex is not durable serde state; normal
publication/restore reconstructs through existing insertion.

Target expected-linear logical membership probes over declaration participation,
preserving Vec iteration; borrowed lookup before cloning where practical. Report
payload/capacity and transient clones without allocator claims. Establish intended
budget RED against current scans separately from characterization GREEN, then
unchanged budget GREEN at 1/2/4/8/16/64/256, exact Vec/set invariant and declaration-
versus-endpoint tests, publisher removal/readd/clone/root lifecycle, finite AST
writer/scan drift enforcement, and isolated real scan mutation RED/restored GREEN.
Enclosing add_edges counters must separately report unaffected NgRx rebuilds and
edge/structural work. This approves neither new declaration authority nor faster
queries/adjacency; it proposes the same finite representation technique for a
separate measured owner. Await explicit representation disposition before edits.

## Updated priorities and delegated reconstruction gaps

1. Declaration membership: confirmed independent quadratic comparison growth;
   small correction design above; latency significance beyond fixture unmeasured.
2. NgRx rebuild (#173): already measured populated cumulative export reads/parses
   1/3/10/36/136 during growing insertion. Newly traced object_member_value_bindings
   scans all forward adjacency and clones four strings per eligible binding even
   when candidates are empty; each map candidate then loops all bindings. SWITCH
   paths also filter bindings and perform declaration joins. Dimensions include
   unrelated graph edges, candidates, bindings and same-name declaration density.
   Next count those visits and populated empty/duplicate mutations; skipping a
   rebuild might miss changed on-disk exports, so no skip/deferral is approved.
3. Witness/lifecycle: census remains measured work, not proven redundant observations;
   currentness protocol and mutex revision decisions remain separate DESIGN REVIEW.
4. Graph/query: transitive_dependencies_with_scope builds a global node index after
   validating a starting identity, before traversal; unrelated graph size and
   reachable component size differ. ProgramGraph find_node/fan_in/fan_out are linear
   Vec scans; optional ProgramGraphPass builds from instructions, but repeated live
   consumer reachability is unestablished. MONITOR, not measured defect or approval.

## Next gate and independent adversarial review

Affected characterization sources require operator-owned all-feature warning-fatal
Clippy, format check and encoding verification; no agent gates were started.
No broad historical acceptance campaign is warranted.

Owner explicitly requires Agent Popper independently after first-pass enumeration.
That stage has not begun: first-pass source/behavioral coverage is incomplete.
Popper must challenge missing entry points, delegated sinks, independent dimensions,
misclassification, no-op/negative paths and moved work, returning concrete source-
grounded counterexamples or evidence-backed acceptance. MaxHeadRoom agreement or
historical tests cannot substitute. No discovery completion or #174 closure claimed.
