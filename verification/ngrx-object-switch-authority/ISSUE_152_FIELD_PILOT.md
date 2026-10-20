# Issue #152 NgRx semantic-envelope field pilot

Run this only after the deterministic gate and production-binary harness pass.
It is the final real-workspace check, not a substitute for tracked regressions.

## Privacy boundary

Keep proprietary source and paths local. Report only sanitized identities,
relations, coverage fields, counts, and pass/fail evidence. Use names such as
`ActionObjectA`, `ACTION_1`, `ReducerA`, and `reducers` in the report.

## Operator preparation

Build the release binary, record its SHA-256 and commit, configure Claude to use
that exact binary, stop the old MCP process completely, and start a cold session.

## Claude prompt

```text
Use Clean-CTX to verify Issue #152 against the current application workspace.
Never reproduce proprietary source, literal values, real paths, or business names.
Always pass workspaceRoot explicitly and preserve filesystem paths versus CBM
project identifiers. Use CBM only for advisory discovery; use Clean-CTX source and
workspace semantics as authority.

Locate one real chain equivalent to:
constant declaration -> shorthand/reference-valued exported object member ->
qualified member dispatched through an exact @ngrx/store Store receiver;
exact StoreModule registration -> reducer map -> map member -> reducer declaration;
bare switch case bound to the same constant declaration.

Verify through workspace_query:
1. The Action identity is the qualified exported member, never the bare constant,
   alias, or literal.
2. The reducer map has no authority apart from its exact NgRx registration consumer.
3. Dispatches and TriggersReducer connect the qualified Action and resolved Reducer.
4. entities_in_file(module) in a cold session has the same semantic entities and
   coverage as after related files are queried. Both must not claim identical exact
   completeness while returning different truth.
5. Supported action, reducer, and handler files have no corresponding
   semantic_omissions; report every remaining token exactly.
6. No duplicate entities/edges, alias leakage, spelling/literal joins, or
   provider-state-dependent semantic differences occur.
7. If a safe disposable checkout exists, remove and restore one map member or
   registration consumer, refresh, and verify retraction then recovery. Otherwise
   mark lifecycle NOT RUN; never modify the authoritative checkout for this pilot.
8. If two authorized qualified members share one backing constant in the real
   workspace, confirm a bare case does not arbitrarily choose one. If absent, mark
   this natural-workspace observation NOT RUN; deterministic coverage already exists.

Return a sanitized table with rows for qualified member identity, exact Store
dispatch, reducer-map registration, bare-case join, cold/warm equivalence, coverage,
deduplication, ambiguity, and lifecycle. Mark each PASS, FAIL, or NOT RUN. Finish
with exactly PASS, FAIL, or INCONCLUSIVE. Do not convert NOT RUN into PASS.
```

Retain full transcripts locally. Share only the sanitized report with Agent
MaxHeadRoom.
