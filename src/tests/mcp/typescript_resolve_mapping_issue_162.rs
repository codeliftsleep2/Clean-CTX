//! Neutral immediate resolve mapping authority, never ResolvedBy admission.
use super::typescript_resolve_field_tests::setup;
use super::typescript_route_guard_field_support::{call, capture};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use serde_json::json;
fn prove(a: &TypeScriptAuthority) -> usize {
    for p in &a.resolve_mappings {
        assert_eq!(p.source, a.source);
        assert_eq!(p.source_hash, a.source_hash);
        let (mapping, id) = a.resolve_mapping_at(p.pair).expect("exact mapping replay");
        assert_eq!(mapping, p);
        assert_eq!(
            a.demanded_reference_at(&a.source, &a.source_hash, p.value.0, p.value.1),
            Some(id)
        );
        assert_eq!(
            a.value_reference_at(
                crate::ir::invocation_evidence::SourceSpan::new(p.value.0, p.value.1).unwrap()
            ),
            Some(id)
        );
    }
    a.resolve_mappings.len()
}
#[test]
fn issue_162_resolve_mapping_forms_exact_keys_serde_old_payload() {
    let _serial = crate::protocol::handler_response_serial();
    for form in ["direct", "alias", "barrel", "namespace", "local"] {
        let (root, config) = setup(form);
        let p = root.path();
        let state = crate::mcp::McpState::new(config);
        for owner in ["one.ts", "two.ts"] {
            let a = capture(&state, p, owner);
            assert_eq!(prove(&a), 7);
            for mapping in &a.resolve_mappings {
                let (_, id) = a.resolve_mapping_at(mapping.pair).unwrap();
                let relative = id.source.strip_prefix(p).unwrap().to_str().unwrap();
                let independent = capture(&state, p, relative);
                assert!(
                    independent.declarations.contains(id),
                    "exact independent selected declaration"
                );
            }
            let encoded = serde_json::to_value(&a).unwrap();
            let restored: TypeScriptAuthority = serde_json::from_value(encoded.clone()).unwrap();
            assert_eq!(restored.resolve_mappings, a.resolve_mappings);
            assert_eq!(prove(&restored), 7);
            let mut old = encoded;
            old.as_object_mut().unwrap().remove("resolve_mappings");
            let legacy: TypeScriptAuthority = serde_json::from_value(old).unwrap();
            assert!(legacy.resolve_mappings.is_empty());
            assert!(
                legacy
                    .resolve_mapping_at(a.resolve_mappings[0].pair)
                    .is_none()
            );
        }
    }
}
#[test]
fn issue_162_resolve_mapping_forged_borrowed_crossed_refused() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    let other = capture(&state, p, "two.ts");
    assert_eq!(prove(&a), 7);
    let pair = a.resolve_mappings[0].pair;
    for mode in 0..14 {
        let mut b = a.clone();
        let second = b.resolve_mappings[1].clone();
        match mode {
            0 => b.resolve_mappings[0].source = other.source.clone(),
            1 => b.resolve_mappings[0].source_hash = "wrong".into(),
            2 => b.resolve_mappings[0].owner = second.owner,
            3 => b.resolve_mappings[0].field_pair = second.field_pair,
            4 => b.resolve_mappings[0].field_key = second.field_key,
            5 => b.resolve_mappings[0].object = second.object,
            6 => b.resolve_mappings[0].key = second.key,
            7 => b.resolve_mappings[0].value = second.value,
            8 => b.resolve_mappings[0].name = "borrowed".into(),
            9 => b.resolve_mappings.push(b.resolve_mappings[0].clone()),
            10 => b
                .demanded_occurrences
                .retain(|o| (o.start, o.end) != b.resolve_mappings[0].key),
            11 => b.demanded_occurrences.retain(|o| (o.start, o.end) != pair),
            12 => b.capture_current = false,
            _ => b.declaration_facets.clear(),
        }
        assert!(b.resolve_mapping_at(pair).is_none(), "forgery mode {mode}");
    }
}
#[test]
fn issue_162_resolve_mapping_duplicates_quotes_and_unsupported_scope() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let owner = p.join("one.ts");
    let original = std::fs::read_to_string(&owner).unwrap();
    let altered=original.replacen("resolve:{data0:Gate}","resolve:{data0:Gate,'d\\u0061ta':Gate,data0:Right,computed:()=>Gate,nested:{data0:Gate},[Gate]:Gate,...other,Gate},resolve:{data0:Gate}",1);
    assert_ne!(altered, original);
    std::fs::write(&owner, altered).unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    assert_eq!(prove(&a), 10);
    assert_eq!(
        a.resolve_mappings
            .iter()
            .filter(|m| m.name == "data")
            .count(),
        1
    );
    assert!(
        !a.resolve_mappings
            .iter()
            .any(|m| ["computed", "nested", "Gate"].contains(&m.name.as_str()))
    );
    let first = &a.resolve_mappings[0];
    let same = a
        .resolve_mappings
        .iter()
        .filter(|m| m.object == first.object)
        .collect::<Vec<_>>();
    assert_eq!(same.len(), 3);
    assert_ne!(same[0].pair, same[2].pair);
    assert_eq!(same[0].name, same[2].name);
    assert_eq!(
        crate::angular_meta::route_guards::project_kinds(&a).len(),
        32
    );
}
#[test]
fn issue_162_resolve_mapping_sqlite_before_queries_and_owner_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("barrel");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let before = capture(&state, p, "one.ts");
    assert_eq!(prove(&before), 7);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("one.ts")}),
    );
    drop(state);
    let state = crate::mcp::McpState::new(config.clone());
    call(
        &state,
        p,
        "restore_context",
        json!({"filePath":p.join("one.ts")}),
    );
    let alias = state
        .alias_for_path(&p.join("one.ts").to_string_lossy())
        .unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let restored = snapshot.authority.typescript_authority.as_ref().unwrap();
    assert_eq!(restored.resolve_mappings, before.resolve_mappings);
    assert_eq!(prove(restored), 7);
    drop(state);
    let original = std::fs::read(p.join("one.ts")).unwrap();
    std::fs::write(
        p.join("one.ts"),
        "import type {Routes} from '@angular/router';export const routes:Routes=[];",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    let changed = capture(&state, p, "one.ts");
    assert_ne!(changed.source_hash, before.source_hash);
    assert!(!snapshot.authority.is_current(&changed.source_hash));
    assert!(changed.resolve_mappings.is_empty());
    drop(state);
    std::fs::write(p.join("one.ts"), original).unwrap();
    let state = crate::mcp::McpState::new(config);
    let repaired = capture(&state, p, "one.ts");
    assert_eq!(prove(&repaired), 7);
    assert_eq!(repaired.resolve_mappings, before.resolve_mappings);
}

#[test]
fn issue_162_resolve_mapping_unresolved_typeonly_unapproved_origins() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let owner = p.join("one.ts");
    let original = std::fs::read_to_string(&owner).unwrap();
    let source = format!(
        "import type {{Gate as TypeOnly}} from './guard-b'; {}",
        original.replacen(
            "resolve:{data0:Gate}",
            "resolve:{missing:Missing,typeonly:TypeOnly},other:{data0:Gate}",
            1
        )
    );
    std::fs::write(owner, source).unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    assert_eq!(a.resolve_mappings.len(), 8);
    for m in &a.resolve_mappings {
        if ["missing", "typeonly"].contains(&m.name.as_str()) {
            assert!(a.resolve_mapping_at(m.pair).is_none());
        }
    }
    assert_eq!(
        a.resolve_mappings
            .iter()
            .filter(|m| a.resolve_mapping_at(m.pair).is_some())
            .count(),
        6
    );
    assert!(!a.resolve_mappings.iter().any(|m| m.name == "other"));
}
#[test]
fn issue_162_resolve_mapping_dependency_barrel_package_config_retirement_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..4 {
        let (root, config) = setup("barrel");
        let p = root.path();
        let owner = std::fs::read(p.join("one.ts")).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let before = capture(&state, p, "one.ts");
        assert_eq!(prove(&before), 7);
        let old_id = before
            .resolve_mapping_at(before.resolve_mappings[0].pair)
            .unwrap()
            .1
            .clone();
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("one.ts")}),
        );
        drop(state);
        let dep = p.join(match mode {
            0 => "guard-a.ts",
            1 => "guard-api.ts",
            2 => "node_modules/@angular/router/package.json",
            _ => "tsconfig.json",
        });
        let bytes = std::fs::read(&dep).unwrap();
        let changed=match mode {
            0=>"import type {CanActivateFn} from '@angular/router';export const Gate:CanActivateFn=()=>false;".into(),
            1=>"export {Gate as A} from './guard-b';export {Gate as B} from './guard-b';".into(),
            2=>{let mut v:serde_json::Value=serde_json::from_slice(&bytes).unwrap();v["version"]=json!("20.0.1");v.to_string()},
            _=>{let mut v:serde_json::Value=serde_json::from_slice(&bytes).unwrap();v["compilerOptions"]["baseUrl"]=json!(".");v.to_string()},
        };
        std::fs::write(&dep, changed).unwrap();
        assert!(
            before
                .resolve_mapping_at(before.resolve_mappings[0].pair)
                .is_none()
        );
        let state = crate::mcp::McpState::new(config.clone());
        let refused = super::angular_bootstrap_inline_controls_tests::raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("one.ts")}),
        );
        assert!(refused.get("error").is_some());
        let now = capture(&state, p, "one.ts");
        assert_eq!(prove(&now), 7);
        assert_eq!(now.source_hash, before.source_hash);
        let id = now
            .resolve_mapping_at(now.resolve_mappings[0].pair)
            .unwrap()
            .1;
        if mode == 0 {
            assert_ne!(id, &old_id);
        }
        if mode == 1 {
            assert!(id.source.ends_with("guard-b.ts"));
        }
        assert_eq!(std::fs::read(p.join("one.ts")).unwrap(), owner);
        drop(state);
        std::fs::write(dep, bytes).unwrap();
        let state = crate::mcp::McpState::new(config);
        let repaired = capture(&state, p, "one.ts");
        assert_eq!(prove(&repaired), 7);
        assert_eq!(repaired.resolve_mappings, before.resolve_mappings);
    }
}
