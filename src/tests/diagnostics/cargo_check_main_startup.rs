use super::*;

#[test]
fn cargo_check_startup_no_arguments_retains_mcp_default() {
    let parsed = StartupCli::try_parse_from(["clean-ctx"]).unwrap();
    assert!(parsed.command.is_none());
    assert!(parsed.authority.workspace_root.is_none());
    assert!(parsed.authority.cargo_path.is_none());
}

#[test]
fn cargo_check_startup_root_options_select_mcp_without_a_subcommand() {
    let parsed = StartupCli::try_parse_from([
        "clean-ctx",
        "--workspace-root",
        "/operator/workspace",
        "--cargo-path",
        "/operator/cargo",
    ])
    .unwrap();
    assert!(parsed.command.is_none());
    assert_eq!(
        parsed.authority.workspace_root.unwrap(),
        std::path::Path::new("/operator/workspace")
    );
    assert_eq!(
        parsed.authority.cargo_path.unwrap(),
        std::path::Path::new("/operator/cargo")
    );
}

#[test]
fn cargo_check_startup_existing_commands_keep_their_dispatch() {
    for arguments in [
        vec!["clean-ctx", "init"],
        vec!["clean-ctx", "setup", "--force"],
        vec!["clean-ctx", "proxy", "--stop"],
        vec!["clean-ctx", "--config-dump"],
        vec!["clean-ctx", "claude-hook", "post-tool-use"],
    ] {
        assert!(
            StartupCli::try_parse_from(arguments)
                .unwrap()
                .command
                .is_some()
        );
    }
    let parsed = StartupCli::try_parse_from([
        "clean-ctx",
        "cargo-check",
        "--workspace-root",
        "/approved/workspace",
        "--cargo-path",
        "/approved/cargo",
        "--json",
    ])
    .unwrap();
    assert!(matches!(parsed.command, Some(Cli::CargoCheck { options }) if options.json));
}

#[test]
fn cargo_check_startup_flags_do_not_silently_grant_cli_authority() {
    assert!(
        StartupCli::try_parse_from([
            "clean-ctx",
            "--workspace-root",
            "/operator/workspace",
            "init",
        ])
        .is_err()
    );
}

#[test]
fn cargo_check_startup_help_and_version_never_start_mcp() {
    for flag in ["--help", "-h", "--version", "-V"] {
        let error = StartupCli::try_parse_from(["clean-ctx", flag]).unwrap_err();
        assert!(matches!(
            error.kind(),
            clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
        ));
    }
    let help = StartupCli::try_parse_from(["clean-ctx", "--help"])
        .unwrap_err()
        .to_string();
    assert!(help.contains("--workspace-root") && help.contains("--cargo-path"));
}
