use crate::native_text::{
    angular_filter::AngularOperation, cargo_filter::CargoOperation, dotnet_filter::DotnetOperation,
    maven_filter::MavenOperation, node_build_filter::NodeBuildOperation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DiagnosticTarget {
    GitDiffStdout,
    CargoStderr(CargoOperation),
    TscStdout,
    AngularStdout(AngularOperation),
    EslintStdout,
    NodeBuildStdout(NodeBuildOperation),
    DotnetStdout(DotnetOperation),
    MavenStdout(MavenOperation),
}

pub(super) fn diagnostic_target(command: &str) -> Option<DiagnosticTarget> {
    let command = command.trim();
    if command.is_empty()
        || command
            .as_bytes()
            .first()
            .is_some_and(|byte| matches!(byte, b'\'' | b'"'))
    {
        return None;
    }
    let words = simple_words(command)?;
    match words.as_slice() {
        [executable, operation, ..] if executable == "git" => match operation.as_str() {
            "diff" | "show" => Some(DiagnosticTarget::GitDiffStdout),
            _ => None,
        },
        [executable, operation, ..] if executable == "cargo" => {
            let operation = match operation.as_str() {
                "build" => CargoOperation::Build,
                "check" => CargoOperation::Check,
                "clippy" => CargoOperation::Clippy,
                _ => return None,
            };
            Some(DiagnosticTarget::CargoStderr(operation))
        }
        [executable, ..] if executable == "tsc" => {
            tsc_is_diagnostic(&words).then_some(DiagnosticTarget::TscStdout)
        }
        [runner, executable, ..]
            if matches!(runner.as_str(), "npx" | "bunx") && executable == "tsc" =>
        {
            tsc_is_diagnostic(&words).then_some(DiagnosticTarget::TscStdout)
        }
        [executable, operation, ..] if executable == "ng" => {
            if requests_watch_mode(&words) || requests_help(&words) {
                return None;
            }
            let operation = match operation.as_str() {
                "build" => AngularOperation::Build,
                "test" => AngularOperation::Test,
                "lint" => AngularOperation::Lint,
                _ => return None,
            };
            Some(DiagnosticTarget::AngularStdout(operation))
        }
        [executable, ..] if executable == "eslint" => {
            eslint_is_diagnostic(&words).then_some(DiagnosticTarget::EslintStdout)
        }
        [runner, executable, ..]
            if matches!(runner.as_str(), "npx" | "bunx") && executable == "eslint" =>
        {
            eslint_is_diagnostic(&words).then_some(DiagnosticTarget::EslintStdout)
        }
        [manager, script, ..]
            if is_eslint_package_manager(manager)
                && matches!(script.as_str(), "eslint" | "lint") =>
        {
            eslint_is_diagnostic(&words).then_some(DiagnosticTarget::EslintStdout)
        }
        [manager, run, script, ..]
            if is_eslint_package_manager(manager)
                && run == "run"
                && matches!(script.as_str(), "eslint" | "lint") =>
        {
            eslint_is_diagnostic(&words).then_some(DiagnosticTarget::EslintStdout)
        }
        [manager, run, script, ..] if is_node_package_manager(manager) && run == "run" => {
            node_build_target(script, &words)
        }
        [manager, script, ..] if is_node_package_manager(manager) => {
            node_build_target(script, &words)
        }
        [executable, operation, ..] if executable == "dotnet" => dotnet_target(operation, &words),
        [executable, first_goal, remaining @ ..] if executable == "mvn" => {
            maven_target(first_goal, remaining)
        }
        _ => None,
    }
}

fn maven_target(first_goal: &str, remaining: &[String]) -> Option<DiagnosticTarget> {
    if requests_help(remaining)
        || remaining.iter().any(|argument| {
            matches!(
                argument.as_str(),
                "--version" | "-v" | "-V" | "--show-version"
            )
        })
    {
        return None;
    }
    let mut operation = maven_operation(first_goal);
    if first_goal != "clean" && operation.is_none() {
        return None;
    }
    for argument in remaining {
        if argument.starts_with('-') || argument == "clean" {
            continue;
        }
        operation = Some(maven_operation(argument)?);
    }
    operation.map(DiagnosticTarget::MavenStdout)
}

fn maven_operation(goal: &str) -> Option<MavenOperation> {
    match goal {
        "compile" => Some(MavenOperation::Compile),
        "package" => Some(MavenOperation::Package),
        "install" => Some(MavenOperation::Install),
        _ => None,
    }
}

fn dotnet_target(operation: &str, words: &[String]) -> Option<DiagnosticTarget> {
    if requests_help(words) || requests_dotnet_information(operation, words) {
        return None;
    }
    let operation = match operation {
        "build" => DotnetOperation::Build,
        "test" => DotnetOperation::Test,
        _ => return None,
    };
    Some(DiagnosticTarget::DotnetStdout(operation))
}

fn requests_dotnet_information(operation: &str, words: &[String]) -> bool {
    words.iter().any(|word| {
        (operation == "test" && matches!(word.as_str(), "--list-tests" | "-t"))
            || (operation == "build"
                && [
                    "--getProperty",
                    "-getProperty",
                    "--getItem",
                    "-getItem",
                    "--getTargetResult",
                    "-getTargetResult",
                ]
                .iter()
                .any(|prefix| word.starts_with(prefix)))
    })
}

fn is_eslint_package_manager(word: &str) -> bool {
    matches!(word, "npm" | "pnpm" | "yarn")
}

fn is_node_package_manager(word: &str) -> bool {
    matches!(word, "npm" | "pnpm" | "yarn" | "bun")
}

fn node_build_target(script: &str, words: &[String]) -> Option<DiagnosticTarget> {
    if requests_watch_mode(words) || requests_help(words) {
        return None;
    }
    let operation = match script {
        "build" => NodeBuildOperation::Build,
        "compile" => NodeBuildOperation::Compile,
        "bundle" => NodeBuildOperation::Bundle,
        _ => return None,
    };
    Some(DiagnosticTarget::NodeBuildStdout(operation))
}

fn requests_watch_mode(words: &[String]) -> bool {
    words
        .iter()
        .any(|word| matches!(word.as_str(), "--watch" | "--watch=true" | "-w"))
}

fn requests_help(words: &[String]) -> bool {
    words
        .iter()
        .any(|word| matches!(word.as_str(), "--help" | "-h"))
}

fn tsc_is_diagnostic(words: &[String]) -> bool {
    !requests_watch_mode(words)
        && !words.iter().any(|word| {
            matches!(
                word.as_str(),
                "--version"
                    | "-v"
                    | "--help"
                    | "-h"
                    | "--init"
                    | "--showConfig"
                    | "--listFiles"
                    | "--listFilesOnly"
                    | "--explainFiles"
                    | "--traceResolution"
            )
        })
}

fn eslint_is_diagnostic(words: &[String]) -> bool {
    !words.iter().any(|word| {
        matches!(
            word.as_str(),
            "--help" | "-h" | "--version" | "-v" | "--print-config" | "--env-info"
        )
    })
}

fn simple_words(command: &str) -> Option<Vec<String>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Quote {
        None,
        Single,
        Double,
    }

    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = Quote::None;
    let mut chars = command.chars().peekable();
    while let Some(character) = chars.next() {
        match quote {
            Quote::Single => {
                if character == '\'' {
                    quote = Quote::None;
                } else {
                    word.push(character);
                }
            }
            Quote::Double => match character {
                '"' => quote = Quote::None,
                '`' => return None,
                '$' if chars.peek() == Some(&'(') => return None,
                '\\' => word.push(chars.next()?),
                _ => word.push(character),
            },
            Quote::None => match character {
                '\'' => quote = Quote::Single,
                '"' => quote = Quote::Double,
                '\\' => word.push(chars.next()?),
                '$' if chars.peek() == Some(&'(') => return None,
                ';' | '|' | '&' | '<' | '>' | '`' | '(' | ')' | '{' | '}' | '\n' | '\r' => {
                    return None;
                }
                value if value.is_whitespace() => {
                    if !word.is_empty() {
                        words.push(std::mem::take(&mut word));
                    }
                }
                _ => word.push(character),
            },
        }
    }
    if quote != Quote::None {
        return None;
    }
    if !word.is_empty() {
        words.push(word);
    }
    Some(words)
}

#[cfg(test)]
#[path = "../tests/claude_native/command.rs"]
mod tests;
