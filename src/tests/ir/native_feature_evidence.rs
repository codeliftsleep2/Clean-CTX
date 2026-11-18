//! Native TypeScript proof ownership remains valid without the Angular meta layer.
use crate::compression::Fidelity;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};
use crate::layers::meta::semantic::SemanticRelation;

fn native(source: &str) -> PassContext {
    let mut context = PassContext::new(source.into(), "alias".into(), Fidelity::High);
    context.canonical_path = Some("native.ts".into());
    context.language = crate::compression::language::safe_typescript_language();
    context.query_string = crate::queries::TS_QUERY.into();
    CoreIRPass::new().run(&mut context).unwrap();
    context
}

#[test]
fn ci_runtime_native_typescript_evidence_preserves_source_ownership() {
    let discriminant = "import {Action} from '@ngrx/store'; export const SET_AUDIT = '[Audit] Set'; export function auditReducer(state:any, action:Action) {switch(action.type) {case SET_AUDIT: return state; default: return state;}}";
    let context = native(discriminant);
    assert_eq!(context.ngrx_legacy_discriminants.len(), 1);
    let proof = &context.ngrx_legacy_discriminants[0];
    assert!(proof.matches_source("native.ts", discriminant));
    assert!(!proof.matches_source("other.ts", discriminant));
    assert!(!proof.matches_source(
        "native.ts",
        &discriminant.replace("'[Audit] Set'", "'[Audit] Changed'")
    ));
    let declaration = proof.declaration_edge();
    assert_eq!(declaration.subject.name, "SET_AUDIT");
    assert_eq!(declaration.subject.file.as_deref(), Some("native.ts"));
    assert_eq!(declaration.relation, SemanticRelation::Defines);
    let transition = proof.transition_edge();
    assert_eq!(transition.subject, declaration.subject);
    assert_eq!(transition.object.name, "auditReducer");
    assert_eq!(transition.relation, SemanticRelation::TriggersReducer);

    // Existing public service-dispatch primary source, exercised at the native producer boundary.
    let service = "import {Injectable} from '@angular/core'; import {Action, Store} from '@ngrx/store'; export class AuditAction implements Action {readonly type='audit';} @Injectable() export class AuditService {constructor(private store: Store<unknown>) {} run() {this.store.dispatch(new AuditAction());}}";
    let context = native(service);
    assert_eq!(context.ngrx_service_dispatches.len(), 1);
    let proof = &context.ngrx_service_dispatches[0];
    assert_eq!(proof.service, "AuditService");
    assert_eq!(proof.action, "AuditAction");
    assert_eq!(proof.file, "native.ts");
    assert_eq!(
        proof.source_hash,
        crate::cache::content_hash(service.as_bytes())
    );
    let invocation = context
        .invocation_occurrences
        .iter()
        .find(|call| call.span().start_byte() == proof.start_byte)
        .unwrap();
    assert_eq!(
        invocation.declaration_context().unwrap().declaration(),
        proof.method_span
    );
    let owner = context
        .angular_initializers
        .iter()
        .find(|class| class.consumer == proof.service)
        .unwrap();
    assert_eq!(owner.class_span, proof.class_span);
}
