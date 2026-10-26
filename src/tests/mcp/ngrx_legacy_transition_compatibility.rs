use super::*;

#[test]
fn ngrx_legacy_transition_generations_exclude_pre_transition_products() {
    let source = "import {Action} from '@ngrx/store'; const SET_AUDIT='value'; class AuditAction implements Action {readonly type=SET_AUDIT;}";
    let config = crate::tests::test_config();
    let path = Path::new("actions.ts");
    let current =
        derive_with_catalog(source, path, &config, &GenerationCatalog::current()).unwrap();
    let prior = derive_with_catalog(
        source,
        path,
        &config,
        &GenerationCatalog::current().with_generation(ProducerKey::AngularSemantic, 60),
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
    assert_eq!(crate::layers::meta::ANGULAR_MARKER_PRODUCER_GENERATION, 6);
    assert_eq!(
        crate::layers::meta::ANGULAR_SEMANTIC_PRODUCER_GENERATION,
        61
    );
    assert_eq!(crate::ir::pipeline::CANONICAL_PIPELINE_GENERATION, 1);
}
