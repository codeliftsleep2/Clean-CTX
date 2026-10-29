//! Source-owned neutral evidence retained by the real compilation artifact.
use super::{DeclarationId, Namespace, Resolution, Resolver, Witness};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TypeScriptAuthority {
    pub source: PathBuf,
    pub source_route: PathBuf,
    pub source_hash: String,
    pub declarations: Vec<DeclarationId>,
    #[serde(default)]
    pub member_declarations: Vec<super::member_declarations::MemberDeclaration>,
    #[serde(default)]
    pub constructor_parameters: Vec<super::constructor_parameters::ConstructorParameter>,
    #[serde(default)]
    pub declaration_facets: Vec<super::declaration_facets::DeclarationFacet>,
    #[serde(default)]
    pub demanded_occurrences: Vec<super::demanded_occurrences::DemandedOccurrence>,
    #[serde(default)]
    pub invocation_facets: Vec<super::declaration_facets::InvocationFacet>,
    #[serde(default)]
    pub declared_return_members: Vec<super::declared_return_member::DeclaredReturnMember>,
    #[serde(default)]
    pub arrow_selections: Vec<super::arrow_selection::ArrowSelection>,
    pub references: Vec<(String, Namespace, Resolution)>,
    /// Exact value occurrences proven by shared lexical/import mechanics.
    pub reference_sites: Vec<(usize, usize, Resolution)>,
    /// Bare value occurrences with no source binding and a closed empty global
    /// lookup. Symbolic compatibility only; never declaration authority.
    #[serde(default)]
    pub unbound_value_sites: Vec<(usize, usize, String)>,
    /// Exact type occurrences remain distinct from runtime value references.
    pub type_reference_sites: Vec<(usize, usize, Resolution)>,
    #[serde(default)]
    pub package_exports: Vec<super::package_evidence::PackageExportEvidence>,
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
    /// Select a unique resolved value occurrence, without assigning a consumer role.
    /// Type occurrences and competing entries cannot supply this evidence.
    pub(crate) fn value_reference_at(
        &self,
        span: crate::ir::invocation_evidence::SourceSpan,
    ) -> Option<&DeclarationId> {
        let mut sites = self
            .reference_sites
            .iter()
            .filter(|(start, end, _)| *start == span.start_byte() && *end == span.end_byte());
        let Some((_, _, resolution)) = sites.next() else {
            return self
                .declared_return_member_at(span)
                .map(|proof| &proof.method);
        };
        if sites.next().is_some() {
            return None;
        }
        let Resolution::Resolved(id) = resolution else {
            return None;
        };
        Some(id)
    }

    pub(crate) fn capture(file: &Path, source: &str, roots: &[PathBuf]) -> Self {
        Self::capture_in_project(file, source, roots, None)
    }

    pub(crate) fn capture_in_project(
        file: &Path,
        source: &str,
        roots: &[PathBuf],
        project: Option<&Path>,
    ) -> Self {
        Self::capture_with_declaration_fields(file, source, roots, project, &[])
    }

    pub(crate) fn capture_with_declaration_fields(
        file: &Path,
        source: &str,
        roots: &[PathBuf],
        project: Option<&Path>,
        fields: &[&str],
    ) -> Self {
        Self::capture_with_demands(file, source, roots, project, fields, &[])
    }

    pub(crate) fn capture_with_demands(
        file: &Path,
        source: &str,
        roots: &[PathBuf],
        project: Option<&Path>,
        fields: &[&str],
        call_apis: &[(&str, &str)],
    ) -> Self {
        let mut result = Self {
            source: file.to_path_buf(),
            source_route: file.to_path_buf(),
            source_hash: crate::cache::content_hash(source.as_bytes()),
            declarations: Vec::new(),
            member_declarations: Vec::new(),
            constructor_parameters: Vec::new(),
            declaration_facets: Vec::new(),
            demanded_occurrences: Vec::new(),
            invocation_facets: Vec::new(),
            declared_return_members: Vec::new(),
            arrow_selections: Vec::new(),
            references: Vec::new(),
            reference_sites: Vec::new(),
            unbound_value_sites: Vec::new(),
            type_reference_sites: Vec::new(),
            package_exports: Vec::new(),
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
            result.member_declarations = super::member_declarations::capture(
                &mut resolver,
                file,
                source,
                &result.declarations,
            );
            result.declarations.extend(
                result
                    .member_declarations
                    .iter()
                    .map(|entry| entry.member.clone()),
            );
            result.constructor_parameters = super::constructor_parameters::capture(
                &mut resolver,
                file,
                source,
                &result.declarations,
            );
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
            result.package_exports =
                super::package_evidence::capture(&mut resolver, file, &result.reference_sites);
            result
                .package_exports
                .extend(super::package_evidence::capture_in_namespace(
                    &mut resolver,
                    file,
                    &result.type_reference_sites,
                    Namespace::Type,
                ));
            let (sites, packages, mut selected) =
                super::invocation_sites::capture(&mut resolver, file, source, call_apis);
            for (start, end, resolution) in sites {
                if let Some((_, _, known)) = result
                    .reference_sites
                    .iter_mut()
                    .find(|(s, e, _)| *s == start && *e == end)
                {
                    if !matches!(
                        known,
                        Resolution::Resolved(_) | Resolution::Merged(_) | Resolution::Ambiguous(_)
                    ) {
                        *known = resolution;
                    }
                } else {
                    result.reference_sites.push((start, end, resolution));
                }
            }
            result
                .reference_sites
                .sort_by_key(|(start, end, _)| (*start, *end));
            result.package_exports.extend(packages);
            let (receiver_proofs, receiver_occurrences) =
                super::declared_return_member::capture(&mut resolver, file, source, &selected);
            // Export proof for the already-demanded explicit return reference.
            for proof in &receiver_proofs {
                result
                    .package_exports
                    .extend(super::package_evidence::capture_in_namespace(
                        &mut resolver,
                        &proof.factory.source,
                        &[(
                            proof.return_site.0,
                            proof.return_site.1,
                            Resolution::Resolved(proof.receiver.clone()),
                        )],
                        Namespace::Type,
                    ));
            }
            selected.extend(receiver_proofs.iter().map(|proof| proof.access));
            result.declared_return_members = receiver_proofs;
            let (facets, packages, occurrences, invocations, selections) =
                super::declaration_facets::capture(
                    &mut resolver,
                    file,
                    &result.declarations,
                    fields,
                    &result.constructor_parameters,
                    &selected,
                );
            result.arrow_selections = selections;
            result.declaration_facets = facets;
            result.demanded_occurrences = occurrences;
            for site in receiver_occurrences {
                if !result.demanded_occurrences.contains(&site) {
                    result.demanded_occurrences.push(site);
                }
            }
            result.invocation_facets = invocations;
            result.package_exports.extend(packages);
            Ok::<_, String>(())
        })();
        result.unbound_value_sites = resolver
            .unbound_value_sites
            .iter()
            .filter(|(owner, start, end, _)| {
                owner == &result.source
                    && result
                        .reference_sites
                        .iter()
                        .any(|(s, e, _)| s == start && e == end)
            })
            .map(|(_, start, end, name)| (*start, *end, name.clone()))
            .collect();
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
