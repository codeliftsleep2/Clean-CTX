# Occurrence-aware sequence delta transport

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

### Corrected Sequence Delta Envelope (`dv:2`)

```json
{
  "dv": 2,
  "file": "α1",
  "from": 1,
  "to": 2,
  "target_hash": "…",
  "edits": [
    {
      "op": "insert",
      "at": 2,
      "value": {
        "identity": {"opcode": "DEF_METHOD", "key": ["DEF_M", "C1", "M3"]},
        "occurrence": 0
      },
      "instruction": ["DEF_M", "C1", "M3", "newMethod"]
    }
  ],
  "intent": null
}
```

Legacy `+` / `~` / `-` envelopes are decode-only compatibility input.
Production generation emits only occurrence-aware `dv:2` sequence edits.


Source: [docs/COMPILER_IR.md](../../COMPILER_IR.md); original section SHA-256 `337ab03c1f40df95cdbc975ed52a83e1d4eedffcc6bfd5137c83954724ada335`. Original wording and tense are retained; admission is limited to the implemented scope described here.


```architecture-graph
node: AUTH.DELTA_SEQUENCE
kind: AUTHORITY
status: IMPLEMENTED
title: Occurrence-aware sequence delta transport
track: REPOSITORY
artifact: docs/architecture/contracts/delta-sequence.md
preserves:
  - IRDELTA-001
  - IRDELTA-002
  - IRDELTA-003
  - DELTA-001
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
