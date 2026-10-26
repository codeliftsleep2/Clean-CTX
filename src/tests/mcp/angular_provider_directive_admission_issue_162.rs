use crate::angular_meta::{decorators::ClassKind, provider_semantics};
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;

#[test]
fn issue_162_provider_directive_owner_refusals_and_strategy_preservation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut declarations = std::fs::read_to_string(&core).unwrap();
    declarations.push_str("export declare function NgModule(metadata: unknown): any;\n");
    std::fs::write(core, declarations).unwrap();
    std::fs::write(
        p.join("tokens.ts"),
        "export class Token {} export class Target {} export function Build() {}",
    )
    .unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function Directive(metadata:unknown){return metadata;}",
    )
    .unwrap();
    let source = "import {Directive} from '@angular/core'; import {Token,Target,Build} from './tokens'; @Directive({selector:'[probe]',providers:[{provide:Token,useClass:Target},{provide:Token,useFactory:Build},{provide:Token,useExisting:Target},{provide:Token,useValue:Target}]}) export class ProviderHost {}";
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let capture = |text: &str| {
        std::fs::write(p.join("app.ts"), text).unwrap();
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(162),
            "compress_code_context",
            &json!({"arguments":{"workspaceRoot":p,"filePath":p.join("app.ts"),"fidelity":"high"}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        let alias = state
            .alias_for_path(&p.join("app.ts").to_string_lossy())
            .unwrap();
        state.semantic_snapshot(&alias).unwrap()
    };
    for (api, kind) in [
        ("Directive", "Directive"),
        ("Component", "Component"),
        ("NgModule", "Module"),
    ] {
        let baseline = capture(&source.replace("Directive", api));
        let registrations: Vec<_> = baseline
            .edges
            .iter()
            .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
            .collect();
        assert_eq!(
            registrations.len(),
            4,
            "all established strategy entries for {api}"
        );
        assert!(
            registrations
                .iter()
                .all(|edge| edge.subject.entity_type == kind
                    && edge.subject.canonical.is_some()
                    && edge.object.canonical.is_some())
        );
        for relation in [
            SemanticRelation::ProviderUsesClass,
            SemanticRelation::ProviderUsesFactory,
            SemanticRelation::ProviderUsesExisting,
            SemanticRelation::ProviderUsesValue,
        ] {
            assert_eq!(
                baseline
                    .edges
                    .iter()
                    .filter(|edge| edge.relation == relation)
                    .count(),
                1
            );
        }
        assert!(baseline.authority.admits_relation_evidence(&baseline.edges));
        let value = baseline
            .edges
            .iter()
            .find(|edge| edge.relation == SemanticRelation::ProviderUsesValue)
            .unwrap();
        assert!(value.subject.canonical.is_none() && value.object.canonical.is_none());
    }
    for invalid in [
        source.replace("from '@angular/core'", "from './foreign'"),
        source.replace("import {Directive}", "import type {Directive}"),
        source.replace("Directive", "Pipe"),
        source.replace("Directive", "Injectable"),
        source
            .replace("import {Directive}", "import {Directive,Component}")
            .replace("@Directive", "@Component({}) @Directive"),
        source.replace("providers:[", "providers:[],providers:["),
        source
            .replace(
                "{provide:Token,useClass:Target}",
                "{provide:Token,provide:Token,useClass:Target}",
            )
            .replace(
                "{provide:Token,useFactory:Build}",
                "{provide:Token,useFactory:Build,useValue:Target}",
            )
            .replace(
                "{provide:Token,useExisting:Target}",
                "{provide:Token,useExisting:Target,useClass:Target}",
            )
            .replace(
                "{provide:Token,useValue:Target}",
                "{provide:Token,useValue:Target,useValue:Target}",
            ),
        source
            .replace("import {Directive}", "import {Directive as Native}")
            .replace(
                "@Directive",
                "function Directive(metadata:unknown){return metadata;} @Directive",
            ),
    ] {
        let refused = capture(&invalid);
        assert!(
            !refused
                .edges
                .iter()
                .any(|edge| edge.relation == SemanticRelation::RegistersProvider),
            "invalid owner/entry cannot register: {invalid}"
        );
        assert_eq!(
            capture(source)
                .edges
                .iter()
                .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
                .count(),
            4,
            "repair restores without a policy change"
        );
    }
    let baseline = capture(source);
    let authority = baseline.authority.typescript_authority.as_ref().unwrap();
    let path = p.join("app.ts").to_string_lossy().into_owned();
    assert!(
        provider_semantics::project_with_authority(
            &path,
            source,
            "ProviderHost",
            ClassKind::Directive,
            None
        )
        .is_empty()
    );
    let mut missing = authority.clone();
    missing.package_exports.clear();
    assert!(
        provider_semantics::project_with_authority(
            &path,
            source,
            "ProviderHost",
            ClassKind::Directive,
            Some(&missing)
        )
        .is_empty()
    );
    let mut forged = authority.clone();
    forged.source_hash.push_str("forged");
    // A source-mismatched owner cannot supply source-owned occurrence authority.
    forged.source = p.join("other.ts");
    assert!(
        provider_semantics::project_with_authority(
            &path,
            source,
            "ProviderHost",
            ClassKind::Directive,
            Some(&forged)
        )
        .is_empty()
    );
}
