# B7 resource-policy assessment

Agent: **Agent MaxHeadRoom**, 2026-11-12. Assessment only.

The accepted owner profile sets token_budget65536,max_files2048,max_nodes512,
max_edges2048,max_bytes33554432 and docs-only roots. Request contexts use128nodes/
512edges. src/pcg/documents/profile.rs owns positive u64 token_budget validation;
documents/mod.rs:117 bounds each request to that profile. No separate65536hard-coded
maximum was found in that inspected document path;this is not proof that every
upstream caller/host can safely carry arbitrary larger values. Session profile
bytes/permissions are pinned. A new session is required after owner changes.

mod.rs:204-231 projects the requested explicit format,serializes the complete MCP
result including escaped inner JSON,counts the selected tokenizer,and clears all
documents,evidence pools,relationships and witnesses on overflow. The bounded result
is blocked/required_budget_exceeded with context.complete=false. A too-small control
budget becomes token_budget_too_small. Completion rechecks source observations
before commit/return. catalog_current can remain true while knowledge is unavailable.
Freshness is observed boundaries,not a continuous or atomic filesystem snapshot.

The inherited unavailable closures demonstrate knowledge-availability failure for
those legitimate tasks,not that large knowledge is undesirable. Source gets and
related calls expose useful fragments but do not verify a complete combined closure.
Increasing an appropriate finite profile ceiling is already a configuration-shaped
candidate,not necessarily a new schema. It requires separately authorized measured
source volume,host transport/memory/latency controls,exact metering/refusal/currentness
and regression coverage. It may not solve discoverability or interpretation.
Lossless transport/composition or corrected genuinely irrelevant relationships are
alternatives,not approvals. Never prune legitimate knowledge or treat a smaller
response as inherently better. No profile/resource/transport changes were made.
