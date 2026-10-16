use crate::angular_meta::ngrx::legacy::ClassActionEvidence;
use crate::compression::Fidelity;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};

fn declarations(source: &str) -> Vec<ClassActionEvidence> {
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut state = PassContext::new(source.into(), "legacy-source".into(), Fidelity::High);
    state.language = Some(language);
    state.query_string = query.into();
    CoreIRPass::new().run(&mut state).unwrap();
    assert!(
        state
            .captures
            .iter()
            .all(|capture| !capture.name.starts_with("ngrx.legacy."))
    );
    state.ngrx_class_actions
}

#[test]
fn ngrx_legacy_declaration_structured_import_and_class_identity() {
    let source = "import { Action } from '@ngrx/store'; export class NoSuffixRecord implements Other, Action { readonly type = '[Audit] Set Audit'; }";
    let found = declarations(source);
    assert_eq!(found.len(), 1);
    let declaration = &found[0];
    assert_eq!(declaration.name, "NoSuffixRecord");
    assert_eq!(
        &source[declaration.import_span.start_byte()..declaration.import_span.end_byte()],
        "import { Action } from '@ngrx/store';"
    );
    assert_eq!(
        &source[declaration.implements_span.start_byte()..declaration.implements_span.end_byte()],
        "Action"
    );
    assert_eq!(
        &source[declaration.class_span.start_byte()..declaration.class_span.end_byte()],
        "class NoSuffixRecord implements Other, Action { readonly type = '[Audit] Set Audit'; }"
    );
}

#[test]
fn ngrx_legacy_declaration_unsupported_bindings_do_not_guess() {
    for source in [
        "import { Action } from '@ngrx/store'; class PretendAction {}",
        "import { Store } from '@ngrx/store'; interface Action {} class Record implements Action {}",
        "import { Store } from '@ngrx/store'; import { Action } from './local'; class Record implements Action {}",
        "import { Action as StoreAction } from '@ngrx/store'; class Record implements StoreAction {}",
        "import * as Store from '@ngrx/store'; class Record implements Store.Action {}",
        "import { Action } from '@ngrx/store'; class Record<Action> implements Action {}",
        "import { Action } from '@ngrx/store'; function scope() { class Record implements Action {} }",
        "import { Action } from '@ngrx/store'; const text = 'class FakeAction implements Action {}';",
        "import { Action } from '@ngrx/store'; interface Action {} class Record implements Action {}",
        "import { Action } from '@ngrx/store'; import { Something as Action } from './local'; class Record implements Action {}",
        "import { Action } from '@ngrx/store'; import { Action } from './other'; class Record implements Action {}",
        "import { Store } from '@ngrx/store'; export const SET_AUDIT = '[Audit] Set Audit';",
        "import { Store } from '@ngrx/store'; import { Action } from '\\n@ngrx/store'; class Record implements Action {}",
    ] {
        assert!(
            declarations(source).is_empty(),
            "unproven legacy declaration: {source}"
        );
    }
}

#[test]
fn ngrx_legacy_declaration_producer_marker_and_publication_are_distinct_from_modern() {
    let source = "import { Action } from '@ngrx/store'; class First implements Action {type = 'first';} abstract class Second implements Action {abstract type: string;}";
    let evidence = declarations(source);
    assert_eq!(
        evidence
            .iter()
            .map(|decl| decl.name.as_str())
            .collect::<Vec<_>>(),
        ["First", "Second"]
    );
    let shape = super::include(None, &evidence).unwrap();
    assert!(
        shape.actions.is_empty(),
        "legacy must not pretend createAction"
    );
    assert_eq!(shape.class_actions.len(), 2);
    let edges = shape.to_ngrx_semantic_edges();
    assert_eq!(edges.len(), 2);
    for edge in edges {
        assert_eq!(
            edge.relation,
            crate::layers::meta::semantic::SemanticRelation::Defines
        );
        assert_eq!(edge.subject, edge.object);
        assert_eq!(edge.subject.domain, "ngrx");
        assert_eq!(edge.subject.entity_type, "Action");
    }
    let rendered = shape.render(Fidelity::High);
    assert!(rendered.contains("Φaction-class:First"));
    let expanded = crate::angular_meta::ngrx::expand_phi_in_line(&rendered);
    assert!(expanded.contains("NgRx Action class First"));
    assert!(!expanded.contains("createAction"));
}
