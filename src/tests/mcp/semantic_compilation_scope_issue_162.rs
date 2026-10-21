use crate::compression::Fidelity;
use crate::ir::typescript_resolution::{Resolution, Witness};
use crate::mcp::semantic_compilation::executor::{CompilationJob, SemanticCompilerExecutor};
use std::path::Path;
use std::sync::Arc;

fn fixture(root: &Path) -> (String, String) {
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    std::fs::write(root.join("leaf.ts"), "export class Leaf {}").unwrap();
    let source = "import { Leaf } from './leaf'; export class App {}".to_string();
    let file = root.join("app.ts");
    std::fs::write(&file, &source).unwrap();
    (file.to_string_lossy().into_owned(), source)
}

fn assert_leaf(artifact: &super::artifact::SemanticCompilationArtifact) {
    let evidence = artifact.typescript_authority();
    assert!(!evidence.source_only);
    assert!(evidence.disposition.is_none(), "{:?}", evidence.disposition);
    assert!(evidence.references.iter().any(|(name, _, resolution)| {
        name == "Leaf"
            && matches!(resolution, Resolution::Resolved(id) if id.source.ends_with("leaf.ts"))
    }));
    assert!(evidence.witnesses.iter().any(|witness| {
        matches!(witness, Witness::Content { path, .. } if path.ends_with("tsconfig.json"))
    }));
}

#[test]
fn issue_162_compilation_primary_root_reaches_direct_candidate() {
    let root = tempfile::tempdir().unwrap();
    let (file, _) = fixture(root.path());
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let root_text = root.path().to_str().unwrap();
    let artifact = crate::mcp::tool_helpers::compile_file_ir_candidate(
        &file,
        Fidelity::High,
        &state,
        Some(root_text),
    )
    .unwrap();
    assert_leaf(&artifact);
    assert!(
        state.alias_for_path(&file).is_none(),
        "candidate must not install alias ownership"
    );
}

#[test]
fn issue_162_compilation_in_memory_candidate_preserves_snapshot_and_scope() {
    let root = tempfile::tempdir().unwrap();
    let (file, source) = fixture(root.path());
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let edited = source.replace("class App", "class Edited");
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        &file,
        &edited,
        Fidelity::High,
        &state,
        root.path().to_str(),
    )
    .unwrap();
    assert_leaf(&artifact);
    assert!(
        artifact
            .typescript_authority()
            .declarations
            .iter()
            .any(|id| id.name == "Edited")
    );
    assert_eq!(
        artifact.typescript_authority().source_hash,
        crate::cache::content_hash(edited.as_bytes())
    );
    assert_eq!(std::fs::read_to_string(file).unwrap(), source);
}

#[test]
fn issue_162_compilation_batch_preserves_authorized_primary_root() {
    let root = tempfile::tempdir().unwrap();
    let (file, source) = fixture(root.path());
    let config = crate::tests::test_config();
    let executor = SemanticCompilerExecutor::new(1);
    let results = executor.compile(
        &config,
        vec![CompilationJob {
            authorized_roots: vec![root.path().to_path_buf()],
            file_path: file.clone(),
            source: Arc::new(source),
            fidelity: Fidelity::High,
            path_alias: file.clone(),
            previous_version: 0,
            canonical_owner: file,
            skip_set: None,
        }],
    );
    assert_eq!(results.len(), 1);
    assert_leaf(results[0].result.as_ref().unwrap());
}
