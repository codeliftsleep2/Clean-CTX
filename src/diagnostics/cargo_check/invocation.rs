use super::{
    ApprovedCargoExecutable, ApprovedWorkspaceRoot, AuthorityError, CargoCheckEnvironment,
    EnvironmentFacts,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InvocationFacts {
    pub workspace: &'static str,
    pub workspace_source: super::AuthoritySource,
    pub manifest_identity: super::FileIdentity,
    pub cargo: String,
    pub cargo_source: super::AuthoritySource,
    pub cargo_identity: super::FileIdentity,
    pub command: &'static str,
    pub environment: EnvironmentFacts,
    pub transformations: super::TransformationFacts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoCheckRequest {
    workspace: ApprovedWorkspaceRoot,
}

impl CargoCheckRequest {
    pub fn new(workspace: ApprovedWorkspaceRoot) -> Self {
        Self { workspace }
    }

    pub fn workspace(&self) -> &ApprovedWorkspaceRoot {
        &self.workspace
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoCheckInvocation {
    workspace_authority: ApprovedWorkspaceRoot,
    cargo_authority: ApprovedCargoExecutable,
    executable: PathBuf,
    arguments: [OsString; 2],
    current_directory: PathBuf,
    environment: BTreeMap<OsString, OsString>,
    environment_facts: EnvironmentFacts,
}

impl CargoCheckInvocation {
    pub fn prepare<I, K, V>(
        request: &CargoCheckRequest,
        cargo: &ApprovedCargoExecutable,
        environment_snapshot: I,
    ) -> Result<Self, AuthorityError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<OsString>,
    {
        request.workspace.revalidate()?;
        cargo.revalidate()?;
        let environment = CargoCheckEnvironment::from_snapshot(environment_snapshot);
        Ok(Self {
            workspace_authority: request.workspace.clone(),
            cargo_authority: cargo.clone(),
            executable: cargo.canonical_path().to_path_buf(),
            arguments: [
                OsString::from("check"),
                OsString::from("--message-format=json"),
            ],
            current_directory: request.workspace.canonical_root().to_path_buf(),
            environment: environment.entries().clone(),
            environment_facts: environment.facts().clone(),
        })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Recheck the admitted identities at the final process-creation boundary.
    pub(crate) fn revalidate(&self) -> Result<(), AuthorityError> {
        self.workspace_authority.revalidate()?;
        self.cargo_authority.revalidate()
    }

    pub fn arguments(&self) -> &[OsString; 2] {
        &self.arguments
    }

    pub fn current_directory(&self) -> &Path {
        &self.current_directory
    }

    pub fn environment(&self) -> &BTreeMap<OsString, OsString> {
        &self.environment
    }

    pub fn environment_facts(&self) -> &EnvironmentFacts {
        &self.environment_facts
    }

    pub(crate) fn result_facts(&self) -> InvocationFacts {
        let mut transformations = super::model::TransformationFacts::default();
        let mut environment = self.environment_facts.clone();
        for name in environment
            .inherited_names
            .iter_mut()
            .chain(&mut environment.stripped_names)
            .chain(&mut environment.overridden_names)
        {
            *name = super::sanitize::sanitize(name, &mut transformations);
        }
        environment.named_rustup_toolchain = environment
            .named_rustup_toolchain
            .map(|name| super::sanitize::sanitize(&name, &mut transformations));
        InvocationFacts {
            workspace: "<workspace>",
            workspace_source: self.workspace_authority.source(),
            manifest_identity: self.workspace_authority.manifest_identity().clone(),
            cargo: super::sanitize::sanitize(
                &self
                    .executable
                    .file_name()
                    .map(|name| name.to_string_lossy())
                    .unwrap_or_default(),
                &mut transformations,
            ),
            cargo_source: self.cargo_authority.source(),
            cargo_identity: self.cargo_authority.identity().clone(),
            command: "cargo check --message-format=json",
            environment,
            transformations,
        }
    }
}
