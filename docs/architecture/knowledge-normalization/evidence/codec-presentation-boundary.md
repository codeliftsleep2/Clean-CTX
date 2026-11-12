# Historical finding DIS-2026-020

Agent: **Agent MaxHeadRoom**, B6, 2026-11-12.

The exact original ledger section follows, including its observed failure,
negative controls, repair/test attribution and historical stated invariant.
It is historical evidence, not a second current normative owner. Current
ARCH-003 remains controlling. No regression or field scenario was rerun by B6.
Original source: docs/agent/DISCOVERY_REGISTRY.md at 02d8336f983d07734bf0ba43dbb4b3a50d12ef6d.
Exact section SHA-256: acf40d9778cba62c047c336c07fa236f91ef2d221d13480cd181a0eb47ae7e24.

## DIS-2026-020: Model-Visible Content Is Assembled from the Codec, Not a Presentation

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-24 |
| **Environment** | Controlled laboratory (this repository): source audit + measurement harness |
| **Repository/context** | Clean-CTX itself — `src/mcp/tool_handlers/core/content.rs` and its 17 caller sites |
| **Symptom** | Every content-producing handler (`compress_code_context`, `provide_code_context`, `control_full_delta`, `apply_delta`, `restore_context`, `replay_history`, persistence) assembles the model-visible `content` from the CONTROL-FULL codec (`compact_a::render_file_context`, A2), shipping the codec's preamble, grammar legend, envelope schema id and `§BODIES` framing into the model's context. `render_hierarchical_for_llm` (SCHEMA v5), the LLM-facing presentation renderer, has zero production callers. |
| **Root cause** | The feature branch replaced the presentation renderer with the codec, conflating the reversible wire (CTX-001) with the model-visible presentation (ARCH-003). The codec's decode side has no production caller (`decode_cold` / `decode_declarations` / `decode_facts` reference only one another), so its legend is required by nothing in the protocol while being paid in every prompt. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes — dispatch any content handler and read `result.content[0].text`. |
| **Local regression** | `src/tests/mcp/presentation_boundary.rs` (content is not the codec document; carries no decoder legend / envelope schema id / body framing; typed owner + method identity survive). |
| **Live scenario required?** | No — the defect is the production assembly path, not scale-dependent. |
| **Architectural invariant** | ARCH-003 (presentation) vs CTX-001 (reversible codec), now mechanically enforced. |
| **Status** | Fixed — `content` is the SCHEMA-v5 presentation on all paths; the reversible codec stays code-side (`result.ir` + persistence) |

**Decision (2026-09-24):** `content` becomes the presentation renderer
(`render_hierarchical_for_llm`, SCHEMA v5) on every path; the codec stays
code-side (`result.ir` + persistence). Option A now, Option C (a purpose-built
presentation) follows; Option B (codec minus its legend) was rejected because a
positional grammar without its interpretive key is undecodable, not presentable.
See `verification/context-compression/compact-a/PRESENTATION_BOUNDARY_PLAN.md` §6.



```architecture-graph
node: DOC-619
kind: RESEARCH
status: INVESTIGATING
title: Historical DIS-2026-020 codec presentation boundary
track: REPOSITORY
artifact: docs/architecture/knowledge-normalization/evidence/codec-presentation-boundary.md
preserves:
  - ARCH-003
note: Source-attributed exact historical subject and limits retained; no current executable claim, independent certification, reclosure or banking.
```
