//! LGR-001 durable .NET semantic producer compatibility contract.

use super::*;

const SOURCE: &str = "public class OrderService {} [TestClass] public class OrderServiceTests {}";

#[test]
fn declaration_proven_test_subjects_advance_only_dotnet_semantics() {
    assert_eq!(
        crate::dotnet_meta::SEMANTIC_PRODUCER_GENERATION,
        2,
        "removing durable candidate Tests edges and adding TestClass declarations changes semantics"
    );
    assert_eq!(crate::dotnet_meta::MARKER_PRODUCER_GENERATION, 1);

    let config = crate::tests::test_config();
    let current = derive_with_catalog(
        SOURCE,
        Path::new("OrderServiceTests.cs"),
        &config,
        &GenerationCatalog::current(),
    )
    .expect("current .NET identity");
    let prior = derive_with_catalog(
        SOURCE,
        Path::new("OrderServiceTests.cs"),
        &config,
        &GenerationCatalog::current().with_generation(ProducerKey::DotNetSemantic, 1),
    )
    .expect("prior .NET identity");

    assert!(
        current
            .semantic_producers
            .differs_bidirectionally(&prior.semantic_producers)
    );
    assert_eq!(current.canonical_producers, prior.canonical_producers);
    assert_eq!(current.canonical_config, prior.canonical_config);
    assert_eq!(current.semantic_config, prior.semantic_config);
}
