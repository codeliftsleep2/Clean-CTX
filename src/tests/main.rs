// src/tests/main.rs
//
// Regression tests for the Clean-CTX CLI entry point.
//
// These guard against two production bugs that broke MCP clients
// (Claude Code, VS Code, etc.):
//
//   1. **Missing clap `version` field.** Clap 4 requires `version` to be
//      set on the command builder. Without it, `--version` / `-V` are not
//      registered, so MCP clients probing the binary with `--version`
//      during discovery fall into the subcommand-missing path and hang
//      waiting on stdin instead of printing version info and exiting.
//
//   2. **Wrong error-kind guard for no-args.** Clap 4.6 emits
//      `DisplayHelpOnMissingArgumentOrSubcommand` (NOT `MissingSubcommand`)
//      for a no-arg invocation of a subcommand-style enum. `main()` must
//      intercept the correct kind to default to running the MCP server.

use clap::Parser;
use clap::CommandFactory;

use crate::Cli;

/// The clap command MUST have a version set. Without it, `--version`/`-V`
/// are not registered and MCP clients probing the binary hang.
#[test]
fn cli_command_has_version_set() {
    let cmd = Cli::command();
    let version = cmd.get_version();
    assert!(
        version.is_some(),
        "clap command must have a `version` set — without it, `--version` is not \
         registered and MCP clients probing the binary hang in the stdio server loop"
    );
    assert_eq!(version.unwrap(), env!("CARGO_PKG_VERSION"));
}

/// `--version` must be handled by clap (producing a DisplayVersion error),
/// NOT fall through to the MCP server. This is what MCP clients probe with.
#[test]
fn cli_version_flag_is_handled_by_clap() {
    let err = Cli::try_parse_from(["clean-ctx", "--version"]).unwrap_err();
    assert_eq!(
        err.kind(),
        clap::error::ErrorKind::DisplayVersion,
        "`--version` must be handled by clap (DisplayVersion), not fall through \
         to the MCP server and hang"
    );
}

/// `-V` (short version flag) must also be handled by clap.
#[test]
fn cli_short_version_flag_is_handled_by_clap() {
    let err = Cli::try_parse_from(["clean-ctx", "-V"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
}

/// `--help` must be handled by clap (DisplayHelp), not fall through.
#[test]
fn cli_help_flag_is_handled_by_clap() {
    let err = Cli::try_parse_from(["clean-ctx", "--help"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
}

/// `-h` (short help flag) must also be handled by clap.
#[test]
fn cli_short_help_flag_is_handled_by_clap() {
    let err = Cli::try_parse_from(["clean-ctx", "-h"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
}

/// No-args must produce `DisplayHelpOnMissingArgumentOrSubcommand` — the
/// exact error kind that `main()` intercepts to start the MCP server.
///
/// If clap changes this error kind, `main()`'s guard must be updated too,
/// otherwise no-args will print help and exit 2 instead of starting the
/// stdio JSON-RPC server.
#[test]
fn cli_no_args_produces_display_help_on_missing_subcommand() {
    let err = Cli::try_parse_from(["clean-ctx"]).unwrap_err();
    assert_eq!(
        err.kind(),
        clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand,
        "no-args must produce DisplayHelpOnMissingArgumentOrSubcommand so main() \
         can intercept it and start the MCP server. If clap changes this kind, \
         update the guard in main()"
    );
}

/// The `init` subcommand must still parse correctly.
#[test]
fn cli_init_subcommand_parses() {
    let cli = Cli::try_parse_from(["clean-ctx", "init"]).unwrap();
    assert!(matches!(cli, Cli::Init));
}

/// The `setup` subcommand must still parse correctly (with and without --force).
#[test]
fn cli_setup_subcommand_parses() {
    let cli = Cli::try_parse_from(["clean-ctx", "setup"]).unwrap();
    assert!(matches!(cli, Cli::Setup { force: false }));

    let cli = Cli::try_parse_from(["clean-ctx", "setup", "--force"]).unwrap();
    assert!(matches!(cli, Cli::Setup { force: true }));
}

/// The `--config-dump` subcommand must still parse correctly.
#[test]
fn cli_config_dump_subcommand_parses() {
    let cli = Cli::try_parse_from(["clean-ctx", "--config-dump"]).unwrap();
    assert!(matches!(cli, Cli::ConfigDump));
}