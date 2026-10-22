//! Leading AST comments own compiler directives; other text cannot admit sources.
use super::{Resolver, module_resolution::ModuleRequestKind, probe::FileExtensions};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum DirectiveKind {
    Path,
    Types,
    Library,
    NoDefaultLibrary,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DirectiveRequest {
    pub source: PathBuf,
    pub source_hash: String,
    pub start: usize,
    pub end: usize,
    pub kind: DirectiveKind,
    pub specifier: String,
    pub ignored: bool,
    pub resolution_mode: Option<Result<ModuleRequestKind, String>>,
    pub resolved: Result<PathBuf, String>,
}

fn attributes(text: &str) -> Result<Option<BTreeMap<String, String>>, String> {
    let Some(mut rest) = text
        .strip_prefix("///")
        .map(str::trim_start)
        .and_then(|text| text.strip_prefix("<reference"))
    else {
        return Ok(None);
    };
    if !rest.starts_with(char::is_whitespace) {
        return Ok(None);
    }
    let mut attributes = BTreeMap::new();
    loop {
        rest = rest.trim_start();
        if rest == "/>" || rest == ">" {
            return Ok(Some(attributes));
        }
        let length = rest
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
            .count();
        if length == 0 {
            return Err("Malformed reference directive".into());
        }
        let name = &rest[..length];
        rest = rest[length..]
            .trim_start()
            .strip_prefix('=')
            .ok_or("Directive attribute lacks equals")?
            .trim_start();
        let quote = rest
            .chars()
            .next()
            .filter(|quote| matches!(quote, '\'' | '"'))
            .ok_or("Directive attribute is not quoted")?;
        rest = &rest[1..];
        let end = rest.find(quote).ok_or("Unclosed directive attribute")?;
        if attributes
            .insert(name.to_string(), rest[..end].to_string())
            .is_some()
        {
            return Err("Duplicate directive attribute".into());
        }
        rest = &rest[end + 1..];
    }
}

impl Resolver {
    pub(super) fn directive_requests(
        &mut self,
        file: &Path,
        source: &str,
        ignored: bool,
    ) -> Result<Vec<DirectiveRequest>, String> {
        let tree = self
            .source_tree(file)
            .ok_or("Directive source tree unavailable")?;
        let mut cursor = tree.root_node().walk();
        let mut requests = Vec::new();
        for node in tree.root_node().named_children(&mut cursor) {
            if node.kind() != "comment" {
                break;
            }
            let text = node
                .utf8_text(source.as_bytes())
                .map_err(|error| error.to_string())?
                .trim_end();
            let Some(attributes) = attributes(text)? else {
                continue;
            };
            let mut selected = Vec::new();
            for (name, kind) in [
                ("path", DirectiveKind::Path),
                ("types", DirectiveKind::Types),
                ("lib", DirectiveKind::Library),
            ] {
                if let Some(value) = attributes.get(name) {
                    selected.push((kind, value));
                }
            }
            if selected.len() != 1 {
                if attributes
                    .get("no-default-lib")
                    .is_some_and(|value| value == "true")
                {
                    requests.push(DirectiveRequest {
                        source: file.to_path_buf(),
                        source_hash: crate::cache::content_hash(source.as_bytes()),
                        start: node.start_byte(),
                        end: node.end_byte(),
                        kind: DirectiveKind::NoDefaultLibrary,
                        specifier: "true".into(),
                        ignored: false,
                        resolution_mode: None,
                        resolved: Ok(file.to_path_buf()),
                    });
                    continue;
                }
                return Err("Reference directive target is not unique".into());
            }
            let (kind, specifier) = selected.pop().expect("one directive target");
            let resolution_mode =
                attributes
                    .get("resolution-mode")
                    .map(|mode| match mode.as_str() {
                        "import" => Ok(ModuleRequestKind::Import),
                        "require" => Ok(ModuleRequestKind::Require),
                        _ => Err(format!("Invalid type reference resolution-mode: {mode}")),
                    });
            let resolved = if ignored {
                Err("Ignored by noResolve".into())
            } else if resolution_mode.is_some() && !matches!(kind, DirectiveKind::Types) {
                Err("resolution-mode only applies to a type reference".into())
            } else if let Some(Err(error)) = &resolution_mode {
                Err(error.clone())
            } else {
                match kind {
                    DirectiveKind::Path => {
                        let target = file
                            .parent()
                            .ok_or("Directive parent absent")?
                            .join(specifier);
                        if self.host.file(&target) {
                            let physical = self
                                .host
                                .normalized(&target)
                                .ok_or("Directive target outside scope")?;
                            if self.host.normalized(file).as_ref() == Some(&physical) {
                                Err("Self reference directive".into())
                            } else {
                                Ok(physical)
                            }
                        } else {
                            Err("Reference path absent or unavailable".into())
                        }
                    }
                    DirectiveKind::Types => self.type_directive_with_mode(
                        file,
                        specifier,
                        resolution_mode
                            .as_ref()
                            .and_then(|mode| mode.as_ref().ok())
                            .copied()
                            .unwrap_or(ModuleRequestKind::StaticImport),
                    ),
                    DirectiveKind::Library => self.library_source(file, specifier, false),
                    DirectiveKind::NoDefaultLibrary => unreachable!("marker captured separately"),
                }
            };
            requests.push(DirectiveRequest {
                source: file.to_path_buf(),
                source_hash: crate::cache::content_hash(source.as_bytes()),
                start: node.start_byte(),
                end: node.end_byte(),
                kind,
                specifier: specifier.clone(),
                ignored,
                resolution_mode,
                resolved,
            });
        }
        Ok(requests)
    }

    pub(super) fn type_directive(
        &mut self,
        file: &Path,
        specifier: &str,
    ) -> Result<PathBuf, String> {
        self.type_directive_with_mode(file, specifier, ModuleRequestKind::StaticImport)
    }

    pub(super) fn type_directive_with_mode(
        &mut self,
        file: &Path,
        specifier: &str,
        mode: ModuleRequestKind,
    ) -> Result<PathBuf, String> {
        let mut project = self.request_project(file)?;
        self.apply_request_conditions(file, mode, &mut project)?;
        project.extensions = FileExtensions::Declaration;
        let (package, subpath) = super::package_lookup::package_parts(specifier)?;
        let mangled = package
            .strip_prefix('@')
            .unwrap_or(&package)
            .replace('/', "__");
        if let Some(roots) = project.options.get("typeRoots").cloned() {
            for root in roots.value.as_array().ok_or("Invalid typeRoots")? {
                let owner = roots
                    .directory
                    .join(root.as_str().ok_or("Invalid typeRoots entry")?)
                    .join(&mangled);
                if let Some(path) = self.type_reference_package(&owner, &subpath, &project)? {
                    return Ok(path);
                }
            }
        } else {
            let boundary = self
                .host
                .boundary(file)
                .ok_or("Type directive outside scope")?;
            let mut directory = file.parent().ok_or("Type directive parent absent")?;
            loop {
                if let Some(path) = self.type_reference_package(
                    &directory.join("node_modules/@types").join(&mangled),
                    &subpath,
                    &project,
                )? {
                    return Ok(path);
                }
                if directory == boundary {
                    break;
                }
                directory = directory.parent().ok_or("Type directive boundary absent")?;
            }
        }
        Err("Type directive package absent or unresolved".into())
    }
}
