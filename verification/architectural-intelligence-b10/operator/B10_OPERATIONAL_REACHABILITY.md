# Operational reachability checkpoint

Agent: **Agent MaxHeadRoom**. Observation date: 2026-11-12.

`src/mcp/pcg_state.rs` enables the document session only when
`CLEAN_CTX_PCG_DOCUMENT_PROFILE` is present. `src/mcp/tools.rs` advertises its
tools only when that session exists. `src/mcp/tool_dispatch.rs` selects the
corresponding conditional registry. A newly initialized server/session is needed
to observe a corrected launch; editing a file does not establish an existing
host refreshed its advertised tools.

The inspected project `.codex/config.toml` enables clean_ctx but supplies only
project-root/startup-trace environment settings, without the document profile.
It launches `target/release/clean-ctx.exe`, SHA-256
`b4002d070dc87eb0910aed10c8d0a475280e44c2dd65fa1c27dabd777c8ba1fa`.
The accepted B9 executable is the separately verified debug executable. This does
not establish which features the release executable implements. The omitted
profile already explains the conditional registration failure under the inspected
production path. No tool allow/deny filter was found in that project registration.

Current tool enumeration lacks pcg_knowledge_v1 and pcg_documents_v1. Earlier live
process inspection found release-path processes; mapped-image bytes and launch
environment were not accessible, so current on-disk hashes are not asserted to be
the exact historical loaded images.

Configuration receipt: `target/pcg-b10/configuration-receipt.json`. Project config
SHA-256: `4fba1020c110d8ca09e244dc181422b25f53ce070c10a85089e2623bdec971b9`.
Normal user/project configuration is not changed. No credentials are copied.

The prepared `../infrastructure/invoke-host-smoke.ps1` uses supported Codex CLI
configuration overrides, a new ephemeral session and existing authentication.
The agent shell is read-only; the explicitly authorized MCP server may reconcile
its derived database. It uses the existing authoring launch directory to retain
the existing disabled background integrations. It is not a new host implementation.
Actual CLI compatibility, startup, advertisement and invocations remain UNRUN.
An unsupported option or approval failure is an operational blocker to preserve,
not permission to bypass host controls.

Supported stdio launch/environment and configuration overrides are documented in
[Codex MCP](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) and the
[configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference).
Installed CLI help was inspected earlier in B10; it does not prove this wrapper ran.

Successful host smoke would establish reachability in that specific isolated CLI
session. It would not establish deployment in the current interactive host,
natural workflow adoption, uncontaminated evaluation or improved decisions.
