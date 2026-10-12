//! Structural prerequisite: direct type reference and declaration source roles.
use crate::compression::Fidelity;
use crate::compression::capture_pipeline::run_capture_pipeline_nodes;
use crate::compression::language::safe_typescript_language;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};

pub(super) const PRIMARY: &str = "import { Action } from '@ngrx/store'; export const SET_AUDIT = '[Audit] Set Audit'; export class AuditAction implements Action { readonly type = SET_AUDIT; }";

#[test]
fn ngrx_action_type_binding_primary_source_roles() {
    // Independent structural oracle, before asking for new NgRx evidence.
    // These declarations are program children, the field is its class's own
    // body child, and there is no second declaration or enclosing local scope.
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&safe_typescript_language().unwrap()).unwrap();
    let tree = parser.parse(PRIMARY, None).unwrap();
    let root = tree.root_node();
    assert!(!root.has_error());
    let mut cursor = root.walk();
    let children: Vec<_> = root.named_children(&mut cursor).collect();
    assert_eq!(children.len(), 3);
    let declaration = children[1].child_by_field_name("declaration").unwrap();
    assert_eq!(declaration.kind(), "lexical_declaration");
    assert_eq!(declaration.child_by_field_name("kind").unwrap().kind(), "const");
    let constant = declaration.named_child(0).unwrap();
    let constant_name = constant.child_by_field_name("name").unwrap();
    assert_eq!(constant_name.kind(), "identifier");
    assert_eq!(constant_name.utf8_text(PRIMARY.as_bytes()).unwrap(), "SET_AUDIT");
    let owner = children[2].child_by_field_name("declaration").unwrap();
    let body = owner.child_by_field_name("body").unwrap();
    let field = body.named_child(0).unwrap();
    assert_eq!(field.kind(), "public_field_definition");
    assert_eq!(field.child_by_field_name("name").unwrap().utf8_text(PRIMARY.as_bytes()).unwrap(), "type");
    let reference = field.child_by_field_name("value").unwrap();
    assert_eq!(reference.kind(), "identifier");
    assert_eq!(reference.utf8_text(PRIMARY.as_bytes()).unwrap(), "SET_AUDIT");
    // #140a independently admits the class, not the constant.
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut state = PassContext::new(PRIMARY.into(), "source-alias".into(), Fidelity::High);
    state.language = Some(language);
    state.query_string = query.into();
    CoreIRPass::new().run(&mut state).unwrap();
    assert_eq!(state.ngrx_class_actions.len(), 1);
    assert_eq!(state.ngrx_class_actions[0].name, "AuditAction");
    assert_eq!(state.ngrx_class_actions[0].class_span.start_byte(), owner.start_byte());

    // Missing NgRx structural evidence is the intended behavioral RED.
    let captures = run_capture_pipeline_nodes(
        safe_typescript_language().unwrap(), super::QUERY, PRIMARY,
        Fidelity::High, |_, raw, _| Some(raw.into()),
    ).unwrap();
    let refs: Vec<_> = captures.iter().filter(|c| c.name == "ngrx.type.reference").collect();
    assert_eq!(refs.len(), 1, "NgRx evidence must retain the direct Action type identifier role");
    let reference_capture = refs[0];
    assert_eq!((reference_capture.start_byte, reference_capture.end_byte), (reference.start_byte(), reference.end_byte()));
    let same = |name: &str| captures.iter().find(|c| c.name == name && c.match_index == reference_capture.match_index).unwrap();
    assert_eq!((same("ngrx.type.owner").start_byte, same("ngrx.type.owner").end_byte), (owner.start_byte(), owner.end_byte()));
    assert_eq!((same("ngrx.type.field").start_byte, same("ngrx.type.field").end_byte), (field.start_byte(), field.end_byte()));
    let targets: Vec<_> = captures.iter().filter(|c| c.name == "ngrx.type.const_name").collect();
    assert_eq!(targets.len(), 1);
    assert_eq!((targets[0].start_byte, targets[0].end_byte), (constant_name.start_byte(), constant_name.end_byte()));
}
