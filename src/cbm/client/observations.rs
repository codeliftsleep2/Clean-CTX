//! Private provider observations. Job, serving, query and coverage truth are
//! separate snapshots; none certifies native source completeness.
use super::{CbmClient, CbmError};
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub(crate) struct ProviderObservation {
    pub(crate) metadata: Map<String, Value>,
    pub(crate) failure: Option<String>,
}
pub(super) type Observations = HashMap<(String, String), ProviderObservation>;

pub(crate) enum ProviderIndexReadiness {
    Ready,
    InProgress,
    Failed(String),
}

impl ProviderObservation {
    pub(super) fn page(body: &Value) -> Self {
        let mut metadata = body.as_object().cloned().unwrap_or_default();
        for payload in ["columns", "cols", "rows", "groups"] {
            metadata.remove(payload);
        }
        for section in ["packages", "boundaries", "callers", "callees"] {
            metadata.remove(section);
            if let Some(object) = body[section].as_object() {
                let mut limits = object.clone();
                for payload in ["cols", "rows", "groups"] {
                    limits.remove(payload);
                }
                if !limits.is_empty() {
                    metadata.insert(section.into(), Value::Object(limits));
                }
            }
        }
        metadata.entry("pages_consumed").or_insert(json!(1));
        Self {
            metadata,
            failure: None,
        }
    }
    /// Vec-only consumers cannot deliver continuation honestly. Reject partial
    /// evidence using the existing tool-error/native-fallback representation.
    /// This is not a collection policy: no next-page request is made.
    pub(super) fn require_page(&self, tool: &str, count: usize) -> Result<(), CbmError> {
        if self.failure.is_some() {
            return Err(self.partial(tool));
        }
        let m = &self.metadata;
        for key in [
            "has_more",
            "truncated",
            "path_has_more",
            "path_truncated",
            "scope_truncated",
        ] {
            if let Some(v) = m.get(key) {
                if !v.is_boolean() {
                    return Err(CbmError::ParseError(format!("invalid {key}")));
                }
                if v == &json!(true) {
                    return Err(self.partial(tool));
                }
            }
        }
        for key in ["next", "next_cursor", "next_offset", "path_next_offset"] {
            if m.get(key).is_some_and(|v| !v.is_null()) {
                return Err(self.partial(tool));
            }
        }
        for key in [
            "total_relation",
            "callers_total_relation",
            "callees_total_relation",
        ] {
            if let Some(value) = m.get(key) {
                if value != "eq" {
                    return Err(self.partial(tool));
                }
            }
        }
        for key in ["returned", "total"] {
            if let Some(value) = m.get(key) {
                if value.as_u64() != Some(count as u64) {
                    return Err(self.partial(tool));
                }
            }
        }
        // Search's pinned schema has exact total without total_relation. Query
        // requires eq. Trace has directional eq totals instead of one total.
        let established = match tool {
            "search_graph" => m.get("total").is_some(),
            "query_graph" => {
                m.get("total").is_some() && m.get("total_relation") == Some(&json!("eq"))
            }
            "trace_path" => ["callers_total_relation", "callees_total_relation"]
                .iter()
                .any(|k| m.contains_key(*k)),
            _ => true,
        };
        if !established {
            return Err(self.partial(tool));
        }
        Ok(())
    }
    fn partial(&self, tool: &str) -> CbmError {
        CbmError::ToolError {
            tool: tool.into(),
            message: format!(
                "provider result is partial or completeness is unknown; no continuation collected; limits: {}",
                Value::Object(self.metadata.clone())
            ),
        }
    }
}
impl CbmClient {
    pub(crate) fn observe_initial_index_readiness(
        &mut self,
        project: &str,
        root: &Path,
    ) -> Result<ProviderIndexReadiness, CbmError> {
        let response =
            self.call_tool("index_status", json!({"project":project,"format":"json"}))?;
        let status = self.parse_cbm_response(&response)?;
        if status["project"] != project {
            return Err(CbmError::ParseError("serving project mismatch".into()));
        }
        let supplied = status["root_path"]
            .as_str()
            .ok_or_else(|| CbmError::ParseError("unknown serving root".into()))?;
        let actual = Path::new(supplied)
            .canonicalize()
            .map_err(|error| CbmError::ParseError(format!("serving root: {error}")))?;
        let expected = root
            .canonicalize()
            .map_err(|error| CbmError::ParseError(format!("admitted root: {error}")))?;
        if actual != expected {
            return Err(CbmError::ParseError(
                "serving root differs from admitted root".into(),
            ));
        }
        let state = status["status"]
            .as_str()
            .or_else(|| status["state"].as_str());
        match state {
            Some("ready" | "indexed" | "complete" | "completed" | "succeeded") => {
                Ok(ProviderIndexReadiness::Ready)
            }
            Some("queued" | "running" | "indexing" | "initializing") => {
                Ok(ProviderIndexReadiness::InProgress)
            }
            Some("failed" | "cancelled" | "canceled") => Ok(ProviderIndexReadiness::Failed(
                status["error"]
                    .as_str()
                    .unwrap_or("provider-owned initial indexing failed")
                    .to_string(),
            )),
            Some(other) => Err(CbmError::ParseError(format!(
                "unknown provider index status: {other}"
            ))),
            None => Err(CbmError::ParseError("missing provider index status".into())),
        }
    }

    pub(crate) fn provider_observation(
        &self,
        project: &str,
        tool: &str,
    ) -> Option<&ProviderObservation> {
        self.provider_observations
            .get(&(project.into(), tool.into()))
    }
    pub(super) fn retain_observation(&mut self, tool: &str, args: &Value, response: &Value) {
        let Some(text) = response["content"]
            .as_array()
            .and_then(|c| c.first())
            .and_then(|c| c["text"].as_str())
        else {
            return;
        };
        let body = serde_json::from_str::<Value>(text).ok();
        let project = args["project"]
            .as_str()
            .or_else(|| args["name"].as_str())
            .or_else(|| body.as_ref().and_then(|b| b["project"].as_str()));
        let Some(project) = project else {
            return;
        };
        let mut observation = body
            .as_ref()
            .map(ProviderObservation::page)
            .unwrap_or_default();
        if let Some(root) = args.get("repo_path") {
            observation
                .metadata
                .insert("requested_root".into(), root.clone());
        }
        if response["isError"] == true {
            observation.failure = Some(text.to_owned());
        }
        self.provider_observations
            .insert((project.into(), tool.into()), observation);
    }
    /// Exact known paths only: no directory discovery or implicit polling.
    /// Missing optional diagnostics is an error, allowing native acquisition.
    pub(crate) fn observe_candidate_paths(
        &mut self,
        project: &str,
        root: &Path,
        paths: &[String],
    ) -> Result<(), CbmError> {
        let response =
            self.call_tool("index_status", json!({"project":project,"format":"json"}))?;
        let status = self.parse_cbm_response(&response)?;
        if status["project"] != project {
            return Err(CbmError::ParseError("serving project mismatch".into()));
        }
        let supplied = status["root_path"]
            .as_str()
            .ok_or_else(|| CbmError::ParseError("unknown serving root".into()))?;
        let actual = Path::new(supplied)
            .canonicalize()
            .map_err(|e| CbmError::ParseError(format!("serving root: {e}")))?;
        if actual
            != root
                .canonicalize()
                .map_err(|e| CbmError::ParseError(format!("admitted root: {e}")))?
        {
            return Err(CbmError::ParseError(
                "serving root differs from admitted root".into(),
            ));
        }
        if paths.is_empty() {
            return Ok(());
        }
        // This is the pinned tool's input contract, not a new collection cap.
        if paths.len() > 128 {
            return Err(CbmError::CapabilityError(
                "coverage accepts at most 128 exact paths per call; continuation not collected"
                    .into(),
            ));
        }
        let coverage = self.collect(
            "check_index_coverage",
            json!({"project":project,"paths":paths,"format":"json"}),
        )?;
        if coverage["project"] != project {
            return Err(CbmError::ParseError("coverage project mismatch".into()));
        }
        let rows = coverage["paths"]
            .as_array()
            .ok_or_else(|| CbmError::ParseError("missing coverage paths".into()))?;
        if coverage["signal"] != "best_effort"
            || !coverage["metadata"].is_object()
            || coverage["path_returned"].as_u64() != Some(paths.len() as u64)
            || coverage["path_total"].as_u64() != Some(paths.len() as u64)
            || rows.len() != paths.len()
            || rows
                .iter()
                .zip(paths)
                .any(|(row, path)| row["requested_path"] != path.as_str())
        {
            return Err(CbmError::ParseError(
                "coverage identity/table is incomplete or incompatible".into(),
            ));
        }
        // Preserve metadata match/changed/missing/unknown as independent
        // advisory evidence. A clean signal never proves semantic completeness.
        let observation = self
            .provider_observation(project, "check_index_coverage")
            .ok_or_else(|| CbmError::ParseError("missing coverage observation".into()))?;
        observation.require_page("check_index_coverage", 0)
    }
    pub(crate) fn index_project(
        &mut self,
        repo_path: &str,
        mode: &str,
        project: &str,
    ) -> Result<(), CbmError> {
        // Explicit name binds indexing to the admitted root map; no basename
        // inference and no asynchronous conversion.
        let response = self.call_tool(
            "index_repository",
            json!({"repo_path":repo_path,"mode":mode,"name":project}),
        )?;
        let body = self.parse_cbm_response(&response)?;
        if body["project"] != project {
            return Err(CbmError::ParseError("index project mismatch".into()));
        }
        if body["status"] != "indexed" {
            return Err(CbmError::ToolError {
                tool: "index_repository".into(),
                message: format!("index not successfully terminal: {body}"),
            });
        }
        Ok(())
    }
}
