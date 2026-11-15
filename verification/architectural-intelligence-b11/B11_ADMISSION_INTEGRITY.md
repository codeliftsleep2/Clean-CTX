# B11 admission architecture investigation

Agent: **Agent MaxHeadRoom**. Initial inventory, not final findings.

The acquired AUTH.PCG_DOCUMENTS contract owns document-catalog admission;
AUTH.PCG_GLS supplies its exact candidate-state taxonomy and consistency rules.
AUTH.PCG_PREAUTHORING governs preventive retrieval/acquisition. Enrolled evidence
v1 admission, projection and MCP contracts remain separate and require separate
applicability analysis. Authority references and source pins are preserved in
`target/pcg-b11/authority-20261113-152531-723`.

Current paths inspected:

- `documents::Session::query` validates scope/request, starts an immediate SQLite
  transaction, initializes the catalog, scans candidates, runs validate and missing
  identity checks, then publishes only if diagnostics are empty.
- `Session::knowledge` separately invokes the same scanner/validator/missing
  checks before `publish_knowledge`, with reference-first responses and explicit
  acquisition. These parallel orchestrations require equivalent-path verification.
- `scan` reads observed sources, parses declarations, resolves artifacts and exact
  knowledge component selectors, detects duplicate IDs and records witnesses.
- `validate` checks the seven kinds/twelve statuses/thirteen relation types and
  the acquired GLS candidate-state constraints. It is not a revision permission
  validator. `store::missing` refuses disappearance of admitted identities.
- `Capture::completion` re-observes source hashes/permissions and profile state.
  Both transaction completion paths and schema migration/recovery still need
  complete tracing; atomicity is not certified from the transaction's existence.

Known diagnostic limitation: code/path/node/message cannot identify precise
offending lines, fields or explicit publication disposition. Initial tracked RED
tests target this omission. Governing reference resolution must eventually use
accepted authority without publishing candidate policy first; its detailed design
is not yet established. No production changes have been made.

Held questions under investigation: declaration-free new-document scope;
authorized source relocation versus identity appropriation; mechanical evidence
claims versus source-attributed status; and governing-policy changes where revision
authorization is explicitly not established. None is resolved by inventing a new
approval framework, universal lifecycle matrix or prose contradiction heuristic.

## Current B11 implementation checkpoint

The initial findings above retain their original investigation standing. Subsequent
owner dispositions 01-05 establish explicit corpus membership, source-owned
controlling selections and procedural operator authorization, with accepted-state
comparison in schema v3 and atomic graph/governance publication. This is not
authenticated human identity or protection against adversarial shared OS access.

Current focused evidence: 18 B11 tests, 57 document tests, 10 MCP tests and scoped
Clippy passed in `admission-green-20261113-182144-241`. Production receipt
`production-20261113-183428-928` preserved 218 identities/221 relationships/history
and verified 40 exact source spans. Final revised selection is still a proposal:
342 controlling components across 137 identities, with the same 153 governed
owners. Actual protected bootstrap remains inactive; later source corrections
are not yet production-reconciled.

Exact field locations, rejected-publication metadata and accepted governing
references are implemented and covered by preserved original RED/GREEN chains.
Source-owned boundary splitting exposes a further specific gap: statement-only
governing references can omit independently represented constraints. The new
tracked `document_admission_rule_components.rs` witness must reproduce that
omission before correction. No observation, candidate assertion or historical
receipt is allowed to replace the exact previously accepted governing rule.

The proxy authority contradiction is resolved by owner disposition 05 without
production proxy changes. Current verification responsibility is CI-owned for the
consolidated full gate; focused operator tests/lints/builds/guards remain separate.
Remaining obligations are listed in the progress and verification records; no
independent certification, final activation or completed CI gate is claimed.

## 2026-11-14 final proposal production receipt

Operator receipt `production-20261114-201955-538` passed 47 isolated CLI/MCP
assertions and actual-root source verification. It reused the successful build
from the preserved failed harness receipt after checking executable and Rust input
identities. Executable SHA-256:
`4e3da8ee0b160938a238fc273271ce24d1574348c60d16b5b047ae936260387f`.
Profile SHA-256: `0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9`.
Accepted generation:
`9dbaacb94dc0be012a0c23e3d5e0d3d6fc5453c3b56f5cc569df0035dda75703`.

All 218 identities, 221 typed relationships, immutable revision/source rows and
publication history were preserved. The catalog contains 482 components; 342
proposed protected components and 280 distinct exact UTF-8 source spans passed.
The 58 revisions advanced since the captured predecessor are recorded individually
in `B11_PRODUCTION_REVISION_CHANGES.csv`; these are not identity removals.
Actual repository grants and governance publications remained empty.

The verified four-column 342-component selection is now staged as the working
`docs/architecture/knowledge-governance/protected.csv` candidate. This does not
activate governance: subsequent admission must refuse bootstrap until the operator
reviews and explicitly authorizes its exact Rust-computed descriptor. The new
`invoke-bootstrap-review.ps1` captures that descriptor without authorization or
publication. Its execution is handed to the operator, not run by the agent.

The rule matrix now records current grouped dispositions rather than provisional
mechanism holds. It remains a bounded mechanical inventory: prose truth, policy
applicability and human authentication are not automatically inferred. The final
bootstrap, focused repository guards and delivery commit/push remain outstanding.
No full repository gate or independent evaluation was run; CBM remains explicitly
failed/degraded. Historical receipts and reports remain preserved.
## Final bootstrap publication and bounded closeout status

Agent MaxHeadRoom inspected operator authorization receipt
`bootstrap-authorization-20261114-221135-999.json`: exact reviewed digest granted,
accepted_publication=false. Operator publication receipt
`bootstrap-publication-20261114-221250-812` then passed, activating governance at
`4f410b2f2f816ce7dc7a4ca74ad04009d19ee0bb2631ed8a36a9ae7103ff57c3`.
The publication binds exactly to the reviewed state, predecessor and grant.
All 218 identity/revision memberships, 221 relationships, 482 components, prior
revision rows, source bytes and publication history remain intact. Exactly one
new governance publication was added. The protected set has 342 selections across
137 identities; the governed corpus has 153 source owners / 218 identities.

The matrix contains 14 grouped obligations, not 14 individually enumerated GLS
clauses. Ten groups preserve existing mechanical/procedural obligations; four
rows identify three distinct corrected gap families (GOV.AUTH appears in two
related obligations): diagnostic location/accepted authority disclosure, explicit
governed membership, and exact procedural protected-change authorization.
Additional constraint-reference and authorized-removal/history regressions were
corrected inside those families; they are not padded into separate policy counts.
No owner policy decision remains held. Procedural pre-authoring remains procedural;
this is not automated semantic truth validation or authenticated human identity.
Historical matrix observations and failed receipts remain described in the dated
records; the CSV records the final bounded disposition. Structural positive and
negative controls ran in the focused document suite; this is not exhaustive
per-clause transition certification. Source status never proves execution approval.

Final focused evidence: 21 B11 tests, 60 document tests (overlapping selection),
10 MCP tests, scoped all-feature Clippy; six existing proxy unit controls; 47
isolated production CLI/MCP assertions; exact source selection and actual-root
bootstrap preservation. These categories are separate, not additive test counts.
The production executable remains
4e3da8ee0b160938a238fc273271ce24d1574348c60d16b5b047ae936260387f.
No Rust or protected-source edits followed its verified proposal capture.

Outstanding delivery: user-owned encoding/file-size guards, normal guarded commit,
exact committed-state review and push on (fix)Performance-Regression-Issues.
Full repository gates remain CI-only. Live CBM remains failed/degraded. All sixteen
independent evaluations remain UNRUN; no B10, new mission, or certification claim.