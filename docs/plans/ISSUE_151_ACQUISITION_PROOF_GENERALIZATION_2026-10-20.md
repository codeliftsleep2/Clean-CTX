# Issue #151: acquisition proof generalization

Agent: MaxHeadRoom
Status: focused Rust suites user-reported GREEN; full gate and live verification pending.

## Verdict and root cause

PROVEN BY SOURCE: `find_entities` previously used ordinary declaration-mode
discovery with `CleanAuthoritative` hydration. The shared proof infrastructure
was instantiated only for incoming builtin TypeRef relationships. A discovery
cache hit could suppress inventory production without supplying a compatible
acquisition proof. This is a missing certified acquisition capability, rather
than a missing rendering of a compatible proof already in the response.

The maintainer approved generalizing the existing acquisition architecture for
declaration mode. Ordinary occurrence search retains its broader, uncertified
producer obligation and reports a lower bound.

## 151-A scope contract

PROVEN BY SOURCE: `PreparationContext::scope` validates the authorized root set
and optional `withinPath`; occurrence lookup applies this scope. Hydration
receives the workspace root and configured additional roots independently of
the answer narrowing. The schema now describes answer projection by asserting
file. Acquisition behavior is unchanged by `withinPath`.

The tracked Issue #151 scope regression checks the returned primary-root
declaration while separately checking index-owned declarations acquired from
both authorized roots. It proves projection using facts, rather than telemetry.

## 151-B public meaning and authority

Ordinary mode returns matching authoritative indexed entity occurrences after
registered hydration. These may include declarations, relationship endpoints,
and framework roles. Its coverage remains a lower bound.

Declaration mode returns matching source-proven declaration occurrences. An
explicit workspace root enables the shared `SourceDeclarations` acquisition
capability. Exactness describes supported declaration producers within the
authorized source admission policy. Domain/type and `withinPath` filters
project that acquired declaration universe.

Source completeness requires complete inventory of every authorized root,
coverage of every admitted language/producer unit, compatible generations,
validated exhaustive claims, and current High semantic publication of every
selected owner. No selective declaration-name predicate is certified, so the
capability processes the entire admitted unit. This may cost more than ordinary
name discovery; no new performance budget or truncation policy is introduced.

Declaration fact authority alone does not prove exhaustive search. Providers
remain advisory. Discovery counts, provider status, cache presence, and query
order never authorize exactness. Missing roots, processing/publication failures,
or missing producer generations prevent certification.

## Shared production lifecycle

`workspace_query` dispatch -> `prepare_find_entities` -> request-local keyed
hydration -> typed `AcquisitionTarget` -> existing hydration flight -> existing
acquisition completion cache -> Clean inventory -> capability coverage units ->
shared full-unit proof claim -> shared proof validator -> candidate compilation
and publication -> inventory/source/projection currentness check ->
`AcquisitionSatisfaction` -> shared completion cache -> capability-checked
`CompletenessCertificate` -> structured coverage and the existing content
envelope.

Ordinary cached discovery does not satisfy the proof obligation. When no
compatible satisfaction exists and inventories are absent or cover fewer roots,
the acquisition path obtains the required root inventories. Warm requests reuse
the same proof cache; refresh/edit/root invalidation retains the existing cache
lifecycle. There is no declaration-only cache or parallel satisfaction model.

The declaration capability binds generation identity to native canonical and
semantic generations plus the active producer catalog, including framework
declaration producers. Typed capability identity prevents declaration and
incoming-TypeRef requirements or certificates from being interchanged.

## Regression evidence and preservation

PROVEN BY TEST (user-reported RED only):
`find_entities_declarations_require_proof_after_ordinary_discovery_and_on_warm_reuse`
failed on the intended `source_complete` assertion: false versus true. The exact
regression was stashed before implementation and restored afterward unchanged.
The user subsequently reported GREEN for the focused Issue #151 contract,
declaration acquisition, shared acquisition, and reverse-edge completeness
suites after correcting the new test module's registration boundary. This is
user-reported Rust test evidence; the agent did not execute those commands.

Additional authored tests cover full-unit acquisition of a source lacking the
query spelling, publication failure/retry, proof invalidation, missing roots,
and capability/requirement separation. The focused suites are user-reported GREEN.

The existing exact incoming TypeRef query path, provider-neutral proof
validation, and publication/cache ownership remain the preservation authorities.
C# resolution campaigns and NgRx producers were not edited. Their runtime
preservation is NOT VERIFIED until the relevant suites and final gate run.

## Verification and disposition

Fast file-scoped formatting and `git diff --check` ran. Compilation, tests,
Clippy, full formatting gate, encoding guard, file-size scripts, and live
verification were NOT RUN by the agent. Follow `docs/agent/verification.md` for
the final gate. A Claude pilot remains the last field gate, after local evidence
has been collected and any remaining field questions can be batched.

Issue #151 remains open pending GREEN, preservation testing, the final gate, and
any applicable field acceptance. No commit or issue closure is claimed.

## Consolidated operator acceptance

`verification/issue-reconciliation/Invoke-Issue151Acceptance.ps1` is now the
tracked stdio acceptance definition. Its 15 cases cover three provider modes,
fresh and ordinary-discovery-first histories, warm reuse, answer scope,
mixed-language declarations, batch/channel parity, edit/delete refresh, missing
root recovery, and unreadable-source recovery. It reuses `PhaseBSession.ps1`.
PowerShell parsing passed without executing the harness. Live results remain
NOT RUN. The invocation and evidence rules are documented in the verification
directory README; #144/#148 preservation harnesses use the same binary.
