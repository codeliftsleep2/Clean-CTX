# GitHub Issue → Automated Agent Workflow (Design Only)

**Status: DESIGNED / NOT IMPLEMENTED.** The repository does not currently
contain the infrastructure required to run this automation. This document
records the evaluated architecture so the decision can be revisited without
re-deriving it.

## Objective

Allow creating (or labeling) a GitHub Issue to initiate an agent task without
manual intervention.

## Proposed pipeline

```
GitHub Issue
    ↓  (trigger: label `agent` or issue_comment `/agent run`)
GitHub Actions workflow (pull_request/pull_request_target variant)
    ↓  (run a Node/TypeScript runner)
@cline/sdk Agent / ClineCore (Node 22+)
    ↓  (load repository rules/context)
    .clinerules/engineering.md + .clinerules/encoding.md
    docs/agent/*.md  (conditional procedures, routed as applicable)
    docs/ARCHITECTURAL_INVARIANTS.md
    ↓
plan → post as issue comment → human approval → implement
    ↓
commit to branch `agent/N` → open PR (existing PR template)
    ↓
existing CI gates (.github/workflows/ci.yml) run on the PR
    ↓
merge → gated release discipline (docs/agent/releases.md)
```

## Owner per stage

| Stage | Owner |
|-------|-------|
| Issue creation / lifecycle | GitHub (native) |
| Trigger / orchestration | GitHub Actions workflow |
| Agent invocation | Node runner built on `@cline/sdk` (not the IDE extension) |
| Rules/context | Same rule architecture as local sessions (`.clinerules/`
always-loaded + `docs/agent/` routed) |
| Plan + approval | Agent produces plan; human approves via issue comment/PM |
| Implementation | Agent on a feature branch |
| Tests/gates | Existing CI (`.github/workflows/ci.yml`) |
| Merge/release | Human + gated release discipline |

## Triggering model

NOT every issue should auto-start an agent. The safest design gates the agent
on explicit human intent:

- An `agent` label applied to the issue, and/or
- A `/agent run` comment with an optional scope/constraint body.

This prevents spam/dependency-race auto-starts and keeps agent task-class
separate from the issue triage workflow.

## Prerequisites required before implementation

- GitHub Actions permissions to post comments and create branch/PR (`pull-requests: write`,
  `issues: write`).
- An LLM API key (`@cline/sdk` needs a provider credential) stored as a repo
  secret.
- Node.js 22+ available in the runner.
- A vendored or npm-installed `@cline/sdk` dependency.
- An approval model (two-phase: plan comment → explicit `@approve` before any
  write).
- Cost control (token budget caps per task, per-run limits, concurrency guard).

## Security concerns

- Repo secrets leak risk: the runner must never echo model input/output that
  could contain secrets; use least-privilege GitHub token scopes.
- Prompt-injection via issue body: the agent must treat issue text as untrusted
  input and follow repository rules, not issue instructions, as authority.
- Approval gates must be enforced in the workflow, not only in the agent
  prompt.

## Why implementation is deferred

- No existing Cline hook/plugin or automation infrastructure in the
  repository.
- Requires new external dependencies and secrets not present today.
- The manual agent workflow already works; the automation should be added only
  when a concrete bottleneck demands it.
- Per the engineering discipline, do not build infrastructure before a need is
  demonstrated.