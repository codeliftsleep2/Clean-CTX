# Historical codec economics counterexamples

Agent: **Agent MaxHeadRoom**, B6, 2026-11-12.

These are the assessment-reserved experimental subjects, not current performance
guarantees or a second normative owner. Complete original reports are the declared
artifacts; they remain unchanged under verification/context-compression/compact-a.

<!-- pcg:A2-subject:begin -->
EXP-001-A2 is COMPACT-A2's suitability as a production encoding under its measured
raw-source economics gate. The original phased report says it was stopped and
rejected: two qualifying fixtures were 19–55% larger than raw with cl100k and
18–52% larger with o200k. Exact semantic preservation did not make that encoding
economically admissible. This does not falsify every aspect of codec correctness
or authorize a new encoding. The full report retains scope, controls and limits.

<!-- pcg:A2-subject:end -->
<!-- pcg:TABLE-subject:begin -->
EXP-002-TABLE is the scoped type/callee table's saving proposition on the actual
Phase 3C corpus. Its complete report preserves the negative result: every one of
30 measured rows increased by 54–828 tokens. Short repeated strings already cost
one BPE token; table rows and legend increased the complete payload. The table
implementation and measurement command were removed. This does not establish a
universal prohibition on tables for other identified subjects.

<!-- pcg:TABLE-subject:end -->
<!-- pcg:codec-standing:begin -->
FALSIFIED imports these exact source-supported negative subjects. No new campaign,
model test, current binary measurement or independent certification is claimed.
The governing complete-alternative rule remains in AUTH.TOKEN_ECONOMICS.

<!-- pcg:codec-standing:end -->
```architecture-graph
node: EXP-001-A2
kind: EXPERIMENT
status: FALSIFIED
title: COMPACT-A2 production suitability fails measured raw-source gate
track: REPOSITORY
artifact: verification/context-compression/compact-a/PHASED_A2_PLAN.md
consumes_authority:
  - AUTH.TOKEN_ECONOMICS
note: Source-attributed exact historical subject and limits retained; no current executable claim, independent certification, reclosure or banking.
knowledge_scope: proposition
knowledge_content:
  - statement self#A2-subject
  - qualification self#codec-standing
```

```architecture-graph
node: EXP-002-TABLE
kind: EXPERIMENT
status: FALSIFIED
title: Scoped Phase3C table increases every measured complete payload
track: REPOSITORY
artifact: verification/context-compression/compact-a/A3_PHASE3C_TABLE.md
consumes_authority:
  - AUTH.TOKEN_ECONOMICS
note: Source-attributed exact historical subject and limits retained; no current executable claim, independent certification, reclosure or banking.
knowledge_scope: proposition
knowledge_content:
  - statement self#TABLE-subject
  - qualification self#codec-standing
```
