# Host-visible knowledge request schema

Agent: **Agent MaxHeadRoom**. B10 observed limitation, 2026-11-12.

## Actual evidence

Accepted B9 executable/profile remain pinned in all three smoke manifests.
The production tools/list capture exposes 15 root properties, requires
workspaceRoot and mode, and includes oneOf branches distinguishing search from
identity-based operations. Those branches themselves describe mode and the
identity requirement; shared properties are at the root. Conditional allOf
clauses constrain mode-specific fields. See B10_HOST_SCHEMA_COMPARISON.json and
target/pcg-b10/before/1.response.json.

Smoke 02: one recorded host-mediated call sent unsupported query rather than
text and received -32602 pcg/invalid_request. The host can route an invocation.
That malformed request does not establish that a valid request would fail.

Smoke 03: after generic contract instructions named the correct fields, the
agent reported its visible schema as optional search mode or optional
get/related/context/source mode, with shared fields absent. It stopped rather
than bypassing that visible interface. The captured event stream has no MCP
knowledge call. Its exit code 0 means the reporting turn completed, not that
knowledge acquisition succeeded. The two agent reports agree about missing
shared fields, but the wrapper does not capture a raw host-to-model schema.

Thus advertisement is established for this isolated CLI and successful knowledge
use is blocked in the observed agent workflow. Exact source provenance,
currentness and closure remain uninspected through that host. B9 stdio success
remains separate evidence. HTTP transport errors in stderr are not attributed
to the local stdio server without a corresponding trace.

## Supported hypothesis and proposed correction boundary

The reported visible mode alternatives mirror the production oneOf branches.
This supports a composition/projection compatibility hypothesis: shared root
properties may not survive host schema conversion. It does not prove which
host component performs the conversion or exclude model misinterpretation.
Capture the actual host-normalized/model-visible schema in a separately
authorized compatibility investigation before attributing an implementation bug.

The smallest proposed mission is a knowledge-tool request-schema compatibility
repair, if that capture confirms shared-field loss. Preserve all existing
request names, modes, validation/refusal rules and production query behavior.
Evaluate a composition shape supported by the actual host, with explicit common
fields, while retaining mode validation in the existing runtime. Alternatively
correct the host adapter if it is controlled and the intended schema is valid.
Do not change representation, FTS, corpus, traversal or introduce a new tool
taxonomy merely to work around this boundary. A normal-instructions-only remedy
has already been attempted and did not unblock the observed session.

Any descriptor shape change is externally visible and explicitly excluded by
B10's schema/production-host restrictions; this report does not authorize it.
The source-owned production descriptor is primary evidence. The exact schema
and request parser must both govern a repair, not the agent report alone.

Required verification for a future authorized repair: tracked schema/registration
regressions, preserved invalid-request behavior, production tools/list parity,
then one consolidated real-host run covering all five modes and correct returned
references. Keep that reachability evidence distinct from natural discovery and
independent decision-quality evaluation. Do not weaken validation or bypass host
controls. No further identical smoke retry is justified before the boundary changes.
