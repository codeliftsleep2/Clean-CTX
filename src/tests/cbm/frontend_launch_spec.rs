//! Structural coverage for the new launch-spec boundary, not defect RED claims.
use super::FrontendLaunchSpec;
use crate::cbm::client::CbmError;

#[test]
fn cbm_a2_baseline_resolved_launch_values_are_deterministic() {
    let root = tempfile::TempDir::new().unwrap();
    let executable = std::env::current_exe().unwrap();
    let parent = std::env::current_dir().unwrap();
    let relative = executable.strip_prefix(&parent).unwrap_or(&executable);
    let first = FrontendLaunchSpec::resolve(relative, root.path())
        .unwrap()
        .unwrap();
    let second = FrontendLaunchSpec::resolve(&executable, &root.path().join("."))
        .unwrap()
        .unwrap();
    assert!(first.binary.is_absolute());
    assert!(first.root.is_absolute());
    assert_eq!(first.binary, executable.canonicalize().unwrap());
    assert_eq!(first.binary, second.binary);
    assert_eq!(first.root, second.root);
    assert_eq!(first.root, root.path().canonicalize().unwrap());
}

#[test]
fn cbm_a2_baseline_invalid_launch_root_fails_before_spawn() {
    let root = tempfile::TempDir::new().unwrap();
    let executable = std::env::current_exe().unwrap();
    assert!(matches!(
        FrontendLaunchSpec::resolve(&executable, &root.path().join("missing")),
        Err(CbmError::LaunchError(_))
    ));
    let file = root.path().join("not-a-directory");
    std::fs::write(&file, "fixture").unwrap();
    assert!(matches!(
        FrontendLaunchSpec::resolve(&executable, &file),
        Err(CbmError::LaunchError(_))
    ));
}
