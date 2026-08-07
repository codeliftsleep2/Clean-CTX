// src/main.rs — Clean-CTX MCP Server + CLI
//
// The entire server lives in `clean_ctx::mcp`. When run with no
// arguments, this file starts the MCP server. When run with the
// `init` subcommand, it creates a default `.clean-ctx.json` config
// and `.clean-ctx/` directory in the current directory.

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "clean-ctx",
    version = env!("CARGO_PKG_VERSION"),
    about = "Token Waste Reducer & Context Compiler",
)]
enum Cli {
    /// Create default .clean-ctx.json config and .clean-ctx/ directory
    Init,
    /// Check CBM availability and optionally generate config
    Setup {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Print resolved configuration
    #[command(name = "--config-dump")]
    ConfigDump,
    /// Start the Clean-CTX proxy standalone (no MCP server)
    Proxy {
        /// Stop a running auto-started proxy
        #[arg(long)]
        stop: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Idiomatic clap dispatch. With `version` now set on the command builder,
    // clap natively handles `--version` / `-V` / `--help` / `-h` (printing and
    // exiting) with code 0. This prevents MCP clients (Claude Code, VS Code,
    // etc.) that probe the binary with `--version` from hanging in the stdio
    // server loop.
    //
    // When NO subcommand is given, clap emits
    // `DisplayHelpOnMissingArgumentOrSubcommand`, which we intercept to
    // default to running the MCP server. Any future Cli variant is
    // automatically dispatched above — no parallel manual whitelist to keep
    // in sync.
    match Cli::try_parse() {
        Ok(Cli::Init) => cmd_init(),
        Ok(Cli::Setup { force }) => cmd_setup_cbm(force),
        Ok(Cli::ConfigDump) => cmd_config_dump(),
        Ok(Cli::Proxy { stop }) => cmd_proxy(stop),
        // When no arguments are given, clap emits DisplayHelpOnMissingArgumentOrSubcommand.
        // Intercept it to default to running the MCP server (stdio JSON-RPC loop).
        // Any future Cli variant is automatically dispatched above — no parallel
        // manual whitelist to keep in sync.
        Err(e) if e.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            clean_ctx::mcp::run()
        }
        Err(e) => e.exit(),
    }
}

/// Handle `clean-ctx setup` — check CBM availability and optionally generate config.
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

/// Handle `clean-ctx proxy` — start or stop the proxy standalone.
///
/// With no flags, loads `.clean-ctx.json`, spawns the proxy as a child
/// process, prints its PID, and blocks until Ctrl+C (then terminates it).
/// With `--stop`, finds the process listening on the configured port and
/// terminates it.
fn cmd_proxy(stop: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config = clean_ctx::config::CleanCtxConfig::load(std::path::Path::new("."));
    let proxy_cfg = &config.proxy;

    if stop {
        // Find and kill the process listening on the configured port.
        let pid = find_pid_by_port(proxy_cfg.port);
        match pid {
            Some(pid) => {
                #[cfg(windows)]
                {
                    let pid_str = pid.to_string();
                    let _ = std::process::Command::new("taskkill")
                        .arg("/PID")
                        .arg(&pid_str)
                        .arg("/T")
                        .arg("/F")
                        .status();
                }
                #[cfg(not(windows))]
                {
                    // Send SIGTERM, then escalate to SIGKILL after a
                    // 3-second grace period (mirrors shutdown_proxy).
                    let _ = std::process::Command::new("kill")
                        .arg(pid.to_string())
                        .status();
                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
                    loop {
                        if !process_alive(pid) {
                            break;
                        }
                        if std::time::Instant::now() >= deadline {
                            let _ = std::process::Command::new("kill")
                                .arg("-9")
                                .arg(pid.to_string())
                                .status();
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
                eprintln!("[clean-ctx] Stopped proxy on port {} (pid {})", proxy_cfg.port, pid);
                Ok(())
            }
            None => {
                eprintln!("[clean-ctx] No proxy found listening on port {}", proxy_cfg.port);
                Ok(())
            }
        }
    } else {
        // Start the proxy standalone and wait for Ctrl+C.
        // `force = true` — an explicit `clean-ctx proxy` invocation must
        // start the proxy even when `proxy.auto_start` is false (default).
        let cwd = std::env::current_dir()?;
        let mut child = match clean_ctx::proxy_spawner::spawn_proxy(proxy_cfg, &cwd, true)? {
            Some(c) => c,
            None => {
                eprintln!(
                    "[clean-ctx] Proxy could not be started (auto_start disabled or binary not found)."
                );
                return Ok(());
            }
        };
        eprintln!("[clean-ctx] Proxy running — press Ctrl+C to stop.");
        // Block until Ctrl+C, then terminate the child.
        let (tx, rx) = std::sync::mpsc::channel();
        ctrlc::set_handler(move || {
            let _ = tx.send(());
        })?;
        let _ = rx.recv();
        clean_ctx::proxy_spawner::shutdown_proxy(&mut child);
        eprintln!("[clean-ctx] Proxy stopped.");
        Ok(())
    }
}

/// Find the PID of the process listening on the given port.
///
/// Uses `netstat -ano` on Windows and `lsof -i :PORT` on Unix.
///
/// Windows parsing is strict: only lines in the `LISTENING` state whose
/// **local** address ends in `:{port}` are considered. The previous
/// implementation matched `:{port}` anywhere in the line, which could
/// return the PID of an unrelated process that merely had an outbound
/// connection to a remote host on that port (e.g. an ESTABLISHED line
/// whose foreign address was `1.2.3.4:8787`).
fn find_pid_by_port(port: u16) -> Option<u32> {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("netstat")
            .args(["-ano"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        let needle = format!(":{port}");
        for line in text.lines() {
            let upper = line.to_uppercase();
            // Only consider LISTENING lines.
            if !upper.contains("LISTENING") {
                continue;
            }
            // netstat columns: Proto  Local Address  Foreign Address  State  PID
            let mut cols = line.split_whitespace();
            let _proto = cols.next()?;
            let local = cols.next()?;
            // Local address must end with :{port} (e.g. 127.0.0.1:8787 or [::]:8787).
            if !local.to_uppercase().ends_with(&needle) {
                continue;
            }
            // PID is the last column.
            if let Some(pid_str) = cols.last() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    if pid != 0 {
                        return Some(pid);
                    }
                }
            }
        }
        None
    }
    #[cfg(not(windows))]
    {
        let output = std::process::Command::new("lsof")
            .args(["-ti", &format!(":{port}")])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        text.lines().next()?.trim().parse::<u32>().ok()
    }
}

/// Check whether a process with the given PID is still alive.
///
/// Used by the Unix `--stop` path to wait for a SIGTERM'd proxy to
/// exit before escalating to SIGKILL. On Windows this is not needed
/// (`taskkill /F` is synchronous and forceful).
#[cfg(not(windows))]
fn process_alive(pid: u32) -> bool {
    // `kill -0` probes for process existence without sending a signal.
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "tests/main.rs"]
mod tests;

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