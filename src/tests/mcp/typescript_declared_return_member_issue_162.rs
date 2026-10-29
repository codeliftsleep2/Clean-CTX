//! Exact declared-return/member proof controls, independent of Angular relation publication.
use super::angular_bootstrap_inline_controls_tests::{call, capture};
use super::angular_config_composition_controls_tests::setup as base_setup;
use crate::ir::invocation_evidence::SourceSpan;
use crate::ir::typescript_resolution::TypeScriptAuthority;
use serde_json::json;
use std::path::Path;
const FACTORY: &str = "import {PlatformRef} from '@angular/core'; export declare function platformBrowserDynamic():PlatformRef;";
const MEMBER: &str =
    " export declare class PlatformRef { bootstrapModule(module:unknown):unknown; }";
fn setup(
    factory: &str,
    member: &str,
    source: &str,
) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = base_setup("Base,Feature");
    let p = root.path();
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut text = std::fs::read_to_string(&core).unwrap();
    text.push_str(member);
    std::fs::write(core, text).unwrap();
    let package = p.join("node_modules/@angular/platform-browser-dynamic");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@angular/platform-browser-dynamic","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(package.join("index.d.ts"), factory).unwrap();
    std::fs::write(p.join("chain.ts"), source).unwrap();
    std::fs::write(
        p.join("fake-chain.ts"),
        "export function platformBrowserDynamic(){return {bootstrapModule(module:unknown){}};}",
    )
    .unwrap();
    (root, config)
}
const SOURCE: &str = "import {platformBrowserDynamic as make} from '@angular/platform-browser-dynamic'; import {Root} from './root'; make().bootstrapModule(Root); make().bootstrapModule(Root);";
fn proof(authority: &TypeScriptAuthority, index: usize) -> SourceSpan {
    let p = &authority.declared_return_members[index];
    SourceSpan::new(p.access.0, p.access.1).unwrap()
}
fn prove(authority: &TypeScriptAuthority) {
    assert_eq!(authority.declared_return_members.len(), 2, "{authority:?}");
    for index in 0..2 {
        let evidence = authority
            .declared_return_member_at(proof(authority, index))
            .expect("exact neutral proof replay");
        assert_eq!(evidence.factory.name, "platformBrowserDynamic");
        assert_eq!(evidence.receiver.name, "PlatformRef");
        assert_eq!(evidence.method.name, "bootstrapModule");
        assert_eq!(evidence.method.source, evidence.receiver.source);
        assert_eq!(evidence.method.source_hash, evidence.receiver.source_hash);
        assert!(
            evidence.receiver.start <= evidence.method.start
                && evidence.method.end <= evidence.receiver.end
        );
        assert_eq!(
            authority.value_reference_at(proof(authority, index)),
            Some(&evidence.method)
        );
    }
}
#[test]
fn issue_162_declared_return_member_exact_retained_and_serde() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(FACTORY, MEMBER, SOURCE);
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, root.path(), "chain.ts");
    prove(&authority);
    let encoded = serde_json::to_vec(&authority).unwrap();
    let restored: TypeScriptAuthority = serde_json::from_slice(&encoded).unwrap();
    prove(&restored);
    let answer = call(
        &state,
        root.path(),
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapModule"}),
    );
    assert!(
        !answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["relation"] == "DeclaresBootstrapModule"),
        "neutral authority does not publish Angular relation"
    );
}
#[test]
fn issue_162_declared_return_member_crossed_missing_borrowed_refusals() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(FACTORY, MEMBER, SOURCE);
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, root.path(), "chain.ts");
    prove(&authority);
    let site = proof(&authority, 0);
    for mode in 0..16 {
        let mut bad = authority.clone();
        let original = bad.declared_return_members[0].clone();
        match mode {
            0 => bad.declared_return_members.clear(),
            1 => bad.declared_return_members.push(original.clone()),
            2 => bad.declared_return_members[0].factory.source_hash.push('x'),
            3 => bad.declared_return_members[0]
                .receiver
                .source_hash
                .push('x'),
            4 => bad.declared_return_members[0].method.source_hash.push('x'),
            5 => bad.declared_return_members[0].inner = bad.declared_return_members[1].inner,
            6 => bad.declared_return_members[0].property = bad.declared_return_members[1].property,
            7 => bad.declared_return_members[0].return_site.0 += 1,
            8 => bad.declared_return_members[0].factory.start += 1,
            9 => bad.declared_return_members[0].receiver.start += 1,
            10 => bad.declared_return_members[0].method.start += 1,
            11 => bad.demanded_occurrences.retain(|s| {
                !(s.source == original.factory.source && s.start == original.return_site.0)
            }),
            12 => bad.invocation_facets.retain(|s| {
                s.expression.start != original.outer.0 || s.expression.end != original.outer.1
            }),
            13 => bad.demanded_occurrences.retain(|s| {
                !(s.source == original.receiver.source && s.start == original.receiver.start)
            }),
            14 => {
                bad.declared_return_members[0].method.name = "other".into();
            }
            15 => bad.declared_return_members[0].source_hash.push('x'),
            _ => unreachable!(),
        }
        assert!(bad.declared_return_member_at(site).is_none(), "mode={mode}");
        let roundtrip: TypeScriptAuthority =
            serde_json::from_value(serde_json::to_value(&bad).unwrap()).unwrap();
        assert!(
            roundtrip.declared_return_member_at(site).is_none(),
            "serde mode={mode}"
        );
    }
}
#[test]
fn issue_162_declared_return_member_signature_and_receiver_refusals() {
    let _serial = crate::protocol::handler_response_serial();
    let cases = [
        (
            "export declare function platformBrowserDynamic():unknown;",
            MEMBER,
            SOURCE,
        ),
        (
            "export declare function platformBrowserDynamic();",
            MEMBER,
            SOURCE,
        ),
        (
            "import {PlatformRef} from '@angular/core'; export declare function platformBrowserDynamic():PlatformRef; export declare function platformBrowserDynamic(x:unknown):PlatformRef;",
            MEMBER,
            SOURCE,
        ),
        (
            FACTORY,
            " export declare class PlatformRef { static bootstrapModule(module:unknown):unknown; }",
            SOURCE,
        ),
        (
            FACTORY,
            " export declare class PlatformRef { bootstrapModule(module:unknown):unknown; bootstrapModule(module:string):unknown; }",
            SOURCE,
        ),
        (
            FACTORY,
            " export declare class PlatformRef { other(module:unknown):unknown; }",
            SOURCE,
        ),
        (
            FACTORY,
            MEMBER,
            "import {platformBrowserDynamic as make} from './fake-chain'; make().bootstrapModule(1);",
        ),
        (
            FACTORY,
            MEMBER,
            "const make=()=>({bootstrapModule(value:unknown){}}); make().bootstrapModule(1);",
        ),
        (
            FACTORY,
            MEMBER,
            "import {platformBrowserDynamic} from '@angular/platform-browser-dynamic'; function shadow(platformBrowserDynamic:any){platformBrowserDynamic().bootstrapModule(1);}",
        ),
        (
            FACTORY,
            MEMBER,
            "import {platformBrowserDynamic as make} from '@angular/platform-browser-dynamic'; make(1).bootstrapModule(1);",
        ),
        (
            FACTORY,
            MEMBER,
            "import {platformBrowserDynamic as make} from '@angular/platform-browser-dynamic'; make()['bootstrapModule'](1);",
        ),
        (
            FACTORY,
            MEMBER,
            "import {platformBrowserDynamic as make} from '@angular/platform-browser-dynamic'; make()?.bootstrapModule(1);",
        ),
    ];
    for (factory, member, source) in cases {
        let (root, config) = setup(factory, member, source);
        let state = crate::mcp::McpState::new(config);
        let a = capture(&state, root.path(), "chain.ts");
        assert!(
            a.declared_return_members.is_empty(),
            "{source}; {factory}; {member}; {a:?}"
        );
    }
}
fn retained(state: &crate::mcp::McpState, p: &Path) -> TypeScriptAuthority {
    let alias = state
        .alias_for_path(&p.join("chain.ts").to_string_lossy())
        .unwrap();
    state
        .semantic_snapshot(&alias)
        .unwrap()
        .authority
        .typescript_authority
        .unwrap()
}
#[test]
fn issue_162_declared_return_member_sqlite_stale_dependency_and_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(FACTORY, MEMBER, SOURCE);
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let a = capture(&state, p, "chain.ts");
    prove(&a);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("chain.ts")}),
    );
    drop(state);
    let restored = crate::mcp::McpState::new(config.clone());
    call(
        &restored,
        p,
        "restore_context",
        json!({"filePath":p.join("chain.ts")}),
    );
    let b = retained(&restored, p);
    prove(&b);
    assert_eq!(a.declared_return_members, b.declared_return_members);
    drop(restored);
    let file = p.join("node_modules/@angular/platform-browser-dynamic/index.d.ts");
    std::fs::write(
        &file,
        "export declare function platformBrowserDynamic():unknown;",
    )
    .unwrap();
    assert!(
        a.declared_return_member_at(proof(&a, 0)).is_none(),
        "unchanged caller cannot reuse stale return authority"
    );
    let next = crate::mcp::McpState::new(config.clone());
    let changed = capture(&next, p, "chain.ts");
    assert!(changed.declared_return_members.is_empty());
    drop(next);
    std::fs::write(&file, FACTORY).unwrap();
    let repaired = crate::mcp::McpState::new(config);
    prove(&capture(&repaired, p, "chain.ts"));
}
