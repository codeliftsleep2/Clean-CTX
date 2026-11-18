# #136 whole-declaration insert_before repair

## 1. Verdict

**#136 WHOLE-DECLARATION INSERTION SLICE COMPLETE**

The approved placement slice is GREEN. This does not resolve the entire issue.
CBM A0–B5 and other native defect slices were not modified. No #142 work occurred.
Windows execution is not established by Linux checks of CRLF source fixtures.

## 2. Proven root cause

**PROVEN BY CODE INSPECTION, TEST and LIVE MCP:** the handler built a body-only
UnitTable unless a batch contained Delete. InsertBefore resolved the correct
member identity but planned at UnitRecord.start_byte, which is the body's start.
Valid member text at that offset failed syntax validation and wrote nothing.
Parseable comment text succeeded but landed between the signature and body.
Parseability was therefore insufficient to establish correct placement.

In the live C# LF fixture First's authoritative declaration begins at UTF-8 byte
57; the attributed Last group begins at byte 128. The RED oracle records these
positions independently of body offsets, including the non-ASCII prefix.

## 3. Repair and authority

The existing declaration mapper is now requested for InsertBefore as well as
Delete. It uses the existing language query and exact declaration containment/end
match to attach positional authority to the already-resolved body record. It
does not select a declaration by name or proximity. It reuses the existing
declaration-mapping parsing pass; no parallel parser/finder is introduced.

UnitRecord.insert_before_byte is separate from the body span and deletion span.
It starts at the captured header/modifier or the existing structurally identified
attribute/decorator metadata group. InsertBefore requires that authority; no
fallback to body start exists. The planner still uses the existing disjoint-splice,
line-ending adaptation, syntax gate and transactional handler.

A leading sibling comment is not established as declaration-owned syntax by this
mapping. If such a comment borders the metadata group, insertion authority is
withheld and the MCP call truthfully refuses without writing. This is a refusal
of uncertain ownership, not proximity-based assignment of documentation. Comments
outside that group are not moved or reclassified. A parser-owned child remains
inside its declaration capture. No new documentation ownership parser was added.

ReplaceBody retains the body span; Delete retains its existing declaration span;
InsertAfter retains the existing end position. Identity/overload lookup, source
hashes, prior-read authority, syntax validation and durable commit logic are
unchanged. Missing insertion authority fails before a splice or write.

## 4. RED → GREEN matrix

Permanent regression: src/tests/mcp/insert_before_declaration.rs through the
registered dispatcher. Nine tests failed against unchanged production, then the
same assertions passed after the fix. Only those tests and registration were
stashed during implementation and restored; subsequent rustfmt changed layout,
not requests, fixtures or assertions.

Live harness: verification/issue-reconciliation/Invoke-InsertBeforeDeclaration.ps1.
Requests and exact-source oracles are retained as JSON and byte-preserving base64.

| Case | Before | After | Exact-source oracle | MCP boundary |
| --- | --- | --- | --- | --- |
| C# first, LF/CRLF | Syntax refusal, unchanged bytes | Member inserted at declaration start | Prefix + normalized requested insertion + untouched suffix | Registered dispatcher + live stdio |
| C# middle, LF/CRLF | Syntax refusal, unchanged bytes | Correct declaration start | Same oracle | Dispatcher + live |
| C# attributed last, LF/CRLF | Syntax refusal, unchanged bytes | Insert precedes [Obsolete] and complete target | Same oracle | Dispatcher + live |
| TS first, LF/CRLF | Syntax refusal, unchanged bytes | Correct public member start | Same oracle | Dispatcher + live |
| TS middle, LF/CRLF | Syntax refusal, unchanged bytes | Correct public member start | Same oracle | Dispatcher + live |
| TS decorated last, LF/CRLF | Syntax refusal, unchanged bytes | Insert precedes @audit and target | Same oracle | Dispatcher + live |
| Parseable comment before metadata, both languages and EOLs | Success at wrong body offset | Correct metadata/declaration boundary | Byte-for-byte comparison detects misplacement | Dispatcher + live |
| Body-only record without declaration authority | Planner incorrectly grants insertion | Refusal | No fallback splice accepted | Permanent focused test |
| Unproven leading C# XML doc / TS block-doc ownership | Parseable misplaced insertion accepted | Explicit authority refusal, source unchanged | Original file bytes | Live MCP |

The eight successful dispatcher tests each cover two EOL variants (16 placement
scenarios); the ninth tests missing authority. Live GREEN has 16 placement, two
ownership-refusal and five neighboring-control groups: **23/23 PASS**.

RED live evidence: `target/insert-before-declaration/20261012-024207-31464c95`:
18 failing insertion/ownership groups and five passing preservation groups.
GREEN live evidence: `target/insert-before-declaration/20261012-024659-6efb1976`.
Each successful edit additionally asserts the receipt's UTF-8 declaration start
and syntaxGated=true. The syntax checker also validates dispatcher expected output.

An intermediate live run (`20261012-024429-143326f7`) passed 22 groups but encountered
No durable context exists before insertion planning for one new fixture. Its
first provide reported version 3 instead of a fresh version. **INFERENCE:** atomic
replacement followed by fixture creation exposed a separate file-identity reuse
lifecycle problem. Root cause is NOT ESTABLISHED in this slice. The failure remains
in its saved run and was not reclassified as placement success.

The final harness allocates all independent fixture files before any atomic edit
releases an original inode. This changes fixture setup only; requests/source/
placement oracles remain the RED oracles. No retry hides errors and no production
identity change was made. This harness establishes the isolated placement contract,
not general inode-reuse/session lifecycle correctness.

## 5. Preservation

**PROVEN BY TEST:** existing edit:: filter passed 61 tests, including deletion,
body spans, replace/after mechanics, transaction/durable behavior and hard-link
refusal. Existing apply_edit filter passed 12 tests; three pre-existing external
E2E tests remained ignored and are not claimed executed.

**PROVEN BY LIVE MCP:** exact bytes after replace_body, insert_after and delete;
atomic batch rollback on a bad target; invalid syntax rejection without write;
external source-change refusal; never-read-file refusal. The body replacement
is still body-only, and subsequent operations use the refreshed authority.

**PROVEN BY CODE INSPECTION:** successful edits still pass through the existing
syntax gate, candidate compilation and atomic commit; no hash/persistence check
was bypassed to produce GREEN. Unrelated code was not repaired.

## 6. Remaining #136 work

- Qualified overload/getter/setter addressability and current last-match behavior.
- Unsupported constructs and their diagnostic taxonomy; no new construct support.
- Structured documentation/comment ownership beyond the safely rejected cases.
- Decorated non-exported owner identity (#139), generic selector differences and
  class-name/constructor selectors.
- InsertAfter separator/indent policy, deletion doc whitespace and message/verify
  behavior not covered by the preserved tests.
- Original real-workspace/Windows acceptance and general identity-reuse lifecycle.

These are unresolved or unestablished, not bundled into this repair. Neither #136
as a whole nor another GitHub issue was closed.

## 7. Changed files and responsibilities

| File | Responsibility |
| --- | --- |
| src/edit/locate.rs | Derive insertion authority from existing structured declaration mapping; withhold it for unproven sibling-comment ownership |
| src/edit/apply.rs | Require insertion authority; preserve neighboring operation positions and failure semantics |
| src/edit/ops.rs | Document whole-declaration InsertBefore contract |
| src/mcp/tool_handlers/edit.rs | Request declaration-aware mapping when an insertion needs it |
| src/mcp/mod.rs | Register the permanent focused MCP regression module |
| src/tests/mcp/insert_before_declaration.rs | Public dispatcher exact-byte regression matrix and absent-authority refusal |
| verification/issue-reconciliation/Invoke-InsertBeforeDeclaration.ps1 | Reusable PowerShell live registered MCP placement/preservation verification |
| verification/issue-reconciliation/README.md | Copy-pasteable harness command and limits |
| docs/plans/POST_CBM_133_141_NATIVE_DEFECT_RECONCILIATION.md | Link this follow-up without rewriting historical baseline or whole-issue disposition |
| This report | Root cause, authority decision, executed evidence and remaining scope |

## 8. Executed commands and evidence

All Cargo commands ran in /workspace/Clean-CTX with the existing toolchain:
`PATH=/workspace/.tooling/cargo/bin:$PATH`,
`RUSTUP_HOME=/workspace/.tooling/rustup`,
`CARGO_HOME=/workspace/.tooling/cargo`.

| Actually executed command | Result |
| --- | --- |
| cargo test --locked -p clean-ctx --lib --all-features -j 4 insert_before_declaration_ | RED: 0/9; GREEN: 9/9 |
| cargo test --locked -p clean-ctx --lib --all-features -j 4 apply_edit | 12 pass, 3 ignored |
| cargo test --locked -p clean-ctx --lib --all-features -j 4 edit:: | 61 pass |
| rustfmt --edition 2024 --config skip_children=true on the four changed implementation/test files | Formatting completed; no repository-wide formatter |
| cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings | Passed without warnings |
| cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4 | Passed, single candidate binary build |
| PowerShell Invoke-InsertBeforeDeclaration.ps1 against pre-fix binary | RED placement evidence retained |
| Same harness against candidate, then fixture-allocation correction | Intermediate 22/23, final 23/23; no production identity fix |

Pre-fix binary SHA256:
`cd62c9e229f5698875832b792c5f15a22473cbeb9ffd7695ce12de76d2fe8219`.
Candidate SHA256:
`30cddb3280c0a2df79a25395fdb62fba35464b52f127efdba0a0389a1a36bf0d`.
Final header/comment-only cleanup does not alter the tested execution logic.

The PowerShell runs used the installed pwsh with workspace XDG directories and
`-NoProfile -File verification/issue-reconciliation/Invoke-InsertBeforeDeclaration.ps1
-BinaryPath target/debug/clean-ctx`. They start only owned Clean-CTX sessions in
provider-disabled fixtures. No CBM runtime, model-host pilot or full suite ran.
PowerShell AST, UTF-8/no-BOM, file-size and git diff checks are recorded at completion.

Stop after this slice. #137 and every other production slice require separate
owner review and authorization.
