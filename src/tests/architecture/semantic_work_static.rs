//! AF-PERF-001 scoped Rust AST inspection. Not a whole-program call graph.
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::{Node, Parser, Tree};
struct Source {
    module: String,
    text: String,
    tree: Tree,
}
#[derive(Debug, Default)]
struct Findings {
    violations: Vec<String>,
    review: BTreeSet<String>,
}
fn parse(module: &str, text: &str) -> Source {
    let mut parser = Parser::new();
    parser
        .set_language(
            &crate::compression::language::safe_rust_language().expect("Rust feature required"),
        )
        .unwrap();
    let tree = parser.parse(text, None).unwrap();
    assert!(
        !tree.root_node().has_error(),
        "invalid Rust inspection input {module}"
    );
    Source {
        module: module.into(),
        text: text.into(),
        tree,
    }
}
fn text<'a>(n: Node<'_>, s: &'a Source) -> &'a str {
    &s.text[n.byte_range()]
}
fn functions<'a>(
    n: Node<'a>,
    s: &Source,
    out: &mut BTreeMap<String, Node<'a>>,
    ambiguous: &mut BTreeSet<String>,
) {
    if n.kind() == "function_item" {
        let name = text(n.child_by_field_name("name").unwrap(), s);
        // Overloaded impl names are deliberately unresolved rather than guessed.
        let key = format!("{}::{name}", s.module);
        if ambiguous.contains(&key) {
            return;
        }
        if out.insert(key.clone(), n).is_some() {
            out.remove(&key);
            ambiguous.insert(key);
        }
        return;
    }
    for child in n.named_children(&mut n.walk()) {
        functions(child, s, out, ambiguous);
    }
}
fn inspect(sources: &[Source], entry: &str, sinks: &[&str]) -> Findings {
    let mut table = BTreeMap::new();
    for (index, s) in sources.iter().enumerate() {
        let mut local = BTreeMap::new();
        let mut ambiguous = BTreeSet::new();
        functions(s.tree.root_node(), s, &mut local, &mut ambiguous);
        for (name, node) in local {
            table.insert(name, (index, node));
        }
    }
    let mut findings = Findings::default();
    let mut active = BTreeSet::new();
    let &(index, node) = table.get(entry).expect("inspection entry exists");
    let mut walker = Walker {
        sources,
        table: &table,
        sinks,
        active: &mut active,
        out: &mut findings,
    };
    walker.walk(node, index, false, &mut vec![entry.into()]);
    findings
}
struct Walker<'a, 'b> {
    sources: &'a [Source],
    table: &'b BTreeMap<String, (usize, Node<'a>)>,
    sinks: &'b [&'b str],
    active: &'b mut BTreeSet<String>,
    out: &'b mut Findings,
}
impl<'a> Walker<'a, '_> {
    fn walk(&mut self, node: Node<'a>, index: usize, repeated: bool, chain: &mut Vec<String>) {
        let sources = self.sources;
        let table = self.table;
        let sinks = self.sinks;
        let source = &sources[index];
        if node.kind() == "macro_invocation" {
            if repeated {
                self.out.review.insert(format!(
                    "{}:{} macro expansion requires review",
                    source.module,
                    node.start_position().row + 1
                ));
            }
            return;
        }
        if node.kind() == "call_expression" {
            let Some(function) = node.child_by_field_name("function") else {
                return;
            };
            let spelling = text(function, source);
            let method = function
                .child_by_field_name("field")
                .map(|n| text(n, source));
            let args = node.child_by_field_name("arguments");
            // Only this explicit lazy initializer is a reuse boundary. Its closure
            // runs at most once for the same call-local Option, verified by counters.
            if method == Some("get_or_insert_with") {
                if let Some(value) = function.child_by_field_name("value") {
                    self.walk(value, index, repeated, chain);
                }
                return;
            }
            let iteration = matches!(
                method,
                Some(
                    "all"
                        | "any"
                        | "map"
                        | "filter"
                        | "filter_map"
                        | "flat_map"
                        | "for_each"
                        | "fold"
                        | "find"
                )
            );
            if repeated
                && (spelling.contains("std::fs::")
                    || spelling.ends_with("content_hash")
                    || matches!(
                        method,
                        Some("parse" | "canonicalize" | "current" | "contains")
                    ))
            {
                self.out.review.insert(format!(
                    "{}:{} repeated expensive/membership call {spelling}; chain={chain:?}",
                    source.module,
                    node.start_position().row + 1
                ));
            }
            let direct = if function.kind() == "identifier" {
                Some(format!("{}::{spelling}", source.module))
            } else if function.kind() == "scoped_identifier" {
                let parts: Vec<_> = spelling.split("::").collect();
                (parts.len() >= 2)
                    .then(|| format!("{}::{}", parts[parts.len() - 2], parts[parts.len() - 1]))
            } else {
                None
            };
            if let Some(callee) = direct {
                if repeated && sinks.contains(&callee.as_str()) {
                    self.out.violations.push(format!(
                        "{}:{} {chain:?} -> {callee}",
                        source.module,
                        node.start_position().row + 1
                    ));
                } else if let Some(&(next, body)) = table.get(&callee) {
                    if self.active.insert(callee.clone()) {
                        chain.push(callee.clone());
                        self.walk(body, next, repeated, chain);
                        chain.pop();
                        self.active.remove(&callee);
                    } else {
                        self.out.review.insert(format!("recursive call {callee}"));
                    }
                } else if repeated {
                    self.out
                        .review
                        .insert(format!("unresolved direct call {callee}"));
                }
            } else if repeated && !iteration {
                self.out.review.insert(format!(
                    "method/function dispatch requires review: {spelling}"
                ));
            }
            // Walk receiver and arguments separately so only iterator closure bodies
            // acquire the iteration context. Helper-mediated sinks inherit it.
            self.walk(function, index, repeated, chain);
            if let Some(args) = args {
                for arg in args.named_children(&mut args.walk()) {
                    self.walk(
                        arg,
                        index,
                        repeated || (iteration && arg.kind() == "closure_expression"),
                        chain,
                    );
                }
            }
            return;
        }
        let repeated = repeated
            || matches!(
                node.kind(),
                "for_expression" | "while_expression" | "loop_expression"
            );
        for child in node.named_children(&mut node.walk()) {
            self.walk(child, index, repeated, chain);
        }
    }
}
#[test]
fn af_perf_001_ast_direct_and_helper_amplification_detection() {
    for body in [
        "edges.iter().all(|e| project(e))",
        "edges.iter().all(|e| helper(e))",
        "for e in edges { helper(e); }",
    ] {
        let source = parse(
            "fixture",
            &format!("fn entry(){{{body};}} fn helper(e:()){{project(e);}} fn project(e:()){{}}"),
        );
        assert!(
            !inspect(&[source], "fixture::entry", &["fixture::project"])
                .violations
                .is_empty()
        );
    }
    let source = parse(
        "fixture",
        "fn entry(){let mut slot=None; edges.iter().all(|e| {slot.get_or_insert_with(||project()); true});} fn project(){} ",
    );
    assert!(
        inspect(&[source], "fixture::entry", &["fixture::project"])
            .violations
            .is_empty()
    );
    let source = parse("fixture", "fn entry(){edges.iter().all(|e| unknown(e));}");
    assert!(!inspect(&[source], "fixture::entry", &[]).review.is_empty());
    let source = parse(
        "fixture",
        "fn entry(){edges.iter().all(|e| other::project(e));} fn project(){}",
    );
    let findings = inspect(&[source], "fixture::entry", &["fixture::project"]);
    assert!(findings.violations.is_empty());
    assert!(!findings.review.is_empty());
}
#[test]
fn af_perf_001_ast_retained_admission_has_no_known_per_candidate_projection() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = vec![parse(
        "relation_evidence",
        &std::fs::read_to_string(root.join("src/ir/typescript_resolution/relation_evidence.rs"))
            .unwrap(),
    )];
    for module in [
        "route_guards",
        "route_components",
        "route_lazy_components",
        "route_load_children",
        "route_providers",
        "route_resolvers",
        "provider_pair_authority",
        "provider_position",
        "bootstrap_references",
        "application_config_providers",
        "config_composition",
        "provider_imported",
    ] {
        sources.push(parse(
            module,
            &std::fs::read_to_string(root.join(format!("src/angular_meta/{module}.rs"))).unwrap(),
        ));
    }
    let findings = inspect(
        &sources,
        "relation_evidence::admits_relation_evidence",
        &[
            "route_guards::memberships",
            "route_guards::project_relation",
            "route_components::project",
            "route_lazy_components::project",
            "route_load_children::project",
            "route_providers::project",
            "route_resolvers::project",
            "bootstrap_references::project",
            "application_config_providers::project",
            "config_composition::project",
            "provider_imported::project_admission",
            "provider_imported::project",
        ],
    );
    for finding in findings.review {
        eprintln!("AF_REVIEW_REQUIRED {finding}");
    }
    assert!(
        findings.violations.is_empty(),
        "AF-PERF-001 complete projection per candidate: {:?}",
        findings.violations
    );
}

#[path = "index_membership_static_af_perf_002.rs"]
mod index_membership_static;
