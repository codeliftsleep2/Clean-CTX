# Workspace-query portable-content repair

**Status:** implemented and verified

## Boundary

- `structuredContent` remains unchanged and authoritative.
- Successful query results are also rendered into versioned portable `content`
  as `clean-ctx/workspace-query-answer` v1.
- The envelope records query identity/direction, effective declared scope,
  authoritative index-snapshot status, discovery diagnostics, explicit
  zero-result state, and the complete existing query result.
- Entity, edge, and reachable-identity arrays receive rendering occurrence
  ordinals without changing `structuredContent`.
- `transitive_dependencies` retains its existing reachable-identity semantics;
  no path or per-hop contract was introduced.

## Verification outcome

The user-run local gates passed. A fresh registered-path capture from the rebuilt
production binary verified all six query modes plus a negative reverse-edge
case. Structured facts matched the controlled graph, every response used the
v1 content schema, positive semantic records were present in portable
`content`, and the negative result explicitly reported `zero_result: true`.

A real-host trace remains useful field evidence, but portable correctness no
longer depends on a host exposing `structuredContent` to the model.
