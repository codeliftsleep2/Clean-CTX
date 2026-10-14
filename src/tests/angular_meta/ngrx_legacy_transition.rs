use crate::compression::Fidelity;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};

fn compile(source: &str, file: &str) -> PassContext {
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut state = PassContext::new(source.into(), "alias".into(), Fidelity::High);
    state.canonical_path = Some(file.into());
    state.language = Some(language);
    state.query_string = query.into();
    crate::compression::capture_pipeline::reset_parse_count();
    CoreIRPass::new().run(&mut state).unwrap();
    assert_eq!(crate::compression::capture_pipeline::parse_count(), 1);
    assert!(
        state
            .captures
            .iter()
            .all(|c| !c.name.starts_with("ngrx.transition."))
    );
    state
}

fn source(cases: &str, other: bool) -> String {
    format!(
        "import {{Action}} from '@ngrx/store'; const SET_AUDIT='same'; const OTHER_AUDIT='same'; const UNUSED_AUDIT='unused'; class AuditAction implements Action {{readonly type=SET_AUDIT;}} {} export function auditReducer(state: any, action: Action) {{switch(action.type) {{{cases} default: return state;}}}}",
        if other {
            "class OtherAction implements Action {readonly type=OTHER_AUDIT;}"
        } else {
            ""
        }
    )
}

fn identities(state: &PassContext) -> Vec<(&str, &str)> {
    state
        .ngrx_legacy_transitions
        .iter()
        .map(|t| (t.action.name.as_str(), t.reducer.name.as_str()))
        .collect()
}

#[test]
fn ngrx_legacy_transition_identity_and_case_falsifiers() {
    for (cases, expected) in [
        ("case SET_AUDIT: return state;", vec!["AuditAction"]),
        ("case OTHER_AUDIT: return state;", vec!["OtherAction"]),
        (
            "case SET_AUDIT: return state; case OTHER_AUDIT: return state;",
            vec!["AuditAction", "OtherAction"],
        ),
        ("case 'same': return state;", vec![]),
        ("case UNUSED_AUDIT: return state;", vec![]),
        ("case wrapper(SET_AUDIT): return state;", vec![]),
        ("case Actions.SET_AUDIT: return state;", vec![]),
        ("case flag ? SET_AUDIT : OTHER_AUDIT: return state;", vec![]),
        ("", vec![]),
    ] {
        let text = source(cases, true);
        let state = compile(&text, "/a/actions.ts");
        assert_eq!(state.ngrx_class_actions.len(), 2);
        assert_eq!(state.ngrx_action_type_bindings.len(), 2);
        assert_eq!(
            identities(&state),
            expected
                .iter()
                .map(|name| (*name, "auditReducer"))
                .collect::<Vec<_>>(),
            "{cases}"
        );
        for transition in &state.ngrx_legacy_transitions {
            let selected = state
                .ngrx_action_type_bindings
                .iter()
                .find(|b| b.action_name == transition.action.name)
                .unwrap();
            assert_eq!(transition.declaration_span, selected.declaration_span);
            assert_eq!(transition.action.file.as_deref(), Some("/a/actions.ts"));
            assert_eq!(transition.reducer.file.as_deref(), Some("/a/actions.ts"));
            assert!(
                text[transition.case_span.start_byte()..transition.case_span.end_byte()]
                    .starts_with("case ")
            );
            assert_eq!(
                transition.edge().relation,
                crate::layers::meta::semantic::SemanticRelation::TriggersReducer
            );
        }
    }
    assert!(
        compile(
            &source("case OTHER_AUDIT: return state;", false),
            "/a/actions.ts"
        )
        .ngrx_legacy_transitions
        .is_empty()
    );
}

#[test]
fn ngrx_legacy_transition_receiver_owner_and_shadow_refusals() {
    let primary = source("case SET_AUDIT: return state;", false);
    for text in [
        primary.replace("switch(action.type)", "switch(state.mode)"),
        primary.replace("switch(action.type)", "switch(action.other)"),
        primary.replace("switch(action.type)", "switch(action?.type)"),
        primary.replace("action: Action", "action: any"),
        primary.replace("action: Action", "action"),
        primary.replace("function auditReducer(", "function auditReducer<Action>("),
        primary.replace(
            "{switch(action.type)",
            "{let action = {type:'other'}; switch(action.type)",
        ),
        primary.replace(
            "{switch(action.type)",
            "{const SET_AUDIT = 'same'; switch(action.type)",
        ),
        primary.replace(
            "export function auditReducer",
            "function make() { function auditReducer",
        ) + "}",
        primary.replace("{switch(action.type)", "{if (true) { switch(action.type)") + "}",
        primary.replace("readonly type=SET_AUDIT", "readonly type='same'"),
        primary.replace(
            "readonly type=SET_AUDIT",
            "readonly type=wrapper(SET_AUDIT)",
        ),
        primary.replace("action: Action)", "action: Action, extra: any)"),
    ] {
        assert!(
            compile(&text, "/a/actions.ts")
                .ngrx_legacy_transitions
                .is_empty(),
            "{text}"
        );
    }
    let unrelated = primary
        .replace("function auditReducer", "function arbitraryName")
        .replace("action: Action", "message: Action")
        .replace("switch(action.type)", "switch(message.type)");
    assert_eq!(
        identities(&compile(&unrelated, "/a/actions.ts")),
        [("AuditAction", "arbitraryName")]
    );
}

#[test]
fn ngrx_legacy_transition_shared_discriminant_and_duplicate_cases() {
    let primary = source("case SET_AUDIT: return state;", false);
    let shared = primary + " class OtherAction implements Action {readonly type=SET_AUDIT;}";
    let state = compile(&shared, "/a/actions.ts");
    assert_eq!(state.ngrx_action_type_bindings.len(), 2);
    assert!(
        state.ngrx_legacy_transitions.is_empty(),
        "ambiguous inverse Action mapping must refuse"
    );
    let duplicate = compile(
        &source(
            "case SET_AUDIT: return state; case SET_AUDIT: return state;",
            false,
        ),
        "/a/actions.ts",
    );
    assert_eq!(duplicate.ngrx_legacy_transitions.len(), 2);
    assert_ne!(
        duplicate.ngrx_legacy_transitions[0].case_span,
        duplicate.ngrx_legacy_transitions[1].case_span
    );
}

#[test]
fn ngrx_legacy_transition_compilation_identity_lifecycle() {
    for (cases, expected) in [
        ("case SET_AUDIT: return state;", vec!["AuditAction"]),
        ("case OTHER_AUDIT: return state;", vec!["OtherAction"]),
        ("case 'same': return state;", vec![]),
        ("", vec![]),
    ] {
        let state = compile(&source(cases, true), "/a/actions.ts");
        assert_eq!(
            identities(&state)
                .iter()
                .map(|(a, _)| *a)
                .collect::<Vec<_>>(),
            expected
        );
    }
    let mut state = compile(
        &source("case SET_AUDIT: return state;", false),
        "/a/actions.ts",
    );
    let text = state.source.clone();
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let captures = crate::compression::capture_pipeline::run_capture_pipeline_nodes(
        language,
        &format!(
            "{query}{}{}{}",
            crate::queries::TS_CALL_QUERY,
            crate::angular_meta::ngrx::legacy::QUERY,
            super::QUERY
        ),
        &text,
        Fidelity::High,
        |_, raw, _| Some(raw.into()),
    )
    .unwrap();
    state.ngrx_action_type_bindings[0].file = "/b/actions.ts".into();
    assert!(
        super::collect(
            &text,
            Some("/a/actions.ts"),
            &captures,
            &state.ngrx_action_type_bindings
        )
        .is_empty()
    );
    state.ngrx_action_type_bindings[0].file = "/a/actions.ts".into();
    state.ngrx_action_type_bindings[0].source_hash = "stale".into();
    assert!(
        super::collect(
            &text,
            Some("/a/actions.ts"),
            &captures,
            &state.ngrx_action_type_bindings
        )
        .is_empty()
    );
}

#[test]
fn ngrx_legacy_transition_for_in_shadow_refuses_outer_identity() {
    let text = source("case SET_AUDIT: return state;", false).replace(
        "{switch(action.type)",
        "{for(var SET_AUDIT of values) {} switch(action.type)",
    );
    let state = compile(&text, "/a/actions.ts");
    assert_eq!(state.ngrx_class_actions.len(), 1);
    assert_eq!(
        state.ngrx_action_type_bindings.len(),
        1,
        "The class initializer remains bound to its outer program const"
    );
    assert!(
        state.ngrx_legacy_transitions.is_empty(),
        "Hoisted loop binding must not let the case resolve to the outer const"
    );
}
