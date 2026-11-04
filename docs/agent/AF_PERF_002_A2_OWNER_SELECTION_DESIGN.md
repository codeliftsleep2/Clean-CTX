# AF-PERF-002 A2: owner-scoped class selection design return

Agent: Agent MaxHeadRoom
Status: source-traced proposal only; no A2 production change or runtime executed.
Generation78 accepted semantics remain the reference. A3 focused/composed GREEN
is separate evidence and does not certify this proposal.

## Measured problem and production boundary

The dirty public-query fixture records six consumers each visiting F*C classes.
F2/4/8/16 at C32 gives64/128/256/512 visits per consumer; unchanged warm
queries do no projection/reconciliation. This establishes traversal amplification
on dirty reconciliation, not its wall-time fraction or a universal defect in
complete class resolution. Class resolution must still compose partials and
resolve ancestry/attributes across the supplied authority.

Production callers inspected:

- src/mcp/csharp_calls/reconciliation.rs constructs current artifacts, merged
  declaration authority and partition-resolved classes once, then loops artifacts.
  Each artifact projects five edge families and independently projects IR/context.
  It stages projections before replacing index state and committing coverage.
- src/mcp/semantic_compilation/dependencies.rs separately resolves the current
  artifact plus partition peers, projects edges and IR, and checks publication
  stamp/dependency currentness before returning. This is a distinct operation.
- src/mcp/semantic_compilation/artifact.rs calls the six role consumers in
  src/dotnet_meta/semantic.rs. Execution-context projection clones the existing
  CompiledIR then retires/rebuilds realtime instructions; changing that cloning
  boundary belongs to A1, not this selection proposal.

No generation-local selection may cross these two operation boundaries, survive
publication, replace source authority or bypass their currentness/stamp checks.

## Important proof distinction discovered

The six consumers do not share one owner predicate:

| Consumer | Current owner/role selection |
| --- | --- |
| DbContext | Resolved known-external ancestry; any exact class.files owner; raw_parts filtered separately |
| AutoMapper | Resolved profile ancestry; any class.files owner; raw_parts filtered separately |
| SignalR edges | Low-fidelity early return; resolved hub then any class.files owner; per-owner raw parts and method-name suppression |
| ASP.NET | Resolved controller then any class.files owner; role registration only first class.files owner; payload attribution remains per part |
| MSTest | First class.files owner only, then resolved known-external test attribute; testing feature gate outside projector |
| SignalR contexts | Resolved hub, then method_ids owner; does not require matching class.files membership |

A class.files-only filter for all six would silently narrow execution-context
selection. A first-owner filter for all six would lose partial-owner payloads.
Neither is acceptable. Reordering pure role tests can change measured visits,
but may not alter supported/refused identities, duplicates or result order.

## Smallest candidate slice A2a: one owner-demand selection per artifact operation

Propose two explicit borrowed ordered views derived in one traversal of the exact
resolved slice: classes whose class.files contains the exact file, and classes
whose method_ids contains that exact file. Retain all class occurrences and their
original order; never deduplicate by owner/name or rebuild class contents.

Pass the first view to the five edge consumers, retaining all existing role,
first-owner, raw-part, fidelity and testing gates. Pass the method_ids view to
execution-context projection, retaining its resolved-hub check and exact method
owner filter. The original immutable resolved slice remains authority. No new
path normalization, string equality, partial-class composition or method identity.

Share these views only across edges/IR belonging to one named artifact projection
operation. Existing standalone entry points must create fresh local selection;
reconciliation and dependency publication must explicitly own each selection for
their edge+IR pair. Do not store it in artifacts or state. No generic context,
trait registry, currentness receipt or interior mutability is required.

Target work per owner: one C class sweep plus exact file/method-owner membership
checks, then five local class lists and one method-owner list. Across owners,
FC remains. This reduces repeated global role selection but is not a repair of
all F*C amplification. Memory is borrowed O(local class references), transient.
File-vector/method-owner traversal costs must be counted, not hidden in selection.
Feature/fidelity unreachable paths should not request unnecessary view work.

This proposal is deliberately weaker than a generation-wide owner map. Its
benefit must justify API plumbing and extra references before implementation.

## Alternative A2b: operation-local owner buckets at existing coordinator

A single borrowed owner->ordered-class bucket build at reconciliation would
replace repeated FC owner-selection with C plus total class-owner memberships,
followed by owner-local consumer work. Context buckets must be built from method_ids
owners independently of class.files. Duplicates within owner vectors must preserve
current any semantics: one class occurrence per matched owner, while separate
class occurrences remain separate. Public ordering must follow resolved slice,
never hash-map iteration. Feature/fidelity obligations remain explicit.

This is a broader cross-owner projection input design, not approved by A2
characterization. It must also define a fresh separate construction in dependency
publication and preserve all existing stamp/currentness boundaries. It must not
be implemented as a generation cache. Recommend evaluating A2a first unless
measured selection work shows retaining FC is inadequate; owner decides tradeoff.

## Required pre-repair controls and acceptance

Before selecting either representation, characterize positive role and partial
classes with exact per-owner output oracles, first-owner registration, method_ids
ownership independent from class.files where structurally constructible, duplicate
class records, missing/stale authority, testing off and Low fidelity. Verify exact
edge and IR order/provenance, not merely counts. No new eligibility rule may be
introduced to make inconsistent authority convenient.

Then author a real production-operation budget RED against existing paths. Count
selection sweeps, file and method-owner visits, per-family class/role visits,
raw-part/method joins, list capacities and complete enclosing public-query totals.
Use independent F/C and partial-owner/method density axes. Preserve unchanged warm
controls and the A3 exact identity oracle. Compare construction-inclusive timings
without treating visits as equal CPU costs.

Permanent enforcement must cover both reconciliation and dependency publication
callers, classify delegated selection, and demonstrate isolated real full-catalog
regression RED/restored GREEN. Existing AF fitness tests remain mandatory.
No new arbitrary timing threshold, cache, lock protocol, retry, reconciliation
policy or field run follows from this design.

## Decision requested

Approve characterization of positive/partial/owner distinctions first; then choose
A2a's narrower per-artifact selection or request the stronger coordinator-local
A2b design. Neither production implementation is authorized by this document.
A1 publication copying, A4 dependency footprints, declaration membership and
systematic discovery remain independent obligations. Syntax completeness is
unchanged:9763 original records preserved,13 scoped outcomes,9750 pending plus
96 callback supplements; this source tracing is production coverage progress,
not census disposition progress.


## Owner-approved characterization: first ownership packet authored, UNRUN

Owner authorizes characterization only; neither A2a nor A2b is implemented.
Actual selector predicates/order/returns remain unchanged. New cfg(test) statements
inside existing short-circuit predicates count class-file membership, raw-part
joins, ASP.NET method joins, context method-owner matches and emitted method IDs.
The unchanged known-external vocabulary function moved to semantic/known_types.rs
solely to preserve the active-file ceiling after instrumentation.

New tracked artifact-path test compiles legitimate partial sources for all five
edge families and the context consumer, resolves shared authority using the real
resolver, then invokes SemanticCompilationArtifact edge and IR generation methods.
It varies partial owners1/2/4/8/16, tests High/Low, requires exact ordered edges
including layer/domain/files and context IDs, checks first-owner registrations,
raw-part owner order, and SignalR suppression of same-name overloads. Per-part
method names differ across owners, avoiding illegal duplicate partial members.
A second captured-artifact test explicitly disables testing and requires the
TestClass branch unreachable while an exact Hub fact survives.

The existing public F/C fixture is rerun with additional join observations and
unchanged exact-call oracle/warm repeats. Three existing public lifecycle controls
cover cross-consumer changed ancestry, missing owner, partial TestClass registration
and alias/attribute retirement/repair. They remain distinct evidence: existing set-
based lifecycle oracles do not replace the new ordered artifact-path oracle.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-a2-ownership.ps1
```

Expected six tracked executions (two new ownership tests, one public F/C test,
three existing lifecycle tests), then focused all-feature lint/format/size/encoding.
This is characterization GREEN if fixtures/oracles pass, not pre-repair budget RED.
No agent test/build/formatter/lint/runtime execution occurred. Script parsing and
edited-byte UTF-8/line-count review are the only local checks.

### Explicit gaps and legitimate-authority limits

This is a first packet, not the complete owner-requested return. It measures
partial-owner density and observed method-owner records together; independently
varying method-owner density, eligible role population and method count still needs
its own controlled fixtures. No hypothetical temporary view is built; view capacity
and coordinator build costs remain analytical, not runtime measurements. Original
classes/edge/IR observations include no new cache or reusable validation state.

resolve_compilation_units builds files from symbol.parts and builds method_ids
by finding those same part/file/span inputs (compilation.rs). Thus method_ids owner
outside class.files is not produced by these legitimate captured inputs. Tests
will check the exact observed vectors rather than mutating authority to invent that
combination. This source relation does not authorize replacing the context consumer's
actual predicate. Repeated legitimate partial inputs are composed into one symbol;
malformed duplicate resolved records must not be reported as public-path authority.
Separate duplicate/ambiguity refusal and unresolved ancestry matrix coverage remains
required before representation approval. No absence claim follows from this packet.

A2a estimate: one class sweep per artifact, both ownership-vector checks and borrowed
reference allocation, plus original consumer-local work; FC outer work remains.
A2b estimate: one coordinator sweep plus total class/file and method-owner entries,
per-owner reference buckets and existing consumer-local joins; ordering/dedup of
owner membership is additional work. These are analytical formulas, not measured
post-repair budgets or approval for either design. Return actual observations first.
