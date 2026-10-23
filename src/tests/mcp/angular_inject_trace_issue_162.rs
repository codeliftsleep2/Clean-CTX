use crate::compression::Fidelity;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};
use serde_json::json;

#[test]
fn issue_162_inject_trace_unaliased_production_boundary() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"), "export declare function Injectable(metadata?: unknown): any; export declare function inject(token: unknown): unknown;").unwrap();
    std::fs::write(p.join("token.ts"), "export class Dependency {}").unwrap();
    let source = "import { Injectable, inject } from '@angular/core'; import { Dependency } from './token'; @Injectable() export class Owner { dep = inject(Dependency); }";
    let file = p.join("app.ts");
    std::fs::write(&file, source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        file.to_str().unwrap(),
        source,
        Fidelity::High,
        &state,
        p.to_str(),
    )
    .unwrap();
    let authority = artifact.typescript_authority();
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut lowered = PassContext::new(
        source.into(),
        file.to_string_lossy().into_owned(),
        Fidelity::High,
    );
    lowered.language = Some(language);
    lowered.query_string = query.into();
    CoreIRPass::new().run(&mut lowered).unwrap();
    let calls = artifact.invocation_evidence().unwrap();
    let view = calls.view(calls.source_id(), source).unwrap();
    let direct = crate::angular_meta::inject_authority::project(
        source,
        authority,
        &lowered.angular_initializers,
        &view,
    );
    let (published, _, _) = artifact.published_semantics();
    let count = published
        .iter()
        .filter(|edge| edge.relation == crate::layers::meta::semantic::SemanticRelation::Injects)
        .count();
    assert_eq!(
        count,
        1,
        "{}",
        json!({
            "direct":direct, "published":published,
            "classes":format!("{:?}",lowered.angular_initializers),
            "calls":format!("{:?}",view.occurrences()),
            "facets":authority.declaration_facets,
            "references":authority.reference_sites,
            "packages":authority.package_exports,
            "current":authority.capture_current,"disposition":authority.disposition,
        })
    );
}
