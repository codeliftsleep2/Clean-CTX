# Issues #153/#154: C# compilation declaration identity

Agent: Agent MaxHeadRoom

Status: production repair and local verification complete with operator-reported
GREEN; original-workspace Claude field acceptance remains pending. Issue #155
is excluded.

Maintainer update: the handed-off preservation/final-gate checks were reported
GREEN on 2026-10-20. Exact additional counts and logs were not supplied; do not
infer them.

Subsequent maintainer update on 2026-10-20: all checks for the tracked live
harness were reported passing. Record this as operator-reported PASS for the
handed-off build, syntax/guard checks, and binary/stdio harness. Exact assertion
counts and the generated run directory were not supplied or independently
inspected. The original-workspace Claude field acceptance remains unverified.

## Tracked live verification asset

The operator driver is
`verification/live-acceptance/csharp_compilation_identity_live_acceptance.mjs`.
Its campaign-local transport, fixture generator, oracles, and operator README
live under `verification/live-acceptance/csharp-compilation-identity/`.
It covers explicit file admission, exact action identities, four framework
ancestry consumers, generic/partial preservation, cold/warm and publication-order
equivalence, edits, retraction, and restoration. Raw RPC exchanges and generated
state remain under unique `target/live-acceptance/csharp-compilation-identity/`
run directories. It intentionally does not test Issue #155 cold acquisition.

The harness was handed to the user through its README, and the maintainer
subsequently reported all checks passing. The agent did not run this harness.
Operator/live results remain separate from Rust test coverage and CI results.

### Remaining batched field acceptance

Use one Claude session against the rebuilt executable in the original relevant
workspace. Confirm the exact ActionA-ActionE set, no constructor-named action,
and the original generic-method/descendant ancestry scenarios across applicable
frameworks and partial-class arrangements. Repeat the affected queries warm.
Keep proprietary source local and report sanitized observations only. No
additional framework-specific repair or Issue #155 work is authorized by this
acceptance step. Do not mark #153/#154 fully closed until this relevant field
acceptance is reported.

## Earliest divergence

### #153: independent sequences are zipped

`src/ir/pipeline/core.rs::process_method_capture` has the declaration capture
span when it allocates the method ID and emits the method IR. That is the
earliest available association between source declaration and semantic record.

`src/ir/csharp_resolution/compilation.rs::class_inputs` later collects every
`DefMethod` owned by the selected IR class, including constructors.
`compilation/method_inputs.rs::direct_methods` collects only `method.root`
captures, sorts them, and zips them with those records. The C# query captures
method declarations separately from constructors. These sequences have
different domains: the join can attach action attributes to a constructor
name, shift later identities, and truncate a real method. Sorting cannot repair
the missing identity relation. Overloads also rule out joining by bare name.

### #154: descendant syntax supplies owner identity

`src/queries.rs::CS_SEMANTIC_QUERY` captures every `type_parameter_list` under
one unqualified capture name. `class_inputs` chooses the earliest such capture
inside the class, excluding nested classes but not methods. A method's generic
arity is consequently written into the class's `CSharpSymbolKey`. The source
index and ancestry graph then correlate different keys for the base declaration
and its descendants. For partial classes, the polluted key can also separate
parts which should merge.

## Implemented shared repair

Resolve by identity, render by position.

`PassContext::csharp_declarations` owns compilation-local declaration inputs.
Core lowering records each method/constructor's exact source span with its
allocated method ID and emitted name. Compilation input assembly looks up each
method capture by that exact span. Constructor IR and existing method-ID
consumers remain intact. Names, record order, and constructor offsets do not
establish this association.

The semantic query binds each class declaration and its direct type-parameter
children in the same query match. Before projecting captures to `CapEntry`,
core lowering joins those captures by `match_index` and counts unique parameter
node spans under the exact owning class span. No header reparsing, comma
counting, or descendant exclusion establishes class arity. These private
captures are skipped during canonical dispatch so they cannot alter scope
transition timing.

Both meta-layer assembly and the retained compiler-unit path receive the same
inputs. Resolved units own the corrected facts after temporary capture state is
dropped. Existing producer compatibility generations advance for C# canonical
output (5), C# semantic inputs (8), and .NET semantics (7); unrelated producer
keys and the unchanged generic C# projection implementation remain unchanged.
No public contract, framework correction path, or dependency acquisition was
introduced.

## Production path and consumer audit

The default meta-layer pass invokes compilation ancestry assembly and supplies
resolved classes to the .NET semantic context. The compiler also retains a
`CSharpCompilationUnit` in its result-side state. The semantic compilation
artifact owns that unit; C# workspace reconciliation resolves accumulated
units by its existing resolution partitions and regenerates semantic/IR
projections. Both paths must receive the same corrected declaration facts.

ASP.NET consumes resolved method attributes for action edges. DbContext,
AutoMapper, SignalR, and ASP.NET use shared class ancestry for framework
admission. SignalR additionally projects realtime execution contexts from
method IDs. Constructor IDs must remain intact. MSTest class attributes share
the resolved class machinery and require preservation verification as well.

This is a static call-path trace, not proof of live reachability or lifecycle
correctness. Recompilation and artifact replacement need preservation checks
after the repair. No live process was started.

## Nearby association classification

- `direct_methods` positional zip: semantic identity violation in scope.
- Class type-parameter first-contained selection: semantic identity violation
  in scope.
- `class_inputs` removes the first same-named IR class record: separate
  name/occurrence association risk for repeated class names across scopes.
  Record separately; do not silently broaden this campaign.
- Namespace and containing-class sorting: lexical scope navigation/order;
  not an independent declaration-record join.
- First base-list entry selection: source order has C# syntactic meaning,
  after ownership is established. Existing nearest-type filtering in core
  lowering differs from compilation's class-only nested exclusion. Nested
  non-class type ownership deserves a separate regression/investigation.
- Class/namespace name selection from first contained identifier capture:
  parser-shape-dependent association, requiring separate ownership review;
  not mechanically rewritten here.

## Regression envelope

`src/tests/dotnet_meta/csharp_compilation_identity.rs` is registered through
`src/dotnet_meta/semantic.rs` using the existing test-file convention.

- Exact five-action set with constructors at the beginning, middle, and end;
  forward/reverse method order; empty/nonempty constructor bodies.
- Overloaded methods retain distinct attributed declarations and final method.
- All four framework ancestry consumers, two descendant levels, cross-namespace
  references, multiple generic methods and generic overloads of arity 1/2/3.
- Partial declarations with generic methods in either the base-list part or
  the other part.
- Genuine generic classes of arity 1/2/3 preserve framework ancestry despite
  a method of arity 4.

`csharp_compilation_identity_preservation.rs` additionally verifies exact
span-to-distinct-IR-ID associations for constructors and overloads; artifact
ownership and cross-file partial composition for all four ancestry consumers;
emitted action/hub/mapping/DbContext facts; SignalR realtime IDs; base-artifact
replacement/retraction and restoration; compiler reuse; and MSTest class
attribute preservation with genuine class generics and generic members.

The original RED tests were preserved using a scoped Git stash and restored
unchanged before the first GREEN run. Only subsequent rustfmt layout changes
were applied to that file; its assertions and fixtures remain unchanged.

## Captured verification

- Pre-fix RED: 2 passed, 3 failed, 4062 filtered out. Constructor action and
  overload association assertions prove #153; missing descendant Controller
  ancestry proves #154. The generic-class preservation and unrelated NgRx
  lifecycle tests passed.
- First unchanged-test GREEN after repair: 5 passed, 0 failed, 4062 filtered
  out; main target selected 0 tests. This establishes the original RED/GREEN
  pair before adding preservation tests.
- The first expanded-batch compilation exposed an incorrect hash-crate name in
  the new test helper. It was corrected to the repository's existing
  `crate::cache::content_hash`; this compilation error is not behavioral RED.
- Two invalid assertions in the new preservation test were corrected: symbol
  iteration order is unspecified, and a retained `builtin/Class` declaration
  is not a .NET framework role. Complete records are compared after sorting;
  role retraction is checked only in the `dotnet` domain. No production repair
  was based on either test-authoring error.
- Expanded campaign GREEN: 8 passed, 0 failed, 4062 filtered out; main selected
  0 tests. Includes the seven campaign tests and one unrelated NgRx test.
- .NET semantic preservation: 25 passed, 0 failed, 4045 filtered out; main
  selected 0 tests. Output: `target/issues-153-154-dotnet.log`.
- Formatting passed before the final test-only layout adjustment; the final
  formatting rerun remains pending. File-size guard passed with 15 active
  files; nine validator self-tests passed. Twelve untouched oversized legacy
  files were reported without becoming active. Strict UTF-8 guard passed:
  1471 text files, zero BOMs, zero unexplained mojibake signatures.
- C# preservation was interrupted on the user's instruction to hand off tests.
  No completed result is claimed. Its partial output remains at
  `target/issues-153-154-csharp.log`.
- Clippy, full workspace tests, and the Rust encoding test were NOT RUN.
  Remaining preservation and final-gate commands are user-owned. Closure is
  pending those results and any applicable field acceptance.

## Files changed

- `src/queries.rs`: structured class/type-parameter ownership captures.
- `src/ir/pipeline/core/csharp.rs`: private compilation declaration inputs and
  match-identity class arity collection.
- `src/ir/pipeline/core.rs`: record emitted method identity by declaration span;
  collect/skip private ownership captures.
- `src/ir/pipeline.rs`: compilation-local owner for declaration inputs.
- `src/ir/pipeline/meta_layer.rs`, `src/ir/compiler.rs`: pass identical
  declaration inputs to immediate and retained compilation assembly.
- `src/ir/csharp_resolution/compilation.rs` and
  `src/ir/csharp_resolution/compilation/method_inputs.rs`: consume exact spans
  instead of earliest descendant parameters or positional method joins.
- `src/ir/layers/csharp.rs`, `src/dotnet_meta/mod.rs`: advance existing scoped
  compatibility generations for changed canonical/semantic outputs.
- `src/dotnet_meta/semantic.rs`: register both tracked regression files.
- `src/tests/dotnet_meta/csharp_compilation_identity.rs`: established RED tests,
  subsequently formatted only.
- `src/tests/dotnet_meta/csharp_compilation_identity_preservation.rs`: span,
  artifact, cross-file partial, payload, lifecycle, and MSTest preservation.
- `src/tests/mcp/workspace_query_csharp_structured_headers.rs` and
  `src/tests/mcp/dotnet_test_subject_compatibility.rs`: current generation
  contract expectations.
- `docs/ARCHITECTURAL_INVARIANTS.md`: clarify declaration provenance in the
  existing CSHARP-RESOLUTION-001 invariant.
- This report: investigation, implementation, verification, and scope record.

## Operator verification sequence

The same command established RED against unchanged production code and then
GREEN against the shared repair:

```powershell
cargo test --all-features compilation_identity -- --nocapture
```

Behavioral assertion failures for the two defects established RED. Compilation
errors and invalid supplemental assertions were not treated as RED evidence.
The exact original tests were preserved through the scoped stash/restore
procedure and passed before subsequent formatting.

The authorized preservation commands are:

```powershell
cargo test --all-features dotnet_meta::semantic::tests -- --nocapture
cargo test --all-features csharp -- --nocapture
```

The production-repair attachment included execution language; the user then
clarified that tests must be handed off. The active C# suite was stopped and
no further verification will be started by the agent without a new explicit
per-run instruction. The final repository gate remains solely defined in
`docs/agent/verification.md`; its command list is not duplicated here.
No standalone binary, provider, or live MCP process was started.
