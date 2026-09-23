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

Workspace graph edges are outside this grammar. File-local calls and core
injection occurrences are inside it.

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
- `-` is the absent scalar. Empty arrays are represented by no record.
- Record order is semantic wherever the normalized target uses an array.
- Unknown tags, wrong column counts, invalid escapes, invalid handles, dangling
  references, and records illegal for the declared fidelity are errors.

JSON objects are forbidden in A3. JSON string escaping is only a fallback
lexical rule for individual string columns.

## Document framing

```text
A3|3|<L|M|H|E>|<file-id>|<ir-version>|<source-path-json-string>
...
Z|<owner-count>|<method-count>|<call-count>|<body-count>
```

The terminal `Z` record is mandatory. Counts cover decoded semantic records,
not physical lines, and make truncation detectable. Content after `Z` is an
error. A schema number other than `3` is an error rather than an attempted A2
decode.

## Owner scopes

```text
C|<class-id>|<name>|<synthetic-0-or-1>
I|<interface-id>|<name>
X|<parent-reference>
J|<interface-reference>
cm|<value-count>|<value>...
cf|<value-count>|<value>...
D|<dependency-count>|<dependency-reference>...
F|<field-count>|<field-id>|<name>|<type-or-->...
```

`C` or `I` opens the current typed owner scope and closes any prior method and
owner scope. `X` is class/interface extends. `J` is class implements. Repeated
records retain source order. `cm` holds modifier occurrence groups; `cf` holds
legacy class-flag occurrence groups. `D` is one ordered core-injection
occurrence. Empty occurrence groups use a zero count and remain significant.

Fields belong to the current owner. Their canonical field ID is explicit even
when no other record currently refers to it.

## Method scopes and signatures

```text
M|<method-id>|<name>|<declared-arity>|<return-type-or-->
p|<parameter-count>|<parameter-id>|<name>|<type-or-->...
mo|<value-count>|<value>...
cs|<value-count>|<value>...
pf|<fact-count>|<kind>|[<kind-value>]...
lf|<value-count>|<value>...
pt|<pattern-name>|<argument-count>|<argument>...
```

`M` opens a method under the current owner and closes the prior method scope.
One `p` record contains all ordered parameters. Low may omit it for a unique-name
method, but `declared-arity` remains exact; Low includes parameter rows for
same-owner overload families so overload signatures remain distinguishable.
Medium, High, and Edit include all parameter rows.

Physical row order is the occurrence index. Empty and duplicate occurrence
groups remain explicit rows and significant. Pattern-fact kinds are `CTOR`, `OBSERVABLE`, `OVERRIDE`,
`GETTER`, and `SETTER`; only getter/setter consume the following value column.
Values are positional strings, never copied named objects.

## High/Edit behavior records

```text
fc|<kind>|<target>
fd|<direction>|<target>
se|<value>
ec|<value>
```

These records are legal only in High and Edit. Physical order within each
method and family reconstructs exact zero-based ordinals. Family records are
contiguous, so a missing or reordered physical row changes the decoded ordered
array rather than being silently normalized.

## File records

```text
$|<alias-or-->|<module-or-->|<named-export-or-->
T|<alias>|<original-type>
```

Import and alias occurrence order is physical record order; no redundant
occurrence column is serialized. Values are
positional JSON-string columns; named objects are forbidden.

## Local call stream

```text
K|<caller-method-id>|<call-count>|<callee-written-name>|<explicit-argument-count>|[*]...
```

`K` contains one consecutive caller run. A later `K` may return to an earlier
caller; physical entry order is the authoritative global occurrence order.
Each entry is unresolved by definition. `*` follows only a spread call.
Duplicates remain duplicate entries. The count makes truncation and trailing
columns invalid.

This removes repeated occurrence numbers, repeated caller IDs, repeated
`false`, and repeated `"unresolved"` while retaining every canonical fact.

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
| Class/interface/method occurrences | yes | yes | yes | yes |
| Patterns and pattern facts | yes | yes | yes | yes |
| Core injection occurrences | yes | yes | yes | yes |
| File-local calls/arity/spread | yes | yes | yes | yes |
| Imports/type aliases | yes | yes | yes | yes |
| Control/data flow | no | no | yes | yes |
| Side effects/execution contexts | no | no | yes | yes |
| Exact bodies/spans | no | no | no | selected/all |
| Workspace semantic edges | query | query | query | query |

Medium differs from Low through complete signatures. High adds the detailed
behavior families. Edit is High plus exact body frames. Verbatim is byte-exact
raw source and does not use A3.

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
