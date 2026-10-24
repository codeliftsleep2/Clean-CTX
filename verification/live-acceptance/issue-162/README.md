# Issue #162: retained canonical relation binary acceptance

Agent: Agent MaxHeadRoom

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
node verification/live-acceptance/issue_162_live_acceptance.mjs ./target/debug/clean-ctx.exe
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
