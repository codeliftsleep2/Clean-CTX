# A3 file-context vs workspace_query boundary map

**Status:** investigation complete; plan (pending maintainer decision)
**Question:** what belongs in the per-file model context (A2/A3) versus what should
be answered on demand by `workspace_query` — and where is the current boundary
duplicating facts that `workspace_query` already serves?

Every claim below is grounded in code, with file/line references. No inference.

## Handoff context (read this first)

### Two codecs

- **A2** (`src/ir/compact_a.rs` → `render_file_context`) is the **live production**
  file-context wire; MCP `content` handlers emit it today.
- **A3** (`src/ir/compact_a3/`) is the **research replacement codec**, deliberately
  unreachable from MCP production. Wire spec: `A3_GRAMMAR.md`; gated plan:
  `PHASED_A3_PLAN.md`.

### The gate this work is trying to pass

A3 must save **≥ 50%** aggregate tokens vs raw source over a qualifying corpus
(production-selected; raw wins ties). Phase 3A measured **19.30% o200k / 18.32%
cl100k** — far short. That is the baseline this work moves
(`A3_PHASE3A_MATRIX.md`).

### Why the lever is the boundary, not more compression

Two prior findings narrowed the remaining levers:

1. **Reversible primitives are dead.** Symbol dictionary, micro-opcodes, and type
   aliases were measured through the anatomy harness
   (`src/tests/compression/pipeline_anatomy.rs`) at **0 or negative** gain, with
   **zero** MCP call sites. There is no further reversible-compression win — the
   cost is in the *facts* shipped, not their spelling.
2. **The same semantic facts ship twice.** The file wire carries calls (`K`), DI
   (`D`), and derived/inferred facts (`cs`/`fc`/`fd`/`se`/`ec`) even though
   `workspace_query` already serves calls and DI from the workspace index (§2).
   Independently, the MCP result object re-attaches the *full* IR + edge list as
   siblings on every content call regardless of fidelity (§6).

### What this document is

A code-grounded boundary map + plan. **No codec/handler change has been made.**
Pending the maintainer decision below; to be handed to a fresh session.

### Decision required

Approve (or revise): (1) the keep/move/drop classification (§3–§4), and (2) the
result-envelope strip (§6). The plan has **two** surfaces to strip — the file
wire *and* the result envelope — or the change is a no-op on model tokens.

### Sibling documents

- `A3_GRAMMAR.md` — authoritative wire + fidelity spec (grammar legend).
- `A3_PHASE3A_MATRIX.md` — corpus, matrix, 19.30%/18.32% baseline.
- `PHASED_A3_PLAN.md` — gated successor plan.
- `README.md` — candidate lineage (A0/A1/A2 rejected; A3 approved).
- `../../docs/architecture/LLM_CONTEXT_COMPRESSION_RESEARCH.md` — research index.

### Repository rules that bind implementation

- **Encoding:** strict UTF-8, no BOM (`.clinerules/encoding.md`).
- **Tests:** only under `src/tests/**`, referenced via `#[path = "..."]`.
- **File size:** ≤ 615 lines per new/modified file.
- **No agent-run long processes:** `cargo build/test/clippy` are handed to the
  operator, never started by the agent.
- **Architectural approval gate:** removing `result.ir`/`result.semantic_edges`
  is a public MCP contract change — proposed here, not yet authorized.
- **Production Integration Gate:** a fact is dropped only after it is proven
  derivable from source or reachable via `workspace_query`.

## Grammar glossary (family codes)

Authoritative spec: `A3_GRAMMAR.md`. Codes used throughout:

| Code | Meaning | This doc's decision |
|---|---|---|
| `C`/`I` | class / interface (id, name, synthetic) | KEEP |
| `F`/`M`/`P` | field / method / parameter (id, name, type/arity) | KEEP |
| `X`/`J` | extends / implements references | KEEP |
| `cm`/`cf`/`mo` | class modifiers / class flags / method modifiers | KEEP (signature-level) |
| `$`/`T` | imports / type aliases | KEEP |
| `B` | exact body frame (Edit only) | KEEP |
| `K` | file-local call stream (caller, callee, arity, spread) | MOVE → `workspace_query` |
| `D` | core-injection occurrence | MOVE → `workspace_query` |
| `cs` | control summary (if/return/throw counts) | DROP (derived) |
| `pf`/`pt` | pattern facts (CTOR/OBSERVABLE/OVERRIDE/GETTER/SETTER) / pattern | DROP (derived classification) |
| `lf` | legacy flags | DROP (empty on corpus) |
| `fc`/`fd` | control flow / data flow (High+Edit) | DROP (derived) |
| `se`/`ec` | side effects / execution contexts (High+Edit) | DROP (inferred) |
| `E` | workspace semantic edges | already outside the file wire (query) |

`A3_GRAMMAR.md` already makes `fc`/`fd`/`se`/`ec` High+Edit-only and edges
`query`-only. This document goes further: drop those derived families entirely,
and move `K`/`D` to the query side.

## 1. What the production file context actually carries today

The live content boundary is `src/mcp/tool_handlers/core/content.rs:17-25`:

```rust
let normalized = crate::ir::normalize_control_full(...);   // the envelope
let payload = crate::ir::compact_a::render_file_context(&normalized);  // A2
```

`normalize_control_full` (`src/ir/control_full.rs`) emits, per file, exactly these
families: `classes`/`interfaces` (each with `fields`, `modifier_occurrences`,
`class_flag_occurrences`, `extends`, `implements`, `injection_occurrences`,
`patterns`, and `methods`), where each method carries `parameters`, `return_type`,
`modifier_occurrences`, `control_summary_occurrences`, `pattern_fact_occurrences`,
`legacy_flag_occurrences`, `patterns`, `control_flow`, `data_flow`, `side_effects`,
`execution_contexts`, `body`/`body_start`/`body_end`; plus top-level `imports`,
`type_aliases`, `calls`, `semantic_edges`, `navigation`.

`render_file_context` (`src/ir/compact_a.rs:246-262`) then **removes only
`semantic_edges`** (`file_envelope["g"].remove("E")`) and rebuilds navigation as
DI + behavior only (`file_navigation`). Everything else — including the `K` call
records and the `injection_occurrences`/`control_flow`/`data_flow`/`side_effects`/
`execution_contexts` method facts — stays in the file payload.

A3 (`src/ir/compact_a3/`) reproduces the same families: `C/I/X/J/cm/cf/D/F/M/p/
mo/cs/pf/lf/pt` (declarations) and `V/fc/fd/se/ec/K/$/T/Y/B` (facts).

## 2. What `workspace_query` already serves

The workspace index is populated from the same IR at compile time.

- **Calls are already projected to `Calls` edges.** `src/ir/semantic_projection.rs:99-117` maps every `CoreOp::Call` to `SemanticEdge { relation: Calls, … call_evidence: CallEvidence::new(explicit_arg_count, has_spread) }`. `project_generic_facts` (`:124-128`) appends these to the index.
- **Declarations are already registered.** `src/ir/semantic_projection.rs:67-86` projects every `DefMethod`/`DefInterfaceMethod` to a `Defines` occurrence edge.
- **Framework DI/framework relations are already edges.** `src/layers/meta/semantic.rs:79+` defines `SemanticRelation::Injects`, `HasInput`, `HasOutput`, `HasSelector`, `HasTemplate`, `DeclaresInModule`, etc., produced by the framework meta-layers.
- **Query surfaces.** `src/mcp/tool_handlers/query/edges.rs` (`forward_edges`, `reverse_edges`), `graph.rs` (`transitive_dependencies`, `has_cycle`), and `entities.rs` expose these edges on demand, with workspace/provenance scoping.

## 3. The boundary map

| Family | In file context (A2/A3)? | In `workspace_query`? | Duplicated? | Decision |
|---|---|---|---|---|
| Class/interface/field/method/param declarations | yes | `Defines` edges (`semantic_projection.rs:67`) | no (declaration = the entity) | **KEEP in file** |
| Class/method modifiers, class flags (`cm`/`cf`/`mo`) | yes | no | no | **KEEP in file** (signature-level) |
| Imports, type aliases | yes | no | no | **KEEP in file** (file-local structure) |
| Exact bodies (Edit) | yes | no | no | **KEEP in file** (byte-exact, `replace_in_file`) |
| **Calls** (`K`) | yes | `Calls` edges (`semantic_projection.rs:99`) | **YES** | **MOVE to workspace_query** |
| **Injection occurrences** (`D`) | yes | `Injects` edges (framework meta-layers) | **YES** (framework DI) | **MOVE to workspace_query** |
| `control_summary_occurrences` (`cs`) | yes | no | no | **DROP** (derived from source: `if`/`return`/`throw`) |
| `pattern_fact_occurrences` (`pf`) | yes | no (only framework-relation neighbors) | no | **DROP** (derived classification) |
| Patterns (`pt`) | yes | no | no | **DROP** (derived classification) |
| `control_flow` / `data_flow` (`fc`/`fd`) | yes | no | no | **DROP** (derived) |
| `side_effects` (`se`) | yes | no | no | **DROP** (inferred — not in source) |
| `execution_contexts` (`ec`) | yes | no | no | **DROP** (inferred — not in source) |
| `legacy_flag_occurrences` (`lf`) | yes | no | no | **DROP** (empty on the measured corpus) |
| `semantic_edges` (`E`) | no (A2 removed it) | yes | no | ✅ already moved |

## 4. Proposed boundary

**File context (per-file):** declarations + modifiers that are part of the
signature + imports + type aliases + exact bodies (Edit). Nothing else.

**`workspace_query` (on demand):** calls, DI/framework relations, cross-file
edges, graph traversal.

**Dropped:** every *derived/inferred* fact the compiler re-inlined into the file —
control summaries, pattern facts, control/data flow, side effects, execution
contexts, legacy flags. These are not queryable and not required for editing;
they are exactly the "extra information the model doesn't get in normal source"
and are reconstructed from the source or not needed.

## 5. Evidence for the duplication (the core finding)

- Calls exist twice: `compact_a.rs:138-152` serializes `K` rows into the file
  payload, while `semantic_projection.rs:99-117` serializes the identical facts
  (`caller`, `callee`, `explicit_arg_count`, `has_spread`) as `Calls` edges into
  the workspace index. The model gets both; the second is on demand and scoped.
- DI exists twice (framework DI): `compact_a.rs:168-185` serializes
  `injection_occurrences` into `n.D`, while the framework meta-layers emit
  `Injects` edges (`semantic.rs:82`).
- The `se:`/`ec:` facts are pure inference: `SideEffectKind` (`pure`/`io`/
  `mutation`/…, `src/ir/opcodes/semantic.rs:105`) and `ExecutionContextKind`
  (`sync`/`async`/…, `:149`) are **not present in source**; the compiler invents
  them and A2/A3 ships them. The only in-code rationale is a renderer comment
  (`src/ir/render_llm.rs:383-397`): a "shortcut … without a full body read".

## 6. JSON-RPC result envelope — the duplication Claude is describing

The file-content wire is not the whole story. Every context-producing handler
also assembles the MCP `result` object and attaches the **full semantic edges**
and the **full hierarchical IR** as siblings, regardless of fidelity:

| Tool | `result.ir` (full hierarchy) | `result.semantic_edges` (full) | Evidence |
|---|---|---|---|
| `compress_code_context` | direct sibling | direct sibling | `compress.rs:253,257` |
| `provide_code_context` (full & delta) | — | under `_meta` | `provide.rs:345,412` |
| `restore_context` / `replay_history` | direct sibling | direct sibling | `control_full_content.rs:337-341` |
| `control_full_delta` (cached) | direct sibling | direct sibling | `delta.rs:175-176` |
| `control_full_delta` (delta) | — (ships `delta`) | direct sibling | `delta.rs:298` |
| `delta_apply` | under `structuredContent` | under `structuredContent` | `delta_apply.rs:291-292` |

Key facts:

- **Independent of fidelity.** `hierarchy_to_wire(&ir, &hir)` and
  `serde_json::to_value(&semantic_edges)` are emitted unconditionally; fidelity
  only changes the `content` text and the `byte_exact`/`content_kind` metadata.
- **The `ir` sibling re-ships the facts the boundary map proposed to drop.**
  `persistence_lifecycle.rs:394-402` asserts `result.ir.ir.c[0].m[*]` carries
  `se` (`["async"]`) and `ec` (`["async"]`) — so the full hierarchical IR
  includes `se`/`ec`/`cs`/`fc`/`fd`/`pf`/`lf`, exactly the families §3 marks
  DROP.
- **The `semantic_edges` sibling is the authoritative edge snapshot.** The A2
  payload itself has `semantic_edges: []` (removed in `compact_a.rs:251-254`),
  but the result re-attaches the full list (`control_full_content.rs:126-127`).

Consequence: stripping `K`/`D`/`cs`/`fc`/`fd`/`se`/`ec` from the A2/A3 *file
content* is necessary but **not sufficient**. The same facts re-enter the model
context through `result.ir` and `result.semantic_edges`. The boundary work must
therefore also decide what the content-producing handlers put on the result
envelope (drop the `ir`/`semantic_edges` siblings, or reduce them to the same
stripped projection), otherwise the wire-level strip has no token effect.

## 7. Plan of action (when approved)

1. Define the reduced file-context target as a new schema version (A3 schema 4)
   that carries only: file identity, typed declarations (with signature-level
   modifiers), imports, type aliases, and Edit bodies. Drop the `K`, `D`, `cs`,
   `pf`, `pt`, `lf`, `fc`, `fd`, `se`, `ec` families from the wire.
2. **Stop shipping the full facts on the result envelope.** For every
   content-producing handler (`compress.rs`, `provide.rs`, `delta.rs`,
   `delta_apply.rs`, `restore_context`, `replay_history`), either drop the
   `result.ir` / `result.semantic_edges` siblings or replace them with the same
   stripped projection as the `content` text. Without this step, §1-1 is a
   no-op on actual model tokens (§6).
3. Update `normalize_control_full` (or introduce a narrower projection) so the
   file-context encoder no longer receives the dropped families.
4. Re-wire the roundtrip target and the tracked tests (`src/tests/ir/**` and the
   MCP envelope assertions in `src/tests/mcp/control_full_content.rs`,
   `persistence_lifecycle.rs`) to the reduced envelope; confirm
   `cargo test --all-features` stays green.
5. Re-measure the full Phase-3A matrix with the reduced envelope and record the
   new production-selected aggregate (expected materially better than 20%, since
   the dominant `cs`/`fc`/`fd`/`se`/`ec` rows are removed from **both** the wire
   and the result envelope).
6. Confirm `workspace_query` (`forward_edges`/`reverse_edges`) still answers call
   and DI questions from the index, so nothing becomes unreachable.
