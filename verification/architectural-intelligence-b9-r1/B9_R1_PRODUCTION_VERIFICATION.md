# Production verification status

## Latest executable refresh

Agent MaxHeadRoom inspected the operator's
`target/pcg-b9-r1/delivery-refresh-20261113-134605-318` receipt. Document tests:
39 passed; registered knowledge MCP tests: 10 passed; all-feature production
build: exit zero. This is focused verification, not the full CI gate.

Executable SHA-256:
`a555a0ad5374b7bc1b4db4a391ea0e389158880ac7d6c4c210c81f8f708266e0`.
Source HEAD: `df4d0414e0b48f78eba876e0338f00edbf1c1e58` on
`(fix)Performance-Regression-Issues`. The identity receipt explicitly lists the
uncommitted verification/report changes; it does not imply a clean-tree build.
Profile SHA-256 remains
`0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9`.

The refreshed actual-root verification passes: 218 identities, 221 relationships,
326 components, all eleven exact context oracles, graph responses without source
text, and 326 explicit component references with 430 checked component/qualification
occurrences. Individual acquisition, ordered duplicate/refusal behavior, exact
historical component and whole-source bytes, and stale-current-reference refusal
all pass. The same three authorized governing/workflow revisions remain the only
baseline advances. Generation remains
`684f1ca847aaec8398506ca3e8d0ebd3bf8f5bbaacb26ba88ede062fb17c6d70`.

The earlier actual-host receipt below verifies the prior executable. A single
bounded host refresh against this exact new executable remains outstanding;
do not infer current host integration from production stdio success. B10 probes
remain preserved and are not repeated. The external CBM timeout and CI status
remain separate obligations; full gates are CI-only.

Host refresh completed: `target/pcg-b9-r1/host-refresh-20261113-142039` has exit
zero and manifest executable hash matching the latest executable above. Static
capture verification passed on seven actual invocations: search/context/get/
related, single acquisition, three-reference acquisition, and full source.
Single acquisition checked two component occurrences including its qualification;
batch acquisition checked five including required qualifications. References
match prior graph returns, and exact source/interval hashes and bytes match the
current artifacts. Every operation is current and complete at the recorded graph
generation. The agent inspected saved events using the static verifier only;
the operator owned host/runtime execution. Context display truncation was reported,
and no model-visible completeness/comprehension guarantee is inferred. This is a
bounded host integration witness, not independent certification or a passing CI gate.

Agent: **Agent MaxHeadRoom**. Actual-root production verification **PASS** for
the recorded production-01 scope; actual-host workflow and final delivery pending.

Update: actual host-01 now also verifies the representative corrected workflow.
Seven actual completed calls cover all six modes, including individual and batch
acquisition. Raw captured graph payloads carry descriptors rather than text;
acquired refs match actual prior returned descriptors, qualification membership
is retained and captured exact text/source hashes match authoritative bytes.
See host-01/verified-summary.json. Host delivery and agent comprehension are not
inferred from event capture; reported display truncation remains disclosed.

Current B9-R1 executable built successfully in user verification-02, SHA-256
d91702b1c2a5b8e875b2c937e54b9ed4d5ad6fcbc304d79c9001967aff78fa38.
The B10 baseline executable remains separately preserved. Profile remains
0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9.
HEAD remains accepted B9 commit with the declared B10/B9-R1 working-tree changes.

production-01 completes 232 graph requests: PassPipeline/all discovery, all 218
exact structural gets, eleven contexts and typed incident relationships. The
static checker recursively rejects text/body/byte payloads in graph responses.
PassPipeline retains the accepted four direct matches; no shared-source sibling
matching is introduced. Canonical context retains eight nodes/seven edges/eight
witnesses; all eleven accepted closures match their exact baseline oracles.

Seven explicit requests then verify 326 selected component refs, exact requested
and required qualification bytes (430 occurrences), individual acquisition,
ordered duplicates/invalid refs with truthful incomplete state, exact old revision
components, stale current-ref refusal, explicit full current source and exact old
whole authoring artifact. No graph operation implicitly acquired that content.
Source hashes/intervals/provenance and currentness were compared against the actual
source or retained baseline, as appropriate. See verification-summary.json for
the exact three authorized revision advances and new generation.

Production tools/list capture is graph/1.response.json and the original serialized
responses/requests remain in graph/ and acquisition/. All driver processes exited
0. Expected per-reference refusals are successful negative witnesses, not silent
omissions. No host/model delivery claim follows from this production result.

After user-owned focused tests/lint/build, record new binary/source identities,
actual-root graph reconciliation and production tools/list. Exercise the four
reference-first graph modes, individual/batch component acquisition, explicit full
source and historical acquisition. Compare canonical authority membership, all
edges/witnesses/component roles/qualification/evidence references, PassPipeline's
four direct matches/excluded shared-source siblings, representative replay/EF/
resolver/performance knowledge and exact source bytes. Assert no source text in
serialized graph responses. Verify malformed/stale refs and ordered batch failures.

Then one bounded representative host graph-to-acquisition workflow may establish
actual-host reachability for this correction. Previous B10 content-bearing captures
remain historical evidence, not B9-R1 verification. Do not restart large-response
diagnostics or infer model delivery/inspection/comprehension from graph completion.
Full repository gate and stacked commit/PR delivery remain outstanding.
