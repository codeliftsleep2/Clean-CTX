# #139 decorated declaration owner identity repair

## Verdict

**#139 DECORATED OWNER IDENTITY SLICE COMPLETE**

Only the decorated TypeScript class-owner criterion is repaired. Pinned CBM
A0–B5, #136 insertion authority, #137 semantic diff evidence and #135 DI
retention remain frozen. No framework declaration carrier, module/route
resolution, selector grammar, hydration or workspace-index policy changed.

## Proven root cause

**PROVEN BY CODE INSPECTION:** `TS_QUERY` recognizes `class_declaration` and
`abstract_class_declaration`. `CoreIRPass` formerly normalized `class.root`
using `compaction::extract_bare_class_name`. That helper takes the first source
line, strips modifiers/keywords and takes a name token. A decorator inside a
non-exported declaration therefore becomes the class identity instead of the
declaration's own `name` field. Export wrappers happen to give the existing
normalizer a different source shape; export is not the semantic authority.

```text
recognized class declaration, including decorator source
  → first-line class-name normalization
  → DefClass name = @Injectable() / @Audit( / @First()
  → correct span-owned member attached to incorrectly named owner
  → canonical hierarchy and Edit UnitTable carry the wrong owner name
  → OwnerProbe.read cannot resolve
```

**PROVEN BY TEST:** the registered dispatcher returns JSON-RPC -32602,
`focus owner not found: OwnerProbe`, for the non-exported Injectable, multiline
custom decorator and multiple custom decorator fixtures. The same file
contains Alpha.read with a different authored body. Syntax validation passes
before each request; this is not a parser/fixture failure.

**PROVEN BY LIVE MCP:** pre-fix frames show the incorrect owner directly
(for example `owner-frame-16.json`: `C Alpha`, `C @Audit(`). Qualified focus
fails and qualified replacement returns `unit not found: OwnerProbe.read`.
These are distinct from framework recognition/publication results.

The primary permanent source oracle is this declaration pair, preceded by
`import { Injectable } from '@angular/core';` and followed by 100 repetitions
of `// unchanged economics padding\n` to exercise focused skeleton delivery:

```typescript
class Alpha { read() { return 'ALPHA_BODY'; } }
@Injectable()
class OwnerProbe { read() { return 'OWNER_BODY'; } }
```

The registered request is `provide_code_context` with the exact fixture
`filePath`, explicit `workspaceRoot`, `fidelity: "edit"` and
`focusMethods: ["OwnerProbe.read"]`. The live source manifests additionally
include the written OwnerProbe property; they are the complete source oracles
for the live matrix, not reconstructed source from the implementation.

## Repair and production integration

`src/ir/pipeline/core/typescript.rs` appends an IR-only structured query to
the existing single parse. Each name capture is the declaration's direct
`name: (type_identifier)` field, paired with its declaration capture using
the same query-match identity. The resulting name is joined to the canonical
`class.root` by exact start/end bytes. It updates only normalized identity;
raw declaration/decorator source remains intact. Private evidence captures
are removed before canonical emission and the persisted capture projection.

```text
registered provide_code_context
  → default IRCompiler / production PassPipeline / CoreIRPass
  → TypeScript declaration's own name field + exact declaration span
  → DefClass identifier and existing type scope
  → span-owned DefMethod / DefField
  → canonical hierarchy / qualified focus resolution
  → model-facing skeleton with the exact selected body
```

The same instructions reach the existing Edit `UnitTable` and qualified
`apply_edit` resolver. Live replacement changes exactly OwnerProbe.read;
Alpha.read, property, decorators and all other bytes stay unchanged. The next
context request in that session delivers the changed body. This is an
**EXPECTED CONSEQUENCE OF OWNER IDENTITY REPAIR**, not a #136 implementation.

The supplement is selected only for the existing TypeScript query. No base
query, general name normalizer, C# adapter or unrelated language producer was
changed. Ownership still uses the existing declaration-span machinery; names
do not replace canonical IDs. No regex, nearest-name rule, export inference,
framework edge, second parse or additional graph supplies identity.

## RED → GREEN evidence

The permanent regression lives under `src/tests/mcp/decorated_owner_identity.rs`
and is registered in `src/mcp/mod.rs`.

1. The corrected baseline six-test command returned **3 PASS / 3 FAIL**.
2. Only that regression and its module registration were stashed.
3. Production repair was implemented while the regression was absent.
4. The exact regression was restored and the same command returned **6/6 PASS**.
5. Added controls establish same-line/generic forms, abstract-class method and
   property ownership at Low/Medium/High/Edit, distinct canonical method IDs,
   bare-name ambiguity rejection and registered exact-target editing.
6. Final regression command returned **9/9 PASS**. Rustfmt and the subsequent
   Clippy single-element-loop cleanup changed layout/scaffolding only; source
   fixtures and behavioral oracles remain unchanged.

### Declaration matrix

Every live fixture contains Alpha.read and OwnerProbe.read with different bodies.
The focus oracle requires OWNER_BODY, excludes ALPHA_BODY, checks focused-body
metadata and checks source remains unchanged. The write oracle compares the
entire source to the independently authored expected replacement.

| Declaration form | RED | GREEN | Owner oracle | MCP boundary |
|---|---|---|---|---|
| Undecorated / non-exported | PASS | PASS | Exact OwnerProbe.read body/source | Focus + edit |
| Undecorated / exported | PASS | PASS | Same; export retained | Focus + edit |
| Injectable / non-exported | FAIL | PASS | Exact OwnerProbe.read | Focus + edit |
| Injectable / exported | PASS | PASS | Exact OwnerProbe.read | Focus + edit |
| Multiline custom decorator | FAIL | PASS | Decorator's Fake/Other text cannot become owner | Focus + edit |
| Multiple custom decorators | FAIL | PASS | First/Second/Alpha decorator text cannot become owner | Focus + edit |
| Same-line custom decorator | FAIL | PASS | Declaration field owns name independent of line | Focus + edit |
| Decorated generic class | FAIL | PASS | OwnerProbe is identifier, not type parameter | Focus + edit |
| Multiple classes / same member | Included in every row | Included in every row | Other owner's body excluded; entire other declaration preserved | Focus + edit |
| Abstract generic class / property | Not a RED claim | PASS at four fidelities | Both classes own their own read/value by canonical IDs | Production compiler control |

### Live artifacts (operator verification, separate from Rust tests)

All roots below are under `target/decorated-owner-identity/`:

- `20261012-034644-9859eb02`: pre-fix **6 PASS / 10 FAIL**, 16 focus/edit groups.
- `20261012-034840-2f20fbc1`: final candidate **32/32 PASS**, eight forms × focus,
  entities/provenance, exact edit and fresh focus. Entity and fresh-focus
  groups are additional candidate verification, not claimed baseline RED.
- The pre-fix and final authored source manifests compare exactly equal.
- Pre-fix binary SHA256:
  `c1ca63a88a309d97ec3f7b6acc609b8117a6d5b76e8f97029e9f7fca37483692`.
- Candidate binary SHA256:
  `d05c2115b51dad1fdb4a1927f7ad45811972f0b7aa4a2ee4e14e7afc98b80f60`.

The existing Phase B stdio session helper supplies tools/list admission, raw
requests/responses and session cleanup. CBM is disabled explicitly in each
owned Git-root fixture. No model-host invocation or shared daemon is involved.

`20261012-034749-fdb1edcf` is a candidate 24/24 run, although the copy of the
binary used there was mistakenly named `clean-ctx-139-prefixed`: the candidate
build had already completed before the copy. Its hash establishes candidate
provenance; it is **not** baseline evidence. The final run uses clean-ctx directly.

## Identity and publication separation

**PROVEN BY TEST:** the production hierarchy gives Alpha and OwnerProbe
separate class IDs, each with its own read method and value property. Selecting
both qualified methods yields two distinct canonical method IDs. Bare read is
rejected as ambiguous; no first matching member is silently selected.

**PROVEN BY LIVE MCP:** `entities_in_file` delivers exactly one builtin Class
occurrence for each owner with the exact fixture-file provenance. Qualified
focus delivers only the intended body; replacement preserves the entire other
owner's source. Builtin entities and focus/edit results are separate oracles.

**PROVEN BY LIVE MCP:** the edge-free Injectable declarations still deliver
no `angular / Service / OwnerProbe` entity in entities_in_file. Recognition of
the existing service marker and correct core owner do not establish independent
framework declaration publication. That known publication defect remains.

## Executed preservation

| Focused filter | Passed |
|---|---:|
| typescript_export_ownership_tests | 1 |
| regression_canonical_identity | 6 |
| signature_cross_language | 8 |
| ir::calls::typescript_tests:: | 24 |
| angular_meta::semantic_tests | 18 |
| angular_meta::constructor_di_edge_tests | 15 |
| core_ir_nested_ | 3 |
| java_call12_nested_class_method | 1 |
| rust_native_calls_preserve_caller | 1 |
| insert_before_declaration_ (#136) | 9 |
| semantic_signature_evidence_ (#137) | 5 |
| di_omission_ (#135) | 6 |

These establish their existing asserted contracts, including TypeScript nested
call ownership, C# nested class/enum owners and Java nested callable ownership.
The cross-language signature and Rust call checks are not comprehensive
cross-language owner acceptance. No full repository suite or CI gate ran.

## Separate anomalies and invalid initial control

The initial test/harness attempted to focus both owners successively. Even the
undecorated baseline's Alpha.read response omitted ALPHA_BODY. That is an
unrelated existing body-retention/presentation failure, not decorated-owner
RED. Initial live evidence is retained at `20261012-034336-f21b9410`.

Before production changes, the regression and harness were corrected to isolate
OwnerProbe.read (with Alpha still present as a negative exact-target control).
The corrected six-test baseline then produced the accepted 3/3 RED/PASS split.
No assertion was changed to accept the defective OwnerProbe result. Distinct
Alpha/OwnerProbe ownership is separately established by canonical-ID checks.
The cause of the missing earlier method body is **NOT ESTABLISHED** here.

An intermediate eight-focus baseline is retained at
`20261012-034418-d2f21a04`: 3 PASS / 5 FAIL. Final baseline adds controlled edits.

## Commands and outcomes

Cargo commands used the existing cloud Rust setup:

```bash
export PATH=/workspace/.tooling/cargo/bin:$PATH
export RUSTUP_HOME=/workspace/.tooling/rustup
export CARGO_HOME=/workspace/.tooling/cargo
cargo test --locked -p clean-ctx --lib --all-features -j 4 decorated_owner_identity_
# The same Cargo prefix was used for every preservation filter listed above.
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4
```

The regression ran initially with the invalid Alpha-body control (0/6), then
corrected baseline (3/6), unchanged restored GREEN (6/6), expanded controls
(8/8, then 9/9), and final scaffolding cleanup (9/9). The mistaken
`calls_typescript` filter selected zero tests; it supplied no evidence and was
replaced by the actual `ir::calls::typescript_tests::` registration (24/24).
Clippy initially rejected the regression's single-element loop. That test-only
scaffolding was simplified without changing its oracle; final Clippy passed.
The single authorized candidate binary build passed.

Focused rustfmt used the installed Rust toolchain, edition 2024,
`--config skip_children=true`, only on core.rs, typescript.rs, the regression
and its registration. No repository-wide format ran.

Live command used the installed PowerShell executable, `-NoProfile -File
verification/issue-reconciliation/Invoke-DecoratedOwnerIdentity.ps1 -BinaryPath
target/debug/clean-ctx`, with XDG cache/config/data paths under
`/workspace/.tooling/powershell-{cache,config,data}`. Runs and outcomes are
recorded above. All generated state stays under target/.

## Remaining #139 work and stop

- Independent Service/framework declaration publication remains defective.
- Module spreads, imported constants and forRoot/forChild interpretation remain
  unresolved; referenced framework kinds are not repaired.
- Nested/lazy route ownership and full-path resolution are not repaired.
- Basic generic owner identity passes; generic-constraint base-type/field
  interpretation is not repaired.
- Declaration/reference distinction, service injection-site duplicates,
  cross-file/barrel and re-export resolution remain separate criteria.
- Very-large-file empty-context behavior and selector/addressability limits
  are not established or repaired by this bounded matrix.
- The earlier same-named method-body anomaly remains outside this patch.

No whole-issue closure, #138/#140/#141/#142 acceptance or broad real-workspace
claim follows from this slice. Stop here for separate owner review.

## Changed files and final audit

- `src/ir/pipeline/core.rs`: append/consume TypeScript identity evidence before
  canonical emission, remove private evidence before downstream projection.
- `src/ir/pipeline/core/typescript.rs`: language-local direct-field query and
  query-match/exact-span join, leaving raw metadata source intact.
- `src/tests/mcp/decorated_owner_identity.rs`: permanent registered focus/edit
  regressions and canonical member/property/ambiguity controls.
- `src/mcp/mod.rs`: feature-gated test registration only.
- `verification/issue-reconciliation/Invoke-DecoratedOwnerIdentity.ps1`:
  reusable provider-disabled live MCP driver and independent source oracles.
- `verification/issue-reconciliation/README.md`: operator command and scope.
- This report and the historical reconciliation link: evidence and disposition.

Final inspection confirms one existing production parse, no changed frozen
production file, no global query/selector/schema change, no new ownership
framework and no obsolete alternative owner path for supported TypeScript
classes. New/materially edited code remains within 615 lines. Test/runtime
evidence is reported separately; unsupported syntax and remaining issue
criteria are not silently promoted to completion.
