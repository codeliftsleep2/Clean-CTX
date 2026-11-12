# Historical replay reuse hypothesis

Agent: **Agent MaxHeadRoom**. B5, 2026-11-12.

EXP-003-REPLAY is the assessment's exact experimental proposition: an earlier
selected Candidate presentation is equivalent to the later replay presentation
under intervening alias deletion. It is FALSIFIED for that substitution by the
[executed design counterexample](../../agent/AF_PERF_002_REPLAY_RENDER_DESIGN.md).
The complete original report is its artifact, preserving conditions, timing tables
and limitations. This is not a prohibition on all replay optimization or a claim
that successful replay after deletion establishes a new lifecycle policy.

The registered-delete counterexample changes original footer/cache bytes (542-byte
later presentation versus the 607-byte earlier candidate). Test GREEN records
successful detection of that divergence, not correctness of candidate reuse.
The later owner no-repair/retirement decision has its separate PERF-001-REPLAY
identity in the Gitdiff discovery record. No replacement implementation is admitted.

```architecture-graph
node: EXP-003-REPLAY
kind: EXPERIMENT
status: FALSIFIED
title: Earlier Candidate equals later replay under alias deletion
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_REPLAY_RENDER_DESIGN.md
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
```
