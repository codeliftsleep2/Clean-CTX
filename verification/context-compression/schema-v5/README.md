# SCHEMA-v5 presentation reasoning harness — scoping

**Status:** implemented. This document records the boundary, structure, and
oracle catalog for a *separate* evaluation harness that scores
the model against the **model-visible interface** (SCHEMA-v5 presentation +
`workspace_query`) — distinct from the existing codec harness.

## 1. Why a separate harness

The existing `verification/context-compression/` harness is **codec regression
infrastructure**. It answers one question: *is the reversible file-local wire
(A2/A3) still correct and dense?* Its oracles are **preservation/round-trip**
questions — canonical IDs, occurrence order, duplicates, spread, unresolved
callee spellings, injection occurrences, byte spans. Those facts are real, but
they moved **off** the model-visible channel when `content` became the
SCHEMA-v5 presentation (ARCH-003 over CTX-001).

The model never sees the codec. It sees the SCHEMA-v5 presentation in
`content`, plus whatever it fetches via `workspace_query`. A harness that feeds
the codec to the model and scores codec preservation is therefore measuring a
channel that no longer exists for the reader.

This harness answers a **different** question: *can the LLM correctly reason
about, and edit from, what it actually receives?* Its oracles are
**understanding + edit-readiness** questions, not preservation questions.

| | codec harness | schema-v5 harness |
|---|---|---|
| Object under test | A2/A3 reversible wire | SCHEMA-v5 presentation + `workspace_query` |
| Oracle kind | preservation / round-trip | understanding / edit-readiness |
| Identity | canonical IDs (`C6`, `M11`, …) | name + typed owner |
| Facts exercised | calls, injections, occurrence order, byte spans | names, arity, `X`/`I`, signatures, flags, exact bodies, cross-file edges |
| Kept when | we change A3 | (always — this is the production interface) |

## 2. Directory layout

```
verification/context-compression/
  fixtures/                  # SHARED — unchanged
  expected/                  # SHARED — scenarios.json + marginal-fixtures.json
  scripts/                   # SHARED — McpSession.ps1, measure.rs, capture + fixture builders
  score.schema.json          # SHARED — scoring schema (harness root)
  codec/                     # CODEC harness — A2/A3 wire + CONTROL-FULL oracle
    oracles.json
    workspace-query-oracles.json
    transport-assertions.json
    scripts/                 #   codec measure/capture/verify/reasoning runners
  schema-v5/                 # SCHEMA-v5 harness (this directory)
    README.md
    oracles.json             #   the catalog in §3
    workspace-query-oracles.json
    scripts/                 #   validate/worksheet/run/verify/list/show/record/summarize
```

Shared `McpSession.ps1` and `measure.rs` remain in `scripts/` because the
`edge-cases/` and `verification/workspace-query/` harnesses and
`measure-helper/Cargo.toml` reference those exact paths.

The SCHEMA-vNext Phase 0 production economics and independent token anatomy are
recorded in [`PHASE0_BASELINE.md`](PHASE0_BASELINE.md).
The first isolated Tier A experiment is measured with
`scripts/Measure-A1SinglePath.ps1`; it writes generated evidence beneath
`target/context-compression-verification/captures/` without changing the
production renderer. Its result and gate status are recorded in
[`PHASE1_A1_SINGLE_PATH.md`](PHASE1_A1_SINGLE_PATH.md).

Prepare and run the paired A1 path-reasoning gate with:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/schema-v5/scripts/Prepare-A1ReasoningWorksheet.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/schema-v5/scripts/Run-CodexReasoning.ps1 `
  -TemplatePath ./target/context-compression-verification/captures/schema-vnext-a1-reasoning-template.json `
  -ResultsPath ./target/context-compression-verification/captures/schema-vnext-a1-reasoning-results-codex.json `
  -ExpectedCaseCount 6 -Restart
```

## 3. Oracle catalog

Three lanes. **File** and **lifecycle** feed the SCHEMA-v5 presentation
(`content`); **workspace** feeds `workspace_query` responses. 15 oracles total.

### 3.1 File lane (SCHEMA-v5 presentation)

| id | question | expected | zero-tolerance |
|----|----------|----------|----------------|
| `typed-ownership` | Which type owns each `run` method? | Alpha owns its `run` overload family; Beta and `InheritanceProbe` each own distinct `run` methods; `Runner` declares `run` on an interface. Identity is typed owner + name, not an ID. | wrong owner; ID-invented identity |
| `overload-arity` | How are same-name methods on Alpha distinguished? | By parameter count — `run(+1)` and `run(+2)` are distinct arity groups. | deduplication; merging distinct arities |
| `inheritance` | What does each type extend / implement / interface-extend? | Report the `X` (extends) and `I` (implements) relations per owner; keep class and interface families distinct. | class/interface collapse; invented relation |
| `signatures` | What are the parameters and declared return of method M? | Read `p:` and the `→` return from the method line; name + params + return. | wrong param; wrong return |
| `behavior-facts` | Which methods are async / mutating / IO / transactional? | Read the annotation groups `mod:`/`ctl:`/`pf:`/`fl:`/`cf:`/`df:`/`se:`/`ec:`; report only what the question asks. Do not infer absent facts. | wrong owner; invented fact |
| `focused-edit` | Which bodies are byte-exact and safe to edit? | Only the focused method bodies; `_meta.byte_exact` reports `focused_method_bodies`. Unfocused bodies are signature-only and must not be edited. | wrong edit target; unlisted body |
| `source-escalation` | What does Alpha.run return byte-for-byte? | Not available at structural fidelity — request Edit (focused) or Verbatim; never reconstruct a body. | hallucinated implementation |

### 3.2 Lifecycle lane (delta / restore / replay `content`)

| id | question | expected | zero-tolerance |
|----|----------|----------|----------------|
| `delta-summary` | What does delta `content` tell you, and where is the change? | A minimal summary `Δ delta for … (v{from} → v{to}): +N ~N -N ops`; the structured op list is code-side in `result.delta`, not in `content`. | apply without acknowledged baseline; reading ops from `content` |
| `durable-regeneration` | Is the presented text the durable semantic authority? | No — it is regenerated from the durable canonical IR for the selected version. | stale text as authority |

### 3.3 Workspace lane (`workspace_query`)

Because SCHEMA-v5 omits cross-file edges/calls/injections from `content`, the
model must call `workspace_query` for those facts — so this lane is *more*
load-bearing than in the codec harness.

| id | op | question | zero-tolerance |
|----|----|----------|----------------|
| `provenance` | `find_entities` | Are two same-name declarations the same entity? | cross-file identity collapse |
| `framework-edge-provenance` | `forward_edges` (angular) | Report the injection edge: subject type, object type, relation, layer, asserting file. | DI error; provenance error |
| `cross-file-calls` | `reverse_edges` (builtin) | Who calls `audit`, and from which files? (coarse-by-name; §2.3 owner-precision loss is accepted) | invented caller identity |
| `entities-in-file` | `entities_in_file` | Which types/methods live in this file? | missing entity |
| `transitive-dependencies` | `transitive_dependencies` | What is the dependency closure from this node? | dropped hop |
| `has-cycle` | `has_cycle` | Is there a cycle in this subgraph? | wrong cycle verdict |

## 4. Runner & prompt design

The runner is a thin variant of `Run-CodexReasoning.ps1`. Each answer prompt
prepends `REASONING_INSTRUCTIONS.md` — a mandatory-rules guide (modeled on
`docs/CLAUDE_INTEGRATION_RULES.md`) that teaches the SCHEMA-v5 notation,
identity, overload, escalation, workspace-fact, and delta rules before the
payload and question. The codec prompt's "preserve canonical IDs / occurrence
indices / unresolved callees / entity IDs" language is **removed**.

## 5. Capture reuse

`Capture-Baselines.ps1` writes the actual model-visible content
(`content[0].text`) to `content.txt` for every scenario — the SCHEMA-v5
presentation, the byte-exact focused Edit bodies, and the delta summary alike.
The schema-v5 file and lifecycle lanes consume `content.txt` (never the codec
CONTROL-FULL oracle, nor the `control-prod-*` reconstruction, which falls back
to raw source for Edit). The workspace lane reuses `Capture-WorkspaceQuery.ps1`
against the schema-v5 `workspace-query-oracles.json` (6 ops: `find_entities`,
`forward_edges`, `reverse_edges`, `entities_in_file`, `transitive_dependencies`,
`has_cycle`).

## 6. Scoring contract

Same `score.schema.json`, but the evaluator prompt is rewritten to score
**semantic correctness for understanding/edit-readiness**, not preservation:
"pass iff the answer correctly identifies the owner/signature/relation/edit
target and violates no zero-tolerance condition — do not require IDs,
occurrence order, or call evidence, which the payload does not contain."

## 7. Next steps

1. Run the shared captures and the schema-v5 workspace captures, then smoke-run
   the worksheet and reasoning runner against the existing captures.
2. Record results; iterate on any oracle whose wording is ambiguous or
   unanswerable from the SCHEMA-v5 presentation.
3. Once the SCHEMA-v5 reasoning score is known, decide whether Option C (a
   purpose-built presentation) is warranted or SCHEMA v5 suffices.


