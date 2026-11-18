use super::*;
#[test]
fn ngrx_constructor_dispatch_generation_excludes_old_semantic_and_marker_products() {
    let source = "import {Action,Store} from '@ngrx/store'; class AuditAction implements Action {readonly type='audit';}";
    let config = crate::tests::test_config();
    let path = Path::new("dispatch.ts");
    let current =
        derive_with_catalog(source, path, &config, &GenerationCatalog::current()).unwrap();
    let prior = derive_with_catalog(
        source,
        path,
        &config,
        &GenerationCatalog::current()
            .with_generation(ProducerKey::AngularSemantic, 7)
            .with_generation(ProducerKey::AngularMarkers, 4),
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
