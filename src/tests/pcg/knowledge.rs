use crate::pcg::document_fixture::Fixture;

#[test]
fn explicit_knowledge_boundaries_reconcile_without_changing_legacy_shape() {
    let f = Fixture::new();
    let source = "<!-- pcg:PIPE-A:begin -->\nPassPipeline ordering is explicit.\n<!-- pcg:PIPE-A:end -->\n\
<!-- pcg:WIRE-B:begin -->\nSerialization preserves every instruction.\n<!-- pcg:WIRE-B:end -->\n\
```architecture-graph\nnode: DOC-001\nkind: INVARIANT\nstatus: IMPLEMENTED\ntitle: Pipeline order\n\
track: FOUNDATION\nartifact: docs/shared.md\nknowledge_scope: invariant\nknowledge_content:\n  - statement self#PIPE-A\n```\n\
```architecture-graph\nnode: DOC-002\nkind: INVARIANT\nstatus: IMPLEMENTED\ntitle: Wire preservation\n\
track: FOUNDATION\nartifact: docs/shared.md\nknowledge_scope: invariant\nknowledge_content:\n  - statement self#WIRE-B\n```\n";
    std::fs::write(f.root.join("docs/shared.md"), source).unwrap();
    let payload = f.payload("get", Some("DOC-001"));
    assert_eq!(
        payload["result_state"], "ready",
        "{:?}",
        payload["diagnostics"]
    );
    assert_eq!(payload["documents"][0]["source_text"], source);
    assert!(
        payload["documents"][0]["declaration"]
            .get("knowledge_scope")
            .is_none()
    );
}
