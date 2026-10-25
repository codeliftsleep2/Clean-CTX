# Issue #162: native reference admission for value providers

Agent: Agent MaxHeadRoom
Status: bounded W04 V1–V4 reference-admission slice verified; generation-59 deterministic GREEN and corrected binary PASS; W04 remains open.

This proposal is V1 of W04 in the
[remaining-work checklist](ISSUE_162_REMAINING_WORK.md). Completing it would
close a bounded registration-admission slice, not the provider package or #162.

## Approved V2–V4 checkpoint

Corrected V4 binary report
target/live-acceptance/issue-162/2026-10-25T19-21-16-341Z-16680/report.json records
PASS: 1660 checks, 0 failed, no errors and all three sessions FINISHED. Binary
SHA-256 aa2a64437fc940fd01f620ada9228018bbdb65745d6d302a10a53e3f127ef685;
mtime 2026-10-25T19:16:48.871Z. The operator reports the V4 handoff GREEN,
including fmt/Clippy/syntax; separate lint/preflight receipts were not supplied.
No Rust change accompanied the harness correction/rerun. This verifies V4 for
the bounded slice and supersedes pending statements below; the corrected harness
expectation is not an unchanged production RED-to-GREEN claim.

V1–V4 is closed for native reference admission only. The remaining-work checklist
now reconciles W04's six open obligation groups before another implementation
slice. Canonical runtime-value identity, provider satisfaction, injector closure,
W04 closure and #162 completion are not established by this checkpoint.

The first V4 binary run (report 2026-10-25T19-16-51-067Z-1452) failed at
cold-warm-checkpoint/functional/1/value-reference/noncanonical. The harness
incorrectly required the legacy target file to stay absent after publication.
WorkspaceIndex::add_edges fills absent subject/object file metadata with the
asserting source. The returned two projections had exactly those source files;
their labels/counts and absence of exposed canonical fields passed earlier checks.

Original helper/report bytes and hashes are preserved under
target/issue-162/provider59-value-presentation-correction/. The harness now
checks exact asserting-source target metadata and private canonical fields
separately. Internal noncanonical status remains established by tracked Rust
controls, not inferred from public serialization alone. This is a harness
expectation correction, not production RED-to-GREEN evidence. No Rust or
generation change was made. Corrected V4 and all later assertions are UNRUN.

Operator provider59-*.log receipts record 73 passing test executions:
39 provider, 6 provider semantics, 12 string, 14 module and both compatibility
controls (1 each), with 0 failures/ignored and no reported warnings. The unchanged
valid RED is GREEN, including its later provenance/repeated-query assertions.
The batched refusal, durable/currentness and retirement controls also pass.
V2/V3 are verified for this bounded slice; V4 lint/rebuilt binary acceptance is
next. This supersedes their UNRUN checkpoints below. No production changes
accompanied this receipt; prior generation-58 binary results do not verify V4.

The owner disposition approves V1 as written and authorizes routine V2–V4 work
inside that contract, followed by W04 reconciliation before another slice.
No canonical runtime-value or ProviderUsesValue pair authority is authorized.

The corrected RED and isolated registration were frozen in stash
811a4c798deebdee6e77a372d61a7b7c830ef73b and restored byte-identically:
SHA-256 02465f2227b6616b04a93d6902498ca2c3677c3289205a6d7b26abeac7a0a98a.
Normal index and HEAD were untouched.

Generation 59 gives the provider parser its existing retained authority beside
the retained array. Reference-valued useValue positions select a unique exact
field and join its declaration/span with TypeScriptAuthority::value_reference_at.
Class/factory reference parsing shares that exact lookup. Missing or competing
proof cannot fall back to textual binding in the migrated reference branch.
An unproved provide field cannot publish a legacy registration for that branch.
Literal/no-authority compatibility paths retain their established behavior.
The strategy projection remains noncanonical, labelled by the proved declaration
name, with no evaluation or value-equality claim. Existing transport/currentness
and source publication owners remain; no new cache, resolver or lifecycle.

Batched V2/V3 controls are authored: the unchanged route RED; owner/type-only/
missing/ambiguous/duplicate/competing/call-expression refusals and repair; direct
missing/shifted native-reference proof refusal; complete canonical registration
and noncanonical presentation SQLite round-trip and restore before queries;
token/dependency mutation and independent replacement/deletion retirement; and
package export/config/version/ambiguous-target currentness with distinct provided
tokens so local registrations cannot hide a missing remote one. These controls
reuse existing fixture mechanisms and preserve class/factory/alias assertions.

The existing three-session binary helper has an independent reference-value
owner. It asserts canonical registration origins through existing token/function/
owner/package/restart mutations and separately labels the old strategy output as
noncanonical, including its same-label deduplication limitation. It does not
certify canonical runtime-value identity. All V2/V3 checks and the V4 binary/lint
run are UNRUN. No runtime execution was performed by the agent.

## Observed boundary

provider-value-reference-red-rerun.log records 0 passed, 1 failed, 0 ignored.
The corrected ordinary two-registration control passed. Aliased and namespace/
barrel class-value references then returned zero registrations instead of two.
Later provenance/repeated-query assertions remain unclassified. Exact test/log
bytes and hashes are preserved under
target/issue-162/provider-value-reference-red-evidence/.

The earlier run failed inside its control because that control incorrectly used
aliased imports with the legacy binder's allow_alias=false contract. Its bytes
remain under provider-value-control-correction; it is not the behavioral RED.

provider_semantics::parse_provider uses retained reference evidence for class/
factory strategies, but value_identity still uses identifier-only legacy
reference_identity. That helper refuses aliased imports and namespace references
even where native authority already resolves their exact runtime declarations.

## Recommended bounded decision

Authorize useValue registration admission for exact supported reference-valued
positions through existing retained provider-field evidence and native reference
resolution. Require exact Angular owner/API, providers container, unique provide/
useValue fields, source/hash/spans and resolved runtime reference occurrence.
Do not recover missing/type-only/ambiguous/unproved reference evidence from its
name or spelling. Preserve duplicate/competing-field refusal.

Use the provided token's existing canonical identity for RegistersProvider.
For the already-established noncanonical ProviderUsesValue strategy projection,
use the proved declaration name as its label where that reference form is
admitted. Do not turn that label into canonical strategy identity.

Reuse shared retained provider-position selection, exact native reference
accessor, resolver/Host, witnesses, source-owned artifacts, semantic generations,
compatibility, durable session/SQLite transport, publication and retirement.
No new registry, cache, acquisition owner, resolver or lifecycle. Advance existing
compatibility for changed producer/consumer meaning. Keep public schema, enum
values and global EntityRef equality unchanged. Preserve literal and existing
nonmigrated compatibility behavior within their recorded contracts.

## Deliberate limits

This decision repairs demonstrated registration admission only. It does not
authorize canonical runtime-value equality or a canonical ProviderUsesValue pair.
A referenced declaration identifies a reference endpoint; it does not by itself
prove the runtime value of an arbitrary initializer or alias chain.

No primitive-value identity policy, object/array literal identity, expression
evaluation, function invocation, class construction, arbitrary initializer
evaluation, provider satisfaction, multi, scope/precedence or injector closure
is proposed. Existing exact string-token proof remains its own admitted token
contract and does not become a general value evaluator.

Canonical value-strategy identity must be demonstrated separately through an
actual consumer/test before selecting its semantic carrier or value policy.
Class/factory/alias strategy approvals do not pre-authorize that decision.

## Alternatives and approval boundary

Recommended: consume native exact reference proof at the existing useValue
registration boundary and retain the explicitly noncanonical strategy
projection. This fixes the observed public omission without inventing runtime
value identity. Cost: bounded lookup/admission migration and preservation gates.

Rejected: widen the textual binder, trust terminal names, copy a namespace
parser, or apply class-construction/factory/alias strategy semantics to values.
Those duplicate weaker mechanics or misstate value meaning.

Deferral leaves the demonstrated alias/namespace registration gap open.
Immediate canonical value-strategy migration would exceed this RED and require
an additional explicit identity decision.

The prior owner approvals explicitly excluded useValue expansion. This change
extends Angular reference-position admission and public registration behavior;
AGENTS.md's Architectural Approval Gate therefore requires explicit approval.
No production repair has been made.

## Verification after approval

Freeze the unchanged valid RED with isolated registration before repair. Require
native alias/barrel/namespace value routes; exact own-token provenance; repeated
registration; type-only/unresolved/ambiguous/duplicate/competing/foreign-owner
refusals; dependency/config/package mutation and repair without consumer edits;
complete retained SQLite proof and restore before queries; stale refusal;
independent source replacement/deletion; class/factory/alias/string/module/member
preservation; zero-warning and rebuilt public-query/binary acceptance. Report
noncanonical strategy projection separately from canonical registration proof.
Operator owns Cargo/runtime runs; final Claude testing remains last. No broader
value-strategy certification or #162 completion claim follows.
