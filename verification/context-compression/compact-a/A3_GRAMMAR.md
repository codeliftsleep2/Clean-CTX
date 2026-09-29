# COMPACT-A3 grammar and fidelity target

**Status:** Phase 0 specification; no production encoder is authorized yet

**Economics status:** the first complete A3 encoding failed the Phase 3 gate.
The production-selected outputs over the qualifying representative corpus must
save at least 50% in aggregate against the same raw corpus. Each invocation is
independently protected by the raw-economics gate, so a losing candidate emits
raw and contributes zero savings rather than inflation. Fidelity/intent rows
remain separately reported diagnostics, not individual 50% gates.

## Authority and decoded target

A3 is a presentation codec over checked file-local state. It does not alter
canonical IR, semantic identity, persistence, or workspace-query authority.
Its decoder produces a fidelity-specific normalized semantic object. Missing
families are declared unavailable for that fidelity; they are never decoded as
authoritative empty arrays.

Workspace graph edges, file-local calls, and core injection occurrences are
outside this grammar; they are served on demand by `workspace_query`.

## Identity renumbering (Phase 3B)

Canonical identities are the compiler's global `next_id` handles (`C487`,
`M1204`, `F82`, `P633`). The A3 wire spells them as **dense file-local ordinals**
assigned by first appearance, per family:

| Family | Local ordinal |
|---|---|
| class | `C1..CK` |
| interface | `I1..II` |
| method | `M1..MM` (file-local, across owners) |
| field | `F1..FF` (file-local, across owners) |
| parameter | `P1..PP` (method-local) |

Ordinals are assigned in wire (decode) order: owners in `C`/`I` record order,
fields then methods in their scoped order, parameters in `p`-row order. The wire
keeps the bare ordinal exactly as before; only the number changes. Local handles
are payload-local: never persisted, never accepted as selectors, never returned
as workspace identity.

**Decode equality is alpha-normalized for the identity family only.** A decoded
document's IDs are the local ordinals, not the canonical handles; equality against
the normalized oracle is modulo this bijective renumbering. Every other family
(names, order, duplicates, occurrence groups, spans, bodies) remains literal-exact.

**Name-vs-ID references.** `X`/`J` (extends/implements) may carry either a
canonical ID or a written name. The renumber rewrites a value only when it
exactly equals a declared ID in the same family; other values are left opaque.
Definitely-ID references (`B` body) fail closed on a dangling miss.

## Lexical rules

- The stream is strict UTF-8 without a BOM.
- Records are separated by LF. CR is forbidden outside a framed body.
- `|` separates columns.
- Typed declaration/scope records omit the redundant family prefix from
  canonical handles (`C`, `I`, `F`, `M`, `P`); decoding restores it from the
  record type. Other string-valued references retain their exact spelling.
- Strings without `|`, CR, or LF use their bare UTF-8 spelling. Empty strings,
  the reserved absent marker `-`, strings beginning with `"`, and strings with
  delimiters use JSON string escaping and include their quotes.
- Integers are unsigned base-10 without leading zeroes, except `0`.
- Occurrence group records (`cm`, `cf`, `mo`) omit the count column when the
  group holds exactly one value whose leading column is not a bare unsigned
  integer; the count is otherwise required and a bare unsigned integer in the
  first column is always read as a count. Both spellings decode identically.
- `-` is the absent scalar. Empty arrays are represented by no record.
- Record order is semantic wherever the normalized target uses an array.
- Unknown tags, wrong column counts, invalid escapes, invalid handles, dangling
  references, and records illegal for the declared fidelity are errors.

JSON objects are forbidden in A3. JSON string escaping is only a fallback
lexical rule for individual string columns.

## Document framing

```text
A3|4|<L|M|H|E>|<file-id>|<ir-version>|<source-path-json-string>
...
Z|<owner-count>|<method-count>|<body-count>
```

The terminal `Z` record is mandatory. Counts cover decoded semantic records,
not physical lines, and make truncation detectable. Content after `Z` is an
error. A schema number other than `4` is an error rather than an attempted A2
decode.

## Owner scopes

```text
C|<class-id>|<name>|<synthetic-0-or-1>
I|<interface-id>|<name>
X|<parent-reference>
J|<interface-reference>
cm|<value-count>|<value>...
cf|<value-count>|<value>...
F|<field-count>|<field-id>|<name>|<type-or-->...
```

`C` or `I` opens the current typed owner scope and closes any prior method and
owner scope. `X` is class/interface extends. `J` is class implements. Repeated
records retain source order. `cm` holds modifier occurrence groups; `cf` holds
legacy class-flag occurrence groups. Empty occurrence groups use a zero count
and remain significant.

Fields belong to the current owner. Their canonical field ID is explicit even
when no other record currently refers to it.

## Method scopes and signatures

```text
M|<method-id>|<name>|<declared-arity>|<return-type-or-->
p|<parameter-count>|<parameter-id>|<name>|<type-or-->...
mo|<value-count>|<value>...
```

`M` opens a method under the current owner and closes the prior method scope.
One `p` record contains all ordered parameters. Low may omit it for a unique-name
method, but `declared-arity` remains exact; Low includes parameter rows for
same-owner overload families so overload signatures remain distinguishable.
Medium, High, and Edit include all parameter rows.

Physical row order is the occurrence index. Empty and duplicate occurrence
groups remain explicit rows and significant. Values are positional strings,
never copied named objects.

## File records

```text
$|<alias-or-->|<module-or-->|<named-export-or-->
T|<alias>|<original-type>
```

Import and alias occurrence order is physical record order; no redundant
occurrence column is serialized. Values are
positional JSON-string columns; named objects are forbidden.

## Exact Edit bodies

```text
B|<method-id>|<start-byte>|<end-byte>|<utf8-byte-length>\n
<exactly utf8-byte-length bytes>
```

Body bytes are not delimiter parsed. The decoder consumes exactly the declared
byte length, after which the next byte must begin the next LF-delimited record.
The method must exist and be selected by canonical identity. Start/end spans
remain numeric and exact. CRLF, Unicode, braces, pipes, and record-like text
inside the body are opaque bytes.

Low, Medium, and High reject `B`. Unfocused Edit requires one body for every
available method. Focused Edit requires exactly the resolved target set and no
other bodies.

## Fidelity target matrix

| Family | Low | Medium | High | Edit |
|---|---:|---:|---:|---:|
| File identity/path/version | yes | yes | yes | yes |
| Typed class/interface identity | yes | yes | yes | yes |
| Fields and ownership | yes | yes | yes | yes |
| Method identity/name/arity/return | yes | yes | yes | yes |
| Parameter rows | overload families | all | all | all |
| Extends/implements | yes | yes | yes | yes |
| Class/interface/method modifiers | no | semantic only | yes | yes |
| Imports/type aliases | yes | yes | yes | yes |
| Exact bodies/spans | no | no | no | selected/all |
| Workspace edges, calls, injections | query | query | query | query |

Medium and High differ from Low through complete signatures. Modifiers are the
remaining Medium-vs-High distinction: Low drops every `cm`/`mo` modifier;
Medium drops access/visibility and declaration-kind modifiers (`EXPORT`,
`STATIC`, `PRIVATE`, `PROTECTED`, `ABSTRACT`) and keeps the semantic ones
(`ASYNC`, `GEN`, `UNSAFE`); High and Edit keep all. `cf` (legacy class-flag
occurrences) is not fidelity-filtered. Edit adds exact body frames. Verbatim
is byte-exact raw source and does not use A3.

## Navigation policy

A3 initially has no navigation section. Scoped owner/method layout supplies
locality without duplicating facts. A navigation record may be introduced only
after a controlled reasoning failure, must reference typed identities without
copying values, and requires a new schema version if it changes grammar.

## Decoder rejection requirements

The decoder rejects:

- missing header or terminal record;
- unknown schema/fidelity/tag;
- count mismatches or trailing content;
- owner/member records outside their required scope;
- duplicate canonical declarations;
- wrong-family or dangling references;
- malformed counts or order-changing missing records;
- illegal fidelity families;
- malformed JSON-string columns;
- missing, duplicate, truncated, or overlong body frames;
- body spans inconsistent with the normalized oracle.

## Cold legend draft

The Phase 1 encoder will tokenize a concise legend derived from this grammar.
The complete cold header plus legend must target 200 tokens or fewer and may
not exceed 300. This full specification is decoder/test authority; it is not
intended to be copied verbatim into every model response.
