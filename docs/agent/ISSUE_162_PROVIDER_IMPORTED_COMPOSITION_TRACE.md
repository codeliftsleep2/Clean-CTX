# Issue #162: imported provider-array membership

Agent: Agent MaxHeadRoom
Status: P1.C2 owner-approved; generation-64 focused/preservation Rust receipts GREEN (118 passing executions), fmt/Clippy GREEN; major-checkpoint binary acceptance deferred.

The inspected operator log records 0 passed, 6 failed, 0 ignored, all at the
imported stage after the direct/same-source prerequisites. Exact bytes and the
source-qualified authority decision are recorded in the
[imported composition decision](ISSUE_162_PROVIDER_IMPORTED_COMPOSITION_DECISION.md).
Historical UNRUN statements below describe initial test authorship. The approved
implementation checkpoint and its separate UNRUN verification status are in that
decision; this trace does not certify acceptance.

## Canonical question

Which canonical tokens does an exact Component declare through an imported const
provider array in providers or viewProviders, and which Component/position owns
that assertion when the terminal entries are written in another source file?

This follows the verified same-source P1.C1 checkpoint. Its approval explicitly
excluded imported contributor arrays. Investigation/test authoring continues the
existing map, batch RED, observe, trace, decide workflow; no cross-source semantic
extension is authorized here.

## Located boundaries

| Production boundary | Static finding | Evidence still required |
| --- | --- | --- |
| Native declaration facets | Demand::declaration reads the exact declaration source through the resolver Host, checks its hash and syntax identity, and retains demanded const initializers. Demand::reference can expand resolved declarations. | Actual imported GROUP resolution, initializer contents and terminal references in each fixture; function existence alone is not capture acceptance. |
| Provider composition | provider_composition::visit explicitly requires the variable declaration source/hash to equal authority.source/source_hash before following its initializer. | Observe whether each route reaches this refusal with complete retained capture, or loses authority earlier. |
| Producer syntax | provider_semantics::project_at parses the owner source and selects terminal nodes by byte span from that tree. | Foreign-source byte offsets must never select unrelated nodes in the owner file. Removing only the walker's source guard would not establish a safe producer. |
| Occurrence admission | Existing provider token/pair and registration replay share retained membership, source and exact native occurrence checks. | Trace source qualification for every hop and terminal reference before recommending a cross-source carrier or accessor. Span equality alone cannot identify a foreign occurrence. |
| Existing module composition | module_metadata::canonical::collect follows exact retained initializer facets and collects declaration leaves. | Its declaration-set leaf policy does not establish provider-entry authority, owner association or strategy pairing. It is a reuse candidate for mechanics, not an automatic provider solution. |
| Publication/lifecycle | Existing relations name the asserting Component and canonical provided token. | Contributor origin must not replace asserting ownership; dependency changes must invalidate/retract the owner's assertion through the existing lifecycle. No new publication implementation is proposed. |

Clean-CTX reports CBM degraded. Available provider contexts returned skeletons;
targeted native reads supplied bodies. A nonexistent module_composition path was
discarded; file discovery located the actual module_metadata/canonical.rs owner.
This tooling state is not behavioral defect evidence.

## Batched tracked questions

src/tests/mcp/angular_provider_imported_composition_issue_162.rs contains six
tests: aliased direct import, barrel re-export with spread, and namespace barrel
access through nested/spread arrays, each in ordinary and view positions.

Every test first requires a direct two-token answer, then the established
same-source const/spread answer. All stages require exact native Component role.
The imported stage requires the same two same-named declarations from distinct
token files, deduplicated despite repetition, with app.ts as the asserting owner.
Queries repeat and the initiating source bytes must remain unchanged.

Failure diagnostics include actual retained metadata shapes and full demanded
initializer identities/contents. An imported-stage registration failure after
the prerequisites is candidate behavioral RED. A compile error, prerequisite
failure or missing tool response must be classified separately. Do not infer
successful later provenance assertions from an earlier panic.

No tests were executed or frozen as observed evidence. After operator results,
preserve the unchanged valid RED batch and inspect the actual full source-qualified
retention route before proposing the smallest bounded owner decision. Do not
repair by removing the source guard, interpreting foreign offsets in the owner
tree, widening textual lookup or inventing runtime array values.

## Stopping boundary

Only test registration, tests and investigation documentation changed. Production
semantics/generation, relation values, persistence and global identity are unchanged.
No imported-array implementation or architectural carrier is selected yet.

This question concerns declared structural membership. It does not establish
runtime mutation effects, execution order, effective visibility, injector hierarchy,
precedence/multi, winning providers or consumer satisfaction. Directive/Module
composition and other owner contexts remain separately unreconciled. P1, W04 and
Issue #162 remain open. Operator execution owns compilation and runtime; final
Claude field verification remains last.
