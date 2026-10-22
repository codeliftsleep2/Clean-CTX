use crate::compression::Fidelity;
use crate::ir::IRCompiler;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};

fn compile(source: &str) -> Vec<SemanticEdge> {
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler
        .compile_focused(
            source,
            "legacy.actions.ts",
            Some("legacy.actions.ts"),
            crate::compression::language::safe_typescript_language().unwrap(),
            crate::queries::TS_QUERY,
            Fidelity::High,
            None,
            None,
        )
        .expect("fixture compiles");
    compiler.semantic_edges
}

fn source(import: &str, parameter: &str, discriminant: &str, case: &str) -> String {
    format!(
        r#"
{import}
export const SET_AUDIT = '[Audit] Set';
export const APP_TITLE = 'Audit';
export function auditReducer(state: any, {parameter}) {{
  switch ({discriminant}) {{
    case {case}: return state;
    default: return state;
  }}
}}
"#
    )
}

fn action_declarations(edges: &[SemanticEdge]) -> Vec<&str> {
    edges
        .iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::Defines
                && edge.subject.domain == "ngrx"
                && edge.subject.entity_type == "Action"
        })
        .map(|edge| edge.subject.name.as_str())
        .collect()
}

fn transitions(edges: &[SemanticEdge]) -> Vec<(&str, &str)> {
    edges
        .iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::TriggersReducer
                && edge.subject.domain == "ngrx"
                && edge.subject.entity_type == "Action"
                && edge.object.domain == "ngrx"
                && edge.object.entity_type == "Reducer"
        })
        .map(|edge| (edge.subject.name.as_str(), edge.object.name.as_str()))
        .collect()
}

#[test]
fn issue_162_private_value_carriers_do_not_establish_claimed_action_transitions() {
    for (import, claimed) in [
        ("import { Action } from '@ngrx/store';", true),
        ("import { Action } from './fake-action';", false),
    ] {
        let edges = compile(&source(
            import,
            "action: Action",
            "action.type",
            "SET_AUDIT",
        ));
        let carriers: Vec<_> = edges
            .iter()
            .filter(|edge| {
                edge.layer == crate::angular_meta::ngrx::object_actions::SWITCH_VALUE_LAYER
                    && edge.relation == SemanticRelation::TriggersReducer
            })
            .collect();
        assert_eq!(
            carriers.len(),
            1,
            "private source-local carrier remains available"
        );
        assert_ne!(carriers[0].subject.domain, "ngrx");
        assert_eq!(transitions(&edges).len(), usize::from(claimed));
        assert_eq!(action_declarations(&edges).len(), usize::from(claimed));
    }
}

#[test]
fn standalone_discriminant_declaration_and_reducer_relationship_are_authoritative() {
    let edges = compile(&source(
        "import { Action } from '@ngrx/store';",
        "action: Action",
        "action.type",
        "SET_AUDIT",
    ));
    assert_eq!(action_declarations(&edges), ["SET_AUDIT"]);
    assert_eq!(transitions(&edges), [("SET_AUDIT", "auditReducer")]);
}

#[test]
fn same_value_constants_and_lexical_shadowing_preserve_exact_identity() {
    let valid = source(
        "import { Action } from '@ngrx/store';",
        "action: Action",
        "action.type",
        "SET_AUDIT",
    )
    .replace(
        "export const APP_TITLE = 'Audit';",
        "export const ALSO_SET_AUDIT = '[Audit] Set';",
    );
    let edges = compile(&valid);
    assert_eq!(action_declarations(&edges), ["SET_AUDIT"]);
    assert_eq!(transitions(&edges), [("SET_AUDIT", "auditReducer")]);

    let shadowed = valid.replace(
        "export function auditReducer(state: any, action: Action) {",
        "export function auditReducer(state: any, action: Action) { const SET_AUDIT = '[Audit] Inner';",
    );
    let edges = compile(&shadowed);
    assert!(action_declarations(&edges).is_empty());
    assert!(transitions(&edges).is_empty());
}

#[test]
fn constant_and_class_action_identities_coexist_without_merging() {
    let candidate = source(
        "import { Action } from '@ngrx/store';",
        "action: Action",
        "action.type",
        "SET_AUDIT",
    ) + "export class SetAudit implements Action { readonly type = SET_AUDIT; }";
    let edges = compile(&candidate);
    let mut declarations = action_declarations(&edges);
    declarations.sort_unstable();
    assert_eq!(declarations, ["SET_AUDIT", "SetAudit"]);
    let mut reducer_edges = transitions(&edges);
    reducer_edges.sort_unstable();
    assert_eq!(
        reducer_edges,
        [("SET_AUDIT", "auditReducer"), ("SetAudit", "auditReducer")]
    );
}

#[test]
fn strings_and_unproven_reducer_shapes_do_not_supply_authority() {
    let cases = [
        source(
            "import { Action } from './fake-action';",
            "action: Action",
            "action.type",
            "SET_AUDIT",
        ),
        source(
            "interface Action { type: string; }",
            "action: Action",
            "action.type",
            "SET_AUDIT",
        ),
        source(
            "import { Action } from '@ngrx/store';",
            "value: any",
            "value.type",
            "SET_AUDIT",
        ),
        source(
            "import { Action } from '@ngrx/store';",
            "action: Action",
            "action.kind",
            "SET_AUDIT",
        ),
        source(
            "import { Action } from '@ngrx/store';",
            "input: Action",
            "action.type",
            "SET_AUDIT",
        )
        .replace(
            "export function auditReducer(state: any, input: Action) {",
            "export function auditReducer(state: any, input: Action) { const action = state;",
        ),
    ];
    for candidate in cases {
        let edges = compile(&candidate);
        assert!(action_declarations(&edges).is_empty(), "{candidate}");
        assert!(transitions(&edges).is_empty(), "{candidate}");
    }
}
