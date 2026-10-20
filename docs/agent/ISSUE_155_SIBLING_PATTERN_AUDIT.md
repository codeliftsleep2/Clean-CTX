# Issue #155: bounded sibling-pattern audit

Agent: Agent MaxHeadRoom

Date: 2026-10-20

Historical audit outcome: completion blocked at the pre-blocker snapshot. The current draft repairs an important query ancestry
path, but the stronger repository-wide elimination claim is not established.
No further production changes were made after the operator requested this audit.
No tests, builds, Clippy, binaries, MCP tools, providers, or live harnesses ran.

Maintainer subsequently accepted this audit and authorized exactly four in-scope
blockers. Their implementation/static regression handoff is now recorded in
`docs/agent/ISSUE_155_FOCUSED_GREEN_HANDOFF.md`. The table below preserves the
original audit evidence; that handoff supersedes its in-scope open status. Excluded
findings remain open in `docs/agent/ISSUE_155_FOLLOW_UP_CAMPAIGNS.md`. No focused
GREEN or repository-wide elimination claim follows from these documentation updates.

## Scope and proof boundary

One bounded static pass searched production `src/**` for semantic producers,
cross-file reads/resolution, hydration/reconciliation, completeness issuance,
acquisition satisfaction/commit, and semantic/cache generation boundaries.
Targeted reads covered built-in, Angular/NgRx, .NET and Spring producers; native
source relationship projection; query preparation/batching; C# reconciliation;
semantic snapshots/persistence compatibility; and provider/discovery caches.
Relevant existing test setup was read, not executed. This is not an unrelated
cleanup or a runtime reproduction campaign.

SAFE means safe for the stated narrow authority contract on inspected paths;
it does not certify every framework idiom or race. SUSCEPTIBLE identifies a
source-supported missing obligation/validation boundary; it is not a claim that
a newly authored test has failed. All runtime verification remains NOT RUN.

## Audit table

| Location / mechanism | Dependency/proof involved | Order-sensitive? | Classification | Covered by #155 repair? | Action |
|---|---|---|---|---|---|
| `ir/csharp_resolution/compilation/dependencies.rs`; query dependency coordinator | Owner identity, declaration family, bases, aliases, current ancestry disposition | Yes, pre-fix | COVERED BY #155 | Draft query path; unverified | Preserve original RED; run focused GREEN only after blockers below |
| `query/edges.rs`, `query/entities.rs`: former Controller base loops | Manually acquired project ancestry | Yes, pre-fix | COVERED BY #155 | Loops removed; shared query path replaces them | Keep Controller forward/file equivalence regressions |
| `compilation.rs:188-255`, `attributes.rs`; canonical attribute resolution | Class/method attribute declarations, suffix forms and source shadows | Yes, susceptible | SUSCEPTIBLE | No; current frontier follows bases/owner families, not attribute references | Add scoped cold/warm shadow/ambiguity regression; resolve dependency-acquisition scope without reopening #153/#154 identity correlation |
| `query/csharp_resolution/dependencies/efcore.rs:24-81` | Model entity declarations, explicit configurations, assembly marker/configuration universe | Yes, susceptible | SUSCEPTIBLE | Ancestry authority only; model dependencies remain outside closure | Record separate supported-dependency finding; do not redesign EF ownership or add capability silently |
| `core/provide/evaluate.rs:200-233`; `core/compress.rs`; `durable_semantics.rs:349` | Cached canonical/context data and semantic snapshots | Yes, susceptible | SUSCEPTIBLE | No; coordinator is called from workspace query preparation | Add same-source cold/warm context-consumer regression; trace common acquisition/publication boundary before claiming lifecycle integration |
| `csharp_calls/reconciliation.rs:130-165`; `compatibility/validator.rs:240` | Reprojected ancestry facts stored with artifact-owned dependencies | Yes, susceptible | SUSCEPTIBLE | Query proof checks generation, but snapshot dependency lists still omit newly consulted ancestry sources | Preserve canonical/source distinction; persist or validate the required dependency footprint at the actual consumer boundary |
| `angular_meta/module_metadata.rs:76-167`; Angular semantic output | Imported/spread arrays and referenced declarations read from disk | Mutation/cache susceptible | SUSCEPTIBLE | No; closure adapter is C# ancestry | Separate backlog: external reads are not added to the module producer dependency list |
| `ngrx/object_actions.rs:39-109`; `object_registration.rs:19`; `index/ngrx_refinement.rs:39` | Imports plus dispatch, reducer-map, registration and declaration evidence | Yes, susceptible at acquisition/proof boundary | SUSCEPTIBLE | No; explicit carriers/rebuild/retraction help, but are not shared closure satisfaction | Preserve #152 tests; backlog cross-file authority-neighborhood proof and transitive/reexport invalidation |
| `query/entities.rs:237-366`; `coverage.rs::current_file` | Non-C# file certificate after local compile/current projection; optional NgRx omissions | Later refinement can change facts | SUSCEPTIBLE | Only affected C# queries get new proof decoration | General file semantic certificate needs explicit producer obligations/final evidence; do not treat omission labels as dependency closure |
| `query/angular_templates.rs:107-136,195-216`; `edges.rs::apply_angular_template_metadata` | Selector candidate uniqueness; template availability; `discovery_complete` | Susceptible if evidence is subsequently expanded | SUSCEPTIBLE | No | Backlog: bind exact certification to authoritative candidate/source proof and final publication, rather than discovery boolean plus availability |
| `query/batch.rs:93-114`; request-local preparation caches | Captured catalogs/certificates followed by later acquisition/publication | Structurally susceptible | SUSCEPTIBLE | C# proof downgrade detects changed generation; no producer-neutral stabilization barrier | Add in-scope mixed/multi-owner stabilization regression; design shared final-proof validation before broader implementation |
| `hydration/acquisition.rs:62`; `execution.rs:186-217`; acquisition completion cache | Inventory, current High projections, compatible producer/admission/root generations | Safe only for current source-only capabilities | SAFE | New dependency capability uses its own structured path and adds C# publication matching | Preserve #148/#151; do not generalize source-only satisfaction to arbitrary semantic closure |
| `index/edges.rs:236`; declaration certificates; incoming TypeRef proof | Source declarations exclude semantic-role registration; incoming written-reference projections are source-true | No dependency on ancestry for these claims | SAFE | Intentionally retain separate source-only contracts | Keep source declaration and written TypeRef preservation suites |
| `query/classification.rs::neutral_targets_for_forward`; neutral base hydration | Full written base-list declarations, including interface refinement | Specialized acquisition remains | UNRELATED | First-class ancestry closure does not replace all interface/type declaration requirements | Retain until equivalent dedicated obligations are regression-protected; no mechanical deletion |
| `query/dotnet_testing.rs`; TestClass target hydration in edges.rs | Source-proven test subject and unique declaration lookup | Specialized acquisition; separate identity risk | UNRELATED | Ancestry closure cannot substitute for subject declaration acquisition | Keep; record terminal-name/ownership risk separately, as already excluded by #153/#154 scope |
| `routing/structured.rs:247-280`; recorded SemanticDependency checks | Direct lazy-route target source/hash, cache invalidation | Direct dependency represented | SAFE | Separate existing mechanism | Preserve #139; do not infer a proof for unrecorded module/import dependencies from this path |
| Built-in/native written projections; source-local Spring/Angular helpers | Source declaration/signature/syntax facts within declared producer envelope | No admitted-workspace dependency found in inspected local emitters | SAFE | No ancestry workaround needed | No change; framework-name identity concerns are a separate scope |
| `hydration_flight.rs::run`; discovery and CBM caches | Synchronization/advisory discovery, not semantic completion | Followers rerun attempts | SAFE | New ancestry path uses registry; own proof revalidated after wake-up | Preserve HYDRATION-SINGLE-FLIGHT-001; do not promote provider/cache completion to semantic proof |

Paths in the table are relative to `src/`. SAFE rows concern the inspected contract,
not a blanket endorsement of every caller's stronger completeness claim.

## A: unrepresented dependencies

Material susceptible families found: C# attribute authority/shadow declarations;
EF model/configuration ownership evidence; C# context/snapshot ancestry footprints;
Angular imported module metadata; NgRx nonlocal authority neighborhoods; Angular
selector candidate universes. The new adapter's `ancestry_obligations` walks bases,
owner declaration families and aliases; it does not emit all these requirements.

For the attribute case, canonical compilation already resolves class and method
attributes against the supplied source-symbol index. Acquiring a same-name source
attribute later can change that result. The new ancestry frontier does not search
those attribute names/suffixes. This is a missing evidence obligation, not the
#153/#154 declaration-to-method identity repair.

EF's query catalog iterates currently admitted declaration sources and skips a
model/configuration target when `resolve_written_class` cannot find it. Its
existing tests publish `Entities.cs`, configuration, and context files explicitly.
Neither that setup nor a closed DbContext base chain proves cold acquisition of
the explicit configuration or assembly configuration universe. Returned edge
coverage remains a lower bound, which avoids authoritative absence but does not
meet available-source query-order convergence by itself.

Angular module metadata reads imported arrays and declaration files directly;
the Angular evaluation dependency list inspected here is assembled from routing
dependencies and NgRx direct imports, not those module reads. A current target
hash therefore cannot prove that imported metadata is still current.

NgRx intercepts evidence carriers, requires declaration/registration/dispatch
authority, and rebuilds/retracts refined facts on index changes. Those are important
positive safeguards. Direct-relative dependency capture and name-based hydration
still are not a shared proof that the required reverse authority neighborhood or
reexport chain is closed. Missing direct module paths can also disappear from
the recorded list because path resolution uses successful canonicalization.

## B/C: issuance and intermediate-state publication

The positive issuance sites found by repository-wide term search are the
CompletenessCertificate constructors/projector; Angular template coverage
promotion; acquisition record-after-publication/finish; hydration completion-cache
commit; and the new dependency proof issuance. Ordinary edge/graph/occurrence
coverage stays explicitly lower-bound and does not certify absence.

Source-only certificates are materially different: semantic-role registration is
excluded from source declaration registration at `index/edges.rs:236-238`, and
incoming TypeRef proof concerns written references. Their current owner-source
and producer contracts do not require every derived framework role to exist.

Generic batch preparation performs one union pass and one index-backed reprepare
pass, then evaluates under one final index guard. A common final read guard does
not retroactively validate every previously captured catalog or certificate.
There is no producer-neutral fixed-point check or final dependency-proof validator
covering all the issuers above. The new C# proof validates publication again at
evaluation, which prevents false C# completeness after supersession; it is not
the generalized acquisition/stabilization solution for every producer.

The C# generation reconciler itself builds all projections before replacing them
under one index lock and commits capability coverage after replacement. That
publication ordering is sound locally. Its remembered/durable dependency footprint
is nevertheless cloned from original artifacts, whose dotnet dependencies can be
empty. Correct publication ordering cannot repair an unrepresented dependency.

## D: bespoke preparation inventory

Obsolete ancestry compensation removed in the draft: Controller walks in edge and
occurrence query preparation and `aspnet_base_candidate`.

Remaining specialized paths: neutral base-list target hydration; TestClass subject
hydration; Angular template-owner and selector hydration; NgRx object-action and
registration hydration terms; exact C# invocation preparation. They have distinct
semantics and cannot all be removed by a class-base adapter. Several still need
explicit shared dependency obligations. Exact invocation reconciliation is an
existing canonical consumer, not evidence that ancestry acquisition has run.

No remaining Controller base walk was found. It would be incorrect to confirm
that no remaining bespoke path compensates for missing shared obligations.

## E/F: negatives and caches

Ordinary `identity_not_indexed`, empty edge sets, and occurrence results retain
lower-bound metadata. They are not verified absence. Selector uniqueness after
discovery and broad file exact certificates are the stronger claims requiring
attention. NgRx omission metadata narrows the producer envelope but cannot prove
that the remaining available evidence neighborhood has been acquired.

Source cache entries validate their documented source observations; discovery/CBM
caches are advisory; source-only acquisition keys bind producer/admission/root
generations. Request-local catalogs invalidate on recorded publication activity.
These mechanisms cannot validate missing dependencies. In particular,
`semantic_snapshot` and durable validation call `all(dependency.is_current())`:
an empty ancestry footprint passes regardless of a later base publication/change.
The new query proof key adds owner hash, partition and C# publication generation,
but that protection has not been propagated to non-query context consumers.

## Required additional work inside #155

Before calling the repair generalized or implementation-complete:

1. Author unchanged-source attribute shadow/ambiguity cold/warm regressions for
   applicable Controller/TestClass facts. Decide the supported obligation boundary
   from canonical inputs; do not alter declaration-identity correlation.
2. Author a non-query `provide_code_context`/snapshot cold-warm and base
   mutation/retraction regression. Protect retained and durable dependency state
   at the shared production consumer boundary, not only in workspace query output.
3. Author a multi-owner/mixed-item final-publication regression that forces later
   preparation to change evidence relevant to an earlier certificate. Specify a
   shared final validation/stabilization boundary rather than a query reordering.
4. Align the draft's new `semantic_dependencies_unresolved` status/proof metadata
   with the registered response schema in `mcp/tools.rs:470`. The enum currently
   lacks that status; its reasons schema also expects objects while the draft emits
   strings. Resolve the contract deliberately inside the authorized truthful
   completeness change; do not leave schema and implementation inconsistent.

EF model/configuration acquisition, Angular module dependencies, NgRx authority
closure, Angular selector proof, and existing TestClass terminal-name ownership
risks are recorded separately. No unrelated implementation was changed by this
audit. Extending the producer adapters across those systems requires a reviewed
scope/architecture decision; do not silently label them fixed by #155.

## Decision point and limits

The required decision is whether #155 closes only the specified C# dependency
campaign with explicit sibling follow-ups, or also migrates non-C# producer
obligations and all exact issuers onto a common final evidence boundary. The latter
affects Angular/NgRx certificates, shared batch preparation, cache contracts and
non-query context consumers. This audit authorizes investigation, not unrelated
producer redesign. Recommended sequencing: finish the in-scope C# lifecycle and
final-validation gaps, keep sibling findings separate, and explicitly qualify the
repository-wide elimination claim until their own regressions/repairs land.

Requested confirmation 5 is NOT established: other issuance/consumer paths were
found that can rely on incomplete or subsequently changed dependency evidence.
Requested confirmation 6 is NOT established: specialized preparation remains and
the ancestry adapter does not represent every dependency it acquires.

The original exact focused command remains the first runtime gate after the
in-scope audit blockers are resolved. It has not been run after the draft repair:

```powershell
cargo test --all-features tests_issue_155 -- --nocapture
```

No runtime sibling finding, focused GREEN, preservation success, CI result, or
repository-wide absence of this defect class is claimed by this static audit.
