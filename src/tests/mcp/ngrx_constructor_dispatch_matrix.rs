use super::ngrx_constructor_dispatch_tests::{PRIMARY, call};
use crate::compression::Fidelity;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};
use serde_json::json;

fn compile(source: &str) -> PassContext {
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut ctx = PassContext::new(source.into(), "alias".into(), Fidelity::High);
    ctx.canonical_path = Some("/fixture/dispatch.ts".into());
    ctx.language = Some(language);
    ctx.query_string = query.into();
    crate::compression::capture_pipeline::reset_parse_count();
    CoreIRPass::new().run(&mut ctx).unwrap();
    assert_eq!(crate::compression::capture_pipeline::parse_count(), 1);
    assert!(
        ctx.captures
            .iter()
            .all(|c| !c.name.starts_with("ngrx.ctor."))
    );
    ctx
}
#[test]
fn ngrx_constructor_dispatch_source_authority_and_falsifiers() {
    let positives = [
        PRIMARY.to_owned(),
        PRIMARY.replace("new AuditAction()", "new AuditAction(payload())"),
        PRIMARY
            .replace("private store", "private events")
            .replace("this.store.dispatch", "this.events.dispatch"),
    ];
    for source in positives {
        let ctx = compile(&source);
        let records = &ctx.ngrx_constructor_dispatches;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].action_name.as_deref(), Some("AuditAction"));
        assert_eq!(
            records[0].action_span,
            Some(ctx.ngrx_class_actions[0].class_span)
        );
        assert_eq!(records[0].owner_name.as_deref(), Some("Dispatcher"));
        assert_eq!(records[0].file.as_deref(), Some("/fixture/dispatch.ts"));
        assert_eq!(
            records[0].source_hash,
            crate::cache::content_hash(source.as_bytes())
        );
    }
    for source in [
        PRIMARY.replace("implements Action", ""),
        PRIMARY.replace("Store<unknown>", "Logger"),
        PRIMARY.replace(".dispatch(new", ".dispatch?.(new"),
        PRIMARY.replace("this.store.dispatch", "this.logger.dispatch"),
        PRIMARY.replace(
            "import {Action, Store} from '@ngrx/store';",
            "import {Action} from '@ngrx/store'; import {Store} from './fake';",
        ),
        PRIMARY.replace("run() {", "static run() {"),
        PRIMARY.replace("run() {", "run(AuditAction: any) {"),
        PRIMARY.replace("run() {", "run() {const AuditAction=other;"),
        PRIMARY.replace("run() {", "run() {for(var AuditAction of values) {}"),
        PRIMARY.replace("new AuditAction()", "new Actions.AuditAction()"),
        PRIMARY.replace(
            "constructor(private store: Store<unknown>)",
            "constructor(private store: Store<unknown>, public store: Store<unknown>)",
        ),
        PRIMARY.replace("run() {", "store: Logger; run() {"),
    ] {
        let ctx = compile(&source);
        assert_eq!(ctx.ngrx_constructor_dispatches.len(), 1);
        assert!(
            ctx.ngrx_constructor_dispatches[0].action_name.is_none(),
            "{source}"
        );
    }
    for expression in [
        "wrapper(new AuditAction())",
        "flag ? new AuditAction() : other",
        "[new AuditAction()]",
        "makeAction(new AuditAction())",
        "AuditAction",
        "AuditAction()",
    ] {
        let source = PRIMARY.replace("new AuditAction()", expression);
        assert!(
            compile(&source).ngrx_constructor_dispatches.is_empty(),
            "descendant promoted: {expression}"
        );
    }
}
#[test]
fn ngrx_constructor_dispatch_exact_public_matrix_and_reverse() {
    let _serial = crate::protocol::handler_response_serial();
    let base=PRIMARY.replace("@Component", "export class OtherAction implements Action {readonly type='audit';} class NonAction {} @Component");
    for (expression, expected) in [
        ("new AuditAction()", vec!["AuditAction"]),
        ("new OtherAction()", vec!["OtherAction"]),
        ("new NonAction()", vec![]),
        ("new Actions.AuditAction()", vec![]),
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("dispatch.ts");
        let source = base.replace("new AuditAction()", expression) + &"\n// padding".repeat(100);
        std::fs::write(&path, source).unwrap();
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
        let answer = call(
            &state,
            "workspace_query",
            json!({"workspaceRoot":root.path(),"type":"forward_edges","domain":"angular","entity_type":"Component","name":"Dispatcher"}),
        );
        let edges: Vec<_> = answer["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "Dispatches")
            .collect();
        assert_eq!(
            edges
                .iter()
                .map(|e| e["object"]["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            expected,
            "{answer}"
        );
        for edge in edges {
            assert_eq!(edge["subject"]["file"], path.to_string_lossy().as_ref());
            assert_eq!(edge["object"]["domain"], "ngrx");
            assert_eq!(edge["object"]["entity_type"], "Action");
            assert_eq!(edge["object"]["file"], path.to_string_lossy().as_ref());
            let reverse = call(
                &state,
                "workspace_query",
                json!({"workspaceRoot":root.path(),"type":"reverse_edges","domain":"ngrx","entity_type":"Action","name":edge["object"]["name"]}),
            );
            assert!(
                reverse["structuredContent"]["edges"]
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
        let actions: Vec<_> = entities["structuredContent"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["domain"] == "ngrx" && e["entity_type"] == "Action")
            .collect();
        assert_eq!(actions.len(), 2);
        assert!(
            !actions
                .iter()
                .any(|e| e["name"] == "new" || e["name"] == "NonAction")
        );
    }
}
#[test]
fn ngrx_constructor_dispatch_raw_provenance_preserves_modern_symbolic_site() {
    let source = PRIMARY.replace(
        "this.store.dispatch(new AuditAction());",
        "this.store.dispatch(new AuditAction()); this.store.dispatch(modernAction());",
    );
    let ctx = compile(&source);
    let (shape, starts) = crate::angular_meta::ngrx::extract_ngrx_shape_with_sites(
        &source,
        Fidelity::High,
        &crate::meta_util::LexicalRegions::new(&source),
    )
    .unwrap();
    assert_eq!(starts.len(), shape.dispatch_sites.len());
    let original = shape.clone();
    let shape = crate::angular_meta::ngrx::constructors::apply(
        shape,
        &starts,
        &ctx.ngrx_constructor_dispatches,
        &source,
        "/fixture/dispatch.ts",
    );
    assert_eq!(
        shape
            .dispatch_sites
            .iter()
            .map(|s| s.action_name.as_str())
            .collect::<Vec<_>>(),
        ["modernAction", "AuditAction"]
    );
    let stale = crate::angular_meta::ngrx::constructors::apply(
        original,
        &starts,
        &ctx.ngrx_constructor_dispatches,
        &source,
        "/other/dispatch.ts",
    );
    assert!(
        stale.dispatch_sites.len() == 1 && stale.dispatch_sites[0].action_name == "modernAction",
        "wrong-file records cannot add a resolved reference"
    );
}
#[test]
fn ngrx_constructor_dispatch_import_is_a_separate_resolution_boundary() {
    let source = PRIMARY.replace(
        "export class AuditAction implements Action {readonly type='audit';}",
        "import {AuditAction} from './actions';",
    );
    let ctx = compile(&source);
    assert!(ctx.ngrx_class_actions.is_empty());
    assert_eq!(ctx.ngrx_constructor_dispatches.len(), 1);
    assert!(ctx.ngrx_constructor_dispatches[0].action_name.is_none());
}
