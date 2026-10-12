use crate::compression::Fidelity;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};

fn run(source: &str, file: Option<&str>) -> PassContext {
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut state = PassContext::new(source.into(), "alias".into(), Fidelity::High);
    state.canonical_path = file.map(Into::into);
    state.language = Some(language);
    state.query_string = query.into();
    crate::compression::capture_pipeline::reset_parse_count();
    CoreIRPass::new().run(&mut state).unwrap();
    assert_eq!(crate::compression::capture_pipeline::parse_count(), 1);
    assert!(
        state
            .captures
            .iter()
            .all(|c| !c.name.starts_with("ngrx.type."))
    );
    state
}

fn source(expression: &str, extra: &str) -> String {
    format!(
        "import {{ Action }} from '@ngrx/store'; export const SET_AUDIT = '[Audit] Same'; export const OTHER_AUDIT = '[Audit] Same'; {extra} export class AuditAction implements Action {{readonly type = {expression};}}"
    )
}

#[test]
fn ngrx_action_type_binding_exact_declaration_and_value_independence() {
    for target in ["SET_AUDIT", "OTHER_AUDIT"] {
        let text = source(target, "export const SET_THEME = 'dark';");
        let state = run(&text, Some("/fixture/actions.ts"));
        assert_eq!(state.ngrx_class_actions.len(), 1);
        assert_eq!(state.ngrx_action_type_bindings.len(), 1);
        let binding = &state.ngrx_action_type_bindings[0];
        assert_eq!(binding.action_name, "AuditAction");
        assert_eq!(binding.file, "/fixture/actions.ts");
        assert_eq!(
            binding.source_hash,
            crate::cache::content_hash(text.as_bytes())
        );
        assert_eq!(binding.declaration_name, target);
        assert_eq!(
            &text[binding.reference_span.start_byte()..binding.reference_span.end_byte()],
            target
        );
        assert_eq!(
            &text[binding.declaration_name_span.start_byte()
                ..binding.declaration_name_span.end_byte()],
            target
        );
        assert_eq!(
            &text[binding.declaration_span.start_byte()..binding.declaration_span.end_byte()],
            format!("{target} = '[Audit] Same'")
        );
        assert_ne!(binding.reference_span, binding.declaration_name_span);
        assert_ne!(binding.action_span, binding.declaration_span);
        assert_eq!(binding.action_span, state.ngrx_class_actions[0].class_span);
        let shape = super::super::legacy::include(None, &state.ngrx_class_actions).unwrap();
        assert_eq!(shape.to_ngrx_semantic_edges().len(), 1);
        assert_eq!(shape.class_actions[0].name, "AuditAction");
        assert!(shape.actions.is_empty(), "constant is not another Action");
    }
}

#[test]
fn ngrx_action_type_binding_direct_expression_falsifiers() {
    for expression in [
        "'[Audit] Same'",
        "wrapper(SET_AUDIT)",
        "condition ? SET_AUDIT : OTHER_AUDIT",
        "SET_AUDIT || OTHER_AUDIT",
        "`${SET_AUDIT}`",
        "(SET_AUDIT)",
        "Actions.SET_AUDIT",
        "missing",
    ] {
        let state = run(&source(expression, ""), Some("/fixture/actions.ts"));
        assert_eq!(
            state.ngrx_class_actions.len(),
            1,
            "Action survives {expression}"
        );
        assert!(state.ngrx_action_type_bindings.is_empty(), "{expression}");
    }
}

#[test]
fn ngrx_action_type_binding_shadowing_and_unsupported_scope_refusal() {
    for extra in [
        "function unrelated() { const SET_AUDIT = 'inner'; }",
        "function unrelated(SET_AUDIT: string) {}",
        r"class S\u0045T_AUDIT {}",
        "const unrelated = SET_AUDIT => SET_AUDIT;",
        "import SET_AUDIT = require('other');",
        "import SET_AUDIT = Other.value;",
        "function unrelated({SET_AUDIT}: any) {}",
        "const {SET_AUDIT: ignored} = object;",
        "import { something as SET_AUDIT } from './other';",
        "import SET_AUDIT from './other';",
        "const SET_AUDIT = 'duplicate';",
        "namespace Other { export const SET_AUDIT = 'inner'; }",
        "try {} catch (SET_AUDIT) {}",
    ] {
        assert!(
            run(&source("SET_AUDIT", extra), Some("/fixture/actions.ts"))
                .ngrx_action_type_bindings
                .is_empty(),
            "{extra}"
        );
    }
    for text in [
        "import {Action} from '@ngrx/store'; const SET_AUDIT='outer'; class SET_AUDIT implements Action {type=SET_AUDIT;}",
        "import {Action} from '@ngrx/store'; const SET_AUDIT='outer'; class AuditAction<SET_AUDIT> implements Action {type=SET_AUDIT;}",
        "import {Action} from '@ngrx/store'; const SET_AUDIT='outer'; function make() {const SET_AUDIT='inner'; class AuditAction implements Action {type=SET_AUDIT;}}",
        "import {Action} from '@ngrx/store'; const SET_AUDIT='outer'; class AuditAction implements Action {static type=SET_AUDIT;}",
        "import {Action} from '@ngrx/store'; const SET_AUDIT='outer'; class AuditAction implements Action {type=SET_AUDIT; type='other';}",
        "import {Action} from '@ngrx/store'; const SET_AUDIT='outer'; class AuditAction implements Action {type=SET_AUDIT; ['type']='other';}",
        "import {Action} from '@ngrx/store'; const SET_AUDIT='outer'; class AuditAction implements Action {type=SET_AUDIT; constructor(public type: string) {}}",
        "import {Action} from '@ngrx/store'; let SET_AUDIT='outer'; class AuditAction implements Action {type=SET_AUDIT;}",
        "const SET_AUDIT='outer'; class AuditAction {type=SET_AUDIT;}",
        "import {Action} from '@ngrx/store'; const SET_AUDIT='outer'; class AuditAction implements Action {type=SET_AUDIT;}",
    ] {
        assert!(
            run(text, Some("/fixture/actions.ts"))
                .ngrx_action_type_bindings
                .is_empty(),
            "{text}"
        );
    }
}

#[test]
fn ngrx_action_type_binding_file_identity_and_recompile_lifecycle() {
    let text = source("SET_AUDIT", "");
    assert!(run(&text, None).ngrx_action_type_bindings.is_empty());
    let a = run(&text, Some("/a/actions.ts"))
        .ngrx_action_type_bindings
        .remove(0);
    let b = run(&text, Some("/b/actions.ts"))
        .ngrx_action_type_bindings
        .remove(0);
    assert_ne!(a, b);
    let changed = run(&source("OTHER_AUDIT", ""), Some("/a/actions.ts"));
    assert_eq!(
        changed.ngrx_action_type_bindings[0].declaration_name,
        "OTHER_AUDIT"
    );
    assert_ne!(
        a.source_hash,
        changed.ngrx_action_type_bindings[0].source_hash
    );
    assert!(
        run(&source("'literal'", ""), Some("/a/actions.ts"))
            .ngrx_action_type_bindings
            .is_empty()
    );
    assert!(
        run(
            "import {Action} from '@ngrx/store'; const SET_AUDIT='outer';",
            Some("/a/actions.ts")
        )
        .ngrx_action_type_bindings
        .is_empty()
    );
    assert!(run("import {Action} from '@ngrx/store'; class AuditAction implements Action {type=SET_AUDIT;}", Some("/a/actions.ts")).ngrx_action_type_bindings.is_empty());
}

#[test]
fn ngrx_action_type_binding_bounded_positive_syntax_matrix() {
    for text in [
        "import {Action} from '@ngrx/store'; const SET_AUDIT='value'; class AuditAction implements Action {readonly type=SET_AUDIT;}",
        "import {Action} from '@ngrx/store'; export const SET_AUDIT='value', OTHER_AUDIT='value'; export class AuditAction implements Action { public readonly type: string = SET_AUDIT;}",
        "import type {Action} from '@ngrx/store'; const SET_AUDIT='value'; abstract class AuditAction implements Action {readonly type = SET_AUDIT;}",
        "import {Action} from '@ngrx/store'; const clé='value'; class AuditAction implements Action {readonly type = clé;}",
        "import {Action} from '@ngrx/store'; class AuditAction implements Action {readonly type = SET_AUDIT;} const SET_AUDIT='value';",
    ] {
        let state = run(text, Some("/fixture/actions.ts"));
        assert_eq!(state.ngrx_action_type_bindings.len(), 1, "{text}");
        let binding = &state.ngrx_action_type_bindings[0];
        assert_eq!(binding.action_name, "AuditAction");
        assert_eq!(
            binding.declaration_name,
            if text.contains("clé") {
                "clé"
            } else {
                "SET_AUDIT"
            }
        );
    }
}

#[test]
fn ngrx_action_type_binding_shared_constant_does_not_merge_action_identities() {
    let text = "import {Action} from '@ngrx/store'; const SET_AUDIT='value'; class First implements Action {readonly type=SET_AUDIT;} class Second implements Action {readonly type=SET_AUDIT;}";
    let state = run(text, Some("/fixture/actions.ts"));
    assert_eq!(state.ngrx_action_type_bindings.len(), 2);
    let first = &state.ngrx_action_type_bindings[0];
    let second = &state.ngrx_action_type_bindings[1];
    assert_eq!(first.action_name, "First");
    assert_eq!(second.action_name, "Second");
    assert_ne!(first.action_span, second.action_span);
    assert_eq!(first.declaration_span, second.declaration_span);
    assert_eq!(first.declaration_name, "SET_AUDIT");
    assert_eq!(second.declaration_name, "SET_AUDIT");
}
