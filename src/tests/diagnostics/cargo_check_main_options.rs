use super::*;

#[test]
fn cargo_check_cli_requires_workspace_even_when_cargo_path_is_supplied() {
    let error = Cli::try_parse_from([
        "clean-ctx",
        "cargo-check",
        "--cargo-path",
        "/approved/cargo",
    ])
    .unwrap_err();
    assert_eq!(
        error.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn cargo_check_cli_rejects_arbitrary_cargo_arguments() {
    for arguments in [
        vec!["--release"],
        vec!["--", "--all-features"],
        vec!["--command", "cargo build"],
    ] {
        let mut command = vec![
            "clean-ctx",
            "cargo-check",
            "--workspace-root",
            "/approved/workspace",
        ];
        command.extend(arguments);
        assert!(Cli::try_parse_from(command).is_err());
    }
}

#[test]
fn cargo_check_cli_leaves_cargo_path_resolution_to_explicit_startup_authority() {
    let command = Cli::try_parse_from([
        "clean-ctx",
        "cargo-check",
        "--workspace-root",
        "/approved/workspace",
    ])
    .unwrap();
    assert!(
        matches!(command, Cli::CargoCheck { options } if options.cargo_path.is_none() && !options.json)
    );
}

#[test]
fn cargo_check_cli_parses_json_without_changing_execution_authority() {
    let command = Cli::try_parse_from([
        "clean-ctx",
        "cargo-check",
        "--workspace-root",
        "/approved/workspace",
        "--cargo-path",
        "/approved/cargo",
        "--json",
    ])
    .unwrap();
    assert!(
        matches!(command, Cli::CargoCheck { options } if options.json && options.workspace_root == std::path::Path::new("/approved/workspace"))
    );
}
