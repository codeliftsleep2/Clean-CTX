# Pre-pilot correction: workspace paths in producer text

Review of the owner's Windows lifecycle failure evidence showed that retained
Cargo stderr exposed the absolute admitted workspace in progress and failure
lines. Typed span locations already used workspace-relative/redacted display,
but opaque evidence and diagnostic strings only underwent terminal normalization
and recognized-secret redaction. This was a gap in the approved logical workspace
display policy, not a new authority or operational-policy decision.

The compiler now supplies admitted workspace context to the evidence collector
and string sanitizer. After terminal normalization, occurrences of that known
absolute workspace root are replaced with `<workspace>` before secret redaction,
retention, text rendering, or result-budget enforcement. The transformation applies
to retained evidence, diagnostic messages/rendered evidence, child messages,
span labels, and retained suggestions. Standalone compilers without admitted
authority retain their prior behavior.

Windows presentation recognizes the admitted canonical verbatim spelling, its
ordinary drive/UNC spelling, forward-slash spellings, and ASCII case variants.
This is display normalization of already-admitted authority; it performs no
filesystem discovery and supplies no executable/workspace fallback. A suffix
boundary prevents a sibling directory sharing the prefix from being mislabeled
as the workspace. Typed external paths retain their existing redacted handling.
This correction does not introduce a universal parser for arbitrary paths in
opaque producer text.

Existing `workspace_paths_mapped` transformation facts count these replacements.
Capture and original evidence source-byte counts remain based on producer bytes;
sanitized byte counts and the complete structured/text budgets use displayed
strings. The fixed invocation, explicit child environment, ownership, cancellation,
capture/frame budgets, retention limits, and projection limits are unchanged.

Two tracked regressions under `src/tests/diagnostics/cargo_check_opaque_paths.rs`
failed at intended assertions before production changes. The regression file and
registration were stashed during implementation, then restored unchanged. The
same command passed both tests. Three additional rendering checks cover Windows
drive/UNC spellings and a UTF-8 sibling-prefix boundary; all five focused path
checks passed on Linux. Windows spelling checks exercise pure rendering logic on
Linux and do not substitute for the owner's Windows execution verification.

The authorized focused CargoCheck core run passed all 102 tests. Package Clippy
with all targets/features and `-D warnings` passed without warnings. The full
test suite was not run locally; it remains reserved for CI.

The frozen Claude pilot remains unrun. Its prepared handoff is
`CARGOCHECK_CLAUDE_PILOT.md`. Confirm this correction on Windows and the CI gate
before proceeding; the full suite remains CI-only.
