use crate::compression::Fidelity;
use crate::gitdiff::engine::gitdiff_workspace;

fn run_git(root: &str, args: &[&str]) {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn file_count_limit_preserves_discovered_total_and_accounts_for_skips() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().expect("UTF-8 temp path");

    run_git(root, &["init", "-q"]);
    run_git(root, &["config", "user.email", "test@example.com"]);
    run_git(root, &["config", "user.name", "Test"]);

    for name in ["one.txt", "two.txt", "three.txt"] {
        std::fs::write(dir.path().join(name), format!("{name}\n")).expect("write fixture");
    }
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-q", "-m", "add fixtures"]);

    for name in ["one.txt", "two.txt", "three.txt"] {
        std::fs::remove_file(dir.path().join(name)).expect("remove fixture");
    }
    run_git(root, &["add", "-A"]);
    run_git(root, &["commit", "-q", "-m", "delete fixtures"]);

    let summary = gitdiff_workspace(
        root,
        "HEAD~1",
        Some("HEAD"),
        Fidelity::Medium,
        Some(2),
        None,
    )
    .expect("gitdiff_workspace");

    assert_eq!(
        summary.file_count, 3,
        "all discovered files must be reported"
    );
    assert_eq!(
        summary.counts,
        (0, 2, 0, 0),
        "only the bounded processing set contributes processed counts"
    );
    assert_eq!(
        summary.skipped, 1,
        "the truncated remainder must be skipped"
    );
    assert_eq!(
        summary.counts.0 + summary.counts.1 + summary.counts.2 + summary.counts.3 + summary.skipped,
        summary.file_count,
        "processed counts plus skipped must equal the discovered total"
    );
    assert!(
        summary
            .manifest
            .starts_with("§GITDIFF HEAD~1..HEAD (3 files)"),
        "manifest header must report the discovered total: {}",
        summary.manifest.lines().next().unwrap_or("")
    );
    assert_eq!(
        summary
            .manifest
            .lines()
            .filter(|line| line.starts_with("- FILE "))
            .count(),
        2,
        "the manifest body must remain bounded by max_files"
    );
}
