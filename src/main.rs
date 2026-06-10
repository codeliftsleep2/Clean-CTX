// src/main.rs — Clean-CTX MCP Server + CLI
//
// The entire server lives in `clean_ctx::mcp`. When run with no
// arguments, this file starts the MCP server. When run with the
// `init` subcommand, it creates a default `.clean-ctx.json` config
// and `.clean-ctx/` directory in the current directory.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "init" {
        return cmd_init();
    }

    // Default: run the MCP server
    clean_ctx::mcp::run()
}

/// Handle `clean-ctx init` — create default config and directory.
fn cmd_init() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;

    // Create .clean-ctx/ directory (persistence-ready)
    let clean_ctx_dir = cwd.join(".clean-ctx");
    if !clean_ctx_dir.exists() {
        std::fs::create_dir_all(&clean_ctx_dir)?;
        eprintln!("[clean-ctx] Created directory: {}", clean_ctx_dir.display());
    } else {
        eprintln!("[clean-ctx] Directory already exists: {}", clean_ctx_dir.display());
    }

    // Create .clean-ctx.json config file
    let config_path = cwd.join(".clean-ctx.json");
    if config_path.exists() {
        eprintln!("[clean-ctx] Config already exists: {}", config_path.display());
        eprintln!("[clean-ctx] Delete it first if you want to regenerate defaults.");
        return Ok(());
    }

    let config_content = generate_default_config();
    std::fs::write(&config_path, &config_content)?;
    eprintln!("[clean-ctx] Created config: {}", config_path.display());
    eprintln!();
    eprintln!("  Next steps:");
    eprintln!("    1. Review and edit .clean-ctx.json to match your project.");
    eprintln!("    2. Start the MCP server: clean-ctx");
    eprintln!("    3. Use `provide_code_context` as the unified entry point.");
    eprintln!();

    Ok(())
}

/// Generate the default `.clean-ctx.json` content.
fn generate_default_config() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "enabled": true,
        "defaultFidelity": "medium",
        "autoAngular": true,
        "autoDelta": true,
        "persistence": {
            "enabled": false,
            "autoSave": true,
            "maxHistoryDays": 30,
            "dbPath": ".clean-ctx/persistence.db"
        },
        "heuristics": {
            "largeFileThreshold": 300,
            "forceHighFidelity": ["*.service.ts", "*.component.ts", "*.guard.ts"],
            "useAngularMeta": true
        },
        "smartDefaults": {
            "refactor": "high",
            "overview": "low",
            "debug": "medium",
            "edit": "low",
            "implement": "medium"
        }
    }))
    .unwrap_or_else(|_| "{}".to_string())
}
