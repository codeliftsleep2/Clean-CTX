# Consolidated host and production verification

Agent: **Agent MaxHeadRoom**. B10 Addendum 01 checkpoint, 2026-11-12.

The user-run schema-production-host-01 batch completed production checks and one
actual host turn. Production: 13 ready/current/complete requests; generation
80c1195a7a9df253e4221635bd3feb8201d15ac4e0fbc328425e59cbe04b03e5;
218 identities with all original revisions; 221 exact typed relationships.
The twelve paired serialized MCP results are byte-for-byte identical to the
original B10 before-capture. The thirteenth request enumerates all identities.
Malformed query remains -32602 pcg/invalid_request; incorrect source hash remains
-32603 pcg/source_revision_changed. No request alias or validation weakening.

Actual host events contain six completed calls: search, get, related, context,
source and a second context call after display truncation. The agent reports all
15 fields visible and required workspaceRoot/mode correctly exposed. Its chosen
query was pipeline AND ordering; it acquired AUTH.CANONICAL_IR from discovery,
then reached related/context/source. The captured payloads report ready/current/
complete with no diagnostics. Context is eight units/seven edges, related has
eight incident edges, and source selects the original canonical-authority artifact.
This establishes practical schema compatibility in this host session, without
proving the precise internal cause of the earlier composition failure.

## Capture-fidelity limitation

Independent static comparison of saved CLI event component strings found Unicode
differences: e.g. original code points 167,167,49,8211 became
9516,186,9516,186,49,915,199,244. Source-owned and direct production bytes remain
unchanged. No original evidence is repaired or reinterpreted. The host agent's
separate response file contains correct Unicode, but that alone does not establish
exact contents of every tool result. The old event trace cannot support a byte-exact
host-content claim. Its successful invocation evidence remains preserved.

The wrapper previously piped the npm PowerShell shim's native output through
PowerShell strings. It now invokes that same installed npm JavaScript entry via
the same Node selection and copies native stdout/stderr streams to files without
text decoding. Security, model, prompt, profile, tool and executable are unchanged.
This is a capture-only correction. It introduces no production behavior or budgets.
One user-owned capture-fidelity rerun is pending; no rebuild is needed.

## Raw capture verification completed

The user-run host-utf8-01 capture exited 0. Static inspection of actual completed
MCP events verifies six successful calls covering all five modes. This agent
selected PIPELINE-001 from its own search, rather than the previous session's
AUTH.CANONICAL_IR selection. Its get follows the actual returned acquisition
reference; all four exact components match the baseline role/label/path, byte
intervals, hashes and text, including Unicode. Its context is the accepted B9
one-unit closure with zero edges and a zero-step witness. This is complete
declared closure, not proof of decision-wide authority sufficiency.

Related retrieves the incoming canonical-authority preservation edge, then the
agent follows the discovered authority with another related call. Both complete
incident edge sets and directions match the baseline. Source follows the exact
returned hash-pinned source reference. The UTF-8 bytes of its complete original
artifact hash to the returned pin and current original file. The saved raw event
text is no longer corrupted. Each call is ready/current/complete at the accepted
generation with no diagnostics. B10_HOST_INVOCATION_VERIFICATION.csv records each
actual event; B10_PRODUCTION_RESPONSE_PARITY.csv records all twelve exact pairs.

The static inspector initially assumed the earlier context starting identity.
That inspection assumption was corrected to select the accepted B9 closure oracle
for the actually requested identity. No response, agent output or accepted B9
oracle changed, and no additional host run was required for that correction.

The descriptor compatibility correction now has scoped tracked GREEN, production
parity and actual-host exact-content evidence. The precise original host-internal
schema conversion mechanism remains unproven. The supported operational conclusion
is compatibility for the exercised tool/modes in this isolated Codex CLI session.
Normal user/project configuration remains unchanged and the current interactive
host is not claimed to have been redeployed. General adoption and independent
decision quality remain unestablished. No frozen B7 result is replaced by smoke
output, and all sixteen independent runs remain UNRUN.

## Delivery-boundary clarification

The byte-exact comparisons above verify the captured MCP event payload against
source. They do not prove all captured bytes were delivered in the model's
effective context or inspected by the agent. See B10_DELIVERY_BOUNDARY_INVESTIGATION.md
for the owner's requested production/capture/delivery/model/inspection separation.
Actual five-mode invocation remains established; universal end-to-end completeness
is not inferred from that operational success.
