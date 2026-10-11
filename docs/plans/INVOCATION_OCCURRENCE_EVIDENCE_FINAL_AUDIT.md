# Invocation occurrence boundary: final production audit

Date: 2026-10-11. Branch: `(feat)Semantic_Intelligence_Architecture_Boundaries`.
Scope: approved TypeScript/C# written evidence, generic consumer proof, RxJS pipe
structure, production projections, and operator-run MCP verification.

## Findings fixed

1. **UTF-8 summary panic.** `rx/pipes.rs::parse_operator` sliced byte 37 without
   checking a character boundary. A long argument with é at bytes 36–38 panicked.
   The tracked `audit_invocation_unicode_operator_summary_keeps_utf8_boundaries`
   regression observed that exact runtime panic before the fix and passed
   unchanged afterward. The fix rounds the existing 37-byte cutoff down to a
   UTF-8 boundary; the byte budget and ASCII behavior remain the same.
2. **Missing High-fidelity operator details in production.** The first live MCP
   run failed its high-structure assertion: canonical `TYPE/@pipeRx` values held
   only `stream$ = pipe(`, despite the domain renderer producing ordered operator
   details. `MetaBlock` split the marker into physical lines, and MetaLayerPass
   discarded all continuation lines lacking a Φ prefix. A second tracked audit
   regression observed the missing canonical arguments before the fix and passed
   unchanged afterward. RxShape now constructs complete marker entries once;
   standalone rendering and structured sections share those entries. Production
   uses structured Angular entries instead of splitting rendered text again.

The existing Angular **marker** producer generation increments from 1 to 2
because identical inputs now produce corrected durable markers. This follows
`layers/meta/mod.rs`'s independent marker/semantic generation rule. Angular
semantic generation stays 2; language call generations and Binary0x04 schemas
are unchanged. This is an audit repair to marker delivery, not a new public
occurrence model or new compatibility policy.

## Production trace and coverage

| Boundary | Production authority | Evidence |
| --- | --- | --- |
| File read and language selection | MCP compile helpers → IRCompiler::compile_focused | Live compression, context delivery and owner-aware calls for TS/C# |
| Single syntax parse | CoreIRPass combined capture walk | Existing TS/C# and production RxJS parse-count tests |
| Structural assembly | invoke.* matches → collect_invocations | Root merge, generic args, ordered whole arguments, Named/Spread qualifiers, immediate parent and declaration tests |
| Source ownership | compiler clears side output, publishes on success; identity/SHA-256 validated view | 11 consumer/lifecycle tests including populated edit, empty replacement and failure |
| Meta propagation | MetaLayerPass → borrowed view → Angular precomputed RxShape | Production same-parse test and live spaced/commented pipe fixture |
| Domain interpretation | RxJS terminal selection, checked operator slices, domain parser and owner policy | 36 RxJS tests and live operator/owner/fidelity/configuration cases |
| Canonical marker delivery | complete RxShape marker entries → MetaBlock → TypeAlias → named IR/MCP | New RED/GREEN handoff regression; live High ordered arguments now present |
| Canonical calls | existing CallProducer → fresh calls_in_file canonical query | Live TS spread/repetition; C# named count/generics/nesting/overloads; interfaces and same-name owners in separate files |
| Model-facing delivery | existing compression text/named projection, provided text/_meta, mirrored query text/structured result | Live response captures; mirrored call results compared exactly |
| Angular testing preservation | testing meta markers and Tests semantic edges | Live Karma/Jasmine and Vitest/TestBed source fixtures, vi.fn and component linkage |
| Source freshness | current source recompilation and refresh_workspace | Live edit, empty method/source, deletion error, restoration and verbatim equality |
| Durable compatibility | existing marker generation manifest | Marker generation 2; no public schema or binary format change |

No missing production handoff remains in the audited invocation/RxJS slice after
these repairs. Exact byte spans are intentionally private: exposing them via a
new MCP schema would exceed this architecture. The verification package uses
existing live tools and does not claim direct wire observation of private spans.

## Executed verification

- Audit regressions: 2 passed, with observed RED/GREEN for each finding.
- RxJS module: 36 passed.
- Invocation evidence/consumer/lifecycle module: 11 passed.
- Package Clippy, all targets/features, warnings denied: passed.
- Current all-features production binary build: passed.
- PowerShell live MCP harness: **12 cases passed**, 0 failures, against that
  binary in an isolated workspace. Initial 10-case run failed on the real High
  handoff gap; after repair the final harness also covers both Angular testing
  source patterns. Requests and responses are saved per unique run under target.

Final formatting, file-size and UTF-8 checks are recorded with the completion
commit. No full repository suite was run: it remains CI-only. The live harness
ran on Linux with PowerShell 7; Windows execution of this same script remains
operator verification. Neither a model-host pilot nor actual Karma/Vitest
runner execution is claimed.

## Remaining scope boundaries

- Textual RxJS owner policy and operator-name interpretation remain domain-owned;
  standalone compatibility parsing uses the same structural collector.
- The compilation-local cloned vector used to bind a meta view remains an
  allocation cost. No benchmark or allocation-free claim is made.
- Import/receiver resolution, action binding, framework semantics in the shared
  types, public occurrence fields, and Java/Rust evidence adapters remain deferred.
- Full CI and Windows operator results remain separate from this local audit.

Operator entry point: `verification/invocation-evidence/README.md`.

## Expanded MCP boundary acceptance

The original 12-case run was a smoke check, not exhaustive phase acceptance.
The expanded operator package has 59 cases grouped across phases 0–7, with a
T1–T24 coverage matrix in `verification/invocation-evidence/COVERAGE.md`. It
records existing verification ownership instead of duplicating general codec,
batch transport, identity, DI and template contracts.

Added observations include rich canonical calls in both languages, bound-arrow
ownership, private-to-public negative controls, same-length edits and recovery,
Rx import/terminal/nesting policies, all context delivery routes, mixed-file and
mixed-layer questions, same-source Angular/NgRx/Rx composition, and corrected
multiline Rx markers surviving durable save/restart/restore. Internal contracts
remain enforced by tracked tests; no new private fields or tool schema was added.
The phase selector reports partial coverage explicitly. Full CI and Windows
operator verification remain separate gates.

Final expanded live run: **59 PASS, 0 FAIL**, PowerShell 7 on Linux, current
production binary. Phase counts: 0=2, 1=2, 2=9, 3=8, 4=3, 5=13, 6=5, 7=17.
Evidence: `target/invocation-evidence-verification/20261011-152913-a47bccda/captures/summary.json`.
Selected Phase 6 and Phase 7 runs also exercised independent selection. File-size
and strict UTF-8 guards passed. This expansion changed only verification assets
and documentation; no production code or Rust tests changed or were rerun.
