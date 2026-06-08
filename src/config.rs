// src/config.rs — Project-level configuration for Clean-CTX
// Reads .clean-ctx.json from the project root for custom settings

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Project-level configuration loaded from `.clean-ctx.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanCtxConfig {
    /// Custom type aliases: short_name → original_type
    #[serde(default)]
    pub type_aliases: BTreeMap<String, String>,

    /// Fidelity override per file extension
    #[serde(default)]
    pub fidelity_overrides: BTreeMap<String, String>,

    /// File/directory patterns to exclude from compression.
    ///
    /// Supports simple glob syntax: `*` matches any sequence of non-separator
    /// characters, `?` matches exactly one non-separator character. Patterns
    /// are matched against each **path segment** (component), so `"dist"`
    /// matches any directory or file named `dist`, but `"distribute"` does
    /// NOT match `"dist"` (unlike the old substring check). Use `"dist*"`
    /// to match both.
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Custom behavior markers: marker → description
    #[serde(default)]
    pub custom_markers: BTreeMap<String, String>,

    /// Default fidelity level if not specified
    #[serde(default = "default_fidelity")]
    pub default_fidelity: String,

    /// Whether to enable diff-aware compression
    #[serde(default = "default_true")]
    pub diff_compression: bool,

    /// Whether to enable workspace-wide type detection
    #[serde(default = "default_true")]
    pub workspace_type_detection: bool,

    /// Per-framework Meta-Layer configuration (Phase 1: Angular only).
    ///
    /// Each entry configures one framework meta-layer. The key is
    /// the framework name (e.g. `"angular"`); the value is the
    /// per-framework config struct. A missing entry means the
    /// framework meta-layer is on (default behaviour — see the
    /// framework-specific config for the opt-out flag).
    ///
    /// Example `.clean-ctx.json`:
    /// ```json
    /// { "meta_layers": { "angular": { "enabled": false } } }
    /// ```
    #[serde(default)]
    pub meta_layers: BTreeMap<String, MetaLayerConfig>,
}

/// Per-framework Meta-Layer configuration.
///
/// Phase 1 ships only the `angular` variant. Future phases will add
/// `react`, `vue`, `svelte` variants following the same pattern. Each
/// framework gets its own struct (or, when the schema converges
/// further, a shared `MetaLayerConfig` with framework-specific
/// extension fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLayerConfig {
    /// Master switch for this framework's meta-layer. Defaults to
    /// `true` (the meta-layer runs whenever the framework is
    /// detected). Set to `false` in `.clean-ctx.json` to opt out
    /// of the meta-layer entirely (zero overhead).
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for MetaLayerConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

fn default_fidelity() -> String {
    "low".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for CleanCtxConfig {
    fn default() -> Self {
        Self {
            type_aliases: BTreeMap::new(),
            fidelity_overrides: BTreeMap::new(),
            exclude_patterns: Vec::new(),
            custom_markers: BTreeMap::new(),
            default_fidelity: default_fidelity(),
            diff_compression: default_true(),
            workspace_type_detection: default_true(),
            meta_layers: BTreeMap::new(),
        }
    }
}

/// Cached path to the `.clean-ctx.json` config file, looked up once per
/// process lifetime. Edits to the config require a server restart to take
/// effect (the config is treated as immutable for the session).
static CONFIG_PATH: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();

impl CleanCtxConfig {
    /// Load configuration from the project directory, walking up to find `.clean-ctx.json`
    pub fn load(start_dir: &Path) -> Self {
        if let Some(config_path) = Self::find_config(start_dir) {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => {
                            eprintln!("[clean-ctx] Loaded config from: {}", config_path.display());
                            config
                        }
                        Err(e) => {
                            eprintln!("[clean-ctx] Warning: Failed to parse {}: {}", config_path.display(), e);
                            Self::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[clean-ctx] Warning: Failed to read {}: {}", config_path.display(), e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    /// Walk up from start_dir looking for `.clean-ctx.json`.
    /// Result is cached in a process-global `OnceLock` so subsequent calls
    /// do not touch the filesystem.
    pub(crate) fn find_config(start_dir: &Path) -> Option<PathBuf> {
        CONFIG_PATH
            .get_or_init(|| Self::find_config_uncached(start_dir))
            .clone()
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
        let path_obj = Path::new(path);

        // Extract the file name for the substring-glob tier.
        let file_name = path_obj
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();

        for pattern in &self.exclude_patterns {
            // Tier 1: exact-segment glob match.
            for component in path_obj.components() {
                let segment = component.as_os_str().to_string_lossy();
                if glob_match(pattern, &segment) {
                    return true;
                }
            }

            // Tier 2: if the pattern contains a dot, it is likely a
            // filename-oriented pattern (e.g. ".test.", "*.spec.ts").
            // If it also contains glob chars, do glob matching against
            // the full file name; otherwise, do substring matching so
            // that ".test." matches "file.test.ts".
            if pattern.contains('.') {
                if (pattern.contains('*') || pattern.contains('?'))
                    && glob_match(pattern, &file_name)
                {
                    return true;
                }
                if !pattern.contains('*') && !pattern.contains('?')
                    && file_name.contains(pattern.as_str())
                {
                    return true;
                }
            }
        }
        false
    }

    /// Get fidelity override for a file extension
    pub fn get_fidelity_for_extension(&self, ext: &str) -> Option<&str> {
        self.fidelity_overrides.get(ext).map(|s| s.as_str())
    }

    /// Generate a default config file content
    pub fn default_config_content() -> String {
        let default = Self::default();
        serde_json::to_string_pretty(&default).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Minimal glob matcher supporting `*` (matches any non-separator characters)
/// and `?` (matches exactly one non-separator character). All other characters
/// are matched literally. This is intentionally simple — full `globset` support
/// can be added later if needed.
fn glob_match(pattern: &str, text: &str) -> bool {
    glob_match_impl(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_impl(pattern: &[u8], text: &[u8]) -> bool {
    let mut pi = 0;
    let mut ti = 0;
    let mut star_pi = None; // pattern position after last '*'
    let mut star_ti = 0;    // text position when '*' was matched

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

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
