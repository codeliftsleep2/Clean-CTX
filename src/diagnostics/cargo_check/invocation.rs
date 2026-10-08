use super::{
    ApprovedCargoExecutable, ApprovedWorkspaceRoot, AuthorityError, CargoCheckEnvironment,
    EnvironmentFacts,
};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

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
}
