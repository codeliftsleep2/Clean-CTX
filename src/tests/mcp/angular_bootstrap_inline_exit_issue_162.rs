//! Remaining bounded approval obligations; shared tracked fixture, no production edits.
use super::angular_bootstrap_inline_controls_tests::{
    CALL, PREFIX, call, capture, facts, raw, setup,
};
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::ir::typescript_resolution::demanded_occurrences::DemandedValue;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;

#[test]
fn issue_162_bootstrap_inline_exit_foreign_ambiguous_and_malformed_entries() {
    let _serial = crate::protocol::handler_response_serial();
    let body = format!(
        "{CALL} import {{bootstrapApplication as foreign}} from './fake'; import {{bootstrapApplication as ambiguous}} from './ambiguous'; foreign(Root,{{providers:[...GROUP]}}); ambiguous(Root,{{providers:[...GROUP]}}); import {{Token as Left}} from './left'; import {{Token as Right}} from './right'; launch(Root,{{providers:[{{provide:Left,provide:Right,useValue:1}}]}}); launch(Root,{{providers:[{{provide:Left,useValue:1,useClass:Right}}]}}); launch(Root,{{providers:[{{provide:Left,useValue:()=>1}}]}});"
    );
    let (root, config) = setup(&body);
    let p = root.path();
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from '@angular/platform-browser'; export * from './fake';",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, p, "main.ts");
    let edges = crate::angular_meta::bootstrap_references::project(&authority);
    assert_eq!(
        edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
            .count(),
        4,
        "foreign/ambiguous calls must not gain root authority"
    );
    assert!(authority.admits_relation_evidence(&edges));
    facts(&state, p, &[("main.ts", 1, 1), ("neighbor.ts", 1, 1)]);
}

#[test]
fn issue_162_bootstrap_inline_exit_individual_object_field_entry_proof_refusals() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(CALL);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, p, "main.ts");
    let edges = crate::angular_meta::bootstrap_references::project(&authority);
    let registrations = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::RegistersProvider)
        .cloned()
        .collect::<Vec<_>>();
    let roots = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
        .cloned()
        .collect::<Vec<_>>();
    assert!(!registrations.is_empty());
    assert_eq!(roots.len(), 1);
    let ExpressionValue::Call(_, args) = &authority.invocation_facets[0].expression.value else {
        panic!()
    };
    let object = &args[1];
    let ExpressionValue::Object(fields) = &object.value else {
        panic!()
    };
    let providers = &fields.iter().find(|(key, _)| key == "providers").unwrap().1;
    let group = authority
        .declaration_facets
        .iter()
        .find(|f| f.declaration.name == "GROUP")
        .unwrap();
    let array = group.initializer.as_ref().unwrap();
    let ExpressionValue::Array(entries) = &array.value else {
        panic!()
    };
    let entry = &entries[1];
    for mode in 0..9 {
        let mut bad = authority.clone();
        let (source, hash, start, end) = if mode < 3 {
            (
                &authority.source,
                &authority.source_hash,
                object.start,
                object.end,
            )
        } else if mode < 6 {
            (
                &authority.source,
                &authority.source_hash,
                providers.start,
                providers.end,
            )
        } else {
            (
                &group.declaration.source,
                &group.declaration.source_hash,
                entry.start,
                entry.end,
            )
        };
        let index = bad
            .demanded_occurrences
            .iter()
            .position(|v| {
                &v.source == source && &v.source_hash == hash && v.start == start && v.end == end
            })
            .unwrap();
        match mode % 3 {
            0 => {
                bad.demanded_occurrences.remove(index);
            }
            1 => {
                let duplicate = bad.demanded_occurrences[index].clone();
                bad.demanded_occurrences.push(duplicate);
            }
            _ => {
                let DemandedValue::Structure { children, .. } =
                    &mut bad.demanded_occurrences[index].value
                else {
                    panic!()
                };
                children[0].0 = "crossed-field".into();
            }
        }
        assert!(!bad.admits_relation_evidence(&registrations), "mode={mode}");
        assert!(
            bad.admits_relation_evidence(&roots),
            "independent root mode={mode}"
        );
    }
}

#[test]
fn issue_162_bootstrap_inline_exit_edited_caller_stale_restore_and_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(CALL);
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    capture(&state, p, "main.ts");
    facts(&state, p, &[("main.ts", 1, 1), ("neighbor.ts", 1, 1)]);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("main.ts")}),
    );
    drop(state);
    std::fs::write(
        p.join("main.ts"),
        format!("{PREFIX}launch(Root,{{providers:[]}});"),
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config);
    let rejected = raw(
        &state,
        p,
        "restore_context",
        json!({"filePath":p.join("main.ts")}),
    );
    assert!(
        rejected.get("error").is_some() || rejected["result"]["isError"] == true,
        "{rejected}"
    );
    assert!(
        state
            .alias_for_path(&p.join("main.ts").to_string_lossy())
            .is_none()
    );
    facts(&state, p, &[("neighbor.ts", 1, 1)]);
    std::fs::write(p.join("main.ts"), format!("{PREFIX}{CALL}")).unwrap();
    facts(&state, p, &[("main.ts", 1, 1), ("neighbor.ts", 1, 1)]);
}
