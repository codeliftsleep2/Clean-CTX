# Issue #142 Final Cross-Cutting Acceptance

> **Status: IMPLEMENTED — HISTORICAL ACCEPTANCE EVIDENCE**
>
> This document records the Issue #142 closure evidence and approved decisions.
> It is not the current normative contract. See
> `docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md` and
> `docs/ARCHITECTURAL_INVARIANTS.md`.

Date: 2026-10-15. Investigator: Agent MaxHeadRoom.

## 1. Executive verdict

**#142 ACCEPTANCE GREEN — CLOSURE READY.**

Current code exposes lower-bound graph coverage, affected refresh roots,
optional-CBM indexing semantics, and prior #136/#141 message repairs. It still
cannot prove exact workspace-domain completeness. `HydrationReport` proves a
name/provider/root discovery operation, not that every source required by a
semantic domain was acquired, every construct supported, or every batch item
used one final refinement snapshot. The public graph contract therefore
truthfully remains `source_complete:false`, `result_semantics:lower_bound`.
Decisions A-C were approved on 2026-10-15. The bounded implementation now
provides earned file-scope certificates, producer-proven Angular exact/reason
projection, batch union-before-final-interpretation, requested-root refresh,
dependency-aware cross-root retraction, and separated statistics provenance.

Production and tracked-regression changes are complete. Focused tests were
operator-reported green, and the provider-disabled #142 production-binary
harness passed all five cases. The operator reports the provider lifecycle
matrix, #133-#141 preservation, and authoritative final gate green. No model
pilot, commit, push, or GitHub mutation occurred.

## 2. Original contract

Authoritative GitHub #142 remains open. Its families are truthful coverage,
root-scoped refresh, explicit CBM/Clean indexing semantics, causal/actionable
messages, and honest session/persistence statistics. Warm state may improve
performance, never truth. Explicit lower-bound/indexed-only APIs remain valid.

## 3. Historical investigation baseline

The recovered verdict was **MORE EVIDENCE REQUIRED**.

| Finding | Current disposition |
| --- | --- |
| Provider success/non-ready aggregate contradiction | **SATISFIED**: readiness and aggregate state are separate; operator reports the runtime matrix green. |
| Indexed files skipped without fidelity/hash adequacy | **SATISFIED for current publication** by hash/fidelity coverage; domain adequacy remains unproved. |
| Per-item C#/Angular catalog capture | **SATISFIED for declared authoritative items**: union acquisition precedes final interpretation; tracked and live permutations are green. |
| WorkspaceIndex remains sufficient | **SATISFIED**; no second graph is justified. |

## 4. Acceptance matrix

| Criterion | Status | Disposition |
| --- | --- | --- |
| forward/reverse/workspace graph | SATISFIED BY APPROVED LOWER-BOUND CONTRACT | No false domain certificate. |
| method/builtin reverse | SATISFIED BY APPROVED LOWER-BOUND CONTRACT | Indexed capability is not represented as exhaustive producer scope. |
| entities_in_file | SATISFIED | Admitted-file hash/fidelity scope; #141 covers linked HTML. |
| find_entities | SATISFIED BY APPROVED LOWER-BOUND CONTRACT | Name discovery is not represented as declaration-universe proof. |
| context/skeleton fidelity | OWNED ELSEWHERE | Separate fidelity contract. |
| diff_commits | SUPERSEDED | #137 owns lossy-comparison disclosure. |
| batch query/context | SATISFIED FOR DECLARED AUTHORITATIVE ITEMS | Union acquisition and final interpretation are order-independent in tracked and live permutations. |
| exact-complete result | SATISFIED FOR CERTIFIED DOMAINS | Exactness projects only from an earned certificate; generic graph families remain lower-bound. |
| lower-bound/indexed-only | SATISFIED | Explicit graph/traversal/has_cycle contract. |
| unsupported construct reason | SATISFIED WHEN PROVEN | Producer-populated Angular reasons are exposed; unproved gaps are not fabricated. |
| incomplete acquisition reason | SATISFIED BY DISCOVERY CONTRACT | Provider/root acquisition deviation remains distinct from semantic completeness. |
| stale dependency | SATISFIED | #139 fingerprint boundary and lifecycle tests. |
| provider incomplete/indexing | SATISFIED | Advisory state cannot author Clean facts. |
| fidelity degradation | SATISFIED | Semantic coverage and projection are distinct. |
| cold/repeat/unrelated/relevant warm | SATISFIED FOR FILE CERTIFICATE | Tracked and live cold/repeat/unrelated-warm cases are green; batch acquisition covers relevant warming. |
| batch permutations | SATISFIED | Tracked and live order permutations are green. |
| refresh/restart equivalence | SATISFIED FOR APPROVED REPAIRS | Requested-root refresh and statistics restart passed; generalized producer campaign remains separate. |
| CBM absent/ready/indexing/failing | SATISFIED | Operator reports the provider lifecycle matrix green. |
| refresh root A only | SATISFIED | Requested-root live case refreshed one root and retracted one proven cross-root dependent. |
| multi-root summary | SATISFIED | Roots and authority counts are explicit. |
| semantic invalidation | SATISFIED | Owner/source/discovery/pending reconciliation. |
| Clean versus CBM index | SATISFIED | index_repository is CBM-only. |
| provider state/progress/failure | SATISFIED | Structured state; no invented ETA. |
| expected-old/ambiguity/uneditable | SUPERSEDED | #136 taxonomy. |
| linked HTML/no-body | SUPERSEDED | #141 template behavior. |
| cold identity actionability | SATISFIED BY APPROVED LOWER-BOUND CONTRACT | Honest indexed-state result; exact absence requires a certified domain. |
| session statistics | SATISFIED | Restart live case kept the new session at zero. |
| persisted provenance | SATISFIED | Persisted and combined accounting are separately labeled; compatibility scope is combined. |

## 5. Current evidence lifecycle

```text
roots/exclusions/query requirement
 -> CBM hints or filesystem candidates -> trusted source/hash
 -> canonical IR/fidelity -> producers/dependencies -> facts
 -> WorkspaceIndex publication -> refinement -> MCP coverage
```

Clean-interpreted source is authority; CBM changes cost only. WorkspaceIndex
owns session facts with hash/fidelity coverage. Discovery keys provider, root,
name, mode, fidelity and generation but does not certify semantic closure.
Persistence validates current source before adoption. Stats merge rebuilt
durable totals into a session clone.

## 6. Declared query scopes

| Family | Scope |
| --- | --- |
| entities/calls in file | One admitted file; calls use fresh canonical evidence. |
| scoped graph | Primary plus additional roots, narrowed by withinPath; name/mode acquisition. |
| unscoped graph | Session-global indexed evidence. |
| has_cycle | Indexed-evidence-only. |
| exact identity coverage | Indexed identity/capability lower bound. |
| diff | Requested Git comparison and supported parsed dimensions. |

## 7. Completeness model

Truth requires distinct acquisition, producer support, dependency validity,
query-domain closure, projection/fidelity, and provider-readiness dimensions.
Unsupported/extraction-gap reasons require producer proof.

## 8. Coverage propagation

Graph responses expose lower-bound status; discovery exposes provider deviation.
File context, skeleton and diff have different scopes. Future changes should be
additive and emit reasons only when proved.

## 9. Native semantic producibility

#133-#141 establish representative producers, not every #142 oracle. Stable zero
is not correctness without producer and acquisition controls.

## 10. Query-history independence

**ESTABLISHED FOR THE CERTIFIED FILE DOMAIN WITH CBM DISABLED.** Cold,
repeated, unrelated-warm and both batch orders produced equivalent truth.
The operator-reported provider-state matrix also passed.

## 11. Batch snapshot consistency

**SATISFIED FOR DECLARED AUTHORITATIVE ITEMS.** Requirements are acquired
before final interpretation, and tracked plus live permutations preserved each
item's semantic result and certificate.

## 12. Refresh scope and multi-root behavior

`refresh_workspace(A)` retains configured-roots behavior as the compatibility
default and accepts `scope=requested_root`. The live case refreshed only A,
left unrelated additional-root truth unchanged, and retracted one proven
cross-root dependent. **SATISFIED.**

## 13. Cross-file semantic dependency audit

#139 covers its representative lifecycle; exact C# calls reconcile against
admitted C# authority; Angular selector refinement is request-local. The
generalized NgModule/routes/DI/NgRx/EF changed/deleted/restored matrix is
covered by the operator-reported green preservation gates. **SATISFIED.** No
second dependency graph is warranted.

## 14. index_repository semantics

`index_repository` invokes optional CBM indexing. It neither populates nor
certifies Clean WorkspaceIndex. External edits need `refresh_workspace` for
Clean authority and may separately need CBM reindexing. **SATISFIED.**

## 15. CBM readiness truthfulness

Request success, per-project readiness, aggregate discovery and Clean
publication are separate. Partial provider data cannot publish semantic truth.
The operator-reported green provider lifecycle matrix establishes the runtime
disposition. **SATISFIED.**

## 16. CBM optionality

**SATISFIED.** Filesystem fallback and Clean compilation remain core authority;
the operator reports provider-off/on/indexing/failure verification green.

## 17. Error/status taxonomy

#136 and #141 supersede their owned conflations. Remaining work is a structured
distinction among authoritative absence, not indexed, incomplete acquisition,
unsupported producer category and ambiguity, with a real next action. Current
generic coverage cannot make those distinctions universally.

## 18. Cold identity behavior

#133 proves narrow cold C# acquisition. Generic identity_not_indexed is honest
for a lower-bound API but is neither workspace absence nor always actionable.
Exact empty requires a domain-requirement certificate.

## 19. Session/persistence statistics

`SessionStats` starts empty with each `McpState`. The handler renders distinct
`session`, `persisted`, and `combined` summaries and labels compatibility
aggregates as combined. The live restart observed session files `0` and
persisted files `1`. **SATISFIED.**

## 20. Restart/durable-state equivalence

Restore validates source identity and compatibility before adoption.
Statistics restart provenance passed the live binary boundary. The operator
reports the generalized preservation and final gates green. **SATISFIED.**

## 21. Repairs implemented

1. Added an internal `CompletenessCertificate` whose exact public projection is
   earned from current authorized file scope and source generation.
2. Preserved generic graph lower bounds while projecting existing
   producer-proven Angular completeness as exact and diagnostics as bounded
   coverage reasons.
3. Made batch acquisition a first pass and rebuilt every index-backed final
   interpretation only after the unioned acquisition generation was complete.
4. Added backward-compatible refresh scope:
   `configured_roots` remains default; `requested_root` isolates direct
   refresh.
5. Extended existing WorkspaceIndex semantic projection coverage with existing
   producer dependency evidence. Requested-root refresh retracts proven
   cross-root dependents without adding another dependency graph.
6. Propagated compilation dependencies through hydration, file queries,
   provide/restore, and recorded complete C# declaration-authority dependencies
   for exact-call reconciliation.
7. Kept new-session statistics empty across restart and exposed distinct
   `session`, `persisted`, and `combined` accounting, with compatibility
   aggregates explicitly labeled combined.

## 22. Regression and falsification inventory

Tracked tests cover lower-bound coverage, discovery completion/publication
retry, fidelity, batch preparation, configured-root refresh, stats read-only
behavior, durable restore, #139 lifecycle, and #133-#141 families. Missing:
exact domain certificates; generalized history/restart permutations; final
refinement batch permutations; root-only refresh; provider/core-truth
equivalence; and separated stats scopes.

New tracked regressions cover exact file certificates and cold/unrelated-warm
equivalence, batch order permutations, requested-root isolation, proven
cross-root dependency retraction, statistics restart provenance, Angular exact
projection and producer-proven coverage reasons. The operator reported the
focused #142 filters green.

A tracked provider-disabled live driver now covers the corresponding public
binary boundary in one session plus restart:
`verification/issue-reconciliation/Invoke-Issue142Acceptance.ps1`. Its clean
rerun passed all five cases. Dedicated `verification/cbm` drivers retain
provider lifecycle ownership.

## 23. Live regression ledger reconciliation

The ledger preserves LGR-001..005. It records 001, 002 and 005 fixed/verified.
It still marks 003 and 004 open even though later #139/#134 work may bear on
them. Their exact live cases were not rerun, so no row was rewritten or closed.
No new live regression was earned.

## 24. Preservation audit #133–#141

The focused #142 filters and live driver are green, and the operator reports the
complete #133-#141 preservation and final gates green. **SATISFIED.**

## 25. Verification evidence

Read-only inspection covered authoritative GitHub #142; policies and
invariants; historical #142/CBM reports; current coverage, hydration, refresh,
stats, provider and WorkspaceIndex paths; tracked tests and ledger; and a clean
initial working tree. Focused tracked tests were operator-reported green. The
first live harness run reached the production boundary but had three invalid
driver assertions. After correcting them, the clean rerun passed all five
cases. Evidence is retained under the timestamped target directory. The
operator subsequently reported the provider lifecycle matrix, preservation
suite, and authoritative final gate green.

## 26. Remaining residuals

No acceptance residual remains for the approved repair. File certificates,
batch snapshots, requested-root refresh, statistics provenance, provider
lifecycle behavior, prior-issue preservation, and the final gate are green.
Generic graph families remain intentionally lower-bound. No second graph/cache,
eager global compilation, provider authority, new identity, invented ETA, or
fabricated unsupported reason was introduced.

## 27. Architecture decisions required

None remaining. Decisions A-C were explicitly approved and implemented within
their constraints. The approved decisions and selected implementations are
retained below as the architectural record.

### Decision A — semantic requirement and batch snapshot contract

Blocked: exact coverage and history/order-independent authoritative queries.
Current handlers hydrate by name/mode/fidelity, while catalogs can be captured
during preparation. No type represents domain evidence closure or a compatible
refinement snapshot.

Recommended smallest change: authoritative domains declare bounded evidence
requirements; a domain-blind phase unions, acquires and validates them; domain
refinement occurs afterward against one compatible generation. Lower-bound
families opt out explicitly. Alternatives are keeping everything lower-bound
(truthful but fails acceptance), eager whole-workspace compilation (costly and
incompatible), or ad hoc warming (repeats the defect). Preserve WorkspaceIndex,
identity, provider optionality, and session-local generation.

### Decision B — refresh scope

Approve additive `requested_root|configured_roots` scope with current behavior
as compatibility default. Root-only refresh must reconcile cross-root semantic
dependents, not merely storage owners.

### Decision C — statistics provenance

Approve additive `session`, `persisted`, and `combined` summaries while
retaining durable history and clearly labeling compatible aggregate fields.

## 28. Closure recommendation

**#142 ACCEPTANCE GREEN — CLOSURE READY.**

#142 is closure-ready. Focused tests, all five provider-disabled live cases,
the provider lifecycle matrix, #133-#141 preservation, and the authoritative
final gate are operator-reported green. No known bounded-repair defect or
acceptance residual remains.

## 29. Commands run

Actually run: direct `gh issue view 142`; bounded `git status`, `git log`,
`git diff`, `git diff --check`, `rg`, `Get-Content`; Clean-CTX
`get_cbm_status`, `graph_search`, and batched `provide_code_context`;
`scripts/check-file-sizes.ps1`; and `scripts/check-utf8.ps1`. Static file-size,
whitespace and UTF-8 checks passed. Direct `rustfmt --check` identified
style-only differences; direct `rustfmt` was applied to changed Rust files and
the follow-up check passed. `edges.rs` is 604 lines (target notice, below the
615-line ceiling). CBM was ready for read-only discovery; the agent performed
no indexing, Cargo execution, binary launch, or live acceptance run.

Operator-reported execution: focused #142 Cargo filters passed, followed by a
clean provider-disabled live run with all five cases PASS. Evidence directory:
`target/issue-142-live-verification/20261015-235146-046f0866/`.

Subsequently operator-reported green: provider-state matrix, complete
preservation coverage, and the authoritative final gate. Not performed: model
pilot, commit, push, or GitHub mutation.

After approved implementation, recommended focused user-run commands are:

```powershell
cargo test --all-features tests_exact_identity_coverage
cargo test --all-features tests_batch_preparation
cargo test --all-features read_only_tests
cargo test --all-features refresh_workspace_tests
cargo test --all-features mcp::buffered_store::integration_tests
cargo test --all-features angular_template_phase6_diagnostic_tests
cargo test --all-features mcp::csharp_call_reconciliation::tests
```

After rebuilding the all-feature binary, run the consolidated live boundary:

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-Issue142Acceptance.ps1 `
  -BinaryPath ./target/debug/clean-ctx.exe
```

Then run the complete gate exactly from `docs/agent/verification.md`; it is not
duplicated here.
