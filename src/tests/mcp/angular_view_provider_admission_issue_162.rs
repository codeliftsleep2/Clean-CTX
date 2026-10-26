use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;

#[test]
fn issue_162_view_provider_admission_positions_refusals_and_independent_removal() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    std::fs::write(
        p.join("api.ts"),
        "export {Component as View} from '@angular/core';",
    )
    .unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function Component(metadata:unknown){return metadata;}",
    )
    .unwrap();
    std::fs::write(
        p.join("tokens.ts"),
        "export class Token {} export class Target {} export function Build(){}",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let capture = |source: &str| {
        std::fs::write(p.join("app.ts"), source).unwrap();
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(162),
            "compress_code_context",
            &json!({"arguments":{"workspaceRoot":p,"filePath":p.join("app.ts"),"fidelity":"high"}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        state
            .semantic_snapshot(
                &state
                    .alias_for_path(&p.join("app.ts").to_string_lossy())
                    .unwrap(),
            )
            .unwrap()
    };
    let source = "import {View} from './api'; import {Token,Target,Build} from './tokens'; @View({providers:[{provide:Token,useValue:1}],viewProviders:[{provide:Token,useClass:Target},{provide:Token,useFactory:Build},{provide:Token,useExisting:Target},{provide:Token,useValue:1}]}) export class Host {}";
    let baseline = capture(source);
    assert_eq!(
        baseline
            .edges
            .iter()
            .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
            .count(),
        1
    );
    assert_eq!(
        baseline
            .edges
            .iter()
            .filter(|edge| edge.relation == SemanticRelation::RegistersViewProvider)
            .count(),
        4
    );
    assert!(baseline.authority.admits_relation_evidence(&baseline.edges));
    let ordinary = baseline
        .edges
        .iter()
        .find(|edge| edge.relation == SemanticRelation::RegistersProvider)
        .unwrap();
    let view = baseline
        .edges
        .iter()
        .find(|edge| edge.relation == SemanticRelation::RegistersViewProvider)
        .unwrap();
    assert_eq!(ordinary.subject.canonical, view.subject.canonical);
    assert_eq!(ordinary.object.canonical, view.object.canonical);
    let encoded = serde_json::to_string(&SemanticRelation::RegistersViewProvider).unwrap();
    assert_eq!(encoded, "\"RegistersViewProvider\"");
    assert_eq!(
        serde_json::from_str::<SemanticRelation>(&encoded).unwrap(),
        SemanticRelation::RegistersViewProvider
    );
    let without_view = source.replace(
        "viewProviders:[{provide:Token,useClass:Target},{provide:Token,useFactory:Build},{provide:Token,useExisting:Target},{provide:Token,useValue:1}]", "viewProviders:[]");
    let without_ordinary = source.replace("providers:[{provide:Token,useValue:1}]", "providers:[]");
    for (text, ordinary_count, view_count) in [(&without_view, 1, 0), (&without_ordinary, 0, 4)] {
        let snapshot = capture(text);
        assert_eq!(
            snapshot
                .edges
                .iter()
                .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
                .count(),
            ordinary_count
        );
        assert_eq!(
            snapshot
                .edges
                .iter()
                .filter(|edge| edge.relation == SemanticRelation::RegistersViewProvider)
                .count(),
            view_count
        );
    }
    let ordinary_only = capture(&without_view);
    let mut forged = ordinary_only
        .edges
        .iter()
        .find(|edge| edge.relation == SemanticRelation::RegistersProvider)
        .unwrap()
        .clone();
    forged.relation = SemanticRelation::RegistersViewProvider;
    assert!(!ordinary_only.authority.admits_relation_evidence(&[forged]));
    let view_only = "import {Component as View} from '@angular/core'; import {Token} from './tokens'; @View({viewProviders:[{provide:Token,useValue:1}]}) export class Host {}";
    for invalid in [
        view_only.replace("from '@angular/core'", "from './foreign'"),
        view_only.replace("import {Component", "import type {Component"),
        view_only.replace("Component as View", "Directive as View"),
        view_only.replace("Component as View", "Pipe as View"),
        view_only.replace("Component as View", "Injectable as View"),
        view_only.replace("provide:Token", "provide:Token,provide:Token"),
        view_only.replace("useValue:1", "useValue:1,useClass:Token"),
        view_only.replace("viewProviders:[", "viewProviders:[],viewProviders:["),
        view_only.replace("{Token}", "type {Token}"),
    ] {
        let refused = capture(&invalid);
        assert!(
            !refused
                .edges
                .iter()
                .any(|edge| edge.relation == SemanticRelation::RegistersViewProvider),
            "invalid view position: {invalid}"
        );
        assert_eq!(
            capture(view_only)
                .edges
                .iter()
                .filter(|edge| edge.relation == SemanticRelation::RegistersViewProvider)
                .count(),
            1,
            "repair"
        );
    }
}
