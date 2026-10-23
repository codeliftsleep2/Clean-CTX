//! Admission for existing signal-member projections using retained authority.
//! This does not claim canonical member identity, aliases or inherited closure.
use super::{class_role_authority as roles, decorators::ClassKind};
use crate::ir::invocation_evidence::{InvocationOccurrence, SourceSpan, WrittenCallee};
use crate::ir::typescript_resolution::TypeScriptAuthority;

pub(super) fn api<'a>(
    authority: &'a TypeScriptAuthority,
    source: &str,
    call: &InvocationOccurrence,
) -> Option<&'a str> {
    let (span, required) = match call.callee() {
        WrittenCallee::Free { terminal } => (*terminal, false),
        WrittenCallee::Member { receiver, terminal } => {
            if source.get(terminal.start_byte()..terminal.end_byte())? == "required" {
                (*receiver, true)
            } else {
                (
                    SourceSpan::new(receiver.start_byte(), terminal.end_byte()).ok()?,
                    false,
                )
            }
        }
        _ => return None,
    };
    let id = authority.value_reference_at(span)?;
    if !matches!(id.name.as_str(), "input" | "output" | "model")
        || (required && id.name != "input")
        || !roles::public_api(authority, id, "@angular/core", &id.name)
    {
        return None;
    }
    Some(&id.name)
}

pub(super) fn owner(
    authority: &TypeScriptAuthority,
    source: &str,
    class: tree_sitter::Node<'_>,
) -> Option<&'static str> {
    let raw = source.get(class.start_byte()..class.end_byte())?;
    let role = roles::class_role(source, raw, class.start_byte(), authority)?;
    match role.kind {
        ClassKind::Component => Some("Component"),
        ClassKind::Directive => Some("Directive"),
        _ => None,
    }
}
