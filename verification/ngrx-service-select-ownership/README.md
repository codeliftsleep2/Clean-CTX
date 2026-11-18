# Angular Service Select ownership verification

The tracked Rust harnesses
`src/tests/angular_meta/ngrx_service_select_ownership.rs` and
`src/tests/mcp/ngrx_service_select_ownership.rs` exercise real compilation and
the registered MCP boundary. Their independent expectations require the
conjunction of authoritative Angular Service ownership, exact NgRx Store
receiver qualification, and exact Selector declaration authority.

The matrix covers ordinary classes, Components/fake Service names, invalid
Store and Selector authorities, multiple Services and Selectors, neighboring
classes, nested/static/arrow refusals, and source mutation. Existing Service
Dispatch ownership consumes the same syntax-backed callable-to-class Service
owner helper while retaining separate Dispatch semantics.

The separate operator-run L3 harness drives a built binary through registered,
provider-disabled stdio MCP operations:

```powershell
pwsh -NoProfile -File verification/issue-reconciliation/Invoke-NgrxServiceSelect.ps1 `
  -BinaryPath ./target/release/clean-ctx.exe
```

It checks the same authority matrix at high, medium, and low fidelity, exact
multi-owner/multi-target behavior, duplicate-edge semantics, source collisions,
and refresh/removal/restoration/deletion lifecycle. Captures are written under
`target/ngrx-service-select/<timestamp-guid>`. This operator evidence remains
separate from the tracked Rust/CI regression results.
