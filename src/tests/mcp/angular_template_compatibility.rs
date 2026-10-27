//! Phase 4 RED compatibility contract for Angular Template semantics.

use super::*;

const ANGULAR_TEMPLATE_SOURCE: &str = r#"
import { Component } from '@angular/core';
@Component({ selector: 'x-host', templateUrl: './host.html' })
export class HostComponent {}
"#;

#[test]
fn angular_template_semantics_increment_only_the_existing_angular_producer() {
    assert_eq!(
        crate::layers::meta::ANGULAR_SEMANTIC_PRODUCER_GENERATION,
        69,
        "Exact modern Service dispatch ownership advances the existing Angular/NgRx producer cohort"
    );
    assert_eq!(
        crate::layers::meta::ANGULAR_MARKER_PRODUCER_GENERATION,
        6,
        "Legacy class markers advance the existing Angular/NgRx marker cohort"
    );

    let config = crate::tests::test_config();
    let current = derive_with_catalog(
        ANGULAR_TEMPLATE_SOURCE,
        Path::new("host.component.ts"),
        &config,
        &GenerationCatalog::current(),
    )
    .expect("current Angular identity");
    let prior = derive_with_catalog(
        ANGULAR_TEMPLATE_SOURCE,
        Path::new("host.component.ts"),
        &config,
        &GenerationCatalog::current().with_generation(ProducerKey::AngularSemantic, 1),
    )
    .expect("prior Angular identity");

    assert!(
        current
            .semantic_producers
            .differs_bidirectionally(&prior.semantic_producers),
        "generation 1 snapshots must be incompatible with Phase 4 Angular semantics"
    );
    assert_eq!(
        current.canonical_producers, prior.canonical_producers,
        "HasTemplate must not change canonical or marker producer identity"
    );
}
