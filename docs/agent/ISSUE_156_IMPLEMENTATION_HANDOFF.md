# Issue #156: static implementation and focused GREEN handoff

Agent: Agent MaxHeadRoom

Date: 2026-10-20

Status: **focused GREEN established: 18 passed, 0 failed**, including the three
unchanged original RED tests and the unchanged late-publication regression.
The operator executed the command; the agent inspected its captured output.
The operator subsequently reported the full repository gates green. That is an
operator-reported result; no full-gate transcript was inspected by the agent.
The new tracked [live MCP harness](../../verification/live-acceptance/issue-156/README.md)
is authored and syntax-checked, but its operator/live execution remains NOT RUN.
The original regression currently has a formatting-only working-tree change at
its Path::new call, observed after the operator's gate report and left untouched
by this harness work. Its current hash is
`2FB1BD4E694798876479D76FA6C711D925F09E7449094FE03D1D7AC80189EE4A`;
the earlier exact RED/GREEN hash and stash remain historical acceptance receipts.
No behavioral oracle change was observed in that diff.
Broader preservation, the full repository gate, and any applicable field gate
were outstanding at the focused receipt; the later operator report supersedes
the repository-gate status only. Live acceptance remains outstanding. This is
not issue closure. The agent ran no Cargo test,
check, build, Clippy, Clean-CTX binary, MCP/provider operation, or live harness.

## Completed focused GREEN receipt

`target/issue-156/green-2.log` contains 1377886 bytes at inspection. Its summary
at line 11026 is **18 passed; 0 failed; 0 ignored; 0 measured; 4098 filtered out;
finished in 88.72s**. The subsequent target reports 0 tests, with 11 filtered out.
No compiler warning/error diagnostics were found in this log. This is focused
test evidence, not a full Clippy or repository verification result.

The original regression hash was rechecked:
`46847103F63B45A412BACD6CD90D4DFD8B606B9B4C95FB79BA0931A51ADFCDAB`.
The supplemental lifecycle/late-publication regression hash was also rechecked:
`0B9F0EB62FBAAF46DF60796B06F05744C6891D9ABF287A9D6B0EC61B76169F65`.
Both match their respective observed failures. The root-directory ownership
assertion now passes with both paths canonicalized.

No further focused rerun is required for this unchanged snapshot. The next
operator-owned step is the authoritative [Final Verification Gate](verification.md#final-verification-gate),
which covers broad preservation, zero warnings, formatting and repository guards.
Its command list remains solely in verification.md. Earlier NOT RUN statements
below are historical initial-handoff statements, superseded only for this focused
18-test run. No commit or push has been made.

## First GREEN attempt and static follow-up

Receipt: `target/issue-156/green.log`, 1316671 bytes at inspection; final summary
at line 10606. Compilation completed and the log contained no captured compiler
warning/error diagnostics. This observation applies to that tested snapshot only.

| Supplemental failure | Classification and correction |
|---|---|
| root_order_cannot_admit_a_foreign_entity_identity | FIXTURE/ORACLE DEFECT in path representation: the expected relationship and workspace file were present, but an uncanonicalized response path was compared to a canonical Windows directory. Canonicalize both operands; retain the same directory ownership assertion. |
| batch_order_and_late_publication_use_surviving_truth | VALID supplemental RED: the late hook changes configuration ownership to Second and publishes it, but the final answer still contains First. Repreparation reused an older request-local catalog because publication occurred outside that context's hydrate invalidation path. Bind the C# source catalog to a proven publication generation and invalidate EF reuse when it differs from the surviving generation. |

The lifecycle regression file remains unchanged from its observed failure:
SHA-256 `0B9F0EB62FBAAF46DF60796B06F05744C6891D9ABF287A9D6B0EC61B76169F65`.
The original frozen RED SHA-256 also remains unchanged. No behavioral assertion
in either regression was weakened. The new generation check is applied at EF
catalog consumption; it introduces no global caching policy or durable state.

The correction records a generation only when the index actually owns that
publication, rather than labelling an older index snapshot with an in-flight
authority generation. Both static corrections require the focused rerun below.
Earlier NOT RUN statements in this document describe the initial static handoff;
this receipt supersedes them only for the first 18-test operator execution.

## Frozen RED and preservation procedure

The operator's completed RED run established all three original candidates:
0 passed, 3 failed, 4098 filtered out, 32.41 seconds. See
[the investigation](ISSUE_156_EF_DEPENDENCY_INVESTIGATION.md) for the actual receipt.

Before production edits, the exact test and its query.rs registration were
preserved in the narrow Git stash named
`issue-156 frozen RED 46847103F63B45A4`. The regression was absent during initial
implementation, then restored with its registration for final review. Git's
checkout conversion initially restored CRLF bytes; the test was recovered from
the stash's untracked-file blob using a byte-preserving subprocess read, without
editing its content. The final SHA-256 matches the observed RED exactly:

```text
46847103F63B45A412BACD6CD90D4DFD8B606B9B4C95FB79BA0931A51ADFCDAB
```

The original `workspace_query_issue_156.rs` was excluded from formatting. Its
oracle was not modified. The preservation stash remains available. No commit
or push was made.

## Production files changed

Paths are relative to `src/`.

| File | Purpose |
|---|---|
| ir/csharp_resolution/compilation.rs | Read-only accessor for canonical declaration parts; no resolver replacement |
| mcp/tool_handlers/hydration.rs | Reexport existing inventory/root and frontier-publication mechanics to the coordinator |
| mcp/tool_handlers/hydration/dependencies.rs | Widen internal visibility of the existing batched publication helper; ancestry policy unchanged |
| mcp/tool_handlers/hydration/filesystem.rs | Widen internal visibility of configured_roots; traversal/exclusion policy unchanged |
| mcp/tool_handlers/query.rs | Original and supplemental test registration |
| query/csharp_resolution.rs | Capture declaration scope even without a base list, retain canonical parts, build a current-source dependency view |
| query/csharp_resolution/dependencies.rs | Export the EF-owned obligation adapter internally |
| query/csharp_resolution/dependencies/efcore.rs | Canonical reference resolution/partial composition, current-source projection filtering, existing project convention |
| query/csharp_resolution/dependencies/efcore/model.rs | Relocated existing model-registration parser shared by obligations and projection |
| query/csharp_resolution/dependencies/efcore/obligations.rs | EF requirements, frontier acquisition, internal evidence and single-flight coordination |
| query/csharp_resolution/reference_candidates.rs | Reuse reference indexing for dependency views with separately accounted work |
| query/dependencies.rs | Coordinate ancestry plus applicable EF requirements before final preparation |
| query/prepare.rs | Request-local EF evidence/source eligibility and final validation predicate |
| query/work_metrics.rs | Separate EF dependency inventory/catalog/file/syntax work counters |

In the last eight rows, `query/` abbreviates `mcp/tool_handlers/query/`.

## Generalized repair architecture

```text
forward DbContext / reverse Entity semantic query
  -> existing target and C# ancestry preparation
  -> EF-owned scoped structured obligations
  -> shared authoritative source inventory
  -> conservative candidate hints, never identity proof
  -> shared batched publication + canonical compilation
  -> repeat the structured frontier until stable
  -> canonical EF identity/membership resolution
  -> generation reconciliation and surviving-source inventory check
  -> filtered query-local EF catalog
  -> final publication/project validation
  -> existing lower-bound response
```

Obligation kinds are `ModelEntity`, `ExplicitConfiguration`,
`ConfigurationContract`, `ConfigurationEntity`, and `AssemblyMarker`. Each
retains the context/declaration key, written type shape, namespace/nesting,
imports/aliases, effective WorkspaceScope, and canonical resolver disposition.
Explicit registration and its interface/entity requirements form a connected
frontier. The interface requirement must resolve specifically to the known EF
contract; resolving a source lookalike cannot satisfy membership.

The producer remains beside the existing EF semantic catalog. Shared machinery
owns inventory, source admission, compilation, single-flight and reconciliation.
No new universal capability was added; SourceDependencyClosure still means the
existing C# ancestry/attribute acquisition contract. The EF flight uses a distinct
`ef-model-v1` synchronization key with no ancestry capability target. Followers
rerun their own attempt; a completed flight supplies no semantic proof.

## Identity, scope, and assembly-universe authority

Reference resolution uses the existing neutral explicit/import/alias resolvers
and CSharpSourceSymbolIndex. Class declaration parts come from retained canonical
compilation units, preserving partial metadata and declaration spans. The existing
neutral interface syntax captures remain source-shadow evidence. Partial class
families compose; duplicate non-partial or import-ambiguous targets fail closed.
One stable canonical physical part represents a composed entity in the response.

EF's existing authorized query-scope catalog remains its reference universe.
The adapter does not impose ancestry-shaped project partitions on EF references
or infer project-reference legality. #155 continues to own its existing ancestry
partition rule. Candidate discovery is restricted to authorized scope, and names
only choose candidates for canonical resolution. The root-order fixture uses
distinct namespaces with the same terminal entity name; it does not introduce a
new rule for legal cross-project references.

For the supported local assembly form, marker identity must resolve and share
the context's unique nearest `.csproj`. A complete shared source inventory is
filtered to that existing project convention and the existing source-bearing
IEntityTypeConfiguration contract spelling, including supported alias targets.
All candidate configurations are admitted and validated; their entity references
and declaration families extend the same structured frontier. Enumerating only
previously admitted configurations never closes the universe.

Closure requires complete inventory, successful publication, resolved obligations,
the framework contract, and stable final inventory/publication/project evidence.
Unknown assembly expressions gain no capability. Incomplete inventory, ambiguous
membership, missing references or unprovable project evidence leave proof
unproven and ordinary relationship results lower-bound. No new runtime assembly,
NuGet, project-file Compile-item, reflection, or overload semantics were added.

## Lifecycle, publication, proof and schema

Each attempt observes current source bytes through the shared inventory. The
dependency catalog accepts only admitted snapshots whose cached source and
canonical artifact hashes match that observation. Selected changed files use
the existing publication path. Deleted/changed snapshots cannot contribute EF
model/configuration edges merely because they remain in a session's older index.
EF source eligibility reaches the projection catalog explicitly.

An unchanged context can therefore acquire newly appearing entities/configurations,
retract old configuration ownership, observe entity retargeting, exclude deleted
dependencies and restore relationships. Project witnesses also invalidate reuse
when configurations enter/leave the supported nearest-project universe or marker
project evidence changes. Source-local DbSet written facts retain their separate
existing authority; deleting an entity declaration does not erase source syntax.

Evidence binds the publication observed during obligation derivation, rather than
capturing a newer stamp after unrelated work. A second inventory observation and
project checks reject superseded attempts. Ancestry proofs are reacquired if EF
publication supersedes them. Changed EF source eligibility triggers repreparation
even without publication. The existing PreparedQuery validation boundary also
checks EF project/publication evidence; batch coordination can reprepare rather
than publish a stale prepared answer. Non-EF validators remain trivially true.

Public response fields and registered schemas are unchanged. Existing dependency
proof/status/reason forms express unresolved evidence. Successful internal EF
closure does not promote ordinary edge source_complete or exactness. The missing
entity regression checks emitted status/reasons against the registered schema.

Model/configuration edges remain request-local. They were not made durable index
facts, no persistence ownership model was introduced, and no durable EF model-edge
integration is claimed. #155's retained ancestry footprints remain separate.

## Supplemental tracked regressions

`workspace_query_issue_156_dependencies.rs` and its explicitly registered child
`workspace_query_issue_156_lifecycle.rs` add **15 tests**, alongside the frozen
three: **18 authored focused tests expected** under all features.

Coverage includes aliases and namespace lookalikes; missing entities and schema
compatibility; duplicate/import-ambiguous entities; duplicate configurations;
partial entity/configuration/context composition; root-order scope preservation;
cold reverse queries; warm compile/reconciliation reuse; unrelated-source compile
avoidance; EF single-flight followers; entity appearance/deletion/restoration;
configuration retargeting/deletion/restoration; assembly expansion/contraction,
retargeting and project membership; marker identity/project proof changes; and
reversed batch order with late publication supersession.

Most regressions use public tools dispatch. The follower test prepares its owner
through public dispatch, then uses the production EF adapter and existing flight
test barriers to isolate follower behavior, and verifies the final public answer.
It is supplemental concurrency coverage, not a substitute for the frozen public
production-path RED. No new provider simulator or live test was added.

## Preservation and performance implications

#148 provider neutrality is retained: EF obligations use authoritative inventory,
not advisory provider results. #151 capability/completeness contracts and #155
ancestry/attribute logic remain separate. #153/#154 lowering and identity joins,
DbContext role emission, source-local DbSet extraction, and the other .NET,
Angular and NgRx producers were not rewritten. Existing EF positive, project
lookup, catalog reuse and work-count assertions remain unchanged. Their runtime
preservation is NOT RUN and must not be inferred from this static review.

Frontiers are published in batches and use existing current-source compilation
reuse. There is no unconditional compile-all-workspace path. Request-local EF
evidence can be reused only with compatible inventory, publication and project
observations. A warm query is intended to add no compilation or reconciliation;
the supplemental regression asserts this. Shared inventories still enumerate/read
authorized sources, and new requests rebuild EF dependency views. Those inventory,
syntax and project-lookup costs are real and unmeasured, not claimed free.

Existing projection work counters retain their meaning. Separate dependency
catalog/file/syntax/inventory counters expose added work; each dependency view
build includes its reference index. No measured performance claim is made before
operator execution. Provider-equivalence, other framework preservation, and the
complete repository gate remain operator-owned; use the authoritative
[verification procedure](verification.md).

## Live coverage-schema defect: focused RED captured, repair awaiting GREEN

The operator subsequently ran the focused regression. Inspection of
`target/issue-156/coverage-red.log` confirms the intended assertion at test
line 21: `unresolved EF evidence inserted an absent optional field`.
Result: **0 passed; 1 failed; 4116 filtered out; 0.87s**. This establishes
tracked behavioral RED independently of the live harness.

The regression and its three-line registration were preserved in a narrow
stash named `issue-156 coverage frozen RED regression only`; its parent diff
contains only those two paths. With the regression absent, EF `Evidence::apply`
was changed to use `get_mut(...).and_then(Value::as_array_mut)`, avoiding null
insertion while retaining removal of `dependencies_current` from existing
arrays. The exact regression was restored and matched its frozen Git blob.
No schema, live oracle, or earlier frozen test changed. Focused GREEN and live
acceptance are NOT RUN after this repair. The earlier authoring notes below
describe the sequence before operator RED.

```powershell
cargo test --quiet --all-features issue_156_unresolved_coverage_preserves_optional_requirements -- --test-threads=1 *> target/issue-156/coverage-green.log
$LASTEXITCODE
Get-Content target/issue-156/coverage-green.log -Tail 24
```

Agent MaxHeadRoom inspected the operator receipt
`target/live-acceptance/issue-156/2026-10-21T01-48-18-599Z-10208/report.json`:
**639/644 live checks passed; five failed**. The failures are coverage-schema
checks for direct entity deletion, explicit configuration deletion, ambiguous
entity, missing entity, and changed assembly marker. Each response contains
`evidence_requirements: null`, whereas the registered schema requires an array
when this optional field is present. These are operator/live results, not Rust
test results. Live acceptance remains FAIL.

The source cause is `Evidence::apply` in EF `obligations.rs`: mutable JSON
indexing of the absent optional field inserts null before `as_array_mut` fails.
The intended local correction is a non-inserting `get_mut` lookup; neither the
schema nor the live oracle needs changing.

A separate tracked regression,
`workspace_query_issue_156_coverage.rs`, exercises public tool dispatch for an
unresolved model entity and checks that the optional field remains absent.
It is registered through the supplemental test module. Production code and
the frozen regressions remain unchanged. The new regression is NOT RUN;
architecture.md requires operator-observed focused RED before production repair.

```powershell
cargo test --quiet --all-features issue_156_unresolved_coverage_preserves_optional_requirements -- --test-threads=1 *> target/issue-156/coverage-red.log
$LASTEXITCODE
Get-Content target/issue-156/coverage-red.log -Tail 24
```

No binary rebuild or live rerun is needed for this RED step. After capturing
the intended assertion failure, preserve the exact regression using the
required stash procedure, apply the local correction, and hand off the same
focused command for GREEN before batching the remaining verification.

## Architectural decisions and static checks

The implementation uses the authorized producer/mechanism split and existing EF
project convention. Review specifically avoided imposing ancestry partitions on
EF's reference universe. No global root/project legality, public completeness,
durable ownership, unrelated cache compatibility, feature envelope or canonical
resolver authority was changed. No additional architectural approval was required.

Changed Rust files were formatted directly with rustfmt, edition 2024 and
skip_children=true; the original RED was excluded. This is bounded formatting,
not a Cargo check or compilation. Byte-level UTF-8/BOM checks, active code line
counts and git diff whitespace review are the available static receipts.
Repository encoding/file-size scripts and runtime gates have NOT RUN.

## First focused runtime command: handoff only

Run from the repository root in PowerShell 7 and retain complete output:

```powershell
New-Item -ItemType Directory -Force target/issue-156 | Out-Null
cargo test --all-features issue_156_ -- --nocapture --test-threads=1 *> target/issue-156/green-2.log
$LASTEXITCODE
```

Expected authored selection: **18 tests**. This command has NOT RUN. The existing
three observed RED assertions remain unchanged; the fifteen supplemental tests
are unexecuted. Compile errors or fixture failures must be classified separately
from behavioral GREEN. No runtime authorization is inferred from this handoff.
Claude field testing remains the last-resort gate after local evidence is batched.
