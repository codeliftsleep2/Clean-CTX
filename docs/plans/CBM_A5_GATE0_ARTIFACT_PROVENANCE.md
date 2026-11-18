# A5 Gate 0 — cloud pinned artifact provenance

Date: 2026-10-11 UTC. Gate 0 only; live acceptance **NOT RUN**.

## Source verification

- Repository: `https://github.com/codeliftsleep2/codebase-memory-mcp-clean-ctx.git`.
- Authority repository: `/workspace/cbm-authority-audit`.
- Exact HEAD: `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`.
- Git tree: `600104b68d2cddbfcfde8d9f5ab0b85121e0024c`.
- Build source: `/workspace/cbm-authority-source`, an extracted tree rather than
  a separate Git checkout. Its identity was verified against the authority
  repository, not inferred from its directory name.
- All 2,500 tracked blobs, symlink targets and executable permissions matched
  that Git tree before building. The same verification passed after building.
- Post-build extra files were confined to `build/a5-gate0/`; tracked source
  content and modes were unchanged. No source substitution, pull or update.
- Clean-CTX branch: `feature/current-cbm-pinned-upgrade`, commit
  `593a2c7a3dc0e6a74d27648be68eb023b6028530`; required parent
  `4c1627fcefe698977447dcee4e88830c20c7f422` is an ancestor. A0–A4 commits are
  `87cbd04`, `cb005bc`, `14d05cc`, `3fe756d`, `593a2c7`.

## Build authority and commands

Pinned `README.md` lines 418–438 documents prerequisites and both supported
build compositions. `scripts/build.sh` identifies itself as the canonical
production-build entry and invokes `Makefile.cbm` target `cbm` without UI.
The supported `BUILD_DIR` override scopes cleanup and output to one direct
child of `build/`, validated by `scripts/path-safety.sh`.

Selected the smallest supported frontend build: standard native production
`-O2` compilation without optional graph UI. README calls this UI-free
composition development-only; it is **not** the UI-enabled packaged release.
No test seams, coverage or memory-profiling modes were enabled. The default
configuration stamp contains a test sanitizer setting, but the production
compile/link command does not use sanitizer flags or sanitizer libraries.

Build command, working directory `/workspace/cbm-authority-source`:

```bash
mkdir -p /workspace/cbm-gate0-evidence
CBM_NO_CCACHE=1 bash scripts/build.sh --version 0.11.0 BUILD_DIR=build/a5-gate0 CC=gcc CXX=g++ > /workspace/cbm-gate0-evidence/build.log 2>&1
```

Observed exit code **0**, ending with `Built: build/a5-gate0/codebase-memory-mcp`
and the script's build-complete line. Five-way parallelism came from the
supported script's detected `nproc`. Compiler caching was disabled. No dependency
restoration commands were needed: compilers, Make, Git and zlib headers/libraries
were already present; other implementation dependencies are vendored.

Read-only provenance verification commands:

```bash
git -C /workspace/cbm-authority-audit remote -v
git -C /workspace/cbm-authority-audit rev-parse HEAD
git -C /workspace/cbm-authority-audit rev-parse 'HEAD^{tree}'
python3 /workspace/cbm-gate0-evidence/verify-source.py > /workspace/cbm-gate0-evidence/source-verification.json
sha256sum /workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp
stat -c '%s bytes; %y' /workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp
file /workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp
readelf -d /workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp
readelf -l /workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp
```

The preserved verifier hashes each file using Git blob framing, checks symlinks
and executable modes, and asserts that all extra files are local build outputs.
Pre-build verification used the same blob/mode comparison before output existed.
Inspection commands also checked policy, build recipes, tools, process names,
account-path existence and Clean-CTX configuration resolution. No provider
executable was run, including for `--version`.

## Build environment

- Debian GNU/Linux 13.6 (trixie), Linux kernel `6.18.44`, x86-64.
- Account: `agent`, UID 1000.
- GCC and G++: Debian `14.2.0-19`, version `14.2.0`.
- GNU Make `4.4.1`; GNU ld/binutils `2.44`; glibc `2.41-12+deb13u3`.
- Git `2.52.0`; provenance helper Python `3.12.14`.
- Build profile: standard production C/C++ `-O2`, dynamically linked, UI-free,
  not stripped. No environment overrides for test seams, static mode, flags,
  `MAKEFLAGS`, `LD_PRELOAD` or provider cache were present.

## Artifact provenance and runtime composition

| Role | Absolute path | Size | SHA-256 |
| --- | --- | ---: | --- |
| Native CBM frontend, daemon and worker implementation | `/workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp` | 301,278,992 bytes | `6dfaa906ed310795a97ca9b992d4d42b6118a8cf740ea7b4550df6d8146b24f0` |

Artifact is an ELF 64-bit x86-64 PIE, GNU/Linux, build ID
`c04e297981a339ad386d9db52dd526e27b6d61d7`. It is a single native implementation
executable, not a shim, launcher or script. Vendored parsers, store components
and pretrained vector blob were compiled/linked into it. No adjacent CBM
runtime files or generated object files are required after linking.

The OS dynamic loader `/lib64/ld-linux-x86-64.so.2` and system libraries
`libm.so.6`, `libstdc++.so.6`, `libz.so.1`, `libgcc_s.so.1`, `libc.so.6` remain
required. Their resolved paths, sizes and SHA-256 values are recorded in the
machine-readable manifest. These are system runtime dependencies, not a second
dynamically loaded CBM implementation. Preserve this cloud runtime and binary
for acceptance; rehash the binary before a later run.

Package metadata is separate: pinned `server.json` declares `0.11.0`; the
supported build option stamped `CBM_VERSION` as `0.11.0`. This is recorded
compile-time metadata, **not** an observed runtime version response. Exact
revision identity comes from the source comparison and witnessed build, not
from that version string.

Local retained evidence:

- `/workspace/cbm-gate0-evidence/build.log`: complete compiler/link log.
- `/workspace/cbm-gate0-evidence/artifact-provenance.json`: artifact, toolchain,
  system runtime dependency hashes and build-log hash.
- `/workspace/cbm-gate0-evidence/source-verification.json`: source identity and
  generated-output inventory.
- `/workspace/cbm-gate0-evidence/verify-source.py`: reproducible source verifier.

These evidence files and the executable are retained in this current cloud
environment. They are not a published artifact or a guarantee of retention in
a replacement environment.

## Side effects

- No CBM daemon/frontend was launched by this work. Process-name observations
  before and after the build found no CBM process; the selected recipe performs
  compilation/linking and does not run the product.
- No installation, activation, update, global deployment replacement or account
  configuration mutation command was executed.
- No provider cache override was supplied. The normal account provider cache
  directory remained absent at observed checkpoints. No canonical cache root
  was resolved by a running provider; actual cache/cohort admission is untested.
- No existing provider executable was found on PATH or in the checked common
  account/global paths; no installation path was written by the build.
- Tracked provider source content/modes are unchanged. Generated objects,
  embedded vector object, configuration stamp and executable are confined to
  `build/a5-gate0/`; local evidence is outside the source directory.
- No Clean-CTX production changes, Cargo commands, test suite or MCP runs.
  This report is the only repository change for Gate 0.

These conclusions combine the inspected selected build path with observations;
they do not claim a system-wide audit of unrelated concurrent activity.

## A5 invocation path — prepared, not executed

Clean-CTX already accepts an explicit `cbm.binary_path` through `.clean-ctx.json`.
`src/cbm/bridge/binary.rs` gives a valid explicit file priority over PATH and
installation discovery. A later operator-owned acceptance workspace can use:

```json
{
  "cbm": {
    "binary_path": "/workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp"
  }
}
```

Merge that field into the acceptance workspace's existing configuration,
preserving other settings. No configuration was created or changed now. The
later separately authorized Clean-CTX launch will resolve this absolute file
and use normal A2/A3 launch/admission. No global installation or private cache
cohort is necessary to select this executable.

## Scope and next gate

This artifact establishes cloud/Linux provenance only. Windows live
deployment/lifecycle remains unverified.

Gate 0 establishes artifact identity. Initialization, daemon bootstrap/reuse,
account/cohort/cache compatibility, live registry, session isolation, recovery,
watcher freshness, completeness, Phase B and Issue #142 remain unverified by
this build. The prepared A5 live acceptance matrix has not resumed.

**A5 GATE 0 PASSED — LIVE ACCEPTANCE REQUIRES SEPARATE AUTHORIZATION**
