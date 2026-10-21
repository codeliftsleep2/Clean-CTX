pub(crate) mod artifact;
pub(crate) mod compile;
#[cfg(feature = "csharp")]
pub(crate) mod dependencies;
pub(crate) mod executor;

#[cfg(test)]
#[path = "../tests/mcp/semantic_compilation_artifact.rs"]
mod tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/mcp/semantic_compilation_scope_issue_162.rs"]
mod scope_tests;
