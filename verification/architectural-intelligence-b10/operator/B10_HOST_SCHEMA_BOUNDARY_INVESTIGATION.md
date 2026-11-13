# B10 Addendum 01: request-schema boundary

Agent: **Agent MaxHeadRoom**. 2026-11-12. Investigation/RED checkpoint.
The owner explicitly authorizes descriptor compatibility repair within B10;
the earlier B10 production-descriptor exclusion is superseded only in that scope.

## Preserved evidence and controlling implementation

The original tools/list response, comparison JSON, three host smoke manifests,
actual event streams and accepted B9 executable/profile remain untouched.
Smoke 02 preserves the unsupported query-field rejection. Smoke 03 has no
knowledge calls and reports only the two optional mode alternatives.
Neither is a successful five-mode retrieval. Existing production stdio remains
valid interface evidence, not host adoption.

The complete descriptor is knowledge_descriptor in
src/mcp/tool_handlers/pcg_documents.rs. Its 15-field inventory is workspaceRoot,
mode, id, text, track, kind, status, role, label, edgeKind, direction, path,
sourceSha256, maxNodes and maxEdges. Root required fields are workspaceRoot/mode;
additionalProperties is false. Root properties are shared by oneOf mode branches,
with allOf conditional restrictions. The original enum-valued properties omit an
explicit type but constrain values with enums. This is not evidence of invalid
JSON Schema: omitted type does not invalidate a string-only enum.

The Rust handler rejects non-object/null-valued arguments, then deserializes
knowledge::Request using deny_unknown_fields and camelCase. Required common
fields are non-optional Rust fields. Session::knowledge calls Request::validate
before opening or reconciling the catalog, then enforces trusted-root identity
and existing request bounds. The request validator enforces all five modes,
identity requirements, nonblank optional strings, mode-specific filters,
role/label pairing, enums and lowercase 64-hex hash syntax. Source acquisition
compares the supplied hash with the exact source revision. No request alias exists.

Unfiltered search is accepted: text, track and other discovery predicates are
optional. The descriptor repair must not invent a new search-input requirement.
Source path/hash are optional selectors/pins; source still requires a unit id.
Currentness, limits and acquisition refusal remain runtime-owned.

Clean-CTX-first code-context acquisition inspected both descriptor and parser.
The descriptor returned skeleton-only code context, so its known source file was
read directly. Existing task preflight contexts remain applicable: no admitted
source, declaration or edge has been modified by this checkpoint.

## Available host implementation and capture limits

The installed npm Codex package contains a JavaScript launcher, platform-native
executables and runtime resources. The inspected launcher selects/spawns the
native executable; the schema-normalization implementation is not exposed there.
No corresponding native Rust source was found in that package's file inventory.
No third-party host source or normal configuration is modified.

We have production tools/list and actual submitted arguments/results. We do not
have raw host-normalized or model-visible schema bytes. Agent reports consistently
describe only mode branches, matching the production oneOf branch properties.
The first directly observable discrepancy is between the production schema and
the agent's report of visible arguments. It cannot yet be assigned to a specific
internal transformer. Model interpretation remains an alternative explanation.

Official [Codex MCP documentation](https://learn.chatgpt.com/docs/extend/mcp?surface=cli)
documents supported stdio launch/configuration. It does not establish the internal
conversion behavior in these captures. The schema shape comparison must be judged
by actual before/after host behavior, with that limitation retained.

## Bounded candidate and verification boundary

Evaluate a flat object exposing all 15 fields, explicit string/integer types,
unchanged enums/minima/patterns/required common fields and closed properties.
Replace descriptor composition with accurate mode-specific descriptions; preserve
every authoritative runtime check unchanged. This changes descriptor expressiveness,
not the set of accepted runtime requests. One tool/five modes remain.

Tracked tests are added under src/tests/mcp/pcg_knowledge_schema.rs and registered
from the existing knowledge test module. The descriptor assertion requires the
flat candidate. Its expected RED on original composition is a local descriptor
contract regression, not a reproduction of an opaque host conversion algorithm.
The second test exercises registered valid operations and adversarial requests,
including the historical query-field mistake and hash mismatch.

Repository architecture.md requires user-observed RED before production edits,
followed by narrowly stashing/restoring the exact regression. Production remains
unchanged until that result is available. All tests, Cargo commands, builds and
runtimes are user-run. No new production work scope, cache, graph state, token
policy or semantic producer is introduced; AF-PERF operation costs are unchanged.

After RED: implement the bounded descriptor, restore exact tests, obtain focused
GREEN and scoped Clippy/build results, record the new executable identity, inspect
production tools/list and unchanged requests/refusals, then run one consolidated
actual-host smoke. Successful compatibility remains UNESTABLISHED at this checkpoint.

## First tracked run: fixture correction required

The user-run schema-red.log compiled and executed both tests. The descriptor
assertion failed as intended on oneOf. The runtime preservation test failed on
E_MALFORMED_DECLARATION because the test's continued Rust string stripped list
indentation. This is a test-fixture defect, not production RED. The fixture is
now a raw multiline string retaining exact list indentation. Assertions and
production code are unchanged. Repeat the focused run with a new log before
using the corrected regression as a baseline; preserve the original failed log.
