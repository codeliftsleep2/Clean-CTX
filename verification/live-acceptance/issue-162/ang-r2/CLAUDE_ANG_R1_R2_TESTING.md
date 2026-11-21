# Claude field testing: ANG-R1 and ANG-R2

Agent: **Agent MaxHeadRoom**. Prepared 2026-11-19.
Status: **FIELD VERIFICATION BLOCKED BY REPORTED QUERY FAILURE**. This is an operator/Claude test instruction,
not a Rust regression, CI result, owner authorization or capability certification.

## Blocking field report (2026-11-19)

The operator reports that workspace_query with type=resolver_registrations,
direction=forward and a nonexistent route_path does not return and grows memory,
including a workspace with no resolvers. The operator separately reports healthy
hydration. Suspend repeated registration queries until the defect is resolved.
This is a field failure report, not a verified allocation leak or infinite-loop
diagnosis. Exact workspace scale, logs and memory measurements are not yet retained
in this repository; do not invent them or relabel the earlier synthetic PASS.

Static trace identifies query-specific amplification: the handler calls
hydrate_resolver_owners before using the selector; that operation inventories all
eligible TypeScript owners and requests full High semantic preparation. The shared
publication batch prepares all jobs and retains the compiler's complete result
vector before committing. Each compilation independently captures TypeScript
authority with a fresh Resolver/Host. Project resolution can acquire program roots,
units and witnesses for that capture. The absence of the selected route does not
bypass any of this work. This establishes the implementation path and a scaling
risk, not measured field causality or proof of a leak.

The earlier growth witness varied registration records (2/16/64) at fixed owner
count. It did not establish bounded work/memory while the number of unrelated
TypeScript owners or project inputs grows. Correctness CI and 185 synthetic stdio
checks therefore do not discharge this newly reported field failure. Required
repair verification must include cold no-resolver and absent-route requests,
controlled unrelated-owner/project growth, termination, allocation/retention
observations and preservation of real positive/alias/sibling/currentness cases.
Retain lower-bound semantics and distinguish acquisition failure from an empty
answer. A selector-text filter, arbitrary timeout/cap or early empty response is
not authorized as a substitute for correct discovery and bounded acquisition.

## Purpose and boundaries

Use one Claude session in a real Angular workspace to check the Angular baseline
reconciled in ANG-R1 and the public written resolver registrations added in ANG-R2.
ANG-R1 was architectural baseline/reconciliation, not a claim that every Angular
capability was newly implemented or verified. ANG-R2 adds public registration keys;
it does not establish that Angular can execute a resolver successfully.

Keep three questions separate throughout:

1. **Written membership:** which exact declarations are referenced by supported
   resolve mappings of this exact native route occurrence? Existing `ResolvedBy`
   relations answer this question, deduplicated per route occurrence/target.
2. **Written registration:** which exact key maps to which declaration, at which
   route/source occurrence? `resolver_registrations` answers this question.
3. **Executable eligibility:** can Angular invoke it, with which injector/provider
   scope and navigation result? Neither of the above answers this question.

No implementation changes, eligibility work, broad suite, performance campaign,
authority amendment, PR, merge or publication is part of this field session.
Do not reopen accepted P0 batching or use elapsed time alone to reject semantics.

## Operator setup: complete once

- Connect Claude to the already verified Clean-CTX executable:
  `C:\Users\MNasty\Desktop\RustContextLayerAI\target\debug\clean-ctx.exe`.
- Verified SHA-256:
  `f03052a27e2a916ddf52dfb0a18deb246b387c386c04584f8dc8d8115c3ff4cd`.
  Its build source revision is `aee244bfc514cd78cf085dd54739a963f4822541`.
  Delivery `a0340e8a21b0e98f3f7ba6d68cbd74ac7a0b7235` adds evidence only.
- Record the actual connected binary/configuration; a different identity is not
  verification of this pinned build. Do not request another rebuild by default.
- Select the actual absolute Angular workspace root and record its Git revision,
  dirty status, Angular package identity and applicable Clean-CTX configuration.
  Do not confuse the tool repository root above with the Angular workspace root.
- Use an isolated test session/cache/storage outside the Angular source tree.
  Preserve the user's existing durable state. Record whether the first query is
  cold; if isolation cannot be established, label it warm/unknown, not cold.
- Keep proprietary source and raw receipts on the originating work machine.
  Return a redacted summary if that is the workspace's disclosure requirement.

Earlier evidence: CI run 415 passed; current synthetic production stdio passed
185/185 checks in three sessions. These results reduce repeated local work but
are not this real-workspace field execution.

## Copy-paste task for Claude

Follow the instructions in this document. Use Clean-CTX as the primary
code-intelligence layer and follow [the Claude tool rules](../../../../docs/CLAUDE_INTEGRATION_RULES.md).
Use the actual absolute `workspaceRoot` explicitly. Do not invent filesystem paths
from namespaces or project slugs. Inspect the connected tool schemas; if a tool or
required argument is unavailable, preserve the error and report the prerequisite.
Use graph discovery when applicable, then `provide_code_context` for understanding.
If graph acquisition is unavailable, use the documented fallback and report it.
Exact file inspection supplies an independent expected oracle; it must not replace
the semantic operation being tested or turn an acquisition failure into absence.

Record raw tool arguments and complete responses locally, with a session ID,
timestamps and source/config identities. Inspect structured results when present;
do not treat truncated presentation text as the whole response.

### A. Establish the independent source oracle

Find an existing explicitly native `@angular/router` Routes typed const root with
ordinary resolve mappings. Inspect exact source and referenced declarations.
Before querying the target, record route root, each route-object occurrence, path,
written keys, selected declaration sources and expected unique target membership.
Use the ANG-R1 [capability matrix](../ang-r1/ANG_R1_CAPABILITY_MATRIX.csv) to identify
the actual producer envelope, unsupported applicability and authority boundaries.

Prefer existing real cases containing aliases or imported resolvers, two keys
sharing a resolver, repeated route paths, valid siblings and a nearby non-resolver
relation. Do not prewarm selected owners through semantic compression or membership
queries before the first registration query. Source reading is allowed.
If cases are absent, record NOT PRESENT; do not silently manufacture enterprise
coverage with synthetic fixtures. Any scratch additions require operator approval
and must be labeled separately from existing real-workspace observations.

### B. ANG-R2: first public registration and reverse lookup

Replace placeholders with actual source-derived values. Invoke `workspace_query`:

```json
{
  "workspaceRoot": "<absolute Angular workspace root>",
  "type": "resolver_registrations",
  "direction": "forward",
  "route_path": "<exact written route path>"
}
```

Then invoke the reverse query independently for a returned resolver declaration:

```json
{
  "workspaceRoot": "<same absolute Angular workspace root>",
  "type": "resolver_registrations",
  "direction": "reverse",
  "resolver_name": "<exact resolver declaration name>"
}
```

Do not send the opposite selector in either request. An empty string is a valid
written route path; it is not permission to omit the selector. Equal display paths
or resolver names may select multiple distinct occurrences/declarations.

Require format_version 1, matching direction, count equal to registrations length,
and source-backed `route`, `route_path`, `key`, `resolver`, `evidence` records.
Compare source/hash/span/root identity, not display names alone. Evidence spans
are byte offsets: decode the actual source bytes at each span. Escaped key tokens
need not byte-equal their decoded public key; compare using supported decoding.
Check the outer field, mapping object/pair/key/value joins and exact target source.
Distinct keys selecting one resolver must remain distinct records. Equal keys on
different route occurrences must not collide. Namesakes must not be merged.

Inspect semantic_omissions separately from candidate_source_omissions and their
attribution. Require honest lower-bound coverage: source_complete=false and
result_semantics=lower_bound. A successful empty collection is not proof that the
workspace contains no registrations. An acquisition/currentness failure is not an
empty semantic answer. Stop at the earliest blocking prerequisite and retain it.

### C. ANG-R1: baseline membership and preservation

After the first registration query, invoke the existing membership operation:

```json
{
  "workspaceRoot": "<same absolute Angular workspace root>",
  "type": "forward_edges",
  "domain": "angular",
  "entity_type": "Route",
  "name": "<same exact written route path>"
}
```

Compare `ResolvedBy` targets with the independent source oracle and registrations.
Two keys selecting one exact resolver contribute one unqualified membership per
route occurrence, while their key records remain separate. Preserve distinct
route occurrences and exact target identities; do not infer coordinates hidden
by the legacy endpoint display solely from a label. Report public evidence limits.

Check an applicable existing sibling, such as RouteMapsTo component membership,
against source. Record which ANG-R1 matrix families were actually exercised;
untouched provider, DI, module, guard and other Angular families remain UNRUN in
this session. Their existence in the matrix is not general Angular certification.

Where real sources contain unsupported/duplicate/type-only mappings, verify
refusal and preservation of independent valid siblings under the documented
envelope. Outside-native generic relations are not canonical resolver authority.
Do not require unsupported expressions to become executable resolver facts.

### D. One bounded edit/restart sequence, only with operator permission

Ask once before editing real workspace files or restarting the connected MCP
session. Preserve exact original bytes and existing dirty changes. Without that
permission, mark this section UNRUN and finish the read-only observations.

On an approved isolated copy or approved real source, change one existing key
without changing its resolver target. Repeat forward/reverse registration queries:
old key retires, new key appears, unqualified membership and valid siblings remain.
Remove that one mapping: its registration retires, but a second independent key
supporting the same target must preserve membership. If no second support exists,
record the narrower case rather than claiming this preservation was exercised.

If persistence is enabled and restart is approved, restart the same pinned binary
against the same recorded durable identity and repeat queries. Do not enable
persistence or change global configuration solely to obtain a PASS. Historical
restore evidence must not be labeled current after a source change; reacquisition
failure must not promote retained history into current truth.

Restore the exact original source bytes and requery before finishing. If cleanup
or recovery fails, report the remaining change explicitly. Never delete real
source owners merely to exercise retirement. Source deletion and acquisition
failure are different states; the existing tracked/stdio witnesses cover cases
that cannot safely be reproduced in this real workspace.

### E. Return the field receipt

Return a table with obligation, PASS/FAIL/UNRUN/NOT PRESENT, exact request IDs,
response evidence paths, observed identities and limitations. Cover first-query
state, forward/reverse registrations, distinct keys, repeated occurrences,
membership, source provenance/currentness, refusal/siblings, mutation, restart,
cleanup and ANG-R1 family scope. Explain missing coverage without inventing success.

Separate semantic mismatches from tool/schema/config/acquisition prerequisites.
Report elapsed observations descriptively; no timing campaign or enterprise
efficiency claim. Preserve raw errors and stop repeated equivalent retries.
No executable eligibility, injector/provider effectiveness, navigation result,
full-repository CI or all-Angular certification follows from this field receipt.

## Governing references and publication standing

[DOC-270](../../../../docs/agent/ISSUE_162_ROUTE_RESOLVER_ACCEPTANCE.md) covers this
instruction as a bounded mission asset. The source-owned resolver contract remains
[AUTH.ANGULAR_RESOLVER](../../../../docs/architecture/contracts/angular-resolver.md).
Historical ANG-R1 statements that public keys were open retain their date; ANG-R2
supplies the separately approved public key operation. This document creates no
new semantic contract or protected-authority authorization.

Prepared from the delivered sources and retained actual DOC-270/VERIFY-001
acquisition at restore-validator-admission-20261119-112204. Connected PCG tools
are unavailable in the authoring host; no fresh runtime acquisition is claimed.
Actual document admission and normal guards for this new instruction remain
required before branch publication. Its FIELD EXECUTION status stays UNRUN until
the Claude session returns attributable evidence.
