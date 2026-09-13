use super::*;

pub(super) fn shared_fixture_root() -> std::path::PathBuf {
    std::env::current_dir()
        .expect("resolve test working directory")
        .join("target")
        .join(format!("cbm-live-e2e-{}", std::process::id()))
}

pub(super) fn prepare_shared_fixture() -> std::path::PathBuf {
    let root = shared_fixture_root();
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).expect("create shared CBM fixture directory");
    std::fs::write(
        root.join("src/service.ts"),
        "class MyService {\n  doSomething() {\n    return 42;\n  }\n}\n",
    )
    .expect("write shared edit fixture");
    root
}

/// Multi-root, multilingual live-CBM integration test (CBM 0.8.1).
/// Exercises fixture indexing and multilingual graph queries against live CBM.
///
/// Fully self-contained and machine-agnostic: generates a deterministic
/// multilingual fixture (Rust, C#, Java, TypeScript/Angular, JavaScript,
/// HTML, CSS) under Cargo's target directory and registers it as an additional
/// root, proving the complete lifecycle:
/// additional_root → canonical slug → index_repository → readiness →
/// query → cross-language resolution.
#[serial(cbm_live)]
#[test]
fn e2e_cbm_multiroot_multilingual_integration() {
    if !cbm_binary_exists() {
        eprintln!("SKIP — CBM not installed");
        return;
    }
    let state = shared_live_state();

    // ── Multilingual fixture (self-contained, machine-agnostic) ──
    // Generate a temporary polyglot project so every developer/CI machine
    // exercises the identical CBM wire path. No external repositories.
    // eprintln!("\n═══ Step 9: Create multilingual fixture project ═══");
    let fixture_root = shared_fixture_root();
    std::fs::create_dir_all(fixture_root.join("src")).expect("create fixture dir");

    // Rust fixture — CBM emits Function nodes from .rs files
    std::fs::write(
        fixture_root.join("src/fixture_core.rs"),
        r#"
pub struct FixtureCore { value: String }

impl FixtureCore {
    pub fn new(value: &str) -> Self { Self { value: value.to_string() } }
    pub fn fixture_value(&self) -> &str { &self.value }
}
"#,
    )
    .expect("write fixture_core.rs");

    // C# fixture — CBM emits Class/Interface/Method nodes from .cs files.
    // Get/Create/Update/Delete are the cross-language resolution targets.
    std::fs::write(
        fixture_root.join("src/FixtureEngine.cs"),
        r#"
using System;
namespace AuditFixture {
    public interface IFixtureEngine { string Get(string key); void Create(string val); }
    public class FixtureEngine : IFixtureEngine {
        private readonly Dictionary<string, string> _cache = new();
        public string Get(string key) { return _cache.TryGetValue(key, out var v) ? v : ""; }
        public void Create(string val) { _cache[val] = val; }
        public void Update(string key, string val) { _cache[key] = val; }
        public bool Delete(string key) { return _cache.Remove(key); }
    }
}
"#,
    )
    .expect("write FixtureEngine.cs");

    // Java fixture — at minimum indexed as a File node
    std::fs::write(
        fixture_root.join("src/FixtureGateway.java"),
        r#"
public class FixtureGateway {
    public String fetchById(String id) { return id; }
    public void save(String record) { }
}
"#,
    )
    .expect("write FixtureGateway.java");

    // TypeScript fixture — Angular service (meta-layer representative)
    std::fs::write(
        fixture_root.join("src/fixture-client.service.ts"),
        r#"
import { Injectable } from '@angular/core';

export interface FixtureDto { id: string; label: string; }

@Injectable({ providedIn: 'root' })
export class FixtureClientService {
    load(id: string): FixtureDto { return { id, label: 'fixture' }; }
}
"#,
    )
    .expect("write fixture-client.service.ts");

    // JavaScript fixture — CBM emits File/Module nodes from .js files
    std::fs::write(
        fixture_root.join("src/fixture_app.js"),
        r#"
function getFixture(id) { return { id }; }
function createFixture(data) { return data; }
"#,
    )
    .expect("write fixture_app.js");

    // HTML fixture
    std::fs::write(
        fixture_root.join("index.html"),
        "<html><body><h1>AuditFixture</h1></body></html>",
    )
    .expect("write index.html");

    // CSS fixture — indexed as a File node
    std::fs::write(
        fixture_root.join("src/fixture_styles.css"),
        ".fixture-root { color: rebeccapurple; }",
    )
    .expect("write fixture_styles.css");

    // Verify every language fixture exists before indexing
    for f in [
        "src/fixture_core.rs",
        "src/FixtureEngine.cs",
        "src/FixtureGateway.java",
        "src/fixture-client.service.ts",
        "src/fixture_app.js",
        "index.html",
        "src/fixture_styles.css",
    ] {
        assert!(fixture_root.join(f).exists(), "fixture must exist: {f}");
    }

    let fixture_str = fixture_root.to_string_lossy().to_string();

    let index_result = {
        let mut guard = state.graph_bridge_lock();
        guard
            .as_mut()
            .expect("live bridge")
            .reindex_for_file(&fixture_root.join("src/fixture_core.rs"), "fast")
    };
    index_result.expect("index multilingual fixture through shared CBM");

    let fx_slug = {
        let mut g = state.graph_bridge_lock();
        let b = g.as_mut().expect("lf bridge");
        // Resolve the fixture project's canonical CBM slug via its path
        let s = b.resolve_project_id(&fixture_str);
        // eprintln!("  Fixture slug: {s}");
        // eprintln!("  primary slug: {}", b.project_str());
        s
    };
    let fixture_workspace = activate_shared_workspace(&state, &fixture_root);

    // eprintln!("\n═══ Step 11: Fixture architecture ═══");
    {
        let mut g = state.graph_bridge_lock();
        let b = g.as_mut().expect("lf bridge");
        // Switch to the fixture project before querying
        b.set_project(&fx_slug);
        b.invalidate_cache();
        let arch = b
            .get_architecture()
            .unwrap_or_else(|e| panic!("get_architecture must succeed on the fixture: {e}"));
        // eprintln!(
        //     "  {} module(s), {} dep(s)",
        //     arch.modules.len(),
        //     arch.dependencies.len()
        // );
        // for m in &arch.modules {
        //     eprintln!("    module: {} ({} nodes)", m.name, m.file_count);
        // }
        // for d in &arch.dependencies {
        //     eprintln!("    dep: {} -> {} ({})", d.from, d.to, d.kind);
        // }
        assert!(!arch.modules.is_empty(), "Fixture must have modules");
    }

    // eprintln!("\n═══ Step 12: Language-specific symbol discovery ═══");
    {
        let mut g = state.graph_bridge_lock();
        let b = g.as_mut().expect("fixture bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        // Hard requirements: every language must surface its fixture symbol.
        for sym in &[
            "FixtureEngine",      // C# Class/Interface/Method
            "FixtureCore",        // Rust struct
            "FixtureGateway",     // Java class / File node
            "index.html",         // HTML File node
            "fixture_styles.css", // CSS File node
        ] {
            let nodes = b.search(sym);
            // eprintln!("  search(\"{sym}\") = {} hits", nodes.len());
            // for n in nodes.iter().take(3) {
            //     eprintln!("    - {} {} ({})", n.label, n.name, n.file);
            // }
            assert!(
                !nodes.is_empty(),
                "search(\"{sym}\") must find its fixture node"
            );
        }
        let cs = b.search("FixtureEngine");
        assert!(
            cs.iter()
                .any(|n| n.label == "Class" || n.label == "Interface"),
            "C# fixture must yield Class/Interface nodes, got: {:?}",
            cs.iter()
                .map(|n| (n.label.as_str(), n.name.as_str()))
                .collect::<Vec<_>>()
        );
        // Informational: TS symbol parsing support varies by CBM build.
        // let ts = b.search("FixtureClientService");
        // eprintln!(
        //     "  search(\"FixtureClientService\") [TS] = {} hits{}",
        //     ts.len(),
        //     if ts.is_empty() {
        //         " (file-level only)"
        //     } else {
        //         ""
        //     }
        // );
    }

    // eprintln!("\n═══ Step 13: Method, Class and Function nodes ═══");
    {
        let mut g = state.graph_bridge_lock();
        let b = g.as_mut().expect("fixture bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        // LIMIT must exceed the fixture's total Method count (10+ across
        // C#/Java/TS/Rust) or rows get silently truncated.
        let q1 = "MATCH (m:Method) RETURN m.name LIMIT 50".to_string();
        let qr1 = b.query_graph(&q1);
        let methods: Vec<&str> = qr1.nodes.iter().map(|n| n.name.as_str()).collect();
        // eprintln!("  Methods: {methods:?}");
        assert!(!qr1.nodes.is_empty(), "Fixture must have Method nodes");
        assert!(
            methods.contains(&"Get") && methods.contains(&"Create"),
            "C# methods Get/Create must appear as Method nodes, got: {methods:?}"
        );
        let q2 = "MATCH (c:Class) RETURN c.name LIMIT 10".to_string();
        let qr2 = b.query_graph(&q2);
        let classes: Vec<&str> = qr2.nodes.iter().map(|n| n.name.as_str()).collect();
        eprintln!("  Classes: {classes:?}");
        assert!(
            classes.contains(&"FixtureEngine"),
            "C# class FixtureEngine must appear as a Class node, got: {classes:?}"
        );
        // Rust: fixture_value must be reachable as Function OR Method node.
        let q3 = "MATCH (f:Function) RETURN f.name LIMIT 30".to_string();
        let qr3 = b.query_graph(&q3);
        let fns: Vec<&str> = qr3.nodes.iter().map(|n| n.name.as_str()).collect();
        // eprintln!("  Functions: {fns:?}");
        assert!(
            fns.contains(&"fixture_value") || methods.contains(&"fixture_value"),
            "Rust fn fixture_value must appear as a Function/Method node, got fns={fns:?} methods={methods:?}"
        );
    }

    // eprintln!("\n═══ Step 14: Web-file nodes (JS/HTML/TS/CSS) ═══");
    {
        let mut g = state.graph_bridge_lock();
        let b = g.as_mut().expect("fixture bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        for ext in &["js", "html", "ts", "css"] {
            let q = format!(
                "MATCH (n) WHERE n.file_path =~ '.*\\.{ext}$' RETURN n.name, n.file_path LIMIT 5"
            );
            let qr = b.query_graph(&q);
            // eprintln!("  {} nodes: {}", ext.to_uppercase(), qr.nodes.len());
            // for n in &qr.nodes {
            //     eprintln!("    {} ({})", n.name, n.file);
            // }
            assert!(
                !qr.nodes.is_empty(),
                ".{ext} fixture must produce graph nodes"
            );
        }
        // Razor remains intentionally unsupported by CBM — no assertion.
    }

    // eprintln!("\n═══ Step 15: C# cross-language resolution ═══");
    {
        let mut g = state.graph_bridge_lock();
        let b = g.as_mut().expect("fixture bridge");
        for name in &["Get", "Create", "Update", "Delete"] {
            let result = b.resolve_cross_language_endpoint(name);
            // eprintln!("  resolve(\"{name}\") = {result:?}");
            let endpoint =
                result.unwrap_or_else(|| panic!("cross-language resolve(\"{name}\") must succeed"));
            assert!(
                endpoint.contains("FixtureEngine"),
                "resolve(\"{name}\") must map into FixtureEngine, got \"{endpoint}\""
            );
        }
    }

    // eprintln!("\n═══ Step 16: Primary bridge still healthy ═══");
    drop(fixture_workspace);
    {
        let mut g = state.graph_bridge_lock();
        let b = g.as_mut().expect("live bridge");
        let c1 = b.search("GraphBridge").len();
        // eprintln!("  Primary search(GraphBridge) = {c1} hits");
        assert!(c1 > 0, "Primary bridge must still be queryable");
    }

    // eprintln!("\n═══ Audit probe complete — all steps passed ═══\n");
}
