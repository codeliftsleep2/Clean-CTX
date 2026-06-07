// src/mcp/workspace.rs
//
// Workspace-level operations: scanning directories and compressing all files.

use std::path::PathBuf;
use crate::compressor::{compress_file, Fidelity};
use crate::dictionary::PathDictionary;
use crate::cache::LocalStateCache;

/// Scan a directory for .ts/.cs files and compress each one.
pub(crate) fn compress_workspace_dir(
    dir_path: &str,
    fidelity: Fidelity,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut dict = PathDictionary::new();
    let mut cache = LocalStateCache::new();
    let mut manifest = String::new();
    manifest.push_str("// Clean-CTX Workspace Manifest\n");
    manifest.push_str(&format!("// Directory: {}\n", dir_path));
    manifest.push_str(&format!("// Fidelity: {:?}\n\n", fidelity));

    let mut entries: Vec<String> = Vec::new();

    // Collect all .ts and .cs files recursively
    collect_source_files(dir_path, &mut entries);
    entries.sort();

    for entry in &entries {
        match compress_file(PathBuf::from(entry), &mut dict, &mut cache, fidelity) {
            Ok(mut compressed) => {
                compressed.push_str(&dict.format_footer());
                manifest.push_str(&format!("// ===== FILE: {} =====\n", entry));
                manifest.push_str(&compressed);
                manifest.push('\n');
            }
            Err(e) => {
                manifest.push_str(&format!("// ERROR compressing {}: {}\n\n", entry, e));
            }
        }
    }

    // Append the global path map
    manifest.push_str(&dict.format_footer());

    Ok(manifest)
}

/// Recursively collect .ts and .cs files from a directory.
pub(crate) fn collect_source_files(dir: &str, entries: &mut Vec<String>) {
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            // Skip hidden dirs, node_modules, target, etc.
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
                continue;
            }

            if path.is_dir() {
                collect_source_files(&path.to_string_lossy(), entries);
            } else if path.is_file() {
                let ext = path.extension().unwrap_or_default().to_string_lossy();
                if ext == "ts" || ext == "js" || ext == "cs" {
                    entries.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }
}