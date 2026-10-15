# Issue #135 Final Acceptance Reconciliation

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-15  
**Repository state:** `36f97383` plus this read-only reconciliation report  

## 1. Executive verdict

#135 ACCEPTANCE RECONCILED — NOT READY TO CLOSE

> **Implementation addendum (2026-10-15):** R135-C and R135-A are implemented
> and their focused commands were reported green by the maintainer. The
> maintainer approved the R135-B C#-scoped durable exact Method identity.
> Exact projection, semantic generation 4, approved all-admitted-C#
> source-reconstitution reconciliation, and bounded R135-D diagnostics are
> implemented. The maintainer reported the complete focused command set green
> after the final repairs, then reported the authoritative final gate and the
> consolidated provider-disabled Issue #135 stdio harness green. See
> `ISSUE_135_COMPLETE_RESIDUAL_REPAIR.md` for the completed implementation and
> verification record.

The DI ten-entry cap is repaired and permanently covered, but material original
criteria remain unresolved: public C# skeleton fidelity, exact callee ownership,
batch-level fidelity validation, expression-bodied-property call coverage,
partial/record shape, conditional DI provenance, and file-scope diagnostics.

No production or test implementation was performed. No Cargo command, binary,
live script, provider call, Claude pilot, or GitHub mutation was run.

## 2. Original contract

Issue #135 requires: complete or explicitly bounded DI registrations; truthful C#
signatures, attributes, and type shapes; declaration-owned call graph answers with
coverage; effective or rejected context parameters; and declaration/file-scope
query semantics that do not silently mislead.

## 3. Acceptance matrix

| Original criterion | Current evidence | Status | Reasoning |
| --- | --- | --- | --- |
| 40 DI registrations complete or explicitly truncated | Complete recognized vector; tracked 0/9/10/11/40; retained 16/16 live PASS | SATISFIED | The destructive `truncate(10)` is gone. |
| Conditional DI truth/provenance | Both branches remain written mentions without condition/activation authority | UNSATISFIED | Written evidence appears unconditional. |
| Real C# returns at structural fidelities | High has structured evidence; Medium explicitly removes return-type-first returns | UNSATISFIED | `degradation: null` can accompany incomplete signatures. |
| Attributes/defaults/constraints/docs/types | Attributes and defaults are stripped; no complete #135 corpus/disclosure matrix | UNSATISFIED | Selected meta markers are not general fidelity. |
| Positional records and partial parts | Record identity and occurrences exist; no public record-parameter/parts contract | UNSATISFIED | Identity is not shape/linkage. |
| Nested owner and kind | Span-aware owner plus nested-enum regressions | SATISFIED | The reproduced proximity/flattening family is protected. |
| Base-list meaning | Neutral `HasBaseType` plus declaration-aware query classification | SUPERSEDED | Written truth is preserved without fabricated kind. |
| Caller ownership | Innermost callable span; unattributed calls dropped | SATISFIED | Caller-side proximity attribution is repaired. |
| Callee owner / same-name precision | Written callee plus arity/spread, not declaring type | UNSATISFIED | Reverse `Method/<name>` remains non-exact. |
| Graph coverage | Exact-identity edge/traversal coverage and lower-bound semantics | SATISFIED | Cold incompleteness is disclosed. |
| Expression-bodied property calls | No issue-specific public-edge regression found | UNSATISFIED | Requested coverage is not established. |
| Batch-level `fidelity` applies or rejects | Only root/tokenizer propagate from batch parent | UNSATISFIED | The original silent-ignore path remains. |
| Per-item parameters/response channels | Per-item intent/fidelity/focus and response modes have tracked/live coverage | SATISFIED | Documented item contract is protected. |
| Declaration precision | `declarations_in_file` is distinct from occurrence-oriented `entities_in_file` | SUPERSEDED | Declaration authority has an explicit query. |
| Outside-scope disclosure | Unadmitted paths can become empty results | UNSATISFIED | Empty does not distinguish not-in-scope. |

## 4. Skeleton reconciliation

The current pipeline is safer than the issue snapshot. Tree-sitter captures C#
class/interface/struct/enum/record declarations; type scopes use source-span
containment; nested classes and enums retain their own members; and structured
headers feed base-type semantics. These changes resolve the reproduced nested
ownership/kind family and replace the old first-base `B?` behavior with a neutral
written-base contract plus scoped declaration-aware classification.

The public structural presentation is still not a signature round trip.
`compact_method_medium` explicitly removes return-type-first return types. Low
keeps names and bare parameter names. Leading C# attributes are stripped before
ordinary method/class compaction, and Medium removes parameter defaults. There is
no #135 corpus comparing model-facing skeletons to Roslyn, and no comprehensive
omission disclosure for attributes, defaults, constraints, XML docs,
field/property types, positional-record parameters, or partial parts.

Loss is acceptable only when declared. `content_kind: skeleton` and the fidelity
label identify representation, but `degradation: null` does not enumerate these
semantic omissions. The output can therefore still materially mislead a consumer
that treats it as a complete C# signature.

## 5. DI reconciliation

The historical chain was:

```text
recognized source calls -> normalized registration vector
  -> truncate(10) -> MetaBlock aliases -> public context
```

Commit `aa0c3dbf` removed only the destructive cap. The producer now retains every
recognized deduplicated written registration. `src/tests/mcp/di_omission.rs`
authors 0/9/10/11/40 cases at Medium and High and checks exact identities. It also
guards distinct service, implementation, and lifetime combinations. The live
harness accepts compact markers only when the complete expected set/order is
present, or raw fallback only when the full source is byte exact.

No adjacent ten-entry cap remains in this DI producer. Caps in validation,
logging, EF, AutoMapper, and serialization are separate fact families. The
remaining DI residual is semantic scope: both `#if` branches are written mentions
without activation/condition provenance, and unsupported forms remain outside the
recognized set. This repair must not be described as evaluated runtime-container
completeness.

## 6. Call-graph ownership reconciliation

The current production chain is:

```text
tree-sitter invocation occurrence
  -> innermost callable span in PassContext
  -> CallProducer fact with caller identity and written callee
  -> CoreOp::Call(caller, callee, arity, spread)
  -> Method(caller) --Calls--> Method(written callee)
  -> WorkspaceIndex -> public edge response plus coverage
```

This correctly rejects proximity as caller ownership. Nested methods,
constructors, and supported arrow declarations have regressions; an invocation
without a precise callable owner is dropped rather than guessed.

The chain does not resolve the callee to its declaring type. `CallEvidence`
preserves explicit argument count and spread, which separates overload forms but
does not distinguish `Alpha.Save` from `Beta.Save`. Reverse lookup on the shared
`Method/Save` identity can therefore collect callers to different declarations.
`calls_in_file` answers a different question: it uses an exact typed owner to
select a caller and returns written call occurrences without claiming target
resolution.

Coverage is repaired independently. Current exact-identity edge responses expose
identity/capability status, source completeness, omitted-possible, and lower-bound
semantics. That cannot convert a name-only callee into an exact declaration.

### R135-B — exact C# callee ownership

- **Authority chain:** invocation syntax and receiver evidence -> candidate method
  declarations -> exact declaring type and visible signature; ambiguity remains
  explicit.
- **Production ownership:** C# invocation evidence/resolution and semantic
  projection; publish only proven exact targets or a deliberately unresolved
  written-call identity.
- **Regressions:** two classes with the same method name, instance versus
  extension forms, overload arity, ambiguous/unresolved receiver, nested caller,
  expression-bodied property, cold/warm, and file-order permutations.
- **Preservation/negative guards:** retain caller span ownership, written callee,
  argument/spread evidence, no proximity inference, no name-uniqueness assumption,
  no provider requirement, and no fabricated edge on ambiguity.
- **Verification:** focused tracked tests, provider-disabled public MCP harness,
  then the authoritative final gate; Claude only for an unavoidable field-only
  question.
- **Non-goals:** global cross-language resolution, runtime dispatch, reflection,
  dynamic invocation, or generic query-completeness changes.

## 7. `provide_code_context` reconciliation

Single and batch forms are separated, item results are ordered and isolated, and
three response channels are declared. Per-item `intent`, `fidelity`, and
`focusMethods` are effective. Only `workspaceRoot` and `tokenizer` are shared.

The original bug remains visible in `item_params`: the handler copies the item,
removes `id`, and adds only root/tokenizer from the parent. Parent `fidelity` is
neither forwarded nor rejected. The schema exposes top-level `fidelity`; the
batch branch forbids `filePath` but not `fidelity`. A client can still request
top-level `fidelity: "verbatim"` and receive item defaults/skeletons silently.

### R135-C — batch-level parameter rejection

- **Authority chain:** request schema -> runtime validation -> per-item effective
  arguments -> response metadata.
- **Production ownership:** `provide_code_context_request_variants` and batch
  validation/orchestration.
- **Regressions:** top-level batch fidelity must return `-32602` with a clear
  per-item instruction, or an explicitly approved default contract must apply it;
  cover conflicts, typos, all response modes, and singular guards.
- **Preservation/negative guards:** retain per-item overrides, failure isolation,
  duplicate-file refusal, shared root/tokenizer, modes, and single-file behavior.
- **Verification:** dispatcher/schema tests, provider-disabled batch harness, then
  final gate.
- **Non-goals:** new response modes, cache-economics changes, or general MCP
  validation redesign.

Explicit rejection is recommended. It matches the documented per-item ownership
model and avoids inventing precedence. Because this is externally visible request
behavior, implementation requires explicit task authorization.

## 8. Regression, live, and full-gate inventory

### Tracked Rust regressions

- `src/tests/mcp/di_omission.rs`: recognized DI completeness above the old cap.
- `src/tests/ir/calls.rs` and language call suites: caller ownership,
  arity/spread, and refusal to guess unattributed calls.
- `src/tests/ir/csharp_nested_class_semantics.rs`, nested pipeline tests, and
  `src/tests/edit/spans_nested_render.rs`: nested owner/kind preservation.
- `src/tests/mcp/workspace_query_calls_in_file.rs`: owner-aware file-local call
  inspection, overloads, ambiguity, and freshness boundary.
- `src/tests/mcp/workspace_query_exact_identity_coverage.rs`: query coverage.
- `src/tests/mcp/provide_code_context_batch.rs`: per-item batch behavior and
  response channels; it does not close top-level fidelity handling.
- `src/tests/mcp/workspace_query_declarations_in_file.rs`: declaration/use
  authority separation.

### Focused native/live evidence

- `target/di-omission/20261013-115210-850b1e84/`: historical 16/16 PASS recorded
  in `verification/LIVE_GATE_REGRESSIONS.md`.
- Batched context live acceptance is recorded PASS at
  `target/provide-context-batch-verification/captures/verification-report.json`.
  Its contract is per-item fidelity, not top-level rejection/defaulting.
- Structured C# base-type live acceptance is recorded PASS at
  `target/live-acceptance/csharp-base-types/responses.json`.

These are historical/operator records, not current executions or Rust test
results. No current live run was authorized or performed.

### Full/final gate evidence

The maintainer reported the complete final gate green on 2026-10-15 for the
preceding #134 repository state. It establishes health at that checkpoint, not
satisfaction of unimplemented #135 criteria. No final gate was run for this
documentation-only reconciliation.

## 9. Cross-issue boundaries

- #133 and #134 remain closed baselines.
- #136 whole-declaration insertion is unrelated to #135 context fidelity.
- #137's diff snapshot retains return/authorization evidence; that private
  comparison authority does not change #135 model-facing skeleton rendering.
- #138 Angular DI/member forms and #139 Angular modules/routes/entity typing are
  excluded.
- #142 owns cross-cutting completeness, refresh, and diagnostic policy. Bounded
  #135 fixes must not introduce global query-completeness semantics.
- This report reuses `declarations_in_file`; it does not redefine
  `entities_in_file` occurrence semantics.

## 10. Residuals

### R135-A — C# public skeleton fidelity and declared loss

- **Authority chain:** tree-sitter declaration/member nodes -> canonical method,
  field/property, attribute, record and partial evidence -> fidelity projection
  -> public renderer and degradation metadata.
- **Production ownership:** C# compaction/signature capture, canonical type shape,
  hierarchical nodes, and model-facing renderer.
- **Regressions:** a Roslyn-oracle corpus for class/interface methods, returns,
  routing/authorization/parameter attributes, defaults, constraints, XML docs,
  fields/properties, positional records, nested types, and partials at every
  advertised fidelity.
- **Preservation/negative guards:** Low may remain lossy when declared; preserve
  Edit/Verbatim byte exactness, diff snapshot authority, and neutral base truth;
  unsupported syntax must be disclosed, never fabricated.
- **Verification:** focused compaction/IR/MCP tests, provider-disabled context
  harness, final gate, then only unavoidable real-repository field questions.
- **Non-goals:** Roslyn embedding, runtime reflection, or repository-wide fidelity
  redesign without architectural approval.

This slice reaches an Architectural Approval Gate if it changes global fidelity
semantics or the public degradation schema. Begin with a C#-scoped evidence and
corpus repair; stop before any cross-language/public-schema decision.

### R135-B — exact C# callee ownership

Defined in section 6. Coverage is already satisfied independently.

### R135-C — batch-level parameter rejection

Defined in section 7. Explicit rejection is recommended.

### R135-D — file-scope diagnostic distinction

- **Authority chain:** trusted-root/`withinPath` admission -> file resolution ->
  file-query result.
- **Production ownership:** workspace-query file preparation and content response.
- **Regressions:** admitted empty file, existing file outside `withinPath`, outside
  all roots, unsupported file, and deleted file need non-conflated outcomes.
- **Preservation/negative guards:** do not leak unauthorized paths, require CBM,
  or alter occurrence versus declaration semantics.
- **Verification:** focused query tests, provider-disabled harness, final gate.
- **Non-goals:** #142-wide discovery diagnostics or global partial-result policy.

## 11. Closure recommendation

Keep #135 open. Narrow it to R135-A through R135-D, while marking the DI count
cap, nested ownership/kind, graph coverage, per-item batch contract, and separate
declaration query complete or superseded as shown above.

Recommended order: C (isolated request-contract bug), A (establish the public C#
fidelity contract before representation changes), B (callee identity), then D
(bounded diagnostic). A must stop for approval if it requires a global contract;
D must remain local so it does not absorb #142.

## 12. Proposed closure comment if ready

Not applicable. The issue is not ready to close, so no closure comment is drafted.

## 13. Evidence references

- GitHub issue #135 body, read through `gh issue view` on 2026-10-15.
- `docs/plans/ISSUES_133_141_POST_CBM_RECONCILIATION.md`.
- `docs/plans/POST_CBM_133_141_NATIVE_DEFECT_RECONCILIATION.md`.
- `docs/plans/ISSUE_135_DI_OMISSION_REPAIR.md`.
- `docs/plans/ISSUE_137_SEMANTIC_SIGNATURE_EVIDENCE_REPAIR.md`.
- `docs/agent/tooling.md`.
- `verification/issue-reconciliation/README.md` and `Invoke-DiOmission.ps1`.
- `verification/LIVE_GATE_REGRESSIONS.md`.
- Current source and tracked tests cited throughout this report.
- Commits `aa0c3dbf`, `40da966a`, `d4ae2694`, `715902c6`, `96a2280c`,
  `c80bc6ba`, and `d2001cd5`.

## 14. Commands run

- Read the attached task and `docs/agent/tooling.md` with `Get-Content`.
- `gh issue view 135 --repo codeliftsleep2/Clean-CTX --json ...`.
- Clean-CTX `get_cbm_status`, `graph_search`, and batched
  `provide_code_context`. One broad graph search returned no symbols; exact
  symbol searches followed.
- Bounded `rg`/`rg --files`, `git status --short`, `git log`, `git show`, and
  bounded `Get-Content` source ranges.
- Patch utility to create this report. The normal patch tool hit a Windows
  restricted-token write failure; the same patch executable was invoked directly
  with approval. Failed patch attempts changed no repository content.
- `git diff --check`, `git diff -- <report>`, and `git status --short` after edit.

Not run: Cargo, rustfmt, tests, builds, checks, Clippy, binaries, live harnesses,
providers, benchmarks, Claude pilots, or GitHub writes.


