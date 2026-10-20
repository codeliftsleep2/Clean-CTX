# Issue #155: four-blocker implementation handoff

Agent: Agent MaxHeadRoom

Latest acceptance (2026-10-20): operator reports all handed-off focused tests,
build and #155 MCP acceptance passing after the corrections below. MCP receipt
`target/live-acceptance/issue-155/2026-10-20T22-41-41-624Z-1364/report.json`
records PASS, 392/392 checks, 19 finished sessions and zero errors. The agent
read the receipt but did not execute runtime gates. This supersedes the pending
focused/MCP rerun statuses recorded chronologically below. The authoritative
final verification gate and original-workspace Claude field acceptance are not
established by this receipt and remain pending unless separately reported.

Status: operator reported focused GREEN on 2026-10-20 for
`cargo test --all-features tests_issue_155 -- --nocapture` after the compilation
corrections below. Full output and runtime-selected count were not supplied.
Production edits and static regression authoring are complete for the four
accepted in-scope blockers. Preservation, Clippy, binary/stdio and field
acceptance remain pending. No runtime gate was run by the agent.
This is not a claim that Issue #155 or its final gates have passed.

Operator-reported focused compilation stopped at the `workspace_query` schema's
`serde_json::json!` recursion limit in `src/mcp/tools.rs`. The coverage schema is
now constructed in a separate helper, preserving its registered JSON contract
without increasing the crate recursion limit. Direct formatting and static
whitespace inspection passed; both frozen regression hashes remain unchanged.
At this correction's handoff, focused GREEN was pending; the operator has since
reported GREEN as recorded above.

A subsequent operator compile reported an unused `HashSet` import in query edges
and two incorrect `SemanticSnapshot` module paths in the new blocker tests.
The unused import was removed; both helpers now use the type exported through
`crate::mcp::state::durable_semantics`. Related explicit module references in the
blocker tests were inspected statically. The operator subsequently reported
focused GREEN; neither frozen regression file was changed.

## Production components changed

1. Canonical C# dependency obligations in
   `src/ir/csharp_resolution/compilation/dependencies.rs` now include class and
   method attribute references, exact written and suffix discovery hints, aliases,
   original declaration/scope/span and resolver disposition. Existing canonical
   attribute resolution decides identity; discovery names never grant framework
   authority. Known producer attribute authority that remains ambiguous/unsupported
   stays unproven. Unmodeled unrelated custom attributes do not become new framework
   capabilities. All attribute discovery uses the existing SourceDependencyClosure
   frontier, not query-specific prewarming.
2. `src/mcp/semantic_compilation/dependencies.rs` is the common retained/context
   boundary used by workspace queries and non-query C# context consumers. It
   acquires through the accepted shared capability and projects requested rendering
   fidelity through canonical generation functions. C# generation reconciliation
   now attaches exact captured source hashes for consulted base/partial/shadow
   evidence to remembered semantic snapshots. A partition's dependency index is
   reused across owners during footprint construction.
3. `core/provide/evaluate.rs`, `core/provide/batch.rs`, `core/provide/outcome.rs`
   and `core/compress.rs` use that boundary before returning/persisting C# semantics.
   Rendering fidelity remains requested by the caller; the canonical semantic
   index retains its acquired High evidence instead of being overwritten by a
   lower-fidelity rendering. Existing durable persistence receives the consulted
   footprint; existing compatibility validation rejects stale dependency hashes.
   C# semantic producer generation advances from 4 to 5 so older snapshots lacking
   the new lifecycle evidence do not remain compatible. Canonical and unrelated
   language producer generations are unchanged.
4. Typed PublicationStamp validation is shared by dependency evidence, workspace
   prepared evaluators, and provided contexts. Workspace batches reprepare stale
   items until all required stamps match the final guarded index. Context batches
   reevaluate earlier successful C# items when later preparation supersedes their
   publication. Item order is preserved. A single-item supersession fails closed;
   synchronization does not grant authority. No global retry count or timeout was
   introduced.
5. `src/mcp/tools.rs` deliberately registers
   `semantic_dependencies_unresolved`. Reasons preserve legacy object entries and
   explicitly allow the bounded dependency string codes already used by the frozen
   supplemental tests. `dependency_proof` is registered as complete with a string
   generation, or unproven. Implementation and schema use those same forms.

Cache reuse checks the current target projection's recorded dependencies as well
as owner/source, partition, producer, root and publication compatibility. Inventory
acquisition stabilizes its source generation if reading newly changed sources
invalidates a previous root observation. A source-current retained artifact can be
reconciled after a base mutation without recompiling that unchanged owner merely
because its derived dependency footprint became stale.

## Frozen regression receipts

Neither frozen file was edited or passed to the formatter during this work:

| File | Tests | SHA-256 of current file bytes |
|---|---:|---|
| `src/tests/mcp/workspace_query_issue_155.rs` | 8 original RED cases | `2ed2e6f893ef2b8816862cad3007c2efe01276a826e87a4c7be16418c8de5fc3` |
| `src/tests/mcp/workspace_query_issue_155_dependencies.rs` | 12 accepted supplemental cases | `4d2cc161b0e07fcf64d0126872f0f8ee42388cd6aeb833f43e7a80368f44e549` |

The original's clean Git blob remains
`5ef4312270a072621bfa315ebe9e0221ebff0b60`, matching the isolated pre-fix RED stash.
The stash remains available. Its checkout representation is also compared against
the actual original file bytes in the final static receipt. No oracle weakening,
assertion rewrite, fixture change or formatting was applied to either frozen file.

## New regression inventory

`src/tests/mcp/workspace_query_issue_155_blockers.rs` is registered with `#[path]`
under `tests_issue_155_blockers`; the unchanged focused filter includes it.

| Test | Required observation |
|---|---|
| `issue_155_attribute_source_shadow_converges_cold_and_warm` | Local suffix-elided method attribute shadow never acquires HTTP action authority; cold and prior shadow query agree |
| `issue_155_attribute_ambiguity_is_partial_and_matches_registered_schema` | Competing framework/source attribute identity yields partial dependency coverage; emitted status/reasons/proof forms match registered schema |
| `issue_155_class_attribute_shadow_retracts_framework_authority` | Source ApiControllerAttribute lookalike does not establish a Controller role |
| `issue_155_context_snapshot_and_durable_footprint_retract_and_restore` | Cold provide acquires base authority; remembered and SQLite snapshots carry Bridge hash; mutation invalidates both and durable validator rejects stale proof; provide alone retracts/restores the Hub relationship without query prewarming |
| `issue_155_context_batch_revalidates_earlier_items_after_later_acquisition` | Two owners with distinct dependency chains return successful current context evidence in either item order |
| `issue_155_mixed_batch_stabilizes_after_late_publication_without_reordering` | One-shot actual late acquisition supersedes earlier prepared evidence; mixed surfaces stabilize to the current proof while preserving caller order |

The late-acquisition hook is test-only, one-shot and thread-local. It invokes real
shared dependency acquisition after batch preparation, before final validation.
The test compares the returned proof with freshly validated final evidence; it
does not change production query ordering or fabricate a completion boolean.

The focused filter should select 26 cases under all features: eight unchanged
originals, twelve unchanged supplementals, and six new blocker cases. This count
is from authored registration/definitions, not runtime discovery output.

## Explicit exclusions and verification limits

Separate campaigns are recorded in `docs/agent/ISSUE_155_FOLLOW_UP_CAMPAIGNS.md`:
EF model/configuration acquisition; Angular imported module footprints; NgRx
authority/reexport closure; selector completeness proof; TestClass subject
ownership; non-C# producer certificates. These are local campaign identifiers,
not fabricated GitHub issue numbers. The accepted sibling audit remains the static
evidence record; its in-scope unresolved rows are superseded by this handoff.

No repository-wide elimination claim is made for excluded producers. No proprietary
source was requested or copied. Claude acceptance remains the last field gate.

Static checks performed: direct Rust formatting of changed production/new blocker
files (both frozen files excluded), diff whitespace inspection, strict UTF-8/BOM
inspection, active Rust file line counts, registration/source trace, and frozen
file hash comparison. Direct formatting checks syntax, not Rust types or behavior.
Repository scripts, Cargo, tests, builds, Clippy, MCP binaries, providers and live
harnesses were not executed. Do not infer successful compilation or zero warnings.

## Focused runtime gate: operator-reported GREEN

From `C:\Users\MNasty\Desktop\RustContextLayerAI`:

```powershell
cargo test --all-features tests_issue_155 -- --nocapture
```

Expected GREEN: successful compilation and all selected #155 tests pass, with no
new warnings; original cold/warm role/edge/file facts converge; missing/ambiguous
dependencies remain truthful; snapshots retract/restore on dependency changes;
all successful proofs refer to surviving final publication; warm reuse remains
stable; and the registered status/reason/proof contract agrees with output.

Return complete focused output if any compilation or assertion fails. No broader
runtime gate is requested before focused GREEN. Preservation commands and the
authoritative final gate will be handed off only after that result is established.

## Next user-owned verification

Second MCP receipt `2026-10-20T22-37-11-974Z-21604` failed 347/348 checks at
`durable/stale-on-disk/provide/restored/dependency-proof`, with reason
`dependency_source_generation_changed`. Frontier source reads retired the root
observation captured by inventory. Shared dependency acquisition now rebuilds
the request-local inventory and proof when that observation is superseded.
New regression
`issue_155_context_transitive_mutation_stabilizes_inventory_without_refresh`
requires context-first complete proofs and correct remembered Hub retraction/
restoration over repeated transitive edits. The authored focused inventory is
now 28 cases. Both the focused gate and rebuilt MCP rerun remain pending;
the agent has not run them. The prior reported GREEN predates these corrections.

The first operator MCP run failed 23/24 checks at Controller edge coverage:
`evidence_requirements: null` violated its registered array schema. The receipt
is `target/live-acceptance/issue-155/2026-10-20T22-31-43-671Z-16928`.
Shared dependency coverage now uses non-inserting optional-field access in both
complete and unproven branches. New regression
`issue_155_edge_coverage_does_not_insert_null_evidence_requirements` checks the
edge response against the registered field type. This raises the authored
focused inventory to 27 cases; the earlier GREEN predates this correction.
Repeat the exact focused gate before rebuilding/rerunning MCP acceptance.
The harness schema assertion remains unchanged. No runtime was run by the agent.

The #155 binary/stdio acceptance harness is now authored at
`verification/live-acceptance/issue_155_live_acceptance.mjs`, with its matrix,
commands and proof limits in `verification/live-acceptance/issue-155/README.md`.
It is NOT RUN. It reuses the #153/#154 transport while independently testing
cold acquisition without publishing dependencies first. Its operator result
must remain separate from the focused Rust GREEN and final verification gate.

Focused GREEN is now reported. Run the authoritative Final Verification Gate in
`docs/agent/verification.md`; its full workspace test run includes preservation
coverage. The agent has not run it. Binary/stdio and relevant field acceptance
remain separate pending evidence, with Claude reserved for the last field gate.
