# CONTROL-FULL reasoning failure triage

**Gate status:** failed (24/36); codec work remains stopped.

The twelve dossiers are defined in `failure-classification-v1.json`. Each record
joins by `case_id` to `reasoning-results-codex-corrected.json`, which supplies
fixture, effective mode, intent, focus, operation, capture path, exact question,
oracle, answer, and scorer decision. Inspection used only model-visible captures.

## Findings

- No failed case currently demonstrates Category C. Required caller IDs,
  ordinals, arity/spread, endpoint provenance, occurrence groups, file/entity
  identity, delta sections, and exact-body IDs are present in `content`.
- Category A: call filtering (2), cross-file identity/scorer wording (2), and
  under-bounded restore/replay questions (2).
- Category B: dense-row field coverage (1), DI edge locality (2), nested
  occurrence notation (2), and delta state framing (1).
- The scorer was overly literal for `provenance--cross-b`: correct abstention
  does not require the exact phrase “request the other payload.”
- Restore/replay asks for “semantic facts” without bounding which fields must be
  listed. An exhaustive hidden expectation is invalid.

## General prompt repair

The evaluator preamble should state, for every family: filter records by the
requested canonical ID before collecting rows; treat occurrence numbers and
nested arrays as semantic; report every explicitly requested field; distinguish
written unresolved callees from declarations; distinguish file IDs, entity IDs,
and absent files; and abstain from lifecycle claims not stated in the payload.
These instructions disclose representation rules, not case answers.

The scorer should judge semantic equivalence, require only explicitly enumerated
facts, accept equivalent abstention wording, and reject fabricated identity,
ownership, resolution, provenance, occurrence/order, or lifecycle claims.

## Representation hypotheses

For Category B only, test the same facts with: call fields in one row; semantic
edge provenance adjacent to endpoints; explicit occurrence-group labels; and
named `prior`, `transition`, and `post_apply` delta sections. Add or remove no
semantic information. These are diagnostic organization experiments, not codec
selection or production changes.

There is no Category C repair proposal. A genuine multi-file comparison remains
outside the current single-file production payload; the present cases correctly
test identity plus abstention instead.

The completed same-information experiment is interpreted in
`presentation-repair-proposal-v1.md`. It distinguishes a model-facing
presentation defect from canonical semantic loss and defines the approval gate
for any externally visible CONTROL-FULL schema change.
