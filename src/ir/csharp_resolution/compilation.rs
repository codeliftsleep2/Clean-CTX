//! Compilation-local assembly of neutral C# class-resolution inputs.

use super::{
    CSharpAliasContext, CSharpAliasDirective, CSharpInheritanceArc, CSharpInheritanceGraph,
    CSharpOrdinaryImportContext, CSharpResolution, CSharpResolutionEvidence, CSharpSourceSpan,
    CSharpTypeIdentity, resolve_alias_type_reference, resolve_attribute_type_reference,
    resolve_explicit_type_reference, resolve_ordinary_import_type_reference, resolve_type_ancestry,
    source_identity,
};
use crate::compression::capture_pipeline::CapEntry;
use crate::ir::csharp_symbols::{
    CSharpDeclarationPart, CSharpSourceSymbol, CSharpSourceSymbolIndex, CSharpSymbolKey,
};
use crate::ir::pipeline::CSharpDeclarationInputs;

#[path = "compilation/method_inputs.rs"]
mod method_inputs;
use method_inputs::direct_methods;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CSharpResolvedClass {
    pub owner: String,
    pub files: Vec<String>,
    pub raw_parts: Vec<(String, String)>,
    pub method_ids: Vec<(String, Vec<String>)>,
    pub methods: Vec<CSharpResolvedMethod>,
    pub attributes: Vec<CSharpResolvedAttribute>,
    pub ancestry: CSharpResolution<CSharpTypeIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CSharpResolvedAttribute {
    pub written: String,
    pub raw: String,
    pub file: String,
    pub span: CSharpSourceSpan,
    pub resolution: CSharpResolution<CSharpTypeIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CSharpResolvedMethod {
    pub name: String,
    pub file: String,
    pub attributes: Vec<CSharpResolvedAttribute>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CSharpCompilationUnit {
    classes: Vec<ClassInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassInput {
    part: CSharpDeclarationPart,
    scope: Scope,
    raw_class: String,
    method_ids: Vec<String>,
    methods: Vec<MethodInput>,
    attributes: Vec<AttributeInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AttributeInput {
    written: String,
    raw: String,
    span: CSharpSourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MethodInput {
    name: String,
    attributes: Vec<AttributeInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scope {
    namespace_path: Vec<String>,
    containing_type_path: Vec<String>,
    ordinary_imports: Vec<Vec<String>>,
    aliases: CSharpAliasContext,
}

pub(crate) fn resolve_compilation_class_ancestry(
    captures: &[CapEntry],
    file: &str,
    known_external: &[CSharpTypeIdentity],
    instructions: &[crate::ir::opcodes::CoreOp],
    declarations: &CSharpDeclarationInputs,
) -> Vec<CSharpResolvedClass> {
    let unit = collect_compilation_unit(captures, file, instructions, declarations);
    resolve_compilation_units(&[unit], known_external)
}

pub(crate) fn collect_compilation_unit(
    captures: &[CapEntry],
    file: &str,
    instructions: &[crate::ir::opcodes::CoreOp],
    declarations: &CSharpDeclarationInputs,
) -> CSharpCompilationUnit {
    CSharpCompilationUnit {
        classes: class_inputs(captures, file, instructions, declarations),
    }
}

pub(crate) fn resolve_compilation_units(
    units: &[CSharpCompilationUnit],
    known_external: &[CSharpTypeIdentity],
) -> Vec<CSharpResolvedClass> {
    let parts = units
        .iter()
        .flat_map(|unit| unit.classes.iter().map(|input| input.part.clone()));
    let index = CSharpSourceSymbolIndex::build(parts);
    let mut graph = CSharpInheritanceGraph::default();

    for symbol in index.resolved_symbols() {
        let Some(written) = symbol.base_reference.as_deref() else {
            continue;
        };
        let Some(part) = symbol.parts.iter().find(|part| {
            part.base_references
                .iter()
                .any(|base| base.trim() == written)
        }) else {
            continue;
        };
        let Some(input) = units
            .iter()
            .flat_map(|unit| &unit.classes)
            .find(|input| input.part.file == part.file && input.part.span == part.span)
        else {
            continue;
        };
        let mut resolved = resolve_direct_base(written, &input.scope, known_external, &index);
        if let CSharpResolution::Resolved { evidence, .. } = &mut resolved {
            evidence.insert(
                0,
                CSharpResolutionEvidence {
                    file: part.file.clone(),
                    span: part.span,
                },
            );
        }
        graph.arcs.push(CSharpInheritanceArc {
            source: source_identity(symbol),
            resolved_base: resolved,
        });
    }

    index
        .resolved_symbols()
        .map(|symbol| CSharpResolvedClass {
            owner: symbol.key.name.clone(),
            files: symbol.parts.iter().map(|part| part.file.clone()).collect(),
            raw_parts: symbol
                .parts
                .iter()
                .filter_map(|part| {
                    units
                        .iter()
                        .flat_map(|unit| &unit.classes)
                        .find(|input| input.part.file == part.file && input.part.span == part.span)
                        .map(|input| (part.file.clone(), input.raw_class.clone()))
                })
                .collect(),
            method_ids: symbol
                .parts
                .iter()
                .filter_map(|part| {
                    units
                        .iter()
                        .flat_map(|unit| &unit.classes)
                        .find(|input| input.part.file == part.file && input.part.span == part.span)
                        .map(|input| (part.file.clone(), input.method_ids.clone()))
                })
                .collect(),
            methods: resolved_methods(symbol, units, known_external, &index),
            attributes: resolved_attributes(symbol, units, known_external, &index),
            ancestry: resolve_type_ancestry(&source_identity(symbol), &graph),
        })
        .collect()
}

fn resolved_attributes(
    symbol: &CSharpSourceSymbol,
    units: &[CSharpCompilationUnit],
    known_external: &[CSharpTypeIdentity],
    index: &CSharpSourceSymbolIndex,
) -> Vec<CSharpResolvedAttribute> {
    let mut resolved = Vec::new();
    for part in &symbol.parts {
        let Some(input) = units
            .iter()
            .flat_map(|unit| &unit.classes)
            .find(|input| input.part.file == part.file && input.part.span == part.span)
        else {
            continue;
        };
        resolved.extend(
            input
                .attributes
                .iter()
                .map(|attribute| resolve_attribute(attribute, input, known_external, index)),
        );
    }
    resolved
}

fn resolved_methods(
    symbol: &CSharpSourceSymbol,
    units: &[CSharpCompilationUnit],
    known_external: &[CSharpTypeIdentity],
    index: &CSharpSourceSymbolIndex,
) -> Vec<CSharpResolvedMethod> {
    let mut methods = Vec::new();
    for part in &symbol.parts {
        let Some(input) = units
            .iter()
            .flat_map(|unit| &unit.classes)
            .find(|input| input.part.file == part.file && input.part.span == part.span)
        else {
            continue;
        };
        for method in &input.methods {
            methods.push(CSharpResolvedMethod {
                name: method.name.clone(),
                file: part.file.clone(),
                attributes: method
                    .attributes
                    .iter()
                    .map(|attribute| resolve_attribute(attribute, input, known_external, index))
                    .collect(),
            });
        }
    }
    methods
}

fn resolve_attribute(
    attribute: &AttributeInput,
    input: &ClassInput,
    known_external: &[CSharpTypeIdentity],
    index: &CSharpSourceSymbolIndex,
) -> CSharpResolvedAttribute {
    CSharpResolvedAttribute {
        written: attribute.written.clone(),
        raw: attribute.raw.clone(),
        file: input.part.file.clone(),
        span: attribute.span,
        resolution: with_attribute_evidence(
            resolve_attribute_type_reference(
                &attribute.written,
                &CSharpOrdinaryImportContext {
                    namespace_path: input.scope.namespace_path.clone(),
                    containing_type_path: input.scope.containing_type_path.clone(),
                    ordinary_imports: input.scope.ordinary_imports.clone(),
                },
                &input.scope.aliases,
                known_external,
                index,
            ),
            &input.part.file,
            attribute.span,
        ),
    }
}

fn with_attribute_evidence(
    mut resolution: CSharpResolution<CSharpTypeIdentity>,
    file: &str,
    span: CSharpSourceSpan,
) -> CSharpResolution<CSharpTypeIdentity> {
    if let CSharpResolution::Resolved { evidence, .. } = &mut resolution {
        evidence.insert(
            0,
            CSharpResolutionEvidence {
                file: file.to_string(),
                span,
            },
        );
    }
    resolution
}

fn resolve_direct_base(
    written: &str,
    scope: &Scope,
    known_external: &[CSharpTypeIdentity],
    index: &CSharpSourceSymbolIndex,
) -> CSharpResolution<CSharpTypeIdentity> {
    let head = written.split(['.', '<']).next().unwrap_or_default().trim();
    if scope
        .aliases
        .directives
        .iter()
        .any(|directive| directive.alias == head)
    {
        return resolve_alias_type_reference(written, &scope.aliases, known_external, index);
    }
    if written.starts_with("global::") || written.contains('.') {
        return resolve_explicit_type_reference(written, known_external, index);
    }
    resolve_ordinary_import_type_reference(
        written,
        &CSharpOrdinaryImportContext {
            namespace_path: scope.namespace_path.clone(),
            containing_type_path: scope.containing_type_path.clone(),
            ordinary_imports: scope.ordinary_imports.clone(),
        },
        known_external,
        index,
    )
}

fn class_inputs(
    captures: &[CapEntry],
    file: &str,
    instructions: &[crate::ir::opcodes::CoreOp],
    declarations: &CSharpDeclarationInputs,
) -> Vec<ClassInput> {
    let mut available_classes = instructions
        .iter()
        .filter_map(|instruction| match instruction {
            crate::ir::opcodes::CoreOp::DefClass(id, name) => Some((id.clone(), name.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    captures
        .iter()
        .filter(|capture| capture.name == "class.root")
        .filter_map(|owner| {
            let name = (!owner.text.is_empty()).then(|| owner.text.clone())?;
            let span = span(owner);
            let namespace_path = namespace_path(captures, span);
            let containing = containing_classes(captures, owner);
            let generic_arity = declarations
                .class_arities
                .get(&(owner.start_byte, owner.end_byte))
                .copied()
                .unwrap_or(0);
            let base_references = first_base(captures, owner)
                .map(|base| vec![base.raw_text.trim().to_string()])
                .unwrap_or_default();
            let part = CSharpDeclarationPart {
                key: CSharpSymbolKey {
                    namespace_path,
                    containing_type_path: containing,
                    name,
                    generic_arity,
                },
                kind: super::CSharpDeclarationKind::Class,
                is_partial: declaration_is_partial(&owner.raw_text),
                file: file.to_string(),
                span,
                base_references,
            };
            let class_position = available_classes
                .iter()
                .position(|(_, candidate)| candidate == &part.key.name);
            let method_records = class_position.map_or_else(Vec::new, |position| {
                let (class_id, _) = available_classes.remove(position);
                instructions
                    .iter()
                    .filter_map(|instruction| match instruction {
                        crate::ir::opcodes::CoreOp::DefMethod(owner, method, name)
                            if owner == &class_id =>
                        {
                            Some((method.clone(), name.clone()))
                        }
                        _ => None,
                    })
                    .collect()
            });
            let methods = direct_methods(captures, owner, declarations);
            let method_ids = method_records.into_iter().map(|(id, _)| id).collect();
            Some(ClassInput {
                scope: scope_for(captures, span, file),
                part,
                raw_class: owner.raw_text.clone(),
                method_ids,
                methods,
                attributes: direct_class_attributes(captures, owner),
            })
        })
        .collect()
}

fn direct_class_attributes(captures: &[CapEntry], owner: &CapEntry) -> Vec<AttributeInput> {
    captures
        .iter()
        .filter(|capture| capture.name == "csharp.class_attribute" && contains(owner, capture))
        .filter(|capture| {
            !captures.iter().any(|nested| {
                nested.name == "class.root"
                    && nested.start_byte > owner.start_byte
                    && nested.end_byte < owner.end_byte
                    && contains(nested, capture)
            })
        })
        .filter_map(|capture| {
            let written = capture
                .raw_text
                .trim()
                .split_once('(')
                .map_or(capture.raw_text.trim(), |(name, _)| name)
                .trim();
            (!written.is_empty()).then(|| AttributeInput {
                written: written.to_string(),
                raw: capture.raw_text.trim().to_string(),
                span: span(capture),
            })
        })
        .collect()
}

fn scope_for(captures: &[CapEntry], owner: CSharpSourceSpan, file: &str) -> Scope {
    let namespace_path = namespace_path(captures, owner);
    let containing_type_path = captures
        .iter()
        .filter(|capture| capture.name == "class.root")
        .filter(|capture| {
            capture.start_byte < owner.start_byte && owner.end_byte < capture.end_byte
        })
        .map(|capture| capture.text.clone())
        .collect();
    let applicable = captures
        .iter()
        .filter(|capture| capture.name == "import.root")
        .filter(|capture| import_applies(captures, capture, owner))
        .collect::<Vec<_>>();
    let ordinary_imports = applicable
        .iter()
        .filter_map(|capture| parse_ordinary_import(&capture.raw_text))
        .collect();
    let directives = applicable
        .iter()
        .filter_map(|capture| parse_alias(capture, captures, file))
        .collect();
    Scope {
        namespace_path,
        containing_type_path,
        ordinary_imports,
        aliases: CSharpAliasContext { directives },
    }
}

fn namespace_path(captures: &[CapEntry], owner: CSharpSourceSpan) -> Vec<String> {
    let mut roots = captures
        .iter()
        .filter(|capture| capture.name == "csharp.namespace_root")
        .filter(|capture| {
            contains_span(capture, owner)
                || (is_file_scoped_namespace(capture) && capture.start_byte <= owner.start_byte)
        })
        .collect::<Vec<_>>();
    roots.sort_by_key(|capture| capture.start_byte);
    roots
        .into_iter()
        .filter_map(|root| namespace_name(captures, root))
        .flat_map(|name| name.split('.').map(str::to_string).collect::<Vec<_>>())
        .collect()
}

fn contains_span(capture: &CapEntry, owner: CSharpSourceSpan) -> bool {
    capture.start_byte <= owner.start_byte && owner.end_byte <= capture.end_byte
}

fn is_file_scoped_namespace(capture: &CapEntry) -> bool {
    capture
        .raw_text
        .split_once(';')
        .is_some_and(|(declaration, _)| {
            declaration.trim_start().starts_with("namespace ") && !declaration.contains('{')
        })
}

fn namespace_name<'a>(captures: &'a [CapEntry], root: &CapEntry) -> Option<&'a str> {
    captures
        .iter()
        .filter(|capture| capture.name == "csharp.namespace_name" && contains(root, capture))
        .min_by_key(|capture| capture.start_byte)
        .map(|capture| capture.raw_text.trim())
}

fn containing_classes(captures: &[CapEntry], owner: &CapEntry) -> Vec<String> {
    let mut parents = captures
        .iter()
        .filter(|capture| capture.name == "class.root")
        .filter(|capture| {
            capture.start_byte < owner.start_byte && owner.end_byte < capture.end_byte
        })
        .collect::<Vec<_>>();
    parents.sort_by_key(|capture| capture.start_byte);
    parents
        .into_iter()
        .map(|capture| capture.text.clone())
        .collect()
}

fn first_base<'a>(captures: &'a [CapEntry], owner: &CapEntry) -> Option<&'a CapEntry> {
    captures
        .iter()
        .filter(|capture| capture.name == "csharp.base_type" && contains(owner, capture))
        .filter(|capture| {
            !captures.iter().any(|nested| {
                nested.name == "class.root"
                    && nested.start_byte > owner.start_byte
                    && nested.end_byte < owner.end_byte
                    && contains(nested, capture)
            })
        })
        .min_by_key(|capture| capture.start_byte)
}

fn import_applies(captures: &[CapEntry], import: &CapEntry, owner: CSharpSourceSpan) -> bool {
    let enclosing = captures
        .iter()
        .filter(|capture| capture.name == "csharp.namespace_root" && contains(capture, import))
        .min_by_key(|capture| capture.end_byte - capture.start_byte);
    enclosing.is_none_or(|scope| {
        scope.start_byte <= owner.start_byte && owner.end_byte <= scope.end_byte
    })
}

fn parse_ordinary_import(written: &str) -> Option<Vec<String>> {
    let written = written.trim();
    let rest = written
        .strip_prefix("global ")
        .unwrap_or(written)
        .strip_prefix("using ")?;
    let target = rest.trim_end_matches(';').trim();
    (!target.contains('=') && !target.starts_with("static "))
        .then(|| target.split('.').map(str::to_string).collect())
}

fn parse_alias(
    capture: &CapEntry,
    captures: &[CapEntry],
    file: &str,
) -> Option<CSharpAliasDirective> {
    let written = capture.raw_text.trim();
    let rest = written
        .strip_prefix("global ")
        .unwrap_or(written)
        .strip_prefix("using ")?
        .trim_end_matches(';');
    let (alias, target) = rest.split_once('=')?;
    Some(CSharpAliasDirective {
        alias: alias.trim().to_string(),
        target: target.trim().to_string(),
        scope_depth: captures
            .iter()
            .filter(|scope| scope.name == "csharp.namespace_root" && contains(scope, capture))
            .count(),
        file: file.to_string(),
        span: span(capture),
    })
}

fn declaration_is_partial(raw: &str) -> bool {
    raw.split_once("class")
        .is_some_and(|(prefix, _)| prefix.split_whitespace().any(|word| word == "partial"))
}

fn contains(owner: &CapEntry, nested: &CapEntry) -> bool {
    owner.start_byte <= nested.start_byte && nested.end_byte <= owner.end_byte
}

fn span(capture: &CapEntry) -> CSharpSourceSpan {
    CSharpSourceSpan {
        start_byte: capture.start_byte,
        end_byte: capture.end_byte,
    }
}
