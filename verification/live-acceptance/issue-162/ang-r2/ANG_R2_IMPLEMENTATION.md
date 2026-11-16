# ANG-R2 Phase B implementation record

Agent: **Agent MaxHeadRoom**. Governance admitted; behavioral RED pending.
No production implementation or generation advancement yet.

## First bounded step: tracked public capability witness

Added src/tests/mcp/angular_resolver_registration_public_issue_162.rs and its normal
test-only path registration in src/mcp/semantic_compilation.rs. Existing key-only
collision and generation-78 tests remain unchanged. No production behavior changes.

Two tests exercise separate cold forward and reverse public workspace_query calls
for two keys targeting the same declaration, requiring exact full registration
records and provenance, truthful lower-bound coverage and distinct omission arrays.
They use only existing production types and the JSON dispatch boundary, so the
baseline can compile and fail behaviorally for the missing operation.

Native ownership, retained joins, exact target and one deduplicated membership are
asserted first in an independent prerequisite state with a different SQLite path.
Each actual query starts in fresh state and verifies its own durable database does
not exist; no owner seeding, direct index mutation or warm-up query is used there.
Expected public records come from exact existing canonical endpoints and retained
mapping proof, not an invented declaration or path-string join.

Required RED command (handed off, **not run by the agent**):

```powershell
cargo test --lib --all-features --quiet issue_162_resolver_registration_public_cold_ -- --test-threads=1 --nocapture
```

Expected selection: exactly two tracked tests. Intended RED is missing public
resolver_registrations capability after successful prerequisite assertions. A
compiler error, failed prerequisite, zero tests or unrelated failure is not valid
RED. The operator wrapper invoke-registration-red.ps1 runs exactly this command,
capturing both logs, duration, exit code and test/registration hashes. Return the
receipt directory for review. Compilation may take several minutes.

Once valid RED is established, freeze/isolate only this regression and its
test-only registration according to docs/agent/architecture.md; retain all authority
changes and mission evidence. Implement the admitted contract, restore identical
witness bytes and establish GREEN under a separately permitted invocation.
No production repair begins merely because tests are authored.

## First RED invocation observed; warning correction requires recapture

Operator capture target/ang-r2/registration-red-20261115-000616-450 selected
exactly two tests: both failed at stage=public-capability with JSON-RPC -32602,
Unknown query type: resolver_registrations. Earlier native/retained prerequisite
assertions passed. Cargo exit 101; enclosing duration 110.5517039 seconds;
test execution reported 12.54 seconds, 0 passed / 2 failed / 5226 filtered out.
This is the intended missing-capability behavior, not a compilation failure.

The agent-authored file also emitted an unused Value import warning. Removed only
that unused import; no fixture, assertion, selector or behavior was weakened.
The first receipt and original SHA-256 remain historical evidence. The corrected
file must receive a fresh warning-free RED capture before freezing exact witness
bytes and beginning production repair. The existing operator wrapper creates a
fresh directory and preserves the prior run. No test was rerun by the agent.

## Corrected RED verified and frozen

Operator capture target/ang-r2/registration-red-20261115-002324-841 has exit 101,
56.8714958 seconds enclosing duration, 22.18 seconds test duration, exactly two
intended public-capability failures and no warnings/compiler errors. Both native
prerequisites passed before Unknown query type: resolver_registrations. Exact
witness SHA-256: 8b047c6e817adb63aa6ba078375258b68ce2b3677e73b88b15f2d283d78821dc.

Only the tracked regression and its test-only registration were isolated in stash
a4bd12ca4df42c4bf34063bc5139fa2d463f76fa. All authority/evidence work remained in
place; no reset/rebase/branch change. The initial sandboxed stash could not write
the index; the narrowly scoped escalated retry succeeded. The regression remains
isolated while implementation proceeds and will be restored byte-exact for GREEN.

## Current implementation batch — shared production and transport

- layers/meta/resolver_registrations.rs defines versioned typed records, exact
  evidence and source-qualified omissions with optional proved associations.
- route_resolvers::project_with_registrations produces both outputs in the same
  traversal before membership target deduplication. Existing project is a
  membership-only wrapper. route_objects::occurrence centralizes unchanged route
  coordinates used by both outputs. Refusals retain source-qualified diagnostics
  without guessed resolver association. Existing mapping proof/P0 calls stay shared.
- MetaLayerOwnedEvaluation is an additive wrapper. Existing MetaLayerEvaluation
  fields, old public registry tuple and existing trait implementors remain valid;
  the provided owned method defaults to their existing evaluation. Angular overrides
  it using the same evaluator. Applicability moved intact into a cohesive module.
- The default registry/MetaLayerPass/PassContext/IRCompiler/artifact path transports
  the collection. Artifact.take_retained_authority transfers it into the existing
  RetainedAuthority payload, alongside neutral TS proof (not inside that neutral
  capture). Existing snapshot/durable JSON transport already carries this owner.
  Legacy into_parts explicitly discards untaken collection for canonical-only use.
- Three existing test payload literals gained only Default completion for the new
  optional field; their assertions and semantic purpose are unchanged.
- Two tracked transport witnesses require actual production snapshot retention,
  exact native mapping evidence, nine registrations versus eight memberships,
  one projector/nine mapping proofs, and local duplicate refusal/sibling preservation.

This batch is **implemented but uncompiled/unverified**. Index retention, complete
registration admission, old/missing payload enforcement, public request/schema/query
integration, full lifecycle coverage and generation advancement remain unfinished.
Semantic generation stays 78 during this partial work; no new capability acceptance
or durable compatibility certification is claimed. Do not publish this intermediate
state as the completed generation-79 contract.

## Bounded operator verification before continuing lifecycle integration

```powershell
cargo test --lib --all-features --quiet issue_162_resolver_registration_transport_ -- --test-threads=1 --nocapture
cargo test --lib --all-features --quiet issue_162_resolver78_ -- --test-threads=1 --nocapture
```

Expected selections: two new producer/transport tests, then six existing membership
control/lifecycle tests. The first verifies the actual changed default pipeline;
the second checks preservation before expanding persistence/admission/index wiring.
Neither establishes public registration GREEN. invoke-registration-transport.ps1
captures both runs and stops on failure, warnings or unexpected selected counts.
These exact invocations are handed off, not started by the agent. Full gates remain
CI-owned; frozen public RED remains isolated until the complete repair is ready.

## Producer/transport batch actually verified

Operator capture registration-transport-20261115-005055-616 passed both selected
commands without warnings: two new tests (136.2871955 seconds enclosing, 24.28
seconds test time), then six existing resolver78 controls/lifecycle tests
(348.1727118 seconds enclosing, 346.97 seconds test time). Transport witness hash
f1647a7e38dfb028e6e8ae9b771c4fa1afd5e4887dab7ecd1084831c0cb7a9c0 matches.
These are actual earlier-stage results; subsequent integration changes below have
not yet been compiled/tested. They are not banked current acceptance.

## Current integrated source — awaiting focused GREEN

Angular semantic generation is now **79**; markers/canonical pipeline and physical
SQLite schema remain unchanged. The existing optional JSON payload carries the
version-1 collection in RetainedAuthority through current/session/durable snapshots,
compatible publication, edit/delta transitions and restore. No SQL table is added.

TypeScriptAuthority.admits_registration_projection reuses the same operation-local
resolver result for membership and complete produced collection validation. Exact
record/omission sets reject duplicates, missing/extra/altered/crossed proof and
unknown versions. A single support set checks matching complete memberships.
Existing TypeScriptAuthority and RetainedAuthority subset APIs retain their old
meaning. Complete production admission is separate, with applicability from the
current configuration; current missing native-route support cannot become empty
success. Old producer identities refuse through existing compatibility checks.

WorkspaceIndex's complete publication accepts a small owned SemanticProjectionInput
within the existing source coverage entry. This is input data, not an admitted
owner or second store. It keeps workspace code independent of MCP snapshot types.
Currentness, exact source ownership and shared semantic proof validate before
replacement. The collection and its admission marker retire with existing coverage.
All five production publication callers now transfer the full collection: compress,
provide, shared hydration, explicit-file entities and compatible restore/publication.
Index code imports no MCP/session type; no extra lock or maintained inverse index.

The new closed resolver_registrations operation validates direction/selectors/root
and shared scope before acquisition. It conservatively nominates eligible TS owners
through existing inventory/selection/High compilation and fails on required
acquisition/publication failure. A request-local preparation result shares that work
between F/R batch items. Final evaluation validates each consumed source footprint
once, then filters produced records; it performs no resolver interpretation.
Selected-route/selected-resolver omissions require proof, while other inspected-source
occurrences are explicitly unassociated candidate_source_omissions. Coverage remains
a lower bound. Deterministic result sorting, exact new input/output schemas and
text/structured/batch selector/record parity are implemented. Old edge responses,
identity/dedup, query envelope and required-name behavior remain unchanged.

The frozen public regression was restored from the retained stash with SHA-256
8b047c6e817adb63aa6ba078375258b68ce2b3677e73b88b15f2d283d78821dc, unchanged.
The same test-only registration is restored alongside the new focused tests.
The stash remains intact. The original membership collision source is untouched.

Current source-tree tests: two frozen public tests, two transport tests and thirteen
new lifecycle/contract/work-count tests. Their full obligation map and limits are
in ANG_R2_LIFECYCLE_VERIFICATION.md. All current-source results are **UNRUN**.

## Exact next operator batch

invoke-registration-green.ps1 runs only these commands, sequentially and fail-closed:

```powershell
cargo test --lib --all-features --quiet issue_162_resolver_registration_ -- --test-threads=1 --nocapture
cargo test --lib --all-features --quiet issue_162_resolver78_ -- --test-threads=1 --nocapture
cargo test --lib --all-features --quiet issue_162_resolver_semantic_key_only_public_collision -- --test-threads=1 --nocapture
```

Expected counts: **17 / 6 / 1**. The first attempts unchanged public RED -> GREEN
and focused new contracts; the others preserve old membership and historical
collision behavior after current admission/generation changes. The wrapper records
the changed/new source manifest, exact commands, witness/log hashes, durations and
exit codes. It rejects source drift, warnings and unexpected selected counts.
This is handed off, not executed by the agent. No current executable, Clippy/format,
full CI, final guarded commit/push or general Angular acceptance is claimed.

## Exact producer scope, without unrelated prerequisites

Actual configured evaluation carries a runtime-only three-state scope alongside the
collection: unbound, checked outside scope, or collection required. The existing
public evaluation shape and registry tuple remain preserved by the additive owned
transport. Unknown scope cannot grant indexed registration eligibility.

After decoding, validate_current_context/validate_historical_context rebind that
serde-skipped scope through the **existing exact Angular applicability predicate**
using the available source/config/native context. This is applicability checking,
not another semantic evaluation or query-time interpreter. The shared projector
still supplies the expected complete registration result once per admission.

A conservative Angular producer-generation hint can include unrelated decorated
TypeScript. Treating that hint alone as actual evaluation would incorrectly require
an Angular collection there. A new source-tree witness checks generic decorated
source acquisition and SQLite restart; it also verifies that an actually produced
empty Routes result cannot be replaced by missing support. No source-completeness
or resolver-target association is inferred from this runtime scope marker.
