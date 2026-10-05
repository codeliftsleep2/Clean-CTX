use crate::compression::Fidelity;
use crate::gitdiff::engine::gitdiff_workspace;
use crate::gitdiff::workspace::{FileChange, collect_changed_files};

struct TypeChangeRepo {
    dir: tempfile::TempDir,
    regular_commit: String,
}

impl TypeChangeRepo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_str().expect("UTF-8 temp path");

        run_git(root, &["init", "-q"]);
        run_git(root, &["config", "user.email", "test@example.com"]);
        run_git(root, &["config", "user.name", "Test"]);

        std::fs::write(dir.path().join("path.txt"), "regular content\n")
            .expect("write regular file");
        run_git(root, &["add", "path.txt"]);
        run_git(root, &["commit", "-q", "-m", "regular file"]);
        let regular_commit = run_git_output(root, &["rev-parse", "HEAD"])
            .trim()
            .to_string();

        set_gitlink_index_entry(root, &regular_commit);
        run_git(root, &["commit", "-q", "-m", "gitlink object"]);

        Self {
            dir,
            regular_commit,
        }
    }

    fn root(&self) -> &str {
        self.dir.path().to_str().expect("UTF-8 temp path")
    }

    fn prepare_working_tree_change(&self) {
        run_git(
            self.root(),
            &["reset", "-q", "--hard", &self.regular_commit],
        );
        std::fs::remove_file(self.dir.path().join("path.txt")).expect("remove regular file");
        std::fs::create_dir(self.dir.path().join("path.txt")).expect("create gitlink directory");
        set_gitlink_index_entry(self.root(), &self.regular_commit);
    }
}

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

fn run_git_output(root: &str, args: &[&str]) -> String {
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
    String::from_utf8(output.stdout).expect("UTF-8 git output")
}

fn set_gitlink_index_entry(root: &str, commit: &str) {
    let cache_info = format!("160000,{commit},path.txt");
    run_git(root, &["update-index", "--add", "--cacheinfo", &cache_info]);
}

fn assert_git_reports_type_change(root: &str, from: &str, to: Option<&str>) {
    let mut args = vec![
        "diff",
        "--name-status",
        "--find-renames",
        "--end-of-options",
        from,
    ];
    if let Some(to) = to {
        args.push(to);
    }
    assert_eq!(run_git_output(root, &args), "T\tpath.txt\n");
}

fn assert_production_surfaces_type_change(root: &str, from: &str, to: Option<&str>) {
    let changes = collect_changed_files(root, from, to).expect("collect changed files");
    assert_eq!(
        changes,
        vec![FileChange::TypeChanged("path.txt".to_string())]
    );

    let summary =
        gitdiff_workspace(root, from, to, Fidelity::Medium, None, None).expect("gitdiff_workspace");
    assert_eq!(summary.file_count, 1);
    assert_eq!(summary.counts, (0, 0, 0, 0));
    assert_eq!(summary.type_changed, 1);
    assert_eq!(summary.skipped, 0);
    assert_eq!(
        summary.counts.0
            + summary.counts.1
            + summary.counts.2
            + summary.counts.3
            + summary.type_changed
            + summary.skipped,
        summary.file_count
    );
    assert!(summary.manifest.contains("path.txt (type changed)"));
}

#[test]
fn type_change_is_surfaced_for_commit_and_working_tree_comparisons() {
    let repo = TypeChangeRepo::new();

    assert_git_reports_type_change(repo.root(), "HEAD~1", Some("HEAD"));
    assert_production_surfaces_type_change(repo.root(), "HEAD~1", Some("HEAD"));

    repo.prepare_working_tree_change();
    assert_git_reports_type_change(repo.root(), "HEAD", None);
    assert_production_surfaces_type_change(repo.root(), "HEAD", None);
}
