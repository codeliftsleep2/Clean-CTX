use super::*;
use clap::Parser;

#[test]
fn cargo_check_cli_regression_closed_operation_accepts_explicit_authorities() {
    assert!(
        Cli::try_parse_from([
            "clean-ctx",
            "cargo-check",
            "--workspace-root",
            "/approved/workspace",
            "--cargo-path",
            "/approved/cargo",
        ])
        .is_ok()
    );
}

#[test]
fn cargo_check_cli_regression_json_projection_is_available() {
    assert!(
        Cli::try_parse_from([
            "clean-ctx",
            "cargo-check",
            "--workspace-root",
            "/approved/workspace",
            "--cargo-path",
            "/approved/cargo",
            "--json",
        ])
        .is_ok()
    );
}

#[test]
fn cargo_check_cli_regression_help_exposes_the_operation_without_starting_mcp() {
    let error = Cli::try_parse_from(["clean-ctx", "cargo-check", "--help"]).unwrap_err();
    assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(error.to_string().contains("--workspace-root"));
}
