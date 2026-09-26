# SCHEMA-vNext Phase 2 — B3 grouped fields

**Status:** Economics and paired reasoning passed; combined selection pending
**Recorded:** 2026-09-26
**Production renderer:** Unchanged
**Model calls:** 16 isolated paired-gate calls in the accepted run

## Candidate

B3 extends Low fidelity's existing grouped-field presentation to the other
body-free fidelities:

```text
F id:number
F name:string
F email:string
```

becomes:

```text
F id:number name:string email:string
```

Field order and field payload bytes remain unchanged. Only consecutive field
rows belonging to the same rendered owner are grouped. Class/interface
boundaries and every non-field row terminate a group.

Low already uses this grammar and therefore needs no field transformation.
Medium and High are transformed. Edit remains one-field-per-line because it is
body-bearing, including focused Edit; Verbatim bypasses structural rendering.

The isolated candidate changes the cold header version to `SCHEMA vNext` but
does not alter its legend: `F=field` already describes both the existing Low
form and the candidate grouped form. This keeps B3 independent from B1 and B2.

## Corpus audit

Across the 15 current production candidates, Medium and High contain:

| Fixture family | Visible field rows | Owner-local runs | Removable row breaks |
| --- | ---: | ---: | ---: |
| Angular | 140 | 14 | 126 |
| C# | 66 | 12 | 54 |
| TypeScript | 12 | 2 | 10 |
| **Total** | **218** | **28** | **190** |

The Edit captures expose the same structural field counts, but B3 deliberately
does not transform them. Low contains one already-grouped row for each of the
same owner-local runs.

The reasoning risk is higher than B1/B2. Several field payloads contain spaces
or modifiers and many interfaces/classes are adjacent. Token savings cannot
justify B3 if a model merges fields, loses order, or assigns a group to the
wrong owner.

## Measurement assertions

`Measure-B3GroupedFields.ps1` aborts unless:

- the exact current SCHEMA-v5 header occurs once;
- the capture fidelity agrees across both tokenizer records;
- Medium/High candidates preserve every field payload in order while reducing
  each multi-row run to one `F` row;
- Low and Edit candidates preserve the complete presentation after the header;
- field grouping never crosses a non-field row;
- exact Edit bodies remain byte-identical; and
- every candidate begins with the complete vNext header.

The script writes generated candidates and token records beneath `target/` and
makes no model or Clean-CTX calls.

## Next gate

The complete candidate passes the economics gate wherever B3 applies:

| Fixture | Medium/High rows | Tokens saved | Best reduction |
| --- | ---: | ---: | ---: |
| Angular | 70 → 7 | 126 | 9.67% |
| C# | 33 → 6 | 54 | 3.04% |
| TypeScript | 6 → 1 | 10 | 1.29% |

Absolute savings are identical under cl100k and o200k. Low and both Edit lanes
remain byte-identical after the version spelling and therefore save zero, as
intended. B3 does not rely on body-bearing output for its economics.

`Prepare-B3ReasoningWorksheet.ps1` creates eight semantic cases, each evaluated
against the captured SCHEMA-v5 baseline and exact B3 candidate. The 16-call
gate covers field order, multi-word/modifier boundaries, adjacent class and
interface owners, repeated field names, and negative membership questions.

The accepted restarted gate passed 16/16 with no failure categories:

| Semantic case | SCHEMA-v5 | B3 candidate |
| --- | --- | --- |
| Angular field order | Pass | Pass |
| Angular modifier-bearing field boundaries | Pass | Pass |
| Adjacent Angular interface ownership | Pass | Pass |
| Angular negative field membership | Pass | Pass |
| Adjacent C# class ownership | Pass | Pass |
| C# service/controller ownership with repeated `_logger` | Pass | Pass |
| TypeScript modifier stripping and field order | Pass | Pass |
| TypeScript negative constructor-parameter membership | Pass | Pass |

An earlier 14/16 run is rejected as evaluation evidence because two questions
had defective expectations: one omitted three real private C# constant fields,
and one did not ask the model to remove TypeScript modifiers before comparing
base identifiers. The worksheet was corrected from source authority and the
entire gate was restarted; no prior answers were reused.

B3 clears its isolated laboratory gate. Production remains unchanged. The next
gate is selection and combined measurement/reasoning for the independently
proven B1+B2+B3 grammar.
