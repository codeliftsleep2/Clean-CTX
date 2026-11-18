use super::*;
use serial_test::serial;

fn write(root: &tempfile::TempDir, path: &str, source: &str) {
    let path = root.path().join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, source).unwrap();
}

fn publish(root: &tempfile::TempDir, state: &crate::mcp::McpState, path: &str) {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({ "arguments": {
            "workspaceRoot": root.path().to_string_lossy(),
            "filePath": path,
            "fidelity": "high"
        }}),
        state,
    );
    let response = pop_response();
    assert!(response.get("result").is_some(), "{response:?}");
}

fn edges(
    root: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    query_type: &str,
    entity_type: &str,
    name: &str,
) -> Vec<serde_json::Value> {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(2),
        "workspace_query",
        &json!({ "arguments": {
            "workspaceRoot": root.path().to_string_lossy(),
            "type": query_type,
            "domain": "dotnet",
            "entity_type": entity_type,
            "name": name
        }}),
        state,
    );
    let response = pop_response();
    assert!(response.get("error").is_none(), "{response:?}");
    response["result"]["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .clone()
}

fn entity_names(edges: &[serde_json::Value]) -> Vec<&str> {
    edges
        .iter()
        .filter(|edge| edge["relation"] == "HasEntity")
        .map(|edge| edge["object"]["name"].as_str().unwrap())
        .collect()
}

#[test]
#[serial]
fn direct_explicit_and_dbset_entities_are_exact_and_deduplicated() {
    let root = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&root, "App.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\" />");
    write(
        &root,
        "Entities.cs",
        "namespace App; public class Existing {} public class Direct {} public class Explicit {}",
    );
    write(
        &root,
        "ExplicitConfiguration.cs",
        r#"using Microsoft.EntityFrameworkCore;
namespace App;
public class ExplicitConfiguration : IEntityTypeConfiguration<Explicit> {}
"#,
    );
    write(
        &root,
        "AppDbContext.cs",
        r#"using Microsoft.EntityFrameworkCore;
namespace App;
public class AppDbContext : DbContext {
 public DbSet<Existing> ExistingRows { get; set; }
 protected override void OnModelCreating(ModelBuilder builder) {
  builder.Entity<Direct>();
  builder.ApplyConfiguration(new ExplicitConfiguration());
  builder.Entity<Existing>();
 }
}
"#,
    );
    for file in ["Entities.cs", "ExplicitConfiguration.cs", "AppDbContext.cs"] {
        publish(&root, &state, file);
    }

    let forward = edges(&root, &state, "forward_edges", "DbContext", "AppDbContext");
    assert_eq!(
        entity_names(&forward),
        vec!["Existing", "Direct", "Explicit"]
    );
    let reverse = edges(&root, &state, "reverse_edges", "Entity", "Explicit");
    assert_eq!(reverse.len(), 1, "{reverse:?}");
    assert_eq!(reverse[0]["subject"]["name"], "AppDbContext");
}

#[test]
#[serial]
fn proven_project_assembly_scan_returns_ten_entities_and_excludes_other_projects() {
    let root = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&root, "App.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\" />");
    let mut source = String::from("using Microsoft.EntityFrameworkCore; namespace App;\n");
    for index in 0..10 {
        source.push_str(&format!(
            "public class Entity{index} {{}} public class Config{index} : IEntityTypeConfiguration<Entity{index}> {{}}\n"
        ));
    }
    write(&root, "Configurations.cs", &source);
    write(
        &root,
        "AppDbContext.cs",
        r#"using Microsoft.EntityFrameworkCore;
namespace App;
public class AppDbContext : DbContext {
 protected override void OnModelCreating(ModelBuilder builder) {
  builder.ApplyConfigurationsFromAssembly(typeof(AppDbContext).Assembly);
 }
}
"#,
    );
    write(
        &root,
        "Other/Other.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\" />",
    );
    write(
        &root,
        "Other/Foreign.cs",
        "using Microsoft.EntityFrameworkCore; namespace Other; public class Foreign {} public class ForeignConfig : IEntityTypeConfiguration<Foreign> {}",
    );
    for file in ["Configurations.cs", "AppDbContext.cs", "Other/Foreign.cs"] {
        publish(&root, &state, file);
    }

    let forward = edges(&root, &state, "forward_edges", "DbContext", "AppDbContext");
    let names = entity_names(&forward);
    assert_eq!(names.len(), 10, "{forward:?}");
    assert!((0..10).all(|index| names.contains(&format!("Entity{index}").as_str())));
    assert!(!names.contains(&"Foreign"));
}

#[test]
#[serial]
fn unproven_assembly_expression_and_removed_registration_fail_closed() {
    let root = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&root, "App.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\" />");
    write(
        &root,
        "Configured.cs",
        "using Microsoft.EntityFrameworkCore; public class Configured {} public class Config : IEntityTypeConfiguration<Configured> {}",
    );
    write(
        &root,
        "AppDbContext.cs",
        "using Microsoft.EntityFrameworkCore; public class AppDbContext : DbContext { protected override void OnModelCreating(ModelBuilder builder) { builder.ApplyConfigurationsFromAssembly(externalAssembly); builder.ApplyConfiguration(new Config()); } }",
    );
    for file in ["Configured.cs", "AppDbContext.cs"] {
        publish(&root, &state, file);
    }
    let forward = edges(&root, &state, "forward_edges", "DbContext", "AppDbContext");
    assert_eq!(entity_names(&forward), vec!["Configured"]);

    write(
        &root,
        "AppDbContext.cs",
        "using Microsoft.EntityFrameworkCore; public class AppDbContext : DbContext { protected override void OnModelCreating(ModelBuilder builder) {} }",
    );
    publish(&root, &state, "AppDbContext.cs");
    let retracted = edges(&root, &state, "forward_edges", "DbContext", "AppDbContext");
    assert!(entity_names(&retracted).is_empty(), "{retracted:?}");
}
