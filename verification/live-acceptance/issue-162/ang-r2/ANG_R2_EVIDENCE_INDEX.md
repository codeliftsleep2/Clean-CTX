# ANG-R2 mission document and asset coverage

Agent: **Agent MaxHeadRoom**. The governing source-owned knowledge record is
[DOC-270](../../../../docs/agent/ISSUE_162_ROUTE_RESOLVER_ACCEPTANCE.md).
Its formatting-disposition and mission-document-coverage components cover the
following bounded campaign records. Coverage identifies scope and provenance;
it does not promote historical proposals, UNRUN entries or partial receipts into
current verification. Governing contracts retain their existing authority.

## Mission documents

| Document in this directory | Covered role and qualification |
| --- | --- |
| [ANG_R2_ADMISSION_REVIEW.md](ANG_R2_ADMISSION_REVIEW.md) | Dated ANG-R2 design, owner attribution, implementation or governance evidence; preserve its explicit historical/current standing |
| [ANG_R2_COMPATIBILITY.md](ANG_R2_COMPATIBILITY.md) | Compatibility design and dated implementation/verification standing |
| [ANG_R2_DESCRIPTOR_REVIEW.md](ANG_R2_DESCRIPTOR_REVIEW.md) | Dated ANG-R2 design, owner attribution, implementation or governance evidence; preserve its explicit historical/current standing |
| [ANG_R2_EVIDENCE_INDEX.md](ANG_R2_EVIDENCE_INDEX.md) | This document/asset coverage inventory, not semantic certification |
| [ANG_R2_FORMAT_HANDOFF.md](ANG_R2_FORMAT_HANDOFF.md) | Exact post-normalization operator command and selection scope; UNRUN |
| [ANG_R2_FROZEN_FORMAT_REVIEW.md](ANG_R2_FROZEN_FORMAT_REVIEW.md) | Historical concrete proposal and later explicitly attributed adoption |
| [ANG_R2_GOVERNANCE_HANDOFF.md](ANG_R2_GOVERNANCE_HANDOFF.md) | Dated ANG-R2 design, owner attribution, implementation or governance evidence; preserve its explicit historical/current standing |
| [ANG_R2_IMPLEMENTATION.md](ANG_R2_IMPLEMENTATION.md) | Dated ANG-R2 design, owner attribution, implementation or governance evidence; preserve its explicit historical/current standing |
| [ANG_R2_LIFECYCLE_VERIFICATION.md](ANG_R2_LIFECYCLE_VERIFICATION.md) | Lifecycle obligation/witness reconciliation with dated results |
| [ANG_R2_OWNER_DISPOSITION_01.md](ANG_R2_OWNER_DISPOSITION_01.md) | Dated ANG-R2 design, owner attribution, implementation or governance evidence; preserve its explicit historical/current standing |
| [ANG_R2_OWNER_DISPOSITION_03.md](ANG_R2_OWNER_DISPOSITION_03.md) | Owner-attributed formatting/preservation decision; no execution or protected grant |
| [ANG_R2_PERFORMANCE_DISPOSITION.md](ANG_R2_PERFORMANCE_DISPOSITION.md) | Bounded-work scope, controlled measurements and unwaived CI obligations |
| [ANG_R2_PRODUCTION_ACCEPTANCE.md](ANG_R2_PRODUCTION_ACCEPTANCE.md) | Pinned actual binary/stdio evidence and its remaining limits |
| [ANG_R2_PROGRESS.md](ANG_R2_PROGRESS.md) | Chronological campaign status; latest statements qualify prior checkpoints |
| [ANG_R2_PUBLIC_CONTRACT.md](ANG_R2_PUBLIC_CONTRACT.md) | Dated ANG-R2 design, owner attribution, implementation or governance evidence; preserve its explicit historical/current standing |
| [ANG_R2_REPRESENTATION_DISPOSITION.md](ANG_R2_REPRESENTATION_DISPOSITION.md) | Dated ANG-R2 design, owner attribution, implementation or governance evidence; preserve its explicit historical/current standing |
| [ANG_R2_RETIREMENT_REPAIR.md](ANG_R2_RETIREMENT_REPAIR.md) | Chronological shared-retirement investigation, decision, correction and observed evidence |
| [ANG_R2_VERIFICATION.md](ANG_R2_VERIFICATION.md) | Chronological exact-source execution ledger; unrun and superseded stages stay distinct |

## Supporting definitions and identity assets

These are operator definitions, scope/identity pins or concrete review assets
associated with the covered records. They are not separate authorities or gates.
Historical command templates retain their original expected source hashes; use
the current handoff below for the current normalized source.

- `ANG_R2_CANDIDATE_PINS.json`
- `ANG_R2_DOCUMENT_COVERAGE.json`
- `ANG_R2_FORMAT_GOVERNANCE_PINS.json`
- `ANG_R2_FORMAT_NORMALIZATION.json`
- `ANG_R2_FROZEN_FORMAT_REVIEW.json`
- `ANG_R2_RETIREMENT_GOVERNANCE_PINS.json`
- `invoke-deleted-owner-regression.ps1`
- `invoke-format-verification.ps1`
- `invoke-governance-admission.ps1`
- `invoke-governance-review.ps1`
- `invoke-registration-build.ps1`
- `invoke-registration-green.ps1`
- `invoke-registration-prepublication.ps1`
- `invoke-registration-red.ps1`
- `invoke-registration-transport.ps1`
- `invoke-retirement-authority.ps1`
- `invoke-retirement-build-stdio.ps1`
- `invoke-retirement-governance-review.ps1`
- `invoke-retirement-green.ps1`
- `registration-stdio.mjs`

- `historical-witness-evidence.zip`: versioned byte-preserving Git archive of exact original/proposed witnesses, raw RED/GREEN outputs and successful pre-format replay evidence; SHA-256 `ea328ed56679bf05800658c6577d2a919f3de93fdcdd6a1ca9ba61eddd0b8b53`. Archival source inside is never executable regression coverage.

## ANG_R2_FORMAT_HANDOFF

Current handoff: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-format-verification.ps1`. It is UNRUN. See
[the exact coverage and commands](ANG_R2_FORMAT_HANDOFF.md) for its focused scope.
Raw source archives, generated workspaces and runtime receipts remain under the
existing `target/ang-r2` evidence mechanism, with exact hashes and source identities.
Tests remain under `src/tests` and live definitions under `verification`; neither
archives nor live captures substitute for tracked regression coverage or CI.

The index's own identity is pinned by the governing candidate. Other mission
documents are pinned in ANG_R2_DOCUMENT_COVERAGE.json, which the handoff checks.
No ordinary file inventory grants architectural acceptance or execution permission.

The packet has now passed its 19 tests, scoped format/Clippy and first guards/
admission. A documentation-only finalization uses the checked ResumeVerifiedFrom
mode described in ANG_R2_FORMAT_HANDOFF.md. Durable exact bytes also reside in
the versioned Git archive above; ignored target storage is no longer the sole
retention location. Commit/push availability and final revised admission are pending.
