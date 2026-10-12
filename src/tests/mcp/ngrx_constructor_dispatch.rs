use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value,json};
use crate::ir::pipeline::{PassContext,CoreIRPass,IRPass};
use crate::compression::Fidelity;

pub(super) const PRIMARY: &str = "import {Component} from '@angular/core'; import {Action, Store} from '@ngrx/store'; export class AuditAction implements Action {readonly type='audit';} @Component({selector:'audit-dispatch', template:''}) export class Dispatcher {constructor(private store: Store<unknown>) {} run() {this.store.dispatch(new AuditAction());}}";
pub(super) fn call(state: &crate::mcp::McpState, tool:&str, args:Value)->Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(140),tool,&json!({"arguments":args}),state);
    let response=crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(),"{response}");
    assert_ne!(response["result"]["isError"],true,"{response}");
    response["result"].clone()
}
#[test]
fn ngrx_constructor_dispatch_public_primary() {
    let _serial=crate::protocol::handler_response_serial();
    let root=tempfile::tempdir().unwrap(); let path=root.path().join("dispatch.ts");
    let source=format!("{PRIMARY}\n{}","// economics padding\n".repeat(100));
    crate::edit::apply::verify_syntax(&source,"ts").unwrap();
    let (language,query)=crate::compression::language::language_for_extension("ts").unwrap();
    let mut ctx=PassContext::new(source.clone(),"alias".into(),Fidelity::High);
    ctx.canonical_path=Some(path.to_string_lossy().into_owned());ctx.language=Some(language.clone());ctx.query_string=query.into();
    CoreIRPass::new().run(&mut ctx).unwrap();
    assert_eq!(ctx.ngrx_class_actions.len(),1);assert_eq!(ctx.ngrx_class_actions[0].name,"AuditAction");
    // Independent closed-scope AST oracle: direct constructor argument and a
    // unique program Action class; exact owner has the NgRx Store parameter property.
    let mut parser=tree_sitter::Parser::new();parser.set_language(&language).unwrap();
    let tree=parser.parse(&source,None).unwrap();let program=tree.root_node();let mut cursor=program.walk();
    let declarations:Vec<_>=program.named_children(&mut cursor).filter_map(|n| n.child_by_field_name("declaration")).collect();
    assert_eq!(declarations.len(),2);
    let action=declarations[0];assert_eq!((action.start_byte(),action.end_byte()),(ctx.ngrx_class_actions[0].class_span.start_byte(),ctx.ngrx_class_actions[0].class_span.end_byte()));
    let owner=declarations[1];assert_eq!(owner.child_by_field_name("name").unwrap().utf8_text(source.as_bytes()).unwrap(),"Dispatcher");
    let body=owner.child_by_field_name("body").unwrap();let constructor=body.named_child(0).unwrap();
    let parameter=constructor.child_by_field_name("parameters").unwrap().named_child(0).unwrap();
    assert_eq!(parameter.child_by_field_name("pattern").unwrap().utf8_text(source.as_bytes()).unwrap(),"store");
    assert_eq!(parameter.child_by_field_name("type").unwrap().named_child(0).unwrap().child_by_field_name("name").unwrap().utf8_text(source.as_bytes()).unwrap(),"Store");
    let method=body.named_child(1).unwrap();let statement=method.child_by_field_name("body").unwrap().named_child(0).unwrap();let dispatch=statement.named_child(0).unwrap();
    let callee=dispatch.child_by_field_name("function").unwrap();assert_eq!(callee.child_by_field_name("property").unwrap().utf8_text(source.as_bytes()).unwrap(),"dispatch");
    let receiver=callee.child_by_field_name("object").unwrap();assert_eq!(receiver.child_by_field_name("object").unwrap().kind(),"this");assert_eq!(receiver.child_by_field_name("property").unwrap().utf8_text(source.as_bytes()).unwrap(),"store");
    let value=dispatch.child_by_field_name("arguments").unwrap().named_child(0).unwrap();assert_eq!(value.kind(),"new_expression");
    let target=value.child_by_field_name("constructor").unwrap();assert_eq!(target.kind(),"identifier");assert_eq!(target.utf8_text(source.as_bytes()).unwrap(),action.child_by_field_name("name").unwrap().utf8_text(source.as_bytes()).unwrap());
    // The only AuditAction value declaration is the program class; this method
    // has no parameter/local binding, nested scope or imported constructor alias.
    std::fs::write(&path,&source).unwrap();let mut config=crate::tests::test_config();config.additional_roots.push(root.path().to_string_lossy().into_owned());let state=crate::mcp::McpState::new(config);
    call(&state,"provide_code_context",json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":"high"}));
    let entities=call(&state,"workspace_query",json!({"workspaceRoot":root.path(),"type":"entities_in_file","file_path":path}));
    assert!(entities["structuredContent"]["entities"].as_array().unwrap().iter().any(|e| e["domain"]=="ngrx" && e["entity_type"]=="Action" && e["name"]=="AuditAction" && e["file"]==path.to_string_lossy().as_ref()));
    let answer=call(&state,"workspace_query",json!({"workspaceRoot":root.path(),"type":"forward_edges","domain":"angular","entity_type":"Component","name":"Dispatcher"}));
    let edges:Vec<_>=answer["structuredContent"]["edges"].as_array().unwrap().iter().filter(|e| e["relation"]=="Dispatches").collect();
    assert_eq!(edges.len(),1,"{answer}");let edge=edges[0];
    assert_eq!(edge["object"]["name"],"AuditAction","constructor target is not the admitted declaration: {answer}");
    for (role,domain,kind,name) in [("subject","angular","Component","Dispatcher"),("object","ngrx","Action","AuditAction")] {
        assert_eq!(edge[role]["domain"],domain);assert_eq!(edge[role]["entity_type"],kind);assert_eq!(edge[role]["name"],name);assert_eq!(edge[role]["file"],path.to_string_lossy().as_ref());
    }
}
