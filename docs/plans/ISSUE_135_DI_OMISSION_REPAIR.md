# #135 DI registration omission repair

## 1. Verdict

**#135 DI OMISSION DISCLOSURE SLICE COMPLETE**

The silent ten-entry loss is repaired for the existing recognized written DI
registration set. The entire #135 issue remains open. #136, #137, CBM A0–B5,
acquisition, Angular/NgRx and #142 production paths were not modified.

## 2. Proven root cause

**PROVEN BY CODE INSPECTION:** a captured C# class source reaches extract_general,
then extract_di_registrations scans the existing supported generic registration
forms. It collects the entire vector, applies existing adjacent deduplication,
then truncated it to ten. For the forty-distinct-call fixture the normalized set
and its authoritative count existed immediately before truncate(10). The discarded
thirty identities and original count did not reach MetaBlock or context rendering.

The path was:

```text
40 recognized source registrations → normalized vector of 40
→ truncate(10) → only 10 marker lines
→ TypeAlias/context projection → public response with no DI omission signal
```

**PROVEN BY TEST and LIVE MCP:** original Medium/High output for eleven and forty
authored registrations delivered ten. Native degradation metadata did not disclose
that DI loss. The registered dispatcher and live stdio responses, rather than a
private count alone, establish the defect.

**PROVEN BY CODE INSPECTION:** commit 498966c introduced this cap with the initial
.NET meta-layer. Its source does not explain the limit or preserve a count. Narrow
documentation inspection found no independent intentional DI bounded-delivery
contract. Historical intent is **NOT ESTABLISHED**; attributing the cap to old
compression economics is an **INFERENCE**, not approval authority.

## 3. Repair decision: COMPLETE DELIVERY

Only the DI truncation is removed. Existing pattern recognition, adjacent
deduplication, registration content and deterministic grouping remain unchanged.
The producer already allocated/scanned the entire set before truncation, so the
repair does not add an extraction pass or a new fact model. It lets the existing
projection/rendering see all recognized facts.

The current architecture already supports complete context delivery and byte-exact
source fallback. Existing input/resource limits and presentation economics remain
in force; no new cap is added or global fidelity policy changed. At forty calls,
the live fixture selects the existing raw fallback (3,115 UTF-8 bytes) rather than
a less economical expanded skeleton. A larger padded registered-test fixture still
delivers all forty compact DI markers. Both are truthful complete delivery.

There is consequently no reason to add shown/total/omitted fields or borrow query
lower-bound status for this result. No authoritative DI fact is discarded at this
producer cap anymore. Arbitrary pathological-input economics were not benchmarked;
only the actual bounded fixtures and existing policy path are claimed verified.

## 4. RED → GREEN matrix

Permanent regression: src/tests/mcp/di_omission.rs, registered through the normal
MCP test module. Corrected RED: four existing boundary/content controls PASS and
two over-cap tests FAIL. The unchanged regression/registration were stashed during
the production edit, restored, then all six passed. Later rustfmt changed layout
only, not fixture content or semantic assertions.

The initial fact-content oracle used ASCII -> instead of the existing Unicode →.
That was an invalid assertion, corrected before accepting RED; production was
unchanged during the corrected RED run. It is not counted as defect evidence.

Live RED: `target/di-omission/20261012-032435-11090941`: ten passing and six failing
groups. Live GREEN: `target/di-omission/20261012-033104-e31181a6`: **16/16 PASS**.
Both source-manifest files were compared and are identical, including exact source,
expected identities/order and conditional interpretation. The expanded complete-
delivery oracle was also checked against the immutable RED responses: the same six
over-cap groups still fail. The original loss was not made passing by oracle change.

| Count/case | RED | GREEN | Completeness oracle | MCP boundary |
| --- | --- | --- | --- | --- |
| 0 | Empty | Empty | No DI rows in an independently authored zero-call source | Registered dispatcher + live |
| 9 | Nine exact facts | Same nine facts | Complete identity/content/order list | Dispatcher + live |
| 10 | Ten exact facts | Same ten facts | Exactly cap is not partial | Dispatcher + live |
| 11 | Ten, one silently absent | Eleven exact facts | All independently authored identifiers delivered | Dispatcher + live |
| 40 | Ten, thirty silently absent | Forty markers in dispatcher fixture; exact complete source in live fixture | Every authored fact or byte-identical original source | Dispatcher + live |
| Nearby ILogger metadata plus 40 | Ten DI facts; logging present | All DI source plus original logger content | No DI contamination or unrelated loss | Live |
| Same service, different implementation/lifetime | Four distinct facts | Same four facts | Distinctions retained; existing ordering preserved | Focused test + live |
| Conditional source mentions | Both written branch registrations observed | Same written mentions | Preserve observed-source behavior only; active set unknown | Live |

### Legitimate source fallback, not a weakened count oracle

An intermediate candidate run (`20261012-032829-2ebc6521`) reported four harness
failures because the verifier counted only T @di rows while the response contained
the complete raw source. Inspection proved content_kind=raw_passthrough and
byte_exact includes document. The harness now accepts that existing representation
only when returned text equals the entire authored source exactly, then checks the
closed fixture's written-call count. Otherwise it requires the original exact
marker list and order. No fixture was changed to force GREEN and no production
renderer workaround was added. That intermediate result remains in its own run.

One attempted harness command used an incorrect pwsh path and exited 127 without
running any cases; it was corrected to the installed path before the final run.
Neither setup failure is described as production RED.

## 5. Public contract and what an agent receives

Compact delivery retains the existing `T @di = service → registration` facts.
The entire recognized normalized set is returned; no ten-entry subset remains.
When existing rendering chooses raw passthrough, the original C# source is returned
with the existing content_kind/byte_exact metadata. That source contains all the
authored registration calls, preserving source bytes and conditions as written.

No new runtime total, omitted count, complete boolean or parallel completeness
schema was introduced. In the forty-call oracle: visible recognized facts 40,
expected normalized written facts 40, omissions from this cap zero. These numbers
are verification conclusions, not invented response fields. Native degradation=null
is not promoted to a new claim that every other meta-layer is lossless.

**Important boundary:** the current extractor is pattern-based over captured class
text. This repair does not establish compiler-bound DI calls, an evaluated container,
unsupported registration forms, or every possible source occurrence/identity.

## 6. Fidelity and conditional source truth

**PROVEN BY TEST and LIVE MCP:** Medium and High satisfy all tested boundary cases.
They share the same producer collection. The live forty-call source selects exact
passthrough at both; the padded dispatcher fixture selects complete markers. Other
DI fidelities were not independently scored and are not inferred verified.

**PROVEN BY CODE INSPECTION and LIVE:** the existing producer has no build-symbol
evaluation/preprocessor engine. The conditional fixture's DEBUG and else branches
therefore both supply written mentions; their runtime activity is **NOT ESTABLISHED**.
The regression preserves that existing observation, without treating two written
mentions as two simultaneously active registrations or calculating omissions from
an invented active set. No new condition engine or activation/completeness promise
was added. Original conditional attribution/disclosure remains a separate #135
criterion requiring review; this slice removes producer retention loss only.

## 7. Executed preservation

- **PROVEN BY TEST:** six focused DI tests, including 0/9/10/11/40 and distinct
  service/implementation/lifetime content. Both relevant fidelities were exercised.
- **PROVEN BY TEST:** 51 existing dotnet_meta tests passed for general, ASP.NET,
  EF, testing and other currently registered .NET behavior. No unrelated cap changed.
- **PROVEN BY TEST:** frozen #136 insertion filter passed 9 tests; frozen #137
  signature evidence filter passed 5 tests. Their production files are unchanged.
- **PROVEN BY LIVE MCP:** 16 provider-disabled native context groups passed;
  exact marker identities/order or exact source delivery, plus nearby logger facts.
- **PROVEN BY CODE INSPECTION:** ordering remains AddScoped, AddSingleton,
  AddTransient, AddDbContext groups, source order within each group. This is not
  globally interleaved source order. Exact-source fallback retains actual source
  order by its existing byte-exact contract.

No CBM proxy/compression boundary is in this native context path; B4 was not reopened.
No full suite, Windows execution, model-host consumption or new broad fidelity
acceptance is claimed.

## 8. Remaining #135 work

- Static/public context signatures, return/attribute fidelity beyond this DI list.
- Records, nested/partial type shapes and their owner/kind/part links.
- Call ownership and ambiguity, expression-bodied-property calls and coverage.
- Context parameters and batch-level grammar/fidelity behavior.
- Declaration versus occurrence/query semantics.
- Conditional activation/branch provenance, unsupported or falsely recognized
  registration forms and registration semantic relations.
- Original real-repository/Windows acceptance and other untested criteria.

The #137 shared diff repair is not reused as a DI solution; no src/diff code changed.
The #136 file-identity anomaly remains separate. No GitHub issue was closed.

## 9. Other caps: classification only

Narrow related inspection confirms unchanged caps in general.rs (validation 10,
rules 5, logging 5), EF key fields 3, AutoMapper ignored/projection entries 5/3,
and serialization properties 10. Their individual intent/disclosure/public effects
are not fully established here. None was removed or repaired by this patch.

## 10. Changed files

| File | Responsibility |
| --- | --- |
| src/dotnet_meta/general.rs | Retain the complete existing recognized DI vector; remove only DI truncate(10) |
| src/mcp/mod.rs | Register focused permanent MCP DI regressions |
| src/tests/mcp/di_omission.rs | Independent authored counts, exact identities/order, fidelity and over-cap dispatcher assertions |
| verification/issue-reconciliation/Invoke-DiOmission.ps1 | User-runnable live native MCP matrix, source manifests, compact/exact-source oracles |
| verification/issue-reconciliation/README.md | Copy-paste command, delivered representation and scope limits |
| docs/plans/POST_CBM_133_141_NATIVE_DEFECT_RECONCILIATION.md | Link the approved follow-up without erasing historical evidence |
| This report | Root cause, complete-delivery decision, actual evidence and remaining scope |

## 11. Commands and artifacts actually executed

All Cargo commands ran in /workspace/Clean-CTX with the existing tooling:
PATH prepended by /workspace/.tooling/cargo/bin,
RUSTUP_HOME=/workspace/.tooling/rustup, CARGO_HOME=/workspace/.tooling/cargo.

| Executed command | Result |
| --- | --- |
| cargo test --locked -p clean-ctx --lib --all-features -j 4 di_omission_ | Initial invalid arrow oracle 2 pass/4 fail; corrected RED 4 pass/2 fail; unchanged GREEN 6/6 |
| cargo test --locked -p clean-ctx --lib --all-features -j 4 dotnet_meta:: | 51/51 |
| Same package test command, filter insert_before_declaration_ | 9/9 |
| Same package test command, filter semantic_signature_evidence_ | 5/5 |
| Focused rustfmt --edition 2024 --config skip_children=true on general.rs and new regression | Completed; assertions/source unchanged |
| cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings | Passed without warnings |
| cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4 | Single candidate build passed |
| PowerShell Invoke-DiOmission.ps1 against pre-fix binary | Valid RED: 10 passing/6 failing live groups |
| Same harness against candidate | Intermediate marker-only oracle identified exact source fallback; final 16/16 |
| Source manifest and stored RED response replay against complete-delivery oracle | Exact fixtures unchanged; same six original losses still RED |

Pre-fix binary SHA256:
`4c8e244888d18d7d7b2db55f060542d5d491b82452c9956b1cf769ddd62f52ed`.
Candidate SHA256:
`c1ca63a88a309d97ec3f7b6acc609b8117a6d5b76e8f97029e9f7fca37483692`.
Live runs use the installed workspace pwsh with -NoProfile and workspace XDG
settings. Every source oracle, request/result, compact/full-source observation,
stderr and per-case verdict remains under target/di-omission. Artifact syntax,
UTF-8/no-BOM, code-size and whitespace checks are recorded at completion.

Stop here. No other cap, #135 criterion or native production slice is authorized
by completion of this repair.
