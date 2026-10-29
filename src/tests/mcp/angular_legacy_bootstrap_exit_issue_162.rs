//! Remaining approved legacy API/acquisition/dependency exit controls.
use super::angular_bootstrap_inline_controls_tests::{call, capture, raw};
use super::angular_legacy_bootstrap_controls_tests::{CALL, FOUR, PREFIX, facts, setup};
use serde_json::json;
#[test]
fn issue_162_legacy_exit_unresolved_ambiguous_factory_and_sibling_preservation() {
    let _serial = crate::protocol::handler_response_serial();
    for import in [
        "import {missing as make} from './legacy-api';",
        "import {platformBrowserDynamic as make} from './competing';",
        "import {platformBrowserDynamic as make} from './absent';",
    ] {
        let (root, config) = setup();
        let p = root.path();
        std::fs::write(
            p.join("legacy-api.ts"),
            "export {platformBrowserDynamic as start} from '@angular/platform-browser-dynamic';",
        )
        .unwrap();
        std::fs::write(
            p.join("foreign.ts"),
            "export function platformBrowserDynamic(){return {bootstrapModule(module:unknown){}};}",
        )
        .unwrap();
        std::fs::write(
            p.join("competing.ts"),
            "export * from '@angular/platform-browser-dynamic'; export * from './foreign';",
        )
        .unwrap();
        std::fs::write(
            p.join("legacy-a.ts"),
            format!("{import} import {{AppModule}} from './module'; {CALL}"),
        )
        .unwrap();
        let state = crate::mcp::McpState::new(config);
        let a = capture(&state, p, "legacy-a.ts");
        assert!(
            crate::angular_meta::legacy_bootstrap::project(&a).is_empty(),
            "{import}"
        );
        facts(
            &state,
            p,
            &[("legacy-b.ts", "module.ts"), ("legacy-b.ts", "module.ts")],
        );
        super::angular_config_composition_controls_tests::facts(
            &state,
            p,
            &["base.ts", "feature.ts"],
        );
    }
}
#[test]
fn issue_162_legacy_exit_barrel_config_stale_restore_and_unchanged_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..2 {
        let (root, config) = setup();
        let p = root.path();
        std::fs::write(
            p.join("legacy-api.ts"),
            "export {platformBrowserDynamic as start} from '@angular/platform-browser-dynamic';",
        )
        .unwrap();
        std::fs::write(
            p.join("foreign.ts"),
            "export function platformBrowserDynamic(){return {bootstrapModule(module:unknown){}};}",
        )
        .unwrap();
        for f in ["legacy-a.ts", "legacy-b.ts"] {
            std::fs::write(p.join(f),format!("import {{start as make}} from './legacy-api'; import {{AppModule}} from './module'; {CALL}{CALL}")).unwrap();
        }
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "legacy-a.ts");
        let edges = crate::angular_meta::legacy_bootstrap::project(&a);
        assert_eq!(edges.len(), 2);
        facts(&state, p, &FOUR);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("legacy-a.ts")}),
        );
        drop(state);
        let file = p.join(if mode == 0 {
            "legacy-api.ts"
        } else {
            "tsconfig.json"
        });
        let original = std::fs::read_to_string(&file).unwrap();
        let changed = if mode == 0 {
            "export {platformBrowserDynamic as start} from './foreign';".to_owned()
        } else {
            let mut value: serde_json::Value = serde_json::from_str(&original).unwrap();
            value["compilerOptions"]["paths"] =
                json!({"@angular/platform-browser-dynamic":["./foreign.ts"]});
            serde_json::to_string(&value).unwrap()
        };
        std::fs::write(&file, changed).unwrap();
        assert!(!a.admits_relation_evidence(&edges));
        let next = crate::mcp::McpState::new(config.clone());
        let denied = raw(
            &next,
            p,
            "restore_context",
            json!({"filePath":p.join("legacy-a.ts")}),
        );
        assert!(
            denied.get("error").is_some() || denied["result"]["isError"] == true,
            "{denied}"
        );
        facts(&next, p, &[]);
        drop(next);
        std::fs::write(file, original).unwrap();
        let repaired = crate::mcp::McpState::new(config);
        facts(&repaired, p, &FOUR);
    }
}
#[test]
fn issue_162_legacy_exit_all_routes_cold_warm_refresh() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup();
    let p = root.path();
    std::fs::write(
        p.join("legacy-api.ts"),
        "export {platformBrowserDynamic as start} from '@angular/platform-browser-dynamic';",
    )
    .unwrap();
    let routes = [
        (
            "legacy-a.ts",
            "import {platformBrowserDynamic} from '@angular/platform-browser-dynamic';",
            "platformBrowserDynamic",
        ),
        (
            "legacy-b.ts",
            "import {platformBrowserDynamic as launch} from '@angular/platform-browser-dynamic';",
            "launch",
        ),
        (
            "legacy-c.ts",
            "import {start} from './legacy-api';",
            "start",
        ),
        (
            "legacy-d.ts",
            "import * as platform from '@angular/platform-browser-dynamic';",
            "platform.platformBrowserDynamic",
        ),
    ];
    for (file, import, factory) in routes {
        std::fs::write(p.join(file),format!("{import} import {{AppModule}} from './module'; {factory}().bootstrapModule(AppModule); {factory}().bootstrapModule(AppModule);")).unwrap();
    }
    let expected = [
        ("legacy-a.ts", "module.ts"),
        ("legacy-a.ts", "module.ts"),
        ("legacy-b.ts", "module.ts"),
        ("legacy-b.ts", "module.ts"),
        ("legacy-c.ts", "module.ts"),
        ("legacy-c.ts", "module.ts"),
        ("legacy-d.ts", "module.ts"),
        ("legacy-d.ts", "module.ts"),
    ];
    let state = crate::mcp::McpState::new(config);
    facts(&state, p, &expected);
    facts(&state, p, &expected);
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &expected);
}
#[test]
fn issue_162_legacy_exit_module_deletion_recreation_and_empty_owner() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup();
    let p = root.path();
    let original = std::fs::read_to_string(p.join("module.ts")).unwrap();
    let state = crate::mcp::McpState::new(config);
    facts(&state, p, &FOUR);
    std::fs::remove_file(p.join("module.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &[]);
    std::fs::write(p.join("module.ts"), original).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &FOUR);
    std::fs::write(p.join("legacy-a.ts"), PREFIX).unwrap();
    facts(
        &state,
        p,
        &[("legacy-b.ts", "module.ts"), ("legacy-b.ts", "module.ts")],
    );
}
