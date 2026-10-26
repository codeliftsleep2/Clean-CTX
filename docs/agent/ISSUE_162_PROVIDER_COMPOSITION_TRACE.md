# Issue #162: provider-array composition trace

Agent: Agent MaxHeadRoom
Status: P1.C1 six intended REDs observed/frozen; bounded owner decision pending.

The operator supplied all six intended failures. The
[composition decision](ISSUE_162_PROVIDER_COMPOSITION_DECISION.md) records exact
freeze evidence, observed retained structures, downstream interpretation loss
and the proposed bounded source-declared path contract. Historical UNRUN text
below describes the pre-execution checkpoint. Production semantics are unchanged.

## Question and evidence boundary

Which exact tokens does a proved Component declare through an array reference,
array spread or nested array in providers or viewProviders, while preserving
the declared position and asserting Component identity?

Angular 20's [Provider source](https://github.com/angular/angular/blob/20.0.0/packages/core/src/di/interface/provider.ts)
includes arrays among admitted provider forms. This supplies documentary shape
context, not an executed runtime oracle or a new authority policy. A local const
binding does not by itself prove immutable runtime array contents.

The previous P1.VP1 checkpoint verifies direct position distinction. It does not
authorize general initializer evaluation, composition closure, mutation analysis,
arbitrary provider factories or a new resolver. No production semantic change is
authorized by this investigation step.

## Located existing mechanisms

Native declaration_facets::Demand captures Array, Spread and Reference shapes.
Demand::reference follows resolved demanded declarations when expansion is
requested. Demand::declaration retains an initializer only for const declarators.
Whether each actual provider fixture captures its required initializer is a
test observation, not assumed from these functions' existence.

string_token_authority::provider_array_at selects the exact metadata position
and owner. provider_semantics::project_at then requires the facet to correspond
to a syntax array at its own span, and iterates immediate children. Pair authority,
view_token and provider_field similarly select immediate entries/fields from
the retained array. A reference-valued container, spread child or nested array
therefore crosses several entry-selection assumptions, not just a query filter.

module_metadata::canonical::collect already walks native arrays, spreads and
resolved initializer references with an active-declaration cycle set. Its leaf
policy is module composition, and its BTreeSet erases order/duplicates. Do not
copy that policy into provider interpretation or assume it proves provider entry
association. canonical::object follows demanded const object references under
its own contract. The older sources::Contributors path reads textual module
inputs; it must not become an alternate TypeScript provider resolver.

## Batched controls

angular_provider_composition_issue_162.rs contains six independently runnable
tests: reference/spread/nested forms, each in ordinary and view positions.
Every test first requires two established direct-array registrations and exact
native Component proof. The composed variant requires the same two distinct
same-named token origins; repeated registration/query controls remain. Failures
include retained field shapes and captured initializer names, helping distinguish
capture omission from downstream interpretation loss. No behavioral RED is yet
claimed, and no test has been frozen as observed evidence.

After operator results, freeze the unchanged valid RED batch and trace actual
captured paths before selecting a bounded composition contract. Reconcile
asserting owner versus contributor origin, exact entry/reference/span proof,
cycles/missing/mutable/unsupported forms, position separation and dependency
currentness. Existing source-owned durable/session/index lifecycle must remain.

Any production extension needs its actual authority decision after that trace.
No effective injector hierarchy, visibility, ordering/precedence, multi behavior,
winning provider or consumer satisfaction follows from local declared membership.
P1.C1, P1, W04 and Issue #162 remain open. Operator owns runtime execution;
only static reads, formatting and source checks accompanied test authorship.
