use super::ngrx_service_dispatch_tests::{PRIMARY, call};
use crate::compression::Fidelity;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};
use serde_json::{Value, json};

fn compile(source: &str) -> PassContext {
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut ctx = PassContext::new(source.into(), "alias".into(), Fidelity::High);
    ctx.canonical_path = Some("/fixture/service.ts".into());
    ctx.language = Some(language);
    ctx.query_string = query.into();
    crate::compression::capture_pipeline::reset_parse_count();
    CoreIRPass::new().run(&mut ctx).unwrap();
    assert_eq!(crate::compression::capture_pipeline::parse_count(), 1);
    assert!(
        ctx.captures
            .iter()
            .all(|c| !c.name.starts_with("ngrx.service."))
    );
    ctx
}
#[test]
fn ngrx_service_dispatch_structural_owner_falsifiers() {
    for source in [
        PRIMARY.to_owned(),
        PRIMARY.replace("run()", "save()"),
        PRIMARY.replace("AuditService", "NoSuffixOwner"),
    ] {
        let ctx = compile(&source);
        assert_eq!(ctx.ngrx_service_dispatches.len(), 1);
        let owner = &ctx.ngrx_service_dispatches[0];
        let class = ctx
            .angular_initializers
            .iter()
            .find(|c| c.class_span == owner.class_span)
            .unwrap();
        assert_eq!(owner.service, class.consumer);
        assert_eq!(owner.action, "AuditAction");
        assert_eq!(owner.file, "/fixture/service.ts");
        assert_eq!(
            owner.source_hash,
            crate::cache::content_hash(source.as_bytes())
        );
        let call = ctx
            .invocation_occurrences
            .iter()
            .find(|c| c.span().start_byte() == owner.start_byte)
            .unwrap();
        assert_eq!(
            call.declaration_context().unwrap().declaration(),
            owner.method_span
        );
    }
    for source in [
        PRIMARY.replace("@Injectable()", ""),
        PRIMARY.replace("@Injectable()", "@Component({selector:'x'})"),
        PRIMARY.replace("this.store.dispatch", "this.logger.dispatch"),
        PRIMARY.replace("new AuditAction()", "new UnknownAction()"),
        PRIMARY.replace("new AuditAction()", "new Actions.AuditAction()"),
        PRIMARY.replace("Store<unknown>", "Store"),
        PRIMARY.replace("run() {", "static run() {"),
        PRIMARY.replace(
            "this.store.dispatch(new AuditAction());",
            "const callback=()=>{this.store.dispatch(new AuditAction());};",
        ),
        PRIMARY.replace(
            "this.store.dispatch(new AuditAction());",
            "function nested(){store.dispatch(new AuditAction());}",
        ),
        PRIMARY.replace("this.store.dispatch(new AuditAction());", "")
            + "this.store.dispatch(new AuditAction());",
        "this.store.dispatch(new AuditAction());".to_owned()
            + &PRIMARY.replace("this.store.dispatch(new AuditAction());", ""),
    ] {
        assert!(
            compile(&source).ngrx_service_dispatches.is_empty(),
            "{source}"
        );
    }
}
fn dispatches(source: &str) -> (Value, Vec<Value>, String) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("service.ts");
    std::fs::write(&path, format!("{source}\n{}", "// padding\n".repeat(100))).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    call(
        &state,
        "provide_code_context",
        json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":"high"}),
    );
    let reverse = call(
        &state,
        "workspace_query",
        json!({"workspaceRoot":root.path(),"type":"reverse_edges","domain":"ngrx","entity_type":"Action","name":"AuditAction"}),
    );
    let edges: Vec<Value> = reverse["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "Dispatches")
        .cloned()
        .collect();
    for edge in &edges {
        assert_eq!(edge["subject"]["file"], path.to_string_lossy().as_ref());
        assert_eq!(edge["object"]["file"], path.to_string_lossy().as_ref());
        assert_eq!(edge["object"]["entity_type"], "Action");
        let forward = call(
            &state,
            "workspace_query",
            json!({"workspaceRoot":root.path(),"type":"forward_edges","domain":"angular","entity_type":edge["subject"]["entity_type"],"name":edge["subject"]["name"]}),
        );
        assert!(
            forward["structuredContent"]["edges"]
                .as_array()
                .unwrap()
                .contains(edge)
        );
    }
    let entities = call(
        &state,
        "workspace_query",
        json!({"workspaceRoot":root.path(),"type":"entities_in_file","file_path":path}),
    );
    (entities, edges, path.to_string_lossy().into_owned())
}
#[test]
fn ngrx_service_dispatch_multiple_owners_and_fallback_are_occurrence_based() {
    let _serial = crate::protocol::handler_response_serial();
    let other = "@Injectable() export class OtherService {constructor(private store:Store<unknown>){} save(){this.store.dispatch(new AuditAction());}}";
    let helper = "export class Helper {constructor(private store:Store<unknown>){} save(){this.store.dispatch(new AuditAction());}}";
    let source = PRIMARY.replace(
        "this.store.dispatch(new AuditAction());",
        "this.store.dispatch(new AuditAction()); this.store.dispatch(new AuditAction());",
    ) + other
        + helper;
    let (entities, edges, path) = dispatches(&source);
    assert_eq!(edges.len(), 3);
    let mut owners: Vec<_> = edges
        .iter()
        .map(|e| {
            (
                e["subject"]["entity_type"].as_str().unwrap(),
                e["subject"]["name"].as_str().unwrap(),
            )
        })
        .collect();
    owners.sort();
    assert_eq!(
        owners,
        [
            ("DispatchSite", "store"),
            ("Service", "AuditService"),
            ("Service", "OtherService")
        ]
    );
    for name in ["AuditService", "OtherService"] {
        assert_eq!(
            entities["structuredContent"]["entities"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["entity_type"] == "Service" && e["name"] == name && e["file"] == path)
                .count(),
            1
        );
    }
    assert!(
        !entities["structuredContent"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["entity_type"] == "Service" && e["name"] == "Helper")
    );
}
#[test]
fn ngrx_service_dispatch_component_owner_and_independent_service_survive() {
    let _serial = crate::protocol::handler_response_serial();
    let component = "@Component({selector:'x'}) export class ComponentOwner {constructor(private store:Store<unknown>){} run(){this.store.dispatch(new AuditAction());}}";
    let source =
        PRIMARY.replace("import {Injectable}", "import {Injectable, Component}") + component;
    let (_, edges, _) = dispatches(&source);
    assert_eq!(edges.len(), 2);
    assert!(
        edges
            .iter()
            .any(|e| e["subject"]["entity_type"] == "Component"
                && e["subject"]["name"] == "ComponentOwner")
    );
    assert!(edges.iter().any(
        |e| e["subject"]["entity_type"] == "Service" && e["subject"]["name"] == "AuditService"
    ));
    let (entities, edges, _) =
        dispatches(&PRIMARY.replace("this.store.dispatch(new AuditAction());", ""));
    assert!(edges.is_empty());
    assert!(
        entities["structuredContent"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["entity_type"] == "Service" && e["name"] == "AuditService")
    );
}
#[test]
fn ngrx_service_dispatch_private_records_require_current_file_and_hash() {
    let ctx = compile(PRIMARY);
    let (raw, starts) = crate::angular_meta::ngrx::extract_ngrx_shape_with_sites(
        PRIMARY,
        Fidelity::High,
        &crate::meta_util::LexicalRegions::new(PRIMARY),
    )
    .unwrap();
    let mut stale = ctx.ngrx_service_dispatches.clone();
    stale[0].source_hash = "stale".into();
    for (records, file) in [
        (&stale[..], "/fixture/service.ts"),
        (&ctx.ngrx_service_dispatches[..], "/other/service.ts"),
    ] {
        let edges = crate::angular_meta::ngrx::service_owners::project(
            raw.clone(),
            &starts,
            &ctx.ngrx_constructor_dispatches,
            records,
            PRIMARY,
            file,
        );
        assert!(!edges.iter().any(|e| e.subject.entity_type == "Service"
            && e.relation == crate::layers::meta::semantic::SemanticRelation::Dispatches));
    }
}
