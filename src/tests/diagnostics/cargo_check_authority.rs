use crate::diagnostics::cargo_check::{
    ApprovedCargoExecutable, ApprovedWorkspaceRoot, AuthorityError, AuthoritySource,
    PathClassification,
};
use std::path::{Path, PathBuf};

fn create_workspace() -> tempfile::TempDir {
    let temporary = tempfile::tempdir().expect("temporary workspace");
    std::fs::write(
        temporary.path().join("Cargo.toml"),
        "[package]\nname='fixture'\n",
    )
    .expect("manifest fixture");
    temporary
}

fn create_executable(directory: &Path) -> PathBuf {
    #[cfg(windows)]
    let path = directory.join("cargo.exe");
    #[cfg(not(windows))]
    let path = directory.join("cargo");
    std::fs::write(&path, b"synthetic executable identity").expect("executable fixture");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = path.metadata().expect("metadata").permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&path, permissions).expect("executable permission");
    }
    path
}

#[test]
fn workspace_admission_requires_absolute_manifest_root() {
    assert_eq!(
        ApprovedWorkspaceRoot::admit(Path::new("relative"), AuthoritySource::CliArgument),
        Err(AuthorityError::RelativePath)
    );

    let temporary = tempfile::tempdir().expect("temporary directory");
    assert_eq!(
        ApprovedWorkspaceRoot::admit(temporary.path(), AuthoritySource::StartupOption),
        Err(AuthorityError::MissingManifest)
    );
}

#[test]
fn admitted_workspace_is_canonical_and_source_is_retained() {
    let workspace = create_workspace();
    let approved = ApprovedWorkspaceRoot::admit(workspace.path(), AuthoritySource::Environment)
        .expect("workspace admitted");

    assert!(approved.canonical_root().is_absolute());
    assert_eq!(approved.source(), AuthoritySource::Environment);
    assert_eq!(approved.revalidate(), Ok(()));
}

#[test]
fn changed_manifest_invalidates_workspace_authority() {
    let workspace = create_workspace();
    let approved = ApprovedWorkspaceRoot::admit(workspace.path(), AuthoritySource::StartupOption)
        .expect("workspace admitted");
    std::fs::write(
        workspace.path().join("Cargo.toml"),
        "[package]\nname='changed-fixture-with-different-length'\n",
    )
    .expect("manifest changed");

    assert_eq!(approved.revalidate(), Err(AuthorityError::IdentityChanged));
}

#[test]
fn cargo_admission_requires_absolute_regular_platform_executable() {
    assert_eq!(
        ApprovedCargoExecutable::admit(Path::new("cargo"), AuthoritySource::Environment),
        Err(AuthorityError::RelativePath)
    );
    let temporary = tempfile::tempdir().expect("temporary directory");
    assert_eq!(
        ApprovedCargoExecutable::admit(temporary.path(), AuthoritySource::StartupOption),
        Err(AuthorityError::CargoNotFile)
    );
}

#[test]
fn admitted_cargo_is_canonical_and_revalidated_by_identity() {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let executable = create_executable(temporary.path());
    let approved = ApprovedCargoExecutable::admit(&executable, AuthoritySource::StartupOption)
        .expect("Cargo admitted");

    assert!(approved.canonical_path().is_absolute());
    assert_eq!(approved.source(), AuthoritySource::StartupOption);
    assert_eq!(approved.revalidate(), Ok(()));

    std::fs::write(&executable, b"changed executable identity and length")
        .expect("executable changed");
    assert_eq!(approved.revalidate(), Err(AuthorityError::IdentityChanged));
}

#[test]
fn display_paths_are_relative_inside_and_redacted_outside() {
    let workspace = create_workspace();
    let approved = ApprovedWorkspaceRoot::admit(workspace.path(), AuthoritySource::CliArgument)
        .expect("workspace admitted");

    let inside = approved.display_path(&workspace.path().join("src/lib.rs"));
    assert_eq!(inside.value, "src/lib.rs");
    assert_eq!(inside.classification, PathClassification::InsideWorkspace);

    let canonical_spelling = approved.display_path(&approved.canonical_root().join("src/main.rs"));
    assert_eq!(canonical_spelling.value, "src/main.rs");
    assert_eq!(
        canonical_spelling.classification,
        PathClassification::InsideWorkspace
    );

    let outside = approved.display_path(&workspace.path().join("../secret/location.rs"));
    assert_eq!(outside.value, "<external>/location.rs");
    assert_eq!(outside.classification, PathClassification::External);
    assert!(
        !outside
            .value
            .contains(&workspace.path().to_string_lossy().to_string())
    );
}
