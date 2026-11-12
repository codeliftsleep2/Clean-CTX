# AF-PERF-002: same-name overload retrieval design

Agent: Agent MaxHeadRoom
Status: historical initial FIFO proposal, superseded by owner-approved signature arrays.
Final bounded retrieval is CLOSED / FIX / PROTECTED; see
[completed repair evidence](AF_PERF_002_OVERLOAD_REPAIR.md). The proposal below
retains its original source-level reasoning and unmeasured-design limitations.
Snapshot shortcut, A4, structural-edit, replay and stats repairs remain protected.

## Work unit and existing authority

Default diff_code_context reaches diff_snapshots through the rolling snapshot
helper. Modified/renamed compressible Git diffs also use this comparator, through
a separate acquisition boundary. This design changes neither producer nor caller.
Existing name grouping runs once per method collection through group_by_key,
preserving occurrence order inside each group and sorted name-key traversal.
Reuse those groups: do not rebuild name grouping or change method_key.

In differ.rs::diff_methods, both populated groups allocate base_used/cur_used.
The exact pass visits baseline occurrences in original order and finds the first
unused current occurrence equal by CapturedMethod::eq. Only after ALL baseline
exact searches does the signature pass visit unmatched baseline occurrences in
original order and find the first unused current occurrence with equal sig.
Exact actions precede signature actions. A sole unmatched pair is compared using
the existing signature-change contract; ambiguous remainders remove/add in original
baseline/current order. push_method_comparison determines unchanged/modified and
declaration/sig/markers/body reason priority. All these behaviors remain intact.

Frozen characterization diff-work-20261107-103929 establishes W128 all-body
same-name work: 16384 exact candidate visits/equalities, 8256 signature candidate
visits but only 128 signature equalities. Comment-only W32 has 528 exact visits
but 32 actual equalities. Used-entry prefixes and equality byte work are different.
These are executed original measurements, not a new repair RED or allocator cost.

## Recommended retrieval: two phase-local borrowed FIFO maps

Inside the existing both-populated name-group arm only:

1. Build BTreeMap<ExactKey<'a>, VecDeque<usize>> from current group occurrences,
   appending each original current index once, in ascending occurrence order.
2. Visit baseline in its existing exact-pass order. Look up its ExactKey and
   pop_front the matching queue if nonempty. Mark the same used flags and emit
   the same unchanged action immediately. An empty queue means no exact match.
3. Drop the exact map. Walk current occurrences once in their original order;
   for each unused occurrence append its index to
   BTreeMap<&str, VecDeque<usize>> keyed by the existing complete visible sig.
4. Visit unmatched baseline occurrences in the existing signature-pass order.
   Look up base_method.sig.as_str() and pop_front if present. Mark the same used
   flags and call the existing push_method_comparison immediately.
5. Drop the signature map. Keep the remainder pairing/removal/addition code and
   its ordered used-flag enumeration unchanged.

Do not iterate either retrieval map to emit actions. Do not fuse the two passes:
an earlier unmatched signature must not consume a later baseline's exact match.
Do not remove used flags as incidental cleanup. The new queues implement matching
selection; the existing flags continue to define remainder ownership explicitly.
Neither index survives a name group or borrows beyond immutable input lifetimes.
No synchronization, cache, snapshot payload clone or cross-call reuse is needed.

ExactKey is a PRIVATE borrowed value projection, in the same field order as the
derived CapturedMethod equality:

```rust
type ExactKey<'a> = (
    &'a str, // sig
    Option<(&'a str, &'a [String], &'a [String])>, // declaration: return/attrs/modifiers
    &'a [String], // markers
    Option<&'a str>, // body
);
```

The tuple's existing lexicographic Ord/Eq suffice; no public Ord/Hash derivation
or change to CapturedMethod is required. Keys borrow every string/vector payload.
Preserve None versus Some(empty), element order, repeated attributes/modifiers/
markers, exact string bytes and all producer evidence. No normalization, digest,
signature parser or inferred declaration identity is introduced. An optional
private key constructor can make this projection explicit; it must be guarded
against future equality-field drift by tracked equivalence/static coverage.

## Original-order equivalence proof

Fix one current group C and one exact value v. Its queue initially equals the
ascending sequence of indices j where C[j] == v. Exact matching consumes only
indices from this queue, and no different exact value can consume one of them.
Induct on baseline exact-pass order: the queue head is exactly the lowest unused
current index satisfying original equality. Popping it therefore picks the same
pair and changes the same used bits as the original find. Empty queue and failed
original search are equivalent. Duplicate identical methods remain distinct queue
entries, so repeated baseline occurrences consume them one at a time.

After this pass, both algorithms have identical flags and exact action sequence.
For each signature s, build a queue from exactly the unused current indices with
that signature, in ascending order. During signature matching, only that queue
can consume these entries. Induction on unmatched baseline order gives the same
lowest unused signature match, action, reasons and flags. This proof also covers
multiple nonidentical methods sharing a compact signature and differing producer
evidence: exact matching reserves their exact occurrences before this queue exists.

Finally identical flags yield identical ordered remainder vectors. The sole-pair
rule, remove/add multiplicity and action order are unchanged. Sorted outer name
iteration and class/orphan traversal are untouched. Map ordering is an internal
search mechanism, never output ordering or semantic authority.

Example protecting phase separation: baseline [A(sig=s, body=x), B(sig=s, body=y)]
and current [B, C(sig=s, body=z)]. Original exact phase reserves B for the second
baseline item, then signature phase pairs A with C. A fused per-baseline algorithm
could steal B for A. The proposed two-phase queues cannot do so.

## Construction-inclusive costs and tradeoffs

Let B/C be baseline/current group lengths, K distinct exact values in current,
U unmatched current after exact matching, H unmatched baseline, and T distinct
unmatched signatures. Existing name-group construction, used bit vectors, output
allocations and remainder enumeration remain common costs.

| Work | New cost |
| --- | --- |
| Exact setup | C key projections, C B-tree entry operations, C index appends |
| Exact queries | B key projections and B lookups; at most min(B,C) pops |
| Signature setup | C flag visits, U B-tree entries and U appends |
| Signature queries | H lookups; at most min(H,U) pops |
| Additional retained data | Exact phase C indices/K queues/K borrowed tuple keys; signature phase U indices/T queues/T borrowed str keys |

Exact map drops before signature construction: retain the larger phase footprint,
not the sum, subject to temporary allocator behavior. On a 64-bit target logical
index payload is 8*C bytes in the exact phase, 8*U in signature phase. Real retained
VecDeque capacity can exceed lengths; borrowed keys, B-tree nodes, queue headers,
allocator overhead and output storage are additional. No measured memory total is
claimed. Record actual capacity sums and size_of values on the executed target.

Map operations cost O((C+B) log(K+1) + (U+H) log(T+1)), plus linear setup/flag
visits and pops. Key comparisons are variable-length lexicographic work, not O(1).
Common signature prefixes, declaration vectors and long same-signature bodies can
increase comparison-byte cost. Exact field order short-circuits on sig before body,
matching the current equality's early-rejection opportunity. A zero-cardinality
map still has lookups; formulas are upper bounds, not executed work counts.

In W128 distinct-signature/all-body characterization, expected setup/query calls
are 128 exact entries + 128 exact lookups + 128 signature entries + 128 signature
lookups; also 128 flag visits and 128 signature pops. This replaces the measured
16384+8256 candidate visits, but 512 map operations are NOT 512 constant-cost
comparisons. Measure actual comparator calls, payload inspection and construction
before claiming benefit. Fully identical duplicates have K=1 and cheap queue
selection, but original performs only C successful deep equalities despite its
used-prefix visits; extra construction/deep key comparisons can matter there.
Tiny groups may pay more allocation/setup. No latency or allocation improvement
is established by this design or by original visit counts alone.

Alternative HashMap borrowed exact keys plus queues preserves semantics if keys
use full equality and all collisions are checked. It requires hashing full body/
evidence payload even where sig would reject early, uses randomized hashing and
offers expected rather than deterministic search bounds. Reject digest-only keys.
Sorted borrowed occurrence arrays avoid per-value queue allocations but require
sorting by complete key plus original index, run cursors and signature rebuilding;
they merit cost comparison only if queue footprint proves consequential. A map
of signatures alone narrows many cases but retains quadratic same-signature exact
search and does not satisfy the complete retrieval objective. A name-to-one-method
map loses duplicates and is invalid. Recommend straightforward borrowed B-tree
FIFO retrieval, without thresholds or adaptive algorithms.

## Required acceptance before production authorization/implementation

Owner currently authorizes design investigation only. No production matcher rewrite
or new benchmark command is handed off in this return. The next approval should
bound retrieval to the both-populated arm and leave all matching policy intact.

Before edits, freeze original matching and establish genuine original public work
budget RED with semantic GREEN. Track construction entries, projections, actual
key comparisons and relevant string/sequence comparisons, lookups, queue pushes/
pops/capacities, used-flag visits, retained keys/containers and original predicates.
Keep parsing/capture/output costs distinct from local matching cost. No zero-old-
predicate metric may hide replacement work or key payload inspection.

Require exact selected occurrence-index traces as well as complete ordered
DiffActions (including previous_detail/reason_hint), summaries, every fidelity's
formatted output, and real public responses. Equal-looking output alone cannot
prove duplicate pairing identity. Compare frozen-original/new behavior across
exhaustive small sequences over a finite method alphabet and deterministic larger
fixtures: duplicate exact values, unequal duplicate counts, same signature/different
body or evidence, phase-stealing example, reversed order, inserted overloads,
one-sided/empty groups, sole versus ambiguous remainders, marker-only and evidence-
only changes, None/Some(empty), Unicode, long common prefixes and bodies. Include
all producer equality fields and class/orphan callers. Assert selected producer
evidence in language fixtures rather than assuming language detection succeeded.

Measure complete local setup/query/drop work on independent total-method/group-
width axes, singleton populations, all-exact, all-body, reordered, duplicates,
same-signature density, long-payload and sparse-match cases. Report paired descriptive
timings with symmetric observer overhead and allocator claims only if measured.
Consequential construction regressions or complexity require TRADEOFF REVIEW,
not an unapproved fallback threshold. Preserve public cache/currentness and all
failure/caller boundaries; no Git acquisition change is implied.

Then finite protection against restored scans, isolated real restoration RED with
frozen semantics GREEN, exact restored GREEN, focused existing controls/checks
and mandatory composed acceptance are required. Do not rerun completed shortcut
acceptance solely for this design. No new source census disposition follows.

## Disposition

FIX PROPOSED: private per-group full-equality and then remaining-signature borrowed
FIFO retrieval, with explicit additional local allocation and variable key costs.
Source-level equivalence supports concrete review; construction cost is modeled,
not measured for an unimplemented replacement. Acceptance must decide whether
those costs are minor or require tradeoff review. No production code/test edits,
runtime, stage/commit/push performed for this design return. Census remains
13/9763 scoped, 9750 unresolved plus 96 callback supplements.

```architecture-graph
node: PERF-006-FIFO
kind: MISSION
status: SUPERSEDED
title: Historical initial same-name FIFO retrieval proposal
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_OVERLOAD_RETRIEVAL_DESIGN.md
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
```
