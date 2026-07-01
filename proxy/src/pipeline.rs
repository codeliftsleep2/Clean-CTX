// proxy/src/pipeline.rs
//
// Pipeline abstraction: makes the request transform chain pluggable
// and testable in isolation (OCP compliance).
//
// Adding a new transform:
//   1. Create a struct implementing `TransformFn`
//   2. Add it to the `build_pipeline` function
//   3. Done — existing transforms are unchanged

use std::collections::HashSet;
use std::sync::Arc;
use serde_json::Value;

use crate::filter_registry::FilterRegistry;
use crate::platform::PlatformAdapter;
use crate::transform::{TransformStats, self as transform};

/// Pipeline configuration extracted from ProxyConfig at request time.
#[derive(Clone, Default)]
pub struct PipelineConfig {
    pub drop_tools_set: HashSet<String>,
    pub strip_ansi: bool,
    pub trim_bash_git: bool,
    pub model_override: Option<String>,
    pub scrub_secrets: bool,
    pub tool_filters: bool,
    pub filter_registry: Option<Arc<FilterRegistry>>,
    /// Enable sliding context window (age-based tool-result truncation).
    pub sliding_window_enabled: bool,
    /// Maximum age in turns before a tool result is aged (stubbed).
    pub sliding_window_max_age_turns: usize,
    /// Number of most recent turns to always preserve (force-preserve floor).
    pub sliding_window_force_preserve_floor: usize,
}

/// A callable transform step. Each step receives the body, stats, config,
/// and a platform adapter for format-aware operations.
pub type TransformFn = Box<dyn Fn(&mut Value, &mut TransformStats, &PipelineConfig, &dyn PlatformAdapter) + Send + Sync>;

/// A composed pipeline of transform functions.
pub struct Pipeline {
    transforms: Vec<TransformFn>,
}

impl Pipeline {
    /// Build the default pipeline from the given config.
    pub fn build(config: &PipelineConfig) -> Self {
        let mut transforms: Vec<TransformFn> = Vec::new();

        if !config.drop_tools_set.is_empty() {
            transforms.push(Box::new(|body, stats, cfg, _| {
                transform::drop_tools(body, &cfg.drop_tools_set, stats);
            }));
        }

        if config.strip_ansi {
            transforms.push(Box::new(|body, stats, _, adapter| {
                transform::strip_ansi(body, stats, adapter);
            }));
        }

        if config.trim_bash_git {
            transforms.push(Box::new(|body, stats, _, _| {
                transform::trim_bash_git(body, stats);
            }));
        }

        if let Some(ref model) = config.model_override {
            let model = model.clone();
            transforms.push(Box::new(move |body, stats, _, _| {
                transform::override_model(body, &model, stats);
            }));
        }

        if config.scrub_secrets {
            transforms.push(Box::new(|body, stats, _, adapter| {
                transform::scrub_secrets(body, stats, adapter);
            }));
        }

        if config.tool_filters {
            if let Some(ref registry) = config.filter_registry {
                let registry = registry.clone();
                transforms.push(Box::new(move |body, stats, _, adapter| {
                    transform::apply_tool_filters(body, &registry, stats, adapter);
                }));
            }
        }

        // Sliding window transform (added after tool filtering, before cache)
        if config.sliding_window_enabled {
            let max_age = config.sliding_window_max_age_turns;
            let floor = config.sliding_window_force_preserve_floor;
            transforms.push(Box::new(move |body, stats, _, adapter| {
                transform::age_tool_results(body, stats, max_age, floor, adapter);
            }));
        }

        Self { transforms }
    }

    /// Run all transforms in order.
    pub fn run(&self, body: &mut Value, stats: &mut TransformStats, config: &PipelineConfig, adapter: &dyn PlatformAdapter) {
        for transform in &self.transforms {
            transform(body, stats, config, adapter);
        }
    }

    /// Number of transforms in this pipeline.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Returns true if this pipeline has no transforms.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashSet;

    #[test]
    fn test_pipeline_empty_config() {
        let config = PipelineConfig::default();
        let pipeline = Pipeline::build(&config);
        assert_eq!(pipeline.len(), 0);
    }

    #[test]
    fn test_pipeline_with_tools_drop() {
        let mut drop_set = HashSet::new();
        drop_set.insert("Read".to_string());
        let config = PipelineConfig {
            drop_tools_set: drop_set.into_iter().collect(),
            sliding_window_enabled: false,
            sliding_window_max_age_turns: 20,
            sliding_window_force_preserve_floor: 15,
            ..PipelineConfig::default()
        };
        let pipeline = Pipeline::build(&config);
        assert_eq!(pipeline.len(), 1);
    }

    #[test]
    fn test_pipeline_full_config() {
        let config = PipelineConfig {
            drop_tools_set: ["Read".into()].into_iter().collect(),
            strip_ansi: true,
            trim_bash_git: false,
            model_override: Some("claude-opus-4-6".into()),
            scrub_secrets: true,
            tool_filters: false,
            filter_registry: None,
            sliding_window_enabled: false,
            sliding_window_max_age_turns: 20,
            sliding_window_force_preserve_floor: 15,
        };
        let pipeline = Pipeline::build(&config);
        assert_eq!(pipeline.len(), 4);
    }

    #[test]
    fn test_pipeline_drops_tools() {
        let mut drop_set = HashSet::new();
        drop_set.insert("Read".to_string());
        let config = PipelineConfig {
            drop_tools_set: drop_set.into_iter().collect(),
            sliding_window_enabled: false,
            sliding_window_max_age_turns: 20,
            sliding_window_force_preserve_floor: 15,
            ..PipelineConfig::default()
        };
        let pipeline = Pipeline::build(&config);

        let mut body = json!({
            "tools": [
                {"name": "Bash", "description": "Run"},
                {"name": "Read", "description": "Read"}
            ]
        });

        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;
        pipeline.run(&mut body, &mut stats, &config, &adapter);

        assert_eq!(stats.tools_dropped, 1);
        assert_eq!(body["tools"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_pipeline_strips_ansi() {
        let config = PipelineConfig {
            strip_ansi: true,
            ..PipelineConfig::default()
        };
        let pipeline = Pipeline::build(&config);

        let mut body = json!({
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": "\x1B[32mHello\x1B[0m world"}
                ]
            }]
        });

        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;
        pipeline.run(&mut body, &mut stats, &config, &adapter);

        let text = body["messages"][0]["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "Hello world");
    }
}