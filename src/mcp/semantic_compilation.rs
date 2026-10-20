pub(crate) mod artifact;
pub(crate) mod compile;
#[cfg(feature = "csharp")]
pub(crate) mod dependencies;
pub(crate) mod executor;

#[cfg(test)]
#[path = "../tests/mcp/semantic_compilation_artifact.rs"]
mod tests;
