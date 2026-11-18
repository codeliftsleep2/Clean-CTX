use super::*;

#[test]
fn angular_provider_vocabulary_rejects_generation_22_semantics_only() {
    let source = "import { NgModule } from '@angular/core'; class ApiClient {} @NgModule({providers:[ApiClient]}) class CoreModule {}";
    let config = crate::tests::test_config();
    let path = Path::new("core.module.ts");
    let current =
        derive_with_catalog(source, path, &config, &GenerationCatalog::current()).unwrap();
    let prior = derive_with_catalog(
        source,
        path,
        &config,
        &GenerationCatalog::current().with_generation(ProducerKey::AngularSemantic, 22),
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
