use super::*;

/// Cached path to the `.clean-ctx.json` config file, looked up once per
/// process lifetime. Edits to the config require a server restart to take
/// effect (the config is treated as immutable for the session).
static CONFIG_PATH: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();

impl ResourceLimits {
    /// Validate a file size against the configured limit.
    /// Returns `Ok(())` if the file is within limits, or an error message
    /// if it exceeds the maximum allowed size.
    pub fn check_file_size(&self, size: u64) -> Result<(), String> {
        if size > self.max_file_size_bytes as u64 {
            Err(format!(
                "File size {} bytes exceeds maximum allowed size of {} bytes ({} MB). \
                 Consider using compress_workspace or the streaming variant for large files.",
                size,
                self.max_file_size_bytes,
                self.max_file_size_bytes / (1024 * 1024)
            ))
        } else {
            Ok(())
        }
    }

    /// Validate a workspace file count against the configured limit.
    /// Returns `Ok(())` if the workspace is within limits, or an error
    /// message if it exceeds the maximum allowed file count.
    pub fn check_workspace_file_count(&self, count: usize) -> Result<(), String> {
        if count > self.max_workspace_files {
            Err(format!(
                "Workspace contains {} files, which exceeds the maximum allowed count of {} files. \
                 Please reduce the workspace size or adjust the `resource_limits.max_workspace_files` \
                 setting in `.clean-ctx.json`.",
                count, self.max_workspace_files
            ))
        } else {
            Ok(())
        }
    }

    /// Validate memory usage against the configured limit.
    /// Returns `Ok(())` if the estimated memory usage is within limits,
    /// or an error message if it exceeds the maximum allowed memory.
    pub fn check_memory_usage(&self, estimated_bytes: usize) -> Result<(), String> {
        if estimated_bytes > self.max_memory_bytes {
            Err(format!(
                "Estimated memory usage {} bytes exceeds maximum allowed memory of {} bytes ({} MB). \
                 Consider processing files in smaller batches or adjusting the \
                 `resource_limits.max_memory_bytes` setting in `.clean-ctx.json`.",
                estimated_bytes,
                self.max_memory_bytes,
                self.max_memory_bytes / (1024 * 1024)
            ))
        } else {
            Ok(())
        }
    }
}

impl CleanCtxConfig {
    /// Load configuration from the project directory, walking up to find `.clean-ctx.json`
    pub fn load(start_dir: &Path) -> Self {
        let mut config = if let Some(config_path) = Self::find_config(start_dir) {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(config) => {
                        eprintln!("[clean-ctx] Loaded config from: {}", config_path.display());
                        config
                    }
                    Err(e) => {
                        eprintln!(
                            "[clean-ctx] Warning: Failed to parse {}: {}",
                            config_path.display(),
                            e
                        );
                        Self::default()
                    }
                },
                Err(e) => {
                    eprintln!(
                        "[clean-ctx] Warning: Failed to read {}: {}",
                        config_path.display(),
                        e
                    );
                    Self::default()
                }
            }
        } else {
            Self::default()
        };

        // A-14: Auto-disable persistence in CI environments to prevent
        // stale persistence.db from leaking between CI builds.
        // Checks common CI environment variables: CI, TF_BUILD, GITHUB_ACTIONS, GITLAB_CI
        if config.persistence.enabled && Self::is_ci_environment() {
            eprintln!(
                "[clean-ctx] CI environment detected — disabling persistence to prevent stale database issues"
            );
            config.persistence.enabled = false;
        }

        config
    }

    /// Walk up from start_dir looking for `.clean-ctx.json`.
    /// Result is cached in a process-global `OnceLock` so subsequent calls
    /// do not touch the filesystem.
    pub fn find_config(start_dir: &Path) -> Option<PathBuf> {
        CONFIG_PATH
            .get_or_init(|| Self::find_config_uncached(start_dir))
            .clone()
    }

    /// Check if running in a CI/CD environment by detecting common CI env vars.
    ///
    /// A-14: Used to auto-disable persistence in CI to prevent stale
    /// `persistence.db` from leaking between builds and causing SQLite
    /// file lock contention in parallel test runs.
    ///
    /// P1-8: Previously checked `CI == "true"` only. Now checks for any
    /// non-empty CI value (`"true"`, `"1"`, `"yes"`, etc.) since different
    /// CI systems set CI to different values. Also checks Bitbucket Pipelines
    /// and Buildkite which were missing from the original list.
    pub fn is_ci_environment() -> bool {
        // CI is the most universal CI variable — set by GitHub Actions,
        // GitLab CI, CircleCI, Travis CI, Bitbucket Pipelines, Buildkite, etc.
        // Some set it to "true", others to "1" or "yes".
        std::env::var("CI").is_ok_and(|v| !v.is_empty() && v != "false")
            || std::env::var("TF_BUILD").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("GITLAB_CI").is_ok()
            || std::env::var("JENKINS_URL").is_ok()
            || std::env::var("CIRCLECI").is_ok()
            || std::env::var("TRAVIS").is_ok()
            // Additional CI systems
            || std::env::var("BITBUCKET_BUILD_NUMBER").is_ok()
            || std::env::var("BUILDKITE").is_ok()
    }

    /// Uncached directory walk — called exactly once via [`find_config`].
    fn find_config_uncached(start_dir: &Path) -> Option<PathBuf> {
        let mut current = start_dir.to_path_buf();
        loop {
            let config_path = current.join(".clean-ctx.json");
            if config_path.exists() {
                return Some(config_path);
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    /// Check if a file path should be excluded.
    ///
    /// F-12 (FAANG audit): previously used `path.contains(pattern)` which
    /// matched substrings — `"dist"` would exclude `"src/distribute/utils.ts"`.
    ///
    /// The new strategy has two tiers:
    /// 1. **Exact-segment match** (for plain patterns like `"dist"`): glob
    ///    matching against each path component. `"dist"` matches a directory
    ///    literally named `dist` but NOT `distribute`.
    /// 2. **Substring-glob match** (for patterns containing `.` like
    ///    `".test."` or `"*.spec.ts"`): glob matching against the full file
    ///    name, allowing the pattern to appear anywhere within it.
    pub fn is_excluded(&self, path: &str) -> bool {
        self.matching_exclude_patterns(path).is_some()
    }

    /// F-FINAL-04: Return the *list* of exclude patterns that matched
    /// the given path (empty if the path is not excluded). The
    /// `is_excluded` shim above is preserved for backward compatibility.
    /// This richer variant is what the workspace manifest emits so the
    /// user can see *why* a file was excluded.
    pub fn matching_exclude_patterns(&self, path: &str) -> Option<Vec<String>> {
        let path_obj = Path::new(path);
        let file_name = path_obj
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();

        let mut matched: Vec<String> = Vec::new();
        for pattern in &self.exclude_patterns {
            // Tier 1: exact-segment glob match.
            let mut tier1_matched = false;
            for component in path_obj.components() {
                let segment = component.as_os_str().to_string_lossy();
                if glob_match(pattern, &segment) {
                    tier1_matched = true;
                    break;
                }
            }

            // Tier 2: filename-oriented pattern.
            let mut tier2_matched = false;
            if pattern.contains('.') {
                if (pattern.contains('*') || pattern.contains('?'))
                    && glob_match(pattern, &file_name)
                {
                    tier2_matched = true;
                }
                if !pattern.contains('*')
                    && !pattern.contains('?')
                    && file_name.contains(pattern.as_str())
                {
                    tier2_matched = true;
                }
            }

            if tier1_matched || tier2_matched {
                matched.push(pattern.clone());
            }
        }
        if matched.is_empty() {
            None
        } else {
            Some(matched)
        }
    }

    /// Get fidelity override for a file extension
    pub fn get_fidelity_for_extension(&self, ext: &str) -> Option<Fidelity> {
        self.fidelity_overrides.get(ext).copied()
    }

    /// Generate a default config file content
    pub fn default_config_content() -> String {
        let default = Self::default();
        serde_json::to_string_pretty(&default).unwrap_or_else(|_| "{}".to_string())
    }
}

/// P1-7: Made `pub(crate)` so heuristics.rs and other modules use this
/// single implementation instead of duplicating the logic.
///
/// Minimal glob matcher supporting `*` (matches any non-separator characters)
/// and `?` (matches exactly one non-separator character). All other characters
/// are matched literally. This is intentionally simple — full `globset` support
/// can be added later if needed.
pub(crate) fn glob_match(pattern: &str, text: &str) -> bool {
    glob_match_impl(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_impl(pattern: &[u8], text: &[u8]) -> bool {
    let mut pi = 0;
    let mut ti = 0;
    let mut star_pi = None; // pattern position after last '*'
    let mut star_ti = 0; // text position when '*' was matched

    while ti < text.len() {
        if pi < pattern.len() && pattern[pi] == b'*' {
            star_pi = Some(pi);
            star_ti = ti;
            pi += 1;
        } else if pi < pattern.len() && (pattern[pi] == text[ti] || pattern[pi] == b'?') {
            pi += 1;
            ti += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }

    while pi < pattern.len() && pattern[pi] == b'*' {
        pi += 1;
    }

    pi == pattern.len()
}
