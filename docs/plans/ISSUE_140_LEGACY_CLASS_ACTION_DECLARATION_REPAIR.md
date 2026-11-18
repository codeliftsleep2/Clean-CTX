# #140 independent legacy class Action declaration repair

**#140 NGRX LEGACY EXTRACTION/LINKAGE SLICE PARTIALLY COMPLETE**

One coherent first producer boundary is complete: an independently declared, top-level TypeScript class implementing the directly imported NgRx `Action` interface now publishes an Action source occurrence and a truthful class marker. String-constant classification, switch handlers, constructor dispatch naming and declaration-resolved linkage do not share this mechanism. They remain separate. Stop after this slice; do not close #140.

## 1. Patch-boundary decision

**PROVEN BY CODE INSPECTION / LIVE MCP**: the defect family splits. Baseline and candidate partition fixtures are identical. The seven separate probes are recorded as OBSERVED, not PASS, and their observed identity/relationship facts are unchanged after the patch.

| Family / reproduced form | Earliest loss or unresolved authority | Decision |
|---|---|---|
| `class NoSuffixRecord implements Action` with direct named NgRx import | NgRx extractor ignores class declaration, despite valid TS class capture | **Repair independent declaration extraction/publication** |
| `const SET_AUDIT = '[Audit] Set Audit'` | no independent NgRx declaration classification; existing extractor recognizes createAction, not bare string variables | Separate boundary; do not use event-string/name convention or a matching use site |
| `switch(action.type) { case SET_AUDIT: ... }` reducer | no NgRx reducer/handler extraction; native function exists, but NgRx shape/reducer identity does not | Separate producer; no handler repair |
| modern creator declaration plus direct dispatch | declaration published correctly; Dispatches uses written modernAudit; exact declaration binding not represented | Preserve; no name-based resolution acceptance |
| `Actions.modernAudit()` | qualification preserved as endpoint name Actions.modernAudit, distinct from declaration modernAudit | Separate resolver boundary |
| `modernAudit as triggerAudit` | endpoint triggerAudit, not a declaration-resolved target | Separate alias resolution |
| `new NoSuffixRecord(...)` dispatch | existing lexical first-token extraction produces Action/new | Separate dispatch-expression interpretation; unchanged |
| sameAudit declarations in two files, use imports one | source occurrences are distinct, base graph identity is domain/type/name | Exact target-file selection **NOT ESTABLISHED** |

The historical #140 issue includes string constants as the dominant legacy form. That fact does not authorize guessing that an arbitrary string declaration is an Action. The string-constant control deliberately includes a Store import to hold the existing NgRx gate open; it still produces no Action declaration. The historical no-import string file remains historical evidence, not a new live trial in this task.

Class style is the earliest independently provable declaration producer. Its imported interface and explicit implements clause establish static source intent without requiring any dispatch/reducer/effect use. This is not a general NgRx resolver or an issue-sized patch.

## 2. Root cause and corrected authority path

**PROVEN BY CODE INSPECTION**: `ngrx/extract.rs::extract_actions` recognizes modern createAction calls. It does not recognize class implementations of the NgRx Action interface. The class and its name already survive TypeScript parsing/core compilation, but no NgRx Action declaration reaches the shape or publication carrier. This is extraction loss, not a recurrence of the frozen modern publication defect.

**PROVEN BY TEST / LIVE MCP**: baseline `provide_code_context` exposes the native class; `entities_in_file` has no ngrx/Action/NoSuffixRecord occurrence. After repair the same source yields exactly one Action occurrence in its declaration file, independently of any use site.

```text
structured TypeScript source
  ↓
existing class.root + frozen #139 canonical name/span
  + same-match top-level implements-clause evidence
  + same-match direct import from @ngrx/store
  ↓
NgRx-local ClassActionEvidence
  ↓
separate ClassActionDecl collection in NgRxShape
  ↓
existing self-Defines registration carrier
  ↓
existing domain-blind index normalization
  ↓
ngrx / Action / NoSuffixRecord / exact actions.ts source
```

Modern ActionDecl extraction and its self-Defines publication loop are unchanged. Legacy classes are not synthesized as createAction calls or inferred from the word Action in the class name.

## 3. Structured legacy extraction proof

**PROVEN BY CODE INSPECTION / TEST**:

- Supplemental NgRx-local query patterns run in the **existing single TypeScript parse/capture walk**.
- A direct named, unaliased `Action` import and its module literal share a match. The module literal must have one complete string fragment (`@ngrx/store`), with AST anchors excluding surrounding escape fragments. An unrelated escaped module cannot qualify from one matching substring.
- Class/abstract-class candidates must be direct program children or direct declarations of a top-level export statement. Their bare implements type and exact owner span share a match.
- The owner span joins to the existing #139 class.root span; the canonical class name is reused, not parsed again.
- Class type parameters named Action, competing top-level type bindings and conflicting Action import bindings are rejected rather than guessed. Nested, aliased and namespace-qualified implementations remain unsupported here.
- The existing NgRx applicability gate remains unchanged. Import-layout forms not admitted by that gate are outside this bounded support; no new global detection policy was introduced.

The producer records class, implements and import spans privately. Tests verify exact source slices for each role, multiple declarations, interface/import negatives, shadowing/conflicts and fake class text inside a string.

No regex, suffix heuristic, case/event-string heuristic, nearest-function/class search, source scanning for boundaries, second parser, compiler-grade module resolver or shared framework mega-abstraction was introduced. Comparisons of **captured identifiers** and a **captured complete module fragment** are syntax interpretation, not textual boundary recovery. The only textual gate is the pre-existing framework applicability function.

This is static declaration intent, not TypeScript type-check success or runtime Action validation. Event strings, payload contracts and action-type discriminant resolution are not inferred.

## 4. Declaration, reference and relationship separation

The primary fixture deliberately has **no use site** and a class name with no Action suffix:

```ts
import { Action } from '@ngrx/store';
export class NoSuffixRecord implements Action {
    readonly type = '[Audit] Set Audit';
    constructor(public payload: string) {}
}
```

**PROVEN BY TEST / LIVE MCP**: the public Action occurrence has exact kind, name and declaration source file; its forward relationship list is empty. Declaration publication does not invent dispatch, effect or reducer links.

Legacy markers use **`Φaction-class:` / `T @action-class`**, expanded as **NgRx Action class**. They do not use the modern createAction marker or expansion. Modern markers remain unchanged.

The public occurrence schema still has no declaration/reference role or span field. A dispatch endpoint can appear as an Action occurrence even without a recognized declaration. The undeclared-use probe records Action/unrecognizedAction in its reference file; this is an existing endpoint, not proof of an independently declared Action. No new declaration carrier is generated by a use site.

For the modern direct probe, `modernAudit` is independently published in modern-guard.ts and separately appears as a dispatch endpoint in modern-direct-use.ts. Matching names do **not** prove imported declaration identity. The existing edge object.file is the asserting/use occurrence, not a resolved declaration file.

## 5. Qualified, alias and collision status

- Direct written modern reference: **SUPPORTED as a symbolic use endpoint**, exact declaration binding **NOT ESTABLISHED**.
- Qualified modern reference: written qualification **SUPPORTED/preserved**; declaration-resolved linkage is a **SEPARATE DEFECT**. No terminal-suffix reduction was added.
- Aliased modern reference: written alias preserved; resolution is a **SEPARATE DEFECT**.
- Aliased/qualified class implements forms: **UNSUPPORTED by this bounded class declaration producer**, not newly accepted merely because they resemble Action.
- Constructor dispatch: recognized syntactic use produces wrong endpoint `new`; **SEPARATE DEFECT**, not fixed by declaring the class.

Legacy SharedRecord declarations in legacy-a.ts and legacy-b.ts retain distinct source occurrences. Modern sameAudit declarations in modern-a.ts and modern-b.ts also retain distinct source occurrences. A collision consumer imports only modern-a, but the graph uses its existing domain/type/name identity; no native result proves selecting that declaration over modern-b. The patch neither guesses a target file nor redesigns that identity contract. Full linkage/collision acceptance remains unresolved.

## 6. Permanent RED → GREEN

The registered primary regression was added before production changes. It checks native class capture/identity first, then exact public NgRx Action name/kind/file and absence of fabricated forward relationships.

- Initial run: invalid capture oracle expected `export` inside the class node. Tree-sitter puts export on its wrapper. The exact source and public oracle were unchanged when the capture assertion was corrected; this invalid run is retained and **not counted as RED**.
- Corrected registered regression: **0 passed / 1 failed**, missing exact Action declaration at the intended behavioral assertion.
- Stash only regression + test registration; implement producer/propagation; restore unchanged.
- Same focused command: **1 passed / 0 failed**.

The corrected primary remains byte-for-byte identical through final GREEN:

`ad5299e45ba51167447383a4d136aaf4b118028fee7def48ec3f30a07bea9687`

The focused matrix then passed **8/8**, **9/9** after compatibility coverage, and **9/9** after the complete-module-literal audit guard. These comprise the primary registered test, three producer tests, four registered MCP matrix/lifecycle tests and one compatibility test. Internal producer evidence does not substitute for the registered public RED.

## 7. Live RED/GREEN declaration matrix

Rows below are **PROVEN BY LIVE MCP** against baseline/candidate with identical source/oracle manifests. Empty negative rows mean non-admission, not newly supported legacy syntax.

| Case | Baseline Action name(s) | Candidate Action name(s) | Source oracle | Boundary |
|---|---|---|---|---|
| primary | [] | ['NoSuffixRecord'] | ['NoSuffixRecord'] | live MCP |
| non-export | [] | ['PlainRecord'] | ['PlainRecord'] | live MCP |
| abstract | [] | ['BaseRecord'] | ['BaseRecord'] | live MCP |
| multiple-interfaces | [] | ['MultiRecord'] | ['MultiRecord'] | live MCP |
| extends | [] | ['DerivedRecord'] | ['DerivedRecord'] | live MCP |
| multiline | [] | ['LayoutRecord'] | ['LayoutRecord'] | live MCP |
| double-quotes | [] | ['QuotedRecord'] | ['QuotedRecord'] | live MCP |
| type-import | [] | ['TypedRecord'] | ['TypedRecord'] | live MCP |
| ordinary-action-name | [] | [] | [] | live MCP |
| local-interface | [] | [] | [] | live MCP |
| foreign-action-import | [] | [] | [] | live MCP |
| aliased-interface | [] | [] | [] | live MCP |
| qualified-interface | [] | [] | [] | live MCP |
| string-constant | [] | [] | [] | live MCP |
| generic-shadow | [] | [] | [] | live MCP |
| nested-class | [] | [] | [] | live MCP |
| text-only-class | [] | [] | [] | live MCP |
| modern-guard | ['modernAudit'] | ['modernAudit'] | ['modernAudit'] | live MCP |

The modern guard remains published without references. Positive rows inspect exact Action kind/name/file, truthful marker, empty fabricated-link list, and repeated Low/Medium/High queries. Class generics with an unrelated type parameter are additionally covered by the registered MCP matrix; they were not an extra live static case.

## 8. Lifecycle and fidelity

**PROVEN BY TEST / LIVE MCP**: explicit refresh establishes LifecycleRecord, renames it to RenamedRecord with the old occurrence gone, replaces the declaration with an ordinary string constant with no Action occurrence, and deletes the file with no stale Action occurrence remaining. Repeated queries and source recompilation do not duplicate the source occurrence.

The class-specific marker and Action occurrence remain observable at Low, Medium and High. NgRx disabled configuration is checked by a registered MCP negative. Source bytes remain unchanged by observation.

No use/linkage rename lifecycle is claimed: linkage was not repaired. This establishes the tested declaration lifecycle only, not #142 cold/warm equivalence or query-history independence.

## 9. Production lifecycle and durable compatibility

**PROVEN BY CODE INSPECTION / LIVE MCP**:

1. `CoreIRPass` appends the local NgRx query to the existing TypeScript capture walk, after which unchanged #139 canonical identity is available.
2. `legacy::collect` builds independent evidence; private `ngrx.legacy.*` captures are removed before canonical emission/projection.
3. `PassContext.ngrx_class_actions` owns the compilation-local records. `MetaLayerPass` lends them through `MetaLayerContext` only for TypeScript.
4. `angular_semantic::evaluate` calls the unchanged modern extractor once, then the NgRx-local `legacy::include` adds a **separate** class collection when NgRx is enabled. One precomputed shape feeds both marker and semantic consumers.
5. `NgRxShape` emits a truthful class marker and self-Defines carrier. `IRCompiler` drains semantic edges; existing context/hydration publication paths and `WorkspaceIndex::add_edges` retain source occurrences and normalize registration out of forward/reverse relationship graphs.
6. Existing `provide_code_context`, `entities_in_file` and other workspace queries expose the results. Refresh/removal/deletion reuse existing lifecycle mechanisms.

No new graph, index acquisition policy, provider authority, module resolver, public occurrence schema or generic invocation field was added. Source-only modern extraction helpers remain unchanged; they do not synthesize class facts without upstream structured evidence. The default MCP compilation path supplies that evidence.

NgRx currently shares the existing Angular producer cohort. The existing marker generation advances **3→4**, semantic generation **5→6**. Canonical pipeline generation stays **1**. The compatibility regression verifies pre-class producer identities differ with unchanged config identities; the existing generation-mismatch rejection is also green. No old-database live migration is claimed.

## 10. Preservation actually executed

**173 selected preservation/alignment checks passed**. The publication filter includes nine unchanged modern/Service contracts plus one intentionally revised legacy guard; do not describe that guard as unchanged preservation. Its source stayed identical, and it now asserts exact class Action name/kind/file. All other publication assertions are unchanged.

| Executed filter | Passed |
|---|---|
| `angular_meta::ngrx::tests::` | 25 |
| `angular_meta::semantic_tests` | 18 |
| `framework_declaration_publication_` | 10 |
| `decorated_owner_identity_` | 9 |
| `angular_di_pipe_` | 9 |
| `angular_di_inject_token_` | 7 |
| `angular_inject_evidence_` | 4 |
| `angular_template_compatibility_tests` | 1 |
| `workspace_query_angular_selector_occurrence_tests` | 5 |
| `restore_rejects_semantic_producer_generation_mismatch` | 1 |
| `workspace::index::graph_tests` | 27 |
| `workspace::index::query_tests` | 26 |
| `workspace::index::edge_occurrence_tests` | 5 |
| `workspace::index::edge_lifecycle_tests` | 6 |
| `insert_before_declaration_` | 9 |
| `semantic_signature_evidence_` | 5 |
| `di_omission_` | 6 |

These checks ran before the final complete-module-fragment query tightening; the final nine-case NgRx filter and all-target Clippy/build ran after that guard. The guard changes only invalid/escaped module admission; the tested standard source forms are unchanged. No unexecuted broader preservation is claimed.

Existing publication PowerShell harness: **57/57 groups PASS**. Only its former legacy-class absence oracle was superseded; the source is unchanged and it now adds a truthful class-recognition group. Historical publication evidence remains the original 56-group record.

Package Clippy with all targets/features and `-D warnings` passed. One all-feature binary build passed. No full suite or CI gate ran.

## 11. Remaining #140 work and incidental effects

Remaining independent families:

- standalone string-constant/enum Action declaration authority and supported legacy forms;
- object-literal/qualified Action discriminants;
- switch/reducer handler identity and handler-to-Action reference evidence;
- constructor dispatch expression interpretation (`new` endpoint defect);
- modern/legacy exact declaration/use binding;
- aliases, namespace qualification, barrel/module resolution and same-name collisions;
- service-level dispatch/select ownership;
- independent Selector declaration publication and remaining effects/selectors criteria;
- representative repository precision/recall and cold/warm scope behavior.

The private representative corpus was not available or probed here. Repo-scale recall is **NOT ESTABLISHED**. No #141 template query or #142 hydration/query-history work occurred.

Incidental improvement: previously invisible typed legacy classes become discoverable, and their new marker distinguishes class declarations from action creators. No dispatch, effect or reducer linkage improves automatically; all seven partition observations retain their baseline identity/relationship facts. Do not promote declaration discovery to whole-issue linkage acceptance.

## 12. Changed files

Production:

- `src/angular_meta/ngrx/legacy.rs`: local structured query, direct-import/top-level-class qualification, shadow/conflict guards, exact owner association and shape augmentation.
- `src/angular_meta/ngrx.rs`: separate ClassActionDecl/shape collection and truthful class marker vocabulary/expansion. Modern ActionDecl fields/interpretation unchanged.
- `src/angular_meta/ngrx/shape.rs`: include class declarations in non-emptiness, render a class marker and emit the existing registration carrier in a separate loop. Modern publication loop, dispatch/handler/effect/selector relation logic unchanged. Header comments aligned with the addition.
- `src/ir/pipeline/core.rs` and `src/ir/pipeline.rs`: same-walk capture/assembly and compilation-local record ownership; private captures removed.
- `src/ir/pipeline/meta_layer.rs` and `src/layers/meta/context.rs`: borrow the local records at the default framework boundary.
- `src/layers/meta/angular_semantic.rs`: enrich the precomputed NgRx shape through the local producer. Angular DI logic unchanged.
- `src/layers/meta/mod.rs`: existing cohort generation increments only.

Tests/registration:

- `src/tests/mcp/ngrx_legacy_declaration.rs`: permanent exact registered RED/GREEN, no use dependency.
- `src/tests/angular_meta/ngrx_legacy.rs`: import/class spans, canonical names, scope/binding negatives and truthful non-modern carrier/marker.
- `src/tests/mcp/ngrx_legacy_declaration_matrix.rs`: class forms, negatives, disabled config, source-distinct names, deduplication/fidelity and declaration lifecycle.
- `src/tests/mcp/ngrx_legacy_declaration_compatibility.rs`: reject prior class-unaware marker/semantic products.
- `src/mcp/mod.rs` and `src/mcp/compatibility/mod.rs`: registrations only.
- `src/tests/mcp/framework_declaration_publication.rs`: only the explicitly superseded legacy absence guard; same source now requires exact independently published class identity.
- `src/tests/mcp/angular_template_compatibility.rs` and `src/tests/mcp/workspace_query_angular_selector_occurrences.rs`: current shared generation fixture alignment only; template behavior assertions unchanged.

Operator/docs:

- `verification/issue-reconciliation/Invoke-NgrxLegacyDeclarations.ps1`: established Phase B stdio pattern, 78 declaration acceptance groups and seven separately labelled OBSERVED partition probes.
- `verification/issue-reconciliation/Invoke-FrameworkDeclarationPublication.ps1`: same former legacy source, updated supported-class oracle/marker; modern sources and oracles unchanged.
- `verification/issue-reconciliation/README.md`: reproducible command and acceptance/observation distinction; publication group count aligned.
- Historical publication report and post-CBM reconciliation: narrow follow-up links, historical evidence retained.
- This report: root causes, partial boundary, evidence and stop.

## 13. Commands, artifacts and audit

Cargo toolchain environment used `/workspace/.tooling/cargo/bin`, `RUSTUP_HOME=/workspace/.tooling/rustup`, `CARGO_HOME=/workspace/.tooling/cargo`.

Executed commands:

- `cargo test --locked -p clean-ctx --lib --all-features -j 4 ngrx_legacy_declaration_`: invalid capture assertion run retained; corrected **0/1 RED**; unchanged **1/1 GREEN**; **8/8**, **9/9**, final audit **9/9 GREEN**.
- Same test command with each filter in Section 10: completed actual selected runs, 173 passed. No zero-test result is credited.
- Focused-file rustfmt, edition 2024, `skip_children=true`: passed; the preserved primary was not reformatted after RED.
- `cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings`: exit 0, no warnings.
- `cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4`: exit 0.
- Installed PowerShell `-NoProfile -File verification/issue-reconciliation/Invoke-NgrxLegacyDeclarations.ps1 -BinaryPath <binary>`: baseline exit 1, candidate exit 0.
- Installed PowerShell with `Invoke-FrameworkDeclarationPublication.ps1` against candidate: exit 0.
- Focused Git whitespace, strict UTF-8/BOM and active-file-size checks: passed.

| Artifact | Identity / result |
|---|---|
| parent binary | `target/debug/clean-ctx-ngrx-legacy-baseline`, SHA256 `649ebffad9cb908710fdaeea72d832f75e3fad3267e21b57e0c0d76d03d20fe3` |
| candidate binary | `target/debug/clean-ctx`, SHA256 `ae26ae8f9598e7456d5dcea0a6cde8df80b9d8d828ec7d162b3ad976708f2891` |
| baseline live evidence | `target/ngrx-legacy-declarations/20261012-154056-03231977/`: **50 PASS / 28 FAIL / 7 OBSERVED** |
| candidate live evidence | `target/ngrx-legacy-declarations/20261012-155527-76c82edf/`: **78 PASS / 0 FAIL / 7 OBSERVED** |
| publication preservation | `target/framework-declaration-publication/20261012-155528-1a5325e4/`: **57 PASS** |
| manifest equivalence | static and partition source/oracle manifests identical; `source-equivalence-and-partition.json` |
| deferred observations | identity/relationship facts equal before/after for all seven probes; no success reinterpretation |
| test/lint/build logs | copied into candidate evidence root, including invalid capture run and every preservation filter/count |

Artifact names use host timestamps, not client-local date/time claims. No model host was invoked; this is owned native MCP protocol/operator evidence, not proof of Claude/Codex channel consumption or UI rendering. No shared CBM frontend/daemon was started or terminated. Owned stdio sessions were closed.

Audit: the first NgRx loss is independently recognized declaration evidence, not use-site string matching. Class proof is structural and source-bound, canonical owner authority is reused, legacy/modern producers remain distinct, production supplies the evidence once, registration normalization is reused, and persistence versions reject stale class-unaware products. No frozen DI, template, CBM, edit, diff, .NET, acquisition/hydration or public occurrence implementation was changed. All active code files stay under 615 lines, strict UTF-8 without BOM.

An execution transport interruption partially applied the initial patch to the new producer and NgRx types. Actual file state was inspected, and only unapplied integration hunks were resumed; this was not counted as RED/GREEN. A test cfg typo was corrected before its run, with no compilation failure credited. The invalid export-wrapper capture assertion was corrected before any production changes, preserving exact source/public oracle and retaining its run. Final tested production behavior is green; later edits are documentation/header-comment alignment only.

Stop after this completed class-declaration sub-boundary. Do not close #140 or implement its next family, #141 or #142 without separate owner authorization. The work is committed locally; push was not requested.
