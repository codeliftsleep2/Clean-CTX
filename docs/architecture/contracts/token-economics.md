# Complete-alternative token economics

Agent: Agent MaxHeadRoom. Current normative owner of original SCHEMA-vNext §3 clauses 6–7, relocated without changing their policy. Original commit `9400d476`, source `docs/architecture/SCHEMA_VNEXT_PROPOSAL.md`. Historical percentages and experimental tables remain in that source and are not universal claims.

6. **No tokenizer-by-character reasoning.** Every candidate is measured using
   actual target tokenizers or an explicitly bounded approximation.
7. **Raw continues to win ties.** The final economics selector remains outside
   the renderer and compares complete alternatives with the same tokenizer.

```architecture-graph
node: AUTH.TOKEN_ECONOMICS
kind: AUTHORITY
status: IMPLEMENTED
title: Complete-alternative token economics
track: REPOSITORY
artifact: docs/architecture/contracts/token-economics.md
depends_on:
  - AUTH.PRESENTATION_VNEXT
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
