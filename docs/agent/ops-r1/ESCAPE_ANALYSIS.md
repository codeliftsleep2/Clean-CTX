# OPS-R1 historical defect escape analysis

Agent MaxHeadRoom. Historical source/assertion evidence is distinguished from
actually replayed historical builds. No mutation/Cargo command ran in this audit.

## HF-001: query compilation

The original complete IR path compiled invariant TS/C# query strings for each
observation. Existing output tests could remain equal while compilation repeated.
The tracked warm witness additionally asserts query_compilation_count=0 after
warmup, parse_count=1 and cursor_count=1. Frozen historical RED directly establishes
detection, rather than inferring it from a new PASS. The complete-definition and
dynamic mismatch tests preserve the legitimate dynamic path. The escape was a
missing work assertion at that exact boundary, not absence of functional value.

## HF-002: feature isolation

Current CI executes allfeature tests, while early check/Clippy use package defaults.
Defaults include TS, C#, Angular and DotNet, so neither independently compiles
C#-only or TS-only. Missing cfg boundaries can therefore remain hidden. Retained
compiler failure logs and five-mode correction receipts establish the selected
incident; all-feature success cannot stand in for those modes. The smallest guard
is supported-mode compile/list plus mode-native tests, not every feature combination.
Exact historical full-suite membership at every predecessor is not reconstructed.

## HF-003: deletion

The legacy public query witness acquires one membership, relocates the asserting
file, deletes it and recreates it, asserting exact results at each step. It never
calls the new registration operation. That isolates a pre-existing shared retirement
defect from new operation behavior. Earlier positive identity/target tests did not
perform that sequence. Existing repair controls distinguish acquisition errors,
reappearance, sibling supports and relationship families. Historical exact RED is
retained; approved formatting successor is not attributed to original executions.

## HF-004: restore configuration

The unchanged-source bootstrap witness checks current retained authority and
registration applicability before invoking the production current validator.
Its absent optional routing identity exposed the wrong enablement predicate.
Existing restore tests with present identity could pass while this valid case
failed. The config matrix adds enabled/disabled, native/nonnative and current/
historical variations with stale/forged refusal. A newly written matrix originally
misread historical H1 as needing H2 currentness; that failed capture remains one
failure, not retroactively GREEN. The shared repair is smaller than inventing an identity.

## HF-005: Angular broad work

The inspected resolver work test varies registration count 2/16/64 inside one
fixture and compares owner validations while the owner set remains fixed. It
asserts nomination once and inspected records equal count, preserving real output/
identity/work value. It does not increase unrelated owners or project units,
measure complete project acquisitions, count live artifacts or establish bounded
absent-route termination. Thus all its assertions can pass while cold workspace
preparation dominates. The 185-check synthetic production test likewise exercises
specific cold/warm/restore/mutation fixtures, not arbitrary project scale.
Missing counterexample: no-resolver/absent-route, independently scaled owner and
project dimensions, attributable stage/retention counts, external termination bound.
No claim is made that moving expensive measurements out of CI caused this escape.

## HF-006/007/008: verification and knowledge

Actual document admission is a different contract from encoding/format/size.
Existing B11 tests verify adversarial engine behavior. They do not prove each
agent retrieved historical constraints or each publication ran the checker.
Proposed prevention belongs in existing authoring/publication mechanics, not
an alternate agent-generated authorization system. A historical bypass mutation
demonstration remains pending; no invented concrete failure revision is assigned.

CI membership tests mutate evidence, execute the actual Python verifier and assert
its result. This is stronger than checking a command appears in YAML. The verifier
compares execution against listing and explicit required groups. It cannot protect
an architectural obligation never included in those groups. Interleaved/nested real
logs exposed parser assumptions; real frozen fixtures and rejection cases are the
appropriate counterexample, not repeatedly running production suites.

## Additional confirmed test-contract defects

src/tests/mcp/dispatcher_regression.rs::shutdown_stops_new_requests does not call
shutdown and asserts result.is_ok() || result.is_err(). No Result outcome can
violate that assertion. shutdown_handles_stuck_workers spawns a 10-second sleeper
while the shutdown call/elapsed assertion is commented out. It does not prove
timeout handling. shutdown_completes_inflight_requests checks eventual completion
after sleep without shutdown. These are source-confirmed effectiveness gaps, not
evidence the dispatcher caused the Angular hang. Preserve legitimate spawn/cleanup
coverage when proposing replacements; no test deletion or new timeout is authorized.

## Mutation and value review boundary

Do not mutate accepted branch production merely to collect RED. Prefer retained
exact frozen RED first. Where new proof is needed, use an isolated source state,
exact witness/hash and owner-run command with failure reason recorded. Compiler
RED, behavioral RED, harness stop, parser rejection and field observation are
different evidence classes. A test name, body hash or assert count alone cannot
prove useful protection, redundancy or historical execution.
