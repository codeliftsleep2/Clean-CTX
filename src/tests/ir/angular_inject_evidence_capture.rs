//! The IR-only query must retain exact property initializer boundaries.
use crate::compression::capture_pipeline::run_capture_pipeline_nodes;
use crate::compression::{Fidelity, language::safe_typescript_language};

#[test]
fn angular_inject_evidence_initializer_values_are_structural() {
    for expression in [
        "inject(AuditService)",
        "condition ? inject(AuditService) : otherValue",
        "wrapper(inject(AuditService))",
        "makeService(AuditService)",
        "(inject(AuditService))",
    ] {
        let source = format!("class AuditConsumer {{ private unrelated = {expression}; }}");
        let query = format!(
            "{}\n{}\n{}",
            crate::queries::TS_QUERY,
            crate::queries::TS_CALL_QUERY,
            super::DECLARATION_QUERY
        );
        let captures = run_capture_pipeline_nodes(
            safe_typescript_language().unwrap(),
            &query,
            &source,
            Fidelity::High,
            |_, raw, _| Some(raw.into()),
        )
        .unwrap();
        let fields: Vec<_> = captures
            .iter()
            .filter(|c| c.name == "angular.initializer.field")
            .collect();
        assert_eq!(
            fields.len(),
            1,
            "exact structured field association missing: {expression}"
        );
        let field = fields[0];
        let value = captures
            .iter()
            .find(|c| c.name == "angular.initializer.value" && c.match_index == field.match_index)
            .expect("initializer from same query match");
        let receiver = captures
            .iter()
            .find(|c| {
                c.name == "angular.initializer.receiver" && c.match_index == field.match_index
            })
            .expect("receiver from same query match");
        assert_eq!(receiver.raw_text, "unrelated");
        assert_eq!(value.raw_text, expression);
        assert!(field.start_byte <= value.start_byte && value.end_byte <= field.end_byte);
        assert_eq!(&source[value.start_byte..value.end_byte], expression);
        let inject = captures
            .iter()
            .find(|c| c.name == "invoke.root" && c.raw_text == "inject(AuditService)");
        if let Some(inject) = inject {
            assert_eq!(
                (inject.start_byte, inject.end_byte) == (value.start_byte, value.end_byte),
                expression == "inject(AuditService)",
                "descendant is not the initializer"
            );
        }
    }
}
