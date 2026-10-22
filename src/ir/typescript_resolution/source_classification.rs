//! One consumed source-format policy feeds module requests and program globals.
use super::{Resolver, package::Ordered, project::Project, syntax::Unit};
use std::path::Path;

pub(super) fn declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            [".d.ts", ".d.mts", ".d.cts"]
                .iter()
                .any(|suffix| name.ends_with(suffix))
        })
}

impl Resolver {
    pub(super) fn source_is_esm(&mut self, file: &Path) -> Result<bool, String> {
        match file.extension().and_then(|extension| extension.to_str()) {
            Some("mts" | "mjs") => Ok(true),
            Some("cts" | "cjs") => Ok(false),
            _ => Ok(self
                .nearest_manifest(file.parent().ok_or("Source parent absent")?)?
                .is_some_and(|(_, (_, json))| {
                    json.get("type").and_then(Ordered::string) == Some("module")
                })),
        }
    }

    pub(super) fn program_source_external(
        &mut self,
        unit: &Unit,
        project: &Project,
    ) -> Result<bool, String> {
        let file = unit
            .source
            .as_deref()
            .ok_or("Program source identity absent")?;
        if unit.external {
            return Ok(true);
        }
        let detection = project
            .options
            .get("moduleDetection")
            .map(|option| option.value.as_str().ok_or("Invalid moduleDetection"))
            .transpose()?
            .or_else(|| {
                super::project_version::detection_default(
                    &project.options,
                    project.compiler_version.as_deref(),
                )
            })
            .ok_or("Module detection default requires reviewed compiler evidence")?
            .to_ascii_lowercase();
        if !matches!(detection.as_str(), "legacy" | "auto" | "force") {
            return Err("Invalid moduleDetection".into());
        }
        // Force excludes declaration files; their ambient globals remain scripts.
        if declaration_file(file) {
            return Ok(false);
        }
        match detection.as_str() {
            "force" => Ok(true),
            "legacy" => Ok(false),
            _ => {
                if matches!(
                    file.extension().and_then(|extension| extension.to_str()),
                    Some("mts" | "cts" | "mjs" | "cjs")
                ) {
                    return Ok(true);
                }
                let node_module = project
                    .options
                    .get("module")
                    .and_then(|option| option.value.as_str())
                    .is_some_and(|module| {
                        matches!(module.to_ascii_lowercase().as_str(), "node16" | "nodenext")
                    });
                if node_module && self.source_is_esm(file)? {
                    return Ok(true);
                }
                let automatic_jsx = project
                    .options
                    .get("jsx")
                    .and_then(|option| option.value.as_str())
                    .is_some_and(|jsx| {
                        matches!(
                            jsx.to_ascii_lowercase().as_str(),
                            "react-jsx" | "react-jsxdev"
                        )
                    });
                if automatic_jsx {
                    let tree = unit.tree.as_ref().ok_or("Program syntax tree absent")?;
                    let mut pending = vec![tree.root_node()];
                    while let Some(node) = pending.pop() {
                        if matches!(node.kind(), "jsx_element" | "jsx_self_closing_element") {
                            return Ok(true);
                        }
                        let mut cursor = node.walk();
                        pending.extend(node.named_children(&mut cursor));
                    }
                }
                Ok(false)
            }
        }
    }
}
