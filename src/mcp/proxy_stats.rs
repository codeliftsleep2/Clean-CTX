// src/mcp/proxy_stats.rs
//
// Proxy statistics fetcher — synchronous HTTP client that queries the
// Clean-CTX proxy's `GET /stats` endpoint to retrieve tool-filtering
// and cache stats for the context_stats dashboard.
//
// Phase 2 (Filter-First Architecture): the MCP server runs synchronously
// (no tokio runtime), so we use `ureq` for blocking HTTP requests.

use serde::{Deserialize, Serialize};

/// Response from the proxy's `GET /stats` endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyStatsResponse {
    pub filter_stats: ProxyFilterStats,
    pub cache_stats: ProxyCacheStats,
}

/// Mirror of the proxy's FilterStats (serializable subset).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyFilterStats {
    pub total_tokens_saved: u64,
    pub total_lines_filtered: u64,
    pub total_applications: u64,
    #[serde(default)]
    pub programs: std::collections::HashMap<String, ProxyProgramFilterStats>,
}

/// Mirror of the proxy's ProgramFilterStats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyProgramFilterStats {
    pub program: String,
    pub applications: u64,
    pub original_tokens: u64,
    pub filtered_tokens: u64,
    pub tokens_saved: u64,
    pub original_lines: u64,
    pub filtered_lines: u64,
    pub lines_removed: u64,
    pub reduction_pct: f32,
}

/// Mirror of the proxy's CacheStats.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyCacheStats {
    #[serde(default)]
    pub total_injected: u64,
    #[serde(default)]
    pub tools_slots: u64,
    #[serde(default)]
    pub system_slots: u64,
    #[serde(default)]
    pub messages_slots: u64,
    #[serde(default)]
    pub tail_slots: u64,
    #[serde(default)]
    pub small_blocks_filtered: u64,
    #[serde(default)]
    pub client_breakpoints_stripped: u64,
    #[serde(default)]
    pub client_breakpoints_preserved: u64,
    #[serde(default)]
    pub cache_read_tokens: u64,
    #[serde(default)]
    pub cache_creation_tokens: u64,
}

/// Fetch proxy stats from a running Clean-CTX proxy instance.
///
/// `proxy_port` is the port the proxy is listening on (default: 8787).
/// Returns `None` if the proxy is unreachable (not running, wrong port, etc.).
pub fn fetch_proxy_stats(proxy_port: u16) -> Option<ProxyStatsResponse> {
    let url = format!("http://127.0.0.1:{}/stats", proxy_port);
    match ureq::get(&url).call() {
        Ok(response) => match response.into_body().read_json::<ProxyStatsResponse>() {
            Ok(stats) => Some(stats),
            Err(e) => {
                eprintln!("[clean-ctx] WARNING: Failed to parse proxy stats: {e}");
                None
            }
        },
        Err(e) => {
            eprintln!("[clean-ctx] Proxy stats unavailable (proxy not running?): {e}");
            None
        }
    }
}

/// Record proxy filter stats into the session stats.
/// Called from `handle_context_stats()` when proxy stats are available.
pub fn record_proxy_filter_stats(
    session_stats: &mut crate::mcp::session_stats::SessionStats,
    proxy_stats: &ProxyStatsResponse,
) {
    // Record tool-filter domain entries for each program
    for (program, pstats) in &proxy_stats.filter_stats.programs {
        session_stats.record_tool_filter(
            program,
            pstats.original_tokens as usize,
            pstats.filtered_tokens as usize,
        );
    }

    // Record cache domain entries from proxy cache stats.
    // Uses REAL token counts from Anthropic's `usage.cache_read_input_tokens`
    // when available. When no real token data exists, do NOT record a hit —
    // recording a hit with 0 tokens would inflate the hit count while showing
    // zero savings, misleading the dashboard.
    if proxy_stats.cache_stats.cache_read_tokens > 0 {
        session_stats.record_cache_hit(proxy_stats.cache_stats.cache_read_tokens as usize);
    }
    // Note: `cache_creation_tokens` (tokens written to cache) are NOT recorded
    // as savings — they represent the cost of writing, not tokens saved.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_stats_serde_roundtrip() {
        let stats = ProxyStatsResponse {
            filter_stats: ProxyFilterStats {
                total_tokens_saved: 475,
                total_lines_filtered: 95,
                total_applications: 1,
                programs: {
                    let mut m = std::collections::HashMap::new();
                    m.insert(
                        "cargo".to_string(),
                        ProxyProgramFilterStats {
                            program: "cargo".to_string(),
                            applications: 1,
                            original_tokens: 500,
                            filtered_tokens: 25,
                            tokens_saved: 475,
                            original_lines: 100,
                            filtered_lines: 5,
                            lines_removed: 95,
                            reduction_pct: 95.0,
                        },
                    );
                    m
                },
            },
            cache_stats: ProxyCacheStats {
                total_injected: 3,
                tools_slots: 3,
                system_slots: 2,
                messages_slots: 1,
                tail_slots: 1,
                small_blocks_filtered: 1,
                client_breakpoints_stripped: 0,
                ..Default::default()
            },
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: ProxyStatsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.filter_stats.total_tokens_saved, 475);
        assert_eq!(deserialized.cache_stats.total_injected, 3);
        assert!(deserialized.filter_stats.programs.contains_key("cargo"));
    }

    #[test]
    fn test_record_proxy_filter_stats() {
        let mut session_stats = crate::mcp::session_stats::SessionStats::new();
        let proxy_stats = ProxyStatsResponse {
            filter_stats: ProxyFilterStats {
                total_tokens_saved: 475,
                total_lines_filtered: 95,
                total_applications: 1,
                programs: {
                    let mut m = std::collections::HashMap::new();
                    m.insert(
                        "cargo".to_string(),
                        ProxyProgramFilterStats {
                            program: "cargo".to_string(),
                            applications: 1,
                            original_tokens: 500,
                            filtered_tokens: 25,
                            tokens_saved: 475,
                            original_lines: 100,
                            filtered_lines: 5,
                            lines_removed: 95,
                            reduction_pct: 95.0,
                        },
                    );
                    m
                },
            },
            cache_stats: ProxyCacheStats {
                total_injected: 2,
                ..Default::default()
            },
        };

        record_proxy_filter_stats(&mut session_stats, &proxy_stats);

        // Check tool_filter domain was recorded
        let domain_breakdown = session_stats.domain_breakdown();
        let tool_filter = domain_breakdown.get("tool_filter");
        assert!(tool_filter.is_some(), "Expected tool_filter domain");
        if let Some(tf) = tool_filter {
            assert_eq!(tf.total_raw_tokens, 500);
            assert_eq!(tf.total_compressed_tokens, 25);
        }

        // With no real cache_read_tokens, prompt_cache domain should NOT be
        // created — recording a hit with 0 tokens would inflate the hit count
        // while showing zero savings.
        let prompt_cache = domain_breakdown.get("prompt_cache");
        assert!(
            prompt_cache.is_none(),
            "prompt_cache should not exist without real cache_read_tokens"
        );
    }

    #[test]
    fn test_record_proxy_filter_stats_with_real_cache_tokens() {
        let mut session_stats = crate::mcp::session_stats::SessionStats::new();
        let proxy_stats = ProxyStatsResponse {
            filter_stats: ProxyFilterStats::default(),
            cache_stats: ProxyCacheStats {
                cache_read_tokens: 5000,
                cache_creation_tokens: 2000,
                ..Default::default()
            },
        };

        record_proxy_filter_stats(&mut session_stats, &proxy_stats);

        // Real cache-read tokens should be recorded as savings
        let prompt_cache = session_stats.domain_breakdown().get("prompt_cache");
        assert!(
            prompt_cache.is_some(),
            "prompt_cache should exist with real cache_read_tokens"
        );
        if let Some(pc) = prompt_cache {
            assert_eq!(
                pc.total_raw_tokens, 5000,
                "cache_read_tokens should be recorded as savings"
            );
            assert_eq!(pc.cache_hits, Some(1));
        }
    }
}
