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

**Severity:** Test-authority gap; prior repair superseded by P9-10.

The earlier repair routed deletion through `restore_context`. P9-10 established
that restore is durable restoration and must never clear persistence or require
source. The lifecycle regression now proves restore succeeds after source
deletion; public deletion/reset remains a separate lifecycle surface to audit.

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

**Severity:** High externally observable lifecycle contradiction; approved and
implemented pending user verification.

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

Approved option 2 supersedes the original recommendation. `restore_context`
now loads persisted `0x04`, replays durable history, validates the complete
semantic state, and installs session/index ownership only after all checks
succeed. It never recompiles source or clears the checkpoint.

## 15. Finding P9-11: persisted state omits framework semantic edges

**Severity:** High ownership/persistence contradiction; approved and
implemented pending user verification.

Physical `0x04` persists canonical `CoreOp` and `dv:2` persists canonical
sequence changes. Generic method-declaration and call edges can be projected
from those operations, but framework/meta-layer semantic edges are produced
from source captures and live outside `CompiledIR`. SQLite does not currently
persist that edge collection. A durable-only `restore_context` therefore
cannot reinstall the complete workspace-index ownership required by normal
session operation without either silently restoring a subset or recompiling
source, both forbidden by the approved P9-10 contract.

Alternatives:

1. Persist a versioned semantic-edge snapshot with each canonical baseline and
   restore it transactionally with `0x04` plus `dv:2` history. Delta-producing
   paths must replace or advance that snapshot coherently with canonical state.
2. Extend canonical IR so every framework/meta relationship is represented by
   typed operations and derive all edges from canonical IR. This is the
   strongest long-term model but is a much broader semantic-family migration.
3. Restore only generic IR-derived edges. This is smaller but violates the
   approved requirement for normal session-equivalent semantic ownership.

Approved option 1 is implemented as immutable per-version durable edge
snapshots owned by the same context identity, source hash, and semantic version
as canonical IR. Baseline replacement and accepted deltas commit canonical and
edge state in one SQLite transaction. Current and historical replay select the
snapshot matching the replayed canonical version. Restore rejects missing,
malformed, or mismatched state before changing the session or `WorkspaceIndex`.

## 16. Finding P9-12: standalone `dv:2` lacks authoritative target edges

**Severity:** High delta/persistence contract decision; approved and
implemented pending user verification.

Production delta generation compiles the target source and therefore owns the
complete target semantic-edge snapshot. That snapshot can be held pending by
file identity, target version, and source hash until the generated delta is
accepted. The public `apply_delta` path also accepts a standalone client-
supplied `dv:2` payload, however, and that protocol carries canonical sequence
edits only. After restart, or for a delta not generated by the current session,
there is no authoritative framework/meta-layer target edge snapshot. Canonical
IR cannot reconstruct it completely and P9-10 forbids source recompilation.

Alternatives:

1. Require `apply_delta` to possess the server-generated pending edge snapshot
   matching file/from/to/hash whenever durable persistence is enabled. Reject
   standalone or stale deltas structurally rather than applying a state that
   cannot be completely restored.
2. Extend the public delta application contract so the caller supplies a
   versioned complete semantic-edge snapshot plus aligned file/version/hash
   metadata. This supports portable deltas but expands and secures a larger
   externally supplied semantic surface.
3. Allow standalone deltas only as explicitly non-durable session mutations.
   This preserves compatibility but creates two application guarantees and
   requires prominent response/state signaling.

Approved option 1 is implemented. Production delta generation retains the
complete target edges and binds them to durable file identity, from/to version,
the production-emitted `target_hash`, and a digest of the exact `dv:2` payload. Durable application
validates that pending authority, commits delta and target edges atomically,
then installs live canonical/index state and consumes the pending snapshot.
Failure preserves the prior state and pending authority. Portable caller-
supplied edge snapshots remain a separately reviewable future capability.

## 17. Finding P9-13: deletion/reset has no registered lifecycle owner

**Severity:** High externally visible lifecycle gap; repaired and user-verified.

P9-10 correctly removes deletion/reset behavior from `restore_context`, whose
approved sole meaning is transactional durable restoration. The repository has
the necessary cleanup primitives: SQLite context deletion cascades to delta and
per-version semantic-edge ownership, `ContextState` can remove canonical
session state, `WorkspaceIndex` supports occurrence-exact file removal, and
`McpState` can discard fidelity, durable-path, pending-transition, edge, and
compact-render ownership. No registered MCP operation now composes those
primitives into the deletion/reset lifecycle previously (incorrectly) hidden
inside restore.

Alternatives:

1. Introduce an explicitly named file-scoped `delete_context` operation that
   transactionally removes session, workspace-index, pending-transition, and
   durable ownership without touching the source file.
2. Introduce separate `reset_session_context` and `delete_persisted_context`
   operations. This is more explicit but creates two public contracts and a
   caller-visible ordering question when both are desired.
3. Keep cleanup internal only. This leaves no registered production path able
   to satisfy the approved deletion/reset lifecycle requirement.

Recommendation: option 1. A single file-scoped deletion contract matches the
existing ownership aggregate and can remove all associated state coherently.
It must remain distinct from source-file deletion and from durable restore.

**Implementation update (2026-09-20):** The approved registered
`delete_context(filePath)` operation uses durable SQLite deletion as its commit
boundary, then removes only the matching canonical session IR, source hash,
fidelity, alias/durable mapping, pending target-edge authority, semantic-edge
snapshot, compact render, and occurrence-exact `WorkspaceIndex` ownership. The
source file is never mutated. Registered-dispatch coverage includes successful
cascade deletion, peer isolation, mismatched ownership, durable failure
transactionality, truthful response metadata, and restore-after-delete failure.

## 18. Exhaustive audit checkpoint

The remaining audit began from registered dispatch and traced the first
state-producing operations before moving outward to the complete operation and
semantic-family matrix. The following rows are current production evidence,
not final certification:

| Registered operation | Canonical/validation path | Live and durable ownership | Consumer/exposure | Current status and bypass review |
|---|---|---|---|---|
| `compress_code_context` | Default production compilation followed by checked hierarchy | Installs canonical session state, fidelity, complete semantic edges, and `WorkspaceIndex` ownership; then writes physical `0x04` plus the edge snapshot | Compact renderer plus structured hierarchy in the registered response | Checked projection and non-empty canonical persistence are present. P9-14 blocks certification because live publication precedes durable commit. |
| `delta_code_context` | Default production compilation; corrected `SequenceDeltaComputer` emits `dv:2` | Existing baselines are checked against durable state; target edges are retained as pending authority. The first baseline installs live ownership before persisting it. | Registered compact delta response | No production legacy-delta emission was found. P9-14 blocks first-baseline transactionality. |
| `apply_delta` | Strict `dv:2` tuple validation and checked replay; legacy input is compatibility-only | Durable mode requires the exact server-owned pending target-edge snapshot and commits canonical delta plus edges before live installation | Registered response renders the accepted canonical target | Corrected production path confirmed. Durable legacy or standalone `dv:2` input without pending authority fails structurally. |
| `save_context` | Strict reconstruction of the requested session-owned canonical stream | File-scoped physical `0x04` and complete edge snapshot checkpoint | Truthful registered saved/already-durable response | P9-08 user-verified; no substitute-file or empty-IR path found in the handler. |
| `restore_context` / `replay_history` | Physical `0x04`, checked `dv:2` replay, semantic-state validation, checked hierarchy | Durable state is validated completely before session and `WorkspaceIndex` installation | Compact and structured registered response | P9-10 through P9-12 user-verified; no source-recompile fallback remains. |
| `delete_context` | Resolves exact session/durable ownership before deletion | SQLite deletion is the commit boundary; live canonical, edge, alias, hash, fidelity, compact, and pending ownership is removed afterward | Truthful registered deletion response | P9-13 user-verified; source files are not mutated. |
| `apply_edit` | Requires matching disk/live/durable identity, then compiles and checked-projects exact candidate bytes | Durable recovery intent, exact source replacement, aligned `0x04`/edge commit, then live publication | Registered byte-exact edit response | P9-15/P9-16 implemented pending user verification; full-body fallback remains production-covered. |

Static bypass inspection also found that MCP production hierarchy call sites use
the checked projection boundary; production tuple reconstruction is fallible
rather than `filter_map`-lossy; and corrected production delta generation does
not emit the legacy delta representation. The generic optional-IR persistence
trait remains in the storage layer, but no registered production save caller
was found passing absent IR during this checkpoint. The audit remains
incomplete and will resume after P9-14 is resolved and user-verified.

## 19. Finding P9-14: baseline handlers publish live state before durable commit

**Severity:** High transactional lifecycle contradiction; repaired and
user-verified.

When persistence is enabled, `compress_code_context` installs the newly
compiled canonical IR, source hash, fidelity, complete semantic-edge snapshot,
and replacement `WorkspaceIndex` edges before calling
`save_context_with_semantics`. If that atomic write fails, the handler returns
a persistence error but leaves the new live state installed.

The first-baseline branch of `delta_code_context` has the same ordering. It
loads canonical IR, replaces index edges, records semantic edges and fidelity,
and only afterward calls `persist_baseline`. A failed baseline write therefore
reports failure while the session already owns the candidate state. This is
inconsistent with the transactional commit boundary already enforced for
restore, accepted durable deltas, and deletion, and permits live and durable
owners to describe different semantic versions after an unsuccessful request.

The contradiction affects initial compilation/checkpoint creation through two
registered operations, source hash and fidelity ownership, framework/meta edge
ownership, subsequent delta versioning, later explicit save/restore behavior,
and the truthfulness of the MCP failure response.

Alternatives:

1. Treat durable commit as the publication boundary whenever a registered
   operation promises to persist its newly compiled baseline. Build the
   candidate canonical and edge state off-session, commit physical `0x04` plus
   the edge snapshot, and install all live ownership only after success.
2. Snapshot and roll back every affected live owner if persistence fails. This
   can preserve behavior but is more complex and exposes additional race and
   partial-rollback surfaces.
3. Define durable persistence as best-effort after successful live publication
   and return success with explicit non-durable status when it fails. This is a
   new two-tier public guarantee and conflicts with the handlers' current
   structural failure response.

Recommendation: option 1. It matches the approved restore, delta-application,
and deletion transaction model and gives each request one observable commit
boundary. Session-only operations that do not promise persistence can remain
separate; this decision need not make every compilation an automatic durable
checkpoint.

**Implementation update (2026-09-20):** Both registered baseline paths now
compile and validate candidates without creating session alias ownership. When
persistence is enabled, physical `0x04` and the complete aligned semantic-edge
snapshot commit before the candidate receives an alias or changes canonical
session state, source hash, fidelity, `WorkspaceIndex`, edge, durable-path, or
compact ownership. The exact committed candidate is then published live.
Empty durable compact metadata is treated as absent and deterministically
rendered from the checked restored model, preserving the registered compact
response while keeping compact presentation separate from the semantic commit.
Tracked registered-dispatch regressions inject file-specific atomic-save
failures and assert preservation of prior and peer ownership.

## 20. Finding P9-15: `apply_edit` splits source, live, and durable authority

**Severity:** High contradiction; Option 1 repaired and user-verified.

The registered `apply_edit` contract describes an atomic controlled edit. The
production handler validates and syntax-checks the complete candidate in
memory, but then writes the source file before compiling the final canonical
state. If that post-write production compilation fails, the handler logs a
warning, retains the prior session version and semantic ownership, and still
returns an edit-success response. Disk and authoritative live state then
describe different programs.

When post-write compilation succeeds, the handler replaces canonical session
IR, source hash, fidelity, semantic edges, and `WorkspaceIndex` ownership but
intentionally leaves the persisted physical `0x04` baseline and semantic-edge
snapshot unchanged. This does not merely defer an optimization. A subsequent
`delta_code_context` sees the edited source hash already installed in live
state and can return the new stream as cached without generating or persisting
a transition. `restore_context` can then restore the old durable semantics over
the newly edited source. Explicit `save_context` can repair the split, but the
successful edit response neither requires nor signals that extra operation.

The contradiction affects byte-exact source authority, canonical versions,
source hashes, complete semantic-edge snapshots, `WorkspaceIndex`, pending
transition identity, later delta generation, save/restore behavior, restart
rehydration, and the truthfulness of the registered edit response.

Alternatives:
1. Treat the complete edit as a staged source-and-semantic transaction. Build
   and validate the post-edit canonical IR and complete edge snapshot from the
   in-memory candidate before writing. When persistence is enabled, commit the
   corresponding durable semantic transition before publishing live state and
   return success only when source, durable state, and live ownership all
   advance coherently. This requires an explicit recoverable strategy for the
   filesystem write and SQLite commit, which cannot share one native
   transaction.
2. Session-only invalidation and 3. caller-visible stale durability were
   rejected because both weaken the coherent immediate-edit contract.

Recommendation: option 1, with a bounded staged-edit protocol: compile and
checked-project the in-memory candidate first; write the source through an
atomic replace with a retained recovery copy; durably commit the exact
canonical/edge target; publish live state only after both commits; and restore
the prior source if durable commit fails. This is compensation at the
filesystem boundary because SQLite and the filesystem cannot share a native
transaction, not compensating rollback of prematurely published live semantic
state. The precise recovery and crash-consistency contract requires approval
before implementation.

The approved contract additionally makes byte identity authoritative: candidate
validation, atomic replacement, durable `Body`/span ownership, recovery, and
restart must all use the exact UTF-8 byte sequences, preserving BOM, line
endings, whitespace, multibyte content, and every byte outside the requested
edit span. No source regeneration or normalization is permitted.

## 21. Finding P9-16: pre-edit source authority can already be split

**Severity:** High contradiction; Option 1 repaired and user-verified.

`apply_edit` deliberately recompiles the current on-disk bytes to relocate unit
spans, even when those bytes changed outside Clean-CTX after the session
baseline was established. The approved P9-15 protocol, however, requires a
recoverable prior state: exact prior source bytes, prior canonical/version/hash,
and prior durable semantic edges must identify the same program before an edit
intent can be established.

If current source bytes do not match the live and durable source hash, neither
permitted P9-15 failure outcome is coherent without another policy. Restoring
the exact pre-edit bytes after a target durable-commit failure would still leave
live/durable semantics describing an older byte sequence. Advancing durable or
live state to the externally changed source before applying the requested edit
would itself be an externally observable reconciliation transition not covered
by the approved edit transaction.

Affected contracts include stale-edit detection, existing unit relocation,
external editor interoperability, canonical version authority, recovery intent
identity, and whether `apply_edit` may implicitly checkpoint changes it did not
make.

Alternatives:
1. Require the exact current source hash to match both live and durable prior
   ownership before `apply_edit` may start. Reject a mismatch structurally and
   require an explicit refresh/checkpoint operation first.
2. Make `apply_edit` first compile, validate, and transactionally reconcile the
   externally changed current source into durable and live ownership, then start
   a second edit transaction. This preserves relocation convenience but lets an
   edit request implicitly adopt unrelated external changes.
3. Treat current disk bytes as prior solely for recovery while retaining older
   semantic ownership. This preserves current behavior but violates the
   approved requirement that source, durable semantics, and live semantics
   remain aligned on every failure boundary.

Recommendation: option 1. It gives the staged transaction one unambiguous prior
authority, prevents `apply_edit` from silently adopting unrelated changes, and
keeps explicit source refresh/checkpoint semantics separate. The structural
failure should report the expected and actual source hashes and must not create
an edit intent or mutate source, durable, or live state.

**Implementation update (2026-09-20):** `apply_edit` now requires exact disk,
live, and durable source-hash agreement before candidate compilation. It
compiles and checked-projects exact in-memory candidate bytes, records a durable
recovery intent, atomically replaces the source, atomically commits aligned
physical `0x04` and complete semantic edges, and only then publishes the exact
target live. Durable failure restores the exact prior bytes without changing
live ownership. Restore deterministically resolves an interrupted intent before
loading durable state. Full-body fallback and byte-addressed `Body`/`UnitTable`
semantics remain authoritative and have registered production-path coverage.

## 22. Verification result and next audit action

P9-10 through P9-16 were user-verified green. The exhaustive Phase 9
production operation/lifecycle, semantic-family, and obsolete-path matrix audit
resumed in
[`IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION.md`](IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION.md).
Phase 9 is not certified.
