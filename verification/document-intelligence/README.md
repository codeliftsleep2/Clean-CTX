# Semantic Document Intelligence Benchmark

**Status:** Phase 0 benchmark definition in progress
**Authority:** Evaluation material only; this package is not a tracked test
suite and does not define a production contract

This package evaluates whether Clean-CTX can reduce the cost and improve the
safety of Markdown-heavy engineering work. Clean-CTX's documents are the
controlled laboratory, not the product boundary. The benchmark must eventually
also run against unrelated repositories during live validation.

The governing investigation is
[`docs/architecture/SEMANTIC_DOCUMENT_INTELLIGENCE_INVESTIGATION.md`](../../docs/architecture/SEMANTIC_DOCUMENT_INTELLIGENCE_INVESTIGATION.md).

## Contents

- `benchmark-manifest.json` - versioned corpus, task, evidence, and measurement
  definitions;
- `full-document-control-oracles.json` - human-reviewed expected answers for
  the full-document control lane;
- `token-baseline.schema.json` - generated token-capture output contract;
- `scripts/Capture-TokenBaselines.ps1` - exact `cl100k`/`o200k` capture using
  the repository's existing measurement helper;
- `fixtures/structural-edge-cases.md` - concentrated parser and hierarchy cases;
- `fixtures/linked-overview.md` and `fixtures/linked-decision.md` - explicit,
  missing, external, and advisory cross-document relationships; and
- `fixtures/preservation-*.md` - safe and unsafe edits over the same protected
  concept.

Real repository documents are referenced in place. They are not copied into
this package. Their hashes in the manifest make source drift visible rather
than silently changing the benchmark.

## What Phase 0 measures

The initial tasks cover:

1. exact section retrieval from large documents;
2. evidence assembly across documents;
3. governing-instruction navigation without treating retrieval as policy
   injection;
4. write-context selection;
5. ambiguous versus structurally qualified edit targets;
6. deterministic Markdown structure; and
7. safe versus authority-reversing semantic edits.

Each task names the evidence sections a correct system must retain. Those
labels are human-authored evaluation authority; heading names and future model
scores are not ground truth by themselves.

## Benchmark states

- `defined` - task, sources, and expected evidence are reviewable;
- `baseline_pending` - raw/full-document token and task-result captures still
  need to be recorded;
- `baselined` - the full-document control has been captured;
- `candidate_ready` - a parser or retrieval candidate can be compared;
- `evaluated` - candidate results and failure analysis have been recorded.

The exact token baseline is captured and tracked. Model-produced full-document
control answers remain pending, so the overall manifest remains
`baseline_pending`. No approximate token counts are substituted for supported
tokenizer measurements.

Human-reviewed control oracles are defined separately from model-produced
control captures. An oracle states what a correct answer must preserve; it is
not evidence that a model run succeeded.

## Token baseline capture

The capture script requires the existing measurement helper. Building that
helper is an operator-owned Cargo command under repository policy:

```powershell
pwsh -File verification/context-compression/scripts/Build-MeasureHelper.ps1
pwsh -File verification/document-intelligence/scripts/Capture-TokenBaselines.ps1
```

The first command builds `target/context-compression-verification/scripts/measure.exe`.
The second verifies pinned document hashes, counts every corpus document with
both exact tokenizers, and writes:

```text
target/document-intelligence/baselines/full-document-tokens.json
```

Generated output remains under `target/` until reviewed. It must not be
reported as a tracked-test result.

## Evaluation rules

- Split semantic training and evaluation by document, never by random section.
- Preserve exact path and heading provenance for every selected section.
- A partial response must disclose that it omitted document content.
- Governing instructions remain loaded by the host's authoritative mechanism;
  this benchmark evaluates navigation and explanation only.
- Explicit links are deterministic evidence. Textual similarity is advisory.
- An inferred role, status, or authority never establishes repository fact.
- A live harness result is observational evidence, not a tracked test result.
- Implementation regressions required by later phases belong under
  `src/tests/**`.

## Phase 0 completion work

Before Phase 0 is complete:

- capture model-produced full-document control answers and compare them with
  `full-document-control-oracles.json`;
- add unrelated external-repository scenarios for the field gate; and
- document any task whose answer is legitimately ambiguous.

No parser, embedding model, classifier, vector index, public tool, or global
retrieval limit is selected by this package.
