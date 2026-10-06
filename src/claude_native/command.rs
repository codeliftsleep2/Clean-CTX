use crate::native_text::cargo_filter::CargoOperation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DiagnosticTarget {
    GitDiffStdout,
    CargoStderr(CargoOperation),
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
        _ => None,
    }
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
