//! Angular external-template ownership.
//!
//! The existing TypeScript capture pipeline and decorator parser are the only
//! declaration authorities. Filename convention is a fallback route, not
//! ownership evidence.

use std::path::{Path, PathBuf};

/// Return whether a TypeScript source structurally declares the target as an
/// external component template.
pub(crate) fn source_owns_external_template(
    source: &str,
    declaring_file: &Path,
    target: &Path,
) -> bool {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return false;
    };
    let Ok(captures) = crate::compression::capture_pipeline::run_capture_pipeline_nodes(
        language,
        crate::queries::TS_QUERY,
        source,
        crate::compression::Fidelity::Low,
        |_, raw, _| Some(raw.to_string()),
    ) else {
        return false;
    };
    let target_identity =
        crate::dictionary::path::canonical_identity_key(&target.to_string_lossy());

    captures
        .iter()
        .filter(|capture| capture.name == "class.root")
        .filter_map(|capture| {
            let entry = capture.to_entry();
            let raw_class = crate::meta_util::class_source_from_capture(source, &entry);
            crate::angular_meta::decorators::extract_component_template_url(raw_class)
        })
        .any(|template_url| {
            resolve_template_path(declaring_file, &template_url).is_some_and(|path| {
                crate::dictionary::path::canonical_identity_key(&path.to_string_lossy())
                    == target_identity
            })
        })
}

fn resolve_template_path(declaring_file: &Path, written: &str) -> Option<PathBuf> {
    let written = written.trim();
    if written.is_empty() || written.contains("://") {
        return None;
    }
    let path = Path::new(written);
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        declaring_file.parent().map(|parent| parent.join(path))
    }
}
