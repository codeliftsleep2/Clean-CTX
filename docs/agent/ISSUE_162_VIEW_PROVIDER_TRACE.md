# Issue #162: Component viewProviders boundary trace

Agent: Agent MaxHeadRoom
Status: both intended REDs observed and frozen; production carrier approval pending.

The operator supplied both failures. See the
[position carrier decision](ISSUE_162_VIEW_PROVIDER_POSITION_DECISION.md) for the
frozen test hash, observed acquisition loss, downstream collision analysis and
smallest recommended contract. Historical UNRUN statements below describe the
test-authoring checkpoint. Production change remains unauthorized.

## Investigation disposition and batched controls

The owner authorizes investigation/test authoring only. Production semantic
change is explicitly not authorized. Follow map -> batch RED -> observe ->
trace -> decide; preserve the exact observed tests before any approved repair.

Two tracked controls in angular_view_provider_position_issue_162.rs are authored,
UNRUN. Each requires an established ordinary registration and native Component
proof first. The view-only case requires a public owner/token fact. The same-token
both-arrays case requires a public answer distinguishable from providers alone,
without choosing a new relation name, field or number of edges in advance.
Failures include retained decorator-field diagnostics. Mere public inequality
will not by itself certify the future semantic position contract; interpretation,
refusal and lifecycle acceptance remain required after a carrier decision.

## Retention/index trace before observed RED

SemanticEdge currently carries relation, endpoints, layer and call-specific
evidence. DurableSemanticEdge transports those same fields with explicit
canonical endpoints. Neither has a provider-position field. WorkspaceIndex's
EdgeKey uses asserting file, typed relation, canonical endpoint keys and legacy
endpoint labels, plus call-specific arity/spread. It does not use provider-array
position. Consequently two identical owner/token/relation facts asserted by the
same file have the same current index key. This is a static deduction, not an
observed both-arrays result: current capture/production already omits the view
position before that hypothetical collision.

The controls query the existing workspace_query forward_edges surface after
normal compression/publication; they do not populate an index manually. Observe
their diagnostics before choosing a carrier or attributing a failure solely to
index deduplication. No production changes or runtime runs were performed.

## Concrete next question

Which tokens does an exact Component declare in viewProviders, and can public
answers distinguish these entries from its ordinary providers entries?

Angular 20's [Component API](https://v20.angular.dev/api/core/Component) lists
viewProviders and providers separately. This is documentary API evidence,
not an executed visibility oracle. AQ08 requires registration context; the
position distinction must survive before effective injector reasoning can use it.

## Located boundaries

module_metadata::DECLARATION_FIELDS requests providers and strategy fields,
but not viewProviders. string_token_authority::provider_array selects only
providers; provider_semantics' compatibility lookup does the same. The evaluator
already nominates Components. This is a metadata-position gap, not a new owner.

RegistersProvider's existing owner/token tuple does not itself distinguish two
positions when the same Component and token appear in both. Do not assume index
deduplication preserves a distinction absent from that fact. Simply selecting
both arrays could also misclassify them as competing ordinary arrays.

## Bounded sequence and decision point

First author a tracked public-query regression with nonempty ordinary providers
and exact native Component proof, then a viewProviders-only variant requiring
local canonical registration. Observe the omission before repair. That control
tests admission, not the eventual position distinction or visibility semantics.

After valid RED, audit existing carriers and consumers before recommending the
retained/public position contract. Compare a distinct typed relation, an existing
retained position plus an approved public context representation, and a weaker
undifferentiated local-registration fact. The last alone cannot answer the full
position question. Recommend preserving explicit position through shared provider
mechanics; carrier selection remains open and requires owner approval if it
changes semantic authority or public contracts.

Batch verification: both arrays on one owner; same token in both; same-named
distinct tokens; owner/API/position refusals; currentness; complete SQLite restore;
independent field removal/repair; public distinction; existing owner preservation.
Reuse existing lifecycle. Do not change global EntityRef equality or introduce
another resolver, registry or provider pipeline.

No child/content visibility, effective provider, hierarchy, precedence, winning
registration or consumer satisfaction follows. Those remain P3–P6 obligations.
No production edits, tests or runtime execution accompanied this trace.
