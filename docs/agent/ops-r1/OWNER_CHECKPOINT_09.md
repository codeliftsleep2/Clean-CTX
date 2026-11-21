# OPS-R1 Owner Disposition 09: qualification checkpoint

Agent: **Agent MaxHeadRoom**. Date: 2026-11-20.
Standing: architectural contract accepted by owner; implementation withheld.
This checkpoint is **not implementation-ready**. No complete repair patch or
executable fitness regression is represented as finished.

## Accepted contract

Disposition 09 approves operation-local immutable acquisition sharing only after
complete relevant authoritative equivalence is established. Private owner
resolvers, supplied-source authority, failure isolation and publication remain.
Unknown equivalence uses independent acquisition. No global cache, new retry
semantics, mandatory CI change, source application or execution is authorized.

## Concrete qualification algorithm

The algorithm below is a proposal. Step 3's proof extraction is the unresolved
implementation prerequisite described in the next section.

1. Create one compilation-operation context. Build provisional buckets using exact
   authorized root inputs, explicit-project selection mode, request directory and
   source kind, and identical capture demand policy. These are hints only. Do not
   normalize away logical routes, symlink distinctions or authorization boundaries.
   Different request directories initially stay separate because automatic type
   lookup can start from the request directory even under one config.
2. For each job, retain its private Resolver and source seed. Resolve that job's
   actual project through existing selection/membership/config mechanisms. Compare
   selected physical and logical config identity, effective options with origins,
   compiler version, reference/extends closure and route/root admission. This can
   read configuration and lookup witnesses; it must not enumerate the whole
   source program merely to decide bucket membership. Count those operations.
3. On first actual program demand, perform exactly one authoritative acquisition
   for the provisional candidate. Export immutable program inputs and their full
   applicable proofs, not owner-resolved semantic results or publication tickets.
   Do not eagerly acquire projects for owners that never demand a program.
4. Qualify each subsequent job against that acquisition: its own membership proof,
   project/config identity, options/origins and request basis must agree. Its
   supplied owner bytes must match the snapshot's bytes under the exact owner
   authority; reject divergent, unconsumed or unverified overlays from sharing.
   Validate route/authorization observations and acquisition witnesses through
   existing currentness rules. A stale, partial or failed candidate is not success.
5. Import only applicable acquisition bytes, parsed units and proof into the
   private owner capture. Retain owner-local resolution, evidence accumulation,
   canonical source identity and publication. Do not copy another owner's mutable
   merge groups, augmentation conclusions, recursion guards or demand results.
6. On any unknown equality or failed qualification, execute the existing independent
   path for that job. Do not propagate a provisional leader's failure to siblings;
   do not store failures as reusable successes or introduce a retry loop.

Qualification has two distinct costs: per-owner project/route/supplied-byte proof,
and applicable snapshot witness validation. The latter can be O(W) checks per
owner under current rules. The design does **not** claim O(1) qualification or
elimination of O(owners * witnesses) I/O. No witness checks are removed merely
because parsed project units are shared. Complete source enumeration/parsing and
program construction must not be repeated by qualification itself.

## Source-backed limitation: acquisition proof is not currently exportable

Current implementation provides a per-Resolver program cache, not a standalone
immutable acquisition result with exact dependency attribution:

- `host.rs::read` returns immediately on a cached source; it does not replay that
  source's earlier Content/Physical/route witnesses as a newly attributed proof.
- `host.rs::seed` stores supplied bytes without a disk Content witness. That is
  valid owner authority, but it is not proof for another owner's dependency.
- `resolver.rs::unit` can return an already parsed Unit from its private cache.
- `program.rs::program_units` can consume those prior sources/units while recording
  new program observations. Its `Result<Vec<Unit>, String>` is cached locally.
- `capture.rs` performs owner-specific declaration/reference work before exporting
  the complete cumulative host witness list and program evidence.
- Program evidence later gains owner-local merge groups and augmentation proofs.

Consequently, neither candidate extraction shortcut establishes the approved
boundary:

| Shortcut | Why it is insufficient |
|---|---|
| Copy the complete first owner's Resolver/Host state | Includes mutable owner state and potentially owner-only observations/dependencies. Could change sibling currentness or refusal behavior. |
| Copy only witnesses appended during program acquisition | Can omit proof for sources/config/units already consumed through local caches. |
| Copy ProgramEvidence alone | Contains source identities and semantic program evidence, not the complete consumed Host source/probe/route evidence. |
| Clone the whole program/cache into each owner | Does not provide qualified proof and retains complete-set copying per owner. |
| Revalidate the original cumulative list | Can establish that list's currentness; does not establish which evidence legitimately belongs to the shared acquisition. |

The limited source search found no existing scoped acquisition export/replay API.
This is a source finding, not a behavioral reproduction or proof that safe sharing
is impossible. It prevents presenting a reviewed, contract-preserving exact repair
patch at this checkpoint without first resolving proof extraction and import.

Two concrete implementation directions remain to evaluate within the accepted
contract:

1. A clean acquisition-only Resolver, created lazily on first demand, exports only
   program acquisition evidence. Prove that it sees the same source snapshot,
   request project context, native packages, default libraries and automatic-type
   basis as the demanding owner. It must not change an owner's existing qualified
   refusal into success or introduce eager complete-project acquisition.
2. A scoped immutable proof export in the existing Resolver/Host, including proof
   replay on cache hits and supplied-source/dependency distinctions. Preserve
   current persisted witness representations and all owner-local behavior.

Recommend evaluating the clean acquisition-only Resolver first because it can
avoid changing Host observation semantics globally. It is **not yet selected as
verified equivalent**, and no extra Resolver pass should be hidden in work counts.
No new owner authorization requirement is inferred solely from this uncertainty;
the approved contract remains the constraint on further proposal preparation.

## Source/API impact manifest

Expected minimum files, subject to resolution of the prerequisite above:

| Path under src | Proposed responsibility |
|---|---|
| `mcp/semantic_compilation/executor.rs` | Own operation context; retain ordering, parallel owner compilation and panic/error isolation. |
| `mcp/semantic_compilation/compile.rs` | Optional borrowed acquisition input on internal CompilationRequest. |
| `ir/typescript_resolution/capture.rs` | Retain private seed; attach only qualified program acquisition. |
| `ir/typescript_resolution/program.rs` | Separate immutable acquired units/proof from mutable owner program conclusions. |
| `ir/typescript_resolution/resolver.rs` | Private owner state plus immutable acquisition access; no mutable Resolver sharing. |
| `ir/typescript_resolution/host.rs` | Only if scoped proof/import is necessary; do not replace the current witness authority scheme. |
| `mcp/tool_helpers.rs` | Keep standalone requests independent when request fields change. |
| `tests/ir/` and `tests/mcp/` | Deterministic acquisition, negative-control and public-path preservation tests. |

`program_units` has consumers in `ambient_modules.rs`, `ambient_groups.rs` and
`program.rs` itself. A borrowed/Arc unit API requires their reconciliation; changing
the return type without those callers is not a complete patch. Existing owned Unit
consumption must not be silently replaced with shared mutable unit state.

No applicable source patch is supplied: the source/API list is an impact manifest,
not a substitute labeled as an exact implementation diff.

## Failure and invalidation matrix

| Condition | Required outcome |
|---|---|
| First acquisition fails or remains partial | No shared success; preserve each owner's applicable independent result/refusal. |
| Supplied bytes differ or cannot be verified | Keep original owner seed; independent acquisition. |
| Same config path, changed effective options or inheritance | Distinct authority; do not attach prior acquisition. |
| Different root/route, package, source revision or lookup basis | Distinct unless complete applicable equivalence is proved. |
| Source/config/package changes during operation | Existing currentness validation remains controlling; no stale-current promotion. |
| Concurrent executor requests | Separate operation contexts; no cross-request sharing. |
| Owner retirement, replacement or ticket supersession | Existing prepublication/source checks refuse as applicable. |
| Cancellation or retry | Existing behavior; no new policy, retries or retained failure cache. |
| Standalone explicit-file capture | Existing independent path; preserve supplied source authority. |
| Non-TS executor job | Existing compilation path and publication behavior. |

## Fitness and negative-control proposal

Record separately: provisional group candidates; qualification attempts;
qualification config reads, route probes, owner-byte comparisons and witness
checks; successful equivalent groups; complete acquisition attempts/successes;
actual program-unit traversals; owner compilations and publications.

For K established stable equivalent groups, complete acquisition succeeds once
per group and complete traversal equals the sum of group project sizes. This
does not constrain total owner compilation work or replace failure/currentness
assertions with count assertions.

Historical owner-1/4/16 observation counts remain 1/4/16 acquisitions and 1/16/256
unit visits. The future one-project fitness expectation is 1/1/1 acquisitions and
1/4/16 visits, while owner compilation/publication remains 1/4/16. These expected
assertions have **not been executed as RED or GREEN**.

Required tracked controls, proposed identities not yet registered:

- `ops_r1_shared_project_equivalent_sources_acquire_once`.
- `ops_r1_shared_project_divergent_overlay_stays_private`.
- `ops_r1_shared_project_config_and_reference_changes_do_not_share`.
- `ops_r1_shared_project_authorized_routes_remain_distinct`.
- `ops_r1_shared_project_source_and_package_mutation_refuse_stale`.
- `ops_r1_shared_project_failed_and_partial_acquisition_preserve_outcomes`.
- `ops_r1_shared_project_distinct_projects_acquire_independently`.
- `ops_r1_shared_project_owner_retirement_preserves_publication_refusal`.
- `ops_r1_shared_project_standalone_and_non_ts_paths_preserved`.
- `ops_r1_shared_project_does_not_import_other_owner_only_witnesses`.
- `ops_r1_shared_project_replays_required_cache_hit_dependency_proof`.

No complete test patch is supplied while the proof interface remains unresolved.
Fixture helpers, registration and instrumentation need explicit source review;
writing assertion names does not establish test coverage.

## Existing cross-consumer selections and execution plan

Source-selected tracked behaviors to reuse (exact runtime identities/feature
registration must still be reconciled before an execution handoff):

- Resolver: existing `ops_r1_absent_route_observation` and registration lifecycle,
  support and persistence modules; retain nonempty positive registrations as well
  as empty-query coverage.
- Bootstrap: `issue_162_bootstrap_controls_unchanged_caller_dependency_mutation_and_stale_restore`
  and `issue_162_bootstrap_controls_independent_argument_call_and_source_retirement`.
- Project authority: `issue_162_project_reference_selects_owner_and_retarget_invalidates`,
  `issue_162_project_overlapping_references_do_not_choose_first`, and
  `issue_162_project_missing_reference_restoration_revalidates_context`.
- Configuration/explicit capture: `issue_162_capture_retains_effective_options_and_their_source_origins`
  and `issue_162_explicit_context_never_authorizes_foreign_config`.
- Ordinary hydration, C# reconciliation and batch publication: retain existing
  pipeline tests, but precise required runtime selections remain unresolved; no
  blanket cross-consumer protection is claimed.

Proposed sequence after a complete reviewed patch exists: compile the tracked
fitness test with the defective production path, run it once under separately
approved safeguards to establish RED; apply only a separately approved repair;
compile once with all features; run the same fitness and negative controls plus
reused applicable consumer tests. Preserve source/test/binary identities at every
stage. No broad gate duplication; full repository verification remains CI-owned.

Cost evidence: prior compilation-only build took 590.986 seconds. Owner diagnostic
cells took 1.529/1.318/3.125 seconds. A cold test build may again take about ten
minutes; no reliable incremental build or negative-control cost estimate exists
until the patch/test selection is concrete. No timing campaign is proposed.

## Evidence, governance and remaining work

Historical archive and member identities remain in
`verification/ops-r1/shared-project-design/evidence-index.json`. Source/authority
pins for this checkpoint are in `qualification-source-pins.json` beside it.
The historical OPS-001 admission/acquisition receipt remains attributable to its
earlier source revision. This new checkpoint is not freshly graph-admitted.
Connected knowledge tools were not available; no runtime was started to replace
them. Text/hash inspection is not actual document admission.

Outstanding: resolve exact acquisition-proof extraction/import; prepare the actual
minimal repair and tracked-test patches; reconcile full runtime selections and
obtain exact patch identities; then return the completed implementation proposal.
Any later material architectural expansion must be returned to the owner. No
production/source application, build, test, CI change, grant, commit or push
occurred under Disposition 09.
