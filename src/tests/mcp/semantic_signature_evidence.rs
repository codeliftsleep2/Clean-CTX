use crate::compression::Fidelity;
use crate::diff::{DiffKind, DiffTarget, build_snapshot, diff_snapshots};
use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;

fn source(ret: &str, attribute: &str, param: &str, body: &str) -> String {
    format!(
        "public class DiffProbe {{\n  {attribute}\n  public {ret} Get({param})\n  {{\n    return {body};\n  }}\n  public int Neighbor()\n  {{\n    return 9;\n  }}\n}}\n"
    )
}

// The first-method body/next-marker defect is a separate #137 slice. Preserve
// the already-supported last-method body contract; declaration changes above
// retain their original following-neighbor stability oracle.
fn body_source(value: &str) -> String {
    format!(
        "public class DiffProbe {{\n  public int Neighbor() {{ return 9; }}\n  public int Get()\n  {{\n    return {value};\n  }}\n}}\n"
    )
}

fn changed(before: &str, after: &str) {
    let a = build_snapshot(before, Fidelity::Medium).unwrap();
    let b = build_snapshot(after, Fidelity::Medium).unwrap();
    assert_eq!(
        a.classes[0].methods[0].sig, b.classes[0].methods[0].sig,
        "compact display should stay compact; comparison needs independent evidence"
    );
    let changes = diff_snapshots(&a, &b);
    assert!(
        changes.iter().any(|a| a.kind == DiffKind::Modified
            && a.target == DiffTarget::Method
            && a.label == "method Get"),
        "{changes:?}"
    );
    assert!(
        !changes
            .iter()
            .any(|a| a.kind == DiffKind::Modified && a.label == "method Neighbor")
    );
}

#[test]
fn semantic_signature_evidence_return_type_survives_display_reduction() {
    changed(&source("int", "", "", "1"), &source("long", "", "", "1"));
}

#[test]
fn semantic_signature_evidence_authorization_survives_same_reduction() {
    changed(
        &source("int", "[Authorize(Roles = \"Reader\")]", "", "1"),
        &source("int", "[Authorize(Roles = \"Admin\")]", "", "1"),
    );
}

#[test]
fn semantic_signature_evidence_generic_and_constructor_preservation() {
    let source = "public class Probe {\n public Probe(int x) {}\n public T Get<T>(T value) { return value; }\n}\n";
    let before = build_snapshot(source, Fidelity::Medium).unwrap();
    let after = build_snapshot(&format!("// comment only\n{source}"), Fidelity::High).unwrap();
    let changes = diff_snapshots(&before, &after);
    assert!(
        changes
            .iter()
            .all(|change| change.kind == DiffKind::Unchanged)
    );
    assert_eq!(before.classes[0].methods.len(), 2);
}

#[cfg(all(feature = "typescript", feature = "java", feature = "rust"))]
#[test]
fn semantic_signature_evidence_other_producers_keep_existing_equality() {
    for source in [
        "export class Probe { get(): number { return 1; } }",
        "package fixture; final class Probe { public int get() { return 1; } }",
        "pub struct Probe; impl Probe { pub fn get(&self) -> i32 { 1 } }",
    ] {
        let before = build_snapshot(source, Fidelity::Low).unwrap();
        let after = build_snapshot(source, Fidelity::High).unwrap();
        assert!(before.classes.iter().any(|class| !class.methods.is_empty()));
        assert!(
            before
                .classes
                .iter()
                .flat_map(|class| &class.methods)
                .all(|method| method.declaration.is_none())
        );
        assert!(
            diff_snapshots(&before, &after)
                .iter()
                .all(|change| change.kind == DiffKind::Unchanged)
        );
    }
}

fn git(root: &std::path::Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn semantic_signature_evidence_registered_committed_history_matrix() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "--quiet"]);
    git(root.path(), &["config", "user.name", "Signature fixture"]);
    git(
        root.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(root.path(), &["config", "commit.gpgsign", "false"]);
    let hooks = root.path().join("empty-hooks");
    std::fs::create_dir(&hooks).unwrap();
    git(
        root.path(),
        &["config", "core.hooksPath", hooks.to_str().unwrap()],
    );
    let cases = [
        (
            "Return.cs",
            source("int", "", "", "1"),
            source("long", "", "", "1"),
            true,
        ),
        (
            "Attribute.cs",
            source("int", "[Authorize(Roles = \"Reader\")]", "", "1"),
            source("int", "[Authorize(Roles = \"Admin\")]", "", "1"),
            true,
        ),
        (
            "Param.cs",
            source("int", "", "int value", "1"),
            source("int", "", "long value", "1"),
            true,
        ),
        ("Body.cs", body_source("1"), body_source("2"), true),
        (
            "Unchanged.cs",
            source("int", "", "", "1"),
            format!("// no semantic change\n{}", source("int", "", "", "1")),
            false,
        ),
    ];
    for (name, a, _, _) in &cases {
        std::fs::write(root.path().join(name), a).unwrap();
    }
    git(root.path(), &["add", "--", "*.cs"]);
    git(root.path(), &["commit", "--quiet", "-m", "before"]);
    let a = git(root.path(), &["rev-parse", "HEAD"]);
    for (name, _, b, _) in &cases {
        std::fs::write(root.path().join(name), b).unwrap();
    }
    git(root.path(), &["add", "--", "*.cs"]);
    git(root.path(), &["commit", "--quiet", "-m", "after"]);
    let b = git(root.path(), &["rev-parse", "HEAD"]);
    assert_eq!(
        git(root.path(), &["diff", "--name-only", &a, &b])
            .lines()
            .count(),
        5
    );
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for fidelity in ["low", "medium", "high", "edit", "verbatim"] {
        crate::protocol::captured_responses().clear();
        dispatch_tools_call(
            &json!(1),
            "diff_commits",
            &json!({"arguments": {
                "workspaceRoot": root.path(), "fromRef": a, "toRef": b, "fidelity": fidelity
            }}),
            &state,
        );
        let answer = crate::protocol::captured_responses().pop().unwrap();
        assert!(answer.get("error").is_none(), "{answer}");
        let text = answer["result"]["content"][0]["text"].as_str().unwrap();
        for (name, _, _, expect_change) in &cases {
            let start = text.find(&format!(": {name} (")).expect("file header");
            let rest = &text[start..];
            let section = rest.split("┌ FILE").next().unwrap();
            assert_eq!(
                section.contains("~ method Get"),
                *expect_change,
                "{fidelity}/{name}: {section}"
            );
            assert!(!section.contains("~ method Neighbor"), "{section}");
            if *expect_change {
                assert!(!section.contains("class DiffProbe (unchanged)"));
            } else {
                assert!(section.contains("class DiffProbe (unchanged)"));
            }
        }
    }
}
