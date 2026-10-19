//! Framework-neutral C# type-reference resolution contract.
//!
//! R0 introduces the value boundary only. Later campaign slices populate the
//! admitted-source universe, imports, aliases, partial symbols, and ancestry.

use crate::compression::capture_pipeline::CapEntry;

mod aliases;
pub use aliases::{CSharpAliasContext, CSharpAliasDirective, resolve_alias_type_reference};
mod attributes;
pub use attributes::resolve_attribute_type_reference;
mod ancestry;
pub use ancestry::{CSharpInheritanceArc, CSharpInheritanceGraph, resolve_type_ancestry};
mod compilation;
pub(crate) use compilation::{
    CSharpCompilationUnit, CSharpResolvedClass, collect_compilation_unit,
    resolve_compilation_class_ancestry, resolve_compilation_units,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CSharpSourceSpan {
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CSharpDeclarationKind {
    Class,
    Interface,
    Struct,
    Enum,
    Record,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CSharpDeclarationFact {
    pub name: String,
    pub kind: CSharpDeclarationKind,
    pub span: CSharpSourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CSharpTypeReferenceKind {
    BaseType,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CSharpTypeReferenceFact {
    pub written: String,
    pub kind: CSharpTypeReferenceKind,
    pub span: CSharpSourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CSharpAttributeReferenceFact {
    pub written: String,
    pub span: CSharpSourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CSharpUsingDirectiveKind {
    Namespace,
    Alias,
    Static,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CSharpUsingDirectiveFact {
    pub written: String,
    pub kind: CSharpUsingDirectiveKind,
    pub span: CSharpSourceSpan,
}

/// Parser-owned, framework-neutral C# syntax facts from one compilation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CSharpSyntaxFacts {
    pub declarations: Vec<CSharpDeclarationFact>,
    pub type_references: Vec<CSharpTypeReferenceFact>,
    pub attributes: Vec<CSharpAttributeReferenceFact>,
    pub using_directives: Vec<CSharpUsingDirectiveFact>,
}

pub(crate) fn collect_syntax_facts(captures: &[CapEntry]) -> CSharpSyntaxFacts {
    let mut facts = CSharpSyntaxFacts::default();
    for capture in captures {
        let span = CSharpSourceSpan {
            start_byte: capture.start_byte,
            end_byte: capture.end_byte,
        };
        match capture.name.as_str() {
            "csharp.class_name" => facts
                .declarations
                .push(declaration(capture, CSharpDeclarationKind::Class)),
            "csharp.interface_name" => facts
                .declarations
                .push(declaration(capture, CSharpDeclarationKind::Interface)),
            "csharp.struct_name" => facts
                .declarations
                .push(declaration(capture, CSharpDeclarationKind::Struct)),
            "csharp.enum_name" => facts
                .declarations
                .push(declaration(capture, CSharpDeclarationKind::Enum)),
            "csharp.record_name" => facts
                .declarations
                .push(declaration(capture, CSharpDeclarationKind::Record)),
            "csharp.base_type" => facts.type_references.push(CSharpTypeReferenceFact {
                written: capture.raw_text.trim().to_string(),
                kind: CSharpTypeReferenceKind::BaseType,
                span,
            }),
            "csharp.attribute" => {
                let raw = capture.raw_text.trim();
                let written = raw.split_once('(').map_or(raw, |(name, _)| name).trim();
                if !written.is_empty() {
                    let leading = capture
                        .raw_text
                        .find(written)
                        .expect("written attribute name is a source subslice");
                    facts.attributes.push(CSharpAttributeReferenceFact {
                        written: written.to_string(),
                        span: CSharpSourceSpan {
                            start_byte: capture.start_byte + leading,
                            end_byte: capture.start_byte + leading + written.len(),
                        },
                    });
                }
            }
            "import.root" => {
                let written = capture.raw_text.trim();
                let rest = written.strip_prefix("using ").unwrap_or(written);
                let kind = if rest.starts_with("static ") {
                    CSharpUsingDirectiveKind::Static
                } else if rest.contains('=') {
                    CSharpUsingDirectiveKind::Alias
                } else {
                    CSharpUsingDirectiveKind::Namespace
                };
                facts.using_directives.push(CSharpUsingDirectiveFact {
                    written: written.to_string(),
                    kind,
                    span,
                });
            }
            _ => {}
        }
    }
    facts
}

fn declaration(capture: &CapEntry, kind: CSharpDeclarationKind) -> CSharpDeclarationFact {
    CSharpDeclarationFact {
        name: capture.raw_text.clone(),
        kind,
        span: CSharpSourceSpan {
            start_byte: capture.start_byte,
            end_byte: capture.end_byte,
        },
    }
}

/// A framework-neutral C# type identity admitted by a resolution authority.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CSharpTypeIdentity {
    pub qualified_name: String,
    pub namespace_path: Vec<String>,
    pub containing_type_path: Vec<String>,
    pub terminal_name: String,
    pub generic_arity: usize,
    pub origin: CSharpTypeIdentityOrigin,
}

impl CSharpTypeIdentity {
    /// Declare a non-generic external identity known by the caller.
    pub fn known_external(qualified_name: impl Into<String>) -> Self {
        Self::known_external_with_arity(qualified_name, 0)
    }

    pub fn known_external_with_arity(
        qualified_name: impl Into<String>,
        generic_arity: usize,
    ) -> Self {
        let qualified_name = qualified_name.into();
        let mut segments = qualified_name
            .split('.')
            .map(str::to_string)
            .collect::<Vec<_>>();
        let terminal_name = segments.pop().unwrap_or_default();
        Self {
            qualified_name,
            namespace_path: segments,
            containing_type_path: Vec::new(),
            terminal_name,
            generic_arity,
            origin: CSharpTypeIdentityOrigin::KnownExternal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CSharpTypeIdentityOrigin {
    KnownExternal,
    AdmittedSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CSharpResolutionBasis {
    ExplicitQualified,
    GlobalQualified,
    LexicalDeclaration,
    OrdinaryImport,
    NamespaceAlias,
    TypeAlias,
    Inheritance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CSharpResolutionReason {
    MissingAuthority,
    UnsupportedSimpleName,
    UnsupportedAliasQualifiedName,
    UnsupportedQualifiedName,
    MalformedTypeReference,
    CompetingCandidates,
    UnknownImportContents,
    DuplicateAlias,
    AliasCycle,
    InheritanceCycle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CSharpResolutionEvidence {
    pub file: String,
    pub span: CSharpSourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CSharpResolution<T> {
    Resolved {
        identity: T,
        basis: CSharpResolutionBasis,
        evidence: Vec<CSharpResolutionEvidence>,
    },
    Ambiguous(CSharpResolutionReason),
    Unsupported(CSharpResolutionReason),
    Unresolved(CSharpResolutionReason),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CSharpOrdinaryImportContext {
    pub namespace_path: Vec<String>,
    pub containing_type_path: Vec<String>,
    pub ordinary_imports: Vec<Vec<String>>,
}

/// Resolve one written C# type reference against caller-supplied neutral
/// external identities.
pub fn resolve_known_external_type(
    written: &str,
    known_external: &[CSharpTypeIdentity],
) -> CSharpResolution<CSharpTypeIdentity> {
    resolve_explicit_type_reference(
        written,
        known_external,
        &crate::ir::csharp_symbols::CSharpSourceSymbolIndex::default(),
    )
}

/// Resolve an explicitly qualified C# type reference against neutral external
/// identities and the admitted-source symbol index.
pub fn resolve_explicit_type_reference(
    written: &str,
    known_external: &[CSharpTypeIdentity],
    source_symbols: &crate::ir::csharp_symbols::CSharpSourceSymbolIndex,
) -> CSharpResolution<CSharpTypeIdentity> {
    let written = written.trim();
    let (written, basis) = if let Some(qualified) = written.strip_prefix("global::") {
        (qualified, CSharpResolutionBasis::GlobalQualified)
    } else if written.contains("::") {
        return CSharpResolution::Unsupported(
            CSharpResolutionReason::UnsupportedAliasQualifiedName,
        );
    } else {
        (written, CSharpResolutionBasis::ExplicitQualified)
    };
    let Some((path, generic_arity)) = parse_explicit_type_path(written) else {
        return CSharpResolution::Unsupported(CSharpResolutionReason::MalformedTypeReference);
    };
    if path.len() < 2 {
        return CSharpResolution::Unsupported(CSharpResolutionReason::UnsupportedSimpleName);
    }

    let qualified_name = path.join(".");
    let external = known_external
        .iter()
        .filter(|identity| {
            identity.qualified_name == qualified_name && identity.generic_arity == generic_arity
        })
        .collect::<Vec<_>>();
    let source = source_symbols.matching_qualified_path(&path, generic_arity);
    if external.len() + source.len() > 1
        || source.iter().any(|composition| {
            matches!(
                composition,
                crate::ir::csharp_symbols::CSharpSymbolComposition::Ambiguous(_)
            )
        })
    {
        return CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates);
    }
    if let Some(identity) = external.first() {
        return CSharpResolution::Resolved {
            identity: (*identity).clone(),
            basis,
            evidence: Vec::new(),
        };
    }
    let Some(crate::ir::csharp_symbols::CSharpSymbolComposition::Resolved(symbol)) = source.first()
    else {
        return CSharpResolution::Unresolved(CSharpResolutionReason::MissingAuthority);
    };
    CSharpResolution::Resolved {
        identity: source_identity(symbol),
        basis,
        evidence: Vec::new(),
    }
}

/// Resolve one simple C# type reference under ordinary-import rules.
pub fn resolve_ordinary_import_type_reference(
    written: &str,
    context: &CSharpOrdinaryImportContext,
    known_external: &[CSharpTypeIdentity],
    source_symbols: &crate::ir::csharp_symbols::CSharpSourceSymbolIndex,
) -> CSharpResolution<CSharpTypeIdentity> {
    let written = written.trim();
    if written.contains('.') || written.contains("::") {
        return CSharpResolution::Unsupported(CSharpResolutionReason::UnsupportedQualifiedName);
    }
    let Some((name, generic_arity)) = parse_named_segment(written) else {
        return CSharpResolution::Unsupported(CSharpResolutionReason::MalformedTypeReference);
    };

    for depth in (0..=context.containing_type_path.len()).rev() {
        let mut path = context.namespace_path.clone();
        path.extend(context.containing_type_path[..depth].iter().cloned());
        path.push(name.clone());
        let candidates = candidates_for_path(&path, generic_arity, known_external, source_symbols);
        if candidates.ambiguous {
            return CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates);
        }
        if let Some(identity) = candidates.unique() {
            return CSharpResolution::Resolved {
                identity,
                basis: CSharpResolutionBasis::LexicalDeclaration,
                evidence: Vec::new(),
            };
        }
    }
    for depth in (0..context.namespace_path.len()).rev() {
        let mut path = context.namespace_path[..depth].to_vec();
        path.push(name.clone());
        let candidates = candidates_for_path(&path, generic_arity, known_external, source_symbols);
        if candidates.ambiguous {
            return CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates);
        }
        if let Some(identity) = candidates.unique() {
            return CSharpResolution::Resolved {
                identity,
                basis: CSharpResolutionBasis::LexicalDeclaration,
                evidence: Vec::new(),
            };
        }
    }

    let mut imported = Vec::new();
    let mut ambiguous = false;
    let mut unknown_import = false;
    for import in &context.ordinary_imports {
        let external_namespace_known = known_external
            .iter()
            .any(|identity| identity.namespace_path == *import);
        if !external_namespace_known && !source_symbols.knows_namespace(import) {
            unknown_import = true;
        }
        let mut path = import.clone();
        path.push(name.clone());
        let candidates = candidates_for_path(&path, generic_arity, known_external, source_symbols);
        ambiguous |= candidates.ambiguous;
        imported.extend(candidates.identities);
    }
    if ambiguous || imported.len() > 1 {
        return CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates);
    }
    if unknown_import {
        return if imported.is_empty() {
            CSharpResolution::Unresolved(CSharpResolutionReason::UnknownImportContents)
        } else {
            CSharpResolution::Ambiguous(CSharpResolutionReason::UnknownImportContents)
        };
    }
    let Some(identity) = imported.pop() else {
        return CSharpResolution::Unresolved(CSharpResolutionReason::MissingAuthority);
    };
    CSharpResolution::Resolved {
        identity,
        basis: CSharpResolutionBasis::OrdinaryImport,
        evidence: Vec::new(),
    }
}

struct ResolutionCandidates {
    identities: Vec<CSharpTypeIdentity>,
    ambiguous: bool,
}

impl ResolutionCandidates {
    fn unique(mut self) -> Option<CSharpTypeIdentity> {
        if self.identities.len() == 1 {
            self.identities.pop()
        } else {
            None
        }
    }
}

fn candidates_for_path(
    path: &[String],
    generic_arity: usize,
    known_external: &[CSharpTypeIdentity],
    source_symbols: &crate::ir::csharp_symbols::CSharpSourceSymbolIndex,
) -> ResolutionCandidates {
    let qualified_name = path.join(".");
    let mut identities = known_external
        .iter()
        .filter(|identity| {
            identity.qualified_name == qualified_name && identity.generic_arity == generic_arity
        })
        .cloned()
        .collect::<Vec<_>>();
    let source = source_symbols.matching_qualified_path(path, generic_arity);
    let ambiguous = source.iter().any(|composition| {
        matches!(
            composition,
            crate::ir::csharp_symbols::CSharpSymbolComposition::Ambiguous(_)
        )
    });
    identities.extend(
        source
            .into_iter()
            .filter_map(|composition| match composition {
                crate::ir::csharp_symbols::CSharpSymbolComposition::Resolved(symbol) => {
                    Some(source_identity(symbol))
                }
                crate::ir::csharp_symbols::CSharpSymbolComposition::Ambiguous(_) => None,
            }),
    );
    ResolutionCandidates {
        ambiguous: ambiguous || identities.len() > 1,
        identities,
    }
}

fn source_identity(symbol: &crate::ir::csharp_symbols::CSharpSourceSymbol) -> CSharpTypeIdentity {
    let mut path = symbol.key.namespace_path.clone();
    path.extend(symbol.key.containing_type_path.iter().cloned());
    path.push(symbol.key.name.clone());
    CSharpTypeIdentity {
        qualified_name: path.join("."),
        namespace_path: symbol.key.namespace_path.clone(),
        containing_type_path: symbol.key.containing_type_path.clone(),
        terminal_name: symbol.key.name.clone(),
        generic_arity: symbol.key.generic_arity,
        origin: CSharpTypeIdentityOrigin::AdmittedSource,
    }
}

fn parse_explicit_type_path(written: &str) -> Option<(Vec<String>, usize)> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    for (index, character) in written.char_indices() {
        match character {
            '<' => depth = depth.checked_add(1)?,
            '>' => depth = depth.checked_sub(1)?,
            '.' if depth == 0 => {
                segments.push(&written[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    segments.push(&written[start..]);

    let terminal = segments.pop()?;
    let (terminal_name, generic_arity) = parse_named_segment(terminal)?;
    let mut path = segments
        .into_iter()
        .map(parse_non_generic_segment)
        .collect::<Option<Vec<_>>>()?;
    path.push(terminal_name);
    Some((path, generic_arity))
}

fn parse_non_generic_segment(segment: &str) -> Option<String> {
    (!segment.contains('<') && !segment.contains('>') && is_identifier(segment))
        .then(|| segment.to_string())
}

fn parse_named_segment(segment: &str) -> Option<(String, usize)> {
    let Some(open) = segment.find('<') else {
        return is_identifier(segment).then(|| (segment.to_string(), 0));
    };
    if !segment.ends_with('>') {
        return None;
    }
    let name = &segment[..open];
    if !is_identifier(name) {
        return None;
    }
    let arguments = &segment[open + 1..segment.len() - 1];
    let arity = generic_argument_count(arguments)?;
    Some((name.to_string(), arity))
}

fn generic_argument_count(arguments: &str) -> Option<usize> {
    if arguments.trim().is_empty() {
        return None;
    }
    let mut depth = 0usize;
    let mut arity = 1usize;
    let mut argument_start = 0usize;
    for (index, character) in arguments.char_indices() {
        match character {
            '<' => depth = depth.checked_add(1)?,
            '>' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                if arguments[argument_start..index].trim().is_empty() {
                    return None;
                }
                arity += 1;
                argument_start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    (depth == 0 && !arguments[argument_start..].trim().is_empty()).then_some(arity)
}

fn is_identifier(segment: &str) -> bool {
    let segment = segment.strip_prefix('@').unwrap_or(segment);
    let mut chars = segment.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic())
        && chars.all(|character| character == '_' || character.is_alphanumeric())
}

#[cfg(test)]
#[path = "../tests/ir/csharp_direct_resolution_r2.rs"]
mod tests_r2;

#[cfg(test)]
#[path = "../tests/ir/csharp_import_resolution_r3.rs"]
mod tests_r3;
