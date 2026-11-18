# #138b structured inject evidence prerequisite and token repair

**#138b STRUCTURED INJECT EVIDENCE PREREQUISITE COMPLETE — TOKEN REPAIR RESUMED AND COMPLETE**

The supported direct free property initializer now preserves its dependency token through both Angular marker rendering and registered semantic-edge publication. The frozen primary test passed unchanged. Constructor extraction and #138a Pipe admission were not changed. Stop after this slice; #138 remains open for its other criteria.

## 1. Evidence-gate root cause

**PROVEN BY CODE INSPECTION**: generic invocation evidence retained free-callee, argument and field-declaration spans, but the field context did not establish that the call was the initializer itself. It also did not carry the containing canonical class into Angular DI. A free call inside a conditional, wrapper or deferred arrow could still have field-associated descendant evidence. Taking the first argument of such a descendant would introduce a false positive.

**PROVEN BY TEST / LIVE MCP**: the previous Angular extraction did not consume that generic argument evidence. Its assignment scanner placed the receiving member or placeholder in `SignalField.name`, and `extract_graph_entries` used that as the semantic target. The primary spaced assignment published `AuditConsumer ──Injects──> Service/?`. Compact assignment published `completelyDifferentName`, not AuditService. Multiple fields lost their separate token identities.

```text
field declaration
  ↓
generic descendant invocation evidence
  ↓
no direct-initializer / canonical-class association at Angular boundary
  ↓
legacy assignment scanner selected receiver or ?
  ↓
wrong Injects target and functional marker
```

The blocked-run report and original live evidence remain at `docs/plans/ISSUE_138B_INJECT_TOKEN_EVIDENCE_GATE.md` and `target/angular-inject-token/20261012-145418-89830ad9/`. That report is explicitly historical, not erased or silently rewritten as a successful repair.

## 2. Structured evidence repair

**PROVEN BY CODE INSPECTION / TEST**:

```text
existing TypeScript parse and capture walk
  ├─ #139 class.root: canonical name + exact declaration span
  ├─ same-match direct class-body field: receiver + declaration span
  ├─ field.value: exact initializer-expression span
  └─ existing invocation: free callee + ordered argument spans
                    ↓
Angular-local ClassInitializerEvidence / InjectInitializerEvidence
                    ↓
initializer span == invocation span
free identifier callee == inject
first argument is captured plain identifier
optional call excluded by AST capture
                    ↓
receiver remains separate; exact written token retained
                    ↓
same admitted token list → High functional marker + existing Injects mechanics
```

The supplemental IR-only query captures the field, receiver, value and owning `class_declaration`/`abstract_class_declaration` in one match. Its owner span is joined to the existing #139 `class.root` by exact start/end bytes. A direct class-body pattern avoids assigning a nested anonymous-class field to an outer owner.

The existing generic invocation types and collector are unchanged. The first argument remains source-bound parser evidence. Extra identifier-argument and optional-call captures qualify this local consumer without changing generic call semantics. A qualified expression remains captured verbatim in evidence, but is not normalized into a supported bare token.

No second parse, regex, source-line scanning, nearest owner, substring boundary recovery, rendered-marker parsing or generic expression framework was introduced. Comparing the text of a **captured identifier** with `inject`, and slicing a **captured argument span**, are interpretation of parser evidence, not source scanning. The existing class-source helper supplies unchanged decorator-bearing text only; it does not establish ownership or initializer boundaries.

## 3. Direct-versus-descendant evidence matrix

| Form | Expected classification | Actual | Evidence |
|---|---|---|---|
| direct `inject(AuditService)` | direct | DirectSupported | exact value/call spans + canonical class |
| unrelated receiver direct inject | direct | DirectSupported; receiver and token differ | permanent MCP regression + typed evidence |
| conditional descendant | not direct | NotDirect | initializer spans full conditional |
| wrapper descendant | not direct | NotDirect | initializer is wrapper invocation |
| non-inject initializer | not Angular inject | NotAngularInject | structured free callee is makeService |
| multiple fields | independently associated | first→AuditService; second→LoggerService | same-match fields and distinct argument spans |
| parenthesized inject | wrapper, outside direct rule | NotDirect | initializer includes parenthesized node |
| optional `inject?.(...)` | unsupported call | UnsupportedCall | anonymous optional-call token captured by query |
| qualified token | preserved but unsupported identity | UnsupportedToken; `namespace.AuditService` retained | exact first-argument span |
| member `obj.inject(...)` | not supported free call | NotAngularInject | WrittenCallee::Member |

Four focused evidence tests pass: exact captures, typed direct/descendant/token classification, multiple fields/canonical owners, and anonymous nested-class rejection. The initial exact-capture test was registered first and failed before implementation (**0/1 RED**). It was stashed with its registration, restored unchanged, and passed (**1/1 GREEN**) before semantic repair resumed. Subsequent formatting changed only presentation; the gate assertions were not weakened.

## 4. Owner authority and receiver/token separation

For the primary source:

```ts
@Injectable()
export class AuditService {}
@Injectable()
export class AuditConsumer {
    private readonly completelyDifferentName = inject(AuditService);
}
```

- Canonical consumer: **AuditConsumer**, from the frozen #139 class-name authority.
- Receiver: **completelyDifferentName**, from the property name capture.
- Dependency token: **AuditService**, from the first argument capture.
- Directness: initializer-value span equals invocation span.
- Public relationship: angular/Service/AuditConsumer **Injects** angular/Service/AuditService.

The receiver is never selected as a target. Both declarations are checked independently through `entities_in_file`; declaration existence is not used as an edge oracle. No owner heuristic or new declaration carrier was added.

## 5. Resume rule and unchanged primary RED → GREEN

All five Section 20 prerequisites were met: canonical owner, exact field value, directness qualification, separate argument/receiver roles, and no unrelated architectural change. Semantic repair then resumed within this authorized item.

The primary registered test file remains byte-for-byte unchanged from the blocked RED:

`23e665126bce91af43c73b4100ae6ab7338047968bb382d9ba9f542d5c448a41`

It was run RED before prerequisite changes, stashed with its registration while production changes were made, restored unchanged, and run GREEN with the same command. No fixture simplification or nonzero-count substitution occurred. Both stashes were restored explicitly; overlapping registrations prevented automatic stash application, so the exact registration diff and untracked test blob were restored from the stash. No regression was dropped.

## 6. Public RED/GREEN matrix

Rows below are **PROVEN BY LIVE MCP** against baseline/candidate binaries using identical saved source/oracle manifests. `[]` means no admitted Injects edge. Unsupported-control GREEN means correct non-admission, not newly supported syntax.

| Syntax / case | Baseline actual target(s) | Candidate actual target(s) | Token oracle | Boundary |
|---|---|---|---|---|
| primary | ['?'] | ['AuditService'] | ['AuditService'] | live MCP |
| matching | ['?'] | ['AuditService'] | ['AuditService'] | live MCP |
| compact | ['completelyDifferentName'] | ['AuditService'] | ['AuditService'] | live MCP |
| spacing | ['?'] | ['AuditService'] | ['AuditService'] | live MCP |
| multiline | ['?'] | ['AuditService'] | ['AuditService'] | live MCP |
| public | ['?'] | ['AuditService'] | ['AuditService'] | live MCP |
| protected | ['?'] | ['AuditService'] | ['AuditService'] | live MCP |
| multiple | ['?'] | ['AuditService', 'LoggerService'] | ['AuditService', 'LoggerService'] | live MCP |
| same-token | ['?'] | ['AuditService'] | ['AuditService'] | live MCP |
| non-inject | [] | [] | [] | live MCP |
| conditional | [] | [] | [] | live MCP |
| wrapper | [] | [] | [] | live MCP |
| member-call | [] | [] | [] | live MCP |
| parenthesized | [] | [] | [] | live MCP |
| optional | [] | [] | [] | live MCP |
| qualified | ['?'] | [] | [] | live MCP |
| missing | ['?'] | ['MissingService'] | ['MissingService'] | live MCP |
| symbol | ['?'] | ['MY_TOKEN'] | ['MY_TOKEN'] | live MCP |
| options | ['?'] | ['AuditService'] | ['AuditService'] | live MCP |
| component | ['AuditService', '?'] | ['AuditService', 'LoggerService'] | ['AuditService', 'LoggerService'] | live MCP |
| directive | ['AuditService', '?'] | ['AuditService', 'LoggerService'] | ['AuditService', 'LoggerService'] | live MCP |
| pipe | ['AuditService'] | ['AuditService'] | ['AuditService'] | live MCP |
| constructor | ['AuditService'] | ['AuditService'] | ['AuditService'] | live MCP |
| arrow-descendant | [] | [] | [] | live MCP |
| method-call | [] | [] | [] | live MCP |

The multiple/same-token cases prove exact relationship identity and count, not request success. Every positive edge verifies angular domain, consumer kind/name, Service symbolic target kind/name and asserting-source file provenance. Repeated queries do not multiply edges.

## 7. Supported and unsupported forms

**SUPPORTED** for this static token interpretation: direct free property calls with a captured bare identifier argument, including unrelated receiver names, private/public/protected/readonly modifiers, compact/spaced/multiline layouts, multiple fields and repeated tokens. A bare `MY_TOKEN` is a truthful written symbolic token, not InjectionToken/provider resolution. Additional options leave the first argument as the token; no runtime optionality is inferred.

**UNSUPPORTED / classified non-admission**: parenthesized wrappers, conditional/wrapper/arrow descendants, optional calls, member callees, qualified/member token expressions and spread arguments. Qualified and optional forms retain their written argument evidence where available; they are not reduced to an unqualified name or receiver. Parenthesized expression support would require a separately approved contract, not whitespace normalization.

**NOT ESTABLISHED**: imported binding identity, namespace resolution, same-named module disambiguation, runtime provider selection, injector success/failure, factory semantics and client model consumption. No field host/model trial was run.

Pipe functional DI remains outside the admitted path. Constructor-only Pipe behavior is preserved. Component/Service/Directive functional token projection uses the existing framework-kind admission.

## 8. Declaration separation and symbolic unresolved target

`inject(MissingService)` produces the existing symbolic Service endpoint. It does **not** produce a source declaration or self-Defines declaration carrier for MissingService. The tracked test inspects compiler semantic edges to reject fabricated Defines evidence. Independent Service publication is unchanged.

The public occurrence schema has no declaration/reference role discriminator: an edge endpoint can appear as a Service occurrence. Do not describe that as a fabricated source declaration or claim that public `entities_in_file` alone establishes declaration provenance. The same limitation was recorded in the frozen publication and #138a reports.

The cross-file control publishes two CollisionService source occurrences and a CollisionConsumer→CollisionService symbolic edge. The edge's object.file identifies the asserting occurrence, **not** the imported target file. No import resolution acceptance is claimed.

## 9. Deduplication, fidelity and lifecycle

**PROVEN BY TEST / LIVE MCP**: two receiving fields requesting AuditService produce one semantic edge and one functional token marker. Multiple different tokens remain separate. Repeated queries and source recompilation do not multiply relationships.

The live harness also publishes every static case at High, Medium and Low. Functional/constructor DI marker lines remain High-only; exact semantic edges remain reachable at the lower fidelities. Source bytes remain unchanged.

**PROVEN BY TEST / LIVE MCP**: explicit `refresh_workspace` replaces AuditService with LoggerService, retracts the old edge, removes all functional DI after replacing inject with makeService, and leaves no stale edge after source deletion. This establishes the tested refresh lifecycle only, **not #142 cold/warm or query-history independence**.

## 10. Production lifecycle and persistence

**PROVEN BY CODE INSPECTION / LIVE MCP**:

1. `CoreIRPass` runs the existing single TypeScript parse with supplemental local captures, applies unchanged #139 canonical names and constructs the initializer records.
2. `PassContext.angular_initializers` owns the records for that compilation. Private local captures are removed before canonical emission/projection. Generic invocation evidence is unchanged.
3. `MetaLayerPass` lends those records through `MetaLayerContext`; authoritative Some(empty) does not trigger a second parse or legacy token recovery.
4. `angular_semantic::evaluate` supplies the same admitted token records to marker rendering and semantic projection. Constructor syntax stays in its original producer; Pipe guard behavior is unchanged.
5. `IRCompiler` drains semantic edges. `compile_file_ir_candidate` and the existing context/hydration publication paths pass them to durable semantic ownership and `WorkspaceIndex::add_edges`.
6. Existing `workspace_query` exposes the edges in structuredContent and mirrored text; `provide_code_context` exposes the corrected High marker.
7. Refresh recompiles changed sources and existing file deletion retracts their evidence/edges. No new index, cache policy or hydration authority was introduced.

Initializer records remain private and compilation-scoped; they are not serialized in canonical IR, database rows or public schemas. Existing durable products do change, so the existing Angular marker generation advances **2→3**, semantic generation **4→5**. Canonical pipeline generation remains **1**. Compatibility tests verify prior producer identities differ and the existing durable generation-mismatch rejection remains green. No live old-database migration trial is claimed.

Legacy source-only helpers now omit functional DI when no authoritative initializer evidence is supplied. They still render/extract their supported decorator, signal and constructor facts; the default MCP IR path supplies the records. The obsolete receiver-based functional scanner was removed, rather than retained as a second truth path.

## 11. Preservation actually executed

The 180 preservation checks below passed. The four new evidence checks and seven new MCP/matrix/compatibility checks are reported separately: **11 focused new checks**, all green. These are selected checks, not a full repository suite or CI result.

| Executed filter | Passed |
|---|---|
| `ir::invocation_evidence::` | 11 |
| `angular_meta::rx::invocation_evidence_tests` | 7 |
| `angular_meta::constructor_injects::tests::` | 21 |
| `angular_meta::constructor_di_edge_tests` | 15 |
| `angular_meta::semantic_tests` | 18 |
| `angular_meta::decorators::tests` | 28 |
| `angular_meta::ngrx::tests::` | 25 |
| `angular_template_compatibility_tests` | 1 |
| `workspace_query_angular_selector_occurrence_tests` | 5 |
| `restore_rejects_semantic_producer_generation_mismatch` | 1 |
| `decorated_owner_identity_` | 9 |
| `framework_declaration_publication_` | 10 |
| `angular_di_pipe_` | 9 |
| `insert_before_declaration_` | 9 |
| `semantic_signature_evidence_` | 5 |
| `di_omission_` | 6 |

Package all-target/all-feature Clippy passed with `-D warnings`; one all-feature binary build passed. No broad template, NgRx or other issue acceptance is inferred beyond the named filters. No full suite was run.

## 12. Remaining #138 work and incidental effects

Remaining criteria include broader functional expressions, qualified-token identity, optional-call/parenthesized support, constructor token overrides, import/module ambiguity, signals and host/view/provider metadata, and runtime provider resolution. None was implemented here. #140 legacy NgRx, #141 linked-HTML exposure, and #142 hydration/query history remain separate.

Incidental downstream improvements: corrected token markers agree with semantic edges; multiple unrelated receiving fields no longer collapse to a placeholder; qualified unsupported expressions no longer publish receiver/placeholder targets; stale prior marker/edge products are rejected by existing producer generations. These are consequences of #138b, not acceptance for another issue.

## 13. Changed files

Production:

- `src/angular_meta/inject_evidence.rs`: new Angular-local typed associations, directness/token classification and admitted-token view.
- `src/ir/pipeline/core/typescript.rs`: narrow IR-only field/value/owner, identifier argument and optional-call captures; unchanged canonical owner implementation; evidence regression registration.
- `src/ir/pipeline/core.rs`: collect associations from the existing walk; remove private captures from canonical processing.
- `src/ir/pipeline.rs`: compilation-local owner for records.
- `src/ir/pipeline/meta_layer.rs` and `src/layers/meta/context.rs`: borrow records at the framework boundary, no reparsing.
- `src/angular_meta/decorators.rs`: remove receiver-based functional scanning; render only supplied functional tokens. Signal input/output/model branches and constructor extraction are unchanged.
- `src/angular_meta/mod.rs`: supply evidence-backed classes/tokens to existing marker orchestration.
- `src/layers/meta/angular_semantic.rs`: consume canonical owner and qualified functional tokens through the existing edge projection.
- `src/layers/meta/mod.rs`: existing marker/semantic generation increments.
- `src/angular_meta/semantic.rs`: **comment only**, describing the preserved #138a constructor-only guard; no behavior change.

Tests and registration:

- `src/tests/ir/angular_inject_evidence_capture.rs`: permanent capture-boundary RED/GREEN.
- `src/tests/ir/angular_inject_evidence.rs`: typed classifications, exact spans, class/field associations and anonymous-class negative.
- `src/tests/mcp/angular_di_inject_token.rs`: preserved original exact registered primary regression, unchanged hash.
- `src/tests/mcp/angular_di_inject_token_matrix.rs`: public forms/negatives, symbolic declaration separation, deduplication and lifecycle.
- `src/tests/mcp/angular_di_inject_token_compatibility.rs`: prior marker/edge identity rejection.
- `src/mcp/mod.rs` and `src/mcp/compatibility/mod.rs`: test registrations only.
- `src/tests/mcp/angular_template_compatibility.rs` and `src/tests/mcp/workspace_query_angular_selector_occurrences.rs`: expected existing generation values aligned; template behavior/assertions unchanged.

Operator assets/documentation:

- `verification/issue-reconciliation/Invoke-AngularInjectToken.ps1`: expanded established stdio pattern, 25 static cases×5 groups plus 2 symbolic cross-file groups and 4 lifecycle groups = **131**.
- `verification/issue-reconciliation/README.md`: reproducible command and scope.
- Historical evidence-gate report: historical-status banner linking this report, original blocked evidence preserved.
- Frozen #138a and post-CBM reconciliation reports: narrow follow-up links, preserving historical findings and scope.
- This report: prerequisite, resumed repair, commands, evidence and scope limits.

## 14. Commands and artifacts

All Cargo commands used the existing workspace toolchain:

```sh
export PATH=/workspace/.tooling/cargo/bin:$PATH
export RUSTUP_HOME=/workspace/.tooling/rustup
export CARGO_HOME=/workspace/.tooling/cargo
```

Executed:

- `cargo test --locked -p clean-ctx --lib --all-features -j 4 angular_inject_evidence_`: 0/1 RED; 1/1 GREEN after exact restore; 4/4 evidence matrix and final preservation run.
- Same command with `angular_di_inject_token_`: 0/1 primary RED; 1/1 unchanged GREEN; 6/6 expanded matrix; **7/7 final** including compatibility and strengthened marker assertions.
- Same command with each preservation filter in Section 11: all pass, 180 total. No zero-test filter is credited.
- Focused-file rustfmt, edition 2024, `skip_children=true`: passed. The original primary test file was not reformatted.
- `cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings`: exit 0, no warnings.
- `cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4`: exit 0.
- Installed PowerShell, `-NoProfile -File verification/issue-reconciliation/Invoke-AngularInjectToken.ps1 -BinaryPath <binary>`: baseline exit 1, candidate exit 0.
- Strict UTF-8/BOM, active file sizes and focused Git whitespace checks: passed.

No runtime was left running. Owned fixtures live only under target; no shared CBM daemon was started or stopped.

| Artifact | Identity / result |
|---|---|
| baseline binary | `target/debug/clean-ctx-inject-token-baseline`, SHA256 `104f63f4bbcf8456ee1d7418de43f340afba87626cd1f281e06d05eebfa7037b` |
| candidate binary | `target/debug/clean-ctx`, SHA256 `649ebffad9cb908710fdaeea72d832f75e3fad3267e21b57e0c0d76d03d20fe3` |
| expanded baseline evidence | `target/angular-inject-token/20261012-152201-2cf3ea2d/`: **67 PASS / 64 FAIL** |
| expanded candidate evidence | `target/angular-inject-token/20261012-152445-267ab97a/`: **131 PASS / 0 FAIL** |
| source equivalence | parsed static source/oracle manifests identical; saved `source-oracle-equivalence.json` |
| tracked test/Clippy/build logs | copied into candidate evidence root; preservation results include exact commands/filters and counts |

Artifact folder names are execution-host timestamps, not client-local time claims.

## 15. Audit notes and stop

No production change was made before the initial prerequisite/public RED. The original primary hash is unchanged. Typed source roles are separate, directness is structural, captures are private, production receives evidence on its default path, and existing generation enforcement prevents stale results from being treated as current. All new/materially changed code files are under 615 lines and strict UTF-8 without BOM. No production test assertions or unrelated caps were removed.

Intermediate evidence-only builds produced unused-field/method warnings before consumption was integrated; final test/Clippy/build runs are warning-free. An automatic stash restore refused an overlapping helper edit; explicit exact restoration preserved the regressions. A scratch preservation driver incorrectly treated `10 passed` as `0 passed`, then had a corrected-loop syntax typo; no test failed or was skipped as a result. The driver resumed at the first unexecuted filter, and every listed preservation result is an actual completed Cargo run. A test-registration patch initially missed its context and made no changes; it was reapplied with the correct context. None is counted as behavioral RED or GREEN.

The original frozen implementation slices remain behaviorally preserved by the executed controls. No public schema, constructor extraction, Pipe admission, generic invocation contract, CBM, discovery/hydration policy, templates, NgRx, edit, diff or .NET implementation was changed. Producer-generation fixture alignment and the Pipe comment are explicit bookkeeping, not reopened repairs.

Stop after #138b. Do not close #138 or begin another production slice without owner authorization. Publication/push was not requested; the completed work is committed locally for review.
