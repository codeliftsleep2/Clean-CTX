use crate::angular_meta::ngrx::{direct_imports, extract_ngrx_shape};
use crate::compression::Fidelity;

const SOURCE: &str = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { SET_AUDIT } from './legacy.actions';
@Component({selector: 'audit-dispatch', template: ''})
export class DispatchProbe {
  constructor(private store: Store<unknown>) {}
  send() { this.store.dispatch({type: SET_AUDIT, payload: 1}); }
}
"#;

fn candidates(source: &str) -> Vec<crate::layers::meta::semantic::SemanticEdge> {
    let shape = extract_ngrx_shape(source, Fidelity::High).expect("NgRx shape");
    direct_imports::candidates("/fixture/dispatch.component.ts", source, &shape)
}

#[test]
fn direct_unaliased_named_import_produces_only_an_unresolved_workspace_carrier() {
    let edges = candidates(SOURCE);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].subject.name, "DispatchProbe");
    assert_eq!(edges[0].object.name, "SET_AUDIT");
    assert_eq!(edges[0].object.file.as_deref(), Some("./legacy.actions"));
    assert_eq!(edges[0].layer, direct_imports::REFINEMENT_LAYER);
}

#[test]
fn shadow_alias_namespace_and_nonrelative_imports_refuse_local_authority() {
    for source in [
        SOURCE.replace("send() {", "send() { const SET_AUDIT = other;"),
        SOURCE
            .replace("{ SET_AUDIT }", "{ SET_AUDIT as SET }")
            .replace("type: SET_AUDIT", "type: SET"),
        SOURCE
            .replace("{ SET_AUDIT }", "* as Actions")
            .replace("type: SET_AUDIT", "type: Actions.SET_AUDIT"),
        SOURCE.replace("'./legacy.actions'", "'@app/legacy.actions'"),
        SOURCE.replace("import { SET_AUDIT }", "import type { SET_AUDIT }"),
    ] {
        assert!(candidates(&source).is_empty(), "{source}");
    }
}
