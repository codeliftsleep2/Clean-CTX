//! CBM binary discovery.
//!
//! Resolves the `codebase-memory-mcp` executable from explicit config, PATH, and
//! the common install locations. Split out of `bridge.rs`; no behavior changed.

use crate::cbm::config::CbmConfig;
use std::path::PathBuf;

pub(super) fn resolve_cbm_binary(config: &CbmConfig) -> Option<PathBuf> {
    // 1. Config path (explicit user override)
    if let Some(ref path) = config.binary_path {
        let p = PathBuf::from(path);
        if p.exists() && p.is_file() {
            return Some(p);
        }
        // Config path doesn't exist — fall through to PATH
        eprintln!(
            "[clean-ctx-cbm] Config binary_path '{}' not found, trying PATH...",
            path
        );
    }

    // 2. PATH search — try all candidate names (e.g. .exe, .cmd on Windows)
    let names = cbm_binary_names();
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for name in &names {
                let candidate = dir.join(name);
                if candidate.exists() && candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    // 3. Common install locations — try all candidate names per directory
    common_install_locations(&names)
}

/// Return all candidate binary names for the current platform.
///
/// On Windows, npm global installs create `.cmd` shims, while standalone
/// installers and cargo builds produce `.exe`. We check all variants.
/// On Unix, there's only one name (no extension).
fn cbm_binary_names() -> Vec<String> {
    if cfg!(windows) {
        vec![
            "codebase-memory-mcp.exe".into(),
            "codebase-memory-mcp.cmd".into(),
            "codebase-memory-mcp.bat".into(),
        ]
    } else {
        vec!["codebase-memory-mcp".into()]
    }
}

/// Return the list of common install directories for the current platform.
///
/// Shared between `common_install_locations()` (binary resolution) and
/// `checked_paths()` (diagnostics) to avoid duplication.
fn install_dirs() -> Vec<PathBuf> {
    let home = home_dir();
    if cfg!(windows) {
        vec![
            // Standalone installer (e.g. %LOCALAPPDATA%\Programs\codebase-memory-mcp)
            home.join("AppData\\Local\\Programs\\codebase-memory-mcp"),
            // Program Files
            PathBuf::from(r"C:\Program Files\codebase-memory-mcp"),
            // Cargo install
            home.join(".cargo\\bin"),
            // Manual install / Claude setup
            home.join(".local\\bin"),
            // npm global (Windows)
            home.join("AppData\\Roaming\\npm"),
            // Alternative local install
            home.join("AppData\\Local\\codebase-memory-mcp"),
        ]
    } else {
        vec![
            // System-wide
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/bin"),
            // Cargo install
            home.join(".cargo/bin"),
            // Manual install / Claude setup
            home.join(".local/bin"),
            // npm global (Unix)
            home.join(".npm-global/bin"),
            // Homebrew Apple Silicon
            PathBuf::from("/opt/homebrew/bin"),
        ]
    }
}

fn common_install_locations(names: &[String]) -> Option<PathBuf> {
    // Try each directory × each candidate name
    for dir in &install_dirs() {
        for name in names {
            let candidate = dir.join(name);
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Return a list of all candidate binary paths that `resolve_cbm_binary` checks.
///
/// This is used by `get_cbm_status` for diagnostics — when CBM is unavailable,
/// the response includes this list so the user can see exactly what was searched
/// and identify why detection failed (e.g. binary installed in an unlisted dir).
pub fn checked_paths() -> Vec<String> {
    let names = cbm_binary_names();
    let mut paths = Vec::new();

    for dir in &install_dirs() {
        for name in &names {
            paths.push(dir.join(name).to_string_lossy().into_owned());
        }
    }

    // Also note PATH search
    paths.push("(PATH search for all candidate names)".into());

    paths
}

fn home_dir() -> PathBuf {
    // C-3 fix: use HOME/USERPROFILE env var instead of hardcoded username
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        return PathBuf::from(home);
    }
    // Last resort for legacy windows
    if let Ok(drive) = std::env::var("HOMEDRIVE") {
        if let Ok(path) = std::env::var("HOMEPATH") {
            return PathBuf::from(drive).join(path);
        }
    }
    PathBuf::from(".")
}
