//! C#-scoped exact method declaration and call projection.
//!
//! Generic CoreOp::Call remains written-call evidence. This projection adds a
//! distinct exact relation only when canonical declarations plus structured
//! invocation evidence prove one owner/signature target.

use std::collections::{HashMap, HashSet};

use crate::ir::invocation_evidence::{DeclarationContextKind, InvocationOccurrence, WrittenCallee};
use crate::ir::opcodes::{CoreOp, DeclarationModifier};
use crate::layers::meta::semantic::{CallEvidence, EntityRef, SemanticEdge, SemanticRelation};

pub(crate) const CSHARP_DOMAIN: &str = "csharp";
const METHOD: &str = "Method";
const LAYER: &str = "csharp-exact-call";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Declaration {
    id: String,
    owner: String,
    name: String,
    parameters: Vec<String>,
    return_type: String,
    is_static: bool,
}

impl Declaration {
    fn identity(&self) -> String {
        format!(
            "{}::{}({})->{}",
            self.owner,
            self.name,
            self.parameters.join(","),
            self.return_type
        )
    }
}

pub(crate) fn project(
    source: &str,
    instructions: &[CoreOp],
    occurrences: &[InvocationOccurrence],
    file: &str,
) -> Vec<SemanticEdge> {
    let declarations = declaration_authority(instructions);
    project_with_authority(source, instructions, occurrences, file, &declarations)
}

pub(crate) fn project_with_authority(
    source: &str,
    instructions: &[CoreOp],
    occurrences: &[InvocationOccurrence],
    file: &str,
    authority: &[Declaration],
) -> Vec<SemanticEdge> {
    let local_declarations = declaration_authority(instructions);
    let mut edges: Vec<_> = local_declarations
        .iter()
        .map(|declaration| registration(declaration, file))
        .collect();
    let callers: HashMap<_, _> = local_declarations
        .iter()
        .map(|declaration| (declaration.id.as_str(), declaration))
        .collect();
    let mut occurrence_index = 0;

    for op in instructions {
        let CoreOp::Call(caller_id, written, argument_count, has_spread) = op else {
            continue;
        };
        let Some(caller) = callers.get(caller_id.as_str()).copied() else {
            continue;
        };
        let Some((occurrence, next)) = matching_occurrence(
            source,
            occurrences,
            occurrence_index,
            written,
            *argument_count,
        ) else {
            continue;
        };
        occurrence_index = next;
        let Some(target) = resolve(
            source,
            occurrence,
            caller,
            authority,
            *argument_count,
            *has_spread,
        ) else {
            continue;
        };
        edges.push(SemanticEdge {
            relation: SemanticRelation::Calls,
            subject: entity(caller, file),
            object: entity(target, file),
            layer: LAYER,
            call_evidence: Some(CallEvidence::new(*argument_count, *has_spread)),
        });
    }
    for occurrence in occurrences {
        let Some(caller) = property_caller(source, occurrence, instructions) else {
            continue;
        };
        let Some(target) = resolve(
            source,
            occurrence,
            &caller,
            authority,
            occurrence.arguments().len(),
            false,
        ) else {
            continue;
        };
        edges.push(SemanticEdge {
            relation: SemanticRelation::Calls,
            subject: entity(&caller, file),
            object: entity(target, file),
            layer: LAYER,
            call_evidence: Some(CallEvidence::new(occurrence.arguments().len(), false)),
        });
    }
    edges
}

pub(crate) fn declaration_authority(instructions: &[CoreOp]) -> Vec<Declaration> {
    let owners: HashMap<_, _> = instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefClass(id, name) | CoreOp::DefInterface(id, name) => {
                Some((id.as_str(), name.as_str()))
            }
            _ => None,
        })
        .collect();
    let mut declarations: Vec<_> = instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefMethod(owner, id, name) | CoreOp::DefInterfaceMethod(owner, id, name) => {
                Some(Declaration {
                    id: id.clone(),
                    owner: owners.get(owner.as_str())?.to_string(),
                    name: name.clone(),
                    parameters: Vec::new(),
                    return_type: "void".to_string(),
                    is_static: false,
                })
            }
            _ => None,
        })
        .collect();
    let positions: HashMap<_, _> = declarations
        .iter()
        .enumerate()
        .map(|(index, declaration)| (declaration.id.clone(), index))
        .collect();
    for op in instructions {
        match op {
            CoreOp::Param(method, _, parameter_type, _) => {
                if let Some(index) = positions.get(method) {
                    declarations[*index].parameters.push(parameter_type.clone());
                }
            }
            CoreOp::Return(method, return_type) => {
                if let Some(index) = positions.get(method) {
                    declarations[*index].return_type = display_type(return_type).to_string();
                }
            }
            CoreOp::MethodModifiers(method, modifiers) => {
                if let Some(index) = positions.get(method) {
                    declarations[*index].is_static =
                        modifiers.contains(&DeclarationModifier::Static);
                }
            }
            _ => {}
        }
    }
    declarations
}

fn display_type(value: &str) -> &str {
    match value {
        crate::ir::opcodes::TYPE_VOID => "void",
        crate::ir::opcodes::TYPE_STRING => "string",
        crate::ir::opcodes::TYPE_BOOLEAN => "boolean",
        crate::ir::opcodes::TYPE_NUMBER => "number",
        _ => value,
    }
}

fn property_caller(
    source: &str,
    occurrence: &InvocationOccurrence,
    instructions: &[CoreOp],
) -> Option<Declaration> {
    let context = occurrence.declaration_context()?;
    if context.kind() != DeclarationContextKind::PropertyInitializer {
        return None;
    }
    let declaration_source = slice(source, context.declaration())?;
    if !declaration_source.contains("=>") {
        return None;
    }
    let name = slice(source, context.written_name()?)?;
    let owners: HashMap<_, _> = instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefClass(id, owner) | CoreOp::DefInterface(id, owner) => {
                Some((id.as_str(), owner.as_str()))
            }
            _ => None,
        })
        .collect();
    let mut candidates = instructions.iter().filter_map(|op| match op {
        CoreOp::DefField(owner, _, field) | CoreOp::DefInterfaceField(owner, _, field)
            if field == name =>
        {
            Some(owners.get(owner.as_str())?.to_string())
        }
        _ => None,
    });
    let owner = candidates.next()?;
    if candidates.next().is_some() {
        return None;
    }
    let head = &declaration_source[..declaration_source.find(name)?];
    let return_type = head.split_whitespace().last()?.to_string();
    let is_static = head.split_whitespace().any(|token| token == "static");
    Some(Declaration {
        id: format!("property:{owner}:{name}"),
        owner,
        name: name.to_string(),
        parameters: Vec::new(),
        return_type,
        is_static,
    })
}

fn registration(declaration: &Declaration, file: &str) -> SemanticEdge {
    let declaration = entity(declaration, file);
    SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: declaration.clone(),
        object: declaration,
        layer: LAYER,
        call_evidence: None,
    }
}

fn entity(declaration: &Declaration, file: &str) -> EntityRef {
    EntityRef::new(CSHARP_DOMAIN, METHOD, declaration.identity()).with_file(file.to_string())
}

fn matching_occurrence<'a>(
    source: &str,
    occurrences: &'a [InvocationOccurrence],
    start: usize,
    written: &str,
    argument_count: usize,
) -> Option<(&'a InvocationOccurrence, usize)> {
    occurrences
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, occurrence)| {
            let terminal = match occurrence.callee() {
                WrittenCallee::Free { terminal } | WrittenCallee::Member { terminal, .. } => {
                    slice(source, *terminal)?
                }
                WrittenCallee::Unsupported { .. } => return None,
            };
            (terminal == written && occurrence.arguments().len() == argument_count)
                .then_some((occurrence, index + 1))
        })
}

fn resolve<'a>(
    source: &str,
    occurrence: &InvocationOccurrence,
    caller: &Declaration,
    declarations: &'a [Declaration],
    argument_count: usize,
    has_spread: bool,
) -> Option<&'a Declaration> {
    if has_spread {
        return None;
    }
    let (written, receiver_type) = match occurrence.callee() {
        WrittenCallee::Free { terminal } => (slice(source, *terminal)?, caller.owner.clone()),
        WrittenCallee::Member { receiver, terminal } => {
            let receiver = slice(source, *receiver)?.trim();
            let owner = receiver_owner(source, receiver, occurrence.span().start_byte(), caller)?;
            (slice(source, *terminal)?, owner)
        }
        WrittenCallee::Unsupported { .. } => return None,
    };

    let direct: Vec<_> = declarations
        .iter()
        .filter(|candidate| {
            candidate.owner == receiver_type
                && candidate.name == written
                && candidate.parameters.len() == argument_count
        })
        .collect();
    if direct.len() == 1 {
        return direct.into_iter().next();
    }
    if !direct.is_empty() {
        return None;
    }

    let extensions: Vec<_> = declarations
        .iter()
        .filter(|candidate| {
            candidate.is_static
                && candidate.name == written
                && candidate.parameters.len() == argument_count + 1
                && candidate
                    .parameters
                    .first()
                    .is_some_and(|first| normalize_type(first) == normalize_type(&receiver_type))
        })
        .collect();
    (extensions.len() == 1)
        .then(|| extensions.into_iter().next())
        .flatten()
}

fn receiver_owner(
    source: &str,
    receiver: &str,
    call_start: usize,
    caller: &Declaration,
) -> Option<String> {
    if receiver == "this" || receiver == "base" {
        return Some(caller.owner.clone());
    }
    if let Some(owner) = constructed_type(receiver) {
        return Some(owner);
    }
    let terminal = receiver.rsplit('.').next()?.trim();
    if terminal.is_empty() {
        return None;
    }
    if terminal.chars().next().is_some_and(char::is_uppercase) {
        return Some(strip_generic_arity(terminal));
    }
    declared_identifier_type(&source[..call_start.min(source.len())], terminal)
}

fn constructed_type(receiver: &str) -> Option<String> {
    let rest = receiver.strip_prefix("new ")?.trim_start();
    let terminal = rest
        .split(|character: char| character == '(' || character.is_whitespace())
        .next()?;
    (!terminal.is_empty())
        .then(|| strip_generic_arity(terminal.rsplit('.').next().unwrap_or(terminal)))
}

fn declared_identifier_type(prefix: &str, identifier: &str) -> Option<String> {
    let mut matches = HashSet::new();
    for separator in [';', '{', '}', '(', ',', '\n', '\r'] {
        for fragment in prefix.rsplit(separator).take(96) {
            let words: Vec<_> = fragment
                .split(|character: char| {
                    character.is_whitespace()
                        || matches!(character, '=' | '(' | ')' | '{' | '}' | ';' | ',')
                })
                .filter(|word| !word.is_empty())
                .collect();
            for pair in words.windows(2) {
                if pair[1].trim_matches(|c: char| !c.is_alphanumeric() && c != '_') == identifier {
                    let candidate = pair[0].trim_matches(|c: char| {
                        !c.is_alphanumeric() && c != '_' && c != '.' && c != '<' && c != '>'
                    });
                    if !candidate.is_empty() && candidate != "var" {
                        matches.insert(strip_generic_arity(
                            candidate.rsplit('.').next().unwrap_or(candidate),
                        ));
                    }
                }
            }
        }
    }
    (matches.len() == 1)
        .then(|| matches.into_iter().next())
        .flatten()
}

fn strip_generic_arity(value: &str) -> String {
    value.split('<').next().unwrap_or(value).trim().to_string()
}

fn normalize_type(value: &str) -> &str {
    value.trim().trim_start_matches("this ")
}

fn slice(source: &str, span: crate::ir::invocation_evidence::SourceSpan) -> Option<&str> {
    source.get(span.start_byte()..span.end_byte())
}

#[cfg(test)]
#[path = "../tests/ir/csharp_exact_calls.rs"]
mod tests;
