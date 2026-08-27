# Gated Release Discipline (`docs/agent/releases.md`)

This is the detailed release, changelog, and versioning procedure. It is NOT
always-loaded; consult it when a behavior-affecting build ships.

## Release/build semantics

The CI pipeline releases on every build: every behavior-affecting commit that
passes the gates is a real release, not merely an entry pending under
"Unreleased". Any task constituting such a gated release MUST additionally:

- Add or update its dated, versioned section in `docs/CHANGELOG.md`. An
  "Unreleased" label must not survive past its release.
- Map versions chronologically onto the last released package version: the
  oldest shipped-but-unlabeled commit cluster takes the next patch increment,
  and every later shipped cluster increments by exactly one patch level
  (0.0.1).
- Cross-check `git log` against existing changelog sections so all topics
  introduced by the SAME build share that build's single version number —
  never double-bump one commit across multiple entries.
- Documentation-only and test-infrastructure-only commits consume no version
  slot unless explicitly declared a release.

## Changelog requirements

- Every behavior-affecting build gets its own dated, versioned `## [x.y.z]`
  section in `docs/CHANGELOG.md` describing what changed.
- An "Unreleased" label must not survive past a release; the changelog must
  finish with zero `Unreleased` occurrences.

## Versioning requirements

- Bump `version` in `Cargo.toml`, sync `Cargo.lock`, and confirm the result
  with `cargo pkgid -p clean-ctx`.
- Append one row per release to the Version-history registry under the
  `## Versioning` heading in `docs/CHANGELOG.md` (newest-first). Highlights
  must quote actually shipped changelog content, never invented claims.
- Registry gaps discovered retroactively may be filled only with explicit user
  approval.
- Do not invent release claims or alter the repository's release history.

## Encoding verification after changelog edits

The changelog intentionally carries mojibake quotations behind a registered
allowlist in both guards (`scripts/check-utf8.ps1` and `src/tests/encoding.rs`).
After editing the changelog, pass the encoding gates:

```bash
powershell -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
cargo test encoding
```

These must pass alongside the standard Final Verification Gate (see
`docs/agent/verification.md`).

## The changelog as a code-review surface

The changelog is part of the deliverable. Its "Verification" sections record
focused-suite results, final-gate outcomes, and (where relevant) every
`cargo fmt` / `cargo clippy` / full-suite / encoding-guard result for the
shipped build, so a later release can reconstruct what actually shipped and
how it was verified.