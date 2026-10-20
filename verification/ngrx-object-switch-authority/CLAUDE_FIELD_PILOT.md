# Issue #150 NgRx object-action / switch-reducer Claude field pilot

## Purpose

Run this pilot on the computer that has access to the real application
workspace after pulling the Issue #150 Clean-CTX changes and building the
release binary.

This is a real-workspace, model-facing acceptance check. It complements the
tracked Rust regressions and deterministic MCP stdio harness; it does not
replace either one.

## Privacy boundary

- Do not copy, paste, upload, or reproduce proprietary source code.
- Keep generated evidence on the local computer.
- Report only sanitized semantic identities, relation names, metadata, counts,
  tool behavior, and pass/fail results.
- Redact repository names, paths, business terminology, literal action values,
  and other identifying information before sharing results externally.

## Preparation

From the Clean-CTX repository root, run the authoritative verification gate in
`docs/agent/verification.md`, then build the production artifact:

```powershell
cargo build --release --all-features
Get-FileHash -Algorithm SHA256 -LiteralPath ./target/release/clean-ctx.exe
```

Configure Claude to use that exact `target/release/clean-ctx.exe`, completely
stop the previous MCP process, and start a fresh Claude session. Record locally:

- Clean-CTX commit SHA;
- release-binary SHA-256;
- sanitized application revision identifier;
- whether CBM is enabled and its initial status;
- pilot date and operator.

Do not reuse semantic results from an older Clean-CTX process.

## Prompt for Claude

Copy the complete block below into the fresh Claude session.

```text
Use Clean-CTX against the current application workspace to validate Issue #150's
NgRx object-action/switch-reducer capability.

Privacy boundary:
- Do not reproduce, quote, or transmit proprietary source.
- Do not include real repository paths, business names, string literal values, or
  source excerpts in the final report.
- Use sanitized labels such as ActionObjectA, ACTION_1, ComponentA, and ReducerA.
- Report only semantic identities, relationships, metadata, counts, and tool behavior.

Tool protocol:
- Use Clean-CTX graph_search for discovery when CBM is available.
- If CBM is unavailable or degraded, use search_codebase for discovery.
- Use provide_code_context for understanding known files.
- If provide_code_context returns a skeleton and bodies are required, use read_files
  only for the known file and relevant range.
- Always pass workspaceRoot explicitly.
- Do not infer filesystem paths from CBM project identifiers or namespaces.
- Use workspace_query for semantic entities and relationships.
- Record every Clean-CTX tool name used and summarize its structured result without
  exposing proprietary source.

Perform this acceptance matrix:

1. Locate one real exported string-constant action object that is used through a
   named import alias.

2. Select at least two distinct members of that object. If two members have equal
   string values, confirm that Clean-CTX preserves them as distinct qualified action
   identities. Do not report the literal values.

3. For each selected member, identify the authoritative owner that dispatches it.
   Confirm the dispatch receiver is structurally established as an NgRx Store rather
   than accepted merely because a method is named dispatch.

4. Identify the reducer registered through the application's supported
   StoreModule.forFeature form. Confirm registration resolves to the reducer
   declaration even when the import uses an alias.

5. Establish and report the semantic path for each selected action:

   dispatching owner
     -> Dispatches
   qualified exported action member
     -> TriggersReducer
   registered reducer

6. Confirm that a local import alias does not become semantic identity. The
   authoritative result must use the exported object/member identity.

7. Query the relevant action and reducer files with entities_in_file. For the
   supported object-action and switch-handler facts, report whether
   coverage.semantic_omissions is absent. If an omission remains, report its exact
   metadata token without including source.

8. Repeat representative find_entities, forward_edges, reverse_edges, and
   entities_in_file queries in the same session. Compare cold and warm results and
   confirm semantic equivalence. Timing may differ; identities, relationships, and
   coverage truth must not.

9. If a naturally occurring neighboring unsupported or incomplete form exists,
   inspect it without modifying the proprietary workspace. Confirm Clean-CTX either
   retains truthful omission metadata or publishes no unsupported framework
   relationship. It must not guess.

10. Audit the observed responses for:
    - duplicate identical entities;
    - duplicate identical edges;
    - alias spellings leaking into semantic identity;
    - contradictory completeness or coverage claims;
    - semantic differences caused only by CBM/provider state;
    - misleading fallback_reason wording;
    - duplicate identical discovery.projects diagnostics.

11. If the workspace safely permits a pre-existing alternate checkout or disposable
    local copy, perform one bounded lifecycle check there: remove or alter one
    authority input, refresh, verify retraction, restore it, refresh, and verify
    recovery. Never modify the authoritative proprietary checkout solely for this
    pilot. Mark this item NOT RUN if no safe disposable copy exists.

Return a sanitized report using exactly this structure:

# Issue #150 Claude field-pilot report

## Provenance
- Clean-CTX commit: <sha or locally retained receipt id>
- Clean-CTX binary SHA-256: <hash>
- Application revision: <sanitized local receipt id>
- CBM initial state: <available, degraded, unavailable>
- Cold session confirmed: <yes/no>

## Results
| ID | Criterion | Result | Sanitized evidence |
|---|---|---|---|
| 1 | Exported object and alias discovered | PASS/FAIL | ... |
| 2 | Distinct qualified member identities | PASS/FAIL | ... |
| 3 | Exact Store dispatch authority | PASS/FAIL | ... |
| 4 | Exact reducer registration authority | PASS/FAIL | ... |
| 5 | Dispatch-to-reducer paths | PASS/FAIL | ... |
| 6 | Alias normalized to export identity | PASS/FAIL | ... |
| 7 | Supported coverage omissions cleared | PASS/FAIL | ... |
| 8 | Cold/warm semantic equivalence | PASS/FAIL | ... |
| 9 | Unsupported neighbor fails closed | PASS/FAIL/NOT RUN | ... |
| 10 | Diagnostics and deduplication audit | PASS/FAIL | ... |
| 11 | Retraction and restoration | PASS/FAIL/NOT RUN | ... |

## Tool evidence
List each Clean-CTX tool used, the sanitized request purpose, and the relevant
structured-result facts. Do not include source text or identifying paths.

## Discrepancies
List every unexpected omission, missing relationship, fabricated relationship,
duplicate fact, provider-dependent semantic difference, or misleading diagnostic.
Write "None" only if no discrepancy was observed.

## Final verdict
Return exactly one:
- PASS — real-workspace capability verified
- FAIL — capability or contract discrepancy observed
- INCONCLUSIVE — required evidence unavailable

Do not convert NOT RUN into PASS. Do not infer facts that Clean-CTX did not expose.
```

## Result handling

Retain the full unsanitized tool transcript only on the authorized local computer.
Share the sanitized report with Agent MaxHeadRoom. A field-pilot failure or
inconclusive result does not become green because deterministic tests passed; report
the boundary exactly and preserve the local evidence needed to reproduce it.
