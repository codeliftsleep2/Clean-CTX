use crate::angular_meta::inject_evidence::{
    ClassInitializerEvidence, InitializerClassification as Kind,
};
use crate::compression::Fidelity;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};

fn evidence(source: &str) -> Vec<ClassInitializerEvidence> {
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut state = PassContext::new(source.into(), "evidence-source".into(), Fidelity::High);
    state.language = Some(language);
    state.query_string = query.into();
    CoreIRPass::new().run(&mut state).unwrap();
    assert!(
        state
            .captures
            .iter()
            .all(|capture| !capture.name.starts_with("angular.initializer."))
    );
    state.angular_initializers
}

#[test]
fn angular_inject_evidence_direct_descendant_and_token_roles() {
    for (expression, expected, token) in [
        (
            "inject(AuditService)",
            Kind::DirectSupported,
            Some("AuditService"),
        ),
        (
            "inject( AuditService )",
            Kind::DirectSupported,
            Some("AuditService"),
        ),
        (
            "inject(\n AuditService\n)",
            Kind::DirectSupported,
            Some("AuditService"),
        ),
        (
            "condition ? inject(AuditService) : otherValue",
            Kind::NotDirect,
            None,
        ),
        ("wrapper(inject(AuditService))", Kind::NotDirect, None),
        ("makeService(AuditService)", Kind::NotAngularInject, None),
        ("obj.inject(AuditService)", Kind::NotAngularInject, None),
        ("(inject(AuditService))", Kind::NotDirect, None),
        (
            "inject(namespace.AuditService)",
            Kind::UnsupportedToken,
            Some("namespace.AuditService"),
        ),
        ("inject(MY_TOKEN)", Kind::DirectSupported, Some("MY_TOKEN")),
        (
            "inject?.(AuditService)",
            Kind::UnsupportedCall,
            Some("AuditService"),
        ),
        (
            "inject(AuditService, { optional: true })",
            Kind::DirectSupported,
            Some("AuditService"),
        ),
        (
            "inject(...tokens)",
            Kind::UnsupportedToken,
            Some("...tokens"),
        ),
    ] {
        let source = format!(
            "@Injectable() class AuditConsumer {{ private readonly completelyDifferentName = {expression}; }}"
        );
        let classes = evidence(&source);
        assert_eq!(classes.len(), 1);
        let class = &classes[0];
        assert_eq!(class.consumer, "AuditConsumer");
        assert_eq!(
            &source[class.class_span.start_byte()..class.class_span.end_byte()],
            source
        );
        let field = &class.fields[0];
        assert_eq!(field.receiver, "completelyDifferentName");
        assert_eq!(field.classification, expected, "{expression}");
        assert_eq!(field.written_token.as_deref(), token);
        assert_eq!(
            &source[field.initializer_span.start_byte()..field.initializer_span.end_byte()],
            expression
        );
        assert!(class.class_span.start_byte() <= field.field_span.start_byte());
        assert!(field.field_span.end_byte() <= class.class_span.end_byte());
        if expected == Kind::DirectSupported {
            assert_eq!(field.invocation_span, Some(field.initializer_span));
            let argument = field.dependency_argument.unwrap();
            assert_eq!(
                &source[argument.start_byte()..argument.end_byte()],
                token.unwrap()
            );
            assert_ne!(field.receiver, token.unwrap());
        }
    }
}

#[test]
fn angular_inject_evidence_multiple_fields_and_canonical_owners_do_not_cross_wire() {
    let source = "@Injectable() class First { first = inject(AuditService); second = inject(LoggerService); }\n@Injectable() abstract class Second { first = inject(OtherService); }";
    let classes = evidence(source);
    assert_eq!(
        classes
            .iter()
            .map(|class| class.consumer.as_str())
            .collect::<Vec<_>>(),
        ["First", "Second"]
    );
    assert_eq!(
        classes[0]
            .fields
            .iter()
            .map(|field| (field.receiver.as_str(), field.written_token.as_deref()))
            .collect::<Vec<_>>(),
        [
            ("first", Some("AuditService")),
            ("second", Some("LoggerService"))
        ]
    );
    assert_eq!(
        classes[1].fields[0].written_token.as_deref(),
        Some("OtherService")
    );
}

#[test]
fn angular_inject_evidence_no_outer_owner_for_anonymous_class_field() {
    let classes = evidence("class Outer { value = class { receiver = inject(AuditService); }; }");
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].fields.len(), 1);
    assert_eq!(classes[0].fields[0].receiver, "value");
    assert_ne!(classes[0].fields[0].classification, Kind::DirectSupported);
}
