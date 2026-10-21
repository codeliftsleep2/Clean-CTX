//! Native TypeScript authority: snapshots, project policy and exported declarations.
mod capture;
mod host;
mod package;
mod project;
mod resolver;
mod syntax;

pub(crate) use capture::TypeScriptAuthority;
pub(crate) use host::{DeclarationId, Host, Namespace, Witness};
pub(crate) use resolver::{Resolution, Resolver};

#[cfg(test)]
#[path = "../../tests/ir/typescript_resolution_issue_162.rs"]
mod tests;
