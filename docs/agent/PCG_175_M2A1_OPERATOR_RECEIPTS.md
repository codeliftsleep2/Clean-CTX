# PCG-175-M2A1-OPERATOR-RECEIPTS

Agent: **Agent MaxHeadRoom**

Date: 2026-11-10 (America/New_York).
Authority: owner-approved single-case selection and immediate phase receipt
retention in the existing PowerShell driver, plus owner verification handoff.
No new tests, production changes, scaling execution, broad gates or M2a-2.

Disposition (owner-directed, 2026-11-11): **PCG-175-M2A1 ACCEPTANCE READY — M2a-1
closed**. The successful single-case lifecycle remains directly corroborated.
The supplied Clippy GREEN (47.31s) and successful build excerpt (2m 07s) remain
owner-reported evidence. The later owner rebuild/hash response now closes the
source/binary attribution gap, as recorded at the end of this report. All requested
evidence items are reconciled; the owner's subsequent closure supersedes the
earlier pending/blocked dispositions without changing their historical evidence.
No executable verification was launched by the agent.
M2a-2 remains unauthorized.

Owner disposition following receipt review: the operator acceptance blocker is
**provisionally lifted**. The driver, test definitions and operator campaign remain
closed; the historical blocked verdict in the acceptance report is not grounds
to reopen them. Final acceptance bookkeeping is restricted to the final-source
library Clippy outcome and attribution of the executed binary to the identified
production source. Final attribution remains subject to the owner's disposition
below. No architectural or persistence correction is indicated.

## Changes

Only `verification/pcg-175/run-cli.ps1`, its README and this mission report changed
during this mission. Earlier acceptance inventories remain historical snapshots.

1. `-SingleCase` selects exactly `BaseRecords`. With `-BaseRecords 1`, the driver
   schedules one one-record fixture, not 1/2/4 or 8/16/32. Scaling mode remains
   unchanged when the switch is absent; it is not part of this handoff.
2. The existing Git/CLI helper path now retains every native phase independently.
   `ProcessStartInfo.ArgumentList` preserves argument boundaries, invokes the
   resolved executable directly and redirects both streams. Both streams drain
   concurrently to avoid pipe deadlock. Each completed/failed native call writes
   stdout/stderr and its JSON receipt before returning or propagating failure.

Per-case receipt layout under `target/pcg-175/m2a1-<run>/n-1/phases/`:

```text
git-init.{stdout.txt,stderr.txt,receipt.json}
git-config.{stdout.txt,stderr.txt,receipt.json}
git-add.{stdout.txt,stderr.txt,receipt.json}
git-commit.{stdout.txt,stderr.txt,receipt.json}
git-head.{stdout.txt,stderr.txt,receipt.json}
git-blob.{stdout.txt,stderr.txt,receipt.json}
cli-import.{stdout.txt,stderr.txt,receipt.json}
cli-reimport.{stdout.txt,stderr.txt,receipt.json}
cli-rebuild.{stdout.txt,stderr.txt,receipt.json}
cli-audit.{stdout.txt,stderr.txt,receipt.json}
```

Each receipt contains phase, executable and literal arguments, working directory,
whether the process started, actual exit code, status, parsed result when
available, error, output-file paths, duration and PCG binary hash. Launch failure
has no invented native exit code. A successful child with invalid expected JSON
is recorded with exit code zero but failed result handling. A nonzero native exit
retains its output and failed status; no successful structured result is invented.

The run-level `operator-receipt.json` is retained on normal success or exception.
It reports planned populations, single-case mode, overall status/error, binary
identity and completed case reports. Earlier phase receipts remain if a later
phase or cross-phase assertion fails. No files are automatically cleaned up.
These are ordinary file writes, not a crash-durable logging contract against
forced driver termination or storage failure.

## Revision and static review

Branch: `(fix)Performance-Regression-Issues`.
HEAD: `8ce0a6865ff8a2b55866895c6a239f6bcc3245a3`.
HEAD tree: `10d8ecadd0a151ea0638e2a066bd50b40ad022e0`.
No branch/ref/index/commit/publication action occurred.

Unchanged production fingerprint:
`664065f6c9e530c4eeb391d433dd5915e48244b96c0daa277574af9444ac46c6`.
Definition: SHA-256 over ordinal-path-sorted lines containing lowercase file
SHA-256, two spaces, slash path and LF, including final LF. Scope is `src/lib.rs`,
`src/main.rs` and the 12 files directly under `src/pcg/`, including `schema.sql`.
This is a source identity witness, not proof of which source built an executable.

Corrected driver SHA-256:
`abcac45786f23c0350719e17b480fad9b3f9bc101f0228639cb54e118d34c09b`.

Bounded inspection: PowerShell AST parsing, argument/phase construction review,
exact source hashes, UTF-8/BOM/whitespace and source-preservation inspection.
AST inspection is not runtime success. No Git fixture commands, binary, builds,
Clippy, tests, scripts, benchmarks or servers were launched during this mission.

## Original bounded handoff and current evidence status

This table retains the original command references for attribution. It is not a
new request to execute them. The owner controls execution and timing. No command
was executed by the agent; the operator command now has directly inspected
owner-run evidence and must not be labelled unexecuted.

| Original command reference | Purpose / scope | Current evidence status |
| --- | --- | --- |
| `cargo clippy --package clean-ctx --lib --all-features -- -D warnings` | Current library lint, including the two previously corrected expressions; no tests, examples, binary or proxy targets. | **OWNER-REPORTED GREEN**, with supplied `Checking clean-ctx` and successful `Finished dev profile` excerpt; reported duration 47.31s. Scope attributed to the targeted owner question, not independently reconstructed from the excerpt. |
| `cargo build --package clean-ctx --bin clean-ctx --all-features` | Build the real CLI against the unchanged implementation. | **OWNER-REPORTED GREEN**, with `Compiling clean-ctx` and successful finish excerpt, reported duration 2m 07s. Executable bytes independently match the operator receipt; explicit attribution of that build to the exact source/binary pair remains pending. |
| `pwsh -NoProfile -ExecutionPolicy Bypass ./verification/pcg-175/run-cli.ps1 -Binary ./target/debug/clean-ctx.exe -BaseRecords 1 -SingleCase` | One record/source; import, idempotent reimport, transactional rebuild and read-only audit. | **DIRECTLY VERIFIED** from run `m2a1-20261110-193539-9838`: all four CLI phases exited zero, with the expected operations and identical generation. Operator work is closed; no repeat campaign is requested. |

No new test definitions or Rust test rerun are proposed for these driver-only
changes. The two prior Rust edits were behavior-preserving lint corrections;
their missing post-fix test timestamp is not evidence of a regression.

## Acceptance boundary

A successful one-case run establishes only that fixture's actual CLI lifecycle,
generation equality/idempotency, rebuild and audit observations. It does not
independently establish every M2a-1 security, concurrency or crash invariant.
Those retain the separately attributed, previously owner-reported focused-test
evidence and its historical source boundary.

No broad suite is required for this bounded decision. The supplied final-source
lint/build attribution and directly inspected CLI receipt support the bounded
acceptance recommendation. No CI-wide GREEN, performance improvement, production
enrollment or later-stage authorization is inferred.

## Owner-supplied operator result

**DIRECTLY VERIFIED — artifact-backed single-case CLI lifecycle.**

Run: `target/pcg-175/m2a1-20261110-193539-9838/`.
Summary: `operator-receipt.json`, SHA-256
`c5cc7e1bda1ad0741641d7537dc2d44c533821ec03b119e74b0158947a3c0599`.
Its recorded completion is `2026-11-10T19:35:46.9135465Z` (14:35:46 EST).
This is an artifact timestamp, not an independently observed execution time.
The owner supplied the path; the agent inspected existing JSON, text outputs and
byte hashes without rerunning the driver, binary, Git fixture commands or audit.

The summary reports `status=succeeded`, `single_case=true`, `planned_records=[1]`
and exactly one completed one-record case. All four phase receipts have
`started=true`, `exit_code=0`, `status=succeeded`, no error, and retained stdout/
stderr files. The stdout JSON agrees with each receipt's operation and generation.
All four retained CLI stderr files are empty.

| Phase | Operation | Exit | Recorded process duration (ms) |
| --- | --- | --- | --- |
| `cli-import` | `import` | 0 | 1656.247 |
| `cli-reimport` | `idempotent_import` | 0 | 789.8998 |
| `cli-rebuild` | `rebuild` | 0 | 1974.0936 |
| `cli-audit` | `audit` | 0 | 389.9003 |

Identical generation across all four:
`49a7d51b4b380e8c19186c62078cc2175797f9efd329dc9e147b2403a6bd3082`.
Each phase reports one distinct current source at each observation, 201 Git/source
bytes, 402 working-file bytes across both boundaries, one evidence validation and
2769 total accounted bytes. These are measurements of one tiny functional fixture,
not statistical performance or enterprise-efficiency evidence.

The retained profile's approved raw manifest digest matches its manifest bytes;
the current fixture document matches the enrolled source SHA-256; the manifest
contains exactly one record. All four phase receipts agree on PCG binary SHA-256
`b570600f03d0f3c3a19839dfce60eb06d3dd527d5a6a6e9753bd60cab023ce19`,
which also matches the executable currently present at `target/debug/clean-ctx.exe`.
The current driver still matches the corrected hash recorded above. The successful
driver summary corroborates its generation/work and main-DB audit nonmutation
checks; the agent did not perform another SQLite audit or reconstruct the earlier
pre-audit database bytes independently.

This closes the lack of a completed operator artifact and demonstrates receipt
retention for successful phases. A failing phase was not executed in this run;
failure-path retention remains statically inspected, not newly fault-injected.
Security, concurrency and crash invariants remain supported separately by the
previously owner-reported focused tests, not by this one fixture.

At the initial receipt review, attribution limits were: the receipt identifies the actual executed binary
but does not record which source built it. No final-source library Clippy result
or build output/source binding accompanied the supplied receipt. Those remain
unreported evidence items at that time, not failed checks or instructions to execute anything.
Final acceptance and any further execution remain owner decisions. No M2a-2 or
publication is authorized by this receipt.

### Final attribution follow-up

The production fingerprint remains
`664065f6c9e530c4eeb391d433dd5915e48244b96c0daa277574af9444ac46c6`;
the current executable still matches the successful run's binary hash
`b570600f03d0f3c3a19839dfce60eb06d3dd527d5a6a6e9753bd60cab023ce19`.
These were rechecked by bounded byte hashing, without executable verification.
The owner then answered the explicit final-source library-Clippy and binary/source
question with “clippy was green” and excerpts showing `Checking clean-ctx v0.10.0`
followed by successful dev-profile completion in 47.31s, then `Compiling clean-ctx
v0.10.0` followed by successful dev-profile completion in 2m 07s. Record this as
**OWNER-REPORTED GREEN** for lint/build. The earlier assistant classification of
**OWNER-CONFIRMED ATTRIBUTION** inferred more than the owner subsequently accepted;
that classification is withdrawn. No command echo, numeric exit
code, execution timestamp or cryptographic build attestation was supplied; none
is invented. This is attributable owner evidence, not an agent-executed or fully
independently verified build/lint result.

Bounded hashing at closure again confirmed both exact source and binary hashes
above. The binary matches all four successful operator phase receipts. These
observations establish source and executable identities separately; they do not
themselves prove the build relationship. Existing build evidence or an explicit
owner attribution can resolve that relationship without repeating the CLI run.

**Current owner disposition: M2A1 EVIDENCE PENDING — operator gate satisfied.**
The focused tests remain owner-reported historical evidence; the one-case CLI
lifecycle is artifact-backed; the supplied lint/build excerpts are retained with
their actual attribution limits. Final acceptance awaits the owner's resolution
of final-source lint scope and build attribution. No existing GREEN is erased,
no new test or operator campaign is requested, and no driver or implementation
is reopened. Broad unrun checks are not failures. M2a-2 requires separate authority.

### Rebuild attribution resolved — 2026-11-11

The owner elected to rebuild from the unchanged M2a-1 worktree. In response to
the agreed package/binary build and SHA-256 handoff, the owner supplied executable
hash `B570600F03D0F3C3A19839DFCE60EB06D3DD527D5A6A6E9753BD60CAB023CE19`.
Record the rebuild/source attribution as owner-reported evidence; no numeric
build exit code, new build duration or complete build transcript was supplied.

Bounded byte hashing independently confirms that the current production fingerprint
is still `664065f6c9e530c4eeb391d433dd5915e48244b96c0daa277574af9444ac46c6`
and the executable matches the owner's hash and all four successful operator
phase receipts. This resolves the previously requested attribution through the
owner's rebuild response plus unchanged-source and matching-binary observations.
It is not an agent-executed build or a cryptographic build attestation.

Final-source lint is already owner-reported GREEN; the successful single-case CLI
lifecycle is artifact-backed. No requested evidence item remains outstanding.
At that checkpoint the remaining gate was the owner's M2a-1 acceptance decision.
The owner subsequently directed M2a-1 closure on November 11 with disposition
ACCEPTANCE READY, without another execution. That gate is now closed.
No tests, driver changes, operator campaign, broad checks or M2a-2 are requested
or started. Separate M2a-2 authorization remains necessary.

```architecture-graph
node: PCG-175-M2A1-OPERATOR-RECEIPTS
kind: RESEARCH
status: IMPLEMENTED
title: M2a-1 bounded operator and owner-reported evidence reconciliation
track: REPOSITORY
artifact: docs/agent/PCG_175_M2A1_OPERATOR_RECEIPTS.md
note: Source record and its attributed disposition only; no new execution, authenticated approval, independent certification, reclosure or banking.
```
