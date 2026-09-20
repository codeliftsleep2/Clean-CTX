# IR Production Integration Audit

**Status:** Phase 9 in progress. This document records production-path
evidence; it is not a completion certificate.

**Started:** 2026-09-20

**Governing contract:**
[`IR_ARCHITECTURE_LOCKDOWN_PLAN.md`](IR_ARCHITECTURE_LOCKDOWN_PLAN.md), Phase 9

## 1. Completion boundary

A family is complete only when evidence covers:

```text
producer
  -> default production compiler, validation, and transformations
  -> returned canonical IR
  -> session and persistent ownership
  -> recompilation, deletion, reset, and replacement
  -> real consumer
  -> registered MCP/API exposure
```

Unit tests and type existence are implementation evidence, not production
integration evidence.

## 2. Confirmed shared production path

| Boundary | Production evidence | Status |
|---|---|---|
| External dispatch | `src/mcp/router.rs` delegates tool calls to `dispatch_tools_call`; `src/mcp/tool_handlers/registry.rs` registers the context, delta, replay, restore, and edit handlers. | Confirmed |
| Default compilation | `IRCompiler::compile_inner` constructs `PassPipeline::default_production()` in `src/ir/compiler.rs`. | Confirmed |
| Pass ordering | Core IR, language layer, meta layer, pattern recognition, alias resolution, then validation in `src/ir/pipeline.rs`. | Confirmed |
| Validation | The production `ValidationPass` consumes the shared exhaustive identity contracts before returning canonical IR. | Confirmed |
| Checked projection | Interactive context handlers call `checked_hierarchy_or_respond`, which maps `try_ir_to_hierarchical` failures to the established MCP error response. | Confirmed |
| Compact LLM projection | `provide_code_context`, `compress_code_context`, and restore paths render the checked hierarchy through the separate LLM renderer. | Confirmed |
| Session owner | `McpState` owns canonical instruction tuples, versions, source hashes, aliases, and durable-path mappings. | Confirmed |
| Persistent owner | Production save paths encode canonical physical binary `0x04` and queue it through `BufferedStore` into SQLite. | Confirmed; Phase 8C user-verified |
| Delta history | `delta_code_context` and incremental `provide_code_context` produce `SequenceDeltaComputer` output; `apply_delta` persists normalized `dv:2` payloads. | Confirmed |
| Reload | `replay_history` loads the `0x04` baseline, checks file identity, replays version-discriminated history, and restores session ownership. | Confirmed |
| Edit preservation | The registered lifecycle regression reloads/replays before exercising byte-exact `apply_edit`. | Confirmed |

Tracked cross-boundary authority is
`src/tests/mcp/persistence_lifecycle.rs`. Codec-only and store-only tests remain
supporting evidence rather than production-integration proof.

## 3. Family evidence

| Family | Producer | Validation/transformation | Projection/consumer | Current audit state |
|---|---|---|---|---|
| Class, method, field definitions | Core capture pipeline | Shared typed identity graph; pattern transformations retain declaration identity | Checked hierarchy and compact renderer | Production path confirmed |
| Parameters, returns, field types | Signature/field capture | Shared target-kind and singularity contracts | Stable-ID checked projection | Production path confirmed |
| Method/class modifiers | Four language declaration layers | Typed vocabulary; declaration-head ownership; modifiers retained through patterns | Dedicated hierarchy fields and compact renderer | Production path confirmed |
| Control summaries | Core capture pipeline | Typed ordered payload; duplicate preserving | Dedicated hierarchy field and compact `ctl:` projection | Production path confirmed |
| Pattern facts | Additive pattern producers | Typed payload; identity-preserving pattern contract | Dedicated hierarchy field and compact pattern projection | Production path confirmed |
| Side effects | Language semantic layers | Closed typed vocabulary; unsafe consumptive patterns decline | Dedicated hierarchy field and compact `se:` projection | Production path confirmed |
| Execution contexts | Language semantic layers | Closed five-value method-scoped vocabulary | Dedicated hierarchy field and compact projection | Production path confirmed |
| Class relationships and metadata | Language/pattern layers | Typed class targeting; ordered occurrence contracts | Stable-ID class projection | Production path confirmed; injectable class metadata remains separately deferred |
| Bodies and spans | Edit-fidelity compiler | Singular body/span validation; conservative pattern guard | Unit table and `apply_edit` | Production path confirmed |
| Delta/replay | Canonical before/after streams | Typed occurrence-aware positional sequence edits | Registered delta/apply/replay MCP handlers | Production path confirmed |
| Binary persistence | Canonical returned IR | Physical `0x04` exact round trip | Buffered SQLite persistence and reload | Production path confirmed |
| Interfaces | TS/Java/C# roots emit `DefInterface` plus interface-owned members | Typed interface ownership and wrong-kind validation | Distinct `InterfaceNode` and compact `Q` projection | Repair implemented; user verification pending |

The confirmed rows remain subject to the remainder of the Phase 9 obsolete-
path and lifecycle audit. They are not final certification.

## 4. Finding P9-01: interfaces silently normalize to classes

**Severity:** High architectural contract violation.

`CoreOp::DefInterface` is produced and validated as an interface identity, but
`src/ir/hierarchical/encode.rs` constructs an ordinary `ClassNode` for it. The
hierarchical model has no explicit interface collection or node kind, so the
LLM consumer cannot distinguish an interface from a class after projection.

This contradicts the approved operation matrix, which requires explicit
interface representation or an unsupported-projection failure and forbids
silent class normalization.

### Affected boundaries

- TypeScript, Java, C#, and any other producer emitting `DefInterface`;
- hierarchical schema and decoder compatibility;
- compact LLM rendering;
- registered context-tool responses;
- binary persistence replay followed by projection.

### Alternatives

1. Add an explicit `InterfaceNode` collection and advance the hierarchical
   schema. This preserves valid interface input and makes the semantic kind
   visible to consumers.
2. Reject `DefInterface` during hierarchical projection as unsupported. This
   is contract-correct but removes current externally visible interface
   context from successful responses.

### Recommendation

Choose option 1: add explicit interface representation, retain interface
identity and members, revise the hierarchy schema, and keep the compact LLM
projection independently minimal. This is externally observable and requires
maintainer approval before implementation.

### Approved direction and newly exposed canonical contradiction

Option 1 was approved on 2026-09-20. Implementation inspection then proved
that P9-01 begins before hierarchical projection:

- the default core capture pipeline routes `interface.root` through the same
  arm as classes, structs, enums, traits, and records;
- that arm allocates a `C*` identity and emits `DefClass`, never
  `DefInterface`;
- there is no production constructor of `CoreOp::DefInterface`; its current
  reachable inputs are decoders and manually constructed IR;
- `DefMethod` and `DefField` require a `ClassId` owner in the approved
  canonical contract and shared validator;
- interface inheritance and declaration facts likewise currently flow
  through class-scoped operations where producers expose them.

Consequently, adding only `InterfaceNode` would create a correct destination
for decoder/test IR while leaving the TypeScript, Java, and C# production
producers unchanged. It would fail the Production Integration Gate and could
not satisfy the approved requirement to preserve real interface members.

The hierarchy-only approval was superseded on 2026-09-20 after this deeper
canonical finding. The maintainer approved explicit interface-owned canonical
declarations rather than a class-or-interface owner union.

#### Implemented repair (verification pending)

- `InterfaceId`, `DefInterfaceMethod`, and `DefInterfaceField` keep interface
  ownership compiler-visible without weakening class contracts.
- `InterfaceModifiers` and `InterfaceExtends` keep interface declaration facts
  out of class-owned semantic families.
- TypeScript, Java, and C# `interface.root` captures now enter that production
  path; no new producer normalizes an interface to `DefClass`.
- Hierarchical revision 8 adds a distinct `InterfaceNode` collection. Older
  approved revisions decode with an empty interface collection and historical
  class nodes are never reinterpreted.
- Physical binary `0x04` adds opcodes 26–29 without changing existing meanings.
- Compact LLM output adds `Q` only when interfaces exist, leaving class-only
  output byte-for-byte unchanged.
- Tracked identity, exact-round-trip, persistence/reload, and registered MCP
  coverage awaits the user-run gate.

## 5. Finding P9-02: buffered deletion transaction composition

**Severity:** High lifecycle correctness defect; repaired during audit.

Buffered saves and clears execute within a batch transaction. SQLite baseline
replacement and file cleanup formerly opened nested transactions. Baseline
replacement was exposed by Phase 8C tests and moved to a savepoint; Phase 9
identified the equivalent queued-clear path and moved cleanup to a savepoint.

Existing tracked authority:
`src/tests/mcp/buffered_store.rs::test_buffered_store_clear` plus registered
restore/reset lifecycle coverage in `src/tests/mcp/persistence_lifecycle.rs`.
User-run verification is pending for the Phase 9 repair.

## 6. Compatibility-path reachability

- Production delta generation uses `SequenceDeltaComputer` only.
- Legacy `IRDelta` remains readable/applicable solely as the approved
  compatibility path.
- Generic method `Flags` has no production producer; it remains a decoder and
  compatibility representation for unknown legacy payloads.
- `ClassFlags` is not legacy-only: the Rust language layer still produces
  residual class metadata, so its production lifecycle remains in scope.

## 7. Next audit action

Phase 9 pauses for user verification of P9-01 and P9-02. Once green, the audit
continues through obsolete-path removal, family-specific registered MCP
evidence, and lifecycle gaps before Phase 10 documentation and final gate.
