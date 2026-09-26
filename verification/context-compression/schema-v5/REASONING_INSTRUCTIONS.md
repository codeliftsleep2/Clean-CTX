# Clean-CTX SCHEMA-v5 Reasoning Rules

You are evaluating a Clean-CTX context payload and answering a question about
it. Use only the supplied payload; do not inspect files or invoke tools. These
rules are **mandatory** — violating one produces a wrong answer.

---

## RULE 1 — Identity: Typed Owner + Method Name, NEVER Numeric IDs

A method is identified by its **typed owner** (the class line) plus its **method
name** — never a numeric ID. The payload carries no canonical IDs, no call rows,
and no occurrence indices; do not invent them.

```
// ── Alpha ──
M run(+1) → p:value:string → string mod:async
M run(+2) → p:value:number → string
```

`Alpha.run(+1)` and `Alpha.run(+2)` are two distinct overloads of `Alpha.run`.

## RULE 2 — Overloads: `name(+N)`, Never Merge

Same-name methods on one owner are distinguished by **parameter count**, written
`name(+N)`. Never collapse `run(+1)` and `run(+2)` into a single method.

## RULE 3 — Exact Source: Request Edit/Verbatim, NEVER Reconstruct

Low/Medium/High render structural members only — no exact bodies. If a question
asks for a body's exact bytes and the payload is structural, the correct answer
is to request **Edit** (focused) or **Verbatim**. Never reconstruct or invent a
body.

## RULE 4 — Workspace Facts: workspace_query, Not File Content

Cross-file edges, calls, and injections are NOT in a file presentation. They are
answered by `workspace_query` responses, which carry per-entity provenance
(asserting file). Caller identity is coarse-by-name; do not invent a caller or
edge the response does not list.

## RULE 5 — Delta: Summary, Not Op List

Delta content is the minimal summary `Δ delta for <file> (v{from} → v{to}):
+N ~N -N ops`. The structured op list is code-side; do not read it from the
payload.

## RULE 6 — Annotations: Read Flags, Never Infer

Read the annotation groups `mod:` (modifiers), `ctl:` (control summary), `pf:`
(pattern facts), `fl:` (legacy flags), `cf:` (control flow), `df:` (data flow),
`se:` (side effects), `ec:` (execution contexts). Do not infer a fact that is
absent.

## RULE 7 — File Alias: Resolve Through `§PATHMAP`

The final `// αN` line identifies the file by a request-scoped alias. Resolve
its exact source path only through the trailing `§PATHMAP` entry. The alias is
not itself a path, and a path must never be guessed from it.

---

## Quick Reference

| Marker | Meaning |
|--------|---------|
| `// ── ClassName ──` | class boundary |
| `X Parent` / `I Iface` | extends / implements |
| `F name:type` | field |
| `M name(+N)` | method (overload by arity) |
| `→` (first) | scope arrow before `p:` params |
| `→` (second) | return type |
| `mod:` `ctl:` `pf:` `fl:` `cf:` `df:` `se:` `ec:` | annotation groups |
| `$ alias` / `T alias` / `P name` | import / type alias / pattern |
| `// αN` + `§PATHMAP` | file alias + authoritative exact path mapping |
