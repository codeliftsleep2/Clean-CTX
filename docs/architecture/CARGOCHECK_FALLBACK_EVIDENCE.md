# CargoCheck sanitized fallback evidence

This phase follows semantic retention commit `653be25`. It implements the
approved evidence reservations, selection, and disclosure inside the existing
CargoCheck boundary. It changes no capture, frame, execution, invocation,
environment, cancellation, or public adapter policy.

## Bounded selection

Every evidence event is normalized and redacted before fragment selection,
including events observed after a reservation fills. The collector retains only
sanitized fragments, counts every event, and rotates tails rather than stopping
at the first full budget. Empty sanitized records retain no item or identity;
they contribute an explicit empty-record counter.

| Category | Reservation | Selection |
|---|---:|---|
| stderr | 48 KiB | 16 KiB stable head, 32 KiB rotating tail |
| stdout non-JSON | 16 KiB | 8 KiB head, 8 KiB tail |
| malformed/truncated/decoding | 16 KiB | first fault exemplars plus head/tail |
| unknown/incompatible structured | 8 KiB | 4 KiB summary head, 4 KiB summary tail |
| authority/completion mismatch | 8 KiB | reserved 4 KiB head, 4 KiB tail |

Anomalies use half their reservation for first exemplars and half for equal
head/tail context. The exemplar half is divided among the six existing fault
classes: frame limit, admission cut, missing delimiter, unavailable decoding,
invalid UTF-8, and malformed JSON. Repeated fault floods cannot evict a different
class's first exemplar. Class counts disclose the selected classification; the
parser's overlapping fault counters remain the authoritative detailed coverage.
When multiple faults apply, classification uses that listed priority order.
Exemplar and head/tail fragments never overlap within an event.

The fixed reservations sum to the approved 96 KiB sanitized text limit. No unused
reservation is borrowed; `borrowed_bytes` is explicitly zero. This is permitted
by the approved conditional borrowing policy. A smaller explicit local shared
budget is also enforced at finish, with omissions recomputed; the production
approved budget remains 96 KiB. Fragment metadata and JSON encoding are outside
this text-byte count and require the subsequent final result-budget boundary.

## Fragment truth and secrecy

Head and tail ranges remain separate items, even when they come from one frame.
Each item carries sanitized source-range offsets and original sanitized record
length, source size, selection kind, observation kind, and source-withheld flag.
Anomaly items carry their classified kind. UTF-8 boundaries are preserved.
Nonadjacent fragments are never concatenated into a fabricated continuous string.
Later CLI/MCP projection must preserve or disclose those gaps.

`producer_order` is the collector's observation order, preserving per-stream
observation order. It does not claim a synchronized chronology between stdout
and stderr. Terminal stderr samples are distinct evidence events and share the
same tail reservation; no additional sample budget or raw spool is introduced.
Invalid terminal samples emit only a safe marker with the actual sample source
length, never lossy decoded bytes.

The shared sanitizer returns sanitized text or fails before selection; there is
no exception path returning the original producer text. Static redaction rules
are initialized by the existing native-text boundary. Partial or undecodable
producer frames remain generic markers, and unknown JSON remains a sanitized
summary rather than a serialized raw object.

## Accounting domains

Category counters measure evidence events, not physical frames or Cargo records.
A summary and a late sample may represent the same physical frame. Conversely,
one event can contribute multiple nonoverlapping fragments. Retained record
counts count unique represented events rather than item count.

`original_bytes` counts supplied source lengths: complete producer text includes
original whitespace; withheld-frame/JSON summaries use the observed source size;
generated authority-mismatch text uses its generated message size. It is not a
claim that evidence events reconstruct the complete captured transcript.

The explicit sanitized byte domain is:

```text
sanitized_bytes = retained_bytes + omitted_sanitized_bytes
retained_bytes = head_bytes + tail_bytes + exemplar_bytes
original_records = retained_records + omitted_records + sanitized_empty_records
```

`partially_retained_records` counts represented events with omitted sanitized
ranges. `withheld_source_bytes` separately counts original source sizes replaced
by safe summaries. Normalization and redaction can change length, so source bytes
must not be subtracted from sanitized representation bytes. The compatibility
`omitted_bytes` field remains the sum of withheld source bytes and omitted
sanitized representation bytes; it is not a completeness denominator. The new
explicit domain counters should be used by subsequent projections.

## Verification

Five tracked `evidence_regression_` tests failed at their intended assertions
before production edits. The exact tests and registration were stashed and
restored unchanged. GREEN verifies late stdout retention, the approved stderr
split, complete source-length accounting, empty-record storage, and separate
head/tail ranges.

Eight additional tracked cases cover independent category budgets, Unicode
fragment ranges, tail eviction, rare anomaly exemplars, pre-selection redaction,
empty normalization accounting, invalid terminal sample secrecy/source size,
and a smaller explicit shared budget.

Owner-authorized Linux verification:

- RED: 0 passed, 5 failed, 3,544 filtered out.
- Unchanged GREEN: 5 passed, 0 failed, 3,544 filtered out.
- Combined CargoCheck checks: 76 passed, 0 failed, 0 ignored, 3,481 filtered out.
- Package all-target/all-feature Clippy with `-D warnings`: passed, exit 0.

Windows verification remains owner-run with:

```text
cargo test --locked -p clean-ctx --lib --all-features -j 4 diagnostics::cargo_check::
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
```

No full suite or Claude pilot was run. The full suite remains exclusively the CI
gate. A transient initializer-placement compile error in the additional metadata
was corrected before the combined 76-test GREEN run; it is not RED evidence.

## Next boundary

Enforce final result budgets: 512 KiB structured output and 24 KiB/240-line text.
Authority, process, capture, completeness, and omission facts must survive the
approved optional-detail reduction order. If mandatory facts cannot fit, return
the approved bounded internal failure. CLI projection/exit codes, typed MCP
integration, production lifecycle verification, and the final bundled Claude
pilot remain subsequent work.
