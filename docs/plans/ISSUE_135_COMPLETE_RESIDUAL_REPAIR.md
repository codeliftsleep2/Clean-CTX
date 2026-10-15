# Issue #135 Complete Residual Repair

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-15  
**Repository state:** `36f97383` plus the working-tree changes listed below

## 1. Executive verdict

#135 COMPLETE RESIDUAL REPAIR — FOCUSED VERIFICATION GREEN

R135-C and R135-A are implemented and their focused commands were reported green
by the maintainer. R135-B now has a C#-scoped owner/signature Method identity
plus approved source-reconstitution reconciliation across every admitted C#
caller-bearing file. Additions, removals, signature changes, and ambiguity can
establish or revoke exact calls independent of file order and warm history.
Invocation evidence remains ephemeral. R135-D is implemented. The maintainer
reported every focused command in section 12 green after the final repairs.

## 2. Files changed

- `src/ir/csharp_exact_calls.rs`
- `src/mcp/csharp_call_reconciliation.rs`
- `src/ir/compiler.rs`
- `src/ir/mod.rs`
- `src/ir/semantic_projection.rs`
- `src/tests/ir/csharp_exact_calls.rs`
- `src/tests/mcp/csharp_call_reconciliation.rs`
- `src/mcp/tool_handlers/query.rs`
- `src/mcp/tool_handlers/query/entities.rs`
- `src/tests/mcp/workspace_query_file_diagnostics.rs`
- R135-D expectation updates in existing workspace-query tests
- the previously reported R135-A/R135-C production, tests, and documentation

## 3. R135-C implementation

Batch requests reject top-level `fidelity` with `-32602`; fidelity remains
item-owned. Schema and runtime enforcement agree. The maintainer reported the
focused tracked and live commands green.

## 4. R135-A implementation and fidelity contract

ARCH-003 permits explicit bounded language-scoped loss only in model-facing
projections. C# structural responses publish stable omission categories by
fidelity; canonical authority remains complete, non-C# behavior is unchanged,
and Verbatim/raw output remains exact with `degradation: null`. The maintainer
reported the focused C# degradation corpus green.

## 5. R135-B implementation and exact callee authority

Implemented:

- distinct `csharp/Method` declaration identities shaped as
  `Owner::Method(ParameterTypes)->ReturnType`;
- exact C# registrations and `Calls` edges at the production compilation
  boundary;
- receiver/type binding for proven `this`, static, constructed, parameter,
  local, and field spellings;
- exact-arity overload selection and extension-method selection;
- fail-closed behavior for spread, missing, unresolved, or ambiguous targets;
- preservation of generic written-call edges and ephemeral invocation evidence;
- generation bumps `CSharpCanonicalProducer: 2 -> 3`,
  `CSharpSemanticInput: 2 -> 3`, and `CSharpSemanticProjection: 3 -> 4`;
- tracked same-name receiver, overload, ambiguity, and expression-bodied
  property regressions.

Cross-file lifecycle:

- after C# declaration authority changes, pass one recompiles every admitted C#
  source owner to reconstruct the complete current declaration catalog;
- pass two recompiles every admitted C# caller against that catalog;
- all source hashes are revalidated before one WorkspaceIndex replacement;
- session semantic snapshots are updated for owned aliases;
- normal publication, hydration, query publication, edit/delta, restore/replay,
  recovery, and deletion paths trigger reconciliation;
- no invocation-resolution candidate enters canonical IR or persistence;
- the complete affected set remains all admitted caller-bearing C# files until a
  narrower dependency set can prove equivalent completeness.

## 6. R135-D implementation

`entities_in_file` and `declarations_in_file` now distinguish:

- admitted supported files, including valid zero-result files;
- `excluded_by_within_path`;
- `outside_authorized_roots`;
- `unsupported_file_type`;
- `file_not_found`.

Unauthorized responses do not echo the requested path. File queries remain
provider-independent and do not acquire #142 discovery semantics.

## 7. Regression coverage added

- `provide_code_context_batch_fidelity.rs`
- `csharp_skeleton_degradation.rs`
- `csharp_exact_calls.rs`
- `csharp_call_reconciliation.rs`
- `workspace_query_file_diagnostics.rs`
- adjusted legacy missing/deleted/withinPath/outside-root expectations

## 8. Generation/invalidation decisions

R135-C and R135-A do not change canonical or durable semantic output. R135-B
corrects expression-bodied C# return authority and changes durable semantic
output, so the C# canonical producer and semantic-input generations are bumped
to 3 and the C# semantic projection generation is bumped to 4. R135-D changes
only bounded file-query diagnostics.

## 9. Cross-issue boundary audit

No #133/#134 behavior was reimplemented. No #136-#142 production policy was
introduced. R135-D remains file-query-local. Generic call identity and every
non-C# language remain unchanged. CBM is not required.

## 10. Architectural approval decisions encountered

The maintainer approved the R135-A degradation contract, the R135-B C#-scoped
durable exact Method identity, and the all-admitted-C# source-reconstitution
lifecycle. All three decisions are implemented within their approved scopes.

## 11. Remaining residuals

No known implementation or focused-verification residual remains within
R135-A/B/C/D. The authoritative final verification gate remains separate.

## 12. Focused verification reported green

```powershell
cargo test --locked -p clean-ctx --lib --all-features csharp_exact_calls
cargo test --locked -p clean-ctx --lib --all-features csharp_call_reconciliation
cargo test --locked -p clean-ctx --lib --all-features workspace_query_file_diagnostics
cargo test --locked -p clean-ctx --lib --all-features workspace_query_declarations_in_file
cargo test --locked -p clean-ctx --lib --all-features workspace_query_within_path
cargo test --locked -p clean-ctx --lib --all-features workspace_query_scope
```

The maintainer reported all commands above green on 2026-10-15. These are
operator-run tracked-test results; the agent did not execute them. Focused
success does not imply the authoritative final gate.

## 13. Commands actually run

Bounded `rg`, `Get-Content`, line counts, `scripts/check-file-sizes.ps1`,
`git diff --check`, `git status --short`, and patch-based edits. The
file-size guard passed after reporting only untouched legacy debt.

Not run: Cargo test/check/build/Clippy/fmt, binaries, MCP servers, live scripts,
benchmarks, providers/CBM, Claude pilots, final suites, or GitHub mutations.
