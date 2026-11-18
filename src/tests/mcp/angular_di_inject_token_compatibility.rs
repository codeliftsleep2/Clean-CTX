use super::*;

#[test]
fn angular_di_inject_token_generations_reject_prior_markers_and_edges() {
    let source = "import { Injectable, inject } from '@angular/core'; @Injectable() class AuditConsumer { private different = inject(AuditService); }";
    let config = crate::tests::test_config();
    let path = Path::new("audit.ts");
    let current =
        derive_with_catalog(source, path, &config, &GenerationCatalog::current()).unwrap();
    let prior = derive_with_catalog(
        source,
        path,
        &config,
        &GenerationCatalog::current()
            .with_generation(ProducerKey::AngularSemantic, 4)
            .with_generation(ProducerKey::AngularMarkers, 2),
    )
    .unwrap();
    assert!(
        current
            .semantic_producers
            .differs_bidirectionally(&prior.semantic_producers)
    );
    assert!(
        current
            .canonical_producers
            .differs_bidirectionally(&prior.canonical_producers)
    );
    assert_eq!(current.canonical_config, prior.canonical_config);
    assert_eq!(current.semantic_config, prior.semantic_config);
    assert_eq!(crate::ir::pipeline::CANONICAL_PIPELINE_GENERATION, 1);
}
