# Live handoff — Architectural Hardening branch

**Branch at handoff:** `(feat)Architectural-Hardening`  
**Baseline:** `main`  
**Purpose:** give the operator using this branch on a real repository one
code-grounded explanation of why it exists, what changed, and what behavior to
watch during live work.

## Authority and scope

This is a narrative handoff, not an architectural authority. When this file
and implementation differ, the production implementation wins. The primary
code authorities are:

- `src/ir/opcodes.rs` and `src/ir/opcodes/semantic.rs` — canonical fact types;
- `src/ir/compiler.rs`, `src/ir/pipeline.rs`, and `src/mcp/tool_helpers.rs` —
  production compilation;
- `src/ir/hierarchical.rs` and `src/ir/identity/**` — checked projection and
  identity rules;
- `src/ir/render_llm.rs` and `src/mcp/tool_handlers/core/content.rs` —
  model-visible file presentation;
- `src/ir/binary_wire.rs`, `src/mcp/persistence_ir.rs`, and
  `src/mcp/sqlite_store/**` — durable representation;
- `src/ir/delta/sequence.rs` and `src/mcp/tool_handlers/core/delta/**` — delta
  computation and lifecycle;
- `src/layers/registry.rs` and `src/ir/pipeline/meta_layer.rs` — production
  meta-layer dispatch;
- `src/workspace/**` and `src/mcp/tool_handlers/query/**` — workspace facts and
  on-demand queries.

Tracked tests under `src/tests/**` are the executable contracts. Other design,
research, and ADR documents explain intent and history but do not override the
code.

## Executive summary

This branch began as a hardening of the compiler IR beyond the SCHEMA-v2 era.
The work found that several concepts had been conflated:

1. the canonical semantic model used for correctness;
2. the durable format used for persistence and replay;
3. the application-facing structured result;
4. the compact text shown to an LLM.

An intermediate implementation made a reversible CONTROL-FULL/COMPACT codec
the model-visible response. That was architecturally wrong and empirically
inefficient: the codec carried canonical IDs, grammar machinery, occurrence
framing, and a decode contract that the model did not need. The branch was
corrected so each representation has one job.

The current result is:

- a typed, identity-checked, occurrence-preserving canonical IR;
- binary `0x04` plus semantic-edge snapshots as durable authority;
- checked `SequenceDelta`/`dv:2` transport on the code side;
- SCHEMA-v5 as the compact model-visible structural presentation;
- WorkspaceIndex for cross-file graph facts;
- `calls_in_file` for the narrow owner/overload detail that global Model-C
  method identity cannot represent;
- production meta-layers that honor the active runtime configuration;
- explicit response metadata describing what the model actually received.

## The SCHEMA-v2 starting point on `main`

The `main` implementation in `src/ir/render_llm.rs` already had several good
properties. SCHEMA-v2 was a stateless name-oriented projection, did not expose
internal `C1`/`M1` aliases, grouped members under classes, used `+N` for
same-name overloads, and could include exact bodies at Edit fidelity.

The limitation was not simply that its text syntax was old. The canonical and
hierarchical models underneath it could not express or reliably preserve all
of the distinctions later work required:

- interfaces were not first-class hierarchical owners with their own members;
- known declaration modifiers and behavioral facts shared broad string flag
  channels;
- side effects and execution contexts were stringly typed and singular;
- repeated operations could be flattened or overwritten instead of retaining
  occurrence boundaries and duplicates;
- owner and payload identity were not checked consistently across producers,
  projection, codecs, persistence, and replay;
- the binary persistence format was `0x03`, predating the complete semantic
  operand set now represented by the canonical IR;
- global method query identity intentionally used `(builtin, Method, name)`,
  which could not distinguish same-named owners or overloads inside one file.

SCHEMA-v2 was therefore a useful presentation baseline, but it could not be
the correctness model for the expanded compiler architecture.

## Why the branch became larger than a renderer upgrade

The branch traced facts through their entire production lifecycle:

```text
source
  -> language extraction
  -> typed CoreOp stream + semantic edges
  -> checked hierarchical projection
  -> session ownership and WorkspaceIndex publication
  -> binary persistence / delta replay
  -> MCP response and model-visible presentation
```

That trace exposed gaps that isolated unit tests or custom pipelines did not:

- some facts existed in extractors but were not represented canonically;
- some canonical facts did not survive hierarchical projection;
- some facts survived projection but not binary persistence or replay;
- meta-layer registration existed, but runtime enable/disable configuration
  was not consistently propagated through the production compiler;
- fidelity and visible-content metadata could change across persistence and
  restart;
- delta acknowledgements could be generated from the wrong source snapshot;
- documentation sometimes described intended architecture rather than actual
  production reachability.

The remediation therefore followed facts from producer to live consumer and
added regression coverage at the real registered MCP paths.

## Current representation architecture

```text
                         +-> SCHEMA-v5 / raw fallback -> LLM content
                         |
source -> CompiledIR ----+-> reduced structured IR ----> application view
       + SemanticEdge[]  |
                         +-> binary 0x04 + edges ------> persistence/replay
                         |
                         +-> WorkspaceIndex ----------> cross-file queries
                         |
                         +-> read-only candidate ------> calls_in_file
```

### 1. Canonical semantics

`CompiledIR` and its `CoreOp` stream are the semantic authority. The branch
added or hardened first-class operations for:

- interface-owned methods and fields;
- method, class, and interface declaration modifiers;
- typed control summaries;
- typed method-pattern facts;
- typed side-effect and execution-context values;
- interface inheritance;
- structural calls with explicit written argument count and spread evidence;
- exact method bodies and spans for editing.

Projection now validates typed ownership, ID kind, payload shape, reference
targets, cardinality, and occurrence behavior. Invalid identity is returned as
a structured error instead of silently producing a plausible hierarchy.

### 2. Durable authority

`src/ir/binary_wire.rs` now emits binary version `0x04`, described in code as
the complete `CompiledIR` identity and semantic-operand format. Persistence
normalizes session aliases to durable file identity, stores the binary IR with
the aligned semantic-edge snapshot, and restores both transactionally.

Binary `0x04` did not replace SCHEMA-v5 or a former COMPACT-FULL presentation.
It superseded older binary persistence versions for the expanded canonical
model. It is intentionally non-human-readable and code-side.

### 3. Model-visible presentation

`src/mcp/tool_handlers/core/content.rs` routes structural model content through
`render_hierarchical_for_llm`, whose current header is SCHEMA-v5.

SCHEMA-v5 retains SCHEMA-v2's compact, name-oriented structure and adds the
distinctions supported by the hardened canonical model:

- explicit interface sections and interface-owned members;
- `mod:` method modifiers and `cmod:` class modifiers;
- `ctl:` typed control summaries;
- `pf:` typed pattern facts;
- residual `fl:` legacy metadata kept separate from known typed families;
- `cl:` residual class metadata;
- typed side-effect and execution-context annotations;
- typed ownership, signatures, imports, aliases, inheritance, and patterns;
- exact full or focused bodies only when fidelity promises them.

It still omits code-side machinery such as canonical IDs, navigation tables,
occurrence framing, binary grammar, and workspace graph payloads.

The local token-economics boundary may return byte-exact raw source when the
structural candidate is not safely cheaper. Focused Edit is an exception: it
must preserve the resolved focused-body contract and cannot substitute the full
raw document.

#### Does SCHEMA-v5 provide information that raw source does not?

Yes, in the limited and important sense that it makes compiler-derived facts
explicit. Raw source contains the evidence from which these facts are derived,
but usually does not state the normalized conclusions directly. Depending on
fidelity and available evidence, SCHEMA-v5 can expose:

- normalized declaration modifiers through `mod:` / `cmod:` / `imod:`;
- typed control summaries through `ctl:`;
- recognized method-pattern facts through `pf:`;
- control-flow and data-flow summaries through `cf:` and `df:`;
- classified side effects and execution contexts through `se:` and `ec:`;
- normalized interface, overload, ownership, inheritance, import, alias, and
  meta-layer structure.

This is semantic enrichment, not external knowledge: every fact must be
supported by source and compiler/meta-layer evidence. Its value is that the
model receives the conclusion compactly instead of having to rediscover it by
reading every body.

The stronger statement once associated with the CONTROL-FULL/COMPACT work is
not valid for SCHEMA-v5. SCHEMA-v5 is intentionally a lossy presentation, not a
reversible or correctness-complete encoding of canonical IR. It omits canonical
IDs, occurrence framing, navigation machinery, the complete call table,
workspace graph facts, persistence grammar, and delta transport. It may also
collapse redundant annotations—for example reporting async once at the most
structural available level—without deleting the underlying canonical facts.

Therefore the precise contract is:

> SCHEMA-v5 can contain useful normalized and derived semantic facts that are
> not textually explicit in raw source, but it is not a lossless replacement
> for canonical IR.

When token economics selects raw passthrough, the model receives the byte-exact
source instead of these SCHEMA-v5 annotations for that response. The canonical
facts remain code-side, and the appropriate workspace facts remain available
through on-demand queries.

### 4. Application-facing structured view

The reduced `result.ir` hierarchy remains useful to application consumers, but
it is not the durable or correctness authority. Reduction is allowed to omit
facts that canonical binary persistence retains.

### 5. Workspace facts

WorkspaceIndex stores cross-file entity and relationship evidence. The six
existing operations retain Model-C identity and scoped hydration behavior:

- `find_entities`;
- `forward_edges`;
- `reverse_edges`;
- `entities_in_file`;
- `transitive_dependencies`;
- `has_cycle`.

Candidate discovery can use CBM or filesystem fallback, but candidates are
compiled by Clean-CTX; discovery never supplies authoritative semantic edges.
`workspaceRoot`, configured additional roots, provenance, and optional
`withinPath` determine the effective query scope.

The seventh operation, `calls_in_file`, answers a different question. It
compiles an unpublished High-fidelity canonical candidate for one trusted file
and selects a typed owner and method. It preserves overload order, call order,
duplicates, written callee spelling, written argument-node count, and spread
evidence. It does not hydrate or publish WorkspaceIndex/session state, expose
canonical IDs, resolve the callee, or change global Model-C identity.

## Main improvements over the SCHEMA-v2 iteration

### Identity and information preservation

- Typed owner relationships are validated instead of inferred late.
- Interface identity survives compilation, projection, binary persistence,
  restore, and presentation.
- Repeated facts retain operation boundaries, order, and duplicates.
- Known semantic families no longer compete in one ambiguous flag bucket.
- Malformed projection and delta tuples fail explicitly and transactionally.

### Presentation quality

- SCHEMA-v5 exposes richer reasoning facts without exposing storage grammar.
- Model content uses names and ownership rather than internal aliases.
- Content metadata is a typed `ContentKind` derived from the visible result.
- `byte_exact` identifies the document or bodies that are genuinely safe for
  exact editing.
- Redundant async annotations collapse at presentation time: structural
  `mod:ASYNC` wins over duplicate `se:async`/`ec:async`, without deleting the
  canonical facts.
- Measured High-fidelity compression after that collapse was 67.1% for the
  TypeScript fixture, 61.8% for Angular, and 43.9% for C#.

### Persistence and lifecycle correctness

- Binary `0x04` stores the expanded canonical instruction set.
- Canonical IR and semantic edges commit atomically.
- Fidelity survives durable restore and restart.
- `auto_save` controls ordinary read checkpoints; Edit still requires a
  durable checkpoint because exact editing depends on recoverable state.
- Publication follows durable commit where policy requires persistence.
- Restore, replay, deletion, and edit recovery update the live owners
  transactionally.
- Read-only history, stats, and file-local inspection remain observational.

### Delta correctness

- Positional `SequenceDelta` preserves occurrence identity and order.
- The structured delta is code-side; the LLM sees only a small summary such as
  `Δ delta for ...`, unless the economics boundary explicitly selects raw
  source.
- Missing durable baselines are repaired without silently changing fidelity.
- Delta generation, application, restart restoration, and explicit
  acknowledgement share the same canonical lifecycle.
- Post-commit acknowledgements compile the committed source snapshot rather
  than a stale or pre-commit source.

### Production meta-layers

- The production compiler installs language layers by detected extension.
- `IRCompiler` invokes the global `LayerRegistry` for framework semantics.
- The active `CleanCtxConfig` reaches applicability and enrichment, so Angular
  and .NET enable/disable settings are honored on the real MCP path.
- The always-on builtin layer supplies ordinary declaration entities without
  taking authority away from framework-specific layers.

### MCP contract clarity

- Visible content kind and byte-exact coverage describe the actual response.
- SCHEMA-v5 vocabulary is taught in the system/cache prompt rather than the
  research codec grammar.
- Workspace-query answers use the canonical MCP content plus
  `structuredContent` envelope.
- Raw source, Angular template output, structural presentation, and delta
  summaries are explicitly classified rather than treated as one format.

## The corrected wrong turn: CONTROL-FULL and COMPACT-A

The branch built CONTROL-FULL and COMPACT-A variants to establish a reversible
correctness oracle and measure lossless codecs. Those artifacts are valuable
for research and verification, but wiring them directly into model-visible
`content` was a category error.

The reversible formats include information needed by a decoder—canonical IDs,
grammar/version markers, positional framing, and occurrence structure. The
production model does not decode that format. Measurements also showed that
some variants were larger than raw source or materially worse than SCHEMA-v5.

The correction was not to delete the research. It was to restore the boundary:

- normalized CONTROL-FULL: regenerated correctness oracle;
- COMPACT-A/A3: research codecs and measurement fixtures;
- binary `0x04`: physical durable authority;
- SCHEMA-v5: production LLM presentation.

No production claim should call these interchangeable.

## Deliberate non-changes

- Delta transport remains entirely code-side; no LLM delta protocol was added.
- Global WorkspaceIndex method identity was not changed to include owner or
  signature. `calls_in_file` solves only the proven local ambiguity.
- CBM remains a candidate-discovery aid, not semantic authority.
- Option C—the next purpose-built presentation—is not implemented by this
  branch. `calls_in_file` is a prerequisite that makes one Option-C omission
  safe to evaluate later.
- Research codecs are not presented as production compression formats.

## How the work was carried out

The branch followed an incremental preservation strategy:

1. identify the existing production behavior on `main`;
2. add typed canonical operations without changing external behavior;
3. make every language producer emit the new ownership and semantic facts;
4. harden checked hierarchical projection and compatibility migration;
5. extend binary encoding/decoding to `0x04`;
6. integrate persistence, replay, restore, deletion, and edit transactions;
7. trace production MCP handlers rather than relying on custom pipelines;
8. separate reversible codecs from model presentation after measurement;
9. restore SCHEMA-v5 as the production presentation and align metadata;
10. repair configuration, fidelity, auto-save, delta acknowledgement, and
    source-snapshot lifecycle defects found by the final audit;
11. add the narrow `calls_in_file` operation without widening global identity.

Regression tests live under `src/tests/**` and are registered from their
production modules. They cover producers, canonical projection, round trips,
binary `0x04`, persistence/restart, edit recovery, delta fidelity, content
classification, meta-layer configuration, workspace scoping, and file-local
call inspection.

## Live-use expectations

This branch is intended to be exercised through normal work on a large real
codebase. During live use, the important question is not whether a marker is
present in isolation; it is whether the complete capability remains reachable
and trustworthy through Claude's real MCP workflow.

Healthy behavior should look like:

- ordinary context arrives as SCHEMA-v5 or a declared raw/template form;
- Edit/focused requests expose only the regions declared byte-exact;
- repeated reads can use compact code-side deltas without asking the model to
  reconstruct state from a delta stream;
- restart/restore retains fidelity, canonical state, and workspace semantics;
- disabled framework meta-layers stay disabled;
- workspace queries respect repository scope and provenance;
- owner-sensitive local call questions use `calls_in_file` and do not mix
  same-named owners or overloads;
- failures are explicit rather than silently producing partial identity.

When a live regression appears, capture the natural task, tool request,
response, relevant file, expected behavior, and whether restart/persistence
was involved. Reproducible findings should become tracked tests; genuinely
scale-dependent findings belong in `docs/agent/DISCOVERY_REGISTRY.md`.

## Current verification and next boundary

At this handoff, the operator reported the requested formatter, focused
`calls_in_file` and workspace-query content tests, and all-feature Clippy run
green. Repository release verification remains governed by
`docs/agent/verification.md`; this document does not redefine that gate.

The next architectural work is not another correction to this branch. It is a
separate decision: evaluate the branch during real Claude work, record any
capability regressions, and only then decide whether to implement Option C's
purpose-built presentation. Option C must be authored against the current
canonical boundaries, not by reviving SCHEMA-v2 assumptions or exposing the
durable codec to the model.

## Reference documents

These are useful explanations, plans, and historical records after inspecting
the code:

- `docs/ARCHITECTURAL_INVARIANTS.md`;
- `docs/architecture/CORE_OP_CONTRACT_MATRIX.md`;
- `docs/architecture/BINARY_V04_CONTRACT.md`;
- `docs/architecture/IR_PROJECTION_RECONNAISSANCE_REPORT.md`;
- `docs/architecture/LLM_CONTEXT_COMPRESSION_RESEARCH.md`;
- `docs/architecture/FORWARD_THINKING_OPTION_C.md`;
- `docs/agent/CODEX_HANDOFF.md`;
- `docs/agent/verification.md`.
