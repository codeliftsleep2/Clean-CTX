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

    if args.len() > 1 && args[1] == "setup" && args.get(2).map(|s| s.as_str()) == Some("--with-cbm") {
        let force = args.get(3).map(|s| s.as_str()) == Some("--force");
        return cmd_setup_cbm(force);
    }

    // A-15: --config-dump flag to print resolved configuration
    if args.len() > 1 && args[1] == "--config-dump" {
        return cmd_config_dump();
    }

    // Default: run the MCP server
    // Note: --with-cbm is the default behavior now (auto-detect CBM on PATH).
    // Use CBM_DISABLE=1 env var to explicitly disable CBM integration.
    clean_ctx::mcp::run()
}

/// Handle `clean-ctx setup --with-cbm` — check CBM availability and
/// optionally generate config.
///
/// P3-22: Added `--force` flag to skip confirmation prompt. Without `--force`,
/// the user is prompted to confirm before modifying `.clean-ctx.json`.
fn cmd_setup_cbm(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let info = clean_ctx::cbm::setup::cbm_setup_check();
    let output = clean_ctx::cbm::setup::format_setup_output(&info);
    eprint!("{}", output);

    // If CBM is found and ready, offer to write binary_path into .clean-ctx.json
    if info.is_ready {
        let config_path = std::path::Path::new(".clean-ctx.json");
        if config_path.exists() {
            // P3-22: Ask for confirmation unless --force is set
            if !force {
                eprintln!();
                eprintln!("[clean-ctx] This will update .clean-ctx.json with CBM configuration.");
                eprint!("[clean-ctx] Proceed? [y/N] ");
                // Flush stdout to ensure prompt appears before input
                use std::io::Write;
                std::io::stderr().flush()?;
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();
                
                if input != "y" && input != "yes" {
                    eprintln!("[clean-ctx] Aborted.");
                    return Ok(());
                }
            }
            
            // Read existing config and merge cbm block
            match std::fs::read_to_string(config_path) {
                Ok(content) => {
                    if let Ok(mut config_val) = serde_json::from_str::<serde_json::Value>(&content) {
                        let cbm_block = clean_ctx::cbm::setup::generate_cbm_config_block(&info);
                        config_val["cbm"] = cbm_block;
                        if let Ok(pretty) = serde_json::to_string_pretty(&config_val) {
                            std::fs::write(config_path, &pretty)?;
                            eprintln!("[clean-ctx] Updated .clean-ctx.json with CBM configuration.");
                        }
                    }
                }
                Err(_) => {
                    eprintln!("[clean-ctx] Could not read .clean-ctx.json to update. Update manually.");
                }
            }
        } else {
            eprintln!("[clean-ctx] No .clean-ctx.json found. Run `clean-ctx init` first, then rerun setup.");
        }
    }

    Ok(())
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

/// Handle `clean-ctx --config-dump` — print the resolved configuration.
///
/// A-15: Shows the final configuration after all precedence rules
/// (tool args > env vars > config file > defaults) have been applied.
/// Useful for debugging configuration issues.
fn cmd_config_dump() -> Result<(), Box<dyn std::error::Error>> {
    let config = clean_ctx::config::CleanCtxConfig::load(std::path::Path::new("."));
    
    eprintln!("═══════════════════════════════════════════════════════════════════════════════════════");
    eprintln!("  Clean-CTX Resolved Configuration");
    eprintln!("═══════════════════════════════════════════════════════════════════════════════════════");
    eprintln!();
    
    // Print config file location if found
    if let Some(config_path) = clean_ctx::config::CleanCtxConfig::find_config(std::path::Path::new(".")) {
        eprintln!("  Config file: {}", config_path.display());
    } else {
        eprintln!("  Config file: (none found — using defaults)");
    }
    eprintln!();
    
    // Print CI environment detection
    let is_ci = clean_ctx::config::CleanCtxConfig::is_ci_environment();
    eprintln!("  CI environment detected: {}", is_ci);
    if is_ci {
        eprintln!("  ⚠️  Persistence auto-disabled in CI");
    }
    eprintln!();
    
    // Print effective persistence setting
    eprintln!("  Persistence enabled: {}", config.persistence.enabled);
    eprintln!();
    
    // Print full config as pretty JSON
    eprintln!("  Full configuration:");
    eprintln!("  ────────────────────────────────────────────────────────────────────────────────────");
    
    // Convert to JSON and print with indentation
    let config_json = serde_json::to_string_pretty(&config)?;
    for line in config_json.lines() {
        eprintln!("  {}", line);
    }
    
    eprintln!("  ────────────────────────────────────────────────────────────────────────────────────");
    eprintln!();
    eprintln!("  Configuration precedence (highest to lowest):");
    eprintln!("    1. Tool argument (per-call overrides)");
    eprintln!("    2. Environment variable (proxy settings, CI detection)");
    eprintln!("    3. Config file (.clean-ctx.json)");
    eprintln!("    4. Default (built-in defaults)");
    eprintln!();
    eprintln!("  For more information, see: docs/CONFIGURATION.md");
    
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
            "enabled": true,
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
        },
        "cache": {
            "enabled": true,
            "system_prompt_ttl": "1h",
            "tools_ttl": "1h",
            "baseline_ttl": "1h",
            "tail_ttl": "5m",
            "vocab_version": "v1",
            "tool_defs_version": "v1"
        }
    }))
    .unwrap_or_else(|_| "{}".to_string())
}
