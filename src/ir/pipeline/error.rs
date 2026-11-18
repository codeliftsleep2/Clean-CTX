/// Error type for pass execution.
#[derive(Debug, Clone)]
pub struct PassError {
    /// Name of the pass that failed
    pub pass_name: String,
    /// Error message
    pub message: String,
}

impl std::fmt::Display for PassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pass '{}' failed: {}", self.pass_name, self.message)
    }
}

impl std::error::Error for PassError {}
