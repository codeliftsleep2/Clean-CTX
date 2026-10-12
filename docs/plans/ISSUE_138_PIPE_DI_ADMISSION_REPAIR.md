# #138 precise Angular DI: Pipe constructor admission

## 1. Verdict

**#138 PRECISE ANGULAR DI RELATIONSHIP SLICE PARTIALLY COMPLETE**

One coherent repair is complete: Pipe constructor dependencies now reach public
Injects relationships. Functional inject(...) token interpretation requires a
separate authority change and remains unmodified. This is the handoff's explicit
first-boundary stop, not whole-issue acceptance.

CBM A0–B5, #136 insertion, #137 signatures, #135 DI retention, #139 owner identity
and independent framework declaration publication retain their implementations.
No NgRx, template, .NET, acquisition, query-history or index policy changed.

## 2. Patch-boundary decision

**PROVEN BY CODE INSPECTION:** both paths ultimately reach Angular's
class_to_semantic_edges, but their first authority losses differ:

| Path | Existing evidence | First loss | Required repair |
|---|---|---|---|
| Constructor DI | Existing constructor producer returns parameter type names | ClassKind gate excludes Pipe | Admit supported constructor evidence for Pipe |
| Functional inject(...) | collect_signal_fields stores kind, receiving name and alias | Token argument role is discarded; sf.name enters injects | Consume structured callee/argument/owner evidence in a separate producer interpretation |

Sharing the final vector/projector does not make the missing authorities the
same. A kind gate cannot recover an argument token from a receiving field.
The task therefore completes the first bounded Pipe repair and stops.

There is a necessary guard: extract_graph_entries merges constructor types with
the flawed functional field-name entries. Blindly admitting that whole vector
for Pipe would introduce incorrect functional-DI edges. The new Pipe path uses
only the existing constructor producer. Existing Component/Service/Directive
input handling is unchanged. No functional token is repaired or inferred here.

The compiler already has source-bound InvocationOccurrence callee/argument
spans; production MetaLayerContext carries an invocation view. The current
Angular signal-field interpreter does not consume those roles for DI.
**PROPOSAL, not implementation:** a subsequent slice should connect that
structured evidence to Angular-local token/consumer interpretation. This report
does not claim a generic parser lacks argument capture or implement a new parser.

## 3. Proven Pipe root cause

```text
owned @Pipe class capture and typed constructor parameter
  → extract_constructor_injects / extract_graph_entries
  → dependency evidence = AuditService
  → class_to_semantic_edges admits Component / Service / Directive only
  → Pipe emits its existing name relationship but no Injects
  → registered forward_edges for angular / Pipe / AuditConsumer omits AuditService
```

**PROVEN BY CODE INSPECTION:** the old explicit three-kind condition is the
admission loss. The existing constructor extractor identifies the parameter
type; private/public/readonly modifiers and parameter names are not its target.

**PROVEN BY TEST:** before production changes, the permanent registered test
checks syntax, correct owner, constructor evidence, both dependency declarations
and the Pipe occurrence. Only the exact admitted dependency count fails (0/1).
The expected receiver is completelyDifferentName and dependency is AuditService.

**PROVEN BY LIVE MCP:** baseline has the constructor marker and recognized Pipe
but lacks its Injects edge. Entity presence is checked separately from edges;
PipeName Defines is not counted as DI.

## 4. Repair architecture

```text
existing constructor parameter producer
  → supported written dependency type names
  → Angular-local Pipe admission (constructor channel only)
  → angular / Pipe / consumer ──Injects──> angular / Service / dependency
  → existing file-provenance attachment and semantic projection
  → existing domain-blind index/deduplication
  → actual registered public edge query
```

class_to_semantic_edges reuses extract_constructor_injects for newly admitted
Pipes rather than consuming the contaminated merged functional vector. This is
an additional invocation of the existing helper for Pipe, not a second extractor
implementation or TypeScript parse. No receiver, rendered marker or entity
lookup supplies a target. Existing extraction/type eligibility is not redesigned.

The admitted kinds are Component, Service, Directive and Pipe. Module and plain
class consumers remain outside this Angular DI admission contract. Shared
graph/index code is unchanged and knows nothing about Pipe or inject syntax.

Semantic producer generation advances **3 → 4** under the existing durable
compatibility policy. Angular marker generation remains **2**; canonical identity
and configurations remain unchanged. The generation regression compares current
Pipe identities against the pre-gate generation-3 catalog. Existing persisted
generation-rejection coverage passed; an old-database live migration was not run.

## 5. RED → GREEN contracts and matrix

The permanent proving source contains independently declared AuditService and
LoggerService plus this differently named Pipe consumer:

```typescript
@Pipe({name:'audit'})
export class AuditConsumer {
  constructor(private readonly completelyDifferentName: AuditService) {}
  transform(value: string): string { return value; }
}
```

Using the registered tools, the test requests High context and entities_in_file,
then forward_edges with explicit workspaceRoot and exact
angular / Pipe / AuditConsumer identity. It filters Injects, requires exact
target names/counts and checks source/target kind and asserting-file provenance.
It repeats the query to guard deduplication and verifies source was not changed.

The proving regression returned **0/1 RED**. Only its file and module registration
were stashed; the fix was implemented while the regression was absent. Restoring
the exact regression and repeating the command returned **1/1 GREEN**. Subsequent
controls extended the helper while preserving the original proving fixture and
oracle. Final Pipe/compatibility filter: **9/9 PASS**.

| Consumer / form | Live RED | Live GREEN | Dependency oracle | MCP boundary |
|---|---|---|---|---|
| Service constructor | PASS control | PASS | AuditService, not parameter name | context, entities, exact forward_edges |
| Component constructor | PASS control | PASS | AuditService | same |
| Directive constructor | PASS control | PASS | AuditService | same |
| Pipe private readonly / different receiver | FAIL | PASS | AuditService | same |
| Pipe two dependencies | FAIL | PASS | AuditService and LoggerService once each | same |
| Pipe two receivers / same type | FAIL | PASS | One deduplicated AuditService relationship | same |
| Pipe bare/public/readonly parameters | FAIL | PASS | Same AuditService identity | same |
| Multiline Pipe metadata/parameter layout | FAIL | PASS | AuditService | same |
| Pipe empty constructor / no constructor | PASS control | PASS | No fabricated Injects | same |
| Pipe functional-only field | PASS exclusion control | PASS | No newly admitted functional dependency | exact forward_edges |
| Pipe constructor + functional field | FAIL constructor edge | PASS | AuditService only; flawed field-name evidence excluded | exact forward_edges |
| Module/plain class constructors | PASS exclusion controls | PASS | No expanded Angular-kind admission | exact forward_edges |
| Pipe unresolved written dependency | FAIL | PASS | MissingService symbolic edge; no resolved-declaration claim | exact forward_edges |
| Two same-named declarations / importing Pipe | FAIL edge | PASS symbolic edge | CollisionService; import binding NOT ESTABLISHED | entities + exact forward_edges |
| Refresh AuditService → LoggerService | FAIL | PASS | Old dependency gone, exact new dependency | refresh + exact forward_edges |
| Remove DI / delete source | PASS absence controls | PASS | No retained Injects | refresh + exact forward_edges |

## 6. Deferred receiver/token defect: actual observations

The live driver captures these outside its PASS/FAIL acceptance matrix.
Baseline and candidate observation JSONs compare exactly equal:

| Service property syntax | Written token | Actual Injects targets | Status |
|---|---|---|---|
| completelyDifferentName = inject(AuditService) | AuditService | ? | Not repaired |
| completelyDifferentName=inject(AuditService) | AuditService | completelyDifferentName | Not repaired |
| completelyDifferentName = inject?.(AuditService) | AuditService | none | Not repaired / no supported optional-call claim |
| receiver = inject(SomeNamespace.Token) | SomeNamespace.Token | ? | Not repaired |

**PROVEN BY CODE INSPECTION:** SignalField has kind/name/alias, no retained token
argument role. collect_signal_fields scans assignments, consumes argument text
for alias handling, and walks backward to the field name. Whitespace before '='
can produce the '?' fallback. extract_graph_entries then pushes sf.name into
injects. Rendered markers have the same problematic functional interpretation.

**PROVEN BY LIVE MCP:** the table reports actual public results rather than
forcing the anticipated 'audit' example. This is not a functional-DI GREEN claim.
No token-registry, expression parser, namespace normalization, optional-call
support, import binding or field-name substitution rule was implemented.

## 7. Framework-kind admission and publication separation

Existing Service/Component/Directive constructor behavior remains GREEN. Pipe
now follows the same supported written-constructor dependency relationship model.
The only new channel is Pipe constructor DI; functional-only Pipe remains
excluded instead of receiving newly incorrect edges.

The independent Service publication implementation is unchanged. AuditService
and LoggerService declarations are checked as separate facts, with exact
domain/type/name/file counts. The Pipe already has its intrinsic PipeName
Defines relationship and correct core owner even without DI; that supplies its
existing public occurrence. No independent Pipe declaration carrier was added.

The edge oracle examines actual Injects identities; neither dependency entities,
Pipe occurrence counts nor rendered markers substitute for that relationship.

## 8. Identity, unresolved targets and deduplication

The existing occurrence/edge model remains unchanged. Multiple parameters with
the same written type produce one indexed semantic relationship per asserting
file, not a physical parameter-occurrence count. Repeated context publication
and edge queries preserve exact counts. Same-file Service registration plus DI
endpoints does not multiply the declaration-source occurrence.

Object.file remains **asserting occurrence provenance**, not a resolved dependency
declaration file. With two CollisionService declarations and one importing Pipe,
both declaration occurrences survive, but no import-resolved target binding is
established or selected. The edge remains the written symbolic constructor type.
No simple-name lookup is used to select a declaration.

MissingService follows that same existing symbolic model. Its source fixture
declares no MissingService class; the DI endpoint is not an independently
recognized declaration. This phase does not change the public untyped occurrence
schema or fabricate a Service declaration carrier to satisfy an edge.

The constructor helper's existing type eligibility/normalization is inherited;
qualified/complex constructor types and parameter token overrides are not new
acceptance claims. Runtime injector resolution is not modeled by this slice.

## 9. Lifecycle actually verified

Registered refresh_workspace is used after each owned source mutation. Public
queries verify AuditService, then LoggerService with the old edge gone, then
empty DI, then deleted source with no remaining edge. This preserves existing
source/index lifecycle; no cache, acquisition, refresh or query-history code
changed. It does not establish #142 cold/warm completeness.

## 10. Executed preservation and commands

Cargo used PATH=/workspace/.tooling/cargo/bin:$PATH,
RUSTUP_HOME=/workspace/.tooling/rustup and CARGO_HOME=/workspace/.tooling/cargo.
All test commands use:

```bash
cargo test --locked -p clean-ctx --lib --all-features -j 4 FILTER
```

| Filter | Passed |
|---|---:|
| angular_di_pipe_ | 9 final; primary 0/1 RED → 1/1 GREEN |
| angular_meta::constructor_injects::tests:: | 21 |
| angular_meta::constructor_di_edge_tests | 15 |
| angular_meta::semantic_tests | 18 |
| angular_meta::ngrx::tests:: | 25 |
| angular_template_compatibility_tests | 1 |
| workspace_query_angular_selector_occurrence_tests | 5 |
| restore_rejects_semantic_producer_generation_mismatch | 1 |
| decorated_owner_identity_ | 9 |
| framework_declaration_publication_ | 10 |
| insert_before_declaration_ | 9 |
| semantic_signature_evidence_ | 5 |
| di_omission_ | 6 |

That is **9 Pipe/compatibility checks and 125 preservation checks**. Template
evidence here is the executed compatibility/selector filters, not an unrun
complete template suite. Frozen slice filters are reported only where executed.

Focused rustfmt ran on changed Rust files only, edition 2024 with
--config skip_children=true. Package Clippy passed with zero warnings:
`cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings`.
The single authorized candidate build passed:
`cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4`.
The full test suite and CI gate were not run.

## 11. Live commands, artifacts and corrections

The reusable harness is verification/issue-reconciliation/Invoke-AngularPipeDi.ps1.
It reuses PhaseBSession.ps1, actual stdio MCP tools/list admission, bounded
request/response/stderr capture and cleanup of owned processes. CBM is disabled
in the owned Git-root fixture. It builds nothing and invokes no model host.

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-AngularPipeDi.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

On Linux omit .exe. Cloud runs used the installed PowerShell 7 under
/workspace/.tooling/powershell/opt/microsoft/powershell/7/ with XDG cache/config/
data directories under /workspace/.tooling. Baseline used the saved
clean-ctx-pipe-baseline copy; candidate used clean-ctx.

Evidence roots under target/angular-pipe-di:

- `20261012-141739-52d7145d`: final baseline **35 PASS / 21 FAIL**, 56 groups.
- `20261012-142048-69459e58`: final candidate **56/56 PASS**.
- Authored source manifests compare exactly equal; all four deferred observation
  JSONs also compare equal. Deferred observations are not included in 56 passes.
- Baseline SHA256:
  `04301e3194c3850cae42ba75499d1942a11994f385f391f2222a4d05d4b34bb8`.
- Candidate SHA256:
  `104f63f4bbcf8456ee1d7418de43f340afba87626cd1f281e06d05eebfa7037b`.

The earlier baseline `20261012-141510-718bccb1` is retained. Its collision
control counted the consumer's Service endpoint before the missing edge oracle.
The final control checks independent declaration files and the Pipe occurrence,
then inspects the exact symbolic edge; this corrects the evidence ordering
without changing sources or expected edge identity. Primary permanent RED is
unchanged. One apply_patch context mismatch during test-helper formatting made
no production/runtime change; it was corrected before further verification.

Cargo logs are copied into the final candidate evidence. Runtime outputs remain
under target/. No Claude/Codex selection, UI behavior or real-corpus acceptance
is inferred from local protocol verification.

## 12. Remaining #138 work

- Structured inject(...) token interpretation and receiving-member separation.
- Supported optional-call/expression and qualified/token forms; current observed
  omissions/wrong targets are not blessed as correct.
- Functional DI generally, complex/qualified constructor types and token overrides.
- Cross-file/module/import ambiguity and resolved target-declaration provenance.
- Signal inputs/outputs, host/view/provider metadata and other original criteria.
- Broader independent declaration/public occurrence contracts and #142 acquisition
  remain separate work, not prerequisites repaired here.

## 13. Incidental effects

The new Pipe edge becomes reachable through existing forward/reverse/index
mechanics. The bounded forward, deduplication and refresh results above are
verified consequences. No reverse-query, blast-radius, runtime DI, declaration
publication or other-issue acceptance is inferred without separate execution.

## 14. Changed files and audit

- src/angular_meta/semantic.rs: admit Pipe constructor evidence only, preserving
  existing consumer paths and preventing contaminated functional data admission.
- src/layers/meta/mod.rs: existing semantic generation 4, marker generation unchanged.
- src/mcp/mod.rs and src/tests/mcp/angular_di_pipe.rs: registered public regression
  and constructor/admission/deduplication/guard/refresh controls.
- src/mcp/compatibility/mod.rs and src/tests/mcp/angular_di_pipe_compatibility.rs:
  generation-3 versus current semantic comparison with canonical identity preserved.
- Existing Angular template/selector compatibility tests: expected current
  semantic generation aligned to 4; no template processing changed.
- Invoke-AngularPipeDi.ps1: reusable public-boundary driver, independent sources,
  exact edge oracles and explicitly deferred observations.
- README, this report and historical reconciliation link: bounded disposition.

Audit confirms no frozen owner/publication/NgRx/diff/edit/DI-retention implementation
change. The reused constructor helper implementation and functional extractor
remain unchanged. Shared index/identity/deduplication mechanics stay domain-blind.
No graph, token registry, runtime simulation, new parser or source-name recovery
heuristic was introduced. Active code stays below 615 lines and uses UTF-8
without BOM. Source-binding and unsupported-expression limits are explicit.

## 15. Stop

Pipe constructor admission is GREEN and documented. The distinct inject-token
authority repair is deliberately not implemented. No #138 issue was closed;
no legacy NgRx, #141 or #142 work began. Stop for the next owner-reviewed slice.
