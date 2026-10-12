# Independent framework declaration publication repair

## 1. Verdict and scope

**INDEPENDENT FRAMEWORK DECLARATION PUBLICATION SLICE COMPLETE**

Already-recognized Angular Service and modern NgRx Action declarations now
publish independently of relationships. The bounded registered-MCP regression
and live matrix are GREEN. This earns the isolated Service publication criterion
and modern recognized Action publication criterion, not whole #139/#140 closure.

Pinned CBM A0–B5, #136 insertion authority, #137 semantic signature evidence,
#135 DI retention and #139 TypeScript owner implementation remain unchanged.
No legacy extraction, Angular DI, route/module/template semantics, WorkspaceIndex
contract, hydration, query-history policy, optional-provider authority or second
graph was implemented.

## 2. Proven root cause

**PROVEN BY CODE INSPECTION — Angular:** the existing owned class capture reaches
`extract_graph_entries`, which returns the recognized name and ClassKind::Service.
`class_to_semantic_edges` constructs that typed subject but previously only
emitted relationships. An edge-free Service with no injections produced no
semantic record, although its rendered `T @svc` marker and core owner existed.

**PROVEN BY CODE INSPECTION — NgRx:** existing extraction produces
`NgRxShape.actions: Vec<ActionDecl>` with the action creator's name, event string
and already-captured props evidence. Rendering consumes that vector. The old
`to_ngrx_semantic_edges` projected reducer/effect/store/dispatch/select
relationships, but never consumed actions to register their declarations.

```text
Angular class metadata / modern NgRx ActionDecl recognition
  → markers survive the compilation result
  → only unrelated relationships enter semantic_edges
  → edge-free declaration produces no registration record
  → WorkspaceIndex has no framework occurrence at its declaration source
  → registered entities_in_file omits the recognized Service / Action
```

**PROVEN BY TEST:** before production changes, both permanent public regressions
failed their exact-one framework occurrence assertion. Recognition markers were
present; the Service also rendered `C OwnerProbe`. Syntax validation passed and
the source contained no relationship required to publish either declaration.

**PROVEN BY LIVE MCP:** final baseline has 30 PASS / 26 FAIL across 56 groups.
Recognition, existing relationships and supported negative controls passed;
edge-free declaration enumeration, cross-source declaration completeness and
declaration lifecycle publication failed.

## 3. Architecture repair and production path

The existing self-referential `Defines(entity, entity)` carrier already registers
entities without a legacy IR node or dependency edge. `WorkspaceIndex::add_edges`
normalizes it into the existing entity/name/file indexes and never places it in
forward/reverse adjacency, edge_set or file_edges. This mechanic is domain-blind.

- Angular's existing typed projection emits that carrier for recognized
  ClassKind::Service only. Other Angular declaration kinds are unchanged.
- NgRx's existing typed projection emits a carrier for each already-recognized
  ActionDecl, using its written creator name. It does not infer Actions from
  dispatches, reducers, effects or legacy class/string forms.
- The existing MetaLayerPass attaches canonical file provenance.
- Existing compilation/publication code passes semantic records to
  replace_semantic_projection; the existing index performs deduplication and
  file-local replacement/removal. No index implementation changed.

```text
registered provide_code_context / workspace_query entities_in_file
  → existing IRCompiler / default PassPipeline / MetaLayerPass
  → Angular producer-local Service recognition / NgRx ActionDecl vector
  → producer-defined self-Defines registration carrier
  → existing canonical-file provenance attachment
  → existing semantic projection / entity index publication
  → registered entities_in_file and find_entities
```

Relationships retain their existing producers and written identity operands.
The publication core does not interpret decorators, createAction, Injects or
Dispatches. No new ontology, registry, parser or owner interpretation was added.

### Durable compatibility

The existing policy requires a semantic generation increment when identical
Angular producer inputs yield different durable semantic output. Angular and
NgRx share that producer key. Its semantic generation advances **2 → 3**;
marker generation remains **2**. Existing compatibility derivation/validation
rejects pre-publication semantic projections without changing canonical identity.

The new compatibility regression exercises both proving sources against a
generation-2 catalog: semantic identities differ, while canonical identities
and configuration identities are unchanged. The existing durable generation
rejection test also passed. Old-database live migration was **NOT ESTABLISHED**;
no new persistence or acquisition policy was introduced.

## 4. RED → GREEN contracts and matrix

The two permanent proving fixtures are:

```typescript
import { Injectable } from '@angular/core';
@Injectable()
class OwnerProbe {}
```

```typescript
import { createAction } from '@ngrx/store';
export const setAudit = createAction(
 '[Audit] Set Audit'
);
```

The test appends 100 unchanged padding comments to exercise marker delivery,
validates syntax, requests registered provide_code_context at High, then requests
registered workspace_query/entities_in_file with exact filePath/workspaceRoot.
It requires the recognition marker, exactly one expected domain/type/name/file
occurrence, zero outgoing relationships and unchanged source bytes.

Before changing production: **0/2 PASS**, both failed for the missing declaration.
Only those regressions and registration were stashed; the repair was implemented
while they were absent. Restoring the exact tests and repeating the command gave
**2/2 PASS**. Added matrix and compatibility controls finish at **10/10 PASS**.
Formatting changed layout only; the proving fixtures/oracles were not weakened.

| Case | Live RED | Live GREEN | Declaration/source oracle | Public boundary |
|---|---|---|---|---|
| Isolated Angular Service | FAIL | PASS | angular / Service / OwnerProbe at its own file; marker/core owner present | context + entities + forward_edges |
| Service with constructor consumers | PASS control | PASS | One ConsumedService occurrence; duplicate parameter mentions retain one Injects relationship | entities + relationships |
| Two Services in one file | FAIL | PASS | Both exact names once with the same declaring file | entities |
| Same-named Services in two files | FAIL | PASS | One SharedService occurrence at each declaring file, plus separate existing reference-file occurrence | entities + find_entities |
| Isolated modern Action | FAIL | PASS | ngrx / Action / setAudit at its own file; T @action present | context + entities + forward_edges |
| Action with repeated dispatch | PASS control | PASS | One usedAudit occurrence; existing Dispatches fact deduplicated | entities + relationships |
| Two Actions in one file | FAIL | PASS | Both exact creator names once | entities |
| Same-named Actions in two modules | FAIL | PASS | Two sharedAudit declaration-source occurrences plus existing reference-file occurrence | entities + find_entities |
| Add / rename / re-add declarations | FAIL | PASS | Exact current name/file; previous name absent | refresh + entities + find_entities |
| Remove declaration / delete source file | PASS negative control | PASS | No declaration-source occurrence remains | refresh + entities + find_entities |
| Remove declaration while reference remains | FAIL initial publication | PASS | Declaration file disappears; unchanged use file survives with its existing endpoint evidence | refresh + entities + find_entities |
| Unsupported legacy class Action | PASS negative control | PASS | No fabricated ngrx Action declaration | entities |

Live counts are operator verification, separate from tracked Rust regressions.
The corrected final baseline and candidate authored source manifests compare
exactly equal, including the retained-reference sources.

## 5. Declaration versus reference contract

The existing public entity schema exposes **domain, entity_type, name and file**.
It has no declaration/reference role tag or source-span field. The text answer's
occurrence ordinal is not a declaration role, span or globally stable identity.
This repair does not silently introduce those meanings.

**PROVEN BY TEST AND LIVE MCP:** the recognized declaration registers an
occurrence anchored to the declaring file, independently of a consumer. The
consumer's existing Injects/Dispatches relationship remains anchored to the
consumer source, and its endpoint occurrence retains the consumer's provenance.
Two declaration files and one reference file produce three source-distinct
occurrences; the test does not mislabel all three as declarations.

When declaration source is removed and the consumer remains, the declared-file
occurrence disappears and the existing use-file endpoint remains. Public global
lookup can therefore still return a reference after declaration removal; that
is preserved occurrence behavior, not a stale declaration carrier.

Fixture roles in the saved source manifest are independent source oracles, not
invented public fields. A future public declaration-only filter, typed role,
span or cross-file binding contract requires separate owner approval. Broader
declaration/reference linkage remains **NOT ESTABLISHED** by this slice.

## 6. Deduplication and source-distinct identity

The existing semantic identity remains (domain, entity_type, name). Occurrence
identity remains (domain, entity_type, name, file). These are not interchangeable.
The repair does not create global name uniqueness or collapse same-named files.

**PROVEN BY TEST AND LIVE MCP:** repeated publication and multiple relationship
mentions produce one occurrence per identity/file. A self-Defines record plus
Injects/Dispatches endpoints cannot duplicate the same-file occurrence. Forward
queries expose the existing relationships and no registration self-edge.

Repeated identical dispatch sites still represent one deduplicated written
relationship fact, not a physical call-site count. Existing reference occurrences
are retained; no declaration-count claim is made for untyped global occurrences.

## 7. Lifecycle evidence

Both producer families passed registered add, rename, remove-declaration,
re-add and delete-file stages. Each mutation uses the existing refresh_workspace
operation and checks both exact-file enumeration and absence of removed names.
The retained-reference control checks the separate surviving endpoint occurrence.

This establishes source-generation-bound publication through the current
refresh/recompilation lifecycle. No source-cache, refresh, index removal or
query-history implementation changed. It is not #142 cold/warm acceptance.

## 8. Angular boundary

Only already-recognized ClassKind::Service gains an independent carrier.
No extra Component/Directive/Pipe/Module declaration publication is claimed.
Constructor relationship extraction is unchanged and was exercised as a control.
This does not earn #138 DI, providers, inject-token, signals, host/view or Pipe
relationship acceptance.

The former #139 harness asserted Service absence as a publication-separation
control. The new approved slice supersedes that absence. That harness now records
the publication observation while retaining all exact owner/focus/edit oracles;
the new publication harness owns the strict Service existence assertion. Frozen
#139 production and permanent owner regressions were not changed.

## 9. NgRx boundary

Only the existing modern ActionDecl vector drives new publication. Event/props
recognition and rendered markers are preserved, not reinterpreted as new public
payload metadata. Legacy class/string Actions remain unrecognized/unpublished
as Action declarations in the negative control.

Legacy extraction, namespace-qualified action resolution, handlers, switch
reducers, action/effect causal linkage and cross-file binding are not repaired.
The unchanged dispatch/effect/reducer/selector projection is preservation
evidence, not whole #140 acceptance.

## 10. Executed checks

All Cargo checks use the existing cloud Rust environment and the prefix:

```bash
export PATH=/workspace/.tooling/cargo/bin:$PATH
export RUSTUP_HOME=/workspace/.tooling/rustup
export CARGO_HOME=/workspace/.tooling/cargo
cargo test --locked -p clean-ctx --lib --all-features -j 4 FILTER
```

| Filter | Passed |
|---|---:|
| framework_declaration_publication_ | 10 final; proving contract 0/2 RED → 2/2 GREEN |
| angular_meta::semantic_tests | 18 |
| angular_meta::constructor_di_edge_tests | 15 |
| angular_meta::ngrx::tests:: | 25 |
| workspace::index::graph_tests:: | 27 |
| workspace::index::query_tests:: | 26 |
| workspace::index::edge_lifecycle_tests:: | 6 |
| angular_template_compatibility_tests | 1 |
| workspace_query_angular_selector_occurrence_tests | 5 |
| restore_rejects_semantic_producer_generation_mismatch | 1 |
| decorated_owner_identity_ (#139) | 9 |
| insert_before_declaration_ (#136) | 9 |
| semantic_signature_evidence_ (#137) | 5 |
| di_omission_ (#135) | 6 |

That is **10 publication/compatibility regressions and 153 preservation checks**.
The initial plural `workspace_query_angular_selector_occurrences` filter selected
zero tests and supplied no evidence; the actual registration above ran 5/5.

Focused rustfmt ran on the changed Rust files only, using edition 2024 and
`--config skip_children=true`. Package Clippy passed:
`cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings`.
The single authorized all-feature binary build passed:
`cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4`.
No full test suite or CI gate was run.

## 11. Live artifacts, corrections and limitations

All paths below are under `target/framework-declaration-publication/`:

- Final baseline `20261012-114730-0e255310`: **30 PASS / 26 FAIL**, 56 groups.
- Final candidate `20261012-114731-3f903c55`: **56/56 PASS**.
- Baseline SHA256:
  `d05c2115b51dad1fdb4a1927f7ad45811972f0b7aa4a2ee4e14e7afc98b80f60`.
- Candidate SHA256:
  `04301e3194c3850cae42ba75499d1942a11994f385f391f2222a4d05d4b34bb8`.

The live driver reuses PhaseBSession.ps1 for actual stdio MCP sessions, tools/list
admission, bounded response/stderr capture and owned-process cleanup. Provider
is explicitly disabled in owned Git-root fixtures. Both source manifests, raw
requests/results, provenance, lifecycle stages and verdicts are retained. Logs
for the executed Cargo commands are copied into the final candidate evidence.

Earlier captures are retained, not promoted to valid evidence for invalid cases:

- `20261012-113629-4eaa6e49`: initial harness exposed null/empty-array Count
  handling in the removal controls. Corrected without changing expected counts.
- `20261012-113753-789f4032` and `20261012-114051-de0d886a`: preceding baseline
  iterations. Source-distinct lookup cases later proved to send case labels.
- `20261012-114415-7a7f8977`: candidate 54/56. The runner's `$Name` parameter
  shadowed the caller's `$name`; saved requests searched for
  angular-source-distinct-with-reference / ngrx-source-distinct-with-reference
  instead of SharedService / sharedAudit. The fixture lookup variable was
  corrected to targetEntityName and both binaries rerun with the same oracle.

Two-declaration fixtures were also corrected to use one import per symbol;
permanent proving fixtures were unchanged. None of these harness corrections
required production changes or a query-history repair.

The operator command uses PowerShell 7:

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-FrameworkDeclarationPublication.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

On Linux omit .exe. Cloud runs used the installed pwsh under
`/workspace/.tooling/powershell/opt/microsoft/powershell/7/` with XDG cache,
config and data directories under /workspace/.tooling. Baseline runs used the
saved clean-ctx-publication-baseline copy; candidate runs used clean-ctx directly.
The driver builds nothing and invokes neither CBM nor a model host. No Claude/
Codex selection, UI rendering or broad real-workspace acceptance is claimed.

## 12. Changed files and audit

- `src/angular_meta/semantic.rs`: Service-only producer registration record.
- `src/angular_meta/ngrx/shape.rs`: registration from existing ActionDecl records.
- `src/layers/meta/mod.rs`: existing semantic generation increment, no marker change.
- `src/mcp/mod.rs`: feature-gated registered regression module.
- `src/tests/mcp/framework_declaration_publication.rs`: public proving contracts,
  occurrence/deduplication controls, lifecycle and legacy-negative checks.
- `src/mcp/compatibility/mod.rs` and
  `src/tests/mcp/framework_publication_compatibility.rs`: test registration and
  generation-2/current identity comparison, with canonical identity preserved.
- `src/tests/mcp/angular_template_compatibility.rs` and
  `src/tests/mcp/workspace_query_angular_selector_occurrences.rs`: generation
  expectations aligned to semantic 3 / existing marker 2; the prior marker-1
  literal was already stale against the baseline producer. No template behavior
  changed; unrelated template or marker implementation was not altered.
- `verification/issue-reconciliation/Invoke-FrameworkDeclarationPublication.ps1`:
  reusable native live driver and independent fixture/count/source oracles.
- `verification/issue-reconciliation/Invoke-DecoratedOwnerIdentity.ps1`: replace
  the superseded absence assertion with an observation; owner oracles unchanged.
- README, this report and historical report links: current disposition without
  erasing earlier evidence.

Audit confirms producer-local semantics, one existing publication/index path,
unchanged identity/occurrence policy and no dependency on relationships for the
new declarations. No new raw-source parser, durable stale carrier, index mutation
policy or shared framework inference exists. Changed code is UTF-8 without BOM
and stays within the 615-line active-file ceiling.

## 13. Remaining work and stop

Legacy NgRx extraction; Angular DI; action handler/reference linkage; qualified
references; module kinds/spreads; routes; broader declaration/reference roles
and spans; unsupported declaration kinds; #141 HTML entities; #142 acquisition,
hydration and query-history work all retain separate disposition.

No #139/#140 issue was closed. No further production slice is authorized by this
completion. Stop after this bounded publication repair for owner review.
