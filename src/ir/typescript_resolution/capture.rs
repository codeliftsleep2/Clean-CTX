//! Source-owned neutral evidence retained by the real compilation artifact.
use super::{DeclarationId, Namespace, Resolution, Resolver, Witness};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TypeScriptAuthority {
    pub source: PathBuf,
    pub source_hash: String,
    pub declarations: Vec<DeclarationId>,
    pub references: Vec<(String, Namespace, Resolution)>,
    pub project: Option<(PathBuf, Option<PathBuf>, String)>,
    pub witnesses: Vec<Witness>,
    pub capture_current: bool,
    pub disposition: Option<String>,
    /// Supplied bytes authorize only this source when no project roots were passed.
    pub source_only: bool,
}

impl TypeScriptAuthority {
    pub(crate) fn capture(file: &Path, source: &str, roots: &[PathBuf]) -> Self {
        let mut result = Self {
            source: file.to_path_buf(),
            source_hash: crate::cache::content_hash(source.as_bytes()),
            declarations: Vec::new(),
            references: Vec::new(),
            project: None,
            witnesses: Vec::new(),
            capture_current: false,
            disposition: None,
            source_only: roots.is_empty(),
        };
        // A supplied source snapshot authorizes that one file. It does not
        // authorize an inferred parent/project/workspace or dependency reads.
        let roots = if roots.is_empty() {
            vec![file.to_path_buf()]
        } else {
            roots.to_vec()
        };
        let mut resolver = match Resolver::new(&roots) {
            Ok(resolver) => resolver,
            Err(error) => {
                result.disposition = Some(error);
                return result;
            }
        };
        let capture = (|| {
            resolver.host.seed(file, source)?;
            result.declarations = resolver.source_declarations(file)?;
            result.project = Some(resolver.project_identity(file)?);
            for name in resolver.source_names(file) {
                for namespace in [Namespace::Value, Namespace::Type] {
                    let resolution = resolver.local(file, &name, namespace);
                    result
                        .references
                        .push((name.clone(), namespace, resolution));
                }
            }
            Ok::<_, String>(())
        })();
        result.capture_current = resolver.host.current() && capture.is_ok();
        result.witnesses = resolver.host.witnesses;
        if let Err(error) = capture {
            result.disposition = Some(error);
        }
        result
    }
}
