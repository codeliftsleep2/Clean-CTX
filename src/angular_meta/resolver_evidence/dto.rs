//! Owned qualified wire value. This type has no canonical publication conversion.
use super::{Failure, Query};
use crate::ir::source_observations::{
    Span,
    request_budget::{Cost, Limits, Request, Stop},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub struct OwnedDto {
    value: Value,
}
impl OwnedDto {
    pub fn value(&self) -> &Value {
        &self.value
    }
    pub(crate) fn checked(value: Value) -> Result<Self, Failure> {
        validate(&value)?;
        Ok(Self { value })
    }
}
pub(super) fn source(path: &Path, hash: &str) -> Value {
    json!({"path":path,"sha256":hash})
}
pub(super) fn source_id(path: &Path, hash: &str) -> String {
    format!("source:{}@{hash}", path.display())
}
pub(super) fn span(s: Span) -> Value {
    json!({"start":s.start,"end":s.end})
}
pub(super) fn issue(code: &str, stage: &str, affected: &str, detail: impl ToString) -> Value {
    json!({"code":code,"stage":stage,"affected":[affected],"detail":detail.to_string()})
}
pub(super) fn stop_code(stop: Stop) -> &'static str {
    match stop {
        Stop::Cancelled => "cancelled",
        Stop::Deadline => "deadline",
        Stop::ActiveTasks => "active_expensive_tasks",
        Stop::SourceBytes => "source_bytes",
        Stop::SupportBytes => "support_bytes",
        Stop::RetainedBytes => "retained_bytes",
        Stop::Units => "traversal_units",
        Stop::Edges => "traversal_edges",
        Stop::ValidationBytes => "validation_bytes",
        Stop::ValidationChecks => "validation_checks",
        Stop::ResponseBytes => "response_bytes",
    }
}
fn cost(cost: Cost) -> Value {
    json!({"source_bytes":cost.source_bytes,"support_bytes":cost.support_bytes,"retained_bytes":cost.retained_bytes,"traversal_units":cost.units,"traversal_edges":cost.edges,"validation_bytes":cost.validation_bytes,"validation_checks":cost.validation_checks,"response_bytes":cost.response_bytes})
}
pub(super) fn limits(limits: Limits) -> Value {
    let mut result = cost(limits.total);
    result["elapsed_ms"] = json!(limits.deadline.as_millis());
    result["active_expensive_tasks"] = json!(limits.active_tasks);
    result["finalization"] = json!({"elapsed_ms":limits.finalization_time.as_millis(),"validation_bytes":limits.finalization.validation_bytes,"validation_checks":limits.finalization.validation_checks,"response_bytes":limits.finalization.response_bytes});
    result
}
pub(super) fn execution(request: &Request, stopped: Option<Stop>) -> Value {
    let snapshot = request.snapshot();
    let mut used = cost(snapshot.used);
    used["elapsed_ms"] = json!(snapshot.elapsed.as_millis());
    used["active_expensive_tasks"] = json!(snapshot.active_tasks);
    let mut protected = request.limits().finalization;
    protected.retained_bytes = protected
        .retained_bytes
        .saturating_sub(snapshot.finalization_used.retained_bytes);
    protected.validation_bytes = protected
        .validation_bytes
        .saturating_sub(snapshot.finalization_used.validation_bytes);
    protected.validation_checks = protected
        .validation_checks
        .saturating_sub(snapshot.finalization_used.validation_checks);
    protected.response_bytes = protected
        .response_bytes
        .saturating_sub(snapshot.finalization_used.response_bytes)
        .max(snapshot.pending_response_bytes);
    let mut reserved = cost(protected);
    reserved["active_expensive_tasks"] = json!(0);
    let stop = stopped.or(snapshot.last_stop);
    json!({"state":match stop {Some(Stop::Cancelled)=>"cancelled",Some(_)=>"exhausted",None=>"completed"},"stops":stop.map(|s|vec![issue(stop_code(s),"request","request",format!("{s:?}"))]).unwrap_or_default(),"effective_limits":limits(request.limits()),"requested_limits":limits(request.limits()),"accounting_scope":"request","used":used,"reserved":reserved,"peak_retained_bytes":snapshot.retained_peak,"released_retained_bytes":snapshot.retained_released,"inventory_work":request.work_counts(),"completed_after_stop":if snapshot.deadline_overshoot.is_zero(){vec![]}else{vec![json!({"stage":"non_interruptible_quantum","elapsed_ms":snapshot.deadline_overshoot.as_millis(),"charged_bytes":snapshot.used.source_bytes+snapshot.used.support_bytes})]},"enforcement":"cooperative","unmeasured_overhead":["pre-ledger ingress and transport","allocator and parser overhead beyond conservative reservations","non-interruptible metadata/parser quanta"]})
}
pub(super) fn predicate(query: &Query, sources: Value, kind: &str) -> Value {
    json!({"kind":kind,"sources":sources,"direction":query.direction,"selector":query.selector,"selector_basis":query.selector_basis,"rule_version":1})
}
pub(super) fn base(
    request: &Request,
    query: &Query,
    roots: &[PathBuf],
    stopped: Option<Stop>,
) -> Value {
    let mut coverage = json!({});
    for (name, kind) in [
        ("inventory", "inventory"),
        ("supported_written_source", "supported_written_mapping"),
        ("angular", "angular"),
        ("workspace", "workspace"),
    ] {
        coverage[name] = json!({"state":"open","predicate":predicate(query,json!([]),kind),"evidence":[],"omissions":[]});
    }
    json!({"schema":"clean-ctx/resolver-evidence-answer","contract_version":2,"rule_version":1,"query":{"direction":query.direction,"selector":query.selector,"selector_basis":query.selector_basis,"mode":"interactive","canonical_policy":"none","workspace_root":roots[0],"additional_roots":roots[1..],"within_path":query.within_path,"source_files":query.source_files.clone().unwrap_or_default()},"execution":execution(request,stopped),"examined_scope":{"enumerated":[],"read":[],"admitted":[],"unexamined":[],"excluded":[],"unexamined_count_known":false,"inventory_omissions":[]},"coverage":coverage,"observations":[],"direct_evidence":[],"interpretations":[],"canonical_registrations":[],"matching_records":{"counting_scope":"current_admitted_matching_records","established":0,"returned":0,"withheld":0},"response_completeness":{"state":"complete","qualifications_communicated":true,"closure_support_communicated":false},"currentness":{"components":[],"atomic_filesystem_snapshot":false,"continuous_currentness":false},"failures":[],"obligations":[],"search_conclusion":{"state":"not_observed_unknown","predicate":predicate(query,json!([]),"supported_written_mapping"),"evidence":[]},"response_omissions":[]})
}
pub(super) fn base_cost(query: &Query, roots: &[PathBuf]) -> Result<u64, Stop> {
    let mut size = 16384u64;
    for bytes in roots
        .iter()
        .map(|p| p.as_os_str().len())
        .chain(
            query
                .source_files
                .iter()
                .flatten()
                .map(|p| p.as_os_str().len()),
        )
        .chain(std::iter::once(query.selector.len()))
        .chain(query.within_path.iter().map(String::len))
    {
        size = size
            .checked_add((bytes as u64).checked_mul(16).ok_or(Stop::RetainedBytes)?)
            .ok_or(Stop::RetainedBytes)?;
    }
    Ok(size)
}
pub(super) fn validation(id: &str, state: super::super::Validation) -> Value {
    use super::super::Validation as V;
    let (state, reason) = match state {
        V::Current => ("current", None),
        V::Stale => ("stale", Some("source/support changed".into())),
        V::Incompatible => ("incompatible", Some("crossed evidence".into())),
        V::Missing => ("missing", None),
        V::Unvalidated(stop) => ("unvalidated", Some(format!("{stop:?}"))),
    };
    json!({"component":id,"state":state,"boundary":"serving_boundary","reason":reason})
}
fn invalid(message: &str) -> Failure {
    Failure::Integrity(message.into())
}
pub(super) fn validate(value: &Value) -> Result<(), Failure> {
    if value["contract_version"] != 2
        || value["query"]["mode"] != "interactive"
        || value["query"]["canonical_policy"] != "none"
        || value["canonical_registrations"] != json!([])
    {
        return Err(invalid("wrong evidence contract"));
    }
    let observations = value["observations"]
        .as_array()
        .ok_or_else(|| invalid("missing observations"))?;
    let evidence = value["direct_evidence"]
        .as_array()
        .ok_or_else(|| invalid("missing evidence"))?;
    let interpretations = value["interpretations"]
        .as_array()
        .ok_or_else(|| invalid("missing interpretations"))?;
    let mut ids = BTreeSet::new();
    let mut mapping = BTreeMap::new();
    for s in value["examined_scope"]["read"]
        .as_array()
        .into_iter()
        .flatten()
    {
        ids.insert(format!(
            "source:{}@{}",
            s["path"].as_str().unwrap_or_default(),
            s["sha256"].as_str().unwrap_or_default()
        ));
    }
    for s in value["examined_scope"]["admitted"]
        .as_array()
        .into_iter()
        .flatten()
    {
        ids.insert(format!(
            "source:{}@{}",
            s["path"].as_str().unwrap_or_default(),
            s["sha256"].as_str().unwrap_or_default()
        ));
    }
    for e in evidence {
        for s in e["support"].as_array().into_iter().flatten() {
            ids.insert(format!(
                "source:{}@{}",
                s["path"].as_str().unwrap_or_default(),
                s["sha256"].as_str().unwrap_or_default()
            ));
        }
    }
    for record in observations.iter().chain(evidence).chain(interpretations) {
        let id = record["id"]
            .as_str()
            .ok_or_else(|| invalid("missing component ID"))?;
        if !ids.insert(id.to_owned()) {
            return Err(invalid("duplicate component ID"));
        }
    }
    for o in observations {
        let id = o["id"].as_str().expect("checked id");
        mapping.insert(id, o);
        if o["claim"] != "written_source_mapping"
            || o["classification"] != "proven"
            || o["validation"]["state"] != "current"
            || o["source"] != o["claim_scope"]["source"]
            || o["object"] != o["claim_scope"]["object_span"]
            || o["mapping_pair"] != o["claim_scope"]["mapping_span"]
        {
            return Err(invalid("incompatible written claim"));
        }
        let mut roots = vec![value["query"]["workspace_root"].clone()];
        roots.extend(
            value["query"]["additional_roots"]
                .as_array()
                .ok_or_else(|| invalid("missing root scope"))?
                .iter()
                .cloned(),
        );
        if o["claim_scope"]["authorized_roots"] != json!(roots) {
            return Err(invalid("crossed authorization scope"));
        }
        let source_id = format!(
            "source:{}@{}",
            o["source"]["path"].as_str().unwrap_or_default(),
            o["source"]["sha256"].as_str().unwrap_or_default()
        );
        if !value["currentness"]["components"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|s| s["component"] == source_id && s["state"] == "current")
        {
            return Err(invalid("written claim lacks current source admission"));
        }
    }
    let mut endpoints = BTreeMap::new();
    for e in evidence {
        let o = mapping
            .get(e["source_claim"].as_str().unwrap_or_default())
            .ok_or_else(|| invalid("dangling source claim"))?;
        if e["claim_scope"] != o["claim_scope"]
            || e["authorized_roots"] != o["claim_scope"]["authorized_roots"]
            || (e["kind"] == "direct_value" && e["namespace"] != "value")
            || (e["kind"] == "angular_role" && e["namespace"] != "type")
        {
            return Err(invalid("crossed declaration evidence"));
        }
        endpoints.insert(e["id"].as_str().expect("id"), e);
        if e["classification"] == "proven"
            && (e["validation"]["state"] != "current"
                || !e["support"].as_array().into_iter().flatten().any(|s| {
                    s["path"] == e["target"]["source"] && s["sha256"] == e["target"]["source_hash"]
                }))
        {
            return Err(invalid("unbacked declaration endpoint"));
        }
    }
    for i in interpretations {
        let o = mapping
            .get(i["source_claim"].as_str().unwrap_or_default())
            .ok_or_else(|| invalid("dangling interpretation"))?;
        if i["claim_scope"] != o["claim_scope"]
            || i["claim"] != "written_angular_routes_mapping"
            || i["classification"] == "proven"
            || i["classification"] == "unsupported"
        {
            return Err(invalid("crossed interpretation"));
        }
        for (field, kind) in [
            ("value_evidence", "direct_value"),
            ("role_evidence", "angular_role"),
        ] {
            if let Some(id) = i[field].as_str() {
                let e = endpoints
                    .get(id)
                    .ok_or_else(|| invalid("dangling identity support"))?;
                if e["kind"] != kind || e["source_claim"] != i["source_claim"] {
                    return Err(invalid("crossed identity support"));
                }
            }
        }
        if i["classification"] == "strongly_supported"
            && (i["composition_compatible"] != true
                || i["applicability"] != "supported"
                || ["value_evidence", "role_evidence"].iter().any(|field| {
                    i[*field]
                        .as_str()
                        .and_then(|id| endpoints.get(id))
                        .is_none_or(|e| {
                            e["validation"]["state"] != "current" || e["target"].is_null()
                        })
                }))
        {
            return Err(invalid("unsupported strength upgrade"));
        }
    }
    for item in observations.iter().chain(evidence).chain(interpretations) {
        let support = item["supporting_evidence"]
            .as_array()
            .ok_or_else(|| invalid("missing support references"))?;
        if support.is_empty()
            || support
                .iter()
                .any(|x| !ids.contains(x.as_str().unwrap_or_default()))
        {
            return Err(invalid("dangling supporting evidence"));
        }
    }
    for status in value["currentness"]["components"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let id = status["component"]
            .as_str()
            .ok_or_else(|| invalid("missing currentness component"))?;
        if !(ids.contains(id)
            || (id.starts_with("attempt:")
                && status["state"] != "current"
                && value["failures"].as_array().into_iter().flatten().any(|f| {
                    f["affected"].as_array().is_some_and(|a| {
                        a.iter().any(|x| x == id)
                            && a.iter()
                                .any(|x| ids.contains(x.as_str().unwrap_or_default()))
                    })
                })))
        {
            return Err(invalid("dangling currentness component"));
        }
    }
    let counts = &value["matching_records"];
    let n = counts["established"]
        .as_u64()
        .ok_or_else(|| invalid("missing match count"))?;
    let returned = counts["returned"]
        .as_u64()
        .ok_or_else(|| invalid("missing returned count"))?;
    let withheld = counts["withheld"]
        .as_u64()
        .ok_or_else(|| invalid("missing withheld count"))?;
    if returned.checked_add(withheld) != Some(n) || returned != observations.len() as u64 {
        return Err(invalid("inconsistent match counts"));
    }
    let closed = value["coverage"]["supported_written_source"]["state"] == "closed";
    if closed && value["response_completeness"]["closure_support_communicated"] != true {
        return Err(invalid("uncommunicated closure"));
    }
    if value["search_conclusion"]["state"] == "absent_within_closed_predicate"
        && (!closed || n != 0)
    {
        return Err(invalid("unproved absence"));
    }
    if withheld > 0
        && returned == 0
        && value["search_conclusion"]["state"] != "observed_not_returned"
    {
        return Err(invalid("withheld positives presented as empty"));
    }
    Ok(())
}
