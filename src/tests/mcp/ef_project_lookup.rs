use super::{ProjectLookup, work_metrics};

fn file_text(path: &std::path::Path) -> &str {
    path.to_str().expect("UTF-8 fixture path")
}

#[test]
fn build_local_project_lookup_preserves_some_none_and_file_identity() {
    let same_project = tempfile::tempdir().expect("same-project root");
    let first = same_project.path().join("First.cs");
    let second = same_project.path().join("Second.cs");
    std::fs::write(&first, "class First {}").expect("first fixture");
    std::fs::write(&second, "class Second {}").expect("second fixture");
    let project = same_project.path().join("Same.csproj");
    std::fs::write(&project, "<Project />").expect("project fixture");

    work_metrics::reset();
    let mut lookup = ProjectLookup::default();
    assert_eq!(
        lookup.nearest_project(file_text(&first)),
        Some(project.clone())
    );
    assert_eq!(
        lookup.nearest_project(file_text(&first)),
        Some(project.clone())
    );
    assert_eq!(
        lookup.nearest_project(file_text(&second)),
        Some(project.clone())
    );
    let counts = work_metrics::snapshot();
    assert_eq!(counts.ef_project_root_lookups, 3);
    assert_eq!(counts.ef_project_root_filesystem_resolutions, 2);
    assert_eq!(counts.ef_project_root_cache_hits, 1);

    let missing = tempfile::tempdir().expect("missing-project root");
    let missing_file = missing.path().join("Missing.cs");
    std::fs::write(&missing_file, "class Missing {}").expect("missing fixture");
    work_metrics::reset();
    let mut lookup = ProjectLookup::default();
    assert_eq!(lookup.nearest_project(file_text(&missing_file)), None);
    assert_eq!(lookup.nearest_project(file_text(&missing_file)), None);
    let counts = work_metrics::snapshot();
    assert_eq!(counts.ef_project_root_filesystem_resolutions, 1);
    assert_eq!(counts.ef_project_root_cache_hits, 1);

    let ambiguous = tempfile::tempdir().expect("ambiguous-project root");
    let ambiguous_file = ambiguous.path().join("Ambiguous.cs");
    std::fs::write(&ambiguous_file, "class Ambiguous {}").expect("ambiguous fixture");
    std::fs::write(ambiguous.path().join("One.csproj"), "<Project />")
        .expect("first ambiguous project");
    std::fs::write(ambiguous.path().join("Two.csproj"), "<Project />")
        .expect("second ambiguous project");
    work_metrics::reset();
    let mut lookup = ProjectLookup::default();
    assert_eq!(lookup.nearest_project(file_text(&ambiguous_file)), None);
    assert_eq!(lookup.nearest_project(file_text(&ambiguous_file)), None);
    let counts = work_metrics::snapshot();
    assert_eq!(counts.ef_project_root_filesystem_resolutions, 1);
    assert_eq!(counts.ef_project_root_cache_hits, 1);

    let other_project = tempfile::tempdir().expect("other-project root");
    let other_file = other_project.path().join("Other.cs");
    let other = other_project.path().join("Other.csproj");
    std::fs::write(&other_file, "class Other {}").expect("other fixture");
    std::fs::write(&other, "<Project />").expect("other project fixture");
    work_metrics::reset();
    let mut lookup = ProjectLookup::default();
    assert_eq!(
        lookup.common_project(file_text(&first), file_text(&second)),
        Some(project)
    );
    assert_eq!(
        lookup.common_project(file_text(&first), file_text(&other_file)),
        None
    );
    let counts = work_metrics::snapshot();
    assert_eq!(counts.ef_project_root_lookups, 2);
    assert_eq!(counts.ef_project_root_filesystem_resolutions, 3);
}
