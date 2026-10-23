use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};
use serde_json::json;

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState, String) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    std::fs::write(p.join("tsconfig.json"), r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    let package = p.join("node_modules/@angular/core");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@angular/core","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(package.join("index.d.ts"), "export declare function NgModule(metadata: unknown): any; export declare function Directive(metadata: unknown): any;").unwrap();
    let leaf = "import { Directive as Dir } from '@angular/core'; @Dir({selector:'[leaf]'}) export class Leaf {}";
    std::fs::write(p.join("leaf.ts"), leaf).unwrap();
    std::fs::write(
        p.join("shared.ts"),
        "import { Leaf } from './leaf'; export const SHARED = [Leaf];",
    )
    .unwrap();
    let source = "import { NgModule as Mod } from '@angular/core'; import { SHARED } from './shared'; @Mod({declarations:[...SHARED]}) export class AppModule {}".to_string();
    std::fs::write(p.join("app.ts"), &source).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config), source)
}

#[test]
fn issue_162_module_facets_retains_exact_dependency_and_roundtrips() {
    let (root, state, source) = fixture();
    let file = root.path().join("app.ts");
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        file.to_str().unwrap(),
        &source,
        crate::compression::Fidelity::High,
        &state,
        root.path().to_str(),
    )
    .unwrap();
    let authority = artifact.typescript_authority();
    assert!(authority.capture_current, "{:?}", authority.disposition);
    let leaf = authority
        .declaration_facets
        .iter()
        .find(|facet| {
            facet.declaration.source.ends_with("leaf.ts") && facet.declaration.name == "Leaf"
        })
        .expect("demanded exact dependency facet");
    let leaf_bytes = std::fs::read(root.path().join("leaf.ts")).unwrap();
    assert_eq!(
        leaf.declaration.source_hash,
        crate::cache::content_hash(&leaf_bytes)
    );
    assert!(leaf.declaration.start < leaf.declaration.end);
    assert!(leaf.declaration.end <= leaf_bytes.len());
    let [decorator] = leaf.decorators.as_slice() else {
        panic!("one dependency decorator");
    };
    let ExpressionValue::Reference(Resolution::Resolved(api)) = &decorator.callee.value else {
        panic!("canonical dependency API reference");
    };
    assert_eq!(
        &leaf_bytes[decorator.callee.start..decorator.callee.end],
        b"Dir"
    );
    assert!(
        api.source
            .ends_with("node_modules/@angular/core/index.d.ts")
    );
    assert_eq!(api.name, "Directive");
    assert!(
        authority
            .package_exports
            .iter()
            .any(|package| package.declaration == *api
                && package.public_export == Resolution::Resolved(api.clone()))
    );
    let value = serde_json::to_value(authority).unwrap();
    let restored: TypeScriptAuthority = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(restored).unwrap(), value);
    assert!(
        !serde_json::to_string(&leaf)
            .unwrap()
            .contains(std::str::from_utf8(&leaf_bytes).unwrap()),
        "facets must not persist a whole dependency source"
    );
}

#[test]
fn issue_162_module_facets_imported_spread_survives_public_composition() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state, _) = fixture();
    for _ in 0..2 {
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(162),
            "workspace_query",
            &json!({"arguments":{"type":"forward_edges","workspaceRoot":root.path(),
                "domain":"angular","entity_type":"Module","name":"AppModule"}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        let edges = response["result"]["structuredContent"]["edges"]
            .as_array()
            .unwrap();
        let members: Vec<_> = edges
            .iter()
            .filter(|edge| edge["relation"] == "DeclaresInModule")
            .collect();
        assert_eq!(members.len(), 1, "{response}");
        assert_eq!(members[0]["object"]["name"], "Leaf");
        assert_eq!(members[0]["object"]["entity_type"], "Directive");
        assert!(
            std::path::Path::new(members[0]["object"]["file"].as_str().unwrap())
                .ends_with("leaf.ts")
        );
        let owner = state.semantic_owner_path(root.path().join("app.ts").to_str().unwrap());
        let index = state.workspace_index_read();
        let authority = index
            .retained_typescript_authority(&owner)
            .expect("graph hydration must retain current typed dependency authority");
        assert!(
            authority.declaration_facets.iter().any(|facet| facet
                .declaration
                .source
                .ends_with("leaf.ts")
                && !facet.decorators.is_empty())
        );
    }
}

fn exported_alias_trace(after_hydration: bool) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state, _) = fixture();
    let file = root.path().join("leaf.ts");
    let before = "import { Directive } from '@angular/core'; @Directive({selector:'[leaf]'}) export class Leaf {}";
    let after = "import { Directive as DirectiveAlias } from '@angular/core'; @DirectiveAlias({selector:'[leaf]'}) export class Leaf {}";
    let query = || {
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(162),
            "workspace_query",
            &json!({"arguments":{"type":"entities_in_file","file_path":file,
                "workspaceRoot":root.path(),"fidelity":"high"}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        response
    };
    let count = |response: &serde_json::Value| {
        response["result"]["structuredContent"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entity| {
                entity["domain"] == "angular"
                    && entity["entity_type"] == "Directive"
                    && entity["name"] == "Leaf"
            })
            .count()
    };
    std::fs::write(&file, before).unwrap();
    if after_hydration {
        let owner = root.path().join("app.ts");
        std::fs::write(&owner, "import { NgModule } from '@angular/core'; import { Leaf } from './leaf'; @NgModule({declarations:[Leaf]}) export class AppModule {}").unwrap();
        for arguments in [
            json!({"type":"entities_in_file","file_path":owner,"fidelity":"high"}),
            json!({"type":"forward_edges","domain":"angular","entity_type":"Module","name":"AppModule"}),
        ] {
            crate::protocol::captured_responses().clear();
            let mut arguments = arguments;
            arguments["workspaceRoot"] = json!(root.path());
            crate::mcp::tools::dispatch_tools_call(
                &json!(162),
                "workspace_query",
                &json!({"arguments":arguments}),
                &state,
            );
            let response = crate::protocol::captured_responses().pop().unwrap();
            assert!(response.get("error").is_none(), "{response}");
            if let Some(edges) = response["result"]["structuredContent"]["edges"].as_array() {
                assert_eq!(
                    edges
                        .iter()
                        .filter(|edge| edge["relation"] == "DeclaresInModule")
                        .count(),
                    1,
                    "hydrated positive control: {response}"
                );
            }
        }
    } else {
        assert_eq!(count(&query()), 1, "unaliased control");
    }
    std::fs::write(&file, after).unwrap();
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        file.to_str().unwrap(),
        after,
        crate::compression::Fidelity::High,
        &state,
        root.path().to_str(),
    )
    .unwrap();
    let authority = artifact.typescript_authority();
    let facets: Vec<_> = authority
        .declaration_facets
        .iter()
        .filter(|facet| {
            facet.declaration.source == authority.source && facet.declaration.name == "Leaf"
        })
        .collect();
    let roles: Vec<_> = facets
        .iter()
        .map(|facet| {
            format!(
                "{:?}",
                crate::angular_meta::class_role_authority::declaration_role(
                    authority,
                    &facet.declaration
                )
            )
        })
        .collect();
    let start = after.find("@DirectiveAlias").unwrap();
    let direct_role = crate::angular_meta::class_role_authority::class_role(
        after,
        &after[start..],
        start,
        authority,
    )
    .map(|role| format!("{:?}", role.kind));
    let (edges, _, _) = artifact.published_semantics();
    let owner = state.semantic_owner_path(file.to_str().unwrap());
    let current_before_query = state
        .workspace_index_read()
        .has_current_semantic_projection(
            &owner,
            crate::workspace::index::SemanticFidelity::High,
            &crate::cache::content_hash(after.as_bytes()),
        );
    let entities_before_query = {
        let index = state.workspace_index_read();
        serde_json::to_value(index.entities_in_file(&owner)).unwrap()
    };
    let response = query();
    assert_eq!(
        count(&response),
        1,
        "{}",
        json!({
            "capture_current":authority.capture_current,"disposition":authority.disposition,
            "source_hash":authority.source_hash,"facets":facets,"roles":roles,
            "direct_role":direct_role,"artifact_edges":edges,
            "stale_witnesses":authority.witnesses.iter().filter(|witness| !witness.current()).collect::<Vec<_>>(),
            "after_hydration":after_hydration,"current_before_query":current_before_query,
            "entities_before_query":entities_before_query,"response":response,
        })
    );
}

#[test]
fn issue_162_module_alias_trace_exported_role_after_mutation() {
    exported_alias_trace(false);
}

#[test]
fn issue_162_module_alias_trace_after_graph_hydration() {
    exported_alias_trace(true);
}
