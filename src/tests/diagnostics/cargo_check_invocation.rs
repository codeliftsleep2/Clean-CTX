use crate::diagnostics::cargo_check::{
    ApprovedCargoExecutable, ApprovedWorkspaceRoot, AuthorityError, AuthoritySource,
    CargoCheckEnvironment, CargoCheckInvocation, CargoCheckRequest,
};
use std::ffi::OsString;
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

fn admitted_authority() -> (
    tempfile::TempDir,
    tempfile::TempDir,
    CargoCheckRequest,
    ApprovedCargoExecutable,
) {
    let workspace = create_workspace();
    let producer_directory = tempfile::tempdir().expect("producer directory");
    let producer_path = create_executable(producer_directory.path());
    let approved_workspace =
        ApprovedWorkspaceRoot::admit(workspace.path(), AuthoritySource::StartupOption)
            .expect("workspace admission");
    let approved_cargo =
        ApprovedCargoExecutable::admit(&producer_path, AuthoritySource::Environment)
            .expect("Cargo admission");
    (
        workspace,
        producer_directory,
        CargoCheckRequest::new(approved_workspace),
        approved_cargo,
    )
}

#[test]
fn invocation_is_exact_and_contains_no_caller_argument_surface() {
    let (_workspace, _producer_directory, request, cargo) = admitted_authority();
    let invocation = CargoCheckInvocation::prepare(&request, &cargo, [("PATH", "C:\\toolchain")])
        .expect("invocation prepared");

    assert_eq!(invocation.executable(), cargo.canonical_path());
    assert_eq!(
        invocation.arguments(),
        &[
            OsString::from("check"),
            OsString::from("--message-format=json")
        ]
    );
    assert_eq!(
        invocation.current_directory(),
        request.workspace().canonical_root()
    );
}

#[test]
fn strict_environment_inherits_runtime_and_strips_authority_overrides() {
    let environment = CargoCheckEnvironment::from_snapshot([
        ("PATH", "C:\\tools"),
        ("USERPROFILE", "C:\\Users\\fixture"),
        ("CARGO_HOME", "C:\\cargo-home"),
        ("RUSTUP_HOME", "C:\\rustup-home"),
        ("RUSTUP_TOOLCHAIN", "stable-x86_64-pc-windows-msvc"),
        ("RUSTC", "repository-rustc"),
        ("RUSTFLAGS", "-C target-cpu=native"),
        ("CARGO_ENCODED_RUSTFLAGS", "secret flags"),
        ("CARGO_BUILD_TARGET", "unexpected-target"),
        ("CARGO_TARGET_DIR", "outside-target"),
        ("CI", "true"),
        ("UNRELATED_VALUE", "not inherited"),
    ]);

    assert_eq!(value(&environment, "PATH"), Some("C:\\tools"));
    assert_eq!(
        value(&environment, "RUSTUP_TOOLCHAIN"),
        Some("stable-x86_64-pc-windows-msvc")
    );
    for stripped in [
        "RUSTC",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_TARGET",
        "CARGO_TARGET_DIR",
        "CI",
        "UNRELATED_VALUE",
    ] {
        assert_eq!(value(&environment, stripped), None, "{stripped}");
    }
    assert_eq!(value(&environment, "RUSTUP_AUTO_INSTALL"), Some("0"));
    assert_eq!(value(&environment, "RUSTC_WRAPPER"), Some(""));
    assert_eq!(value(&environment, "RUSTC_WORKSPACE_WRAPPER"), Some(""));
    assert_eq!(value(&environment, "CARGO_TERM_COLOR"), Some("never"));
}

#[test]
fn strict_environment_strips_credentials_and_proxies_without_recording_values() {
    let secret = "CLEAN_CTX_TEST_SECRET_abc123";
    let environment = CargoCheckEnvironment::from_snapshot([
        ("CARGO_REGISTRIES_PRIVATE_TOKEN", secret),
        ("MY_API_KEY", secret),
        ("HTTPS_PROXY", "https://user:password@example.invalid"),
        ("NO_PROXY", "localhost"),
    ]);

    assert_eq!(environment.facts().credential_variables_stripped, 2);
    assert_eq!(environment.facts().proxy_variables_stripped, 2);
    let serialized = serde_json::to_string(environment.facts()).expect("facts serialize");
    assert!(!serialized.contains(secret));
    assert!(!serialized.contains("password"));
}

#[test]
fn rustup_override_must_be_a_named_toolchain_not_a_path() {
    let environment = CargoCheckEnvironment::from_snapshot([(
        "RUSTUP_TOOLCHAIN",
        "C:\\repository\\custom-toolchain",
    )]);
    assert_eq!(value(&environment, "RUSTUP_TOOLCHAIN"), None);
    assert!(environment.facts().rustup_toolchain_rejected_as_path);
    assert_eq!(environment.facts().named_rustup_toolchain, None);
}

#[test]
fn invocation_preparation_revalidates_both_authorities() {
    let (workspace, _producer_directory, request, cargo) = admitted_authority();
    std::fs::write(
        workspace.path().join("Cargo.toml"),
        "[package]\nname='changed-after-admission'\n",
    )
    .expect("manifest changed");

    assert_eq!(
        CargoCheckInvocation::prepare(&request, &cargo, std::iter::empty::<(&str, &str)>()),
        Err(AuthorityError::IdentityChanged)
    );
}

fn value<'a>(environment: &'a CargoCheckEnvironment, name: &str) -> Option<&'a str> {
    environment
        .entries()
        .iter()
        .find(|(candidate, _)| candidate.to_string_lossy().eq_ignore_ascii_case(name))
        .and_then(|(_, value)| value.to_str())
}
