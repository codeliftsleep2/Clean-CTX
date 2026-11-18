//! Test-only counter for .NET detection-pass regressions.

thread_local! {
    static DETECTION_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static AST_PARSE_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn record_detection() {
    DETECTION_COUNT.with(|count| count.set(count.get() + 1));
}

#[cfg(feature = "dotnet")]
pub(crate) fn reset_detection_count() {
    DETECTION_COUNT.with(|count| count.set(0));
}

#[cfg(feature = "dotnet")]
pub(crate) fn detection_count() -> usize {
    DETECTION_COUNT.with(std::cell::Cell::get)
}

pub(crate) fn record_ast_parse() {
    AST_PARSE_COUNT.with(|count| count.set(count.get() + 1));
}

#[cfg(all(feature = "dotnet", feature = "angular"))]
pub(crate) fn reset_ast_parse_count() {
    AST_PARSE_COUNT.with(|count| count.set(0));
}

#[cfg(all(feature = "dotnet", feature = "angular"))]
pub(crate) fn ast_parse_count() -> usize {
    AST_PARSE_COUNT.with(std::cell::Cell::get)
}
