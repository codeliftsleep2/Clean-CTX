# B4 proxy/compression truth

Scope: pinned upgrade plan §24/B4, existing six-operation proxy whitelist and
existing response modes. Standing user authorization covers focused checks on
this session/branch; the full suite remains CI-only.

**PROVEN BY EXISTING TEST:** eight compressor regressions observed behavioral
RED on unchanged B3 production. Two real-dispatcher regressions independently
observed lost soft-error structured evidence and stale startup availability.
Regression-only files/registrations were stashed before implementation, restored
unchanged, and all ten cbm_proxy_truth_ checks passed GREEN.

**PROVEN BY CURRENT CODE:** admitted raw response → existing proxy RPC/soft-error
classification → caller verification → compress_cbm_response → existing text
content and metadata. Current MCP JSON is compacted without shorthand collision,
null/empty stripping, string trimming or flattened row/cell boundaries. Only a
sole unannotated text block with no other result fields can be unwrapped. Multiple
content blocks, structuredContent, isError and metadata remain distinct in the
compact text representation. Tree text is retained verbatim. RPC error code/data
survive alongside the existing error marker. Soft-error responses retain both the
existing classification/recovery hint and original provider evidence. Success
metadata reads current bridge status rather than the immutable startup snapshot.
No public graph schema or extra proxy operation was added.

The historical bare internal-enrichment compressor retains its notation/savings
contract. Admitted MCP results/errors do not use that lossy path. Protocol-invalid
input is rejected upstream; the existing fallback is not claimed to preserve a
valid current provider response that never reaches it. Current wire truth takes
priority over a guaranteed compression ratio. Serialized channels in text are
not a claim that the host consumed provider structuredContent directly.

Existing inline compressor tests were relocated unchanged to src/tests/cbm with
the same module registration. All active files remain within the size ceiling.

Preservation passed: compressor baseline 8, retry/cache/enrichment 21, proxy
errors/project identity 2, integration 8, B3 continuation 14, B2 provider state 23,
B1 current wire 14, A4 recovery 24: 114 selected checks. Package Clippy passed
with all targets/features and -D warnings. No broad/live test filter was used.

## Operator verification

The portable PowerShell Phase B harness and per-phase matrix are now tracked at
verification/cbm. It reuses the established ProcessStartInfo/stdio session
pattern and creates only owned C#/TypeScript primary/additional roots. It checks
provider and Clean registries, grouped identity, independent status/coverage,
real cursor paging, default typed collection, a cross-file trace hub, architecture,
additional-root routing, raw single-page consistency, tree layout, errors and
whitelist boundaries. PowerShell AST validation passed. Live execution and
platform-specific binary provenance are separate evidence, not implied by syntax
validation or script creation. B5 remains the bundled live acceptance gate.

## Follow-up issue reconciliation

The owner requested current acceptance reconciliation for GitHub #133–#141 after
the upgrade. Current issue bodies were retrieved from their public GitHub pages
(the GraphQL/REST API route was forbidden). All nine were OPEN at retrieval.
The older architecture investigation explicitly lacked those bodies and mapped
finding families only; its #134–#137 labels are not the actual issue titles or
closure criteria. Reconciliation must use the retrieved subfindings and original
acceptance criteria, distinguish code/test/live evidence, and not equate a CBM
adapter repair with native C#/Angular/NgRx/edit/diff feature resolution. No issues
were changed or closed.
