use super::*;
use crate::native_text::{
    angular_filter::AngularOperation, cargo_filter::CargoOperation, dotnet_filter::DotnetOperation,
    maven_filter::MavenOperation, node_build_filter::NodeBuildOperation,
};

#[test]
fn selects_only_approved_operation_and_field_pairs() {
    assert_eq!(
        diagnostic_target("  cargo build --workspace  "),
        Some(DiagnosticTarget::CargoStderr(CargoOperation::Build))
    );
    assert_eq!(
        diagnostic_target("cargo check --all-targets"),
        Some(DiagnosticTarget::CargoStderr(CargoOperation::Check))
    );
    assert_eq!(
        diagnostic_target("cargo clippy -- -D warnings"),
        Some(DiagnosticTarget::CargoStderr(CargoOperation::Clippy))
    );
    assert_eq!(
        diagnostic_target("git show --format='value with spaces' HEAD"),
        Some(DiagnosticTarget::GitDiffStdout)
    );
}

#[test]
fn cargo_test_run_and_unknown_operations_have_no_authority() {
    for command in [
        "cargo test",
        "cargo run",
        "cargo metadata",
        "cargo",
        "./cargo build",
        "command cargo build",
        "cargo +nightly check",
        "'cargo' build",
        "FOO=bar cargo build",
    ] {
        assert_eq!(diagnostic_target(command), None, "admitted {command}");
    }
}

#[test]
fn shell_control_constructs_are_conservatively_rejected() {
    for command in [
        "cargo check && echo done",
        "cargo build; echo done",
        "cargo clippy || true",
        "cargo check | tail -20",
        "cargo check &",
        "cargo check > result.txt",
        "cargo check 2> errors.txt",
        "cargo check 2>&1",
        "cargo check < args.txt",
        "cargo check $(printf arg)",
        "cargo check `printf arg`",
        "(cargo check)",
        "{ cargo check; }",
        "cargo check\necho done",
    ] {
        assert_eq!(diagnostic_target(command), None, "admitted {command}");
    }
}

#[test]
fn quoted_metacharacters_in_arguments_do_not_create_shell_control() {
    assert_eq!(
        diagnostic_target("cargo check --message-format 'value;still-an-argument'"),
        Some(DiagnosticTarget::CargoStderr(CargoOperation::Check))
    );
    assert_eq!(
        diagnostic_target("git diff -- 'path with spaces'"),
        Some(DiagnosticTarget::GitDiffStdout)
    );
}

#[test]
fn unbalanced_quotes_and_substitution_inside_double_quotes_are_rejected() {
    assert_eq!(diagnostic_target("cargo check 'unterminated"), None);
    assert_eq!(diagnostic_target("cargo check \"$(printf arg)\""), None);
    assert_eq!(diagnostic_target("cargo check \"`printf arg`\""), None);
}

#[test]
fn selects_supported_typescript_and_angular_operations() {
    for command in ["tsc --noEmit", "npx tsc -p tsconfig.json", "bunx tsc"] {
        assert_eq!(
            diagnostic_target(command),
            Some(DiagnosticTarget::TscStdout)
        );
    }
    for (command, operation) in [
        (
            "ng build --configuration production",
            AngularOperation::Build,
        ),
        ("ng test --watch=false", AngularOperation::Test),
        ("ng lint", AngularOperation::Lint),
    ] {
        assert_eq!(
            diagnostic_target(command),
            Some(DiagnosticTarget::AngularStdout(operation))
        );
    }
}

#[test]
fn selects_supported_eslint_and_bounded_node_script_forms() {
    for command in [
        "eslint src",
        "npx eslint src",
        "bunx eslint src",
        "npm lint",
        "npm run eslint",
        "pnpm run lint",
        "yarn eslint",
    ] {
        assert_eq!(
            diagnostic_target(command),
            Some(DiagnosticTarget::EslintStdout)
        );
    }
    for (command, operation) in [
        ("npm run build", NodeBuildOperation::Build),
        ("pnpm compile", NodeBuildOperation::Compile),
        ("yarn run bundle", NodeBuildOperation::Bundle),
        ("bun build", NodeBuildOperation::Build),
    ] {
        assert_eq!(
            diagnostic_target(command),
            Some(DiagnosticTarget::NodeBuildStdout(operation))
        );
    }
}

#[test]
fn rejects_unapproved_wrappers_scripts_and_watch_modes() {
    for command in [
        "pnpm tsc",
        "yarn tsc",
        "./node_modules/.bin/tsc",
        "npx ng build",
        "bun lint",
        "npm run arbitrary",
        "npm install",
        "npm run dev",
        "tsc --watch",
        "tsc --version",
        "tsc --showConfig",
        "npx tsc --listFilesOnly",
        "eslint --print-config src/app.ts",
        "npm run lint -- --help",
        "ng build --watch=true",
        "ng build --help",
        "ng test -w",
        "npm run build -- --help",
        "npm run build && echo done",
    ] {
        assert_eq!(diagnostic_target(command), None, "admitted {command}");
    }
}

#[test]
fn typescript_watch_policy_does_not_leak_into_existing_producers() {
    assert_eq!(
        diagnostic_target("git diff -- -w"),
        Some(DiagnosticTarget::GitDiffStdout)
    );
    assert_eq!(
        diagnostic_target("cargo check --config profile.dev.opt-level=\"w\""),
        Some(DiagnosticTarget::CargoStderr(CargoOperation::Check))
    );
}

#[test]
fn selects_only_diagnostic_dotnet_operations() {
    assert_eq!(
        diagnostic_target("dotnet build CleanCtx.sln --no-restore"),
        Some(DiagnosticTarget::DotnetStdout(DotnetOperation::Build))
    );
    assert_eq!(
        diagnostic_target("dotnet test tests/CleanCtx.Tests --no-build"),
        Some(DiagnosticTarget::DotnetStdout(DotnetOperation::Test))
    );
    for command in [
        "dotnet run",
        "dotnet watch test",
        "dotnet build --help",
        "dotnet test --list-tests",
        "dotnet test -t",
        "dotnet build --getProperty:TargetFramework",
        "dotnet build-server shutdown",
        "./dotnet build",
        "dotnet build && echo done",
    ] {
        assert_eq!(diagnostic_target(command), None, "admitted {command}");
    }
}

#[test]
fn selects_only_approved_maven_lifecycle_operations() {
    for (command, operation) in [
        ("mvn compile", MavenOperation::Compile),
        ("mvn package -DskipTests", MavenOperation::Package),
        ("mvn install -DskipTests", MavenOperation::Install),
        ("mvn clean package -DskipTests", MavenOperation::Package),
        ("mvn clean compile package", MavenOperation::Package),
    ] {
        assert_eq!(
            diagnostic_target(command),
            Some(DiagnosticTarget::MavenStdout(operation))
        );
    }
    for command in [
        "mvn clean",
        "mvn test",
        "mvn verify",
        "mvn --version",
        "mvn package --help",
        "mvn clean --show-version package",
        "mvn help:effective-pom",
        "mvn package exec:java",
        "mvn package dependency:tree",
        "mvn -q package",
        "./mvnw package",
        "mvn package && echo done",
    ] {
        assert_eq!(diagnostic_target(command), None, "admitted {command}");
    }
}
