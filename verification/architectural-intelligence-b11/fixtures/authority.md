# Synthetic declaration authority

<!-- pcg:SUMMARY:begin -->
Synthetic source-owned declaration authority.
<!-- pcg:SUMMARY:end -->
<!-- pcg:RULE:begin -->
Reject duplicate declaration fields.
<!-- pcg:RULE:end -->
<!-- pcg:LIMIT:begin -->
Grammar admission is not authenticated owner identity.
<!-- pcg:LIMIT:end -->

```architecture-graph
node: AUTH.PCG_DOCUMENTS
kind: AUTHORITY
status: IMPLEMENTED
title: Synthetic admission contract
track: FOUNDATION
artifact: docs/authority.md
knowledge_scope: authority
knowledge_content:
  - statement self#SUMMARY
  - constraint:duplicate_fields self#RULE
  - qualification self#LIMIT
```
