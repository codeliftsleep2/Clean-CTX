# #137 semantic signature evidence repair

## 1. Verdict

**#137 SEMANTIC SIGNATURE EVIDENCE SLICE COMPLETE**

The return-type and same-boundary authorization-attribute repair is GREEN at the
registered diff_commits boundary. The entire #137 issue is not resolved. #136,
CBM A0–B5, acquisition, native framework publication and #142 remain untouched.
No issue closure, push or merge was performed for this slice.

## 2. Proven root cause

**PROVEN BY CODE INSPECTION, TEST and LIVE MCP:** Git delivers different committed
files and both members are extracted. build_snapshot forces Medium for stable
display extraction. Its method capture closure calls extract_method_sig; that
strips C# attributes and compact_method_medium drops the return-type-first return
type. Both int Get() and long Get() therefore become the same display signature.
CapturedMethod previously held only that signature, markers and body. Identical
bodies/markers meant the comparator received equal reduced values and truthfully
compared those values, then falsely reported semantic equality to the caller.

Authorization attributes disappear at this exact same capture-to-snapshot
transition, rather than a separate framework admission boundary. The approved
conditional attribute case is included for that reason, not as general fidelity
work. Parameters/body already working demonstrate that Git or all comparison is
not universally broken.

Old path:

```text
distinct Git blobs → method AST → reduced Medium signature
→ identical snapshot values → unchanged → misleading MCP result
```

## 3. Repair: producer-owned evidence separate from display

The C# snapshot producer extends only its local query with method_declaration
returns-field and attribute_list captures. The existing capture walk returns
query-match IDs and exact source spans. Those associate each return/attribute
with its owning method declaration; no field is inferred from a preceding token,
rendered signature, name, whitespace split or textual proximity.

CapturedMethod.declaration carries optional DeclarationEvidence with written
return_type and ordered attributes. These are AST-designated source-node values;
the comparator treats them as opaque equality evidence, without C# spelling rules.
The display sig stays the existing compact value. Constructors and producers
without this evidence retain their existing comparison fields/behavior.

The shared comparator includes declaration evidence in equality. If it differs,
the same matched member is Modified with reason declaration; the formatter emits
the compact `(declaration changed)` indication. It does not expand every Medium
signature, fabricate the type, or report a whole-class textual difference as every
child changing. Global language/IR/compression queries and fidelity levels are
unchanged. The existing snapshot walk still parses once; no second parser was added.

New path:

```text
distinct Git blobs → existing parser walk
→ compact display + separate structured declaration evidence
→ same member, unequal declaration facts → Modified
→ compact truthful change indicator → registered MCP result
```

Written node spellings are compared directly. Compiler-resolved type aliases,
cosmetic attribute normalization, full modifier/constraint semantics and unsupported
member forms are not claimed established. This is the minimum evidence for the
proven method return/attribute distinction, not a new public completeness contract.

## 4. RED → GREEN matrix

The corrected permanent regression first ran against unchanged production:
**three tests RED**, then **the same three tests GREEN**. Only regression/module
registration were stashed while the production patch was applied. Formatting
subsequently changed layout only. Two additional preservation controls brought
the final focused result to **5/5 PASS**.

Live immutable history:

- Commit A: `bbae8476c85a43cf2448de9841fd53e983a8365b`.
- Commit B: `ccd9e5884f0239f9af721dc8164593bdf9afc8e9`.
- Both runs use the same owned repository and exact commit hashes.
- All five files are Git-modified; Unchanged.cs changes only a comment, while its
  declaration/body remain identical. Source at both commits and Git name-status
  evidence are stored in the manifest/provenance, and admitted against the oracle.
- RED: `target/semantic-signature-evidence/20261012-030223-98660f63`:
  ten false-unchanged failures, fifteen passing controls.
- GREEN: `target/semantic-signature-evidence/20261012-031015-bd30c92b`:
  **25/25 case groups PASS**.

| Case | RED | GREEN | Semantic oracle | MCP boundary |
| --- | --- | --- | --- | --- |
| int Get() → long Get(), same body | Class falsely unchanged at all five fidelities | Same Get modified, declaration changed | Compact sig unchanged privately; source-proven return facts differ | Dispatcher + live stdio |
| Authorize Reader → Admin, same method/body | Class falsely unchanged at all five fidelities | Same Get modified, declaration changed | Owned attribute node differs | Dispatcher + live |
| Parameter int value → long value | Changed | Changed with existing signature presentation | Existing compact parameter evidence | Dispatcher + live |
| Last-method body return 1 → return 2 | Changed body | Same changed body behavior | Existing normalized body fingerprint | Dispatcher + live |
| Identical declaration/body, comment-only file change | Unchanged | Unchanged | Git changed file is not semantic member change | Dispatcher + live |
| Return/attribute target followed by identical Neighbor | Neighbor unchanged | Neighbor unchanged | No whole-class textual inequality substituted | Dispatcher + live |

Fidelities executed: low, medium, high, edit, verbatim. A successful RPC alone is
not counted as semantic acceptance: each file's section is checked for the intended
member change/unchanged result and absence of unjustified neighbor modification.

### Separate anomalies retained, not repaired

The initial body fixture changed the first method and exposed the already-known
neighbor marker-attribution defect: Get body changed, Neighbor markers changed.
Its saved live run is `20261012-025942-834a3934`; the initial candidate's dispatcher
output is retained in /tmp/137-green.log. This behavior predates this repair and
is not caused by return/attribute evidence. No marker-attribution production code
was changed.

The preservation body fixture was corrected to the established last-method body
contract. Return/attribute fixtures still keep their original following-neighbor
assertions. Production was stashed back to baseline and corrected RED re-run
before freezing/restoring the corrected regression. The corrected live history
was created before the candidate build and reused verbatim for GREEN. Initial
failures were not relabeled passes or silently removed from the evidence.

An additional preservation fixture initially assumed Java syntax containing
public class would select the Java producer. Current detection checks the C#
heuristic first. The private no-C#-evidence assertion was invalid for that fixture;
using Java final class avoids that ambiguity and proves the intended Java producer
control. No detector fix or weakened public equality oracle was introduced.
The original private failure is recorded in the investigation command history;
it is not a production RED claim. Windows/platform execution remains unestablished.

## 5. Identity versus equality

Existing class/member keys and overload grouping are unchanged. Get remains the
corresponding member across revisions; a changed return/attribute does not produce
an unrelated removed/added pair. Independent declaration equality determines that
the existing corresponding member changed. Existing sig/marker/body comparisons
continue to operate, with the additional producer fact comparison.

Attribute order/occurrences are retained by source spans. Query-match identity
ties a child capture to its actual owner, including multiple attribute lists.
No generic comparator-side parser, framework interpretation or identity graph
was introduced.

## 6. Executed preservation

**PROVEN BY TEST:** existing diff:: filter passed **94 tests**; this filter also
matches gitdiff:: and therefore executed the requested Git preservation surface
without a redundant second run. Results cover file/member add/remove, unchanged
classes, imports, body comparisons, constructors/base/this initialization, ref
validation/resolution, historical source access and TypeScript/Java/Rust examples.
The existing binary-related runner coverage exercised missing Git executable
errors; arbitrary binary-file content classification was not independently scored.

**PROVEN BY TEST:** new controls execute generic method/constructor unchanged
behavior and nonempty TypeScript/Java/Rust snapshots with unchanged cross-fidelity
comparison. Synthetic old test snapshots explicitly supply no producer evidence,
preserving their established expectations. The former oversized differ test file
was split without changing existing assertion logic to respect the 615-line limit.

**PROVEN BY LIVE MCP:** parameter, supported last-body and unchanged/neighbor
controls at all five fidelities; five-file Git detection and exact SHA resolution.
No new claim is made for merge/rename edge cases beyond actually executed existing
tests, expression/member forms not in the matrix, compiler semantic equivalence,
or model-host consumption/rendering.

## 7. #135 interaction

Global context signature extraction, Medium return rendering, DI caps, call
ownership and batch grammar were not changed. #135 acceptance was not run or earned.

**INCIDENTAL EFFECT — NOT #135 ACCEPTANCE:** code inspection establishes that
diff_code_context also consumes build_snapshot/diff_snapshots. Its shared comparison
can benefit from these retained facts, but no live acceptance of that separate
surface occurred here. Static provide_code_context signature fidelity is unchanged.

## 8. Remaining #137 work

- First-method/body-region marker attribution to neighboring methods: reproduced
  separately and not repaired.
- Unchanged-member noise and generic loss/compares disclosure.
- Modifier/constraint and unsupported member declaration comparison not covered
  by the retained returns/attribute_list method facts.
- Ref grammar changes (^1/^2), binary-content labeling and other original
  working-tree/merge/rename acceptance dimensions beyond existing executed tests.
- Cosmetic whitespace equivalence, compiler-resolved type equivalence and
  original real-workspace/platform corpus.

#137 remains open. None of these criteria was bundled into this production patch.
The #136 file-identity reuse observation remains separate; this history harness
does not structurally edit source or claim that anomaly repaired.

## 9. Changed files and responsibilities

| File | Responsibility |
| --- | --- |
| src/diff/csharp_evidence.rs | Diff-local C# AST capture patterns and match/span-owned fact adapter |
| src/diff/snapshot.rs | Separate typed declaration comparison evidence from display signature |
| src/diff/builder.rs | Collect that evidence during the existing single capture walk and attach by declaration span |
| src/diff/differ.rs | Include opaque declaration evidence in equality without C# semantics or identity changes |
| src/diff/formatter.rs | Show compact truthful declaration-change reason |
| src/diff/action.rs | Document the additional reason string |
| src/diff/mod.rs | Register the local producer adapter; original file EOL retained |
| src/tests/diff/differ.rs, differ_controls.rs | Preserve synthetic fixture evidence defaults; split existing oversized tests unchanged in behavior |
| src/tests/diff/formatter.rs | Supply absent producer evidence in existing synthetic fixtures |
| src/mcp/mod.rs | Register permanent focused MCP tests |
| src/tests/mcp/semantic_signature_evidence.rs | Same-display/different-evidence regressions, committed public matrix, producer and constructor/generic preservation |
| verification/issue-reconciliation/Invoke-SemanticSignatureEvidence.ps1 | Reusable live immutable-history MCP RED/GREEN oracle |
| verification/issue-reconciliation/README.md | Operator command, immutable history reuse and limitations |
| docs/plans/POST_CBM_133_141_NATIVE_DEFECT_RECONCILIATION.md | Link follow-up while preserving original historical findings |
| This report | Authority path, executed evidence, anomalies and remaining scope |

## 10. Actually executed commands and artifacts

Cargo commands used the existing /workspace/.tooling Rust installation and ran
in /workspace/Clean-CTX, with PATH prepended by /workspace/.tooling/cargo/bin,
RUSTUP_HOME=/workspace/.tooling/rustup and CARGO_HOME=/workspace/.tooling/cargo.

| Executed command | Result |
| --- | --- |
| cargo test --locked -p clean-ctx --lib --all-features -j 4 semantic_signature_evidence_ | Initial RED 0/3; initial candidate 2/3 exposed separate body control; corrected baseline RED 0/3; corrected GREEN 3/3; extra preservation control iteration 4/5 exposed fixture language assumption; final 5/5 |
| cargo test --locked -p clean-ctx --lib --all-features -j 4 diff:: | 94/94, including gitdiff::; no ignored tests |
| Focused rustfmt --edition 2024 --config skip_children=true on changed Rust files | Completed; regression inputs/expectations unchanged by formatting |
| cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings | Passed, including final preservation additions |
| cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4 | Single candidate build passed |
| PowerShell Invoke-SemanticSignatureEvidence.ps1 against pre-fix binary | Initial body-anomaly run retained; corrected RED 10 fail / 15 pass |
| Same harness with -HistoryPath pointing to corrected RED history, candidate binary | 25/25 PASS; same exact commits/sources |

Pre-fix binary SHA256:
`30cddb3280c0a2df79a25395fdb62fba35464b52f127efdba0a0389a1a36bf0d`.
Candidate SHA256:
`4c8e244888d18d7d7b2db55f060542d5d491b82452c9956b1cf769ddd62f52ed`.
Exact sources, both commits, requests/responses, stderr, result sections and
verdicts are under the run roots above. PowerShell used -NoProfile and the installed
workspace pwsh with workspace XDG settings. Source history was owned and CBM disabled.
No full suite, live CBM provider, #135 implementation, or model-host pilot ran.

PowerShell AST, UTF-8/no-BOM, changed-code size and git diff checks (with
cr-at-eol for the original CRLF Rust files) complete the
artifact checks. Stop here; every remaining production slice needs separate review.
