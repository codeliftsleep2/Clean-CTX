# Issue #136 Final Acceptance Reconciliation

**Investigator:** Agent MaxHeadRoom

## 1. Executive verdict

**#136 COMPLETE REPAIR IMPLEMENTED - VERIFICATION REQUIRED**

The approved selector architecture and all bounded #136 residual repairs are
implemented. Runtime verification remains entirely pending. Same-owner
candidates retain multiplicity; language-derived signature selectors address
them; legacy names work only when unique; recognized unsupported declarations
return explicit uneditable errors; and mutation/message residuals are repaired.

## 2. Original contract

The issue requires whole-declaration insert-before; independent overload
selection and truthful ambiguity; support or kind-specific refusal for nine
uneditable construct families; and reconciliation of separator, deletion,
selector, message, syntax, receipt, and TypeScript-listing observations.

## 3. Acceptance matrix

| Obligation | Status | Finding |
| --- | --- | --- |
| C#/TS first, middle, metadata insert-before | SATISFIED | Separate declaration start; 9 tracked cases and historical 23/23 live. |
| Never substitute body start | SATISFIED | Missing declaration start refuses. |
| Leading documentation placement | SUPERSEDED | Approved ownership rule requires refusal when syntax does not prove sibling-comment ownership. |
| Three overloads independently editable | SATISFIED | Ordered parameter types form caller-visible signature selectors; candidate multiplicity is retained. |
| Same name across distinct owners | SATISFIED | Qualified lookup separates; bare lookup refuses ambiguity. |
| Getter/setter and nested/outer precision | SATISFIED | Signature/owner regressions select exactly one declaration. |
| Missing target refusal | SATISFIED | Fails before splice/write. |
| All ambiguity truthful | SATISFIED | Omitted or still-colliding discriminators return ambiguous_target. |
| Nine constructs supported or explicitly uneditable | SATISFIED | Expression bodies use body authority; other named families retain parser-derived uneditable kinds. |
| Insert-after separator | SATISFIED | Missing leading newline gains file EOL and anchor indentation. |
| Delete declaration plus metadata | SATISFIED | Declaration mapping/tests establish this. |
| Delete docs/whitespace/precondition truth | SUPERSEDED | Unproven docs now refuse; full declaration preconditions are accepted, with legacy body compatibility. |
| Generic selector consistency | SATISFIED | Unique generic methods accept base-name and signature aliases; collisions refuse. |
| Decorated non-exported TS owner | SUPERSEDED | Closed #139 owns this baseline. |
| Class-name/constructor collision | SATISFIED | The class declaration and constructor remain distinct candidates; bare class name refuses ambiguity. |
| Wrong expected-text message | SATISFIED | Message now states that expectedOldText mismatches the current tracked body. |
| Stale source safety | SATISFIED | Hash checks plus commit-time byte CAS. |
| Syntax safety and verify receipts | SATISFIED | Candidate gated; replacement/insertion text reported. |
| TS type-literal listing | OWNED ELSEWHERE | Context/IR classification, not edit mutation. |
| Existing preservation guards | SATISFIED | Historical evidence retained; not rerun. |

## 4. Current edit authority chain

apply_edit -> path/prior ownership -> source hashes -> current Edit compile ->
body UnitTable -> declaration mapping where needed -> string resolution ->
boundary/precondition -> atomic plan -> syntax/recompile -> commit byte CAS ->
atomic/durable commit -> live publication.

IR/UnitTable own edit-addressing identity, CoreOp Body owns body spans, and
tree-sitter captures joined by containment/exact end own declaration spans. A
request-local syntax inventory owns unsupported-kind evidence. The handler owns
source CAS and response messages.

## 5. Whole-declaration boundary reconciliation

Commit c14a6cd0 introduced insert_before_byte separately from body/deletion
spans. Nine tracked dispatcher tests cover both languages, positions, metadata,
parseable comment placement, and body-only refusal. Historical evidence is RED
0/9, GREEN 9/9, and live 23/23; none was rerun. The approved rule supersedes
the original leading-doc placement expectation when sibling ownership is not
structurally proven: insert/delete now refuse rather than scanning backward.

## 6. Same-name / overload precision

Materialization now retains every same-owner candidate. Ordered parser-derived
parameter types form Owner.Name(type,...) selectors. Legacy Owner.Name and bare
names resolve only when one combined editable/uneditable candidate remains.
Signature collisions also return ambiguity. Source ordinals, opaque handles,
and repository-wide identity were deliberately not introduced.

## 7. Unsupported and uneditable constructs

Queries see named unsupported constructs, and the request-local inventory now
retains their owner/name and parser kind beside body-bearing edit units.
Resolution returns uneditable_target without creating a durable/second index.
Expression-bodied methods retain existing body authority. Tracked cases cover
the other named families, unchanged bytes, and no neighboring mutation.

## 8. Error / refusal message reconciliation

The existing error machinery now distinguishes not_found, ambiguous_target,
uneditable_target, unit_mismatch, and stale_edit_source. Missing declaration
authority is reported as uneditable with its known boundary reason. Mismatch
prose states exactly which expectedOldText comparison failed. No global
diagnostics framework was added.

## 9. Stale-source / mutation safety

Disk bytes must agree with live and durable hashes before resolution. After
planning and validation, commit re-reads and exact-compares the original bytes.
A mismatch commits nothing. Successful writes use staged bytes, durable intent,
atomic replacement, semantic commit, and prior-byte recovery. No global
transaction redesign is required for this criterion.

## 10. Repairs implemented

- Preserved every same-owner candidate instead of overwriting qualified keys.
- Added language-derived Owner.method(type,...) signature selectors.
- Kept legacy qualified/bare and generic-base aliases only when unique.
- Added request-local parser-derived uneditable declaration inventory.
- Split not_found, ambiguous_target, and uneditable_target response kinds.
- Corrected expectedOldText mismatch prose.
- Added safe insert-after EOL/indent separation and exact applied-text receipts.
- Accepted full-declaration delete guards while preserving legacy body guards.
- Refused whole-declaration edits when leading documentation ownership is not
  structurally proven.
- Updated the public schema and authoritative apply_edit guidance.

## 11. Regression inventory

Existing and newly added tracked evidence:

- src/tests/mcp/insert_before_declaration.rs: nine dispatcher cases.
- src/tests/edit/locate.rs: lookup, cross-owner ambiguity, missing/spanless.
- src/tests/edit/delete.rs: declaration and metadata deletion.
- src/tests/edit/apply.rs: splice, mismatch, syntax, preservation.
- src/tests/mcp/apply_edit.rs: schema, envelopes, errors, verify behavior.
- src/tests/mcp/apply_edit_transaction.rs: transaction/CAS behavior.
- src/tests/mcp/apply_edit_issue_136.rs: C#/TS overload selection, getter/setter,
  nested/outer owners, ambiguity, unsupported kinds, expression bodies,
  constructor collision, missing targets, and byte-identical refusal.

The initial selector/public regressions were written before production changes;
additional preservation guards were added during implementation. The authorized
task forbids execution, so no RED-to-GREEN runtime claim is made yet.

## 12. Verification evidence

Tests were inspected, not executed. Historical insertion evidence is RED 0/9,
GREEN 9/9 and live 23/23. Neighboring historical filters record 12 pass/3
ignored and 61 pass. The operator's first focused run exposed the documentation
refusal being classified as ambiguous because the body-backed method was also
entered in the unsupported inventory. That ownership bug was repaired. The operator subsequently reported the focused
regression green and reported later user-run gates green; the latter commands were
not identified, so no broader or final-gate PASS is claimed.

## 13. Cross-issue boundary audit

#133, #134, #135, and #140 remain closed. #139 owns decorated TS owner identity.
#137, #138, broader #139, #141, and #142 were not absorbed. The TS type-literal
listing anomaly is owned elsewhere. Work did not proceed to #137.

## 14. Remaining residuals

No known bounded implementation residual remains. Runtime evidence is missing.
Documentation comments remain intentionally unowned unless structured syntax
proves association; whole-declaration operations refuse those targets. No new
canonical identity, durable index, CBM authority, ordinal, or opaque lifecycle
was introduced.

## 15. Closure recommendation

#136 is not ready for closure until verification passes. Implementation is
complete under the approved selector and documentation-ownership architecture.
Run the focused tracked tests, preservation filters, applicable live harness,
and authoritative final gate before using an acceptance-green verdict.

## 16. Commands run

Actually run: read-only Get-Content; gh issue view 136; bounded rg; git status
--short; bounded git log and git show; patch helper for this report.

Not run: Cargo or formatter commands; binaries/MCP/live scripts; provider/CBM;
model pilots; UTF-8/file-size/final gates; GitHub writes; commits/pushes/PRs.

Recommended next verification commands:

    cargo test --locked -p clean-ctx --lib --all-features -j 4 issue_136_
    cargo test --locked -p clean-ctx --lib --all-features -j 4 apply_edit
    cargo test --locked -p clean-ctx --lib --all-features -j 4 edit::
    pwsh -NoProfile -File verification/issue-reconciliation/Invoke-InsertBeforeDeclaration.ps1 -BinaryPath target/debug/clean-ctx.exe
    pwsh -NoProfile -File verification/issue-reconciliation/Invoke-Issue136Acceptance.ps1 -BinaryPath target/debug/clean-ctx.exe
    pwsh -NoProfile -File scripts/check-utf8.ps1
    pwsh -NoProfile -File scripts/check-file-sizes.ps1
    git diff --check

The consolidated live harness is now tracked. The
authoritative final gate remains docs/agent/verification.md.
