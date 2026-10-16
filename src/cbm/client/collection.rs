//! Approved B3 logical-query deadline and retained encoded-payload budget.
//! Only consumed typed operations collect. Raw proxy remains one response.
use super::{CbmClient, CbmError, MAX_RESPONSE_BYTES, decoder, observations::ProviderObservation};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::time::Instant;

#[derive(Clone)]
pub(super) struct CollectionState {
    pub(super) deadline: Instant,
    pub(super) retry_spent: bool,
    pages: usize,
    bytes: usize,
    evidence: Vec<Value>,
}
fn failure(tool: &str, message: &str) -> CbmError {
    CbmError::ToolError {
        tool: tool.into(),
        message: format!("provider collection incomplete: {message}"),
    }
}
impl CbmClient {
    pub(super) fn check_collection_deadline(&self) -> Result<(), CbmError> {
        if self
            .collection
            .as_ref()
            .is_some_and(|s| Instant::now() >= s.deadline)
        {
            return Err(CbmError::Timeout(self.timeout));
        }
        Ok(())
    }
    pub(super) fn retry_pause(&mut self) -> Result<(), CbmError> {
        if let Some(state) = self.collection.as_mut() {
            state.retry_spent = true;
            if state.deadline.saturating_duration_since(Instant::now())
                <= std::time::Duration::from_millis(100)
            {
                return Err(CbmError::Timeout(self.timeout));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.check_collection_deadline()
    }
    pub(super) fn collect(&mut self, tool: &str, args: Value) -> Result<Value, CbmError> {
        self.collection = Some(CollectionState {
            deadline: Instant::now() + self.timeout,
            retry_spent: false,
            pages: 0,
            bytes: 0,
            evidence: Vec::new(),
        });
        let result = self.collect_inner(tool, args.clone());
        let state = self.collection.take().unwrap();
        if let Some(project) = args["project"].as_str() {
            let observation = self
                .provider_observations
                .entry((project.into(), tool.into()))
                .or_default();
            match &result {
                Ok(body) => *observation = ProviderObservation::page(body),
                Err(error) => {
                    observation.failure = Some(error.to_string());
                    observation
                        .metadata
                        .insert("pages_consumed".into(), json!(state.pages));
                    observation
                        .metadata
                        .insert("retained_encoded_bytes".into(), json!(state.bytes));
                    observation
                        .metadata
                        .insert("page_observations".into(), json!(state.evidence));
                }
            }
        }
        result
    }
    fn collect_inner(&mut self, tool: &str, base: Value) -> Result<Value, CbmError> {
        let mut args = base.clone();
        let mut merged = Value::Null;
        let mut bytes = 0usize;
        let mut pages = 0usize;
        let mut seen = HashSet::new();
        let mut count = 0usize;
        loop {
            self.check_collection_deadline()?;
            let response = self.call_tool(tool, args.clone())?;
            let encoded = response
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .ok_or_else(|| CbmError::ParseError("missing encoded page".into()))?;
            bytes = bytes
                .checked_add(encoded.len())
                .ok_or_else(|| failure(tool, "aggregate payload overflow"))?;
            if bytes > MAX_RESPONSE_BYTES {
                return Err(failure(
                    tool,
                    "aggregate retained encoded payload exceeds 4 MiB",
                ));
            }
            let mut page = self.parse_cbm_response(&response)?;
            pages += 1;
            if page.get("project").is_some_and(|p| p != &base["project"]) {
                return Err(failure(tool, "project changed between pages"));
            }
            let page_count = validate(tool, &page, &base)?;
            count += page_count;
            let observed = ProviderObservation::page(&page);
            let state = self.collection.as_mut().unwrap();
            state.pages = pages;
            state.bytes = bytes;
            state.evidence.push(Value::Object(observed.metadata));
            let more = page["has_more"] == true || page["path_has_more"] == true;
            let total_key = if tool == "check_index_coverage" {
                "path_total"
            } else {
                "total"
            };
            if more
                && page[total_key]
                    .as_u64()
                    .is_some_and(|total| total <= count as u64)
            {
                return Err(failure(
                    tool,
                    "continuation contradicts exact total/no progress",
                ));
            }
            let next = continuation(tool, &page, &args, page_count)?;
            if !merged.is_null() {
                for key in [
                    "total",
                    "total_relation",
                    "callers_total",
                    "callees_total",
                    "callers_total_relation",
                    "callees_total_relation",
                    "indexed_at",
                    "metadata",
                    "path_total",
                    "scope_total",
                ] {
                    if merged.get(key) != page.get(key) {
                        return Err(failure(tool, "snapshot/totals changed between pages"));
                    }
                }
            }
            merge(tool, &mut merged, &mut page)?;
            self.check_collection_deadline()?;
            if let Some((key, value)) = next {
                if page_count == 0 || !seen.insert((key.to_owned(), value.to_string())) {
                    return Err(failure(tool, "continuation made no progress or repeated"));
                }
                args = base.clone();
                args[key] = value;
                continue;
            }
            if matches!(tool, "query_graph" | "search_graph") {
                merged["returned"] = json!(count);
            }
            if tool == "check_index_coverage" {
                merged["path_returned"] = json!(count);
            }
            merged["pages_consumed"] = json!(pages);
            merged["retained_encoded_bytes"] = json!(bytes);
            merged["page_observations"] = json!(self.collection.as_ref().unwrap().evidence);
            if matches!(tool, "query_graph" | "search_graph") {
                merged.as_object_mut().unwrap().remove("offset");
            }
            self.check_collection_deadline()?;
            return Ok(merged);
        }
    }
}
fn validate(tool: &str, page: &Value, base: &Value) -> Result<usize, CbmError> {
    let n = match tool {
        "query_graph" => decoder::table(page, "columns")?.rows.len(),
        "search_graph" => decoder::search(page)?.len(),
        "trace_path" => {
            decoder::trace(
                page,
                base["function_name"].as_str().unwrap(),
                base["direction"].as_str().unwrap(),
            )?;
            ["callers", "callees"]
                .iter()
                .filter_map(|l| page[*l]["groups"].as_array())
                .flat_map(|g| g.iter())
                .map(|g| g["rows"].as_array().map_or(0, Vec::len))
                .sum()
        }
        "check_index_coverage" => page["paths"]
            .as_array()
            .ok_or_else(|| failure(tool, "missing coverage paths"))?
            .len(),
        _ => return Err(failure(tool, "unsupported typed continuation")),
    };
    let returned = if tool == "check_index_coverage" {
        "path_returned"
    } else {
        "returned"
    };
    if page
        .get(returned)
        .is_some_and(|v| v.as_u64() != Some(n as u64))
    {
        return Err(failure(tool, "page returned count mismatch"));
    }
    for key in [
        "total_relation",
        "callers_total_relation",
        "callees_total_relation",
    ] {
        if page.get(key).is_some_and(|v| v != "eq") {
            return Err(failure(tool, "total is lower-bound or unknown"));
        }
    }
    Ok(n)
}
fn continuation(
    tool: &str,
    page: &Value,
    args: &Value,
    n: usize,
) -> Result<Option<(&'static str, Value)>, CbmError> {
    let flag = if tool == "check_index_coverage" {
        "path_has_more"
    } else {
        "has_more"
    };
    for key in [flag, "truncated"] {
        if page.get(key).is_some_and(|v| !v.is_boolean()) {
            return Err(failure(tool, "malformed continuation flag"));
        }
    }
    let more = page[flag] == true;
    let reason = page["truncation_reason"]
        .as_str()
        .or_else(|| page["reason"].as_str());
    if page["truncated"] == true && (!more || reason.is_none_or(|s| s != "page_limit")) {
        return Err(failure(tool, "unresolved provider truncation/budget"));
    }
    if !more {
        let keys: &[&str] = match tool {
            "check_index_coverage" => &["path_next_offset"],
            "search_graph" => &["next_offset"],
            _ => &["next_cursor", "next", "next_offset"],
        };
        if keys
            .iter()
            .any(|k| page.get(*k).is_some_and(|v| !v.is_null()))
        {
            return Err(failure(tool, "terminal page still carries continuation"));
        }
        return Ok(None);
    }
    if matches!(tool, "query_graph" | "trace_path") {
        let token = page
            .get("next_cursor")
            .or_else(|| page.get("next"))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| failure(tool, "missing opaque cursor; offset fallback forbidden"))?;
        if page.get("next").is_some()
            && page.get("next_cursor").is_some()
            && page["next"] != page["next_cursor"]
        {
            return Err(failure(tool, "conflicting cursors"));
        }
        return Ok(Some(("cursor", json!(token))));
    }
    let (response, request) = if tool == "search_graph" {
        ("next_offset", "offset")
    } else {
        ("path_next_offset", "path_offset")
    };
    let offset = args[request].as_u64().unwrap_or(0);
    let next = page[response]
        .as_u64()
        .ok_or_else(|| failure(tool, "missing continuation offset"))?;
    if next
        != offset
            .checked_add(n as u64)
            .ok_or_else(|| failure(tool, "offset overflow"))?
        || next <= offset
    {
        return Err(failure(tool, "offset made no progress or skipped rows"));
    }
    Ok(Some((request, json!(next))))
}
fn append(target: &mut Value, page: &mut Value, key: &str) -> Result<(), CbmError> {
    let Some(rows) = page[key].as_array_mut() else {
        return Err(CbmError::ParseError("missing page payload".into()));
    };
    target[key]
        .as_array_mut()
        .ok_or_else(|| CbmError::ParseError("missing accumulated payload".into()))?
        .append(rows);
    Ok(())
}
fn merge(tool: &str, target: &mut Value, page: &mut Value) -> Result<(), CbmError> {
    if target.is_null() {
        *target = page.take();
        return Ok(());
    }
    let mut payload = serde_json::Map::new();
    match tool {
        "query_graph" | "search_graph" => {
            let (header, rows) = if tool == "query_graph" {
                ("columns", "rows")
            } else {
                ("cols", "groups")
            };
            if target[header] != page[header]
                || (tool == "search_graph" && target["qn_rule"] != page["qn_rule"])
            {
                return Err(failure(tool, "columns/identity rule changed"));
            }
            append(target, page, rows)?;
            payload.insert(rows.into(), target[rows].take());
        }
        "trace_path" => {
            for leg in ["callers", "callees"] {
                if !target[leg].is_null() {
                    if target[leg]["cols"] != page[leg]["cols"]
                        || target[leg]["qn_rule"] != page[leg]["qn_rule"]
                    {
                        return Err(failure(tool, "trace columns/identity changed"));
                    }
                    append(&mut target[leg], &mut page[leg], "groups")?;
                    payload.insert(leg.into(), target[leg].take());
                }
            }
        }
        "check_index_coverage" => {
            append(target, page, "paths")?;
            payload.insert("paths".into(), target["paths"].take());
        }
        _ => return Err(failure(tool, "unsupported payload merge")),
    }
    *target = page.take();
    for (key, value) in payload {
        target[key] = value;
    }
    Ok(())
}
