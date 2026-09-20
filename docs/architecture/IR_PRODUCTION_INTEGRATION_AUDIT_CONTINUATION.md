# IR Production Integration Audit — Continuation

**Status:** Phase 9 in progress. This document continues
[`IR_PRODUCTION_INTEGRATION_AUDIT.md`](IR_PRODUCTION_INTEGRATION_AUDIT.md)
after P9-15/P9-16 were user-verified green.

## 23. Finding P9-17: registered `apply_edit` description advertises a removed contract

**Severity:** High externally visible tool-contract contradiction; Option 1
implemented pending user verification.

The registered `apply_edit` input schema correctly requires `filePath` and an
`operations` array containing `replace_body`, `delete`, `insert_after`, or
`insert_before`. Its registered description begins by advertising a different
generic text editor: `insert_line`, `new_text`, `old_text`, arbitrary
replacement, and file creation. None of those parameters or behaviors exists
in the production handler.

The real handler requires previously tracked byte-exact state, refuses new or
unowned files, resolves structural units, validates expected unit text, applies
the approved P9-15/P9-16 source-identity and durable transaction contract, and
returns operation-specific byte spans. The runtime prompt and authoritative
tooling guide describe this structural contract; only the public registered
tool description still exposes the obsolete generic contract.

This can cause MCP clients and LLMs to send requests that the registered schema
and handler reject, or to believe `apply_edit` can create arbitrary files. It
also omits the now-authoritative stale-source and durability preconditions.

### Alternatives

1. Replace the obsolete description with the existing structural operation
   contract and explicitly summarize tracked-state, byte-exact, stale-source,
   and transactional durability behavior. Keep the correct input/output schema
   and production handler unchanged.
2. Restore the advertised generic editor behavior. This conflicts with the
   approved structural, tracked-state, and byte-exact edit architecture.
3. Introduce a separately named generic file-edit tool. That is a new public
   capability outside the Phase 9 repair and requires separate review.

### Recommendation

Choose option 1. This is a bounded public-description correction: it makes MCP
discovery truthful without changing the approved handler, operation schema, or
edit behavior. Add registered tool-list coverage that rejects the obsolete
vocabulary and asserts the structural contract.

## 24. Approval gate and next audit action

The obsolete generic-editor description has been replaced with the approved
structural, tracked-state, byte-exact, stale-source, staged-durability, span,
and full-body contract. Input/output schemas and production behavior are
unchanged. Tracked `tool_list()` coverage protects both required and forbidden
vocabulary. The exhaustive audit is paused for user-run P9-17 verification;
Phase 9 is not certified.
