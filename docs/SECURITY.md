# Clean-CTX — Security Guide

> **Owner:** Security model + hardening + compliance · **Status:** Living reference
> **Last updated:** 2026-09-27

---

## Security Posture

Current normative owner: [Network and data-handling boundaries](architecture/contracts/network-effects.md). Original text and chronology are retained at commit `9400d476`; this location is navigation, not a second current contract.

## Compliance Checklist

### Network

Current normative owner: [Network and data-handling boundaries](architecture/contracts/network-effects.md). Original text and chronology are retained at commit `9400d476`; this location is navigation, not a second current contract.

### Supply Chain

| Requirement | Status | Detail |
|-------------|--------|--------|
| Runtime tokenizer downloads | None | BPE data is embedded at compile time. |
| Dependency audit | CI-owned | Consult the current CI run; this guide does not freeze a passing audit claim. |
| License policy | Repository-owned | `deny.toml` and the current CI workflow are authoritative. |

### Code Quality

| Requirement | Status | Detail |
|-------------|--------|--------|
| Warning policy | Zero warnings | The authoritative command and ownership boundary live in `docs/agent/verification.md`. |
| Test status | CI-owned | Consult the current CI run; volatile counts and pass claims are intentionally not duplicated here. |

### Data Handling

Current normative owner: [Network and data-handling boundaries](architecture/contracts/network-effects.md). Original text and chronology are retained at commit `9400d476`; this location is navigation, not a second current contract.

### Source and workspace boundaries

Current normative owner: [Trusted workspace roots](architecture/contracts/trusted-roots.md). Original text and chronology are retained at commit `9400d476`; this location is navigation, not a second current contract.

## Binary Verification

### Checksum and Integrity

The binary is a single statically-linked executable. To verify integrity:

```bash
# SHA-256 checksum after build
sha256sum target/release/clean-ctx.exe
```

### Reproducible Builds

The Rust toolchain is pinned in `rust-toolchain.toml` (or via `rustup default 1.85`). Given the same toolchain and dependencies:

```bash
cargo build --release --locked
```

The `--locked` flag prevents dependency resolution drift by using the exact versions in `Cargo.lock`.

---

## Deployment Guide

### Minimal Deployment

```bash
# Copy the single binary to the target machine
scp target/release/clean-ctx user@target:~/bin/

# Verify it starts
echo '{}' | ~/bin/clean-ctx
# Output: (no response — ctrl-c to exit)
```

### Docker Deployment (Read-Only Filesystem)

```dockerfile
FROM rust:1.85-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM scratch
COPY --from=builder /build/target/release/clean-ctx /clean-ctx
ENTRYPOINT ["/clean-ctx"]
```

The core can run with persistence disabled on a read-only filesystem. Default
persistence, proxy logs/cache, CBM state, and `apply_edit` require writable
locations and are incompatible with this example unless explicitly disabled or
redirected.

### Docker Compose

```yaml
services:
  clean-ctx:
    build: .
    image: clean-ctx:latest
    read_only: true
    stdin_open: true
    tty: true
```

---

## Hardening Recommendations

### Deployment Recommendations

1. **Run with minimal OS privileges** — grant source read access, stdout/stderr,
   and only the specific write locations required by enabled persistence,
   logging, cache, or edit features.

2. **Pin the binary version** — use `cargo build --release --locked` with the committed `Cargo.lock` to ensure deterministic builds

3. **Run `cargo audit` regularly** — as part of CI, or manually:
   ```bash
   cargo install cargo-audit --locked
   cargo audit
   ```

4. **Run `cargo deny check` regularly** — to enforce license policies and ban unsafe dependencies:
   ```bash
   cargo install cargo-deny --locked
   cargo deny check
   ```

5. **Verify no new dependencies** — compare `Cargo.lock` against the last approved baseline before each release

### For Air-Gap Deployment

1. Download all crate dependencies to an air-gapped mirror or vendored directory:
   ```bash
   mkdir -p vendor
   cargo vendor --locked vendor/
   ```

2. Build with offline mode:
   ```bash
   cargo build --release --locked --offline
   ```

3. The resulting binary is fully self-contained and requires no network access.

---

## Vulnerability Disclosure

This project has no external vulnerability reporting process yet. For security issues:

1. Open an issue on the GitHub repository with:
   - Component and version affected
   - Type of vulnerability
   - Steps to reproduce
   - Proof-of-concept (if available)

2. For critical vulnerabilities, contact the maintainers directly through GitHub.

---

## Known Security-Relevant Design Decisions

| Decision | Rationale |
|----------|-----------|
| No `unsafe` blocks | Eliminates memory-safety vulnerabilities by construction |
| `tokio` not used | Avoids async runtime complexity; stdio transport does not benefit from async I/O |
| Explicit write ownership | Persistence, proxy logging/cache, and `apply_edit` write only through their documented owners and configured paths. |
| Config is cached at startup | Prevents TOCTOU (time-of-check-time-of-use) attacks on `.clean-ctx.json` |
| BPE data embedded in binary | Prevents man-in-the-middle attacks on BPE model data at startup |
| Meta-Layer is purely additive | Non-Angular files produce byte-identical output; no risk of silent data corruption |
| WorkspaceIndex is session-owned | Cross-file semantic occurrences are scoped and replaced by asserting file; persisted snapshots restore through the registered lifecycle. |
| No regex in Meta-Layer | Detection and template extraction use string scanning and word-boundary heuristics; avoids ReDoS-class vulnerabilities |
| tree-sitter-html parse-only | Template parser receives HTML and returns structural metadata; no code execution, no network calls |
| Fidelity controls Meta-Layer depth | Low fidelity emits minimal markers; reduces attack surface by limiting the amount of framework metadata exposed |

---

## SBOM (Software Bill of Materials)

Generated via `cargo install cargo-bom` or `cargo sbom`:

```bash
# Generate SPDX SBOM
cargo install cargo-bom
cargo bom --output-path docs/sbom.spdx.json
```

### Core Dependencies

| Crate | Version | License | Purpose |
|-------|---------|---------|---------|
| `tree-sitter` | 0.20.10 | MIT | AST parsing framework |
| `tree-sitter-typescript` | 0.20.5 | MIT | TypeScript grammar |
| `tree-sitter-c-sharp` | 0.20.0 | MIT | C# grammar |
| `tree-sitter-html` | 0.20.0 | MIT | HTML grammar (Angular template parsing) |
| `tiktoken-rs` | 0.11 | MIT | cl100k BPE token counting |
| `serde` | 1.0 | MIT/Apache-2.0 | JSON serialization |
| `serde_json` | 1.0 | MIT/Apache-2.0 | JSON parsing |
| `sha2` | 0.10 | MIT | SHA-256 content hashing |
| `clap` | 4.6.1 | MIT/Apache-2.0 | CLI argument parsing |

All dependencies are MIT or Apache-2.0 licensed. Zero GPL or AGPL dependencies.

### Meta-Layer Dependency Notes

The `tree-sitter-html` crate (added in Phase 2) is the only dependency introduced by the Meta-Layer. It is:
- **Parse-only** — receives HTML content and returns a syntax tree; no file I/O, no network calls
- **Memory-safe** — written in safe Rust with the same tree-sitter runtime used by the TypeScript and C# grammars
- **Pinned version** — `=0.20.0` in `Cargo.toml` prevents automatic upgrades that could introduce behavioral changes
- **Audit-friendly** — `cargo audit` covers it alongside all other dependencies

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.
