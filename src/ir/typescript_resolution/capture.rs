//! Source-owned neutral evidence retained by the real compilation artifact.
use super::{DeclarationId, Namespace, Resolution, Resolver, Witness};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TypeScriptAuthority {
    pub source: PathBuf,
    pub source_route: PathBuf,
    pub source_hash: String,
    pub declarations: Vec<DeclarationId>,
    pub references: Vec<(String, Namespace, Resolution)>,
    /// Exact value occurrences proven by shared lexical/import mechanics.
    pub reference_sites: Vec<(usize, usize, Resolution)>,
    /// Exact type occurrences remain distinct from runtime value references.
    pub type_reference_sites: Vec<(usize, usize, Resolution)>,
    pub module_requests: Vec<super::module_references::ModuleReference>,
    pub compiler_libraries: Vec<super::compiler_libraries::LibraryRequest>,
    pub automatic_types: Vec<super::automatic_types::AutomaticTypeRequest>,
    pub program_directives: Vec<super::program_directives::DirectiveRequest>,
    pub programs: Vec<super::program::ProgramEvidence>,
    pub namespace_scopes: Vec<super::declared_scopes::DeclaredScopeEvidence>,
    pub project: Option<(PathBuf, Option<PathBuf>, String)>,
    pub project_ownership: Option<super::project_membership::Ownership>,
    pub compiler_version: Option<String>,
    /// Consumed effective inputs and their origins, not a blanket certificate
    /// that every compiler option is already interpreted by the resolver.
    pub compiler_options: std::collections::BTreeMap<String, super::CompilerOption>,
    pub witnesses: Vec<Witness>,
    pub capture_current: bool,
    pub disposition: Option<String>,
    /// Supplied bytes authorize only this source when no project roots were passed.
    pub source_only: bool,
}

impl TypeScriptAuthority {
    pub(crate) fn capture(file: &Path, source: &str, roots: &[PathBuf]) -> Self {
        Self::capture_in_project(file, source, roots, None)
    }

    pub(crate) fn capture_in_project(
        file: &Path,
        source: &str,
        roots: &[PathBuf],
        project: Option<&Path>,
    ) -> Self {
        let mut result = Self {
            source: file.to_path_buf(),
            source_route: file.to_path_buf(),
            source_hash: crate::cache::content_hash(source.as_bytes()),
            declarations: Vec::new(),
            references: Vec::new(),
            reference_sites: Vec::new(),
            type_reference_sites: Vec::new(),
            module_requests: Vec::new(),
            namespace_scopes: Vec::new(),
            programs: Vec::new(),
            program_directives: Vec::new(),
            automatic_types: Vec::new(),
            compiler_libraries: Vec::new(),
            project: None,
            project_ownership: None,
            compiler_version: None,
            compiler_options: std::collections::BTreeMap::new(),
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
        let resolver = match project {
            Some(project) => Resolver::with_project(&roots, Some(project)),
            None => Resolver::new(&roots),
        };
        let mut resolver = match resolver {
            Ok(resolver) => resolver,
            Err(error) => {
                result.disposition = Some(error);
                return result;
            }
        };
        let capture = (|| {
            result.source = resolver
                .host
                .normalized(file)
                .ok_or("Source outside authorized roots")?;
            resolver.host.seed(file, source)?;
            result.declarations = resolver.source_declarations(file)?;
            result.reference_sites = reference_sites(&mut resolver, file, source)?;
            result.type_reference_sites = type_reference_sites(&mut resolver, file, source)?;
            result.module_requests =
                super::module_references::capture(&mut resolver, file, source)?;
            for (_, _, resolution) in result
                .reference_sites
                .iter()
                .chain(&result.type_reference_sites)
            {
                let ids = match resolution {
                    Resolution::Resolved(id) => std::slice::from_ref(id),
                    Resolution::Merged(ids) => ids.as_slice(),
                    _ => &[],
                };
                result
                    .declarations
                    .extend(ids.iter().filter(|id| id.source == result.source).cloned());
            }
            result.declarations.sort();
            result.declarations.dedup();
            result.namespace_scopes =
                resolver.in_source_project(file, |resolver| resolver.declared_scope_evidence(file));
            let project = resolver.project_identity(file)?;
            result.project = Some((project.root, project.config, project.mode));
            result.project_ownership = project.ownership;
            result.compiler_version = project.compiler_version;
            result.compiler_options = project.options;
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
        result.compiler_libraries = resolver.library_evidence.clone();
        result.automatic_types = resolver.automatic_type_evidence.clone();
        result.program_directives = resolver.directive_evidence.clone();
        result.programs = resolver.program_evidence.values().cloned().collect();
        result.capture_current = resolver.host.current() && capture.is_ok();
        result.witnesses = resolver.host.witnesses;
        if let Err(error) = capture {
            result.disposition = Some(error);
        }
        result
    }
}

fn reference_sites(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
) -> Result<Vec<(usize, usize, Resolution)>, String> {
    let tree = resolver
        .source_tree(file)
        .ok_or("TypeScript parse failed")?;
    if tree.root_node().has_error() {
        return Err("Invalid TypeScript syntax".into());
    }
    let mut pending = vec![tree.root_node()];
    let mut sites = Vec::new();
    while let Some(node) = pending.pop() {
        if matches!(
            node.kind(),
            "identifier" | "shorthand_property_identifier" | "member_expression"
        ) {
            let span =
                crate::ir::invocation_evidence::SourceSpan::new(node.start_byte(), node.end_byte())
                    .map_err(|error| error.to_string())?;
            if let Some(resolution) = resolver.value_reference(file, source, span) {
                sites.push((node.start_byte(), node.end_byte(), resolution));
            }
        }
        let mut cursor = node.walk();
        pending.extend(node.named_children(&mut cursor));
    }
    sites.sort_by_key(|site| (site.0, site.1));
    Ok(sites)
}

fn type_reference_sites(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
) -> Result<Vec<(usize, usize, Resolution)>, String> {
    let tree = resolver
        .source_tree(file)
        .ok_or("TypeScript parse failed")?;
    let mut pending = vec![tree.root_node()];
    let mut sites = Vec::new();
    while let Some(node) = pending.pop() {
        if matches!(
            node.kind(),
            "type_identifier"
                | "nested_type_identifier"
                | "member_expression"
                | "call_expression"
                | "type_query"
        ) {
            let span =
                crate::ir::invocation_evidence::SourceSpan::new(node.start_byte(), node.end_byte())
                    .map_err(|error| error.to_string())?;
            if let Some(resolution) = resolver.type_reference(file, source, span) {
                sites.push((node.start_byte(), node.end_byte(), resolution));
            }
        }
        let mut cursor = node.walk();
        pending.extend(node.named_children(&mut cursor));
    }
    sites.sort_by_key(|site| (site.0, site.1));
    Ok(sites)
}
