# CargoCheck final result budgets

This phase follows fallback evidence commit `1433e82`. It implements the
approved 512 KiB serialized structured limit and 24 KiB/240-line text limit.
CLI dispatch, CLI exit codes, and MCP registration remain subsequent phases.

## Enforced boundaries

The semantic compiler now performs final optional-detail reduction before
returning its internal result. A counting writer measures UTF-8 JSON bytes,
including escaping and disclosure metadata, without allocating an oversized
serialized buffer. The serialized-byte disclosure is measured to a fixed point
so its own digit count is included.

`project_cargo_check` is the final adapter boundary. It clones the internal
semantic result, measures the full structured envelope, and returns a checked
JSON byte buffer and separately bounded text. The envelope includes invocation
authority, actual root outcome, ownership, cleanup/cancellation, capture facts,
the intrinsic-timeout fact, semantic results, and text-budget disclosure.
Projection never mutates the execution record or its parser/producer facts.

Invocation result facts carry logical workspace position, approved source and
file identities, sanitized Cargo basename, the fixed command, and sanitized
environment facts with separate transformation counts. Canonical paths and
child environment values are not serialized. Facts describe the admitted
invocation that was revalidated at launch; they do not invent startup source
shadowing or additional rustup observations not yet implemented.

Adapters must consume the checked projection. Direct serialization of internal
`CargoCheckExecution` is not a bounded wire projection. For a smaller explicit
local policy, the internal semantic result can carry a mandatory-budget failure;
the projection boundary refuses to emit it when mandatory facts cannot fit.
The approved production policy remains unchanged.

## Approved reduction order

If the measured result exceeds the limit, reduction proceeds deterministically:

1. Remove optional rendered diagnostics.
2. Remove evidence fragments, preserving all category coverage/omission facts.
3. Remove child detail and update child/suggestion omissions.
4. Remove related spans and update span/suggestion omissions.
5. Remove remaining suggestions and update occurrence and replacement-byte facts.
6. Remove lower-priority diagnostics: warnings before errors; preserve the first
   causal and last terminal error while dropping intervening errors where possible.

Each stage removes its optional field category, then remeasures the whole result.
Authority, process, capture, producer assertions, parser coverage, transformation
counts, and omission disclosure are never removed. If only mandatory facts remain
and still exceed the limit, return `BoundedResultError::MandatoryFacts` rather
than an oversized successful projection. Serialization failures also return a
bounded static error, without exporting producer strings.

Suggestion records carry original replacement-byte size and their retained edit
slot. These facts let later reductions preserve source-byte omission totals and
distinct-edit counts despite redaction or removal of duplicate edit occurrences.
Diagnostic selection labels let final reductions keep head/tail counts accurate.
Removing a compacted diagnostic converts every represented occurrence to an
omission, preserving the seen/retained/repeat/omitted equation.

Budget disclosure records original detail counts, semantic-core pre-reduction
serialized size, activation, reduced fields, and final serialized bytes. In a
checked projection, final serialized bytes describe the complete envelope; the
pre-reduction byte count describes the semantic core before its initial reduction.
A labeled, sanitized causal preview uses the existing text-byte limit. It allows
text to retain the first causal message even when the complete diagnostic cannot
fit structured output. Preview source length and truncation are disclosed.

## Text contract

Text always reports invocation authority, OS process outcome separately from
Cargo's completion assertion, ownership/cancellation/cleanup facts, incomplete
coverage, capture loss, diagnostic omissions, and structured-budget activation.
Non-JSON stdout remains visible as incomplete semantic coverage even if a host
ignores structured evidence.

Messages and the Cargo basename are quoted so embedded newlines cannot forge
authority lines. Message excerpts are marked when clipped. Text checks both
UTF-8 bytes and actual rendered line count, preserves the first causal message,
and reserves a final truncation/count statement pointing to the bounded
structured result. Text facts record bytes, lines, activation, represented versus
available diagnostics, and whether only a causal preview was shown.

If mandatory text or the first causal label/excerpt cannot fit an explicit local
text policy, return `BoundedResultError::MandatoryText`. The projection returns
no partially approved artifact on either structured or text failure.

## Verification

Four tracked `result_budget_regression_` tests failed against the unfixed code
at their intended serialized-size assertions. The exact tests and registration
were stashed and restored unchanged. GREEN covers oversized rendered evidence,
JSON escaping, evidence-item metadata, and accumulated diagnostic messages.

Seven additional tracked projection cases cover whole-envelope measurement and
authority/process preservation, text line limits and newline escaping, mandatory
failure, ordered nested reductions and source-byte accounting, huge causal
previews, occurrence/head-tail equations, and non-JSON visibility in text.

Owner-authorized Linux results:

- RED: 0 passed, 4 failed, 3,557 filtered out.
- Unchanged GREEN: 4 passed, 0 failed, 3,563 filtered out.
- Final combined CargoCheck checks: 87 passed, 0 failed, 0 ignored, 3,481 filtered out.
- Package all-target/all-feature Clippy with `-D warnings`: passed, exit 0.

The initial Clippy run identified a loop-style lint and a redundant closure.
Both were corrected without suppression before final verification.
Windows verification remains owner-run with:

```text
cargo test --locked -p clean-ctx --lib --all-features -j 4 diagnostics::cargo_check::
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
```

No full test suite or Claude pilot was run. The full suite remains exclusively
the CI gate.

## Next boundary

Implement CLI projection and the approved exit-code contract around the prepared
invocation, owned execution, and checked projection. Then integrate the typed MCP
tool, complete production lifecycle verification, and finally conduct the bundled
Claude pilot. Startup authority must remain explicit, with no heuristic fallback
or caller-supplied Cargo argument surface.
