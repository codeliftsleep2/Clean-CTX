//! Operator-only execution authority, captured before server initialization.
use super::*;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, clap::Args)]
pub struct CargoCheckStartupOptions {
    /// Approved absolute workspace for the native MCP CargoCheck tool
    #[arg(long)]
    pub workspace_root: Option<PathBuf>,
    /// Approved absolute Cargo executable for the native MCP CargoCheck tool
    #[arg(long)]
    pub cargo_path: Option<PathBuf>,
}

pub fn prepare_cargo_check_startup(
    options: &CargoCheckStartupOptions,
    snapshot: &[(OsString, OsString)],
) -> Result<CargoCheckInvocation, &'static str> {
    fn selected(
        explicit: &Option<PathBuf>,
        snapshot: &[(OsString, OsString)],
        name: &str,
        missing: &'static str,
    ) -> Result<(PathBuf, AuthoritySource), &'static str> {
        if let Some(path) = explicit {
            return Ok((path.clone(), AuthoritySource::StartupOption));
        }
        snapshot
            .iter()
            .rev()
            .find(|(key, _)| super::environment::authority_name_matches(key, name))
            .map(|(_, value)| (PathBuf::from(value), AuthoritySource::Environment))
            .ok_or(missing)
    }
    let (workspace, source) = selected(
        &options.workspace_root,
        snapshot,
        "CLEAN_CTX_PROJECT_ROOT",
        "workspace_authority_missing",
    )?;
    let workspace = ApprovedWorkspaceRoot::admit(&workspace, source)
        .map_err(|_| "workspace_admission_failed")?;
    let (cargo, source) = selected(
        &options.cargo_path,
        snapshot,
        "CLEAN_CTX_CARGO_PATH",
        "cargo_authority_missing",
    )?;
    let cargo =
        ApprovedCargoExecutable::admit(&cargo, source).map_err(|_| "cargo_admission_failed")?;
    CargoCheckInvocation::prepare(
        &CargoCheckRequest::new(workspace),
        &cargo,
        snapshot.iter().cloned(),
    )
    .map_err(|_| "authority_changed_during_startup")
}
