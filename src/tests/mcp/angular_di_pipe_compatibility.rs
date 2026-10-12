use super::*;

#[test]
fn angular_di_pipe_semantic_generation_excludes_prior_gate_projection() {
    let source = "import { Pipe } from '@angular/core';\n@Pipe({name:'audit'}) class AuditPipe { constructor(private dependency: AuditService) {} }";
    let config = crate::tests::test_config();
    let path = Path::new("audit.pipe.ts");
    let current =
        derive_with_catalog(source, path, &config, &GenerationCatalog::current()).unwrap();
    let prior = derive_with_catalog(
        source,
        path,
        &config,
        &GenerationCatalog::current().with_generation(ProducerKey::AngularSemantic, 3),
    )
    .unwrap();
    assert!(
        current
            .semantic_producers
            .differs_bidirectionally(&prior.semantic_producers)
    );
    assert_eq!(current.canonical_producers, prior.canonical_producers);
    assert_eq!(current.canonical_config, prior.canonical_config);
    assert_eq!(current.semantic_config, prior.semantic_config);
}
