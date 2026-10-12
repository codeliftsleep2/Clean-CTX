use super::*;
#[test]
fn ngrx_service_dispatch_generation_changes_only_semantic_products() {
    let source = "import {Injectable} from '@angular/core'; import {Action,Store} from '@ngrx/store'; class AuditAction implements Action{} @Injectable() class Owner {constructor(private store:Store<unknown>){} run(){this.store.dispatch(new AuditAction());}}";
    let config = crate::tests::test_config();
    let path = Path::new("service.ts");
    let current =
        derive_with_catalog(source, path, &config, &GenerationCatalog::current()).unwrap();
    let prior = derive_with_catalog(
        source,
        path,
        &config,
        &GenerationCatalog::current().with_generation(ProducerKey::AngularSemantic, 8),
    )
    .unwrap();
    assert!(
        current
            .semantic_producers
            .differs_bidirectionally(&prior.semantic_producers)
    );
    assert_eq!(current.canonical_producers, prior.canonical_producers);
    assert_eq!(current.semantic_config, prior.semantic_config);
    assert_eq!(current.canonical_config, prior.canonical_config);
    assert_eq!(crate::layers::meta::ANGULAR_MARKER_PRODUCER_GENERATION, 5);
    assert_eq!(crate::ir::pipeline::CANONICAL_PIPELINE_GENERATION, 1);
}
