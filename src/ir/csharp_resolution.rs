//! Framework-neutral C# type-reference resolution contract.
//!
//! R0 introduces the value boundary only. Later campaign slices populate the
//! admitted-source universe, imports, aliases, partial symbols, and ancestry.

use crate::compression::capture_pipeline::CapEntry;

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
            generic_arity: 0,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CSharpResolutionReason {
    MissingAuthority,
    UnsupportedSimpleName,
    UnsupportedAliasQualifiedName,
    MalformedTypeReference,
    CompetingCandidates,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CSharpResolution<T> {
    Resolved {
        identity: T,
        basis: CSharpResolutionBasis,
    },
    Ambiguous(CSharpResolutionReason),
    Unsupported(CSharpResolutionReason),
    Unresolved(CSharpResolutionReason),
}

/// Resolve one written C# type reference against caller-supplied neutral
/// external identities.
pub fn resolve_known_external_type(
    written: &str,
    known_external: &[CSharpTypeIdentity],
) -> CSharpResolution<CSharpTypeIdentity> {
    let written = written.trim();
    if written.is_empty() {
        return CSharpResolution::Unsupported(CSharpResolutionReason::MalformedTypeReference);
    }

    let (qualified_name, basis) = if let Some(qualified) = written.strip_prefix("global::") {
        (qualified, CSharpResolutionBasis::GlobalQualified)
    } else if written.contains("::") {
        return CSharpResolution::Unsupported(
            CSharpResolutionReason::UnsupportedAliasQualifiedName,
        );
    } else if !written.contains('.') {
        return CSharpResolution::Unsupported(CSharpResolutionReason::UnsupportedSimpleName);
    } else {
        (written, CSharpResolutionBasis::ExplicitQualified)
    };

    if !is_qualified_identifier(qualified_name) {
        return CSharpResolution::Unsupported(CSharpResolutionReason::MalformedTypeReference);
    }

    let mut candidates = known_external
        .iter()
        .filter(|identity| identity.qualified_name == qualified_name);
    let Some(identity) = candidates.next() else {
        return CSharpResolution::Unresolved(CSharpResolutionReason::MissingAuthority);
    };
    if candidates.next().is_some() {
        return CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates);
    }

    CSharpResolution::Resolved {
        identity: identity.clone(),
        basis,
    }
}

fn is_qualified_identifier(name: &str) -> bool {
    let mut segments = name.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    is_identifier(first) && segments.all(is_identifier) && name.contains('.')
}

fn is_identifier(segment: &str) -> bool {
    let segment = segment.strip_prefix('@').unwrap_or(segment);
    let mut chars = segment.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic())
        && chars.all(|character| character == '_' || character.is_alphanumeric())
}
