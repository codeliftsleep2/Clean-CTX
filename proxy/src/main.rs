// proxy/src/main.rs
//
// Clean-CTX Anthropic Prompt-Cache Proxy.
//
// A local HTTP reverse proxy that sits between Cline and api.anthropic.com,
// injecting cache_control breakpoints for ~90% API cost savings.
//
// Usage:
//   export ANTHROPIC_BASE_URL=http://127.0.0.1:8787
//   AUTO_CACHE=1 clean-ctx-proxy
//
// Environment variables (all optional):
//   PORT               : Local port (default: 8787)
//   PROXY_UPSTREAM_URL : Upstream URL (dedicated; takes precedence)
//   COPILOT_BRIDGE_URL : Alias for the Copilot bridge upstream
//   ANTHROPIC_BASE_URL : Legacy upstream URL (default: https://api.anthropic.com)
//   AUTO_CACHE         : Enable cache injection (1/true)
//   TAIL_TTL           : Tail breakpoint TTL (default: "5m")
//   DROP_TOOLS         : Comma-separated tool names to drop
//   STRIP_ANSI         : Strip ANSI codes (default: 1)
//   TRIM_BASH_GIT      : Trim Bash git section (default: 0)
//   MODEL_OVERRIDE     : Override model name
//   LOG_BODIES         : Log request/response bodies (1/true)
//   LOG_DIR            : Log directory (default: .clean-ctx/proxy-logs)

mod config;
mod error;
mod cache;
mod transform;
mod logger;
mod rate_limiter;
mod server;
mod scrub;
mod scrub_patterns;
mod filter_rules;
mod filters;
mod filter_registry;
mod community_filters;
mod filter_stats;
mod filter_loader;
mod pipeline;
mod platform;

use tokio::sync::watch;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::ProxyConfig;
use crate::error::ProxyError;
use crate::server::run_server;

#[tokio::main]
async fn main() -> Result<(), ProxyError> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    // Parse configuration from environment
    let config = ProxyConfig::from_env();

    // Validate configuration (rejects self-forwarding loops).
    config.validate()?;

    // Print non-fatal configuration warnings (e.g. AUTO_CACHE against a local bridge)
    // BEFORE the banner so they're visible without box-formatting truncation.
    for warning in config.warnings() {
        println!("⚠  {warning}");
        info!("[proxy] WARNING: {}", warning);
    }

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║     Clean-CTX Anthropic Proxy                           ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  Listen:     http://127.0.0.1:{:<24}             ", config.port);
    println!("║  Upstream:   {:<34} ", config.upstream_url);
    if config.upstream_url.contains("127.0.0.1") || config.upstream_url.contains("localhost") {
        println!("║              (local upstream — e.g. Copilot bridge)          ");
    }
    println!("║  Auto-cache: {}                                    ", if config.auto_cache { "ON " } else { "OFF" });
    if config.auto_cache {
        println!("║  Tail TTL:   {}                                       ", config.tail_ttl);
    }
    if !config.drop_tools.is_empty() {
        println!("║  Drop tools: {}                              ", config.drop_tools.join(", "));
    }
    println!("║  Strip ANSI: {}                                    ", if config.strip_ansi { "ON " } else { "OFF" });
    if config.trim_bash_git {
        println!("║  Bash trim:  ON                                     ");
    }
    if let Some(ref model) = config.model_override {
        println!("║  Model:      {}                              ", model);
    }
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    info!("Proxy configuration:");
    info!("  Port: {}", config.port);
    info!("  Upstream: {}", config.upstream_url);
    info!("  Auto-cache: {}", config.auto_cache);
    info!("  Strip ANSI: {}", config.strip_ansi);

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Handle SIGINT / SIGTERM for graceful shutdown
    let shutdown_tx_handle = shutdown_tx.clone();
    tokio::spawn(async move {
        // Wait for Ctrl+C
        tokio::signal::ctrl_c().await.ok();
        info!("[proxy] Ctrl+C received, initiating graceful shutdown...");
        let _ = shutdown_tx_handle.send(true);
    });

    // Start the server
    info!("Starting proxy server on 127.0.0.1:{}", config.port);
    info!("Set ANTHROPIC_BASE_URL=http://127.0.0.1:{} in your client", config.port);

    run_server(config, shutdown_rx).await?;

    info!("Proxy server stopped");
    Ok(())
}