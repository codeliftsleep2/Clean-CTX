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
supporting evidence rather than production-integration proof. Its registered
dispatch coverage includes a real producer fixture for migrated modifiers,
control summaries, pattern facts, side effects, execution contexts, data flow,
and control flow, followed by physical `0x04` persistence and MCP replay. The
regression checks compact low-fidelity markers only for families that renderer
contract exposes at that fidelity; side effects and execution contexts are
asserted through the structured MCP hierarchy and durable canonical IR.

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
| Interfaces | TS/Java/C# roots emit `DefInterface` plus interface-owned members | Typed interface ownership and wrong-kind validation | Distinct `InterfaceNode` and compact `Q` projection | Production path confirmed; user-verified |

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

#### Implemented and verified repair

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
  coverage was user-verified on 2026-09-20.

## 5. Finding P9-02: buffered deletion transaction composition

**Severity:** High lifecycle correctness defect; repaired during audit.

Buffered saves and clears execute within a batch transaction. SQLite baseline
replacement and file cleanup formerly opened nested transactions. Baseline
replacement was exposed by Phase 8C tests and moved to a savepoint; Phase 9
identified the equivalent queued-clear path and moved cleanup to a savepoint.

Existing tracked authority:
`src/tests/mcp/buffered_store.rs::test_buffered_store_clear` plus registered
restore/reset lifecycle coverage in `src/tests/mcp/persistence_lifecycle.rs`.
The Phase 9 repair was user-verified on 2026-09-20.

## 6. Finding P9-03: duplicate unchecked hierarchy projection

**Severity:** Medium production error-path inconsistency; repaired during audit.

`compress_code_context` first used `try_ir_to_hierarchical` through the shared
MCP error mapper, but then discarded that checked result when constructing its
hierarchical wire field and invoked the panic-based convenience projector a
second time. The response now wraps the already-checked hierarchy in the same
wire envelope. Production therefore has one projection result and one
structured failure path; the convenience API remains available to internal
callers and tests that already possess valid canonical IR.

## 7. Finding P9-04: abstract TypeScript classes absent from production IR

**Severity:** High production reachability defect; repaired during audit.

The TypeScript grammar represents an abstract class with
`abstract_class_declaration`, while the production query captured only
`class_declaration`. A valid abstract class therefore produced no canonical
definition or members. The query now routes both declaration node kinds
through the same `class.root` production path. The registered lifecycle
regression protects the abstract class, its typed class modifier, its members,
and its downstream persistence and MCP exposure.

## 8. Compatibility-path reachability

- Production delta generation uses `SequenceDeltaComputer` only.
- Legacy `IRDelta` remains readable/applicable solely as the approved
  compatibility path.
- Generic method `Flags` has no production producer; it remains a decoder and
  compatibility representation for unknown legacy payloads.
- `ClassFlags` is not legacy-only: the Rust language layer still produces
  residual class metadata, so its production lifecycle remains in scope.

## 9. Finding P9-05: replay restored state without exposing semantics

**Severity:** High production lifecycle defect; repaired during audit.

The registered `replay_history` path decoded and restored canonical IR but
returned only an instruction-count status sentence. No checked projection or
semantic response consumed the restored state. Replay now loads the persisted
fidelity, applies checked projection, refreshes the compact cache, and returns
both compact semantic content and the current hierarchical envelope. Projection
failures use the same structured MCP error mapping as compilation paths.

## 10. Finding P9-06: TypeScript export wrappers lose declaration ownership

**Severity:** High producer-ownership gap; repaired pending user verification.

Tree-sitter TypeScript places `export` outside the captured class or interface
declaration node. The language layer correctly recognizes `export` when given
the complete declaration head, but the default production query does not give
it that wrapper. Consequently, direct layer tests can pass while real exported
classes and interfaces omit their typed `Export` modifier. This is distinct
from P9-04: abstract declarations now have an explicit structural capture,
while export ownership still requires wrapper-to-declaration association
without duplicating the declaration capture.

The production query now captures declaration children directly from export
wrappers. The core producer joins that marker to the ordinary class/interface
capture by exact byte span and emits the typed owner-specific `Export`
modifier. Named/default-exported classes, exported interfaces, non-exported
declarations, and misleading descendant text have tracked production-compiler
coverage; no surrounding-text inference was introduced.

## 11. Finding P9-07: deletion evidence bypassed registered production path

**Severity:** Test-authority gap; repaired during audit.

The lifecycle regression deleted the source and then called the persistence
store directly. That proved SQLite cleanup but not production integration.
The regression now invokes registered `restore_context` after source deletion.
That path clears session IR, path ownership, compact cache, workspace-index
provenance, and persistent state before the expected source-read failure. The
handler invalidates its source cache before that read so a deleted file cannot
be resurrected from stale cached content.

## 12. Finding P9-08: `save_context` does not implement its public contract

**Severity:** High externally observable contract contradiction; repaired
pending user verification.

The registered tool and documentation describe `save_context(filePath)` as an
explicit checkpoint of the requested in-memory context. Its handler ignores
`filePath`, calls `BufferedStore::flush()`, ignores the returned flush count,
and always reports `saved: 1` whenever persistence is enabled. Because normal
production save paths immediately flush, this commonly reports a successful
save when no write occurred.

The approved repair implements the original file-scoped contract. Session
state now owns fidelity alongside canonical IR and source hash. The handler
resolves the requested session alias and exact durable path, rejects missing or
mismatched ownership, strictly reconstructs canonical operations, reuses the
session compact output, encodes durable physical `0x04`, and verifies the
requested SQLite checkpoint after buffered persistence. An identical existing
checkpoint reports `saved: 0, already_durable: true`; a newly written and
verified checkpoint reports `saved: 1`. Registered-dispatch coverage proves
that another file cannot substitute and that replay restores the exact stream.

## 13. Finding P9-09: malformed session tuples can be silently discarded

**Severity:** High externally observable replay and delta contradiction;
repaired pending user verification.

Production delta computation converts session tuples back to `CoreOp` with
`filter_map`, silently dropping any tuple that `tuple_to_op` rejects. Durable
replay performs the same lossy conversion after applying persisted deltas.
Corrected `dv:2` occurrence validation proves positional identity but does not
fully validate tuple arity or operand syntax; the legacy compatibility path can
also carry malformed replacements/additions. A malformed stream can therefore
be accepted into session history and later appear to replay successfully with
fewer operations.

The approved repair makes tuple-to-canonical conversion fallible at production
boundaries. Corrected sequence insertions and replacements are validated before
mutation; legacy additions, replacements, and patches validate the complete
candidate before commit. Session delta generation, baseline persistence, and
durable replay now fail structurally on the first invalid tuple. Tracked tests
cover transactional sequence and legacy rejection, registered MCP application,
and persisted replay. No path drops, repairs, or synthesizes malformed tuples.

## 14. Finding P9-10: `restore_context` has contradictory public semantics

**Severity:** High externally observable lifecycle contradiction; architectural
approval required.

The registered tool describes restoring compressed context, and
`docs/agent/tooling.md` says it restores a previously persisted context from
the database. The production handler instead clears session and persistent
ownership, then recompiles source; developer documentation describes that
reset/recompression behavior. On successful recompilation the handler renders
the result but does not install canonical IR, source hash, fidelity, durable
path ownership, or semantic edges back into session state. Its response can
therefore claim restoration while leaving no restored production owner.

Alternatives:

1. Define `restore_context` as reset plus authoritative full recompilation.
   Install the new canonical/session state and workspace edges, keep the
   explicit database clear, and make tool/docs wording say reset/recompile.
2. Define it as persisted restore. Load physical `0x04` plus history like the
   replay path, without deleting the requested checkpoint or requiring source.
3. Split the behaviors into separately named tools. This is clearest but adds
   a new public MCP contract and compatibility/migration work.

Recommendation: option 1 preserves the established production behavior and
developer contract while making its successful result real. `replay_history`
already owns persisted restoration. The repair should retain deletion cleanup,
install every recomputed owner atomically on success, and align the public tool
and tooling documentation with reset/recompile semantics.

## 15. Approval gate and next audit action

Phase 9 pauses at P9-10's approval gate. P9-09 remains pending user-run
verification and is not Phase 9 certification.
