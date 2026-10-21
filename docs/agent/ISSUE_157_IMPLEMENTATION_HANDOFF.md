# Issue #157: producer provenance repair, static GREEN handoff

Agent: Agent MaxHeadRoom

Date: 2026-10-20

Status: implementation and supplemental regressions authored; **runtime GREEN
is NOT ESTABLISHED**. No Cargo, binary, MCP/provider process or live verification
was executed by the agent. Compilation, Clippy, regression results and the final
repository gate are NOT RUN. No commit or push was made.

## Subsequent operator full-GREEN report and live handoff

The operator subsequently reported **full green**. This supersedes the pending
repository-gate status above as an operator-reported result; no full-gate
transcript was inspected for this report. Earlier failures and NOT RUN statements
below are historical. The agent has started no runtime gate.

The [Issue #157 live MCP harness](../../verification/live-acceptance/issue-157/README.md)
is now authored, covering 27 isolated actual-binary sessions. Operator/live
execution remains NOT RUN. It is separate from tracked Rust/CI coverage and
Claude field acceptance; no issue closure is claimed.

## Frozen RED provenance

### First GREEN attempt: interrupted, not GREEN

Subsequent operator receipt `target/issue-157/green-2.log`: **14 passed, 3 failed**,
47.62 seconds, 4117 filtered out. No compiler warning/error diagnostics were
captured before execution. The frozen deletion/restoration test and supplemental
array-membership and missing-source restoration tests fail at restoration, with
empty membership where a Directive/Pipe is required. Other focused tests passed
in that tested snapshot; this is not overall GREEN.

Static cause: module identity was indexed only through relationships. Retraction
of its last membership erased the module entity, preventing later name queries
from finding its still-recorded footprint. `angular_meta/semantic.rs` now uses
the existing normalized Defines declaration carrier for source-proven NgModule
declarations as well as Services. This retains declaration ownership without
inventing membership or a graph self-edge. The frozen oracle is unchanged.
This follow-up correction is NOT RUN.

The operator reported three frozen lifecycle failures followed by an apparent
hang. The agent inspected the partial green.log: it selects 17 tests, reports
those failures, and has no completed result summary. Individual assertion
details are not yet available from the interrupted quiet run.

Static follow-up found that clearing request-local hydration did not bypass the
session discovery-completion cache. A stale known module owner could therefore
receive no compilation; final dependency validation failed for a single query
and could repeatedly reject a batch without progress. The correction revalidates
only known stale module owners through existing candidate compilation, preserving
their admitted fidelity. Failed revalidation returns an error before the batch
loop can retry unchanged evidence. No discovery cache policy or oracle changed.
hydration.rs reexports that existing mechanism; publication.rs widens its internal
visibility to the tool-handler boundary. This correction is NOT RUN.

The operator receipt `target/issue-157/red.log` established 1 passed / 4 failed,
12.73 seconds, with valid nonempty fresh controls. The initial order test passed;
mutation, deletion, imported-array leaf mutation and module-first batch were
valid behavioral RED. Unreached assertions remained unclassified.

Before production edits the exact regression and its query.rs registration were
stashed together, leaving both absent during the initial producer repair:

- Stash name: `issue-157 frozen RED A5F8C6C90BD0396B`
- Stash commit: `ef0e3664513c724363caf2046ee75387573d8d30`
- Test: `src/tests/mcp/workspace_query_issue_157.rs`
- SHA-256: `A5F8C6C90BD0396B8F12796B56966C24D49345CA865FA8671CE88420CAB906A1`

The exact test and registration were restored. Git checkout converted the test
to CRLF; restoring its original LF representation recovered the same SHA-256.
No test statement or oracle was changed. The frozen test was excluded from
formatting. The stash remains available; supplemental tests use separate files
and registrations. The initial sandbox denied index writes; the required narrow
stash and restore were completed through approved escalation.

## Production changes

Paths are relative to `src/`.

| File | Purpose |
|---|---|
| angular_meta/module_metadata.rs | ModuleProjection carries semantic dependencies; composition and leaf resolution use the same contributor snapshots |
| angular_meta/module_metadata/sources.rs | Bounded composition-local snapshot ownership and negative path witnesses |
| layers/meta/angular_semantic.rs | Collect the footprint from the very projection that produced each module's edges |
| layers/meta/mod.rs | Angular semantic producer generation 26 -> 27 rejects older durable projections with incomplete provenance; canonical/marker generations unchanged |
| mcp/tool_handlers/hydration/publication.rs | Validate artifact dependencies with target hash before committing a candidate |
| mcp/tool_handlers/query/entities.rs | Reject explicit-file publication when compiled contributors have changed |
| mcp/tool_handlers/query/angular_modules.rs | Select scoped module owners for Module forward queries and module-bearing entities_in_file queries; capture their recorded dependencies |
| mcp/tool_handlers/query/prepare.rs | Module contributor validation at final evaluation; request-local invalidation for superseded module preparation |
| mcp/tool_handlers/query/request.rs | Attach module footprint validation and invalidate stale request-local preparation before repreparing |
| workspace/index/coverage.rs | Read-only accessor for an owner's existing semantic dependency list |
| mcp/tool_handlers/query.rs | Register module preparation helper and original/supplemental tests |

## Generalized repair and exact evidence

```text
supported module metadata expression
  -> composition-local Contributors
  -> one byte snapshot per consulted path
  -> parse those bytes and hash those same bytes
  -> ModuleProjection { edges, unresolved_spreads, dependencies }
  -> Angular evaluation -> registry -> compilation artifact
  -> existing indexed/durable dependency storage
  -> currentness validation -> recompose when incompatible
```

Contributors owns a BTreeMap keyed by resolved PathBuf. Each entry retains optional
Arc<str> source text and a SemanticDependency. Successful reads hash the byte
buffer before converting that same buffer to UTF-8 text. Invalid UTF-8 retains
the consumed byte hash but cannot supply declaration evidence. Failed reads
retain a None hash, using the existing absent/unreadable dependency semantics.
Repeated reads of the same path within a module projection reuse its Arc snapshot;
array expansion and bound-leaf resolution cannot independently reread different
versions of that array.

Resolution keeps existing exact-file-before-.ts precedence. The missing exact
path is a negative witness when the .ts fallback is used. If neither exists,
both candidates remain witnesses, allowing appearance/restoration to invalidate
an empty projection. These are resolution dependencies, not invented declaration
authority. The representation uses existing path/hash fields, without schema or
global identity changes. Paths retain the resolver's established filesystem
spelling; no new path canonicalization or workspace-root policy is imposed.

The minimal supported transitive footprint is retained: target -> imported array
source -> imported leaf declaration source. A source not consulted by the
supported resolver is not added merely because it is Angular/TypeScript. Local
facts require no external source footprint. No acquisition frontier was needed.

## Lifecycle and publication

The unchanged target hash alone no longer suffices: has_current_semantic_projection
checks the retained contributor hashes. Changed/deleted contributors invalidate
the target and invoke existing compilation/publication. Empty or unresolved
recompositions retain missing witnesses so restoration is observable. Replacing
the target projection removes obsolete asserted membership before adding current
edges. This uses existing target ownership and does not mutate contributor
authority or force contributor index admission.

Hydration commit now checks target hash and recorded dependency currentness under
its existing publication ticket. Explicit entities_in_file admission also checks
the compiled dependency list before installing the result. The ordinary provide
publication path already checks dependencies; it now receives module provenance.
The dependency currentness validation is shared with existing routing/NgRx
footprints rather than implementing a module-only hash validator.

For explicit Module forward queries and module-bearing entities_in_file queries,
prepared responses retain a copy of their module owners' footprint. Single
evaluation rejects changed contributors with the existing internal retry style.
Batch publication_is_current checks that footprint against surviving source
bytes, as well as any inner existing validator. A stale footprint causes the
existing batch reprepare loop to run; module preparation clears its stale
request-local hydration memo and derived catalogs before calling normal
preparation again. This is compatibility invalidation, not a source-acquisition
frontier or a new global retry budget/policy.

Other query operations keep their current preparation behavior. Ordinary hydration
still validates target footprints whenever those targets are candidates; this
change does not claim a new general final-publication certificate for every graph
operation. Filesystem validation remains an observation at established boundaries,
not a filesystem transaction or immunity to changes after the final check.

## Supplemental tracked tests

Twelve supplemental tests plus the frozen five give **17 authored tests** under
all features. Every new test is under src/tests and registered normally.

`src/tests/mcp/workspace_query_issue_157_dependencies.rs` adds nine tests:

- Array membership removal and restoration.
- Missing leaf appearance plus imported-array deletion/restoration.
- Array membership retargeting through an existing named-import alias.
- Exact-file precedence appearance/deletion while .ts remains unchanged.
- Contributor-first and module-first batches, followed by repeated target query.
- Warm compilation reuse after an unrelated source appears.
- Contributor supersession after compilation but before hydration commit: reject
  the stale candidate, then acquire current Pipe truth through public query.
- Public provide persistence, exact footprint retention, durable validator stale
  rejection, old producer-generation rejection and current-source refresh.
- Existing late-publication test hook changes and publishes a contributor after
  batch preparation; the module must reprepare against surviving Pipe truth.

`src/tests/angular_meta/module_metadata_sources_issue_157.rs` proves repeated
reads retain the original snapshot and original byte hash after disk mutation;
currentness must then fail. This directly checks against later-read certification.

`src/tests/angular_meta/module_metadata_issue_157.rs` adds two bounded authority
checks: type-only/ambiguous bindings gain no edges; nested imported spread stays
unresolved and does not read its unsupported target. Durable reopen in a new
process is not separately exercised; the durable test covers stored data and the
actual compatibility validator. Supplemental tests are authored, NOT RUN.

## Preservation, scope and performance

No support was added for nested imported spreads, re-export/barrel chains,
arbitrary expressions, package resolution, tsconfig mappings or broader Angular
identity. Existing named-import authority and fail-closed behavior remain the
resolver's boundary. Module edge spelling, asserting ownership and completeness
fields are unchanged. The repair does not claim to resolve the broader identity
limitations documented in the investigation.

#152 NgRx, #155 C# publication and #156 EF producer obligations remain separate.
Routing/NgRx dependency lists continue through existing machinery. The shared
publication check now rejects stale supplied dependencies rather than accepting
them. Preservation suites and existing work-count assertions are NOT RUN; no
runtime preservation claim follows from static review.

Within a projection, contributor bytes are read once per path and reused with
Arc clones. Hash validation performs filesystem reads proportional to the exact
footprint, including negative resolution witnesses. Composition still reparses
text using existing syntax helpers; no parser-performance claim is made. Scoped
module response preparation clones and validates its recorded footprint; stale
batch preparation can repeat discovery through existing hydration after its memo
is invalidated. No unconditional workspace parse, compile-all path, global cache
or generic dependency framework was added. Performance remains unmeasured.

## Static verification and operator GREEN

Direct rustfmt with edition 2024 and skip_children=true formatted only modified
Rust files, excluding the frozen RED. This was formatting, not Cargo or a build.
Git whitespace review, strict UTF-8/BOM validation and code-file line counts are
the available static checks. The repository encoding/file-size scripts, full
gates, compilation and runtime GREEN remain NOT RUN.

Run from the repository root in PowerShell 7:

```powershell
New-Item -ItemType Directory -Force target/issue-157 | Out-Null
cargo test --quiet --all-features issue_157_ -- --test-threads=1 *> target/issue-157/green.log
$LASTEXITCODE
Get-Content target/issue-157/green.log -Tail 120
```

Expected authored selection: **17 tests**. This exact command has NOT RUN.
Compilation or fixture failures must be classified separately from behavioral
GREEN. Keep the frozen regression unchanged. After focused GREEN, preservation
and the authoritative final gate remain in [verification.md](verification.md).
No live session or binary rebuild is requested at this stage.
