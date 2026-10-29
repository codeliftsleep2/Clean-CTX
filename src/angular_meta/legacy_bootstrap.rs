//! Written native Module argument association; no execution or provider inheritance.
use crate::ir::invocation_evidence::SourceSpan;
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::ir::typescript_resolution::{Namespace, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::{CanonicalEndpoint, InvocationOccurrence};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(crate) fn project(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    authority
        .declared_return_members
        .iter()
        .filter_map(|candidate| {
            let proof = authority.declared_return_member_at(
                SourceSpan::new(candidate.access.0, candidate.access.1).ok()?,
            )?;
            if !super::class_role_authority::public_entry_api(
                authority,
                &proof.factory,
                "@angular/platform-browser-dynamic",
                "@angular/platform-browser-dynamic",
                "platformBrowserDynamic",
                Namespace::Value,
            ) || !super::class_role_authority::public_entry_api(
                authority,
                &proof.receiver,
                "@angular/core",
                "@angular/core",
                "PlatformRef",
                Namespace::Type,
            ) || proof.method.name != "bootstrapModule"
            {
                return None;
            }
            let facet = authority.invocation_facets.iter().find(|f| {
                (f.expression.start, f.expression.end) == proof.outer
                    && f.source == proof.source
                    && f.source_hash == proof.source_hash
            })?;
            let ExpressionValue::Call(_, args) = &facet.expression.value else {
                return None;
            };
            let id = super::bootstrap_references::target(authority, args, 0)?;
            if !super::bootstrap_references::native_class(
                authority,
                id,
                super::decorators::ClassKind::Module,
            ) {
                return None;
            }
            let mut subject = EntityRef::new("angular", "ApplicationBootstrap", "bootstrapModule")
                .with_file(proof.source.to_string_lossy().into_owned());
            subject.canonical = Some(CanonicalEndpoint::InvocationOccurrence {
                invocation_occurrence: InvocationOccurrence {
                    source: proof.source.clone(),
                    source_hash: proof.source_hash.clone(),
                    start: proof.outer.0,
                    end: proof.outer.1,
                },
            });
            Some(SemanticEdge {
                subject,
                object: super::string_token_authority::owner(id, "Module"),
                relation: SemanticRelation::DeclaresBootstrapModule,
                layer: "angular",
                call_evidence: None,
            })
        })
        .collect()
}
