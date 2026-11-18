use super::*;

#[test]
fn ngrx_legacy_declaration_generations_exclude_pre_class_products() {
    let source = "import { Action } from '@ngrx/store'; export class NoSuffixRecord implements Action {readonly type = 'record';}";
    let config = crate::tests::test_config();
    let path = Path::new("actions.ts");
    let current =
        derive_with_catalog(source, path, &config, &GenerationCatalog::current()).unwrap();
    let prior = derive_with_catalog(
        source,
        path,
        &config,
        &GenerationCatalog::current()
            .with_generation(ProducerKey::AngularSemantic, 5)
            .with_generation(ProducerKey::AngularMarkers, 3),
    )
    .unwrap();
    assert!(
        current
            .canonical_producers
            .differs_bidirectionally(&prior.canonical_producers)
    );
    assert!(
        current
            .semantic_producers
            .differs_bidirectionally(&prior.semantic_producers)
    );
    assert_eq!(current.canonical_config, prior.canonical_config);
    assert_eq!(current.semantic_config, prior.semantic_config);
    assert_eq!(crate::ir::pipeline::CANONICAL_PIPELINE_GENERATION, 1);
}
