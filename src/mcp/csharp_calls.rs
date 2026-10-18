//! Session-owned lifecycle for exact C# call authority.

pub(crate) mod authority;
pub(crate) mod lifecycle;

#[cfg(all(test, feature = "csharp"))]
#[path = "../tests/mcp/csharp_call_artifact_reconciliation.rs"]
mod csharp_call_artifact_reconciliation;

#[cfg(all(test, feature = "csharp"))]
#[path = "../tests/mcp/csharp_call_authority_fingerprint.rs"]
mod csharp_call_authority_fingerprint;
