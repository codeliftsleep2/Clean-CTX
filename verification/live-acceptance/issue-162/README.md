# Issue #162: retained canonical relation binary acceptance

Agent: Agent MaxHeadRoom

## Independent suites: maximum 500 checks each

The former monolithic driver is replaced by explicit suite selection. Each
entrypoint under suites/ has its own three-session workspace, report, binary/input
hashes and maximum of 500 recorded checks. Exceeding the limit fails the suite;
checks are never truncated into a misleading PASS. Shared fixtures, assertion
helpers, lifecycle scenarios and the established McpClient transport are reused.

For the combined generation-65 C2/C3 major checkpoint, run the logged handoff
from the repository root. It checks JavaScript syntax, rebuilds once with all
features, then runs provider-imported-owners and existing provider-composition
preservation. Each remains a separate bounded three-session receipt. Stop on
any failure; full console output is retained under target/issue-162/.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-composition-checkpoint.ps1
```

The imported-owner suite covers three independently selected alias/barrel/namespace
routes for each of Component, Directive and Module, with same-named owners/tokens,
shared contributors and local siblings. It compares exact typed relation/source
multisets, refuses Directive/Module views, and exercises restart, mutable/cyclic/
deleted contributors, token/type-only barrel changes, independent field/source
retirement, stale restore before queries and unchanged-owner config/package repair.
Generation-65 operator receipts record PASS: 332 imported-owner checks and 190
same-source preservation checks, six FINISHED sessions, zero errors. The receipt
summary is target/issue-162/composition65-binary-receipt.json. These results do not
certify other suites, Rust/CI or final Claude field acceptance.

For an individual same-source preservation run after an explicit rebuild:

```powershell
node verification/live-acceptance/issue_162_live_acceptance.mjs ./target/debug/clean-ctx.exe provider-composition
```

An entrypoint can also run directly:

```powershell
node verification/live-acceptance/issue-162/suites/provider-composition.mjs ./target/debug/clean-ctx.exe
```

| Suite | Selected assertions |
| --- | --- |
| modules | Module/standalone composition |
| inject | Field/functional inject token controls in the DI fixture |
| constructor | Constructor token controls |
| strings | Exact string-token identity |
| members | Input/output members |
| functional-http | Guard/resolver/interceptor roles and inner-role decoys |
| provider-registration | Object-provider registrations |
| provider-shorthand | Class shorthand registrations |
| provider-strategies | Class/factory reference registrations |
| provider-value-registration | Native reference-valued registration |
| provider-directive | Directive-owned registration |
| provider-positions | Ordinary/view declaration distinction and independent field removal |
| provider-composition | Same-source const/alias, spread/nested membership and independent mutation/retirement |
| provider-imported-owners | Combined C2/C3 imported routes, exact actual owner/position and lifecycle |
| provider-class | Canonical class pairs |
| provider-factory | Canonical factory pairs |
| provider-existing | Canonical existing-token pairs |
| provider-value-presentation | Explicitly noncanonical value projection |

The 18-suite inventory is explicit in suite-manifest.mjs. A full run is opt-in:

```powershell
node verification/live-acceptance/issue_162_live_acceptance.mjs ./target/debug/clean-ctx.exe all
```

The launcher writes a summary containing suite counts/statuses and receipt paths,
not a combined thousands-of-checks array. It stops on the first failed suite.
Unrun suites are not certified by a targeted PASS. Full runs may take longer due
to independent session startup; targeted execution is the routine incremental
workflow. Shared lifecycle mutations are preserved, but assertions/queries for
unselected families are not executed. Fixture sources remain shared so existing
cross-family noise and dependency conditions are retained.

No production Rust or runtime transport policy changes accompany this split.
The original 16-suite split has a recorded full PASS at generation 62. The new
composition suite now has an inspected generation-63 PASS (190 checks), alongside
provider-positions preservation (160 checks). Both finished all three sessions
with zero failures. Other suites are not certified at generation 63 by those runs.

Check all suite definitions without starting an MCP runtime:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/check-syntax.ps1
```

### Historical monolithic fixture notes

The following notes describe the earlier combined driver and receipts. Current
suite selection and evidence ownership are governed by the instructions above.

The provider declaration extension reuses both functional checkpoint sources
and their existing token/interface mutation, restart, source replacement and
stale-restore phases. FunctionalProviderHost registers two same-named tokens
through aliases, repeats one registration and includes a refused type-only
token. Exact public token-file/owner pairs and private endpoint separation are
checked, including foreign-owner retraction without losing functional DI.
No new sessions or runtime policy are added. This extension is UNRUN; it does
not establish provider precedence, scope closure or consumer satisfaction.

The functional/public-entry extension adds functional.ts and
functional-neighbor.ts checkpoints and a hashed functional-scenario helper in
the same three sessions. It checks CanMatchFn, generic ResolveFn and HTTP
interceptor roles, distinct same-named token identities, deduplication, private
canonical separation, inner-role decoy refusal, restart, token mutation,
independent owner replacement, wrong HTTP entry/export-map/config mutation and
package-version stale restore before reacquisition. The HTTP entry belongs to
the separate @angular/common owning package. All fixture inputs exist before
capture. This extension is authored and UNRUN; prior binary receipts cannot
verify it. It is operator acceptance evidence, not tracked regression coverage
or framework-wide completeness.

The member extension adds two checkpoint sources and a hashed member-scenario
helper inside the same three sessions. It checks exact HasInput/HasOutput sets
for independent same-named owners, barrel/local/namespace APIs amid noise,
restart, foreign API mutation/repair, independent source replacement and stale
restore before reacquisition. It certifies neither canonical member identity nor
full AQ10 completeness. This extension is authored; execution requires a fresh
operator receipt.

This operator-run driver uses the established McpClient stdio transport and
Node built-ins. It starts only the explicitly supplied binary; it does not build,
install dependencies, start CBM, or contact external services. This is binary/MCP
evidence, separate from tracked Rust regressions, CI, certification and the final
Claude real-workspace field gate.

Run from the repository root after completing edits:

```powershell
cargo build --all-features --bin clean-ctx
node verification/live-acceptance/issue_162_live_acceptance.mjs ./target/debug/clean-ctx.exe all
```

On Unix pass `./target/debug/clean-ctx`. The driver rejects a binary predating
Rust/build inputs, records binary/input SHA-256 hashes, discovers registered
tools and captures the workspace-query output schema before semantic calls.
It checks that executable and repository inputs remain unchanged during the run.
No agent-run execution is claimed. An optional syntax-only preflight is
`node --check verification/live-acceptance/issue_162_live_acceptance.mjs`.

Three sequential server processes share one isolated synthetic workspace and
SQLite database under `target/live-acceptance/issue-162/`:

1. Cold compression and repeated public queries retain two same-named Directive
   endpoints through aliased NgModule declarations/exports and array spreads.
   An aliased barrel-imported NgModule also keeps its canonical target. A real
   save_context checkpoints the owned snapshot.
2. A fresh process restores that snapshot, then checks type-only role mutation,
   deletion of one endpoint, restoration, and another explicit checkpoint.
   The unrelated declaration and imported module must survive independently.
3. A dependency changes before startup. Restore must reject stale proof. Public
   queries must reacquire the surviving fact and later restore both after the
   dependency is repaired, without changing the initiating source.

The same three processes also exercise a default-standalone Dashboard Component
using aliased imports and a shared array of both Directive endpoints and the
barrel-imported NgModule. Both owners are compressed, saved and restored.
Directive type-only mutation/deletion retracts only its imports; explicit
standalone:false removes the direct Component import while preserving the
existing module-role projection. A witnessed core-package change from 20.0.0 to
18.2.0 retracts Dashboard's default eligibility without editing either owner;
restoring the manifest restores its imports. Stale restore is refused for both
owners before reacquisition. Both initiating source hashes remain unchanged.

Every membership assertion compares name, role and exact normalized endpoint
file, multiplicity, asserting owner and domain. Internal canonical annotations
must stay absent from public endpoint JSON. Target bytes remain unchanged during
all dependency transitions. The trace/report records fixtures, configuration,
mutations, checks, schemas, raw RPC and stderr. It does not read private durable
identity internals or assert complete reverse closure from forward answers.

Return the printed PASS/FAIL summary and evidence directory. A failure preserves
the report and stops later sessions. The runner never weakens a failed assertion,
retries to hide a failure, or edits production files. Existing client request and
shutdown bounds remain unchanged. This asset supplements the tracked regressions
in `src/tests/mcp/angular_module_*_issue_162.rs`; it cannot replace them.

The same three sessions now include a mixed-source InjectOwner with barrel and
namespace API aliases, two same-named canonical token owners, repeated fields,
a symbolic missing token, unrelated missing imports, type-only/foreign/ordinary
member calls, optional/nested calls, lexical shadows and misleading strings.
The inject helper is `inject-scenario.mjs`; its hash is retained with the driver
and transport hashes. All three owners are compressed, saved and restored.

Runtime-token interface mutation retracts only its canonical fact. A witnessed
ambient file changes MissingService from absent to class, to interface, and back
to absent: the public token endpoint must move between asserting-source symbolic
provenance, the actual ambient declaration file, and no relationship. The driver
checks exact multiplicity and owner files, public canonical privacy and unchanged
initiating bytes throughout. The third process refuses stale restoration of the
inject owner before reacquiring current facts. These checks add no server session.
They supplement the tracked inject authority/mixed/lifecycle tests; public file
metadata by itself is not proof of a canonical declaration.

After the ambient mutations, session 2 explicitly requires the old restored
AppModule session checkpoint to be refused by save_context. Query publication
refreshes the index rather than replacing that session snapshot. The driver then
reacquires all checkpoint session snapshots through compress_code_context, rechecks
their public facts, and saves the new current checkpoint. This is an explicit
new checkpoint acquisition, not a retry of failed acceptance or a production
change to relax witness validation.

Constructor acceptance now shares those same three sessions. Two additional
consumers in di.ts prevent already-present field facts from hiding constructor
failures: TypedCtorOwner requires exact aliased/namespace class tokens from two
same-named owners; CtorOwner requires a barrel-aliased Inject override to select
a different canonical token than its annotation, then converge with a field use.
All public facts compare exact name/role/file multiplicity and owner provenance.

Restart restores their shared source-owned payload. Token interface mutation
retracts affected constructor facts; API-barrel retargeting to a foreign Inject
restores the annotation token without granting foreign override authority.
Type-only token-barrel mutation removes only the override consumer's facts while
independent field and typed-constructor consumers survive. Restored barrels
restore the original facts. Stale restore is rejected before reacquisition;
package-version transitions revalidate the constructor consumers too. Initiating
bytes remain unchanged. These are operator binary assertions, not additional
Rust test coverage or proof of complete constructor/DI semantics.

The generation-38 batch additionally augments the left class with a same-named
interface, retaining its runtime constructor token despite extra type-only
contributors. Duplicate class bodies then retract only that owner's facts;
repair restores them. The second session checkpoints the repaired merged family,
so the third fresh process also consumes the merged-family snapshot. This is a
bounded existing native declaration-family case, not general type-checking parity.
The tracked merged-family durable test separately checks complete retained IDs/
witnesses in session and index before query acquisition and retraction when the
runtime class is removed.

The generation-54 extension checks canonical class-strategy pairs for explicit
and shorthand forms, repeated registrations, same-named targets and a string
token. An implementation-only interface mutation retracts its class strategies
and the registration requiring that runtime class; the unrelated factory
registration survives. Existing sessions and transport
policy are reused. Corrected generation-54 report
2026-10-25T02-45-45-417Z-28364 records PASS: 1102/1102 checks, three FINISHED
sessions and no errors. The interface-mutation expectation correction and
original failed evidence are documented in the provider strategy identity
decision. This is operator binary evidence, not a Rust/CI or final field gate.

After provider occurrence-span admission repair, generation-55 report
2026-10-25T03-42-21-209Z-24324 records PASS: 1102/1102 checks, three FINISHED
sessions and no errors, using rebuilt binary SHA-256
5d43e0d52ce10fff41bbebb170ad8300b17e6371e3331d87777af032c19ae44a.
The tracked forged-span regression is separate Rust evidence; no full repository
or final field gate is implied by this operator acceptance run.

After the exact native value-reference occurrence join, generation-56 report
2026-10-25T14-09-56-640Z-24816 records PASS: 1102/1102 checks, three FINISHED
sessions and no errors. Rebuilt binary SHA-256:
699bc887fb489df3ec1c9013cac67fca874de0705384c6a6039745200a7210ea.
The tracked shifted-span regression is distinct Rust evidence; this receipt
does not establish the full repository or final field gate.

The generation-57 factory extension requires exact token/function origins for
same-named functions, repeated pairs and a string token, with independent
function-target interface mutation/repair. Existing three-session lifecycle,
owner/package and class-strategy assertions remain. Generation-57 report
2026-10-25T15-44-43-046Z-9404 records PASS: 1292/1292 checks, three FINISHED
sessions and no errors, using rebuilt binary SHA-256
872e4a541f7706b839f461072fdbc76473f54346d34f39c9b6c233bc613ae00b.
This is operator binary evidence, separate from Rust/CI and final field gates.

The generation-58 alias extension adds an independent useExisting owner and
requires exact ProviderUsesExisting endpoints for declaration/string positions
and repeated pairs through the existing three-session mutations and restart.
It does not infer target registration, alias-chain resolution or satisfaction.
Generation-58 report 2026-10-25T17-34-16-718Z-26952 records PASS: 1499/1499
checks, three FINISHED sessions and no errors. Rebuilt binary SHA-256:
4533863e18facb9552e0a9fc4ee4042ff6f64bb9dbd1c18f034b6701e494410c.
This is operator binary evidence, distinct from Rust/CI and final field gates.

The generation-59 reference-value extension verifies canonical registration
origins through the existing sessions and separately checks the legacy value
presentation's same-label deduplication and asserting-source metadata. The first
run's incorrect absent-file expectation is preserved in the value-reference
decision record. Corrected report 2026-10-25T19-21-16-341Z-16680 records PASS:
1660/1660 checks, three FINISHED sessions and no errors. Rebuilt binary SHA-256:
aa2a64437fc940fd01f620ada9228018bbdb65745d6d302a10a53e3f127ef685.
This does not establish canonical runtime-value identity or injector closure.

The generation-53 extension adds an independent strategy-provider owner with
namespace/barrel useClass and useFactory. Its registrations share the existing
restart and mutation scenarios and additionally retract on a type-only strategy
barrel, recovering after repair without removing object/shorthand neighbors.
Generation-53 report 2026-10-24T23-21-30-640Z-3972 records PASS: 936/936 checks,
three FINISHED sessions and no errors. This is operator binary evidence,
separate from tracked Rust results and final field acceptance.

The generation-52 provider extension adds a separate shorthand-only owner to
the existing functional helper. Local and namespace references must preserve two
same-named class identities, deduplicate repeated registration and refuse a
type-only token through restart, dependency mutation and foreign-owner refusal.
Object-provider assertions remain independent. A rebuilt generation-52 binary
receipt verifies this extension: report 2026-10-24T20-18-22-338Z-17500 records
845/845 checks, three FINISHED sessions and no errors. This is operator binary
evidence, separate from tracked Rust controls and final field acceptance.

The generation-41 extension uses the same three fresh processes and database,
adding strings.ts and string-neighbor.ts to the five-source checkpoint. Two
same-named owners retain independent string facts through const/local/barrel/
namespace routes; equivalent escapes converge, while two InjectionToken objects
with equal descriptions remain distinct. A direct field string call must not
gain support. Provider labels share the proved value, without claiming strategy
authority, injector satisfaction or scope closure.

The second session restores those owners, changes the underlying immutable
string without editing consumers, checks replacement alongside a direct literal,
then restores the dependency before the normal checkpoint reacquisition. The
third process changes the dependency before startup, requires stale restores to
be refused, reacquires the changed values and verifies repair. The new helper is
included in the existing input hashes. No extra process, retry or timeout policy
is introduced. Exact full-payload restoration and external-deletion refresh are
covered separately by the tracked string lifecycle tests. This extension is
authored and UNRUN until a new operator receipt exists; older PASS receipts do
not verify it.
