//! Structural assembly from one existing capture walk. No source scanning,
//! parsing, resolution, canonical emission, or framework interpretation.

use super::{
    ArgumentOccurrence, ArgumentQualifier, DeclarationContext, DeclarationContextKind,
    InvocationOccurrence, InvocationParent, ParentSite, SourceSpan, WrittenCallee,
};
use crate::compression::capture_pipeline::CapturedNode;
use std::collections::BTreeMap;

#[derive(Default)]
struct Details {
    function: Option<SourceSpan>,
    receiver: Option<SourceSpan>,
    terminal: Option<SourceSpan>,
    arguments: BTreeMap<SourceSpan, ArgumentQualifier>,
    type_arguments: BTreeMap<SourceSpan, ()>,
    argument_names: BTreeMap<SourceSpan, SourceSpan>,
}

fn span(capture: &CapturedNode) -> Option<SourceSpan> {
    SourceSpan::new(capture.start_byte, capture.end_byte).ok()
}

pub(crate) fn collect_invocations(captures: &[CapturedNode]) -> Vec<InvocationOccurrence> {
    // Root and detail patterns have separate matches; first join each match
    // structurally, then merge duplicates by the invocation's exact span.
    let mut roots = BTreeMap::new();
    let mut declarations = BTreeMap::new();
    let mut declaration_names = BTreeMap::new();
    for capture in captures {
        let Some(span) = span(capture) else { continue };
        let kind = match capture.name.as_str() {
            "invoke.root" => {
                roots.insert(capture.match_index, span);
                continue;
            }
            "invoke.declaration_name" => {
                declaration_names.insert(capture.match_index, span);
                continue;
            }
            "invoke.declaration.callable" => DeclarationContextKind::Callable,
            "invoke.declaration.variable" => DeclarationContextKind::VariableInitializer,
            "invoke.declaration.field" => DeclarationContextKind::FieldInitializer,
            "invoke.declaration.property" => DeclarationContextKind::PropertyInitializer,
            _ => continue,
        };
        declarations.insert(capture.match_index, (span, kind));
    }

    let mut details: BTreeMap<SourceSpan, Details> = roots
        .values()
        .map(|root| (*root, Details::default()))
        .collect();
    let argument_spans: BTreeMap<_, _> = captures
        .iter()
        .filter(|capture| capture.name == "invoke.argument")
        .filter_map(|capture| span(capture).map(|span| (capture.match_index, span)))
        .collect();
    for capture in captures {
        let Some(root) = roots.get(&capture.match_index) else {
            continue;
        };
        let Some(span) = span(capture).filter(|span| root.contains(*span)) else {
            continue;
        };
        let detail = details
            .get_mut(root)
            .expect("every captured root has a builder");
        match capture.name.as_str() {
            "invoke.function" => detail.function = Some(span),
            "invoke.receiver" => detail.receiver = Some(span),
            "invoke.terminal" => detail.terminal = Some(span),
            "invoke.argument" => {
                detail
                    .arguments
                    .entry(span)
                    .or_insert(ArgumentQualifier::Plain);
            }
            "invoke.spread" => {
                detail.arguments.insert(span, ArgumentQualifier::Spread);
            }
            "invoke.type_argument" => {
                detail.type_arguments.insert(span, ());
            }
            "invoke.argument_name" => {
                if let Some(argument) = argument_spans.get(&capture.match_index)
                    && argument.contains(span)
                {
                    detail.argument_names.insert(*argument, span);
                }
            }
            _ => {}
        }
    }

    let mut contexts: BTreeMap<SourceSpan, DeclarationContext> = BTreeMap::new();
    for (match_index, (span, kind)) in declarations {
        let name = declaration_names.get(&match_index).copied();
        let context = contexts
            .entry(span)
            .or_insert_with(|| DeclarationContext::new(kind, span, name));
        if name.is_some() {
            context.written_name = name;
        }
    }
    let mut contexts = contexts.into_values().collect::<Vec<_>>();
    contexts.sort_by_key(|context| {
        (
            context.declaration.start_byte,
            std::cmp::Reverse(context.declaration.end_byte),
        )
    });

    let mut occurrences = details
        .into_iter()
        .map(|(span, mut detail)| {
            for (argument, name) in detail.argument_names {
                detail
                    .arguments
                    .insert(argument, ArgumentQualifier::Named { name });
            }
            let callee = match (detail.receiver, detail.terminal) {
                (Some(receiver), Some(terminal)) => WrittenCallee::Member { receiver, terminal },
                (None, Some(terminal)) => WrittenCallee::Free { terminal },
                _ => WrittenCallee::Unsupported {
                    function: detail.function,
                },
            };
            InvocationOccurrence::new(
                span,
                callee,
                detail
                    .arguments
                    .into_iter()
                    .map(|(span, qualifier)| ArgumentOccurrence::new(span, qualifier))
                    .collect(),
                detail.type_arguments.into_keys().collect(),
                None,
                None,
            )
        })
        .collect::<Vec<_>>();
    occurrences.sort_by_key(|call| (call.span.start_byte, std::cmp::Reverse(call.span.end_byte)));
    assign_containment(&mut occurrences, &contexts);
    occurrences
}

fn parent_site(parent: &InvocationOccurrence, child: SourceSpan) -> Option<ParentSite> {
    if let WrittenCallee::Member { receiver, .. } = parent.callee
        && receiver.contains(child)
    {
        return Some(ParentSite::Receiver);
    }
    // Written argument spans are ordered and disjoint. Binary search avoids
    // rescanning a large argument list for every nested invocation.
    let index = parent
        .arguments
        .partition_point(|arg| arg.span.start_byte <= child.start_byte)
        .checked_sub(1)?;
    parent.arguments[index]
        .span
        .contains(child)
        .then_some(ParentSite::Argument { index })
}

fn assign_containment(occurrences: &mut [InvocationOccurrence], contexts: &[DeclarationContext]) {
    let mut invocation_stack: Vec<usize> = Vec::new();
    let mut declaration_stack: Vec<usize> = Vec::new();
    let mut next_context = 0;
    for index in 0..occurrences.len() {
        let span = occurrences[index].span;
        while invocation_stack
            .last()
            .is_some_and(|&parent| !occurrences[parent].span.contains(span))
        {
            invocation_stack.pop();
        }
        occurrences[index].parent = invocation_stack.last().and_then(|&parent| {
            parent_site(&occurrences[parent], span)
                .map(|site| InvocationParent::new(occurrences[parent].span, site))
        });
        invocation_stack.push(index);

        while next_context < contexts.len()
            && contexts[next_context].declaration.start_byte <= span.start_byte
        {
            let declaration = contexts[next_context].declaration;
            while declaration_stack
                .last()
                .is_some_and(|&outer| !contexts[outer].declaration.contains(declaration))
            {
                declaration_stack.pop();
            }
            declaration_stack.push(next_context);
            next_context += 1;
        }
        while declaration_stack
            .last()
            .is_some_and(|&owner| !contexts[owner].declaration.contains(span))
        {
            declaration_stack.pop();
        }
        occurrences[index].declaration_context = declaration_stack
            .last()
            .map(|&owner| contexts[owner].clone());
    }
}
