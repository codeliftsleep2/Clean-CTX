use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;

fn check(count: usize, fidelity: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("RegistrationProbe.cs");
    let calls = (0..count)
        .map(|i| format!("    services.AddScoped<IReg{i:03}, Impl{i:03}>();"))
        .collect::<Vec<_>>();
    let source = format!(
        "{}\nusing Microsoft.Extensions.DependencyInjection;\npublic class RegistrationProbe {{\n public void Register() {{\n{}\n }}\n}}\n",
        "// unchanged padding\n".repeat(65),
        calls.join("\n")
    );
    std::fs::write(&path, &source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({"arguments": {
            "workspaceRoot":root.path(), "filePath":path, "fidelity":fidelity
        }}),
        &state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let facts: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("T @di = "))
        .collect();
    assert_eq!(
        facts.len(),
        count,
        "{fidelity}: source oracle {count}, delivered {}: {text}",
        facts.len()
    );
    for (i, fact) in facts.iter().enumerate() {
        assert!(
            fact.contains(&format!("IReg{i:03} → AddScoped<Impl{i:03}>")),
            "{fact}"
        );
    }
}

#[test]
fn di_omission_zero_complete() {
    for fidelity in ["medium", "high"] {
        check(0, fidelity);
    }
}
#[test]
fn di_omission_nine_complete() {
    for fidelity in ["medium", "high"] {
        check(9, fidelity);
    }
}
#[test]
fn di_omission_ten_complete() {
    for fidelity in ["medium", "high"] {
        check(10, fidelity);
    }
}
#[test]
fn di_omission_eleven_complete() {
    for fidelity in ["medium", "high"] {
        check(11, fidelity);
    }
}
#[test]
fn di_omission_forty_complete() {
    for fidelity in ["medium", "high"] {
        check(40, fidelity);
    }
}

#[test]
fn di_omission_distinct_contents_and_existing_order() {
    let source = "public class Probe { void Register() { services.AddSingleton<IShared, First>(); services.AddScoped<IShared, First>(); services.AddScoped<IShared, Second>(); services.AddTransient<IThird, Third>(); } }";
    let block =
        crate::dotnet_meta::general::extract_general(source, crate::compression::Fidelity::High)
            .unwrap();
    let facts: Vec<_> = block
        .lines
        .iter()
        .filter(|line| line.starts_with("Φdi:"))
        .collect();
    assert_eq!(facts.len(), 4);
    assert!(facts[0].contains("AddScoped<First>"));
    assert!(facts[1].contains("AddScoped<Second>"));
    assert!(facts[2].contains("AddSingleton<First>"));
    assert!(facts[3].contains("AddTransient<Third>"));
}
