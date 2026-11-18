use super::*;
use crate::diff::{DiffKind, DiffTarget, build_snapshot, diff_snapshots, format_diff};

fn snapshot(source: &str) -> crate::diff::CapturedStructure {
    build_snapshot(source, Fidelity::Medium).expect("snapshot")
}

fn method_changes(before: &str, after: &str) -> Vec<(DiffKind, String, String)> {
    diff_snapshots(&snapshot(before), &snapshot(after))
        .into_iter()
        .filter(|action| action.target == DiffTarget::Method)
        .map(|action| (action.kind, action.label, action.reason_hint))
        .collect()
}

#[test]
fn markers_are_owned_by_the_containing_method_not_the_following_member() {
    let before = concat!(
        "class Probe {\n",
        "  int First() { if (ready) { return 1; } return 0; }\n",
        "  int Following() { return 2; }\n",
        "}\n",
    );
    let after = before.replace("if (ready)", "if (!ready)");
    let changes = method_changes(before, &after);

    assert!(
        changes
            .iter()
            .any(|(kind, label, _)| { *kind == DiffKind::Modified && label == "method First" })
    );
    assert!(
        !changes
            .iter()
            .any(|(kind, label, _)| { *kind == DiffKind::Modified && label == "method Following" })
    );
}

#[test]
fn first_middle_and_final_body_changes_stay_on_their_declarations() {
    let before = concat!(
        "class Probe {\n",
        "  int First() { return 1; }\n",
        "  int Middle() { return 2; }\n",
        "  int Final() { return 3; }\n",
        "}\n",
    );
    for (old, new, expected) in [
        ("return 1", "return 11", "method First"),
        ("return 2", "return 22", "method Middle"),
        ("return 3", "return 33", "method Final"),
    ] {
        let after = before.replace(old, new);
        let modified: Vec<_> = method_changes(before, &after)
            .into_iter()
            .filter(|(kind, _, _)| *kind == DiffKind::Modified)
            .collect();
        assert_eq!(modified.len(), 1, "{modified:?}");
        assert_eq!(modified[0].1, expected);
        assert_eq!(modified[0].2, "body");
    }
}

#[test]
fn overload_insertion_does_not_relabel_unchanged_overloads() {
    let before = concat!(
        "class Probe {\n",
        "  int Get(int value) { return value; }\n",
        "  int Get(string value) { return value.Length; }\n",
        "}\n",
    );
    let after = concat!(
        "class Probe {\n",
        "  int Get(bool value) { return value ? 1 : 0; }\n",
        "  int Get(int value) { return value; }\n",
        "  int Get(string value) { return value.Length; }\n",
        "}\n",
    );
    let changes = method_changes(before, after);
    assert_eq!(
        changes
            .iter()
            .filter(|(kind, _, _)| *kind == DiffKind::Added)
            .count(),
        1,
        "{changes:?}"
    );
    assert_eq!(
        changes
            .iter()
            .filter(|(kind, _, _)| *kind == DiffKind::Unchanged)
            .count(),
        2,
        "{changes:?}"
    );
    assert!(
        !changes
            .iter()
            .any(|(kind, _, _)| { matches!(kind, DiffKind::Modified | DiffKind::Removed) })
    );
}

#[test]
fn csharp_modifier_only_change_is_compared_as_declaration_evidence() {
    // `public class` intentionally selects the C# producer. Bare `class`
    // is syntactically admissible to the TypeScript fallback and would test
    // that producer rather than the C# modifier contract.
    let before = "public class Probe { public int Get() { return 1; } }\n";
    let after = "public class Probe { private int Get() { return 1; } }\n";
    let before_snapshot = snapshot(before);
    let after_snapshot = snapshot(after);
    let before_evidence = before_snapshot.classes[0].methods[0]
        .declaration
        .as_ref()
        .expect("fixture must select the C# declaration-evidence producer");
    let after_evidence = after_snapshot.classes[0].methods[0]
        .declaration
        .as_ref()
        .expect("fixture must select the C# declaration-evidence producer");
    assert_eq!(before_evidence.modifiers, ["public"]);
    assert_eq!(after_evidence.modifiers, ["private"]);
    assert_eq!(
        before_snapshot.classes[0].methods[0].sig, after_snapshot.classes[0].methods[0].sig,
        "medium display strips visibility; independent evidence must retain it"
    );
    let changes = method_changes(before, after);
    assert!(
        changes.iter().any(|(kind, label, reason)| {
            *kind == DiffKind::Modified && label == "method Get" && reason == "declaration"
        }),
        "{changes:?}"
    );
}

#[test]
fn default_projection_omits_unchanged_child_members() {
    let before = concat!(
        "class Probe {\n",
        "  int Changed() { return 1; }\n",
        "  int QuietOne() { return 2; }\n",
        "  int QuietTwo() { return 3; }\n",
        "}\n",
    );
    let after = before.replace("return 1", "return 11");
    let rendered = format_diff(
        &diff_snapshots(&snapshot(before), &snapshot(&after)),
        Fidelity::Medium,
    );
    assert!(rendered.contains("~ method Changed (body changed)"));
    assert!(!rendered.contains("QuietOne"), "{rendered}");
    assert!(!rendered.contains("QuietTwo"), "{rendered}");
    assert!(
        !rendered
            .lines()
            .any(|line| line.trim_start().starts_with('='))
    );
}

fn git(root: &str, args: &[&str]) {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn source_extension_binary_is_labeled_and_never_semantically_parsed() {
    let repo = tempfile::tempdir().expect("temp repo");
    let root = repo.path().to_str().unwrap();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "test@example.com"]);
    git(root, &["config", "user.name", "Test"]);
    std::fs::write(repo.path().join("Binary.cs"), b"class Fake\0before\xff").unwrap();
    git(root, &["add", "Binary.cs"]);
    git(root, &["commit", "-q", "-m", "before"]);
    std::fs::write(repo.path().join("Binary.cs"), b"class Fake\0after\xfe").unwrap();
    git(root, &["add", "Binary.cs"]);
    git(root, &["commit", "-q", "-m", "after"]);

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("binary diff");
    assert!(summary.manifest.contains("binary content changed"));
    assert!(
        summary
            .manifest
            .contains("semantic comparison not attempted")
    );
    assert!(!summary.manifest.contains("class Fake"));
}

#[test]
fn manifest_declares_comparison_scope_and_loss() {
    let repo = tempfile::tempdir().expect("temp repo");
    let root = repo.path().to_str().unwrap();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "test@example.com"]);
    git(root, &["config", "user.name", "Test"]);
    std::fs::write(repo.path().join("Probe.cs"), "class Probe {}\n").unwrap();
    git(root, &["add", "Probe.cs"]);
    git(root, &["commit", "-q", "-m", "before"]);
    std::fs::write(
        repo.path().join("Probe.cs"),
        "class Probe { int Get() { return 1; } }\n",
    )
    .unwrap();
    git(root, &["add", "Probe.cs"]);
    git(root, &["commit", "-q", "-m", "after"]);

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("semantic diff");
    assert!(summary.comparison_lossy);
    assert!(summary.manifest.contains("§COMPARISON lossy=true"));
    assert!(summary.manifest.contains("scope=admitted-declarations"));
    assert!(summary.manifest.contains("unsupported-declarations"));
}
