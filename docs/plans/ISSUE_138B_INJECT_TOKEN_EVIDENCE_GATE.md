# #138b structured inject token identity — evidence gate

> Historical blocked-run record. The prerequisite and resumed repair are now documented in [ISSUE_138B_STRUCTURED_INJECT_REPAIR.md](ISSUE_138B_STRUCTURED_INJECT_REPAIR.md). Statements below describe the preserved blocked draft and its run, not the current branch.

**#138b STRUCTURED INJECT TOKEN IDENTITY SLICE BLOCKED**

No production repair was made. The primary registered regression remains deliberately RED in the working tree; do not merge this draft as a green repair. No commit or push was performed.

## Current authority loss

**PROVEN BY CODE INSPECTION**:

1. `TS_CALL_QUERY` captures the invocation root, free callee identifier, ordered argument expressions, property declaration span and property name. The TypeScript parser knows their syntactic boundaries.
2. `collect_invocations` creates an `InvocationOccurrence` with callee, argument spans and nearest declaration context. That context identifies a field initializer, not its owning class. It contains no separately captured initializer-value span or expression kind.
3. `CoreExtractionPass` retains canonical `class.root` names and byte spans upstream. The #139 name authority is present and unchanged.
4. `MetaLayerPass::class_entries` reduces these captures to capture-kind/raw-source pairs. Their class byte spans and canonical owner names are not passed together to the Angular consumer. `structured_declarations` is currently supplied only for C#.
5. `MetaLayerContext.invocation_evidence` reaches Angular evaluation, but is used there for RxJS. `ProjectionEvidence` does not pass it into Angular class DI interpretation.
6. Angular DI currently uses `extract_graph_entries(raw_class)`. Its `collect_signal_fields` scans assignments textually, stores the receiving name in `SignalField.name`, then appends that name to the merged dependency vector. `class_to_semantic_edges` projects that string as a Service target.
7. For the primary spaced assignment, the backward scan stops at the space immediately before `=`, producing `?`. Compact assignment can instead yield the receiver name. Neither establishes dependency identity.
8. The rendered functional inject marker also uses the receiving name. A future repair must address both model-facing projections from authoritative token evidence, without changing signal input/output semantics.

The parser is **not** generally unable to capture arguments. The missing evidence is the complete, structured property-initializer/owning-class association at the DI interpretation boundary. This is a producer/propagation gap, not a request for a second parser.

```text
existing parser
  ├─ class.root: canonical owner + span (upstream)
  ├─ field declaration: receiver + declaration span
  └─ call: free callee + first argument span
              ↓
Angular boundary loses class span association;
no explicit initializer-value association is retained
              ↓
legacy assignment scanner selects receiving name or ?
              ↓
AuditConsumer ──Injects──> Service/?
```

## Primary RED: unchanged semantic oracle

```ts
import { Injectable, inject } from '@angular/core';
@Injectable()
export class AuditService {}
@Injectable()
export class AuditConsumer {
    private readonly completelyDifferentName = inject(AuditService);
}
```

Both fixtures add 100 unchanged padding comments solely to exercise compact production output. Receiver = `completelyDifferentName`; token = `AuditService`; consumer = `AuditConsumer`.

**PROVEN BY TEST**: the registered test first verifies that existing invocation evidence preserves the receiver and first argument separately. It verifies independently published Service occurrences for AuditConsumer and AuditService through `entities_in_file`, then checks the actual `forward_edges` response.

Actual RED: one `Injects` edge with subject angular/Service/AuditConsumer and object angular/Service/`?`. Expected target is exactly AuditService. Failure is the intended behavioral assertion, not compilation or fixture syntax failure.

**PROVEN BY LIVE MCP**: the existing b2b84bb binary reproduces the same wrong target through owned stdio sessions using the established `PhaseBSession.ps1` pattern, with CBM disabled in a fixture-local configuration and independent Git root. Declaration checks PASS; exact token identity FAILS. The harness terminates only its owned frontend.

## Evidence gate and smallest proposed next boundary

The handoff says:

> If structured owner association is missing for a supported form: STOP and report the owner-evidence gap. Do not build another owner parser.

It also requires stopping if current captures do not expose enough information to distinguish receiver from token. Here the token span itself exists; the owner/initializer qualification needed to admit it faithfully is incomplete at the consumer boundary.

**PROPOSAL**, not implemented:

- Carry existing canonical TypeScript class identity plus exact class span into compilation-scoped Angular evidence. Preserve #139 producer semantics.
- Retain a structured field initializer-value span, associated with its receiver and containing class. Establish whether the initializer itself is the supported free `inject` call rather than accepting any descendant call.
- Preserve the first argument expression and its existing truthful symbolic identity separately from the receiver. Do not resolve imports or runtime providers.
- Feed an Angular-local typed record into both existing functional DI marker and semantic edge projection. Preserve constructor producers and the frozen Pipe constructor-only admission.
- If expression-kind qualification is required, capture it from the existing parse/query rather than reconstruct it from text.

Existing argument capture alone is insufficient to distinguish direct initialization from `condition ? inject(AuditService) : otherValue`, because both expose a free call with field declaration context and no parent invocation. This observation is **PROVEN BY CODE INSPECTION** of the query and occurrence representation; it was not run as an acceptance trial. Class captures upstream provide a plausible solution, but a new propagated association is not an already-established Angular owner contract.

No regex, source scanning, marker parsing, backward owner search, additional parser, production type or semantic policy was introduced.

## Acceptance matrix status

| Syntax | RED | GREEN | Token oracle | MCP |
|---|---|---|---|---|
| unrelated receiver, readonly spaced assignment | target `?` | not run | AuditService | registered + live |
| matching-ish receiver | not run in this slice | not run | AuditService | not established |
| multiple dependencies | not run | not run | AuditService, LoggerService | not established |
| same token / multiple receivers | not run | not run | one semantic AuditService edge | not established |
| non-inject initializer | not run | not run | no Injects edge | not established |

No GREEN, completed matrix, deduplication, unresolved-target behavior or lifecycle acceptance is claimed. The task stops at the explicit evidence gate before repair. The full bounded matrix is required after that gate is resolved.

## Classified forms and limitations

- Primary direct property `inject(AuditService)`: argument/receiver capture **SUPPORTED** at generic evidence level; complete Angular interpretation is **BLOCKED**.
- Optional calls, qualified tokens, MY_TOKEN, call options: **NOT ESTABLISHED in this slice**. #138a historical observations remain historical evidence only.
- Same-named cross-file declarations: resolution **NOT ESTABLISHED**; no import or module resolution change.
- MissingService symbolic references: no new acceptance trial or declaration fabrication was introduced. Existing edge endpoints must not be mistaken for independent declarations.
- Constructor overrides, functional guards/resolvers, runtime injector semantics, signals and host/view/provider metadata remain outside this repair.
- #142 query-history independence is not tested or claimed.

## Preservation and incidental effects

No production changes or downstream improvements occurred. No preservation filters, Clippy or candidate build were run: those would not resolve the evidence gate. Frozen #138a, owner, declaration publication, NgRx, template, edit, diff, .NET and CBM implementations remain byte-for-byte unchanged. This is a change-scope statement, not a fresh test claim.

## Files prepared

- `src/tests/mcp/angular_di_inject_token.rs`: permanent-regression draft, deliberately mismatched receiver/token, structured evidence checks, independent public declarations, exact public edge identity. Currently fails; no completed CI coverage claim.
- `src/mcp/mod.rs`: only feature-gated test-module registration; no runtime behavior change.
- `verification/issue-reconciliation/Invoke-AngularInjectToken.ps1`: primary readiness/RED probe only, not the full acceptance harness. Uses existing session helpers and preserves exact requests/results, source and binary provenance. Intentionally exits 1 for the current defect.
- This document: gate, primary evidence, proposal and outstanding acceptance requirements.
- Verification README: reproducible primary-probe command and explicit incomplete status.

## Executed commands and artifacts

1. Focused rustfmt of the new Rust regression, edition 2024, `skip_children=true`: passed.
2. `cargo test --locked -p clean-ctx --lib --all-features -j 4 angular_di_inject_token_`: **0 passed / 1 failed**, exit 101. Registered primary RED, 3665 other tests filtered out. Toolchain environment used `/workspace/.tooling/cargo` and `/workspace/.tooling/rustup`.
3. PowerShell primary harness against `target/debug/clean-ctx`: **1 PASS / 1 FAIL**, exit 1. Existing binary SHA256 `104f63f4bbcf8456ee1d7418de43f340afba87626cd1f281e06d05eebfa7037b`; no build was performed.

Live evidence: `target/angular-inject-token/20261012-145418-89830ad9/`, including primary context/entities/edges, actual targets, source oracle, provenance, JSON-RPC frames, stderr and verdicts. The Cargo RED log is copied into that directory as `registered-red.log`.

No candidate binary, GREEN, live provider, full suite, CI gate, model-host trial, commit or push was run. Work remains an uncommitted blocked draft. Resume only after owner review of the structured evidence boundary; retain the exact primary regression unchanged for its eventual RED→GREEN procedure.
