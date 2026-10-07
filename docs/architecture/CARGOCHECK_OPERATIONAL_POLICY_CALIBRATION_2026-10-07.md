# CargoCheck Operational Policy Calibration

**Date:** 2026-10-07  
**Investigator:** Agent MaxHeadRoom  
**Status:** All nine operational-policy decisions approved by the owner on 2026-10-07; no remaining policy blocker.  
**Frozen architecture:** `CARGOCHECK_DIAGNOSTIC_VERTICAL_SLICE.md`  
**Production implementation:** Phases 1–2 (semantic compilation and authority admission) implemented, owner-verified, and committed through `ae03c243`. Phase 3 fixed invocation/environment projection is implemented with owner-run verification pending.
**Phase 1 verification:** Owner reported the focused CargoCheck parser suite GREEN on 2026-10-07.
**Phase 2 verification:** Owner reported the combined CargoCheck suite and zero-warning check GREEN, then pushed commit `ae03c243` on 2026-10-07.
**Cargo execution:** None.

## 1. Executive Recommendation

**POLICIES APPROVED — NO REMAINING POLICY BLOCKER**

The calibration found no contradiction in the frozen CargoCheck architecture. The owner approved all nine operational-policy decisions on 2026-10-07. All implementation-blocking policies now have concrete approved values, failure behavior, and disclosure requirements. The numeric policies occupy a broad stable region in the synthetic measurements rather than a one-byte cliff.

The recommended first-slice envelope is:

```text
stdout capture budget             8 MiB
stderr capture budget             4 MiB
aggregate capture budget         12 MiB
maximum input frame               2 MiB, excluding delimiter
semantic diagnostics             64
primary spans / diagnostic        4
related spans / diagnostic        8
children / diagnostic             8
suggestions / diagnostic          4
sanitized fallback evidence      96 KiB
structuredContent projection    512 KiB
content rendering                24 KiB and 240 lines
temporary spool                  none
cancellation                     immediate owned-boundary termination
intrinsic operation timeout      none
rustup automatic installation    disabled
```

These values are approved first-slice policies, not claims about universal Cargo output. Changing them later remains an explicit product-policy decision supported by new evidence; implementation may not silently substitute different values.

## 2. Authoritative Frozen Baseline

This calibration preserves the following unchanged:

```text
CargoCheckRequest { workspace: ApprovedWorkspaceRoot }
cargo_check {}
<approved-absolute-cargo> check --message-format=json
```

Operation identity precedes execution. Workspace and Cargo identity are approved before invocation. There is no shell, caller-controlled argument, model-selected workspace, model-selected environment, inherited executable search, merged stream, raw-result fallback, generic producer schema, or change to the existing PostToolUse compatibility path.

The OS process outcome remains authoritative. Cargo-specific semantic interpretation and parser coverage remain independent authorities. Boundedness applies while reading, not after unlimited accumulation.

## 3. Calibration Method

Calibration combined four evidence classes:

1. repository inspection of current ingress, filtering, result, and configuration limits;
2. prior architecture and falsification evidence;
3. authoritative Cargo, rustup, MCP, Windows, and Linux documentation;
4. deterministic non-Cargo synthetic measurements from:

```text
scripts/investigations/cargocheck-policy-calibration/measure.ps1
```

The fixture synthesizes line-delimited Cargo/rustc-like JSON and stderr without invoking Cargo. It measured:

| Case | Bytes | Frames | Largest frame |
|---|---:|---:|---:|
| Small clean check | 43 | 1 | 42 B |
| Five diagnostics + completion | 6,423 | 6 | 1,275 B |
| 500 diagnostics + completion | 638,043 | 501 | 1,275 B |
| One unusually long diagnostic | 1,057,557 | 1 | 1,057,556 B |
| Large stderr with decisive tail | 1,140,031 | 30,001 | 37 B |
| Large non-JSON stdout | 6,100,000 | 100,000 | 60 B |
| Malformed JSON flood | 4,700,000 | 100,000 | 46 B |
| Unknown-record flood | 5,580,000 | 10,000 | 557 B |
| Build-script record noise | 229,000 | 1,000 | 228 B |
| 1,000 repeated diagnostics | 1,276,000 | 1,000 | 1,275 B |
| Extreme single-line record | 21,006,280 | 1 | 21,006,279 B |

Boundary cases covered below, exactly at, one byte over, and far over the proposed frame limit; terminated and unterminated frames; invalid UTF-8 near the boundary; valid large JSON; and malformed large JSON.

This is a policy characterization, not a representative population study. No synthetic fit is labeled PROVEN for all repositories.

## 4. Evidence Sources

- **PROVEN repository behavior:** MCP JSON-RPC input and Claude-hook input are each capped at 16 MiB. That is evidence that bounded ingress is established practice, but these boundaries carry different representations and threats and do not supply the CargoCheck numbers.
- **PROVEN repository behavior:** the compatibility Cargo text filter retains at most 100 lines with head/tail mechanics. This supports tail-aware disclosure, not a 100-diagnostic semantic limit.
- **OBSERVED:** prior semantic compilation preserved decision-relevant Cargo facts more compactly than raw output within its bounded fixture.
- **OBSERVED:** prior secret-boundary work sanitized both MCP result surfaces and failed closed.
- **OBSERVED:** the calibration fixture produced the size distribution in §3.
- **DOCUMENTED:** Cargo emits one JSON object per line on stdout with `--message-format=json`, while compiler/build tools may still emit non-JSON data ([Cargo external tools](https://doc.rust-lang.org/cargo/reference/external-tools.html)).
- **DOCUMENTED:** `RUSTC`, `RUSTC_WRAPPER`, and `RUSTC_WORKSPACE_WRAPPER` alter compiler execution; `RUSTFLAGS` and `CARGO_ENCODED_RUSTFLAGS` alter compiler arguments ([Cargo environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html)).
- **DOCUMENTED:** Cargo configuration and environment can alter target directory, target, compiler, wrappers, flags, registries, and networking ([Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html)).
- **DOCUMENTED:** rustup toolchain selection considers command override, `RUSTUP_TOOLCHAIN`, directory override, `rust-toolchain.toml`, and default toolchain ([rustup overrides](https://rust-lang.github.io/rustup/overrides.html)).
- **DOCUMENTED:** `RUSTUP_AUTO_INSTALL=0` disables rustup's automatic installation of a missing active toolchain ([rustup environment variables](https://rust-lang.github.io/rustup/devel/environment-variables.html)).
- **DOCUMENTED:** Windows console control delivery depends on console/process-group membership, while `TerminateJobObject` terminates Job members ([console process groups](https://learn.microsoft.com/en-us/windows/console/console-process-groups), [Job APIs](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/)).
- **DOCUMENTED:** Linux `kill` can address a process group; `SIGKILL` cannot be caught or ignored ([kill(2)](https://man7.org/linux/man-pages/man2/kill.2.html), [signal(7)](https://man7.org/linux/man-pages/man7/signal.7.html)).

No live Claude work, MCP selection test, real Cargo run, completed falsifier rerun, or production implementation was performed.

## 5. Capture Budget

Capture budget means bytes admitted to framing/parsing. After a budget activates, the implementation must continue draining the pipe into a fixed scratch buffer so the process cannot deadlock, while discarding excess and counting it. “Discarded” is not “unread.”

### stdout

**RECOMMENDATION — 8 MiB.**

**Classification:** EVIDENCE-BACKED RECOMMENDATION.

**EVIDENCE:** The 8 MiB candidate completely admitted every synthetic stdout case except the deliberate 21 MiB single-line record. It admitted 500 diagnostics (638,043 B), 1,000 repeated diagnostics (1,276,000 B), a 1.06 MiB complex diagnostic, 6.1 MiB non-JSON noise, 4.7 MiB malformed-record noise, and 5.58 MiB unknown-record noise. The 2 MiB lower candidate admitted the semantic-heavy cases but cut all three multi-megabyte anomaly floods. Sixteen MiB added no value for those cases and still did not admit the extreme line.

**TRADEOFF:** Eight MiB is larger than ordinary useful semantic output but bounds parser work and permits anomaly distribution/counting. It is not a promise to retain eight MiB in the result. A lower value is safer but makes coverage characterization stop earlier; a higher value spends memory/CPU on evidence that the result will not expose.

**FAILURE BEHAVIOR:** At 8 MiB, stop admitting new stdout bytes to frames, finish/discard the partial frame safely, continue draining/counting, set `stdout.capture_limit_activated=true`, and report admitted/discarded byte facts. Existing parsed diagnostics remain valid but result completeness is reduced.

**DISCLOSURE REQUIREMENT:** admitted bytes, discarded bytes, complete frames, over-limit partial-frame state, and whether producer completion evidence may have been beyond the limit.

**CONFIDENCE:** Medium. Synthetic sensitivity is stable; real-repository distribution remains unmeasured.

**OWNER DECISION:** APPROVED — 8 MiB; no prerequisite real-output measurement.

### stderr

**RECOMMENDATION — 4 MiB.**

**Classification:** EVIDENCE-BACKED RECOMMENDATION.

**EVIDENCE:** The 1.14 MiB / 30,001-line stderr-tail fixture fit entirely. Unlike stdout, stderr has no expected semantic JSON payload; its value is bounded fallback evidence, especially the tail. One MiB would cut that case just before its decisive final line under naïve capture, demonstrating why continued drain plus independent tail retention is required. Four MiB leaves more than 3.5× headroom. Eight or 16 MiB primarily admits more unstructured noise.

**TRADEOFF:** Four MiB preserves substantial failure characterization without treating stderr as a transcript. Tail retention, not total admitted bytes, protects the terminal cause.

**FAILURE BEHAVIOR:** On activation, continue draining/counting while maintaining bounded category/head/tail summaries. Do not terminate Cargo solely because stderr exceeded its admission budget.

**DISCLOSURE REQUIREMENT:** same facts as stdout, separately reported, plus retained head/tail selection facts.

**CONFIDENCE:** Medium.

**OWNER DECISION:** APPROVED — 4 MiB.

### aggregate

**RECOMMENDATION — 12 MiB, with the 8 MiB/4 MiB stream caps remaining independently authoritative.**

**Classification:** DESIGN RECOMMENDATION.

**EVIDENCE:** Twelve MiB is the arithmetic sum of the recommended independent caps. A smaller shared cap introduces scheduling-dependent behavior: whichever concurrent pipe is read first can consume budget needed by the other. A larger aggregate cap has no effect while both stream caps remain enforced.

**TRADEOFF:** The sum avoids cross-stream starvation but allows both streams to reach their maxima. Memory still need not approach 12 MiB because admitted frames can be parsed and released incrementally.

**FAILURE BEHAVIOR:** If aggregate accounting activates through implementation overhead or future policy changes, both pipes continue draining; the result records which stream's admissions were stopped and why.

**DISCLOSURE REQUIREMENT:** aggregate admitted/discarded bytes plus both per-stream facts.

**CONFIDENCE:** High as a consistency rule, not as an empirical resource optimum.

**OWNER DECISION:** APPROVED — 12 MiB non-competing sum policy.

### sensitivity analysis

| Candidate | Useful effect | Failure/expense |
|---:|---|---|
| 2 MiB stdout | Fits 500 ordinary synthetic diagnostics and the 1.06 MiB long diagnostic | Stops detailed characterization of 4.7–6.1 MiB anomaly floods |
| **8 MiB stdout** | Fits every multi-frame case | Rejects only deliberately extreme 21 MiB line; bounded parser work |
| 16 MiB stdout | No additional measured case | Doubles admitted attack/noise budget; still rejects extreme line |
| 1 MiB stderr | Nearly fits tail fixture | Can activate before late cause; relies heavily on tail sampler |
| **4 MiB stderr** | Fits tail fixture with broad headroom | More unstructured bytes admitted than final result needs |
| 8 MiB stderr | No additional measured value | Doubles noise budget |

## 6. Frame-Length Budget

**RECOMMENDATION — 2 MiB excluding the newline delimiter.**

**Classification:** EVIDENCE-BACKED RECOMMENDATION.

**EVIDENCE:** The 1,057,556-byte long synthetic diagnostic failed the 512 KiB lower candidate but fit under 2 MiB. The 8 MiB higher candidate added no measured semantic case and would permit a single record to consume the entire stdout budget. The 21,006,279-byte extreme record failed all candidates and establishes the need for incremental framing.

Boundary behavior:

| Input | Required result |
|---|---|
| 2 MiB − 1 | Accept as parse candidate |
| exactly 2 MiB | Accept as parse candidate |
| 2 MiB + 1 | Mark over-limit; reject frame; drain to delimiter |
| far over | Same fixed-memory drain behavior |
| EOF below limit without newline | Mark truncated; retain sanitized bounded evidence; do not parse as complete Cargo record |
| EOF over limit | Mark over-limit and truncated; no parse |
| invalid UTF-8 below/at limit | Record decoding fault; sanitize bounded replacement evidence; do not silently claim full parse |
| valid large JSON under limit | Parse normally |
| malformed huge JSON over limit | Never allocate it; reject/drain/count |

**TRADEOFF:** A valid diagnostic above 2 MiB becomes coverage evidence rather than semantics. Raising the limit improves an uncommon record at a direct per-frame allocation/parse-cost increase.

**FAILURE BEHAVIOR:** The frame is rejected, not truncated and parsed. Store only a fixed-size sanitized prefix/suffix fingerprint/evidence after delimiter/EOF; increment over-limit and malformed/truncated categories. Redaction failure still withholds it.

**DISCLOSURE REQUIREMENT:** original observed length when known, at-least length when budget stopped exact accounting, rejection reason, termination state, and no-diagnostic-compilation fact.

**CONFIDENCE:** Medium.

**OWNER DECISION:** APPROVED — 2 MiB.

## 7. Semantic Retention Budgets

### diagnostics

**RECOMMENDATION — retain 64 semantic diagnostics.**

**Classification:** DESIGN RECOMMENDATION informed by measurement.

**EVIDENCE:** The representative uncompressed record was 1,276 B; 64 such records were about 81,664 B before the CargoCheck projection eliminates transport fields and bounds nested data. Thirty-two retains 6.4% of a 500-diagnostic flood; 64 retains 12.8%; 128 retains 25.6% while doubling semantic result pressure. Compiler cascades make exhaustive retention less valuable than causal head coverage.

**TRADEOFF:** Sixty-four is not enough for bulk remediation of every warning. It is enough for a next coding decision plus distribution/totals. Raising it competes directly with model context.

**FAILURE BEHAVIOR:** Continue counting by severity/code/file and maintain a bounded selection; do not stop parsing merely because retention is full.

**DISCLOSURE REQUIREMENT:** total seen, retained, omitted, severity counts, selection policy, and whether exact-repeat compaction occurred.

**CONFIDENCE:** Medium-low; this is primarily product-density judgment.

**OWNER DECISION:** APPROVED — 64.

### spans

**RECOMMENDATION — 4 primary and 8 related spans per retained diagnostic.**

**Classification:** DESIGN RECOMMENDATION.

**EVIDENCE:** Rust diagnostics commonly have one primary site; macro/type errors can have several. The synthetic long record used 16 spans to stress size but did not establish that all 16 were needed for the next action.

**TRADEOFF:** Caps can omit macro expansion context. Keeping distinct totals and prioritized primary spans preserves truth.

**FAILURE BEHAVIOR:** Preserve primary spans first in producer order, then related spans; count omitted spans. Never relabel a related span as primary.

**DISCLOSURE REQUIREMENT:** primary/related seen, retained, omitted per diagnostic.

**CONFIDENCE:** Low-medium.

**OWNER DECISION:** APPROVED — 4 primary / 8 related.

### children

**RECOMMENDATION — 8 child diagnostics per retained diagnostic.**

**Classification:** DESIGN RECOMMENDATION.

**EVIDENCE:** Children carry notes/help and can explain a fix. The long fixture used 16 to show growth. Eight preserves several explanatory branches without allowing nested help to dominate.

**TRADEOFF:** Later notes may be omitted. A head/tail split is preferable when more than eight exist.

**FAILURE BEHAVIOR:** retain first six and last two children, preserve order within retained positions, count omissions.

**DISCLOSURE REQUIREMENT:** seen/retained/omitted and head/tail strategy.

**CONFIDENCE:** Low-medium.

**OWNER DECISION:** APPROVED — 8.

### suggestions

**RECOMMENDATION — 4 distinct suggestions per retained diagnostic.**

**Classification:** DESIGN RECOMMENDATION.

**EVIDENCE:** Suggestions directly inform edits but replacement text can be large. Four supports alternatives and multipart advice while bounding it. No repository evidence establishes a superior count.

**TRADEOFF:** Diagnostics with many alternatives lose some candidates. Applicability and omission facts prevent false completeness.

**FAILURE BEHAVIOR:** retain machine-applicable suggestions first, then producer order; count omitted suggestions and replacement bytes.

**DISCLOSURE REQUIREMENT:** applicability counts and omission facts.

**CONFIDENCE:** Low.

**OWNER DECISION:** APPROVED — 4.

### severity policy

**RECOMMENDATION — errors first with causal head and terminal tail; warnings fill remaining capacity; notes/help remain children rather than top-level competition.**

When errors exceed 64, retain the first 48 and last 16 in producer order. When errors use fewer than 64 slots, warnings fill remaining slots using the same 3:1 head/tail ratio. Exact semantic repeats may collapse into one retained diagnostic with a repeat count, but diagnostics at different locations are not exact repeats.

**Classification:** DESIGN RECOMMENDATION.

**EVIDENCE:** Early compiler errors are often causal; late output may contain a different terminal cause. Existing Clean-CTX filters already support head/tail rather than first-only retention.

**TRADEOFF:** Severity prioritization may omit an actionable warning in an error-heavy run. This is appropriate for the next-fix objective and must be disclosed.

**FAILURE BEHAVIOR:** selection never changes severity or process authority; totals remain counted.

**DISCLOSURE REQUIREMENT:** quotas actually used, head/tail counts, repeats collapsed, and omitted severity distribution.

**CONFIDENCE:** Medium.

**OWNER DECISION:** APPROVED — severity-aware causal head/terminal tail selection.

## 8. Sanitized Evidence Budget

**RECOMMENDATION — 96 KiB total sanitized fallback evidence, reserved as follows:**

| Category | Budget | Selection |
|---|---:|---|
| stderr | 48 KiB | 16 KiB head + 32 KiB tail |
| stdout non-JSON | 16 KiB | equal head/tail |
| malformed/truncated/decoding anomalies | 16 KiB | per-category exemplars plus head/tail |
| unknown/incompatible structured records | 8 KiB | sanitized summaries, not full objects |
| authority/completion mismatch evidence | 8 KiB | reserved; unused bytes may be borrowed only after all categories are represented |

**Classification:** DESIGN RECOMMENDATION informed by measurement.

**EVIDENCE:** The 1.14 MiB stderr case showed that the decisive cause can be last. The anomaly floods showed that retaining proportional raw text is not useful; examples plus exact counts explain the failure class. Ninety-six KiB is materially below the semantic and capture budgets while allowing each category to remain visible.

**TRADEOFF:** Fixed reservations can leave unused capacity while another category truncates. Borrowing unused capacity improves density but must not erase category minimums.

**FAILURE BEHAVIOR:** Keep counting after budgets fill. Evidence omission does not invalidate already parsed diagnostics; redaction failure invalidates all producer-derived evidence and returns the fail-closed processing result.

**DISCLOSURE REQUIREMENT:** for every category: original count/bytes, retained count/bytes, omitted count/bytes, head/tail/exemplar strategy, borrowed bytes, and limit activation.

**CONFIDENCE:** Medium-low.

**OWNER DECISION:** APPROVED — 96 KiB and stated category allocation.

## 9. Final Result Budgets

### structuredContent

**RECOMMENDATION — 512 KiB UTF-8 serialized maximum.**

**Classification:** DESIGN RECOMMENDATION informed by projection.

**EVIDENCE:** Sixty-four untrimmed representative input records totaled roughly 82 KiB. A CargoCheck projection drops package/target transport duplication but may add coverage, authority, and evidence facts. A 512 KiB cap leaves substantial headroom without approaching the existing unrelated 16 MiB MCP request-line ceiling.

**TRADEOFF:** The serializer needs deterministic reduction before emitting, adding a final safety pass. This cap should rarely activate if nested budgets work.

**FAILURE BEHAVIOR:** Authority and process facts are never removed. Reduce optional rendered diagnostics, evidence, child details, related spans, suggestions, then lower-priority diagnostics in that order until within budget. If mandatory authority plus disclosure cannot fit, return an internal bounded-result failure rather than an oversized result.

**DISCLOSURE REQUIREMENT:** serialized bytes, final-budget activation, fields reduced, and pre-reduction counts.

**CONFIDENCE:** Medium.

**OWNER DECISION:** APPROVED — 512 KiB.

### content

**RECOMMENDATION — 24 KiB UTF-8 and 240 lines, whichever activates first.**

**Classification:** DESIGN RECOMMENDATION.

**EVIDENCE:** Current compatibility Cargo rendering caps at 100 lines, but semantic direct-MCP output includes authority and coverage disclosures as well as diagnostics. Twenty-four KiB is approximately 6,000 tokens only under a rough four-bytes-per-token heuristic; bytes are the enforceable policy, not token claims.

**TRADEOFF:** A concise surface improves direct consumption but may omit diagnostics that remain in `structuredContent`. Hosts that ignore structured data receive less detail.

**FAILURE BEHAVIOR:** Always render verdict/process authority, incompleteness, first causal diagnostics, and a pointer-like statement that more bounded structured diagnostics exist. Never silently change authority facts.

**DISCLOSURE REQUIREMENT:** content truncation statement and diagnostic counts represented versus available.

**CONFIDENCE:** Low-medium pending the final Claude rendering pilot.

**OWNER DECISION:** APPROVED — 24 KiB/240 lines; the final pilot may supply evidence for a later explicit revision.

The core owns a single bounded semantic result. Adapters may impose stricter presentation limits, but may not mutate the core or silently omit authority.

## 10. Temporary Spool Decision

**RECOMMENDATION — no spool in slice 1.**

**Classification:** EVIDENCE-BACKED RECOMMENDATION.

**EVIDENCE:** Every tested policy decision can be implemented with incremental framing, bounded semantic state, category counters, and bounded head/tail evidence. A spool does not make an over-limit frame safely parseable and does not improve the final bounded representation. Prior secrecy work shows the value of keeping raw evidence inside a non-persistent boundary.

**TRADEOFF:** An over-limit or later-parser-bug record cannot be replayed from disk. That is preferable to creating raw secret persistence, permissions, crash-residue, cleanup, and platform lifecycle obligations without a first-slice correctness need.

**FAILURE BEHAVIOR:** Excess is drained and counted, not persisted. Parser failures retain only bounded sanitized evidence.

**DISCLOSURE REQUIREMENT:** `spool_used=false`; capture-limit and evidence omissions remain explicit.

**CONFIDENCE:** High.

**OWNER DECISION:** APPROVED — no spool.

## 11. Cancellation Policy

**RECOMMENDATION — immediate forced termination of the owned boundary; no graceful interval in slice 1.**

**Classification:** EVIDENCE-BACKED DESIGN RECOMMENDATION.

**EVIDENCE:** Cargo exposes no special cancellation protocol in the evidence reviewed. Windows console control requires shared-console/process-group conditions that are separate from the frozen suspended-process/Job ownership contract. `TerminateJobObject` directly terminates Job members. Linux can signal a process group, and `SIGKILL` cannot be caught or ignored. Prior process falsification established forced cleanup on both platforms.

**TRADEOFF:** Immediate termination does not give Cargo/build tools an opportunity to perform voluntary cleanup. A graceful phase would introduce platform asymmetry, console coupling on Windows, a new interval, and uncertain descendant cooperation without measured first-slice benefit.

**FAILURE BEHAVIOR:** Windows calls Job termination; Linux sends `SIGKILL` to the owned group. Drain/close remaining pipes, verify quiescence, and report `Cancelled` with forced termination and cleanup status. Cancellation is not producer failure or timeout.

**DISCLOSURE REQUIREMENT:** cancellation source, forced mechanism, request time, observed root outcome, descendant quiescence, and incomplete/uncertain cleanup.

**CONFIDENCE:** Medium-high.

**OWNER DECISION:** APPROVED — immediate forced cancellation; any future graceful phase requires separate evidence.

## 12. Timeout Policy

**RECOMMENDATION — no intrinsic CargoCheck timeout in slice 1.**

**Classification:** EVIDENCE-BACKED RECOMMENDATION.

**EVIDENCE:** Legitimate duration varies with repository size, cold caches, dependency retrieval, build scripts, proc macros, machine load, and toolchain state. No real-run distribution exists from which to derive a duration. Host/user cancellation remains available and uses the owned-boundary cleanup contract.

**TRADEOFF:** A hung build does not self-terminate. An arbitrary timeout would falsely fail large or cold checks and turn a guess into product behavior.

**FAILURE BEHAVIOR:** None specific; the operation remains active until completion or explicit host/user cancellation. External host deadlines are reported as host cancellation when observable, not internal timeout.

**DISCLOSURE REQUIREMENT:** `intrinsic_timeout=false`; cancellation source must remain distinct.

**CONFIDENCE:** High.

**OWNER DECISION:** APPROVED — no intrinsic timeout.

## 13. Environment Policy

Environment inheritance is execution authority. The recommended mode constructs a fresh environment from an explicit category policy captured once at operation start; it does not call an unexamined full-inheritance spawn default.

### runtime

| Variable/category | Policy | Reason |
|---|---|---|
| `PATH` | INHERIT WITH SANITIZED FACT | Cargo/rustc/build tools and linkers may resolve subprocesses; it never resolves the approved Cargo executable |
| `HOME`, `USERPROFILE`, Windows home components | INHERIT WITH SANITIZED FACT | Cargo/rustup default homes and user config |
| `CARGO_HOME`, `RUSTUP_HOME` | INHERIT WITH SANITIZED FACT | Existing caches/config/toolchains; values rendered using path policy |
| `SystemRoot`, `WINDIR`, `COMSPEC`, platform loader/runtime variables | INHERIT | Required platform operation |
| MSVC/SDK discovery variables (`INCLUDE`, `LIB`, `LIBPATH`, installation roots) | INHERIT WITH SANITIZED FACT | Required by approved Windows toolchains |
| `LANG`, `LC_*` | OVERRIDE where supported | Stable UTF-8/nonlocalized machine evidence; exact platform values implemented/tested per OS |
| terminal/color variables | OVERRIDE | Noninteractive/no-color behavior where documented; presentation only |
| `CI` | STRIP | Avoid changing build-script behavior merely because Clean-CTX host is CI |
| arbitrary unrelated variables | STRIP by default | Avoid accidental contract and secret exposure |

**RECOMMENDATION:** use this minimal platform/toolchain runtime set; unsupported projects requiring extra variables are explicit first-slice limitations rather than implicit full-environment inheritance.

**Classification:** OWNER PREFERENCE REQUIRED.

**EVIDENCE:** Cargo executes build scripts/proc macros and platform tools with environment dependencies. A universal tiny allowlist is incompatible with some native toolchains, while full inheritance exposes unrelated secrets and changes behavior.

**TRADEOFF:** Security/determinism improve; projects relying on custom `CC`, SDK, pkg-config, or domain variables may fail until a separately approved operator-forwarding mechanism exists.

**FAILURE BEHAVIOR:** Do not retry with the full environment. Return Cargo's sanitized failure plus an environment-policy fact.

**DISCLOSURE REQUIREMENT:** policy version, inherited/stripped/overridden variable names or categories, never values; path/toolchain facts separately sanitized.

**CONFIDENCE:** Medium on authority, low on universal compatibility.

**OWNER DECISION:** APPROVED STRICT — minimal explicit environment; unsupported environment-dependent projects fail truthfully. Operator-controlled forwarding is deferred pending real-world evidence.

### toolchain selection

| Variable | Policy |
|---|---|
| `RUSTUP_TOOLCHAIN` | INHERIT WITH SANITIZED FACT |
| `RUSTUP_HOME` | INHERIT WITH SANITIZED FACT |
| `RUSTUP_AUTO_INSTALL` | OVERRIDE to `0` |
| rustup server/update/log/debug overrides | STRIP unless separately approved |
| `CARGO_HOME` | INHERIT WITH SANITIZED FACT |

`RUSTUP_TOOLCHAIN` is operator authority and outranks workspace toolchain files. Its presence and sanitized logical selector must be recorded. An absolute custom-toolchain selector is rejected in slice 1 unless separately admitted as an operator-approved toolchain path; a named selector is allowed. Directory overrides and `rust-toolchain.toml` remain rustup/workspace authority and are disclosed by source where observable without Clean-CTX becoming a toolchain resolver.

### compiler wrappers

| Variable | Policy |
|---|---|
| `RUSTC` | STRIP |
| `RUSTDOC` | STRIP |
| `RUSTC_WRAPPER` | OVERRIDE to empty |
| `RUSTC_WORKSPACE_WRAPPER` | OVERRIDE to empty |
| `RUSTFLAGS`, `RUSTDOCFLAGS` | STRIP |
| `CARGO_ENCODED_RUSTFLAGS`, encoded rustdoc flags | STRIP |
| `CARGO_BUILD_TARGET`, target-triple env overrides | STRIP |
| `CARGO_TARGET_DIR` | STRIP |

Cargo documents that empty wrapper variables reset configured wrappers. This prevents environment and repository Cargo config from injecting those wrappers. Repository configuration can still influence compiler/target/linker behavior in ways not fully neutralized by these variables; that is disclosed workspace configuration under the frozen threat model, not proof of sandboxing.

**Classification:** EVIDENCE-BACKED RECOMMENDATION for wrappers/flags; OWNER PREFERENCE REQUIRED for compatibility.

**FAILURE BEHAVIOR:** no automatic relaxation/retry. Sanitized Cargo failure identifies the policy mode.

**DISCLOSURE REQUIREMENT:** names overridden/stripped and presence facts only.

### network

**RECOMMENDATION:** permit Cargo's ordinary dependency network behavior but prohibit implicit rustup toolchain installation with `RUSTUP_AUTO_INSTALL=0`. Do not claim network isolation. Inherit `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, and `NO_PROXY` only if the owner chooses compatibility mode; record presence, never values. Minimal mode strips them.

**Classification:** OWNER PREFERENCE REQUIRED.

Cargo check commonly needs missing dependencies. Forcing offline mode changes ordinary semantics and the frozen invocation. Allowing dependency network does not require allowing rustup to install a missing toolchain.

**FAILURE BEHAVIOR:** a missing toolchain or unavailable dependency yields a sanitized producer failure; Clean-CTX does not install or retry.

**DISCLOSURE REQUIREMENT:** network mode, proxy-variable presence only, and auto-install disabled.

### credentials

**RECOMMENDATION:** strip credential-shaped environment variables, including Cargo registry token variables, from the child environment. Existing Cargo credential files under approved `CARGO_HOME` remain usable according to Cargo behavior; Clean-CTX does not read or report them. Private setups requiring environment tokens are outside minimal mode.

**Classification:** OWNER PREFERENCE REQUIRED.

**TRADEOFF:** less secret exposure to build scripts, but some private dependency checks fail.

**FAILURE BEHAVIOR:** no credential prompt, fallback, logging, or full-env retry.

**DISCLOSURE REQUIREMENT:** count/names stripped, never values or value hashes.

### temporary directories

`TMP`, `TEMP`, and `TMPDIR` are inherited with sanitized presence/path-category facts because compilers and build tools require temporary storage. Clean-CTX creates no raw spool. It does not claim control over temporary files created by Cargo/toolchain/build scripts.

## 14. Rustup Policy

**RECOMMENDATION:** accept an approved rustup Cargo proxy; inherit named `RUSTUP_TOOLCHAIN`; disable auto-install; record selector-source facts; do not query, install, update, or manage toolchains.

**Classification:** EVIDENCE-BACKED RECOMMENDATION.

**EVIDENCE:** Rustup documents its selection precedence and automatic-install switch. The approved proxy identity alone does not identify the final toolchain.

**TRADEOFF:** Disabling automatic installation makes a first run fail when the selected toolchain is absent, but prevents an ostensibly diagnostic operation from becoming a toolchain-management/network mutation.

**FAILURE BEHAVIOR:** missing selected toolchain returns sanitized Cargo/rustup failure. No retry or installation.

**DISCLOSURE REQUIREMENT:** proxy/non-proxy identity, named environment selector if present, nearest observed `rust-toolchain`/`.toml` file identity, directory-override influence if legitimately observable, auto-install false, and no claim that these facts reconstruct every rustup internal decision.

**CONFIDENCE:** High for documented behavior; medium for complete source observability.

**OWNER DECISION:** APPROVED — named rustup override allowed and automatic installation disabled.

## 15. CLI Exit-Code Contract

**RECOMMENDATION:**

| Code | Meaning |
|---:|---|
| `0` | Cargo completed with exit code 0 and cleanup complete |
| `1` | Cargo completed nonzero; Cargo's actual code remains in the result |
| `2` | Clean-CTX authority/admission/configuration failure before start |
| `3` | Clean-CTX internal capture/parse/redaction/result failure |
| `4` | Cancellation requested/observed |
| `5` | Descendant cleanup incomplete or uncertain, regardless of root outcome |

**Classification:** DESIGN RECOMMENDATION.

**EVIDENCE:** The frozen outcome model requires the distinctions; no external standard dictates these numbers. A small stable range is easier than propagating arbitrary Cargo codes, signals, and platform statuses as the Clean-CTX process exit.

**TRADEOFF:** Scripts cannot use the Clean-CTX exit code as Cargo's exact exit code. They can consume structured/text result facts.

**FAILURE BEHAVIOR:** precedence is cleanup (`5`) > cancellation (`4`) > internal (`3`) > admission (`2`) > Cargo nonzero (`1`) > success (`0`). The result records all applicable facts so precedence loses no semantics.

**DISCLOSURE REQUIREMENT:** always render the named outcome and actual Cargo process status when available.

**CONFIDENCE:** High as a coherent design, not empirical evidence.

**OWNER DECISION:** APPROVED — numeric exit-code contract 0–5.

## 16. Path Exposure Policy

**RECOMMENDATION:** resolve by canonical absolute identity internally; render by workspace-relative position externally.

| Path category | Core authority | MCP/CLI display |
|---|---|---|
| Workspace root | Canonical absolute | logical label `<workspace>` plus stable non-secret identity; absolute only in explicitly local verbose CLI mode if later approved |
| Cargo executable | Canonical absolute + identity metadata | basename + approved-source + redacted parent category; no full user-profile path in MCP |
| Diagnostic file inside workspace | Canonicalized/lexically validated against root as appropriate | normalized workspace-relative path, preserving editability |
| Related span inside workspace | Same | normalized workspace-relative path |
| Diagnostic path outside workspace | Preserve internal fact | `<external>/<basename>` plus `outside_workspace=true`; do not expose host prefix |
| Artifact/out directory | Internal bounded fact only | omit by default; summarize inside/outside workspace and basename when relevant |
| Toolchain/sysroot path | Internal/sanitized | logical `<toolchain>/...` or basename; no home prefix |

Do not require target files to exist before displaying a compiler location; generated or deleted paths can still be truthful. Reject `..`-style display escape and normalize separators to `/` for logical paths.

**Classification:** DESIGN RECOMMENDATION.

**EVIDENCE:** Diagnostic locations must identify editable files, while full workspace/Cargo paths reveal unrelated host structure and may contain usernames. The frozen principle explicitly separates identity from display.

**TRADEOFF:** External generated-source locations may be less directly navigable. CLI users may want full paths; that is a later explicit local-display option, not MCP default.

**FAILURE BEHAVIOR:** If a path cannot be safely relativized/classified, render a sanitized basename and classification rather than the absolute path.

**DISCLOSURE REQUIREMENT:** `inside_workspace`, `external`, `generated`, or `redacted` path classification.

**CONFIDENCE:** High.

**OWNER DECISION:** APPROVED — no absolute paths in default MCP results.

## 17. Configuration Source and Precedence

**RECOMMENDATION:** use explicit process-start authority, not repository config or discovery heuristics.

### MCP server

```text
approved Cargo path:
  1. explicit server startup option --cargo-path <absolute>
  2. CLEAN_CTX_CARGO_PATH set by the host/operator
  3. otherwise CargoCheck tool is unavailable with an admission explanation

approved workspace root:
  1. explicit server startup option --workspace-root <absolute>
  2. CLEAN_CTX_PROJECT_ROOT set by the host/operator
  3. otherwise CargoCheck tool is unavailable with an admission explanation
```

For these execution authorities, there is no fallback to `.clean-ctx.json`, CWD walk-up, executable-directory walk-up, bare `cargo`, `PATH`, additional roots, or a model argument. Existing project-root discovery may continue serving non-execution configuration but does not grant CargoCheck authority.

### CLI operation

```text
workspace:
  1. required cargo-check --workspace-root <absolute>
  2. no fallback

Cargo path:
  1. cargo-check --cargo-path <absolute>
  2. CLEAN_CTX_CARGO_PATH
  3. otherwise admission failure
```

CLI selection is local operator authority. Environment sources are captured once at startup; later mutation cannot change an admitted request. Conflicting explicit sources are resolved by the precedence above and disclosed.

**Classification:** DESIGN RECOMMENDATION.

**EVIDENCE:** `CLEAN_CTX_PROJECT_ROOT` already exists and has highest current project-root precedence, while current CWD/executable heuristics are explicitly insufficient execution authority. No existing setting safely carries an absolute approved Cargo path.

**TRADEOFF:** Adding narrow startup options/environment variable is new configuration surface. It avoids placing executable authority in repository-controlled `.clean-ctx.json` or building a general config framework.

**FAILURE BEHAVIOR:** Missing, relative, nonexistent, changed, or conflicting authority prevents CargoCheck from starting; other MCP tools may continue if their existing contracts permit.

**DISCLOSURE REQUIREMENT:** winning source name, shadowed-source presence, canonical identity, and admission status—never hidden fallback.

**CONFIDENCE:** High.

**OWNER DECISION:** APPROVED — startup option, then environment, then failure; no heuristic authority fallback.

## 18. Recommended Policy Table

| Policy | Recommendation | Classification | Evidence | Tradeoff | Owner approval? |
|---|---|---|---|---|---:|
| stdout capture | 8 MiB | Evidence-backed | Synthetic sensitivity | More parser work than 2 MiB | Yes |
| stderr capture | 4 MiB | Evidence-backed | 1.14 MiB tail case | More noise admitted | Yes |
| aggregate | 12 MiB, non-competing | Design | Avoids scheduling starvation | Both streams may max | Yes |
| frame | 2 MiB | Evidence-backed | 1.06 MiB useful vs 21 MiB extreme | Oversized valid record loses semantics | Yes |
| diagnostics | 64, causal head/tail | Design | Density projection/cascade behavior | Omits bulk diagnostics | Yes |
| spans | 4 primary + 8 related | Design | Common semantic shape | Macro context may omit | Yes |
| children | 8, 6 head + 2 tail | Design | Explanation value vs growth | Later notes may omit | Yes |
| suggestions | 4 | Design | Edit value vs replacement growth | Alternatives may omit | Yes |
| evidence | 96 KiB category-aware | Design | Tail/anomaly fixtures | Allocation complexity | Yes |
| structured result | 512 KiB | Design | ~82 KiB representative projection | Final reducer required | Yes |
| text content | 24 KiB / 240 lines | Design | Existing 100-line precedent + semantic needs | Host may ignore richer structured data | Yes |
| spool | None | Evidence-backed | No correctness need found | No replay | Yes |
| cancellation | Immediate forced boundary | Evidence-backed design | Platform docs/falsifier | No graceful cleanup | Yes |
| timeout | None | Evidence-backed | No duration distribution | Hung work requires cancellation | Yes |
| environment | Explicit minimal set | Owner preference | Authority/compatibility conflict | Some projects fail | Yes |
| rustup | Named override allowed; auto-install off | Evidence-backed | Rustup docs | Missing toolchain fails | Yes |
| network | Dependency network permitted; mode disclosed | Owner preference | Normal Cargo needs | Not a sandbox | Yes |
| credentials | Strip env credentials | Owner preference | Secret exposure boundary | Private registries may fail | Yes |
| CLI exits | 0–5 stable contract | Design | Outcome distinctions | Cargo code not propagated | Yes |
| paths | Relative/logical display | Design | Editability + privacy | External navigation reduced | Yes |
| configuration | Explicit startup > env; no fallback | Design | Existing config audit | New narrow options | Yes |

## 19. Sensitivity Summary

| Policy | Lower | Recommended | Higher | Stable-region conclusion |
|---|---:|---:|---:|---|
| stdout | 2 MiB | 8 MiB | 16 MiB | 8 MiB admits all multi-frame cases; 16 adds none |
| stderr | 1 MiB | 4 MiB | 8 MiB | 4 MiB safely clears measured tail case |
| frame | 512 KiB | 2 MiB | 8 MiB | 2 MiB admits 1.06 MiB long record; 8 adds none |
| diagnostics | 32 | 64 | 128 | Linear result cost; 64 balances causal breadth |
| evidence | 48 KiB | 96 KiB | 192 KiB | 96 KiB gives meaningful per-category reservations |
| structured | 256 KiB | 512 KiB | 1 MiB | Representative projection is far below 512 KiB |
| content | 12 KiB | 24 KiB | 48 KiB | Final Claude pilot should test model-visible density |

The numeric recommendation is least certain for `content`, because actual host rendering remains deliberately deferred. The core/structured budgets are not dependent on that pilot.

## 20. Architecture Escalations

**No frozen assumption was contradicted.**

Environment calibration confirmed—not created—that Cargo execution includes toolchain, wrapper, linker, network, credential, build-script, and proc-macro authority. The frozen architecture already required an explicit environment policy and excluded hostile approved producers/repositories from its protection claim. The recommended minimal environment keeps this operational.

Return for architecture review if implementation discovers that:

- a supported project requires arbitrary model/caller environment or arguments;
- Cargo or rustup cannot operate without repository-controlled substitution outside the disclosed workspace authority;
- a useful structured result cannot fit the approved bounds;
- a real diagnostic above 2 MiB is both common and decision-critical;
- persistent raw spooling becomes necessary;
- platform cleanup cannot uphold the existing process contract;
- environment forwarding must become an externally model-selectable API;
- Cargo config neutralization is claimed as sandboxing rather than disclosed workspace authority.

## 21. Owner Decisions Required

**None.** Matt approved all nine decisions on 2026-10-07. This section is retained as the authoritative approval record.

1. **APPROVED — Capture and frame envelope**

   **Recommended:** 8 MiB stdout, 4 MiB stderr, 12 MiB aggregate, 2 MiB frame.  
   **Alternative:** require real Cargo measurements before choosing.  
   **Consequence:** Lower caps reduce resource exposure but characterize less anomalous output; higher caps add no synthetic semantic value.

2. **APPROVED — Semantic and evidence density**

   **Recommended:** 64 diagnostics; 4/8 spans; 8 children; 4 suggestions; 96 KiB evidence; 512 KiB structured; 24 KiB/240-line text.  
   **Alternative:** approve different explicit values after reviewing §7–§9.  
   **Consequence:** Higher values consume more model/client context; lower values omit more decision evidence.

3. **APPROVED — Persistence and cancellation**

   **Recommended:** no spool, immediate forced owned-boundary cancellation, no grace interval.  
   **Alternative:** request a separately evidenced graceful phase or spool design.  
   **Consequence:** Recommended policy minimizes secret persistence and lifecycle complexity but gives Cargo no voluntary cleanup window.

4. **APPROVED — Timeout**

   **Recommended:** no intrinsic CargoCheck timeout; host/user cancellation only.  
   **Alternative:** require owner-run duration measurements and approve a timeout.  
   **Consequence:** No false timeout on large/cold repositories; hung work requires explicit cancellation.

5. **APPROVED STRICT — Environment compatibility posture**

   **Approved:** explicit minimal platform/toolchain environment; strip compiler/flag/target overrides and unrelated variables; no automatic full-environment retry. Unsupported environment-dependent projects fail truthfully. Operator-controlled forwarding is deferred unless real-world evidence establishes the need.  
   **Alternative:** filtered broad inheritance with its larger authority and secret-exposure surface.  
   **Consequence:** Minimal mode is more deterministic and safer but will reject some environment-dependent projects.

6. **APPROVED — Rustup, network, and credentials**

   **Recommended:** allow named `RUSTUP_TOOLCHAIN`, set `RUSTUP_AUTO_INSTALL=0`, permit ordinary dependency network, strip environment credentials, and disclose proxy mode.  
   **Alternative:** offline dependency mode or compatibility mode forwarding proxies/tokens.  
   **Consequence:** Recommended mode may fail for missing toolchains or environment-token private registries but prevents silent installation and reduces secret exposure.

7. **APPROVED — CLI exit codes**

   **Recommended:** approve codes 0–5 from §15.  
   **Alternative:** collapse cancellation/cleanup/internal failures into a smaller contract.  
   **Consequence:** Recommended codes preserve automation distinctions without mirroring platform/Cargo codes.

8. **APPROVED — Path display**

   **Recommended:** workspace-relative diagnostic paths and redacted logical authority paths; no absolute MCP paths.  
   **Alternative:** expose absolute paths to all adapters.  
   **Consequence:** Recommended policy preserves editability while avoiding unrelated host path disclosure.

9. **APPROVED — Authority configuration**

   **Approved:** explicit startup option over environment variable, followed by failure; no repository/CWD/PATH or other heuristic authority fallback; CLI requires an explicit workspace.  
   **Alternative:** environment-only authority.  
   **Consequence:** Startup options are easier to audit and override deliberately; both options require new narrow configuration wiring.

## 22. Implementation Readiness

**READY — NO REMAINING POLICY BLOCKER**

The owner accepted the stated confidence levels, synthetic calibration, and explicit limitations. No owner-run Cargo measurement is mandatory before Phase 1. Real-output measurements remain valuable validation and may tune the policy later through a deliberate policy change.

Implementation must not silently choose different values, widen environment inheritance, add a timeout/grace interval/spool, or treat a limit activation as success without disclosure.

## 23. Next Step

Recommend **CargoCheck Vertical Slice Implementation — Phase 1** as the next separately authorized task.

This document records readiness; it does not itself start or authorize implementation in the current task.
