# OPS-R1 — Disposition 17 preservation failure investigation

Agent MaxHeadRoom — 2026-11-20. Investigation and unapplied proposal only.

## Finding and standing

Classification: **test defect**, supported by the fixture, grouping implementation,
path semantics and retained execution. Original subcase 4 is **A: equivalent to
subcase 2**, rather than a fourth independent authority context. No production
patch is proposed. This finding does not certify the other sharing boundaries.

The invariant should be: distinct acquisition authority contexts cannot share;
equivalent contexts may share only after complete receipt/currentness and
recipient-source qualification. The original aggregate expectation instead
requires independence for four constructed inputs, one of which changes only
an interior `.` spelling.

The original tracked test remains failed and unchanged. Eight preceding controls
passed; this ninth control failed; sixteen controls were not executed. There was
no watchdog/resource stop, retry or new execution during this investigation.

## Four-subcase authority comparison

Let R be the fixture's supplied root, P its canonical physical root, and E a
second independent temporary root. Actual temporary directory names and alias
payloads were not retained by this native control; these are symbolic identities,
not reconstructed runtime paths. The fixture creates only a.ts, b.ts and
tsconfig.json. Sources are `class A { peer?: Z; }` and `class Z {}`.

| Dimension | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| Exact constructed request | R.join("a.ts") | R.join("b.ts") | R.join("a.ts") | R.join(".").join("b.ts") |
| Logical route after lexical normalization | R/a.ts | R/b.ts | R/a.ts | R/b.ts |
| Physical source | P/a.ts | P/b.ts | P/a.ts | P/b.ts |
| Selected config | P/tsconfig.json before mutation | Same file after mutation | Same mutated file | Same mutated file |
| Project root/base | P/P | P/P | P/P | P/P |
| Authorized host roots | [P] | [P] | [P, canonical(E)] | [P] |
| Root route aliases | [(R,P)] | [(R,P)] | [(R,P),(E,canonical(E))] | [(R,P)] |
| Configured strict option | Absent | true | true | true |
| Request parent | R | R | R | R, after Path parent component normalization |
| Own supplied source | a.ts disk bytes seeded | b.ts disk bytes seeded | a.ts disk bytes seeded | b.ts disk bytes seeded |
| Other source acquisition | Disk | Disk | Disk | Qualified proof from 2, with recipient rebinding |
| Demand policy | controls | controls | controls | controls |
| Independent acquisition justification | First context | Changed effective options/config bytes | Different authorized roots/aliases | None: equivalent to 2 |
| Decision attributable from static path | New group/leader | Different group/leader | Different group/leader | Existing group, successful qualification/reuse |

All configurations explicitly specify module=esnext, moduleResolution=bundler,
moduleDetection=legacy, noLib=true, types=[] and include=["*.ts"]. Only strict is
added between cases 1 and 2. Each effective option retains the same declaring
directory P; there is no extends, paths, baseUrl, rootDirs or project reference.
Both TypeScript files are root members. Ownership is discovered, not explicitly
selected; the same config is its sole configuration/root candidate. The extra
authorized root in case 3 does not change the selected project's root files.
Unspecified compiler defaults are not invented here: the loaded options map,
compiler identity, resolution mode, conditions and extensions are key fields;
the fixture does not change those other inputs between cases 2 and 4.

Every resolver is fresh and privately owned; none has inherited project context.
Supplied bytes are read from disk at seeding, but a supplied owner is not thereby
disk authority for another owner. Qualification separately preserves this rule.
The process working directory is not changed by the fixture; the request-directory
key is the request file's parent, not the process working directory.

## Source anchors and decision path

- `src/tests/ir/ops_r1_shared_acquisition_isolation.rs:8`: exact four subcases;
  line 35 is the unchanged failing assertion.
- `src/tests/ir/ops_r1_shared_acquisition_support.rs:12`: fixture bytes and config;
  line 24 creates a private resolver, sets controls demand, enables witnesses and
  seeds the owner's exact disk bytes; line 34 requires two returned units.
- `src/ir/typescript_resolution/host.rs:159`: source seeding; line 189 establishes
  canonical roots and route aliases; lines 262/267 normalize logical/physical
  routes; line 495 removes CurDir components in lexical normalization.
- `src/ir/typescript_resolution/resolver.rs:132`: normalized project selection,
  cached selection receipt attribution, and explicit-versus-discovered dispatch.
- `src/ir/typescript_resolution/project.rs:30`: discovered project; line 77 loads
  config and option origin context. `project_config.rs:9` defines option value
  plus origin directory. `project_membership.rs:19` and its selection routine
  establish configuration candidates and root membership.
- `src/ir/typescript_resolution/shared_acquisition.rs:150`: provisional key:
  selected config, project root/base, mode, paths, conditions, effective options
  with origins, ownership, compiler, extensions, request parent/extension,
  authorized roots/aliases and demand policy. Supplied-input membership is
  required. Owner filename alone is intentionally not an independence boundary.
- `src/ir/typescript_resolution/program.rs:121`: project selection receipts are
  captured before acquisition. A top-level operation slot is selected only after
  the recursion guard. Existing proofs are qualified outside the slot lock.
  On success, bound inputs/witnesses and immutable units/evidence are attached to
  this recipient. Otherwise acquisition executes and only successful, completely
  accounted, current evidence can seal a previously empty slot.
- `shared_acquisition.rs:173` seals source evidence; line 201 qualifies recipients:
  reject incompatible private units, conflicting cached bytes, unaccounted inputs,
  different supplied bytes, missing source support and noncurrent witnesses;
  independently read leader-supplied dependencies when the recipient needs disk
  authority. Role-bound inputs are substituted into the receipt, preserving
  dependencies rather than importing a leader's owner authority indiscriminately.

The local Rust standard-library source was inspected, without running Rust:
`C:/Users/MNasty/.rustup/toolchains/stable-x86_64-pc-windows-msvc/lib/rustlib/src/rust/library/std/src/path.rs`.
Lines 22–34 document component/comparison normalization of interior dots;
line 2617 implements parent through components. Thus BTreeMap supplied-input
membership and request parent do not create a new authority just for `./b.ts`.
This local library inspection supports interpretation; it is not claimed as a
new compiler execution or as a pin of the historical toolchain itself.

The first three group keys differ through options or host roots/aliases. The
fourth key matches the second. Its leader is therefore the second acquisition,
whose b.ts supplied bytes match the fourth recipient's own seed. The other unit
remains backed by disk evidence. No source/config mutation occurs between cases
2 and 4. Equivalent routing plus successful receipt qualification permits reuse.

**Recorded versus inferred:** retained stderr records acquisition=[3,3,6] and
qualification=[1,1,157,54,59]: one attempted qualification, one accepted reuse,
157 seal checks, 54 recipient witness checks and 59 qualification input visits.
These are operation aggregates. Exact serialized keys, per-case reasons,
individual receipt DAGs and physical temporary names were not emitted. Mapping
that one reuse to case 4 follows the deterministic source path above, not a
nonexistent per-case trace. There is no logged refusal to invent for cases 1–3;
their distinct keys select fresh slots rather than reject a candidate proof.

## Preserved failure evidence

Original directory:
`target/ops-r1/p0-config_roots_and_route_changes_stay_independent-20261120-190651/`.
Bundle record: `target/ops-r1/preservation-bundle-20261120-190637/bundle-review.json`.
The investigation index hashes all retained files from all nine executed cases
and the bundle record; none was overwritten or relabeled.

- Test identity: `ir::typescript_resolution::ops_r1_shared_acquisition_controls::isolation::ops_r1_shared_config_roots_and_route_changes_stay_independent`.
- Actual [3,3,6]; expected [4,4,8]; Rust exit 101; one selected test, one failure.
- Source tree: `91ff7766ca9f0aaa965734748c6aab8e6b186dd5da8f0d6a36e2dfb16d11efba`.
- Executable: `d03b03ce7232ebf69b985f19d4a11db1b363d265b1b51a41302c85092ad08d66`.
- PID 17352; elapsed 0.10551470000064 seconds. The single working-set sample was
  2,449,408 bytes; sampling is too sparse to characterize peak memory or leaks.

## Separate proposed test correction

Unapplied proposal:
`verification/ops-r1/proposals/shared-acquisition/review-17/test-correction.patch`.
It changes only the failing tracked test and its import. It explicitly proves
case 4's physical/logical and provisional-basis equivalence to case 2, requires
one successful qualification and no new acquisition, then adds a fifth request
after changing strict=true to strict=false in the same authoritative config.
It asserts the same selected config, different loaded strict value and different
basis, then requires an independent acquisition without another qualified reuse.
Five requests therefore establish four authority contexts and retain final
[4,4,8]. Intermediate checkpoints localize every transition.

Deterministic proposed predicate: equivalent dot request leaves [3,3,6] and
increments accepted qualification once; changed effective-option request moves
to [4,4,8], returns two units and adds no accepted qualification. An implementation
that independently acquires every equivalent request fails the positive reuse
predicate. Reuse across the changed options fails the negative acquisition
predicate. No post-correction RED/GREEN or fault-injection detection is claimed:
the proposal has not been compiled or executed. Existing historical architectural
RED and GREEN retain their original source/execution identities.

Original test raw SHA-256:
`7767841034a7d890f3ee696df399d5d4b101ca2e8963b0201a0e2a974b388db5`.
Proposed LF test SHA-256:
`7f87059494f87b780067875bf71e35de17417df6c988bdceff74764dba8ca582`.
Patch SHA-256:
`7a4f68c8491e3d4edbf76d99ca50a2e7d18bb230e33c958cbde6f128660129de`.
Proposed test is 222 lines. `git apply --check` succeeded; patch not applied.
No production patch is justified by this failure.

## Boundaries and next decision

No production or test files changed. No tests, diagnostics, builds, runtime,
remaining controls, CI, commits, pushes or publication were executed. Only this
report, the proposed patch and its investigation index were authored. Current
source-tree identity was recomputed and matches the pinned GREEN source.

PCG preflight coverage is unavailable: the configured tool catalog exposes no
pcg_knowledge_v1. Relevant source authority and owner disposition were read;
this is not graph acquisition/admission. This report and its proposal require
governing OPS-001 association and actual applicable admission before publication;
neither protected source declarations nor authority grants were changed here.

Next owner decision: approve or reject this exact test-only patch and separately
authorize any resulting compilation/execution. Production corrections remain
separate. Sixteen unexecuted controls remain unverified.
