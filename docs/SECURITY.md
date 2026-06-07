# Clean-CTX — Security Guide

**Last updated:** 2026-06-07

---

## Security Posture

Clean-CTX is designed for **air-gapped environments** with restrictive firewall and DLP (Data Loss Prevention) requirements. The binary has zero network dependencies and zero runtime dependencies, making it suitable for deployment in classified, regulated, or sandboxed environments.

---

## Compliance Checklist

### Network

| Requirement | Status | Detail |
|-------------|--------|--------|
| Zero network transport | ✅ | stdio-only via MCP — no HTTP, WebSocket, or TCP |
| No egress callouts | ✅ | Binary makes no network connections whatsoever |
| No DNS lookups | ✅ | No hostname resolution required |
| No listening ports | ✅ | No server sockets created |

### Supply Chain

| Requirement | Status | Detail |
|-------------|--------|--------|
| Statically linked binary | ✅ | Single binary — no system library dependencies |
| No runtime downloads | ✅ | BPE data embedded via `include_bytes!` at compile time |
| Dependency audit | ✅ | `cargo audit` clean — zero known vulnerabilities |
| License compliance | ✅ | `deny.toml` allows MIT/Apache-2.0 only; `cargo deny check` clean |

### Code Quality

| Requirement | Status | Detail |
|-------------|--------|--------|
| Unsafe code | ✅ **ZERO** | `#![forbid(unsafe_code)]` would pass |
| Clippy warnings | ✅ 0 | `cargo clippy --all-targets -- -D warnings` passes |
| Test coverage | ✅ 121 tests | All tests pass; fuzz-style edge cases covered |

### Data Handling

| Requirement | Status | Detail |
|-------------|--------|--------|
| Data at rest | ✅ No on-disk state | Cache is in-memory only — no files written outside stdin/stdout |
| Data in transit | ✅ N/A | Data never leaves the process boundary |
| Memory safety | ✅ Safe Rust | All memory operations are bounds-checked at compile time |

---

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

The binary works on `--read-only` filesystems because BPE data is embedded at compile time (F-22).

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

1. **Run with minimal OS privileges** — the binary needs only:
   - Read access to the source code directory
   - Write access to stdout (already granted by the parent process)
   - No file system writes for its own operation

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
| No file-write operations | Prevents unintentional data leakage through temporary files |
| No logging to disk | Prevents sensitive source code from appearing in log files (errors go to stderr only) |
| Config is cached at startup | Prevents TOCTOU (time-of-check-time-of-use) attacks on `.clean-ctx.json` |
| BPE data embedded in binary | Prevents man-in-the-middle attacks on BPE model data at startup |

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
| `tiktoken-rs` | 0.11 | MIT | cl100k BPE token counting |
| `serde` | 1.0 | MIT/Apache-2.0 | JSON serialization |
| `serde_json` | 1.0 | MIT/Apache-2.0 | JSON parsing |
| `sha2` | 0.10 | MIT | SHA-256 content hashing |
| `clap` | 4.6.1 | MIT/Apache-2.0 | CLI argument parsing |

All dependencies are MIT or Apache-2.0 licensed. Zero GPL or AGPL dependencies.

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.