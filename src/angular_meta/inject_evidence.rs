//! Compilation-local associations from one existing TypeScript capture walk.
use crate::compression::capture_pipeline::CapturedNode;
use crate::ir::invocation_evidence::{
    ArgumentQualifier, InvocationOccurrence, SourceSpan, WrittenCallee,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InitializerClassification {
    DirectSupported,
    NotDirect,
    NotAngularInject,
    UnsupportedToken,
    UnsupportedCall,
}

#[cfg(test)]
#[path = "../tests/ir/angular_inject_evidence.rs"]
mod angular_inject_evidence_tests;

#[derive(Debug, Clone)]
pub(crate) struct InjectInitializerEvidence {
    pub receiver: String,
    pub field_span: SourceSpan,
    pub initializer_span: SourceSpan,
    pub invocation_span: Option<SourceSpan>,
    /// Syntactic qualifier, independent of whether a callee spells inject.
    #[cfg(feature = "typescript")]
    pub optional_call: bool,
    pub dependency_argument: Option<SourceSpan>,
    /// Exact written expression; no module, namespace or runtime resolution.
    pub written_token: Option<String>,
    pub classification: InitializerClassification,
}

#[derive(Debug, Clone)]
pub(crate) struct ClassInitializerEvidence {
    pub consumer: String,
    pub class_span: SourceSpan,
    pub raw_class: String,
    pub fields: Vec<InjectInitializerEvidence>,
}

impl ClassInitializerEvidence {
    /// Preserve the existing functional framework-kind admission. Pipe's
    /// constructor-only semantic authority remains the frozen #138a path.
    pub(crate) fn admitted_tokens(&self) -> Vec<String> {
        use crate::angular_meta::decorators::ClassKind;
        match crate::angular_meta::decorators::extract_graph_entries(&self.raw_class) {
            Some((
                _,
                ClassKind::Component | ClassKind::Service | ClassKind::Directive,
                _,
                _,
                _,
            )) => self.tokens(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn tokens(&self) -> Vec<String> {
        self.fields
            .iter()
            .filter(|field| {
                field.classification == InitializerClassification::DirectSupported
                    && !field.receiver.is_empty()
                    && self.class_span.start_byte() <= field.field_span.start_byte()
                    && field.field_span.end_byte() <= self.class_span.end_byte()
                    && field.field_span.start_byte() <= field.initializer_span.start_byte()
                    && field.initializer_span.end_byte() <= field.field_span.end_byte()
                    && field.invocation_span == Some(field.initializer_span)
                    && field.dependency_argument.is_some_and(|argument| {
                        field.initializer_span.start_byte() <= argument.start_byte()
                            && argument.end_byte() <= field.initializer_span.end_byte()
                    })
            })
            .filter_map(|field| field.written_token.clone())
            .collect()
    }
}

fn span(capture: &CapturedNode) -> SourceSpan {
    SourceSpan::new(capture.start_byte, capture.end_byte).expect("parser span is ordered")
}

/// Owner association is a same-match direct class-body field, joined to the
/// canonical #139 class capture by exact span. No nearest-owner inference.
pub(crate) fn collect(
    source: &str,
    captures: &[CapturedNode],
    invocations: &[InvocationOccurrence],
) -> Vec<ClassInitializerEvidence> {
    let mut classes: BTreeMap<_, _> = captures
        .iter()
        .filter(|capture| capture.name == "class.root")
        .map(|capture| {
            (
                (capture.start_byte, capture.end_byte),
                ClassInitializerEvidence {
                    consumer: capture.text.clone(),
                    class_span: span(capture),
                    raw_class: crate::meta_util::class_source_from_capture(
                        source,
                        &capture.to_entry(),
                    )
                    .into(),
                    fields: Vec::new(),
                },
            )
        })
        .collect();
    let mut matches: BTreeMap<usize, BTreeMap<&str, &CapturedNode>> = BTreeMap::new();
    for capture in captures
        .iter()
        .filter(|c| c.name.starts_with("angular.initializer."))
    {
        matches
            .entry(capture.match_index)
            .or_default()
            .insert(&capture.name, capture);
    }
    let calls: BTreeMap<_, _> = invocations
        .iter()
        .map(|call| ((call.span().start_byte(), call.span().end_byte()), call))
        .collect();
    let optional: BTreeSet<_> = captures
        .iter()
        .filter(|c| c.name == "angular.initializer.optional_call")
        .map(span)
        .collect();
    let identifiers: BTreeSet<_> = captures
        .iter()
        .filter(|c| c.name == "angular.initializer.identifier_argument")
        .map(span)
        .collect();
    for fields in matches.values() {
        let (Some(owner), Some(field), Some(receiver), Some(value)) = (
            fields.get("angular.initializer.owner"),
            fields.get("angular.initializer.field"),
            fields.get("angular.initializer.receiver"),
            fields.get("angular.initializer.value"),
        ) else {
            continue;
        };
        let Some(class) = classes.get_mut(&(owner.start_byte, owner.end_byte)) else {
            continue;
        };
        let initializer = span(value);
        let call = calls.get(&(value.start_byte, value.end_byte)).copied();
        let is_inject = |call: &InvocationOccurrence| {
            matches!(call.callee(), WrittenCallee::Free { terminal }
            if source.get(terminal.start_byte()..terminal.end_byte()) == Some("inject"))
        };
        let descendant = calls
            .range((value.start_byte, 0)..=(value.end_byte, usize::MAX))
            .any(|(_, call)| is_inject(call) && call.span().end_byte() <= value.end_byte);
        let classification = match call {
            Some(call) if is_inject(call) && optional.contains(&call.span()) => {
                InitializerClassification::UnsupportedCall
            }
            Some(call) if is_inject(call) && !optional.contains(&call.span()) => {
                if call.arguments().first().is_some_and(|argument| {
                    argument.qualifier() == &ArgumentQualifier::Plain
                        && identifiers.contains(&argument.span())
                }) {
                    InitializerClassification::DirectSupported
                } else {
                    InitializerClassification::UnsupportedToken
                }
            }
            _ if descendant => InitializerClassification::NotDirect,
            _ => InitializerClassification::NotAngularInject,
        };
        let argument = call
            .filter(|call| is_inject(call))
            .and_then(|call| call.arguments().first());
        class.fields.push(InjectInitializerEvidence {
            receiver: receiver.raw_text.clone(),
            field_span: span(field),
            initializer_span: initializer,
            invocation_span: call.map(InvocationOccurrence::span),
            #[cfg(feature = "typescript")]
            optional_call: call.is_some_and(|call| optional.contains(&call.span())),
            dependency_argument: argument.map(|argument| argument.span()),
            written_token: argument
                .and_then(|argument| {
                    source.get(argument.span().start_byte()..argument.span().end_byte())
                })
                .map(str::to_owned),
            classification,
        });
    }
    for class in classes.values_mut() {
        class
            .fields
            .sort_by_key(|field| field.field_span.start_byte());
    }
    classes.into_values().collect()
}
