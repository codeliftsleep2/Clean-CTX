# GitHub Issue → Automated Agent Workflow

**Status: IMPLEMENTED (vertical slice, 2026-08).** The vertical slice described
below is implemented and versioned in this repository, and the design was
updated to match verification against the current `@cline/sdk` (0.0.81). It is
gated on the manual setup steps listed at the end of this document.

## Status update (2026-08) — changes from the original design

The original "DESIGN ONLY" architecture was reviewed against the current
`@cline/sdk` and GitHub Actions. Two corrections were required:

1. **Rules must be injected, not auto-discovered.** `.clinerules/` is
   intentionally gitignored, so a CI checkout does not contain it and the SDK
   does not discover it in CI. The runner therefore injects the policy from the
   tracked `AGENTS.md` (a portable projection of `.clinerules/engineering.md`)
   plus `docs/agent/*` routing into the Cline system prompt. `AGENTS.md` is the
   tracked carrier; it is not a second ruleset.
2. **Two-phase execution, not one continuous pipeline.** Because approval is a
   pause between runs, plan (read-only) and implementation run as two separate
   GitHub Actions runs, with per-issue state persisted in a bot-managed issue
   comment (`AGENT-STATE` JSON blob).

Implementation layout: `.github/workflows/agent.yml` + `scripts/agent/*.mjs`
(`agent.mjs` entry, `policy.mjs` tool/command/file policy, `state.mjs` state +
authorization, `gitops.mjs` branch/commit/push/PR) + `package.json` +
`package-lock.json` (`@cline/sdk`). `AGENTS.md` carries the portable policy.

## Objective

Allow an explicit, authorized command on a GitHub Issue to initiate a
Cline agent task without manual local intervention.

## Implemented pipeline

```
GitHub Issue
    ↓  (trigger: authorized collaborator comments /agent run)
GitHub Actions workflow (.github/workflows/agent.yml)
    ↓
@cline/sdk ClineCore (Node 22+), read-only plan session
    ↓  (policy injected; NOT auto-discovered)
  AGENTS.md (portable projection of .clinerules/engineering.md)
  docs/agent/*.md  (routed, consulted when applicable)
  docs/ARCHITECTURAL_INVARIANTS.md
    ↓
plan posted as issue comment (AGENT-STATE blob, AWAITING_APPROVAL)
    ↓  (authorized collaborator comments /agent approve)
GitHub Actions (second run) → ClineCore implement session
    ↓  (host-enforced tool/command/file policy)
isolated branch agent/<issue>-<slug>
    ↓  (host runs docs/agent/verification.md Final Verification Gate)
commit → push ONLY agent/** → gh pr create (existing PR body conventions)
    ↓
existing CI gates (.github/workflows/ci.yml) run on the PR
    ↓
merge → gated release discipline (docs/agent/releases.md)
```

## Owner per stage

| Stage | Owner |
|-------|-------|
| Issue creation / lifecycle | GitHub (native) |
| Trigger / orchestration | GitHub Actions workflow (`agent.yml`) |
| Agent invocation | Node runner on `@cline/sdk` ClineCore (not the IDE extension) |
| Policy/context | Injected policy (`AGENTS.md`) + routed `docs/agent/*`; never auto-injected (`.clinerules/` is gitignored) |
| Plan + approval | Agent produces plan; authorized collaborator approves via `/agent approve` |
| Tool authorization | Host-enforced via `toolPolicies`/`requestToolApproval` in `policy.mjs` |
| Implementation | Agent on an isolated `agent/<issue>-<slug>` branch |
| Git/PR | Runner (`gitops.mjs`): commit, push `agent/**` only, `gh pr create` |
| Tests/gates | Runner Final Verification Gate + existing CI (`.github/workflows/ci.yml`) |
| Merge/release | Human + gated release discipline |

## Triggering model

The agent is gated by explicit human intent. Nothing starts on issue creation
or on a label alone:

- `/agent run` — read-only investigation, posts a plan.
- `/agent approve` — promotes the plan and starts isolated implementation.
- `/agent abort`, `/agent retry` — cancel or re-attempt.
- `workflow_dispatch` — available for maintainer testing.

Every command requires an authorized repo collaborator (write/admin; enforced
in the state module before any action).

## Manual setup required (deployment time only)

- Repo secret `CLINE_API_KEY` (Anthropic provider key used by the SDK). The
  workflow maps it to the runner's `CLINE_AGENT_API_KEY` env var; it is never
  logged or exposed to the model.
- (Recommended) Branch protection on the default branch: require CI status
  checks and deny direct/force pushes. This is the hard backstop that caps the
  `contents: write` blast radius even though the runner itself refuses any
  push outside `agent/**` and never uses `--force`.

## Security concerns

- Repo secrets leak risk: the runner must never echo model input/output that
  could contain secrets; `policy.mjs` blocks `env`/`printenv`/curl/exfiltration
  commands and uses least-privilege GitHub token scopes.
- Prompt-injection via issue body: the issue text is untrusted data; the runner
  injects policy via the system prompt and enforces precedence in code, so
  issue text cannot override policy or tooling.
- Approval gates are enforced by the workflow author checks, not only in the
  agent prompt.
- The runner never pushes to the default branch; pushes are limited to
  `agent/**` and no `--force` is ever used.

## Deliberately deferred (not in this slice)

- Auto-trigger on issue creation or by label (explicitly rejected).
- Auto-merge of PRs (never; merge stays human).
- Multi-agent teams, subagents, scheduling/poll automation.
- A repository Cline lifecycle-hook/plugin package (SDK-native
  `toolPolicies`/`requestToolApproval` is used instead).
- A cost dashboard beyond the per-run limits / usage surfaced by the runner.
- A CI step that rejects PRs altering `.github/` or `scripts/agent/**`
  (recommended follow-up, not required to prove the slice).