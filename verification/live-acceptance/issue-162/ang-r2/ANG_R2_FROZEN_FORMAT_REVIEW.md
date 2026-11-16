# ANG-R2 frozen-source formatting decision

Agent: **Agent MaxHeadRoom**. Proposed formatting only; neither frozen source has
been changed. No exception, approval, CI success or new test execution is inferred.

## Observed conflict

Owner disposition 02 section 4 requires: "Preserve the exact existing behavioral
RED witness, its source, command, output and baseline identity" and "Do not weaken
its assertions." The campaign has preserved both primary witness files byte-for-byte.

The actual operator format check in
`target/ang-r2/prepublication-20261116-101602-601` reports formatting differences
in both files. They have already passed their corresponding exact-source GREEN
witnesses. Formatting changes their current bytes, so it must not silently replace
the exact-source preservation convention. No rustfmt skip or gate bypass is proposed.

The same capture passed the new exact registration replay-adoption test: exit 0,
1 passed / 0 failed, no compiler warnings, 272.8515432 seconds enclosing. Format
failure does not erase that result or establish a Rust test failure.

## Concrete proposal

Apply only the formatter-produced line wrapping, spacing and trailing-comma changes
to the two tracked tests; retain their exact original bytes and RED/GREEN receipts;
keep their assertions, fixture strings and required behaviors intact. Then hand
off focused verification for the normalized tests and the unrelated format-driven
test decomposition. No execution authorization is requested or inferred here.

| File | Original SHA-256 | Proposed SHA-256 |
| --- | --- | --- |
| angular_deleted_owner_membership_issue_162.rs | 69c27a304b59029f7aee0ed2bfd511eb47c2702c0a8a43bb22d09653368b797f | 1399e46e1b5e2573b1ba70021e9172957d448ceb74c3ffa039204b6a04222cc0 |
| angular_resolver_registration_public_issue_162.rs | 8b047c6e817adb63aa6ba078375258b68ce2b3677e73b88b15f2d283d78821dc | 9af4593ad4cc62b462dd8d7ca292970018c3dcf3508724ab2ca8ed30c431978b |

Exact originals and proposals are under
`target/ang-r2/format-application-20261116/{before,proposed}/src/tests/mcp/`.
Reviewable unified diffs are in the same archive root:

- `angular_deleted_owner_membership_issue_162.rs.diff`
- `angular_resolver_registration_public_issue_162.rs.diff`

These are archived evidence and proposed edits, not alternative executable tests.
The original public-witness stash remains intact. Test requirements continue to be
satisfied only by tracked tests under src/tests.

## Independent work completed before requesting disposition

Applied the exact operator-generated rustfmt replacements to 33 unfrozen files;
no formatter, compiler, guard or runtime was started by the agent. The expanded
lifecycle test file would have been 858 lines, so it was decomposed by concern:

- angular_resolver_registration_lifecycle_issue_162.rs: 8 query/contract/work tests.
- angular_resolver_registration_persistence_issue_162.rs: 6 admission/SQLite/replay tests.
- angular_resolver_registration_support_issue_162.rs: shared fixture/query helpers.

All fourteen formatted test bodies match their pre-relocation SHA-256 values.
All active Rust files remain within 615 lines. Strict UTF-8/no BOM and git diff
whitespace inspection passed. These are static checks; post-format compilation,
tracked tests, full format acceptance, guards, CI and final delivery remain pending.

## Disposition 03 applied; post-normalization verification UNRUN (2026-11-16)

The owner explicitly approved only the reviewed wrapping, spacing and trailing-comma replacements for the two primary witnesses, retained source/evidence preservation, and concern-based decomposition of the fourteen lifecycle/persistence tests. `ANG_R2_OWNER_DISPOSITION_03.md` records the attributed decision; it grants no execution permission or protected authority authorization.

Agent MaxHeadRoom applied the exact proposed hashes: retirement witness `1399e46e1b5e2573b1ba70021e9172957d448ceb74c3ffa039204b6a04222cc0`, public witness `9af4593ad4cc62b462dd8d7ca292970018c3dcf3508724ab2ca8ed30c431978b`. Original hashes remain `69c27a304b59029f7aee0ed2bfd511eb47c2702c0a8a43bb22d09653368b797f` and `8b047c6e817adb63aa6ba078375258b68ce2b3677e73b88b15f2d283d78821dc`. The existing target evidence mechanism now consolidates exact originals/proposals, RED/GREEN outputs/receipts, available source manifests and the successful pre-format replay capture under `target/ang-r2/frozen-format-retention-20261116/manifest.json`. Manifest SHA-256 is `7af8f0742cab618c7ecca3c22346758eb1bfd80e6baea88a9165c370ffb695e5`. Every retained entry has an exact source path, byte count and SHA-256. The original public-witness stash remains unchanged. No source archive serves as an alternate test outside src/tests.

Both primary files preserve exact string-literal spellings and assertion macro counts. Fourteen formatted test bodies are retained across eight query tests and six persistence/admission/replay tests, with a shared helper module registered in semantic_compilation. Trailing inter-test separators at new file ends were normalized independently of function bodies. Static UTF-8/no BOM, active-file ceilings and whitespace are checked; compiler/format acceptance and normalized GREEN remain UNRUN. Earlier exact-source RED-to-GREEN, 43 focused passes, successful replay-only capture and 185 stdio checks retain their actual source identities and are not relabeled post-normalization evidence.

`invoke-format-verification.ps1` is the exact operator handoff: active-source read-only rustfmt check; normalized retirement witness (1); all registration tests (18, including two public witnesses, fourteen relocated cases and two transport tests); scoped all-feature library/binary/test Clippy; UTF-8 and size guards; actual normal DOC-270 knowledge admission checker. It captures source/log hashes, exits, counts and times, refuses changed approved identities and stops on failures. It starts no broad/full repository gate, formatter mutation, production build, commit or push. The final local knowledge stdio is explicitly part of the operator command, not an agent invocation.

The governing DOC-270 candidate records disposition 03 and the original/normalized source and evidence associations, and covers all mission documents through ANG_R2_EVIDENCE_INDEX.md and exact document pins. Its new candidate admission remains UNRUN; the earlier admitted revision remains historical, not current approval of this amendment. No protected grant is generated. Applicable CI and final field evidence remain separate.

## Post-normalization packet verified; durable evidence association completed (2026-11-16)

Agent MaxHeadRoom verified `target/ang-r2/format-verification-20261116-111528-924`: normalized retirement witness 1 PASS (104.9900179 seconds), normalized registration/relocation tests 18 PASS (619.2530992 seconds), scoped format PASS (0.5851703 seconds), scoped all-feature Clippy PASS (41.1512796 seconds), UTF-8 PASS (15.1181733 seconds), active-size PASS (6.0137078 seconds). No compiler warnings. All receipt log hashes, unchanged captured before/after manifests and current active Rust source pins match. The actual document checker returned ready/current/complete, no diagnostics, generation `00ee1003cb3a8c07d2c9f2511cf0bf6bbbc7952798643f888ec08ece7e95bfd4`, process exit 0. This verifies normalized witnesses, decomposed module/helper registration and the tested applicable contracts; it is not full CI or field/delivery certification.

The final preservation check found that exact original raw bytes still depended on ignored target storage. The existing versioned Git evidence mechanism now retains an ordinary binary archival artifact, `historical-witness-evidence.zip`, alongside the campaign records. SHA-256 `ea328ed56679bf05800658c6577d2a919f3de93fdcdd6a1ca9ba61eddd0b8b53`. This is a byte-preserving ZIP_STORED copy of the existing target retention directory, including its original manifest and all 30 recorded source/proposal/diff/RED/GREEN/replay receipt/log entries. Each archive entry was read back and verified against the source manifest hash. It introduces no evidence service, schema, importer or alternate executable tests. The bundle is a versioned artifact awaiting normal commit, so durable remote availability is pending commit/push rather than already claimed. Original target receipts/stash remain unchanged.

The governing record now associates that artifact and the observed 19-test packet. Because its substantive content and covered document pins changed, the new governance amendment and final documentation guards still require actual execution. The updated existing wrapper supports `-ResumeVerifiedFrom target/ang-r2/format-verification-20261116-111528-924`: it verifies exact unchanged Rust manifests, baseline, passing log hashes/counts/warnings, retains explicit reuse records, then runs only UTF-8/size guards and the actual normal admission checker. It does not repeat tests, format or Clippy. No new long-running invocation is authorized to the agent. Earlier replay-only and pre-normalized GREEN results retain their original source identities.
