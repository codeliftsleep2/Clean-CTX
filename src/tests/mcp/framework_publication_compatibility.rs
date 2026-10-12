use super::*;

#[test]
fn framework_declaration_publication_rejects_prior_semantics_without_changing_canonical_identity() {
    let config = crate::tests::test_config();
    for source in [
        "import { Injectable } from '@angular/core';\n@Injectable() class OwnerProbe {}",
        "import { createAction } from '@ngrx/store';\nexport const setAudit = createAction('[Audit] Set Audit');",
    ] {
        let path = Path::new("declarations.ts");
        let current =
            derive_with_catalog(source, path, &config, &GenerationCatalog::current()).unwrap();
        let prior = derive_with_catalog(
            source,
            path,
            &config,
            &GenerationCatalog::current().with_generation(ProducerKey::AngularSemantic, 2),
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
}
