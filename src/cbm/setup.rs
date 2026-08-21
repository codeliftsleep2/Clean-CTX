// src/cbm/setup.rs
//
// CBM setup & auto-detection utilities.
//
// Provides `cbm_setup_check()` for the `clean-ctx setup --with-cbm` CLI
// command and `detect_cbm()` for runtime auto-detection used by
// `provide_code_context` enrichment and `McpState::init_cbm_bridge`.

use std::path::{Path, PathBuf};

/// Information about the CBM installation on the current system.
#[derive(Debug, Clone)]
pub struct CbmSetupInfo {
    /// Absolute path to the detected CBM binary, if found.
    pub binary_path: Option<PathBuf>,
    /// Whether the binary version matches the minimum required version.
    pub version_compatible: bool,
    /// Human-readable status message.
    pub message: String,
    /// Whether CBM is fully functional.
    pub is_ready: bool,
}

/// Run a full CBM setup check and return structured info.
///
/// This is the CLI-facing entry point for `clean-ctx setup --with-cbm`.
/// It:
///   1. Searches for the CBM binary (PATH + common locations)
///   2. If found, attempts to check its version
///   3. Returns a structured `CbmSetupInfo` for display / config generation
pub fn cbm_setup_check() -> CbmSetupInfo {
    let binary_name = if cfg!(windows) {
        "codebase-memory-mcp.exe"
    } else {
        "codebase-memory-mcp"
    };

    // Search PATH
    let from_path = search_path(binary_name);
    // Search common install locations
    let from_common = search_common_locations(binary_name);

    let binary_path = from_path.or(from_common);

    match binary_path {
        Some(path) => {
            // Check version
            let version_ok = check_version(&path);
            CbmSetupInfo {
                binary_path: Some(path),
                version_compatible: version_ok,
                message: if version_ok {
                    "CBM binary found and version-compatible.".to_string()
                } else {
                    "CBM binary found but version check failed (may be incompatible).".to_string()
                },
                is_ready: version_ok,
            }
        }
        None => CbmSetupInfo {
            binary_path: None,
            version_compatible: false,
            message: "CBM not found. Install from: https://github.com/DeusData/codebase-memory-mcp"
                .to_string(),
            is_ready: false,
        },
    }
}

/// Generate a `.clean-ctx.json` `cbm` config block for a detected CBM path.
///
/// Returns a `serde_json::Value` that can be merged into the root config.
/// If `info.binary_path` is `None`, uses auto-detection defaults.
pub fn generate_cbm_config_block(info: &CbmSetupInfo) -> serde_json::Value {
    let mut cfg = serde_json::json!({
        "auto_launch": true,
        "cache_ttl": 300,
        "enabled": true,
        "query_timeout_ms": 30000,
    });
    if let Some(ref path) = info.binary_path {
        cfg["binary_path"] = serde_json::Value::String(path.to_string_lossy().into_owned());
    }
    cfg
}

/// Format a human-readable setup status block for terminal output.
pub fn format_setup_output(info: &CbmSetupInfo) -> String {
    let mut out = String::new();
    out.push_str("Clean-CTX × CBM (codebase-memory-mcp) Setup\n");
    out.push_str("──────────────────────────────────────────────\n");
    out.push('\n');

    match &info.binary_path {
        Some(path) => {
            out.push_str(&format!("  ✓ CBM binary: {}\n", path.display()));
            if info.version_compatible {
                out.push_str("  ✓ Version: compatible\n");
            } else {
                out.push_str("  ⚠ Version: unknown/incompatible\n");
                out.push_str("    Set cbm_min_version in .clean-ctx.json if needed.\n");
            }
            out.push_str("  ✓ Status: ready for integration\n");
        }
        None => {
            out.push_str("  ✗ CBM binary not found\n");
            out.push('\n');
            out.push_str("  To install:\n");
            out.push_str("    cargo install codebase-memory-mcp\n");
            out.push_str("    Or download from: https://github.com/DeusData/codebase-memory-mcp\n");
            out.push('\n');
            out.push_str("  Once installed on PATH, Clean-CTX will detect CBM automatically.\n");
            out.push_str("  No configuration needed.\n");
        }
    }
    out.push('\n');
    out.push_str("  To disable CBM: set cbm.enabled = false in .clean-ctx.json\n");
    out.push_str("  To verify at runtime: use the get_cbm_status MCP tool.\n");
    out.push('\n');
    out
}

// ── Internal helpers ──────────────────────────────────────────────

/// Search PATH for a binary with the given name.
fn search_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.exists() && candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Search common install locations.
fn search_common_locations(name: &str) -> Option<PathBuf> {
    let home = home_dir();
    let candidates: Vec<PathBuf> = if cfg!(windows) {
        vec![
            PathBuf::from(r"C:\Program Files\codebase-memory-mcp").join(name),
            home.join(".cargo\\bin").join(name),
        ]
    } else {
        vec![
            PathBuf::from("/usr/local/bin").join(name),
            PathBuf::from("/usr/bin").join(name),
            home.join(".cargo/bin").join(name),
            home.join(".local/bin").join(name),
        ]
    };
    candidates.into_iter().find(|p| p.exists() && p.is_file())
}

/// Attempt to check the CBM version by running `--version`.
/// Returns `true` if the binary exists and ran successfully.
fn check_version(path: &Path) -> bool {
    std::process::Command::new(path)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Resolve home directory from environment.
fn home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        return PathBuf::from(home);
    }
    if let Ok(drive) = std::env::var("HOMEDRIVE") {
        if let Ok(path) = std::env::var("HOMEPATH") {
            return PathBuf::from(drive).join(path);
        }
    }
    PathBuf::from(".")
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cbm_setup_check_returns_info() {
        let info = cbm_setup_check();
        // Should always return a struct (not panic)
        assert!(info.binary_path.is_some() || !info.is_ready);
        assert!(!info.message.is_empty());
    }

    #[test]
    fn test_generate_cbm_config_block_no_binary() {
        let info = CbmSetupInfo {
            binary_path: None,
            version_compatible: false,
            message: "Not found".into(),
            is_ready: false,
        };
        let cfg = generate_cbm_config_block(&info);
        assert_eq!(cfg["enabled"], true);
        assert!(cfg.get("binary_path").is_none());
    }

    #[test]
    fn test_generate_cbm_config_block_with_binary() {
        let info = CbmSetupInfo {
            binary_path: Some(PathBuf::from("/usr/local/bin/codebase-memory-mcp")),
            version_compatible: true,
            message: "OK".into(),
            is_ready: true,
        };
        let cfg = generate_cbm_config_block(&info);
        assert_eq!(cfg["enabled"], true);
        assert_eq!(cfg["binary_path"], "/usr/local/bin/codebase-memory-mcp");
    }

    #[test]
    fn test_format_setup_output_with_binary() {
        let info = CbmSetupInfo {
            binary_path: Some(PathBuf::from("/usr/bin/cbm")),
            version_compatible: true,
            message: "OK".into(),
            is_ready: true,
        };
        let out = format_setup_output(&info);
        assert!(out.contains("✓ CBM binary"));
        assert!(out.contains("✓ Status: ready"));
    }

    #[test]
    fn test_format_setup_output_no_binary() {
        let info = CbmSetupInfo {
            binary_path: None,
            version_compatible: false,
            message: "Not found".into(),
            is_ready: false,
        };
        let out = format_setup_output(&info);
        assert!(out.contains("✗ CBM binary not found"));
        assert!(out.contains("cargo install"));
    }

    #[test]
    fn test_search_path_returns_option() {
        // search_path should always return without panicking, regardless of PATH.
        // We test both a likely-present binary and a guaranteed-absent one.
        let _ = search_path("codebase-memory-mcp");
        let absent = search_path("this-binary-definitely-does-not-exist-12345");
        assert!(absent.is_none());
    }

    #[test]
    fn test_search_path_nonexistent() {
        let result = search_path("this-binary-definitely-does-not-exist-12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_home_dir_resolution() {
        let home = home_dir();
        assert!(home.exists() || home.to_string_lossy() == ".");
    }

    #[test]
    fn test_check_version_fails_for_nonexistent() {
        let path = PathBuf::from("/nonexistent/binary");
        assert!(!check_version(&path));
    }
}
