//! Source changes keep the context bytes fixed; all calls use public dispatch.
use super::*;

#[test]
#[serial_test::serial]
fn issue_156_entity_appearance_deletion_and_restoration_without_target_edit() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), "builder.Entity<App.First>();");
    std::fs::remove_file(root.path().join("Rows.cs")).unwrap();
    let state = state(&[]);
    expect(&target(&state, root.path()), &[]);
    write(
        root.path(),
        "Rows.cs",
        "namespace App; public class First {}",
    );
    expect(&target(&state, root.path()), &["First"]);
    std::fs::remove_file(root.path().join("Rows.cs")).unwrap();
    expect(&target(&state, root.path()), &[]);
    write(
        root.path(),
        "Rows.cs",
        "namespace App; public class First {}",
    );
    expect(&target(&state, root.path()), &["First"]);
}

#[test]
#[serial_test::serial]
fn issue_156_configuration_retargets_retracts_and_restores_without_target_edit() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), "builder.ApplyConfiguration(new Mapping());");
    let state = state(&[]);
    expect(&target(&state, root.path()), &["First"]);
    write(
        root.path(),
        "Mapping.cs",
        "using Microsoft.EntityFrameworkCore; namespace App; public class Mapping : IEntityTypeConfiguration<Second> {}",
    );
    expect(&target(&state, root.path()), &["Second"]);
    std::fs::remove_file(root.path().join("Mapping.cs")).unwrap();
    expect(&target(&state, root.path()), &[]);
    write(
        root.path(),
        "Mapping.cs",
        "using Microsoft.EntityFrameworkCore; namespace App; public class Mapping : IEntityTypeConfiguration<First> {}",
    );
    expect(&target(&state, root.path()), &["First"]);
}

#[test]
#[serial_test::serial]
fn issue_156_assembly_universe_expands_contracts_and_changes_project_membership() {
    let root = tempfile::tempdir().unwrap();
    fixture(
        root.path(),
        "builder.ApplyConfigurationsFromAssembly(typeof(Target).Assembly);",
    );
    let state = state(&[]);
    expect(&target(&state, root.path()), &["First"]);
    write(
        root.path(),
        "Mapping.cs",
        "using Microsoft.EntityFrameworkCore; namespace App; public class Mapping : IEntityTypeConfiguration<Second> {}",
    );
    expect(&target(&state, root.path()), &["Second"]);
    write(
        root.path(),
        "Mapping.cs",
        "using Microsoft.EntityFrameworkCore; namespace App; public class Mapping : IEntityTypeConfiguration<First> {}",
    );
    expect(&target(&state, root.path()), &["First"]);
    write(
        root.path(),
        "Nested/SecondMap.cs",
        "using Microsoft.EntityFrameworkCore; namespace App; public class SecondMap : IEntityTypeConfiguration<Second> {}",
    );
    expect(&target(&state, root.path()), &["First", "Second"]);
    write(root.path(), "Nested/Nested.csproj", "<Project />");
    expect(&target(&state, root.path()), &["First"]);
    std::fs::remove_file(root.path().join("Nested/Nested.csproj")).unwrap();
    expect(&target(&state, root.path()), &["First", "Second"]);
    std::fs::remove_file(root.path().join("Nested/SecondMap.cs")).unwrap();
    expect(&target(&state, root.path()), &["First"]);
}

#[test]
#[serial_test::serial]
fn issue_156_assembly_marker_identity_and_project_proof_retract_and_restore() {
    let root = tempfile::tempdir().unwrap();
    fixture(
        root.path(),
        "builder.ApplyConfigurationsFromAssembly(typeof(Marker).Assembly);",
    );
    write(
        root.path(),
        "Marker.cs",
        "namespace App; public class Marker {}",
    );
    let state = state(&[]);
    expect(&target(&state, root.path()), &["First"]);
    write(
        root.path(),
        "Marker.cs",
        "namespace App; public class OtherMarker {}",
    );
    expect(&target(&state, root.path()), &[]);
    write(
        root.path(),
        "Marker.cs",
        "namespace App; public class Marker {}",
    );
    expect(&target(&state, root.path()), &["First"]);
    write(root.path(), "Ambiguous.csproj", "<Project />");
    expect(&target(&state, root.path()), &[]);
    std::fs::remove_file(root.path().join("Ambiguous.csproj")).unwrap();
    expect(&target(&state, root.path()), &["First"]);
}

#[test]
#[serial_test::serial]
fn issue_156_batch_order_and_late_publication_use_surviving_truth() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), "builder.ApplyConfiguration(new Mapping());");
    for reverse in [false, true] {
        let state = state(&[]);
        let workspace = root.path().to_string_lossy().into_owned();
        let mapping = root.path().join("Mapping.cs");
        crate::mcp::tool_handlers::query::publication_test_support::arm(move |state| {
            std::fs::write(&mapping, "using Microsoft.EntityFrameworkCore; namespace App; public class Mapping : IEntityTypeConfiguration<Second> {}").unwrap();
            let owner = state.semantic_owner_path(&mapping.to_string_lossy());
            state
                .prepare_csharp_semantic_owner(&owner, Some(&workspace))
                .unwrap();
        });
        let mut first = target_args();
        first["id"] = json!("target");
        let mut items = vec![
            first,
            json!({"id":"rows", "type":"entities_in_file", "file_path":"Rows.cs"}),
        ];
        if reverse {
            items.reverse();
        }
        let result = query(&state, root.path(), json!({"queries":items}));
        let results = result["results"].as_array().unwrap();
        assert!(
            results.iter().all(|item| item["status"] == "ok"),
            "{result:#}"
        );
        let item = results.iter().find(|item| item["id"] == "target").unwrap();
        expect(&item["result"], &["Second"]);
        assert_eq!(
            item["result"]["coverage"]["dependency_proof"]["status"], "complete",
            "{result:#}"
        );
        write(
            root.path(),
            "Mapping.cs",
            "using Microsoft.EntityFrameworkCore; namespace App; public class Mapping : IEntityTypeConfiguration<First> {}",
        );
    }
}
