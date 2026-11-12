# Network and data-handling boundaries

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

## Security Posture

The core `clean-ctx` MCP transport is local stdio and can run without network
access. The complete workspace is not categorically network-free: CBM can be
launched as a subprocess, and the separately built `clean-ctx-proxy` is an HTTP
service that forwards requests to a configured upstream. Air-gapped deployment
therefore requires disabling those optional integrations and provisioning Cargo
dependencies offline.

---


Source: [docs/SECURITY.md](../../SECURITY.md); original section SHA-256 `8d08e2b38eb44f1c37de0092147e45f2741c868c9d381b615373780f0cf3208b`. Original wording and tense are retained; admission is limited to the implemented scope described here.

### Network

| Requirement | Status | Detail |
|-------------|--------|--------|
| Core MCP transport | Local | `clean-ctx` serves JSON-RPC over stdin/stdout. |
| CBM integration | Optional subprocess | May launch and exchange JSON-RPC with a configured local CBM process. |
| HTTP proxy | Optional network service | `clean-ctx-proxy` listens locally and forwards to its configured upstream. |
| Air-gapped mode | Supported by configuration | Do not enable/use CBM or the proxy; vendor dependencies for offline builds. |


Source: [docs/SECURITY.md](../../SECURITY.md); original section SHA-256 `30bd0cf8f0b30124264242fa58877bb9967373c2ef4646170ad91f100070eb22`. Original wording and tense are retained; admission is limited to the implemented scope described here.

### Data Handling

| Requirement | Status | Detail |
|-------------|--------|--------|
| Data at rest | Configurable | Persistence defaults to SQLite at `.clean-ctx/persistence.db`; CBM and proxy caches/logs may also write under configured locations. |
| Source mutation | Explicit | `apply_edit` is an authorized write surface with exact-source and transactional guards. |
| Data in transit | Boundary-dependent | MCP is local stdio; the optional proxy forwards prompts/tool data to its configured upstream. |
| Secret scrubbing | Proxy feature | Scrubbing reduces accidental disclosure but is not a substitute for upstream trust or least privilege. |


Source: [docs/SECURITY.md](../../SECURITY.md); original section SHA-256 `7b3b2eddb7976f6ca4c5659c07d9307089df00251e9929971024bdd9ddcc1b9a`. Original wording and tense are retained; admission is limited to the implemented scope described here.


This contract describes effects of each boundary. It does not certify aggregate air-gap, universal secret coverage, or DLP compliance. Source-mutation authority is owned by [its focused contract](source-mutation.md).

```architecture-graph
node: AUTH.NETWORK_EFFECTS
kind: AUTHORITY
status: IMPLEMENTED
title: Network and data-handling boundaries
track: REPOSITORY
artifact: docs/architecture/contracts/network-effects.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```
