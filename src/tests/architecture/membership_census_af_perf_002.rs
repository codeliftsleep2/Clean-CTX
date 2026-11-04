//! Discovery evidence only: AST sites and raw call boundaries, never safety certification.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser};

fn sources(directory: &Path, included: &mut Vec<PathBuf>, excluded: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        // Never follow a symlink outside the declared source roots.
        assert!(
            !std::fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_symlink(),
            "source symlink requires review: {}",
            path.display()
        );
        if path.is_dir() {
            if matches!(
                path.file_name().unwrap().to_str().unwrap(),
                "tests" | "test_files"
            ) {
                excluded.push(path);
            } else {
                sources(&path, included, excluded);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            included.push(path);
        }
    }
}

fn slice<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

fn scan(
    node: Node<'_>,
    source: &str,
    path: &str,
    function: &str,
    iterations: usize,
    sites: &mut Vec<Value>,
    calls: &mut Vec<Value>,
) {
    let function = if node.kind() == "function_item" {
        slice(node.child_by_field_name("name").unwrap(), source)
    } else {
        function
    };
    let manual_loop = matches!(
        node.kind(),
        "for_expression" | "while_expression" | "loop_expression"
    );
    let iteration_depth = iterations + usize::from(manual_loop);
    let mut finding = None;
    if manual_loop {
        finding = Some(("manual-loop", ""));
    } else if node.kind() == "binary_expression"
        && let Some(operator) = node.child_by_field_name("operator")
        && matches!(slice(operator, source), "==" | "!=")
        && iteration_depth > 0
    {
        finding = Some(("equality-in-manual-loop", slice(operator, source)));
    } else if node.kind() == "macro_invocation" {
        finding = Some(("unexpanded-macro-review", ""));
    } else if node.kind() == "call_expression"
        && let Some(callee) = node.child_by_field_name("function")
    {
        let expression = slice(callee, source);
        calls.push(json!({"path":path,"function":function,"call_byte":node.start_byte(),"line":node.start_position().row+1,"callee":expression,"manual_loop_depth":iteration_depth,"resolution":"UNRESOLVED: lexical edge, not type-resolved"}));
        if callee.kind() == "field_expression"
            && let Some(method) = callee.child_by_field_name("field")
        {
            let method = slice(method, source);
            if matches!(
                method,
                "contains"
                    | "any"
                    | "find"
                    | "position"
                    | "rposition"
                    | "filter"
                    | "filter_map"
                    | "retain"
                    | "retain_mut"
                    | "dedup"
                    | "dedup_by"
                    | "dedup_by_key"
                    | "sort"
                    | "sort_by"
                    | "sort_by_key"
                    | "sort_unstable"
                    | "sort_unstable_by"
                    | "sort_unstable_by_key"
                    | "binary_search"
                    | "binary_search_by"
                    | "binary_search_by_key"
                    | "collect"
                    | "keys"
                    | "values"
                    | "values_mut"
                    | "iter"
                    | "iter_mut"
            ) {
                finding = Some(("collection-or-string-operation", expression));
            }
        }
    }
    if let Some((kind, expression)) = finding {
        sites.push(json!({"path":path,"function":function,"byte":node.start_byte(),"line":node.start_position().row+1,"column":node.start_position().column+1,"kind":kind,"expression":expression,"manual_loop_depth":iteration_depth,"classification":"UNRESOLVED","disposition":"MONITOR","inspection":"NOT INSPECTED","limitations":"receiver type, iterator multiplicity, inline cfg(test), macro expansion and delegated ownership require review"}));
    }
    for child in node.named_children(&mut node.walk()) {
        scan(child, source, path, function, iteration_depth, sites, calls);
    }
}

#[test]
fn af002_membership_source_census() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut included = Vec::new();
    let mut excluded = Vec::new();
    for directory in ["src", "proxy/src", "examples"] {
        sources(&root.join(directory), &mut included, &mut excluded);
    }
    // Record build-time Rust if introduced, rather than silently ignoring it.
    for name in ["build.rs", "proxy/build.rs"] {
        if root.join(name).is_file() {
            included.push(root.join(name));
        }
    }
    included.sort();
    included.dedup();
    let mut parser = Parser::new();
    parser
        .set_language(&crate::compression::language::safe_rust_language().unwrap())
        .unwrap();
    let mut sites = Vec::new();
    let mut calls = Vec::new();
    let mut files = Vec::new();
    for path in &included {
        let bytes = std::fs::read(path).unwrap();
        let source = std::str::from_utf8(&bytes).unwrap();
        let path = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let tree = parser.parse(source, None).unwrap();
        let before = sites.len();
        scan(
            tree.root_node(),
            source,
            &path,
            "<module>",
            0,
            &mut sites,
            &mut calls,
        );
        files.push(json!({"path":path,"sha256":format!("{:x}",Sha256::digest(&bytes)),"parse_error":tree.root_node().has_error(),"candidate_sites":sites.len()-before,"scope":"source superset: inline test cfg retained for explicit review"}));
    }
    assert!(
        sites
            .iter()
            .any(|s| s["path"] == "src/workspace/index/declarations.rs")
    );
    assert!(files.iter().any(|f| f["path"] == "proxy/src/cache.rs"));
    let output = root.join("target/af-perf-002/source-wide-membership-census");
    std::fs::create_dir_all(&output).unwrap();
    for (name, value) in [
        ("files.json", json!(files)),
        ("sites.json", json!(sites)),
        ("calls.json", json!(calls)),
    ] {
        std::fs::write(
            output.join(name),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
    }
    std::fs::write(output.join("scope.json"), serde_json::to_vec_pretty(&json!({"roots":["src","proxy/src","examples"],"excluded_directories":excluded,"discovery_complete":false,"type_resolved":false,"macro_expanded":false,"inline_test_cfg_excluded":false,"adversarial_review_complete":false,"note":"AST candidate enumeration is not human classification or fitness acceptance"})).unwrap()).unwrap();
    eprintln!(
        "AF002_CENSUS files={} sites={} calls={} output={}",
        included.len(),
        sites.len(),
        calls.len(),
        output.display()
    );
}

#[test]
fn af002_artifact_parser_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = "src/mcp/semantic_compilation/artifact.rs";
    let bytes = std::fs::read(root.join(path)).unwrap();
    let source = std::str::from_utf8(&bytes).unwrap();
    let mut parser = Parser::new();
    parser
        .set_language(&crate::compression::language::safe_rust_language().unwrap())
        .unwrap();
    let tree = parser.parse(source, None).unwrap();
    let mut pending = vec![tree.root_node()];
    let mut errors = Vec::new();
    let mut functions = Vec::new();
    while let Some(node) = pending.pop() {
        if node.is_error() || node.is_missing() || node.kind() == "function_item" {
            let mut ancestor = node.parent();
            let mut owner = None;
            while let Some(parent) = ancestor {
                if parent.kind() == "impl_item" {
                    owner = parent
                        .child_by_field_name("type")
                        .map(|ty| slice(ty, source));
                    break;
                }
                ancestor = parent.parent();
            }
            let record = json!({"kind":node.kind(),"missing":node.is_missing(),"start_byte":node.start_byte(),"end_byte":node.end_byte(),"start_line":node.start_position().row+1,"end_line":node.end_position().row+1,"impl_owner":owner,"source":slice(node,source)});
            if node.kind() == "function_item" {
                functions.push(record);
            } else {
                errors.push(record);
            }
        }
        pending.extend(node.children(&mut node.walk()));
    }
    assert!(
        !functions.is_empty(),
        "parser must recover some boundaries; no completeness inference"
    );
    let output = root.join("target/af-perf-002/artifact-parser-boundary.json");
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    std::fs::write(&output, serde_json::to_vec_pretty(&json!({"path":path,"source_sha256":format!("{:x}",Sha256::digest(&bytes)),"parse_error":tree.root_node().has_error(),"errors":errors,"recovered_functions":functions,"coverage_accepted":false,"note":"error spans and recovered impl/function boundaries require manual reconciliation with source"})).unwrap()).unwrap();
    eprintln!("AF002_PARSER_BOUNDARY output={}", output.display());
}
