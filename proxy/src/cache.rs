// proxy/src/cache.rs
//
// Prompt-cache breakpoint injection engine.
//
// Injects cache_control breakpoints into /v1/messages request bodies
// following Pino's proven strategy:
//
//   Slot 1: tools       — last entry in body.tools[]       (1h TTL)
//   Slot 2: system      — last block > 500 chars           (1h TTL)
//   Slot 3: messages[0] — last cacheable block             (1h TTL)
//   Slot 4: tail        — last text/tool_result/image      (configurable TTL)
//
// The extended-cache-ttl beta header makes `{type: "ephemeral"}`
// without explicit `ttl` default to 1h instead of 5m.
//
// IMPORTANT: If the client has already sent its own cache_control
// breakpoints, injection is SKIPPED entirely. Client-sent breakpoints
// are presumed intentional (e.g. fine-grained per-file baselines) and
// must not be clobbered by the proxy's coarse 4-slot scheme.

use serde_json::Value;
use tracing::debug;

const SMALL_BLOCK_THRESHOLD: usize = 500;

/// Statistics tracked across cache injections.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CacheStats {
    /// Number of requests where breakpoints were injected.
    pub total_injected: u64,

    /// Number of tools breakpoints placed.
    pub tools_slots: u64,

    /// Number of system breakpoints placed.
    pub system_slots: u64,

    /// Number of messages[0] breakpoints placed.
    pub messages_slots: u64,

    /// Number of tail breakpoints placed.
    pub tail_slots: u64,

    /// Number of small system blocks filtered out.
    pub small_blocks_filtered: u64,

    /// Number of client-sent breakpoints stripped.
    pub client_breakpoints_stripped: u64,

    /// Number of requests where client-sent breakpoints were preserved
    /// (injection skipped entirely).
    pub client_breakpoints_preserved: u64,

    /// Tokens read from Anthropic's prompt cache on this request
    /// (from `usage.cache_read_input_tokens` in the upstream response).
    pub cache_read_tokens: u64,

    /// Tokens written to Anthropic's prompt cache on this request
    /// (from `usage.cache_creation_input_tokens` in the upstream response).
    pub cache_creation_tokens: u64,
}

impl CacheStats {
    /// H-2 fix: Accumulate stats from a per-request local copy into the shared
    /// cumulative stats. Called under a write lock after inject_breakpoints has
    /// run without holding any lock.
    pub fn merge(&mut self, other: &CacheStats) {
        self.total_injected += other.total_injected;
        self.tools_slots += other.tools_slots;
        self.system_slots += other.system_slots;
        self.messages_slots += other.messages_slots;
        self.tail_slots += other.tail_slots;
        self.small_blocks_filtered += other.small_blocks_filtered;
        self.client_breakpoints_stripped += other.client_breakpoints_stripped;
        self.client_breakpoints_preserved += other.client_breakpoints_preserved;
        self.cache_read_tokens += other.cache_read_tokens;
        self.cache_creation_tokens += other.cache_creation_tokens;
    }
}

/// Inject cache_control breakpoints into a /v1/messages request body.
///
/// Modifies `body` in-place. Returns the number of breakpoints placed
/// and updates `stats`.
///
/// If the body already contains client-sent `cache_control` breakpoints,
/// injection is SKIPPED entirely — the client's breakpoints are preserved
/// and `stats.client_breakpoints_preserved` is incremented.
pub fn inject_breakpoints(body: &mut Value, tail_ttl: &str, stats: &mut CacheStats) -> usize {
    if !body.is_object() {
        return 0;
    }

    // Respect client-sent breakpoints — do NOT clobber them.
    // `has_any_breakpoints` detects cache_control on tools, system,
    // and message content blocks. If the client already placed
    // breakpoints, they are presumed intentional (e.g. fine-grained
    // per-file baselines) and must be preserved.
    if has_any_breakpoints(body) {
        debug!("[cache] Client-sent breakpoints detected, skipping injection");
        stats.client_breakpoints_preserved += 1;
        return 0;
    }

    let mut slots: usize = 0;
    let max_slots: usize = 4;

    // ---- Phase 1: Strip any existing client-sent breakpoints ----
    // (Unreachable when client breakpoints exist — the guard above
    //  returns early. Kept for safety in case of nested/edge cases.)
    let stripped = strip_existing_breakpoints(body);
    stats.client_breakpoints_stripped += stripped as u64;

    // ---- Slot 1: tools (last entry in body.tools[]) ----
    if let Some(tools) = body["tools"].as_array_mut() {
        if !tools.is_empty() {
            if let Some(last) = tools.last_mut() {
                last["cache_control"] = serde_json::json!({"type": "ephemeral"});
                slots += 1;
                stats.tools_slots += 1;
                debug!("[cache] Slot 1: tools breakpoint placed");
            }
        }
    }

    if slots >= max_slots {
        stats.total_injected += 1;
        return slots;
    }

    // ---- Slot 2: system (last block > 500 chars) ----
    if let Some(system) = body["system"].as_array_mut() {
        // Find the index of the last large block (> 500 chars)
        let last_large_idx = system.iter().rposition(|block| {
            block["text"]
                .as_str()
                .is_some_and(|t| t.len() > SMALL_BLOCK_THRESHOLD)
        });

        if let Some(target_idx) = last_large_idx {
            // Strip cache_control from small blocks (< 500 chars) — breakpoints
            // on small blocks waste slots. Do NOT remove the block itself.
            for block in system.iter_mut() {
                if block["text"]
                    .as_str()
                    .is_none_or(|t| t.len() < SMALL_BLOCK_THRESHOLD)
                    && block
                        .as_object_mut()
                        .and_then(|o| o.remove("cache_control"))
                        .is_some()
                {
                    stats.small_blocks_filtered += 1;
                }
            }

            // Place breakpoint on the actual last large block
            if let Some(block) = system.get_mut(target_idx) {
                block["cache_control"] = serde_json::json!({"type": "ephemeral"});
                slots += 1;
                stats.system_slots += 1;
                debug!("[cache] Slot 2: system breakpoint placed");
            }
        }
    }

    if slots >= max_slots {
        stats.total_injected += 1;
        return slots;
    }

    // ---- Slot 3: messages[0] (last cacheable block in first message) ----
    if let Some(messages) = body["messages"].as_array_mut() {
        if let Some(first_msg) = messages.first_mut() {
            if let Some(content) = first_msg["content"].as_array_mut() {
                // Find the last text/tool_result/image block (cacheable type)
                let cacheable_idx: Option<usize> = content
                    .iter()
                    .enumerate()
                    .filter(|(_, block)| {
                        block["type"]
                            .as_str()
                            .is_some_and(|t| matches!(t, "text" | "tool_result" | "image"))
                    })
                    .map(|(i, _)| i)
                    .next_back();

                if let Some(idx) = cacheable_idx {
                    content[idx]["cache_control"] = serde_json::json!({"type": "ephemeral"});
                    slots += 1;
                    stats.messages_slots += 1;
                    debug!("[cache] Slot 3: messages[0] breakpoint placed");
                }
            }
        }
    }

    if slots >= max_slots {
        stats.total_injected += 1;
        return slots;
    }

    // ---- Slot 4: tail (last text/tool_result/image across all messages) ----
    // The tail breakpoint uses the configured `tail_ttl` (default "5m") so
    // the rolling tail is invalidated quickly. The other slots use the
    // default 1h TTL from the extended-cache-ttl beta header.
    if let Some(messages) = body["messages"].as_array_mut() {
        let mut found_tail = false;
        for msg in messages.iter_mut().rev() {
            if let Some(content) = msg["content"].as_array_mut() {
                for block in content.iter_mut().rev() {
                    if block["type"]
                        .as_str()
                        .is_some_and(|t| matches!(t, "text" | "tool_result" | "image"))
                    {
                        block["cache_control"] = serde_json::json!({
                            "type": "ephemeral",
                            "ttl": tail_ttl
                        });
                        found_tail = true;
                        slots += 1;
                        stats.tail_slots += 1;
                        debug!("[cache] Slot 4: tail breakpoint placed (TTL: {tail_ttl})");
                        break;
                    }
                }
                if found_tail {
                    break;
                }
            }
        }
    }

    if slots > 0 {
        stats.total_injected += 1;
    }
    slots
}

/// Strip any existing cache_control breakpoints from the body.
/// Returns the number of breakpoints stripped.
fn strip_existing_breakpoints(body: &mut Value) -> usize {
    let mut count = 0;

    // Strip from tools[]
    if let Some(tools) = body["tools"].as_array_mut() {
        for tool in tools.iter_mut() {
            if tool
                .as_object_mut()
                .and_then(|o| o.remove("cache_control"))
                .is_some()
            {
                count += 1;
            }
        }
    }

    // Strip from system[]
    if let Some(system) = body["system"].as_array_mut() {
        for block in system.iter_mut() {
            if block
                .as_object_mut()
                .and_then(|o| o.remove("cache_control"))
                .is_some()
            {
                count += 1;
            }
        }
    }

    // Strip from messages[].content[].cache_control
    if let Some(messages) = body["messages"].as_array_mut() {
        for msg in messages.iter_mut() {
            if let Some(content) = msg["content"].as_array_mut() {
                for block in content.iter_mut() {
                    if block
                        .as_object_mut()
                        .and_then(|o| o.remove("cache_control"))
                        .is_some()
                    {
                        count += 1;
                    }
                }
            }
        }
    }

    count
}

/// Check if the body has any cache_control breakpoints already.
///
/// Used by `inject_breakpoints` to detect client-sent breakpoints and
/// skip injection entirely (preserving the client's cache strategy).
pub fn has_any_breakpoints(body: &Value) -> bool {
    // Check tools
    if let Some(tools) = body["tools"].as_array() {
        for tool in tools {
            if tool.get("cache_control").is_some() {
                return true;
            }
        }
    }

    // Check system
    if let Some(system) = body["system"].as_array() {
        for block in system {
            if block.get("cache_control").is_some() {
                return true;
            }
        }
    }

    // Check messages
    if let Some(messages) = body["messages"].as_array() {
        for msg in messages {
            if let Some(content) = msg["content"].as_array() {
                for block in content {
                    if block.get("cache_control").is_some() {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Get the `anthropic-beta` header value for extended cache TTL.
pub fn anthropic_beta_header() -> &'static str {
    "extended-cache-ttl-2025-04-11"
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Create a test request body with the given number of tools.
    /// Does NOT include any existing cache_control breakpoints.
    fn make_test_body(tools_count: usize) -> Value {
        let tools: Vec<Value> = (0..tools_count)
            .map(|i| {
                json!({
                    "name": format!("Tool{i}"),
                    "description": format!("Tool {i} description"),
                    "input_schema": {"type": "object", "properties": {}}
                })
            })
            .collect();

        json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 8192,
            "tools": tools,
            "system": [
                {"type": "text", "text": "You are Claude, a helpful AI assistant."},
                {"type": "text", "text": "A".repeat(600)}
            ],
            "messages": [
                {"role": "user", "content": [
                    {"type": "text", "text": "Hello"},
                    {"type": "tool_result", "content": "result1"},
                    {"type": "text", "text": "Static reminders block. ".to_owned() + &"x".repeat(5000)}
                ]}
            ]
        })
    }

    #[test]
    fn test_no_existing_breakpoints_in_body() {
        let body = make_test_body(1);
        assert!(
            !has_any_breakpoints(&body),
            "Test body should have no breakpoints initially"
        );
    }

    #[test]
    fn test_injects_tools_breakpoint() {
        let mut body = make_test_body(3);
        let mut stats = CacheStats::default();
        let slots = inject_breakpoints(&mut body, "5m", &mut stats);
        assert!(slots >= 1, "Expected at least 1 slot");
        assert_eq!(stats.tools_slots, 1);

        let tools = body["tools"].as_array().unwrap();
        assert!(tools.last().unwrap().get("cache_control").is_some());
    }

    #[test]
    fn test_system_breakpoint_on_large_block() {
        let mut body = make_test_body(1);
        let mut stats = CacheStats::default();
        inject_breakpoints(&mut body, "5m", &mut stats);
        // Should place a system breakpoint on the 600-char block
        assert_eq!(
            stats.system_slots, 1,
            "Should have placed system slot on large block"
        );
    }

    #[test]
    fn test_injects_messages_breakpoint() {
        let mut body = make_test_body(1);
        let mut stats = CacheStats::default();
        inject_breakpoints(&mut body, "5m", &mut stats);

        let messages = body["messages"].as_array().unwrap();
        let content = messages[0]["content"].as_array().unwrap();
        // The last text block (the long reminders one) should have cache_control
        let last_text = content.last().unwrap();
        assert!(last_text.get("cache_control").is_some());
    }

    #[test]
    fn test_strips_existing_breakpoints() {
        let mut body = make_test_body(2);
        // No breakpoints initially
        assert_eq!(strip_existing_breakpoints(&mut body), 0);

        // Manually add some breakpoints
        body["tools"][0]["cache_control"] = json!({"type": "ephemeral"});
        body["system"][0]["cache_control"] = json!({"type": "ephemeral"});

        let count = strip_existing_breakpoints(&mut body);
        assert_eq!(count, 2, "Should have stripped 2 existing breakpoints");

        // Verify they're gone
        assert!(body["tools"][0].get("cache_control").is_none());
        assert!(body["system"][0].get("cache_control").is_none());
    }

    #[test]
    fn test_no_tools_no_crash() {
        let mut body = json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 8192,
            "messages": [{"role": "user", "content": [{"type": "text", "text": "Hi"}]}]
        });
        let mut stats = CacheStats::default();
        let slots = inject_breakpoints(&mut body, "5m", &mut stats);
        // Without tools, should still place tail slot
        assert!(slots >= 1);
    }

    #[test]
    fn test_has_any_breakpoints() {
        let mut body = make_test_body(1);
        assert!(!has_any_breakpoints(&body));
        body["tools"][0]["cache_control"] = json!({"type": "ephemeral"});
        assert!(has_any_breakpoints(&body));
    }

    #[test]
    fn test_inject_skips_when_client_breakpoints_exist() {
        // Body with client-sent breakpoint on tools[0]
        let mut body = make_test_body(2);
        body["tools"][0]["cache_control"] = json!({"type": "ephemeral"});

        let mut stats = CacheStats::default();
        let slots = inject_breakpoints(&mut body, "5m", &mut stats);

        // Injection must be skipped entirely
        assert_eq!(slots, 0, "Should not inject when client breakpoints exist");
        assert_eq!(stats.client_breakpoints_preserved, 1);
        assert_eq!(stats.total_injected, 0);
        assert_eq!(stats.tools_slots, 0);
        assert_eq!(stats.system_slots, 0);
        assert_eq!(stats.messages_slots, 0);
        assert_eq!(stats.tail_slots, 0);

        // Client breakpoint must be preserved
        assert!(body["tools"][0].get("cache_control").is_some());
    }

    #[test]
    fn test_inject_skips_when_client_breakpoints_on_system() {
        let mut body = make_test_body(1);
        body["system"][0]["cache_control"] = json!({"type": "ephemeral"});

        let mut stats = CacheStats::default();
        let slots = inject_breakpoints(&mut body, "5m", &mut stats);

        assert_eq!(
            slots, 0,
            "Should not inject when client breakpoints exist on system"
        );
        assert_eq!(stats.client_breakpoints_preserved, 1);
        assert!(body["system"][0].get("cache_control").is_some());
    }

    #[test]
    fn test_inject_skips_when_client_breakpoints_on_messages() {
        let mut body = make_test_body(1);
        body["messages"][0]["content"][0]["cache_control"] = json!({"type": "ephemeral"});

        let mut stats = CacheStats::default();
        let slots = inject_breakpoints(&mut body, "5m", &mut stats);

        assert_eq!(
            slots, 0,
            "Should not inject when client breakpoints exist on messages"
        );
        assert_eq!(stats.client_breakpoints_preserved, 1);
        assert!(body["messages"][0]["content"][0]
            .get("cache_control")
            .is_some());
    }

    #[test]
    fn test_tail_breakpoint_uses_configured_ttl() {
        let mut body = make_test_body(1);
        let mut stats = CacheStats::default();
        inject_breakpoints(&mut body, "5m", &mut stats);

        // The tail breakpoint must include the configured TTL
        let messages = body["messages"].as_array().unwrap();
        let content = messages[0]["content"].as_array().unwrap();
        let last_block = content.last().unwrap();
        assert_eq!(
            last_block["cache_control"]["ttl"], "5m",
            "Tail breakpoint must include the configured TTL"
        );
        assert_eq!(last_block["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_tail_breakpoint_custom_ttl() {
        let mut body = make_test_body(1);
        let mut stats = CacheStats::default();
        inject_breakpoints(&mut body, "1h", &mut stats);

        let messages = body["messages"].as_array().unwrap();
        let content = messages[0]["content"].as_array().unwrap();
        let last_block = content.last().unwrap();
        assert_eq!(
            last_block["cache_control"]["ttl"], "1h",
            "Tail breakpoint must use the custom TTL"
        );
    }

    #[test]
    fn test_cache_stats_merge_accumulates_new_fields() {
        let mut a = CacheStats::default();
        let mut b = CacheStats::default();
        a.client_breakpoints_preserved = 3;
        a.cache_read_tokens = 1000;
        a.cache_creation_tokens = 500;
        b.client_breakpoints_preserved = 2;
        b.cache_read_tokens = 2000;
        b.cache_creation_tokens = 300;
        a.merge(&b);
        assert_eq!(a.client_breakpoints_preserved, 5);
        assert_eq!(a.cache_read_tokens, 3000);
        assert_eq!(a.cache_creation_tokens, 800);
    }
}
