# CargoCheck incremental parsing and evidence accounting

This phase implements the frozen parsing boundary after owned execution and
bounded capture. It changes no approved budget, invocation, environment,
ownership, cancellation, CLI, or MCP policy.

## Production boundary

The capture framer submits every observed physical frame, including empty
frames and frames wholly discarded after admission exhaustion. Metadata carries
the observed payload length, actual delimiter observation, frame-limit fact,
admission-cut fact, and availability of the complete bounded bytes. Newline
bytes belong to stream capture accounting, not frame payload accounting.

The compiler distinguishes missing delimiters, admission cuts, oversized frames,
known invalid UTF-8, and unavailable decoding. These fault counters may overlap.
Unavailable bytes cannot establish an invalid-UTF-8 claim. Complete late stderr
samples can establish decoding facts, but never become stdout JSON candidates.
Partial, oversized, cut, or undecodable frames are never semantically compiled.
They contribute explicit unparsed frame and observed source-byte counts plus
bounded generic fallback evidence; raw prefixes are withheld.

Complete UTF-8 stdout frames become JSON candidates only when their first
non-whitespace character is `{`. Deserialization preserves serde_json's default
recursion and numeric limits and counts duplicate fields recursively. A record
with duplicate fields is incompatible evidence rather than a last-value-wins
semantic claim. Known record shapes are validated before semantic compilation,
including diagnostic spans and children. Absent optional fields and unrelated
future metadata retain compatibility. Unknown reasons remain sanitized evidence.
Malformed JSON and incompatible shapes produce safe generic summaries without
exporting raw serialized objects. Stderr never contributes Cargo structured facts.

## Coverage denominators

For stdout, the mutually exclusive physical-frame partition is:

```text
stdout_frames = empty_stdout_frames + non_json_stdout + malformed_json
              + parsed_json_objects + unparsed_stdout_frames
json_candidates = malformed_json + parsed_json_objects
parsed_json_objects = compiler_messages + compiler_artifacts + build_scripts
                    + build_finished + unknown_structured + incompatible_structured
```

The fault counters are explanatory overlays, not additional partition members.
`truncated_frames` means a physical frame without an observed final delimiter;
it does not by itself establish whether EOF, cancellation, or read failure caused
that observation. Capture facts carry those independent outcomes.

Terminal stderr samples have separate sample/invalid-sample counters and do not
double-count physical frames. Evidence category counts describe representation
events: a withheld-frame summary and a terminal sample may represent the same
physical frame. They must not be used as parser completeness denominators.
Summary evidence records the original source length and its withheld byte count;
retained bytes measure the safe representation, not a retained raw prefix.

## Verification

Six tracked `incremental_accounting_` regressions failed against the unfixed
implementation at their intended assertions. Only those regressions were
stashed during implementation, then restored unchanged. The identical focused
command passed all six after the fix. Ten additional tracked cases cover mixed
denominators, nested duplicate fields, trailing JSON, stderr isolation, decoding
availability, incompatible shapes, metadata compatibility, and recursion limits.

Owner-authorized Linux verification:

- RED: 0 passed, 6 failed, 3,515 filtered out.
- GREEN: 6 passed, 0 failed, 3,525 filtered out.
- Combined CargoCheck checks: 50 passed, 0 failed, 0 ignored, 3,481 filtered out.
- Package all-target/all-feature Clippy with `-D warnings`: passed, exit 0.

Focused commands for this phase, including an owner-run Windows check:

```text
cargo test --locked -p clean-ctx --lib --all-features -j 4 diagnostics::cargo_check::
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
```

Windows results for this phase are not yet reported. The previous bounded
execution commit `88118b5` was owner-reported GREEN on Windows. The full test
suite remains exclusively a CI gate. No Claude pilot was performed.

## Next boundary

Semantic retention and omission disclosure are implemented in the subsequent
`CARGOCHECK_SEMANTIC_RETENTION.md` phase. Sanitized fallback evidence is implemented in
`CARGOCHECK_FALLBACK_EVIDENCE.md`. Final result budgets are implemented in `CARGOCHECK_RESULT_BUDGETS.md`. This parsing phase alone does not
certify retention or the final structured-result bound. CLI exit codes, typed MCP integration,
production lifecycle trace, and the final bundled Claude pilot remain later work.
