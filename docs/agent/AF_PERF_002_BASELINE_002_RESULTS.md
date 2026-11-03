# AF-PERF-002: second measured characterization return

Agent: Agent MaxHeadRoom
Status: packet 2 verified GREEN; #174 representation awaits approval; no optimization applied.
Generation 78 remains implemented, targeted GREEN, pushed (owner-reported), with
optional final field/repository certification outstanding. Independent hardening.

## Verified evidence

baseline-packet2-20261103-172310: three characterization tests pass (0.54s), and
one production publication/restore/snapshot/query test passes (18.54s). Zero failed/
ignored. Format log empty, active sizes pass, all receipt source hashes match.
Evidence: target/af-perf-002/packet2-verified-evidence/receipt.json and inspection.json.
Prior failed fixture runs are separately preserved and are not performance RED.
Timing includes instrumented serialization/set work; no uninstrumented speed claim.

## Captured witness multiplicity versus replay

The captured authority stores 1435 witness records with 107 exact typed expectations.
Multiplicities sum to 1435; the maximum identical expectation multiplicity is 332.
This establishes duplicates within the retained vector, independently of later
proof replay. It does not establish whether observations occurred at equivalent
capture times or whether contradictory expected states also need refusal.

The previous lifecycle observation has snapshot validation 7175 checks = five
sweeps of this vector; indexed query 1435 checks = one sweep for this fixture.
These arithmetic correspondences do not substitute for attributing every call to
its exact proof/owner/commit boundary. Next witness design must separate capture
observation history, retained expectations, proof replay and commit/consumption
checks. Do not simply replace 1435 records with 107 or cache by path. Preserve
conflicting typed expectations, source/hash ownership and external-edit behavior.

## Populated NgRx reconstruction

| Inserted callers | Rebuild invocations | Cumulative candidate population / export reads / export parses | Refined edges retired during insertion |
| --- | ---: | ---: | ---: |
| 1 | 1 | 1 | 0 |
| 2 | 2 | 3 | 1 |
| 4 | 4 | 10 | 6 |
| 8 | 8 | 36 | 28 |
| 16 | 16 | 136 | 120 |

Each column is measured in the fixture. Cumulative visits/reads/parses follow
n*(n+1)/2 over tested growth, with n genuine independent caller publications.
The candidate-population counter sums set sizes at rebuild entry, not all internal
loop visits or every syscall. Export counters cover the relocated exports_const/
exports_function helpers; module canonicalization and reexport parser work are
not counted as export reads/parses. Do not equate these with total reconstruction IO.

Removing one caller preserves exactly n-1 dispatches, rebuilds once, rereads/parses
n-1 exports and retires n existing refined edges before rebuilding. Removing the
action declaration retires every remaining dispatch while candidate carriers remain.
At n=16, target retirement visits population 15, retires 15 edges and performs zero
export checks because declaration proof is already absent. These are semantic
preservation controls, not approval to defer reconciliation across public calls.
Populated root-retirement/replacement matrices still need additional coverage;
first packet's empty-candidate root-retirement count remains separate evidence.

## #174 first production design disposition

The measured occurrence/file-map quadratic scans remain the first independent
repair priority. The concrete proposal is AF_PERF_002_INDEX_MEMBERSHIP_DESIGN.md:
three private HashSet companions for name keys, physical occurrence file identity
and asserting-publisher keys, retaining public Vec order and all normal/synthetic
mutation ownership. Expected-linear logical membership work is the target; no
worst-case hash guarantee or semantic identity change is claimed.

Request approval of that exact representation/lifecycle design before implementation.
No production optimization is present. After approval, author scaling-budget RED
against existing production paths, preserve exact tests, then repair. Test growth
must include 64/256 as well as 1/2/4/8/16, duplicate density, cross-publisher ownership,
remove/readd/clone/root retirement and NgRx synthetic add/drop. Static/mutation
coverage must enforce enclosing operations as well as local helpers. Declaration-
index membership, query scans, witness sharing and reconciliation policy stay out
of this first repair unless separately approved.

## Remaining independent architectural work

- Witness: propose exact capture/validation observation protocol before sharing.
- Reconciliation: trace valid mutation batch/query visibility before any deferral.
- Snapshot: existing lock-held measurements stay; revision/pinning/recheck design
  needed before moving IO.
- Structural lookup: instrument exact repeated mapping/facet/entry searches and
  preserve competing-record refusal before proposing lookup companions.
- Coverage drift: blocking new/unclassified expensive paths remains a required
  enforcement design; current printed AST review findings alone do not provide it.

No broad historical campaign, generation advance or resolver-key expansion. No
agent Cargo/test/build/server ran. These four passes characterize existing work;
focused lint/preservation remains required for accepting instrumentation changes.
