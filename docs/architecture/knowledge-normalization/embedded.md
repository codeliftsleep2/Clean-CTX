# Non-prose and embedded architectural knowledge

Agent: Agent MaxHeadRoom. The [production](embedded-production.csv),
[test/fixture](embedded-tests.csv) and [non-prose](embedded-nonprose.csv) indexes
record 1,011 tracked non-prose or source-comment-bearing files with exact hashes and dispositions.
This supplements the 419 prose documents; it does not turn the repository source
tree into graph nodes. Code comments were structurally screened, not treated as
independent approvals or a complete implementation audit.

## Executable and schema witnesses

| Source | Actual architectural information | Proposed graph treatment |
| --- | --- | --- |
| `.github/workflows/ci.yml` | Actual release-check execution, warning-fatal Clippy, size/UTF-8/fitness guards, all-feature tests and audit | Link from verification authority as executable witness. Check/Clippy omit all-features: report C15 remains unresolved |
| `.github/workflows/agent.yml` | Automated investigation/approval/branch/PR workflow | Link from GITHUB_ISSUE_AGENT_WORKFLOW and AGENTS portable policy; no new workflow authority inferred from an issue body |
| `Cargo.toml`, `proxy/Cargo.toml` | Product feature/dependency boundaries; defaults TypeScript/C#/Angular/.NET; Rust/Java opt-in; Spring depends on Java | Link from language support and optional network effect propositions. Build defaults do not establish all-feature development coverage |
| `src/pcg/documents/schema.sql` | Immutable revisions, generations, publication sequence, endpoint foreign keys, reverse edges and FTS5 document search | Witness for AUTH.PCG_DOCUMENTS; distinct from evidence-corpus schema. Do not create another schema or ingestion service |
| `src/pcg/schema.sql` | Enrolled corpus/source/evidence/record revisions with separate execution, attribution and lifecycle fields | Witness for enrolled-v1 contracts. Its claim/decision/constraint/evidence roles are not the document graph's seven kinds |
| `src/mcp/tool_handlers/pcg/schema.json` | Closed evidence-query public request shape | API witness for AUTH.PCG_MCP_V1, not a generic document graph manifest |
| `src/pcg/documents/model.rs`, `validate.rs`, `parse.rs` | Exact document vocabulary, identity grammar, candidate-state consistency and declaration parsing | Source bodies were inspected after successful skeleton context. Contract evidence, not a newly executed parser/validator result |
| `.githooks/pre-commit`, `scripts/check-utf8.ps1`, `scripts/check-file-sizes.ps1` | Actual existing policy enforcement and exemptions | Link from portable policy/encoding/final gate; no new hook framework and no bypass |
| `.cargo/audit.toml`, `deny.toml`, dependency lockfiles | Supply-chain configuration and frozen dependency resolution | Witnesses for the precise supply-chain proposition only; no generic security certification or per-dependency graph admission |
| `filters/*.toml` (33 files) | Shipped tool-output filtering rule vocabulary and configuration | Retain files, link named rule evidence when relevant. Do not admit one authority per filter or infer complete secret-language coverage |
| `scripts/investigations/direct-mcp-secret-boundary/evidence/*` (five indexed files) | Frozen direct-MCP boundary observation and marker/wire/result subjects | Selectively link to a focused security investigation. Never generalize a proxy scrub result to the direct-MCP boundary |

Other JSON/TOML/license records in the non-prose index remain configuration/schema/oracle
or legal artifacts. `LICENSE` establishes licensing, not semantic architecture.
Package/dependency metadata is not an experiment result. No artifact was rewritten.

## Text fixtures and old source snapshots

All nine tracked `.txt` files were accounted for separately from Markdown:

- `scripts/investigations/direct-mcp-secret-boundary/evidence/server-stderr.txt`:
  frozen run stderr, not an instruction or general leak-proof claim.
- `src/test_files/unicode_canary.txt`: encoding witness, not policy.
- `src/tests/diff/fixtures/overload_source_forms_original_comparator.txt` and
  `overload_source_forms_original_helper.txt`: immutable old implementation controls.
- `src/tests/ir/fixtures/binary_reverse_original_string_table.txt`: old encoder
  comparison source, not current serialization authority.
- `src/tests/mcp/fixtures/delta_previous_current_handler.txt`,
  `delta_previous_original_handler.txt`, `diff_source_original_handler.txt` and
  `src/tests/mcp/sqlite_text_before_af_perf_002.txt`: frozen comparison/mutation
  witnesses for bounded performance work, not alternative production paths.

The 15 indexed test fixtures include additional JSON response oracles. Their
values preserve baseline subjects; a fixture's existence does not establish a
passing regression. Actual test authority remains tracked `src/tests/**`, not
the fixture alone and not this inventory.

## Operator definitions and source comments

Twenty-one indexed non-prose verification records contain manifests, oracles,
scoring schemas or lineage. They are reusable definitions/research inputs.
In particular, context-compression codec oracles, task scenarios and workspace
query oracles distinguish codec preservation from model-visible task behavior;
NgRx recall corpus/oracle files describe a replacement benchmark, not recovered
historical repo-scale evidence. Source Markdown explains these distinctions and
is inventoried separately. No benchmark or model evaluation ran here.

The remaining 923 indexed Rust files contain doc comments. Those explain
implementation ownership and supported envelopes but do not independently
approve decisions or change the precedence of current contracts. Their path,
hash, doc-comment count and first comment excerpt are retained to make this
screening auditable. Detailed source lifecycle tracing remains necessary when a
future migration makes an implementation compliance claim. This assessment only
inspected selected PCG bodies and executable policy witnesses; it does not certify
all 923 implementations against their comments.

Generated `target/**` data, local SQLite databases and six untracked corpus/pilot
reports are outside the tracked inventory. Prior source references to those data
remain attributed historical evidence, not newly authenticated runtime results.
Owner-held proprietary receipts stay on the originating machine; no such evidence
was acquired or transmitted for this design.
