use super::*;

#[test]
fn issue_162_provider_declaration_lifecycle_restart_retains_exact_authority_and_endpoints() {
    restart(SOURCE);
}

#[test]
fn issue_162_provider_shorthand_lifecycle_restart_retains_exact_authority_and_endpoints() {
    restart(SHORTHAND_SOURCE);
}

#[test]
fn issue_162_provider_class_strategy_lifecycle_restart_retains_exact_pairs() {
    restart(CLASS_SOURCE);
}

#[test]
fn issue_162_provider_factory_lifecycle_restart_retains_exact_pairs() {
    restart(FACTORY_SOURCE);
}

#[test]
fn issue_162_provider_existing_lifecycle_restart_retains_exact_pairs() {
    restart(&FACTORY_SOURCE.replace("useFactory", "useExisting"));
}

#[test]
fn issue_162_provider_value_lifecycle_restart_retains_registration_and_legacy_projection() {
    restart(&FACTORY_SOURCE.replace("useFactory", "useValue"));
}

#[test]
fn issue_162_provider_declaration_lifecycle_stale_tokens_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(SOURCE);
}

#[test]
fn issue_162_provider_shorthand_lifecycle_stale_tokens_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(SHORTHAND_SOURCE);
}

#[test]
fn issue_162_provider_class_strategy_lifecycle_stale_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(CLASS_SOURCE);
}

#[test]
fn issue_162_provider_factory_lifecycle_stale_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(FACTORY_SOURCE);
}

#[test]
fn issue_162_provider_existing_lifecycle_stale_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(&FACTORY_SOURCE.replace("useFactory", "useExisting"));
}

#[test]
fn issue_162_provider_value_lifecycle_stale_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(&FACTORY_SOURCE.replace("useFactory", "useValue"));
}

#[test]
fn issue_162_provider_directive_lifecycle_restart_retains_exact_authority() {
    restart(&format!("// Directive owner\n{CLASS_SOURCE}"));
}

#[test]
fn issue_162_provider_directive_lifecycle_stale_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(&format!("// Directive owner\n{CLASS_SOURCE}"));
}

#[test]
fn issue_162_provider_directive_lifecycle_api_barrel_stale_restore_and_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let source = format!("// Directive owner\n{CLASS_SOURCE}");
    let (root, config, _) = setup(&source);
    let p = root.path();
    let api = "export {Directive as View} from '@angular/core';";
    std::fs::write(
        p.join("foreign.ts"),
        "export function Directive(metadata:unknown){return metadata;}",
    )
    .unwrap();
    std::fs::write(
        p.join("api.ts"),
        "export {Directive as View} from './foreign';",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config);
    for name in ["app.ts", "neighbor.ts"] {
        let file = p.join(name).to_string_lossy().into_owned();
        let response = call(&state, p, "restore_context", json!({"filePath":file}));
        assert!(
            response.get("error").is_some(),
            "stale API owner adopted: {response}"
        );
        assert!(state.alias_for_path(&file).is_none());
    }
    answers(&state, p, &["app.ts", "neighbor.ts"], &[]);
    strategy_answers(&state, p, &source, 0, &[]);
    std::fs::write(p.join("api.ts"), api).unwrap();
    answers(
        &state,
        p,
        &["app.ts", "neighbor.ts"],
        &["left.ts", "right.ts"],
    );
    strategy_answers(&state, p, &source, 2, &["left.ts", "right.ts"]);
    for name in ["app.ts", "neighbor.ts"] {
        assert_eq!(std::fs::read_to_string(p.join(name)).unwrap(), source);
    }
}
