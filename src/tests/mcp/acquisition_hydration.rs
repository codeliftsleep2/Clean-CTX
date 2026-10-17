use super::{
    HydrationRequirement, TEST_PROJECT_HYDRATION_SERIALIZE, fail_test_publication_once,
    hydrate_workspace_index_for_acquisition,
};
use crate::mcp::acquisition::AcquisitionTarget;

#[test]
fn clean_csharp_acquisition_is_satisfied_only_after_production_publication() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::tempdir().expect("workspace");
    std::fs::create_dir(root.path().join(".git")).expect("Git root marker");
    std::fs::write(
        root.path().join("ContextA.cs"),
        "public sealed class ContextA {}\n",
    )
    .expect("target source");
    std::fs::write(
        root.path().join("Consumer.cs"),
        "public sealed class Consumer { public Consumer(ContextA value) {} }\n",
    )
    .expect("consumer source");
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let workspace_root = root.path().to_string_lossy();

    let report = hydrate_workspace_index_for_acquisition(
        &state,
        "ContextA",
        Some(workspace_root.as_ref()),
        HydrationRequirement::LegacyEdit,
        AcquisitionTarget::exact_type_ref("ContextA"),
    )
    .expect("production hydration");

    assert_eq!(report.candidates_discovered, 2);
    assert!(
        report
            .acquisition_satisfaction
            .as_ref()
            .is_some_and(|satisfaction| satisfaction.is_source_complete()),
        "complete C# inventory plus successful semantic publication must satisfy its unit"
    );
}

#[test]
fn failed_publication_cannot_satisfy_clean_csharp_acquisition() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::tempdir().expect("workspace");
    std::fs::create_dir(root.path().join(".git")).expect("Git root marker");
    std::fs::write(
        root.path().join("ContextA.cs"),
        "public sealed class ContextA {}\n",
    )
    .expect("target source");
    let consumer = root.path().join("Consumer.cs");
    std::fs::write(
        &consumer,
        "public sealed class Consumer { public Consumer(ContextA value) {} }\n",
    )
    .expect("consumer source");
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let workspace_root = root.path().to_string_lossy();
    fail_test_publication_once(&consumer.to_string_lossy());

    let report = hydrate_workspace_index_for_acquisition(
        &state,
        "ContextA",
        Some(workspace_root.as_ref()),
        HydrationRequirement::LegacyEdit,
        AcquisitionTarget::exact_type_ref("ContextA"),
    )
    .expect("production hydration with injected publication failure");

    assert_eq!(report.discovery_status, "partial");
    assert!(
        report
            .acquisition_satisfaction
            .as_ref()
            .is_some_and(|satisfaction| !satisfaction.is_source_complete()),
        "publication failure must leave the required C# unit unsatisfied"
    );
}
