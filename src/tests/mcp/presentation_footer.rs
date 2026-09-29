use super::presentation_document;
use crate::compression::Fidelity;
use crate::ir::hierarchical::try_ir_to_hierarchical;
use crate::ir::{CompiledIR, CoreOp};

#[test]
fn presentation_footer_keeps_one_authoritative_path_mapping() {
    let source_path = r"C:\workspace\src\service.ts";
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let alias = state.get_or_create_alias(source_path.to_string());
    let ir = CompiledIR {
        file_id: alias.clone(),
        instructions: vec![CoreOp::DefClass("C1".into(), "Service".into())],
        version: 1,
    };
    let hierarchy = try_ir_to_hierarchical(&ir).expect("checked hierarchy");

    let text = presentation_document(&ir, &hierarchy, Fidelity::Low, &state);

    assert!(
        text.ends_with(&format!("// {alias}\n§PATHMAP\n  {alias} = {source_path}")),
        "the file boundary must retain the alias and authoritative mapping: {text}"
    );
    assert_eq!(
        text.matches(source_path).count(),
        1,
        "the exact path must appear only in PATHMAP: {text}"
    );
    assert!(
        !text.contains(&format!("// ── {alias} (")),
        "the decorative file boundary must not duplicate the path: {text}"
    );
}
