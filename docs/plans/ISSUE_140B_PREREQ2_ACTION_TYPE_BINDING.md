# #140b-Prereq2 structured Action type-identifier binding

**#140b-PREREQ2 STRUCTURED LEGACY ACTION TYPE-IDENTIFIER BINDING COMPLETE**

Base `142b75c`, branch `feature/current-cbm-pinned-upgrade`.
Complete only for the conservative compilation-local same-file contract below.
No reducer linkage, public discriminant entity or TypeScript module resolver.

## 36.1 Proven source form

```ts
import { Action } from '@ngrx/store';
export const SET_AUDIT = '[Audit] Set Audit';
export class AuditAction implements Action {
  readonly type = SET_AUDIT;
}
```

The new handoff explicitly authorizes investigating this candidate relationship.
It is not retroactively claimed as an older legacy producer contract. Its exact
structural oracle succeeds independently of the new producer: the declarations
are program children, the field belongs to the class body, and the value node is
an identifier. #140a independently admits AuditAction before any new evidence.

Supported target: a same-file, program-level initialized `const` with a plain
identifier name (exported or not). Supported source: an already admitted top-level
class/abstract class with exactly one ordinary `type` member, an instance
`readonly` field whose value is the identifier itself. No type checking or runtime
value evaluation is claimed. Declaration order is irrelevant to lexical identity;
this does not prove safe runtime instance construction before initialization.

## 36.2 First-loss trace

| Stage | Before | After | Evidence |
| --- | --- | --- | --- |
| #140a class qualification/canonical class identity | PRESENT | Unchanged | Current code and registered primary oracle |
| Exact class-owned type field and initializer identifier roles in NgRx query | MISSING | PRESENT | Unchanged structural regression RED→GREEN |
| Target top-level const declaration/name spans | Generic variable context exists; NgRx roles MISSING | PRESENT | Same-parse domain-local capture query |
| Unique, supported same-file lexical association | NOT ESTABLISHED | PRESENT within bounded refusal contract | Registered producer matrix |
| NgRx-local typed association | MISSING | PRESENT in PassContext | CoreIRPass collects private records |
| Public/durable discriminant entity | UNSUPPORTED | Still not introduced | Producer shape tests and live preservation harness |

Classification: PROVEN BY CODE INSPECTION and PROVEN BY TEST for the structural
and private association rows. Live MCP proves only public preservation. No public
binding visibility or model consumption is claimed.

## 36.3 Action authority

`ngrx::legacy::collect` remains the sole class-Action admission producer. Its
existing imports/implements/owner logic is unchanged. The binder receives its
admitted `ClassActionEvidence` and joins the captured owner by exact class span.
No type field, string, filename, suffix or use site can admit an Action.

## 36.4 Initializer structural authority

New supplemental patterns appended to the existing NgRx query capture the same
class body's field, ordinary property name, field span and a direct `(identifier)`
value in one query match. `readonly` is a literal grammar requirement. The
collector validates the member name `type`, exact admitted owner and singleton
member identity; it refuses static fields and unsupported member authorities.

Wrapper, conditional, logical, template, parenthesized and qualified expressions
cannot satisfy the direct identifier node pattern. No descendant identifier is
promoted to the initializer. Private query roles are stripped before canonical
IR emission. The existing parse is reused; deterministic tests assert one parse
per CoreIRPass invocation. No raw-source assignment scanning or second parser.

## 36.5 Identifier binding authority

This is a deliberately restricted resolver, not first/last/nearest-name lookup:

1. The source owner is the program-level Action admitted by #140a, so no enclosing
   function or block scope can supply its initializer binding.
2. The target must be a captured initialized program-level const declaration.
3. All supported declaration-binding captures for the written identifier are
   deduplicated by exact name span. There must be exactly one such binding and
   that span must be the target program-level const name span.
4. Class self names, class type parameters, variables, function names/parameters,
   bare arrow parameters, imports/aliases/namespaces, enum names and catch bindings
   are captured as conflict/refusal evidence. Another same-name binding anywhere
   in the file refuses association, even in an unrelated nested scope.
5. Unsupported destructuring/namespace patterns refuse the whole file; escaped
   binding identifier spellings refuse the file rather than being normalized.
   Computed/string/number member names or constructor parameter properties refuse
   the owning class; duplicate type members and static fields also refuse.
6. Captured ERROR nodes refuse association. This is not a general validation of
   every malformed TypeScript program or a compiler/type-checking claim.

The eligible lookup combines structured lexical scope, unique binding spans and
admitted owner identity. Text equality selects an identifier spelling inside that
proved scope; it is never a standalone identity oracle. Conservative false
negatives are intentional and disclosed. Imports, aliases and namespace resolution
are UNSUPPORTED; no cross-file scope is claimed. Missing canonical file identity
refuses all binding records. Unsupported syntactic scopes do not affect #140a
independent declaration publication.

## 36.6 Identity preservation

`ActionTypeBinding` retains canonical file identity and source content hash,
Action canonical name/class span, field/reference spans, and target declaration
name/declaration/name spans. The Action and target have distinct spans and remain
distinct declarations. Identical source in different canonical files produces
different binding records. Equal values do not enter the binding algorithm.

Two Actions referencing the same constant produce two separate source-bound
records. This does not establish an inverse unique Action identity for the future
`case SET_AUDIT` consumer. That future consumer must handle ambiguity explicitly.

## 36.7 Value independence

The producer never compares or evaluates constant values. SET_AUDIT and
OTHER_AUDIT can have equal literals; only the direct identifier selects the target
name/declaration span. A class with an equal literal initializer has no binding.
Binding to a declaration does not evaluate its value or prove a string type.
No constant folding, normalization or class/constant entity merging.

## 36.8–36.9 Regression and evidence matrix

Primary unchanged registered file:
`src/tests/angular_meta/ngrx_action_type_binding_red.rs`.
Independent AST/class-admission assertions pass on the parent; the final role
assertion fails with zero direct type-reference captures. That is the valid
**structural evidence RED**, not a public MCP or typed-record semantic RED.
The same file is restored byte-for-byte and passes after the query change.
Separate permanent producer tests verify the assembled association.

| Case | Result | Exact oracle |
| --- | --- | --- |
| Primary readonly direct reference | RED 0/1 → GREEN | Exact owner, field, reference and const-name spans |
| SET_AUDIT vs OTHER_AUDIT | GREEN | Chosen declaration span only |
| Equal-value neighboring constants | GREEN | Values irrelevant to selected binding |
| Equal literal initializer | GREEN negative | Class survives; no binding |
| Wrapper/conditional/logical/template/parenthesized/member expression | GREEN negatives | No binding despite descendant token |
| Unrelated exported constant | GREEN negative | No new Action or unrelated association |
| Nested variable/parameter/bare-arrow/catch/import collisions | GREEN refusals | No first/nearest declaration selection |
| Destructuring/namespace/escaped names | GREEN refusals | Unsupported authority is not guessed |
| Class self/type-parameter shadow | GREEN refusal | No outer-constant misbinding |
| Nested Action class | GREEN refusal | Frozen #140a admission excludes it |
| Static/mutable/duplicate/computed member/parameter property | GREEN refusals | No unsupported member binding |
| Same source in different files | GREEN | Canonical file distinguishes records |
| Two admitted Actions sharing a const | GREEN | Separate Action spans, same const span |
| Export/nonexport, multiple declarators, abstract class, type import, Unicode name, later declaration | GREEN | Exact bounded declaration identity |

Final filter selects **7 tests**, all pass. Matrix rows are subcases inside tracked
tests, not an inflated Rust test count. An earlier compilation attempt used the
wrong content_hash argument type and was corrected; it is not RED evidence.

## 36.10 Public/durable semantics and lifecycle trace

Association is **private compilation evidence only**.

Production path: `IRCompiler::compile_inner` supplies canonical path and constructs
PassContext → default-production CoreIRPass performs its existing capture walk →
unchanged #140a admission → new NgRx-local binder → PassContext owns the vector →
remaining passes preserve existing public class products → compiler returns
CompiledIR and drops the private context. No association is persisted or returned.

There is deliberately no reducer consumer yet. The separately authorized resumed
slice must consume the association during compilation before its owner is dropped;
this prerequisite does not claim external reachability for the private record.
Cache hits returning existing durable IR do not promise fresh private evidence.
No cache behavior was changed or private association cached.

Existing meta-layer class publication remains unchanged. The new PowerShell
harness verifies exact class name/file at three fidelities, no duplicate constant
Action, no new public relation, and unchanged source bytes. It cannot inspect the
private binding. The full NgRx declaration preservation filter also passes.

## 36.11 Lifecycle

Deterministic fresh-compilation tests establish direct SET_AUDIT binding, change
to OTHER_AUDIT with a changed source hash and only the new association, change to
literal with no binding, remove class with no binding, and remove constant with
no binding. Canonical-path absence refuses evidence. No durable association exists
to retain stale state. No #142 or query-history independence claim is made.

## 36.12 Compatibility/generation

No durable marker/semantic product changes: generations remain marker **4**,
semantic **6**, canonical unchanged. No new public kind/relation, no discriminant
publication and no new cache compatibility claim; no bump is warranted.

## 36.13 Preservation

All executed preservation filters are listed below; **125 tests pass**:

| Filter | Passed |
| --- | ---: |
| ngrx_legacy_declaration_ | 9 |
| framework_declaration_publication_ | 10 |
| decorated_owner_identity_ | 9 |
| angular_di_pipe_ | 9 |
| angular_di_inject_token_ | 7 |
| angular_inject_evidence_ | 4 |
| angular_meta::ngrx::tests:: | 25 |
| angular_meta::semantic_tests:: | 18 |
| angular_meta::decorators::tests:: | 28 |
| angular_template_compatibility_tests | 1 |
| workspace_query_angular_selector_occurrence_tests | 5 |

These ran during the audit; final seven-case matrix and final Clippy/build follow
all binding-only refusal additions. No frozen #136/#137/#135 code was changed;
their tests were not rerun and no fresh result is claimed. No full suite or CI.

## 36.14 Remaining #140 work

Switch/case structured roles, legacy reducer extraction and resumed TriggersReducer
remain owner-reviewed followups. Standalone discriminant publication, constructor
dispatch, import/alias/namespace/barrel/module binding, service dispatch/select,
Selector/Effect criteria, representative recall and cold/warm behavior remain
separate. Original standalone-constant legacy switch criteria are not satisfied
merely by supporting this explicitly class-associated candidate.

## 36.15 Changed files

- `ngrx/legacy.rs`: append private structural/refusal queries and register unchanged
  RED test. Existing class admission/include logic unchanged.
- `ngrx/type_binding.rs`: source-bound private record and conservative collector;
  future-consumer fields intentionally retained with documented dead-code allowance.
- `ngrx.rs`: local module registration only.
- `ir/pipeline.rs`: per-compilation vector ownership/default.
- `ir/pipeline/core.rs`: invoke binder using admitted class/canonical identity;
  strip private roles before canonical emission.
- `src/tests/angular_meta/ngrx_action_type_binding_red.rs`: primary source-role
  regression and independent AST oracle, unchanged through RED/GREEN.
- `src/tests/angular_meta/ngrx_action_type_binding.rs`: typed association matrix,
  refusal, file/hash and fresh-compilation lifecycle assertions.
- `verification/issue-reconciliation/Invoke-NgrxActionTypeBindingPreservation.ps1`:
  established PhaseBSession MCP driver, nine forms × three fidelities (27 groups).
- `verification/issue-reconciliation/README.md`: operator command and observability
  limits. This report records current contract and evidence; the two preceding
  blocked gate reports are preserved as historical investigation artifacts.

## 36.16 Commands and artifacts

Fresh authorization received for this work item's exact focused tests, relevant
preservation, fmt, package Clippy, one binary build and provider-disabled harness.
Rust environment: PATH includes `/workspace/.tooling/cargo/bin`, RUSTUP_HOME is
`/workspace/.tooling/rustup`, CARGO_HOME is `/workspace/.tooling/cargo`.

```bash
cargo test --locked -p clean-ctx --lib --all-features -j 4 ngrx_action_type_binding_
cargo test --locked -p clean-ctx --lib --all-features -j 4 <each preservation filter above>
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4
```

Focused rustfmt with `--edition 2024 --config skip_children=true` ran only on
modified Rust implementation/matrix files. The unchanged RED file was not
reformatted after its observed RED. Stash restoration initially refused overlap
on legacy.rs; the exact registration/test was restored explicitly and verified
against the stash blob, then the owned stash was dropped.

Final matrix: **7 passed, 0 failed**. Preservation: **125 passed, 0 failed**.
Final package Clippy: exit **0**, warnings denied. Single all-feature binary build:
exit **0**. Provider-disabled live MCP preservation harness: **27/27 PASS**.

Artifact root: `target/ngrx-action-type-binding/20261012-164426-260a93eb`. It contains raw protocol/context/query captures, binary
provenance/source oracles/verdicts, and copied RED/GREEN/matrix, all eleven
preservation logs/results, final Clippy/build and harness logs.

Candidate binary SHA256: `b478d90114a72746ceae6691bdbf21559ac84e1920e41d4512b35bd94e3d703f`.
Unchanged primary regression SHA256: `5d5dfa82ca37cb761b44d3ca20893f6210e815f920e5b488923657e707185bd8`.

PowerShell command executed:

```bash
XDG_CACHE_HOME=/workspace/.tooling/powershell-cache XDG_CONFIG_HOME=/workspace/.tooling/powershell-config XDG_DATA_HOME=/workspace/.tooling/powershell-data /workspace/.tooling/powershell/opt/microsoft/powershell/7/pwsh -NoProfile -File verification/issue-reconciliation/Invoke-NgrxActionTypeBindingPreservation.ps1 -BinaryPath /workspace/Clean-CTX/target/debug/clean-ctx
```

Strict UTF-8/no BOM, active-file ceiling and git diff whitespace checks pass.
Owned MCP sessions were closed by the existing helper; no owned stash remains.
No CBM runtime, model-host pilot, full suite or unrelated harness was run.

STOP after this binding contract. Resuming #140b reducer linkage requires owner
review and authorization.
