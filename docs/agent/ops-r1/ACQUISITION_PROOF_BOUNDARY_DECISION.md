# OPS-R1 Disposition 10: acquisition-proof boundary decision

Agent: **Agent MaxHeadRoom**. Date: 2026-11-20.
Standing: decision-ready source/API recommendation; no implementation or executable
verification. Disposition 09's approved contract remains controlling.

## Decision

Recommend **Candidate B: scoped acquisition proof export/replay in the existing
owner Resolver/Host**. Record immutable input receipts when inputs are first
consumed, including before program acquisition begins. On a cache hit, attribute
the existing receipt to the acquiring scope. Freeze only that scope's dependency
closure after the existing acquisition succeeds. Share its immutable units and
proof, never the owner's mutable resolver or semantic conclusions.

Candidate A is technically conditional, but not the minimum repair: a fresh
Resolver cannot reconstruct an owner's supplied source, already consumed bytes,
inherited project context and failure state from paths alone. Reacquiring from
disk can select a different snapshot. Giving A an exact read-through input view
requires the same attributable input receipts as B, plus another resolver and
context-transfer protocol. Do not select it as an independent disk acquisition.

This resolves the missing conceptual proof boundary. It does not establish a
compiled implementation or waive the later concrete patch review. The next
engineering deliverable is the bounded patch and tracked tests under this API
design, not another general architecture census.

## Source basis and bounded inspection

Paths below are relative to `src/ir/typescript_resolution/` unless stated otherwise.
Line references describe the currently inspected diagnostic candidate. Exact
source/authority hashes are retained in
`verification/ops-r1/shared-project-design/proof-boundary-source-pins.json`.

| Source | Relevant observed behavior |
|---|---|
| `host.rs:106`, `:155`, `:325` | Host owns sources and cumulative witnesses. Seeded bytes have supplied authority; cached reads return without a fresh backing-source receipt. |
| `host.rs:55`, `:415` | Existing witness validation checks bytes, routes, kinds, absence and directory entries. Host currentness checks the retained witness list. |
| `resolver.rs:44`, `:64`, `:116` | Inherited project context can override per-file discovery. Project values are cached by source; requested explicit project is separate state. |
| `resolver.rs:183` | Parsed Unit cache hits can bypass Host reads entirely. Host-only instrumentation is insufficient. |
| `program.rs:110` | Acquisition consumes project selection, config, inventory, automatic types, libraries, units, source directives and module resolution. Local program cache retains both success and failure. |
| `program.rs:268`, `:280` | Merge groups and augmentation conclusions are added later by owner resolution; they are not immutable acquisition state. |
| `automatic_types.rs:15` | Default type-root lookup begins from the requesting source directory and authorized boundary. |
| `compiler_libraries.rs:105` | Library choice depends on effective config and compiler evidence. Absence of installed compiler does not manufacture library authority. |
| `source_classification.rs:28` | Script/module interpretation depends on units, effective options, compiler defaults and applicable package policy. |
| `capture.rs:146`, `:315` | Owner seeds its private Resolver; final capture retains owner evidence and computes currentness. |
| `project_membership.rs` | Per-source membership/reference selection and explicit versus discovered selection remain authoritative. |
| `src/mcp/tool_handlers/hydration/publication.rs:166` | Existing batch error/source checks precede owner publication; sharing does not replace them. |

Inspection was limited to acquisition/capture boundaries and their immediate
program/currentness/publication callers. No repository-wide audit was launched.
`pcg_knowledge_v1` is unavailable in this session. Applicable earlier OPS-001
acquisition remains retained context, not fresh admission of this decision.

## Candidate comparison by correctness obligation

"Feasible" below means a source-supported implementation strategy, not executed
verification or permission to apply it.

| Obligation | Candidate A: clean acquisition-only Resolver | Candidate B: scoped proof on owner Resolver |
|---|---|---|
| Complete consumed-source proof | Needs exact immutable read-through receipts; a fresh disk read is not equivalent to consumed owner bytes. | Record receipts at original reads; compose the acquisition closure as existing code consumes inputs. |
| Supplied owner bytes | Must import the exact seed and preserve supplied authority; cannot replace it with disk bytes. | Existing seed remains untouched; represent its distinct authority basis explicitly. |
| Cache-hit attribution | Must transfer cached input proof from the owner or reacquire a potentially different snapshot. | Host read hits, parsed Unit hits, project cache hits and inherited context all attribute their original receipts. |
| Project/config authority | Must reproduce explicit/discovered selection, origin-bearing options and per-owner membership. | Retain existing selection; carry its receipt and owner-specific membership qualification separately. |
| Request-dependent lookup | Must transfer source directory, logical/physical routes, boundary and current inherited context. | Existing request context remains in place; compare its immutable lookup basis before reuse. |
| Package/ambient dependencies | Fresh resolution can observe different manifests/types/directories. Exact input view needed. | Capture package/probe/type/library/directive dependencies as actually consumed by existing acquisition. |
| Snapshot consistency | A second resolver may combine later disk values with earlier seed values. | Preserve original immutable bytes; existing witness checks still determine currentness. No atomic filesystem snapshot is claimed. |
| Failure/partial isolation | Additional acquisition may fail where the owner had valid consumed input, or vice versa. | Failed/partial scopes never become shared success; retain existing owner outcome. |
| Owner semantic isolation | Clean resolver avoids earlier owner conclusions, but exporting its whole state is still invalid. | Export only acquisition receipts/units; private scopes, demands, groups, augmentations and recursion guards stay private. |
| Publication/currentness | Requires evidence import without adding unrelated dependencies or changing owner tickets. | Instantiate applicable proof in recipient authority; retain all existing checks and tickets. |
| Extra work | If eager, adds a full acquisition. If lazy and exact read-through, duplicates B's receipt mechanism and adds another resolver/context handoff. | One original acquisition per qualified group; receipt recording and qualification are separately measurable. |

### Candidate A feasibility and blocker

A is feasible only if it takes the exact authoritative input view, not a list of
paths and a copied seed. That view must distinguish prior disk reads, owner seed,
parsed-unit origins, effective project context and failed observations. The
present Host/Resolver API cannot supply it. An eager clean acquisition followed
by the old owner acquisition is rejected: it adds the expensive work rather than
replacing it. A lazy clean acquisition would still require the input receipt
boundary and proof import defined below. It is not the smallest choice.

### Candidate B feasibility and blocker resolution

B does not need a new witness authority or persistence format. It needs internal
attribution for existing authority. Add receipts at original consumption points,
not retrospectively by guessing slices of `host.witnesses`. Unit and project
caches must retain their dependency receipt alongside the cached value. A program
scope records receipt references on both hits and misses. A receipt missing its
dependency basis makes that result ineligible for export; it does not grant
authority or erase an existing owner error.

The necessary change is local bookkeeping at Host/source, Unit, Project and
program-acquisition boundaries. Package/module/type/library acquisition already
reaches those input boundaries during the first acquisition. If implementation
discovers another memoized value that bypasses them, its consumed-input receipt
must be connected or that case must remain independent. Never infer coverage
solely from a nonempty receipt.

## Minimal internal proof interface

The following names describe a concrete proposed API, not compiled declarations:

```rust
// Private construction; no serde/persistence or caller-manufactured success.
struct ProjectAcquisitionProof {
    basis: AcquisitionBasis,
    units: Arc<[AcquiredUnit]>,
    inputs: Arc<AcquisitionInputs>,
    program: AcquiredProgramEvidence,
}
enum AcquisitionAttempt {
    Complete(Arc<ProjectAcquisitionProof>),
    Partial(PartialAcquisition),
    Failed(AcquisitionFailure),
}
enum ReuseDecision<'op> {
    Eligible(QualifiedAcquisition<'op>),
    Independent(ReuseRefusal),
}
fn qualify<'op>(proof: &'op ProjectAcquisitionProof,
    owner: &mut Resolver) -> ReuseDecision<'op>;
```

These are a few direct acquisition types, not a general cache, graph service or
new policy framework. Constructors stay inside the native TS acquisition module.

| Member | Minimum contents and invariant |
|---|---|
| `AcquisitionBasis` | Explicit/discovered project mode; physical/logical config route; ordered root/alias admission; effective options with origins; compiler/config/reference inputs; request lookup basis; acquisition/demand policy. No equality by path/hash alone. |
| `AcquiredUnit` | Immutable parsed unit tied to exact source identity/bytes and its input receipt. No mutable owner merge/augmentation state. |
| `AcquisitionInputs` | Typed consumed inputs and their authority receipts: supplied source, disk content/open identity, route, directory, kind, absence, config and package/type/library observations. Cache-hit dependencies are included transitively. |
| `AcquiredProgramEvidence` | Existing acquisition-owned source inventory, module requests and justified missing-module evidence, directives, types and libraries. Excludes later owner-specific groups/augmentations. |
| Snapshot identity | Exact bytes and authority tuple plus immutable receipt membership. A digest is a transport/index identity, not a substitute for tuple equality or currentness. |
| `QualifiedAcquisition` | Borrowed, operation-lifetime capability created only after recipient checks; carries the recipient-specific proof binding and cannot grant publication authority. |
| Partial/failure | Preserve applicable original error/disposition and observations for the owner. Not eligible for successful sharing and not a new public partial-result contract. |

"Complete" means the existing acquisition completed with a fully accounted input
basis; it does not mean every module resolves, universal semantic completeness or
`source_complete=true`. Existing justified `missing_modules` and lower-bound
public semantics remain valid. No public completeness upgrade is introduced.

## Recording and replay algorithm

1. **Original input receipt.** When Host first reads a source, retain immutable
   bytes and backing Content/open-identity proof separately from the current
   logical-route observation. A seeded source gets supplied-owner provenance,
   not an invented disk Content witness. Failed reads retain their failure basis.
2. **Cached derivations.** Store direct receipt references with Unit parsing,
   project selection and inherited project context. A derived cache hit attributes
   those references to the current program scope without rereading/parsing inputs.
   Root/route/config observations used by those derivations are not discarded.
3. **Actual demand.** Start the acquisition scope at the existing `program_units`
   demand. Its project/context receipt is an explicit input even when selection
   happened before the scope. Use existing acquisition logic in that same owner
   resolver. No speculative/eager program acquisition or second disk resolver.
4. **Freeze.** On successful accounted acquisition, move the scope's immutable
   receipts and acquired units into the proof. Do not scan all owner witnesses,
   reread all sources, reparse units or recrawl the project to build the proof.
   Receipt references are accumulated during actual work; sealing is bounded by
   the collected evidence. Failed/partial results stay owner-local.
5. **Recipient qualification.** Resolve recipient membership/context through its
   existing private path. Compare full authority basis, supplied bytes and all
   overlapping already-consumed values. Validate applicable existing witnesses.
   Conflicting bytes, routes, config, package/native evidence, stale or unknown
   inputs cause independent acquisition; do not overwrite any private cache value.
6. **Attach immutable input view.** Eligible owners borrow units/bytes by reference
   or Arc. Lookups record the corresponding receipt; do not clone the complete
   Unit set into every private cache. Owner algorithms can still do legitimately
   required owner-specific iteration and produce their own semantic results.
7. **Finish privately.** Owner resolution accumulates its own demand results,
   namespace/merge/augmentation conclusions and applicable native evidence.
   Materialize applicable existing witness representation for that owner; keep
   capture-currentness, artifact admission and prepublication/source-ticket checks.

### The cross-owner seeded-source rule

For input `a.ts`, source bytes may be identical but authority differs:

| Requesting owner | Authority for consumed `a.ts` |
|---|---|
| `a.ts` itself | Its supplied source and private source identity/route; disk bytes must not replace that source. |
| Another owner `b.ts` | Applicable dependency content/open/route proof; another owner's supplied source is insufficient. |

If the leader's seed is consumed as a sibling's dependency, obtain existing
disk-backed dependency proof and compare exact acquired bytes before granting
reuse. If the recipient's seed replaces a disk-backed project input, require exact
byte equality while preserving recipient supplied-source provenance. Bind input
receipts by recipient role; do not blindly copy the leader's flat witness vector.
This needs at most the necessary seed-role qualification reads, not another
complete program acquisition. Divergent or unverified seeds stay independent.

Existing whole-owner witnesses/currentness remain on the leader. Do not drop
earlier owner observations to make its proof export smaller. Those observations
are simply excluded from the shared acquisition unless actually consumed there.
Recipient witness identity, ordering and multiplicity must be compared with the
standalone path where they affect retained identity; no implicit deduplication,
certificate rewrite or serialization change is authorized.

## Efficiency accounting and limits

For O owners, K proven-equivalent groups, P_g project units and W_g applicable
witnesses, the proposed mechanism separates these work units:

| Work | Expected responsibility |
|---|---|
| Provisional grouping | Cheap exact request inputs; never establishes authority. |
| Selection/config | Per-owner membership, config and lookup-basis checks as legitimately required; count reads/probes independently. |
| Acquisition inventory | Once per qualified group, at actual demand; no second recursive inventory in proof export or qualification. |
| Parse/program construction | Once per qualified group for shared input units; count new parses, unit visits and group acquisitions separately. |
| Proof recording/sealing | Record actual receipt consumption during acquisition; freeze/move immutable state without another filesystem/program traversal. |
| Source/route qualification | Per-owner source comparisons, overlapping private inputs and route/basis checks; no full Unit-set copy per owner. |
| Witness validation | Preserve existing checks, potentially O(sum owners-in-group * W_g). Measure reads, hashes, metadata and directory enumerations explicitly. |
| Owner semantic work | Still per-owner; include legitimate iteration over shared units, demand analysis, evidence materialization and publication. |

**Important residual cost:** `Witness::Directory::current` calls
`directory_entries`; preserving that check can re-enumerate acquired directories
for each owner. Content currentness also rereads/hashes bytes. Therefore this
repair cannot honestly promise one total filesystem enumeration or O(P) total
operation work. It removes repeated acquisition/construction, not required
validation. Count acquisition-directory enumeration and validation-directory
enumeration separately; never hide the latter as constant-cost qualification.
If those checks dominate, report that remaining cost under AF-PERF-002; do not
silently memoize currentness or change authority in this repair.

Sealing shared units is not a promise that all subsequent owner-specific scans
disappear. The fitness predicate concerns repeated equivalent acquisition;
end-to-end benefit must still include qualification, allocation and output costs.
No time/RSS threshold or claim about the field hang/leak follows from this design.

## Failure/currentness/publication preservation

- Preserve existing local program-error memoization in each Resolver. Do not turn
  that existing local behavior into a cross-owner sticky failure entry.
- Shared storage contains sealed successful proofs only. Unknown/partial/failure
  does not become a group-wide refusal; each owner retains its independent path.
- Two executor invocations have disjoint contexts, including concurrent calls.
  Equivalent owners inside one invocation require one group acquisition owner;
  synchronization details must preserve panic/error isolation and existing worker
  policy. Recursive acquisition must keep the existing recursion refusal rather
  than waiting on its own in-flight group.
- Source/config/package/route changes disqualify reuse or current publication under
  existing predicates. Identical bytes alone do not override changed authority.
- No atomic filesystem snapshot is claimed. Immutable consumed bytes plus existing
  currentness checks establish exactly their existing qualified contract.
- Owner retirement/replacement and publication-ticket supersession still refuse
  as before. Shared evidence is not a publication ticket or retained authority.
- Cancellation, retry and batch prepublication behavior remain unchanged.

## Minimum proposed impact and falsification obligations

Production patch scope is operation-context plumbing in executor/request/capture;
internal input receipts in Host and the existing Unit/Project/program caches;
immutable acquired-program storage and qualified attachment. Necessary direct
program consumers must borrow the new unit view while retaining owner-local
resolution. No new fields on persistent authority, no new Witness variant and no
modification to witness validation truth predicates are proposed.

Required tracked falsifiers before acceptance:

1. Pre-cached source, Unit and Project proof is still present on scope export.
2. Earlier owner-only dependencies are absent from sibling acquisition proof;
   mutating an unrelated earlier owner-only input does not invalidate the sibling.
3. Supplied source authority stays private; equivalent seed/dependency roles share
   only with backing proof, and divergent seeds never share.
4. Different effective config/reference membership, route/roots, package identity,
   request-directory type lookup and inherited project context stay distinct.
5. Failed/partial acquisition and stale proof do not grant success or poison peers.
6. Existing qualified missing-module/lower-bound behavior is unchanged.
7. Owner retirement, currentness and prepublication refusal remain effective.
8. One equivalent group acquires/builds once, while each owner still compiles and
   publishes independently. Currentness work is reported separately.
9. Explicit-file capture and non-TS jobs retain standalone behavior; resolver-only
   tests do not stand in for bootstrap, ordinary hydration and batch controls.

These are requirements for a future tracked patch, not registered/passing tests.
Historical owner-1/4/16 receipts remain successful pre-repair observations with
their original source/binary hashes, not retroactively executed RED tests.

## Scope and governance decision

Recommend selecting B for the concrete proposal under the already approved
contract. Internal receipt attribution is a new implementation mechanism, not
permission to change witness truth, dependency authority or persisted evidence.
If it cannot replay complete recipient-applicable proof without changing those
contracts, return the exact conflicting operation/representation for separate
owner disposition. Do not substitute all-owner witness copying as a fallback.

The remaining gates are concrete source/test patch review and identity pinning,
separately authorized RED/GREEN execution, caller/persistence preservation and
normal graph reconciliation/admission before applicable publication. No protected
authority amendment or B11 grant is generated. This recommendation is associated
with OPS-001 as an alternative, not an owner-approved implementation decision.

No production edits, build, test, CI change, global cache, policy change, grant,
commit, push or publication occurred under Disposition 10.
